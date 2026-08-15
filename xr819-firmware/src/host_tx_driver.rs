//! Concurrent retained host-TX ownership with a single class-0 MAC executor.
//!
//! Each of the 30 vendor host contexts owns its original HIF request until its
//! confirmation is published. Pending contexts advance independently, while at
//! most one context may reserve or own the shared class-0 hardware runtime.

use crate::{hif, host_tx_diagnostics, host_tx_policy, tx, vendor_host_tx};

const HOST_CONTEXT_COUNT: usize = crate::host_tx_arena::HOST_CONTEXT_COUNT;
/// Contexts advanced per service pass.
///
/// At 4 (one hardware owner plus three software owners), queued contexts wait
/// many passes before publication. Cumulative iperf timing measures ~39.5 ms
/// admission-to-publication, but that is queue latency behind several frames,
/// not inverse throughput: the serialized publication-to-confirmation cycle is
/// ~8.4 ms. Raising this budget was neutral, so keep the bounded vendor-shaped
/// service loop until hardware concurrency is made ownership-safe.
#[cfg(not(feature = "wide-service-budget"))]
const SERVICE_BUDGET: usize = 4;
#[cfg(feature = "wide-service-budget")]
const SERVICE_BUDGET: usize = HOST_CONTEXT_COUNT;

pub struct HostTxDriver {
    states: [Option<HostTxState>; HOST_CONTEXT_COUNT],
    service_cursor: usize,
    next_confirmation_order: u32,
    #[cfg(feature = "phy-start-before-scheduler")]
    scheduler_phy_started_this_pass: bool,
}

enum HostTxState {
    Owned {
        retained: vendor_host_tx::RetainedHostTx,
        wait_diagnostic: u8,
        hardware: Option<HardwareOwner>,
    },
    Reserved {
        retained: vendor_host_tx::RetainedHostTx,
        reservation: vendor_host_tx::HostSchedulerReservation,
        wait_diagnostic: u8,
    },
    Confirming {
        owner: ConfirmationOwner,
        confirmation: HostTxConfirmation,
        completion_order: u32,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct HardwareOwner {
    pipe: u8,
    slot: u8,
    frame_node: u32,
}

impl HardwareOwner {
    const fn matches(self, owner_context: u32, completion: tx::HostClass0Completion) -> bool {
        host_tx_policy::completion_matches_owner(
            owner_context,
            self.frame_node,
            self.pipe,
            self.slot,
            completion.context,
            completion.frame_node,
            completion.pipe,
            completion.slot,
        )
    }
}

enum ConfirmationOwner {
    Retained(vendor_host_tx::RetainedHostTx),
    Release(hif::RequestReleaseToken),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HostTxConfirmation {
    pub packet_id: u32,
    pub context: u32,
    pub status: u32,
    pub tx_rate: u8,
    pub ack_failures: u8,
    pub rate_try: [u32; 3],
}

impl HostTxDriver {
    pub const fn new() -> Self {
        Self {
            states: [const { None }; HOST_CONTEXT_COUNT],
            service_cursor: 0,
            next_confirmation_order: 0,
            #[cfg(feature = "phy-start-before-scheduler")]
            scheduler_phy_started_this_pass: false,
        }
    }

    /// Management and class-0 share one MAC executor. Pending software owners
    /// do not block management; a scheduler reservation or scheduled class-0
    /// frame does.
    pub fn management_runtime_available(&self) -> bool {
        !self.states.iter().any(|state| match state {
            Some(HostTxState::Reserved { .. }) => true,
            Some(HostTxState::Owned { retained, .. }) => {
                retained.phase() == vendor_host_tx::HostTxPhase::Scheduled
            }
            Some(HostTxState::Confirming { .. }) | None => false,
        })
    }

    /// Admit, classify, encrypt, and insert one ordinary host request into the
    /// vendor pending list. The hardware context index selects the arena slot.
    ///
    /// # Safety
    /// The caller must serialize host-context, pending-list, and HIF mutation.
    pub unsafe fn admit(
        &mut self,
        buffer: hif::RequestBuffer,
        interface: u8,
        transport: &mut hif::Transport,
    ) -> bool {
        let mut retained = match unsafe { vendor_host_tx::admit_host_tx(buffer, interface) } {
            Ok(retained) => retained,
            Err((buffer, _)) => {
                transport.release_request(buffer.into_release());
                return false;
            }
        };
        let index = retained.context().index();
        if self.states[index].is_some() {
            unsafe {
                host_tx_diagnostics::trace(0x4854_00e4, retained.context().raw(), index as u32);
            }
            let release = unsafe { retained.abort() };
            transport.release_request(release);
            return false;
        }
        if unsafe { vendor_host_tx::classify_and_encrypt(&mut retained) }.is_err() {
            unsafe {
                host_tx_diagnostics::trace(0x4854_00e2, retained.context().raw(), 0);
            }
            let release = unsafe { retained.abort() };
            transport.release_request(release);
            return false;
        }
        if unsafe { vendor_host_tx::enqueue_post_crypto(&mut retained) }.is_err() {
            unsafe {
                host_tx_diagnostics::trace(0x4854_00e3, retained.context().raw(), 0);
            }
            let release = unsafe { retained.abort() };
            transport.release_request(release);
            return false;
        }
        unsafe {
            host_tx_diagnostics::trace(
                0x4854_0008,
                retained.context().raw(),
                retained.phase() as u32,
            );
        }
        self.states[index] = Some(HostTxState::Owned {
            retained,
            wait_diagnostic: 0,
            hardware: None,
        });
        true
    }

    /// Advance a bounded subset of contexts. A reserved/scheduled context is
    /// always serviced first; remaining budget walks software-owned contexts
    /// round-robin.
    ///
    /// # Safety
    /// This driver must be the sole class-0 owner of pending/PAS/scheduler and
    /// MAC-event mutation.
    pub unsafe fn service(
        &mut self,
        events: &mut tx::MacEventQueue,
        allow_hardware_publication: bool,
        allow_debug_event: bool,
    ) -> Option<(u32, u32)> {
        let mut diagnostic = None;
        let mut budget = SERVICE_BUDGET;
        #[cfg(feature = "phy-start-before-scheduler")]
        {
            self.scheduler_phy_started_this_pass = false;
        }

        // Service the shared MAC executor once, then route every completion
        // drained in that pass by its registered pipe/slot/frame-node owner.
        // A successful batch may enqueue all of its slots before returning.
        if self.hardware_runtime_owner().is_some() {
            let mut completion = unsafe { tx::service_host_class0_runtime(events, 32) };
            while let Some(completed) = completion {
                self.route_hardware_completion(completed);
                completion = unsafe { tx::take_host_class0_completion() };
            }
        }

        if let Some(index) = self.hardware_runtime_owner() {
            diagnostic = unsafe {
                self.service_index(index, events, allow_hardware_publication, allow_debug_event)
            };
            budget -= 1;
        }

        while budget != 0 {
            let Some(index) = self.next_software_owner() else {
                break;
            };
            let event = unsafe {
                self.service_index(
                    index,
                    events,
                    allow_hardware_publication && !cfg!(feature = "tx-pipelining"),
                    allow_debug_event && diagnostic.is_none(),
                )
            };
            if diagnostic.is_none() {
                diagnostic = event;
            }
            budget -= 1;
        }
        // Vendor pending processing fills PAS queues before the scheduler
        // reserves any pipe. The depth-1 path historically combined those
        // phases and is unchanged. Batching must separate them: after the
        // software pass above has advanced several contexts into PAS, run one
        // bounded scheduler pass that stages up to the configured depth, then
        // arm once.
        #[cfg(all(target_arch = "arm", feature = "tx-pipelining"))]
        if allow_hardware_publication && self.hardware_runtime_owner().is_none() {
            let depth = tx::tx_batch_depth();
            let (_, _, pas_ready, _) = self.batch_phase_counts();
            let mut staged = unsafe { tx::staged_slots_total() };
            while staged < depth {
                let Some(index) = self.next_pas_owner() else {
                    break;
                };
                let event = unsafe {
                    self.service_index(
                        index,
                        events,
                        true,
                        allow_debug_event && diagnostic.is_none(),
                    )
                };
                if diagnostic.is_none() {
                    diagnostic = event;
                }
                let staged_now = unsafe { tx::staged_slots_total() };
                if staged_now == staged {
                    break;
                }
                staged = staged_now;
            }
            unsafe { tx::note_batch_scheduler_capacity(pas_ready, staged) };
            if staged != 0 {
                unsafe { tx::arm_staged_pipes() };
            }
        }
        diagnostic
    }

    fn hardware_runtime_owner(&self) -> Option<usize> {
        self.states.iter().position(|state| {
            matches!(state, Some(HostTxState::Reserved { .. }))
                || matches!(
                    state,
                    Some(HostTxState::Owned { retained, .. })
                        if retained.phase() == vendor_host_tx::HostTxPhase::Scheduled
                )
        })
    }

    fn next_software_owner(&mut self) -> Option<usize> {
        for _ in 0..HOST_CONTEXT_COUNT {
            let index = self.service_cursor;
            self.service_cursor = (self.service_cursor + 1) % HOST_CONTEXT_COUNT;
            if matches!(
                &self.states[index],
                Some(HostTxState::Owned { retained, .. })
                    if retained.phase() != vendor_host_tx::HostTxPhase::Scheduled
            ) {
                return Some(index);
            }
        }
        None
    }

    #[cfg(feature = "tx-pipelining")]
    fn batch_phase_counts(&self) -> (u8, u8, u8, u8) {
        self.states.iter().fold((0, 0, 0, 0), |counts, state| {
            let Some(HostTxState::Owned { retained, .. }) = state else {
                return counts;
            };
            let (post_crypto, pending, pas, total) = counts;
            match retained.phase() {
                vendor_host_tx::HostTxPhase::PostCryptoQueued => {
                    (post_crypto.saturating_add(1), pending, pas, total.saturating_add(1))
                }
                vendor_host_tx::HostTxPhase::PendingEligible => {
                    (post_crypto, pending.saturating_add(1), pas, total.saturating_add(1))
                }
                vendor_host_tx::HostTxPhase::PasQueued => {
                    (post_crypto, pending, pas.saturating_add(1), total.saturating_add(1))
                }
                _ => (post_crypto, pending, pas, total),
            }
        })
    }

    #[cfg(feature = "tx-pipelining")]
    fn next_pas_owner(&self) -> Option<usize> {
        self.states.iter().position(|state| {
            matches!(
                state,
                Some(HostTxState::Owned { retained, .. })
                    if retained.phase() == vendor_host_tx::HostTxPhase::PasQueued
            )
        })
    }

    fn route_hardware_completion(&mut self, completion: tx::HostClass0Completion) {
        let owner = self.states.iter().position(|state| {
            matches!(
                state,
                Some(HostTxState::Owned {
                    retained,
                    hardware: Some(hardware),
                    ..
                }) if retained.phase() == vendor_host_tx::HostTxPhase::Scheduled
                    && hardware.matches(retained.context().raw(), completion)
            )
        });
        let Some(index) = owner else {
            unsafe {
                host_tx_diagnostics::trace(
                    0x4854_3f00,
                    completion.context,
                    u32::from(completion.pipe) | (u32::from(completion.slot) << 8),
                );
            }
            return;
        };
        let Some(HostTxState::Owned { mut retained, .. }) = self.states[index].take() else {
            return;
        };
        unsafe {
            host_tx_diagnostics::capture_completion(
                completion.context,
                completion.status,
                completion.ack_failures,
            );
            host_tx_diagnostics::capture_completion_identity(
                retained.packet_id(),
                completion.context,
                completion.status,
                completion.ack_failures,
            );
        }
        let _ = retained.transition(vendor_host_tx::HostTxPhase::Completing);
        let completion_order = self.allocate_confirmation_order();
        self.states[index] = Some(Self::confirmation_state(
            retained,
            tx::wsm_status_from_internal(completion.status),
            completion.ack_failures,
            completion_order,
        ));
    }

    unsafe fn service_index(
        &mut self,
        index: usize,
        _events: &mut tx::MacEventQueue,
        allow_hardware_publication: bool,
        allow_debug_event: bool,
    ) -> Option<(u32, u32)> {
        let Some(state) = self.states[index].take() else {
            return None;
        };
        let mut event = None;
        let next = match state {
            HostTxState::Confirming {
                owner,
                confirmation,
                completion_order,
            } => HostTxState::Confirming {
                owner,
                confirmation,
                completion_order,
            },
            HostTxState::Reserved {
                mut retained,
                reservation,
                wait_diagnostic,
            } => {
                let pipe = reservation.pipe();
                let slot = reservation.slot();
                match unsafe { reservation.publish(&mut retained) } {
                    Ok(()) => {
                        unsafe {
                            host_tx_diagnostics::trace(
                                0x4854_7000,
                                retained.context().raw()
                                    | (u32::from(pipe) << 24)
                                    | (u32::from(slot) << 28),
                                0,
                            );
                            host_tx_diagnostics::capture_publication_identity(
                                retained.packet_id(),
                                retained.context().raw(),
                                pipe,
                                slot,
                            );
                        }
                        let hardware = HardwareOwner {
                            pipe,
                            slot,
                            frame_node: retained.context().raw().wrapping_add(0x54),
                        };
                        HostTxState::Owned {
                            retained,
                            wait_diagnostic: 3,
                            hardware: Some(hardware),
                        }
                    }
                    Err((reservation, error)) => {
                        unsafe {
                            host_tx_diagnostics::trace(
                                0x4854_7100 | error as u32,
                                retained.context().raw()
                                    | (u32::from(pipe) << 24)
                                    | (u32::from(slot) << 28),
                                0,
                            );
                        }
                        HostTxState::Reserved {
                            retained,
                            reservation,
                            wait_diagnostic,
                        }
                    }
                }
            }
            HostTxState::Owned {
                mut retained,
                mut wait_diagnostic,
                mut hardware,
            } => {
                unsafe {
                    host_tx_diagnostics::trace(
                        0x4854_1000 | u32::from(retained.phase() as u8),
                        retained.context().raw(),
                        0,
                    );
                }
                if matches!(
                    retained.phase(),
                    vendor_host_tx::HostTxPhase::PostCryptoQueued
                        | vendor_host_tx::HostTxPhase::PendingEligible
                ) {
                    match unsafe { vendor_host_tx::service_pending(&mut retained) } {
                        Ok(vendor_host_tx::PendingServiceReport::Complete(status)) => {
                            let _ = retained.transition(vendor_host_tx::HostTxPhase::Completing);
                            let completion_order = self.allocate_confirmation_order();
                            self.states[index] = Some(Self::confirmation_state(
                                retained,
                                tx::wsm_status_from_internal(status),
                                0,
                                completion_order,
                            ));
                            return None;
                        }
                        Ok(vendor_host_tx::PendingServiceReport::PasQueued) => {
                            // Boundary inside admit-to-publish: pending gating
                            // ends here, scheduler reservation begins.
                            #[cfg(target_arch = "arm")]
                            unsafe {
                                tx::note_stage(3)
                            };
                            wait_diagnostic = 0;
                        }
                        Ok(vendor_host_tx::PendingServiceReport::LeaveQueued) => {
                            if cfg!(feature = "vendor-host-tx-diagnostics")
                                && allow_debug_event
                                && wait_diagnostic == 0
                            {
                                let diagnostic =
                                    unsafe { vendor_host_tx::pending_live_diagnostic(&retained) };
                                event = Some((
                                    0x4854_6000
                                        | u32::from(diagnostic.vif_state)
                                        | (u32::from(diagnostic.pipe_allowed) << 8)
                                        | ((diagnostic.global & 0xff) << 16),
                                    u32::from(diagnostic.active_mask)
                                        | (u32::from(diagnostic.effective_mask) << 16),
                                ));
                                wait_diagnostic = 1;
                            }
                        }
                        Err(error) => unsafe {
                            host_tx_diagnostics::trace(
                                0x4854_1f00 | u32::from(error.diagnostic_code()),
                                retained.context().raw(),
                                u32::from(retained.phase() as u8),
                            );
                        },
                    }
                }

                // Depth 1 keeps the historic gate: one frame in flight at a
                // time. Beyond that, a further frame may be staged while the
                // pipe is still unarmed; `reserve_non_aggregate_scheduler`
                // itself refuses an armed pipe, so this cannot append to a
                // running batch.
                let staged_headroom = {
                    let depth = tx::tx_batch_depth();
                    depth > 1 && unsafe { tx::staged_slots_total() } < depth
                };
                if retained.phase() == vendor_host_tx::HostTxPhase::PasQueued
                    && allow_hardware_publication
                    && (self.hardware_runtime_owner().is_none() || staged_headroom)
                {
                    #[cfg(feature = "phy-start-before-scheduler")]
                    if !self.scheduler_phy_started_this_pass {
                        // Vendor calls command 1 once after finding a non-empty
                        // PAS queue and before scanning/building that scheduler
                        // pass (`txq_build_aggregate_lists`, 0xa2c0). Do not
                        // restart it for another reservation attempt in the
                        // same pass.
                        let _ = unsafe { tx::start_phy_operation_1() };
                        self.scheduler_phy_started_this_pass = true;
                    }
                    match unsafe { vendor_host_tx::reserve_non_aggregate_scheduler(&mut retained) }
                    {
                        Ok(reservation) => {
                            let pipe = reservation.pipe();
                            let slot = reservation.slot();
                            // Stage without arming when batching; the arm for
                            // every staged slot happens once, after servicing.
                            let batch = if tx::tx_batch_depth() > 1 {
                                if unsafe { tx::staged_slots(pipe) } == 0 {
                                    tx::BatchPosition::First
                                } else {
                                    tx::BatchPosition::Middle
                                }
                            } else {
                                tx::BatchPosition::Only
                            };
                            match unsafe { reservation.publish_in_batch(&mut retained, batch) } {
                                Ok(()) => {
                                    if !batch.arms() {
                                        unsafe { tx::note_staged_slot(pipe) };
                                    }
                                    unsafe {
                                        host_tx_diagnostics::trace(
                                            0x4854_7000,
                                            retained.context().raw()
                                                | (u32::from(pipe) << 24)
                                                | (u32::from(slot) << 28),
                                            0,
                                        );
                                        host_tx_diagnostics::capture_publication_identity(
                                            retained.packet_id(),
                                            retained.context().raw(),
                                            pipe,
                                            slot,
                                        );
                                    }
                                    hardware = Some(HardwareOwner {
                                        pipe,
                                        slot,
                                        frame_node: retained.context().raw().wrapping_add(0x54),
                                    });
                                    self.states[index] = Some(HostTxState::Owned {
                                        retained,
                                        wait_diagnostic: 3,
                                        hardware,
                                    });
                                }
                                Err((reservation, error)) => {
                                    unsafe {
                                        host_tx_diagnostics::trace(
                                            0x4854_7100 | error as u32,
                                            retained.context().raw()
                                                | (u32::from(pipe) << 24)
                                                | (u32::from(slot) << 28),
                                            0,
                                        );
                                    }
                                    self.states[index] = Some(HostTxState::Reserved {
                                        retained,
                                        reservation,
                                        wait_diagnostic: 0,
                                    });
                                }
                            }
                            return event;
                        }
                        Err(vendor_host_tx::SchedulerReserveError::Expired) => {
                            if unsafe { vendor_host_tx::reject_unscheduled_pas(&mut retained) }
                                .is_ok()
                            {
                                let _ =
                                    retained.transition(vendor_host_tx::HostTxPhase::Completing);
                                let completion_order = self.allocate_confirmation_order();
                                self.states[index] = Some(Self::confirmation_state(
                                    retained,
                                    tx::wsm_status_from_internal(10),
                                    0,
                                    completion_order,
                                ));
                                return event;
                            }
                        }
                        Err(error) => unsafe {
                            host_tx_diagnostics::trace(
                                0x4854_2f00 | u32::from(error.diagnostic_code()),
                                retained.context().raw(),
                                0,
                            );
                        },
                    }
                }

                HostTxState::Owned {
                    retained,
                    wait_diagnostic,
                    hardware,
                }
            }
        };
        self.states[index] = Some(next);
        event
    }

    fn allocate_confirmation_order(&mut self) -> u32 {
        let order = self.next_confirmation_order;
        self.next_confirmation_order = order.wrapping_add(1);
        order
    }

    // Encoding lives in `host_tx_policy::rate_try_for_single_rate`, which is
    // host-testable; see there for why zeros are not harmless.
    fn confirmation_state(
        retained: vendor_host_tx::RetainedHostTx,
        status: u32,
        ack_failures: u8,
        completion_order: u32,
    ) -> HostTxState {
        let context = retained.context().raw();
        let tx_rate = unsafe { (context.wrapping_add(0x63) as *const u8).read_volatile() };
        unsafe {
            host_tx_diagnostics::capture_retry_feedback(context, status, tx_rate, ack_failures);
        }
        // Admission-to-confirmation latency. `context + 0x40` is the vendor
        // submission timestamp the scheduler already compares against.
        #[cfg(all(
            feature = "class0-lifecycle-counters",
            not(feature = "stage-latency-sums"),
            target_arch = "arm"
        ))]
        unsafe {
            let submitted = (context.wrapping_add(0x40) as *const u32).read_volatile();
            let elapsed = vendor_host_tx::vendor_timer_now().wrapping_sub(submitted);
            // A wrapped or unset timestamp would swamp the maximum.
            if elapsed < 10_000_000 {
                // Split the total into driver time, MAC pickup, airtime, and
                // confirmation time. Only the third is the medium; time in the
                // first or last is our own service loop, which deeper TX
                // pipelining would not help.
                let published_at = tx::stage_timestamp(0);
                let started_at = tx::stage_timestamp(1);
                let completed_at = tx::stage_timestamp(2);
                let span = |from: u32, to: u32| to.wrapping_sub(from).min(0xffff);
                host_tx_diagnostics::observe(
                    host_tx_diagnostics::counter::STAGE_ADMIT_PUBLISH,
                    span(submitted, published_at) | (span(published_at, started_at) << 16),
                );
                host_tx_diagnostics::observe(
                    host_tx_diagnostics::counter::STAGE_PENDING_SPLIT,
                    span(submitted, tx::stage_timestamp(3))
                        | (span(tx::stage_timestamp(3), published_at) << 16),
                );
                host_tx_diagnostics::observe(
                    host_tx_diagnostics::counter::STAGE_START_CONFIRM,
                    span(started_at, completed_at)
                        | (span(completed_at, vendor_host_tx::vendor_timer_now()) << 16),
                );
                let worst = host_tx_diagnostics::counters_snapshot()
                    [host_tx_diagnostics::counter::LATENCY_MAX];
                if elapsed > worst {
                    host_tx_diagnostics::observe(
                        host_tx_diagnostics::counter::LATENCY_MAX,
                        elapsed,
                    );
                }
            }
        }
        #[cfg(all(feature = "stage-latency-sums", target_arch = "arm"))]
        unsafe {
            let submitted = (context.wrapping_add(0x40) as *const u32).read_volatile();
            let published_at = tx::stage_timestamp(0);
            let started_at = tx::stage_timestamp(1);
            let completed_at = tx::stage_timestamp(2);
            let confirmed_at = vendor_host_tx::vendor_timer_now();
            let spans = [
                published_at.wrapping_sub(submitted),
                started_at.wrapping_sub(published_at),
                completed_at.wrapping_sub(started_at),
                confirmed_at.wrapping_sub(completed_at),
            ];
            let elapsed = confirmed_at.wrapping_sub(submitted);

            // Each boundary belongs to the sole hardware-owned frame at depth
            // one. Reject unset/stale boundaries rather than letting one bad
            // sample saturate a whole run's sums.
            if elapsed < 10_000_000 && spans.into_iter().all(|span| span < 1_000_000) {
                host_tx_diagnostics::accumulate(
                    host_tx_diagnostics::counter::LATENCY_SUM_ADMIT_PUBLISH,
                    spans[0],
                );
                host_tx_diagnostics::accumulate(
                    host_tx_diagnostics::counter::LATENCY_SUM_PUBLISH_START,
                    spans[1],
                );
                host_tx_diagnostics::accumulate(
                    host_tx_diagnostics::counter::LATENCY_SUM_START_COMPLETE,
                    spans[2],
                );
                host_tx_diagnostics::accumulate(
                    host_tx_diagnostics::counter::LATENCY_SUM_COMPLETE_CONFIRM,
                    spans[3],
                );
                host_tx_diagnostics::bump(
                    host_tx_diagnostics::counter::LATENCY_SAMPLE_COUNT,
                );
            }
        }
        HostTxState::Confirming {
            confirmation: HostTxConfirmation {
                packet_id: retained.packet_id(),
                context,
                status,
                tx_rate,
                ack_failures,
                rate_try: {
                    let reported = unsafe {
                        [
                            (context.wrapping_add(0x28) as *const u32).read_volatile(),
                            (context.wrapping_add(0x2c) as *const u32).read_volatile(),
                            (context.wrapping_add(0x30) as *const u32).read_volatile(),
                        ]
                    };
                    if reported == [0; 3] {
                        host_tx_policy::rate_try_for_single_rate(tx_rate, ack_failures)
                    } else {
                        reported
                    }
                },
            },
            owner: ConfirmationOwner::Retained(retained),
            completion_order,
        }
    }

    fn confirmation_index(&self) -> Option<usize> {
        self.states
            .iter()
            .enumerate()
            .filter_map(|(index, state)| match state {
                Some(HostTxState::Confirming {
                    completion_order, ..
                }) => Some((index, *completion_order)),
                _ => None,
            })
            .min_by_key(|(_, completion_order)| *completion_order)
            .map(|(index, _)| index)
    }

    /// Publish confirmations in the order their owners reached completion.
    /// Context identity resolves the hardware owner; packet-id magnitude has
    /// no ordering meaning and may wrap independently.
    pub fn confirmation(&self) -> Option<HostTxConfirmation> {
        let index = self.confirmation_index()?;
        match self.states[index] {
            Some(HostTxState::Confirming { confirmation, .. }) => Some(confirmation),
            _ => None,
        }
    }

    /// Finish the same first confirmation returned by `confirmation()` only
    /// after it has been admitted to the HIF software output queue.
    ///
    /// # Safety
    /// Completion accounting must already have removed hardware ownership.
    pub unsafe fn finish_confirmation(&mut self) -> Option<hif::RequestReleaseToken> {
        let index = self.confirmation_index()?;
        let HostTxState::Confirming { owner, .. } = self.states[index].take()? else {
            return None;
        };
        unsafe { host_tx_diagnostics::bump(host_tx_diagnostics::counter::CONFIRMED) };
        Some(match owner {
            ConfirmationOwner::Retained(retained) => unsafe { retained.finish() },
            ConfirmationOwner::Release(release) => release,
        })
    }

    /// Convert every reversible owner into a failure confirmation. Scheduled
    /// hardware ownership remains until normal completion.
    ///
    /// # Safety
    /// The caller must serialize scheduler, pending-list, and HIF mutation.
    pub unsafe fn reset(&mut self) {
        for index in 0..HOST_CONTEXT_COUNT {
            let Some(state) = self.states[index].take() else {
                continue;
            };
            self.states[index] = Some(match state {
                state @ HostTxState::Confirming { .. } => state,
                HostTxState::Reserved {
                    mut retained,
                    reservation,
                    ..
                } => {
                    let _ = unsafe { reservation.cancel(&mut retained) };
                    let completion_order = self.allocate_confirmation_order();
                    unsafe { Self::cancelled_confirmation(retained, completion_order) }
                }
                HostTxState::Owned { retained, .. }
                    if retained.phase() != vendor_host_tx::HostTxPhase::Scheduled =>
                {
                    let completion_order = self.allocate_confirmation_order();
                    unsafe { Self::cancelled_confirmation(retained, completion_order) }
                }
                state @ HostTxState::Owned { .. } => state,
            });
        }
    }

    unsafe fn cancelled_confirmation(
        retained: vendor_host_tx::RetainedHostTx,
        completion_order: u32,
    ) -> HostTxState {
        let context = retained.context().raw();
        let packet_id = retained.packet_id();
        let tx_rate = unsafe { (context.wrapping_add(0x63) as *const u8).read_volatile() };
        let rate_try = unsafe {
            [
                (context.wrapping_add(0x28) as *const u32).read_volatile(),
                (context.wrapping_add(0x2c) as *const u32).read_volatile(),
                (context.wrapping_add(0x30) as *const u32).read_volatile(),
            ]
        };
        let release = unsafe {
            retained
                .cancel_before_pas()
                .expect("reversible host TX cancellation")
        };
        HostTxState::Confirming {
            owner: ConfirmationOwner::Release(release),
            confirmation: HostTxConfirmation {
                packet_id,
                context,
                status: 1,
                tx_rate,
                ack_failures: 0,
                rate_try,
            },
            completion_order,
        }
    }
}

impl Default for HostTxDriver {
    fn default() -> Self {
        Self::new()
    }
}
