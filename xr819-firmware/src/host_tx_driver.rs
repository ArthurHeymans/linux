//! Concurrent retained host-TX ownership with a pipe-indexed class-0 MAC executor.
//!
//! Each of the 30 host contexts owns its original HIF request until the inherited
//! confirmation handoff point. The current HIF transport then returns request
//! credit before it enqueues the separately copied confirmation; this module does
//! not redesign that parent ordering. Pending contexts advance independently.
//! Retained hardware owners are discovered and serviced independently per MAC
//! pipe; publication admits at most one new batch per pass and never targets a
//! pipe that still has a retained runtime owner.

use crate::{hif, host_tx_diagnostics, host_tx_policy, tx, vendor_host_tx};

const HOST_CONTEXT_COUNT: usize = crate::host_tx_arena::HOST_CONTEXT_COUNT;
/// Contexts advanced per service pass.
///
/// At 4, exact hardware slot owners consume the front of the budget and the
/// remainder advances software owners. Queued contexts can still wait many
/// passes before publication. Cumulative iperf timing measures ~39.5 ms
/// admission-to-publication, but that is queue latency behind several frames,
/// not inverse throughput: the serialized publication-to-confirmation cycle is
/// ~8.4 ms. Raising this budget was neutral, so keep the bounded vendor-shaped
/// service loop until hardware concurrency is made ownership-safe.
const SERVICE_BUDGET: usize = 4;

pub struct HostTxDriver {
    states: [Option<HostTxState>; HOST_CONTEXT_COUNT],
    service_cursor: usize,
    hardware_service_cursor: u8,
    next_confirmation_order: u32,
    scheduler_phy_started_this_pass: bool,
    scheduler_single_wait: u8,
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
    #[cfg(feature = "experimental-aggregate-rate-feedback")]
    aggregate_head: u32,
    #[cfg(feature = "experimental-aggregate-rate-feedback")]
    aggregate_len: u8,
}

#[cfg(all(target_arch = "arm", feature = "experimental-four-slot-ordinary"))]
struct ReservedBatchMember {
    index: usize,
    retained: vendor_host_tx::RetainedHostTx,
    wait_diagnostic: u8,
    reservation: vendor_host_tx::HostSchedulerReservation,
    frame_node: u32,
}

impl HardwareOwner {
    const fn single(pipe: u8, slot: u8, frame_node: u32) -> Self {
        Self {
            pipe,
            slot,
            frame_node,
            #[cfg(feature = "experimental-aggregate-rate-feedback")]
            aggregate_head: frame_node,
            #[cfg(feature = "experimental-aggregate-rate-feedback")]
            aggregate_len: 1,
        }
    }

    #[cfg(feature = "experimental-aggregate-rate-feedback")]
    const fn aggregate(
        pipe: u8,
        slot: u8,
        frame_node: u32,
        aggregate_head: u32,
        aggregate_len: u8,
    ) -> Self {
        Self {
            pipe,
            slot,
            frame_node,
            aggregate_head,
            aggregate_len,
        }
    }

    #[cfg(not(feature = "experimental-aggregate-rate-feedback"))]
    const fn aggregate(
        pipe: u8,
        slot: u8,
        frame_node: u32,
        _aggregate_head: u32,
        _aggregate_len: u8,
    ) -> Self {
        Self {
            pipe,
            slot,
            frame_node,
        }
    }

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
    pub flags: u16,
    pub rate_try: [u32; 3],
    #[cfg(feature = "experimental-aggregate-rate-feedback")]
    aggregate_head: u32,
    #[cfg(feature = "experimental-aggregate-rate-feedback")]
    aggregate_len: u8,
}

impl HostTxDriver {
    pub const fn new() -> Self {
        Self {
            states: [const { None }; HOST_CONTEXT_COUNT],
            service_cursor: 0,
            hardware_service_cursor: 0,
            next_confirmation_order: 0,
            scheduler_phy_started_this_pass: false,
            scheduler_single_wait: 0,
        }
    }

    /// Management and class-0 share one MAC executor. Every retained class-0
    /// owner blocks admission of a new management publication; an already
    /// active management owner is serviced separately until it completes.
    pub fn management_runtime_available(&self) -> bool {
        self.states.iter().all(Option::is_none)
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
        mac_domain: &mut crate::mac_domain::MacDomain,
        allow_hardware_publication: bool,
        allow_debug_event: bool,
    ) -> Option<(u32, u32)> {
        let mut diagnostic = None;
        let mut budget = SERVICE_BUDGET;
        self.scheduler_phy_started_this_pass = false;

        // Service the shared MAC executor once, then route every completion
        // drained in that pass by its registered pipe/slot/frame-node owner.
        // A successful batch may enqueue all of its slots before returning.
        let Some(owners) = self.hardware_runtime_owners() else {
            crate::halt_always!();
        };
        if !owners.is_empty() {
            let mut completion = unsafe { tx::service_host_class0_runtime(events, 32) };
            while let Some(completed) = completion {
                #[cfg(feature = "experimental-member-requeue")]
                if tx::host_class0_is_requeue(completed) {
                    self.route_hardware_requeue(completed, mac_domain);
                } else {
                    self.route_hardware_completion(completed);
                }
                #[cfg(not(feature = "experimental-member-requeue"))]
                self.route_hardware_completion(completed);
                completion = unsafe { tx::take_host_class0_completion() };
            }
        }

        let Some(owners) = self.hardware_runtime_owners() else {
            crate::halt_always!();
        };
        let mut eligible = owners.slot_mask();
        while budget != 0 {
            let Some((slot_index, index)) =
                owners.next_slot_owner(eligible, self.hardware_service_cursor)
            else {
                break;
            };
            eligible &= !(1_u16 << slot_index);
            self.hardware_service_cursor = slot_index.wrapping_add(1) & 15;
            let event = unsafe {
                self.service_index(
                    index,
                    events,
                    mac_domain,
                    allow_hardware_publication,
                    allow_debug_event && diagnostic.is_none(),
                )
            };
            if diagnostic.is_none() {
                diagnostic = event;
            }
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
                    mac_domain,
                    false,
                    allow_debug_event && diagnostic.is_none(),
                )
            };
            if diagnostic.is_none() {
                diagnostic = event;
            }
            budget -= 1;
        }
        let Some(owners) = self.hardware_runtime_owners() else {
            crate::halt_always!();
        };
        if allow_hardware_publication {
            unsafe { self.publish_ready_batch(mac_domain, owners) };
        }
        diagnostic
    }

    fn hardware_runtime_owners(&self) -> Option<host_tx_policy::Class0RuntimeOwners> {
        let mut owners = host_tx_policy::Class0RuntimeOwners::new();
        for (index, state) in self.states.iter().enumerate() {
            let identity = match state {
                Some(HostTxState::Reserved { reservation, .. }) => {
                    Some((reservation.pipe(), reservation.slot()))
                }
                Some(HostTxState::Owned {
                    retained,
                    hardware,
                    ..
                }) if retained.phase() == vendor_host_tx::HostTxPhase::Scheduled => {
                    let Some(hardware) = hardware else {
                        return None;
                    };
                    Some((hardware.pipe, hardware.slot))
                }
                _ => None,
            };
            if let Some((pipe, slot)) = identity
                && !owners.observe(index, pipe, slot)
            {
                return None;
            }
        }
        Some(owners)
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

    fn route_hardware_completion(&mut self, completion: tx::HostClass0Completion) {
        let Some(owners) = self.hardware_runtime_owners() else {
            crate::halt_always!();
        };
        if owners
            .slot_owner(completion.pipe, completion.slot)
            .is_none()
        {
            unsafe {
                host_tx_diagnostics::trace(
                    0x4854_3f00,
                    completion.context,
                    u32::from(completion.pipe) | (u32::from(completion.slot) << 8),
                );
            }
            return;
        }
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
        let Some(HostTxState::Owned {
            mut retained,
            hardware: Some(hardware),
            ..
        }) = self.states[index].take()
        else {
            return;
        };
        unsafe {
            host_tx_diagnostics::capture_completion(
                retained.context(),
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
            Some(hardware),
            completion_order,
        ));
    }

    #[cfg(feature = "experimental-member-requeue")]
    fn route_hardware_requeue(
        &mut self,
        requeue: tx::HostClass0Completion,
        mac_domain: &mut crate::mac_domain::MacDomain,
    ) {
        let Some(context) = crate::dtcm::host_context_from_raw(requeue.context) else {
            crate::halt_always!();
        };
        let index = context.index();
        if !matches!(
            self.states[index],
            Some(HostTxState::Owned {
                ref retained,
                hardware: Some(hardware),
                ..
            }) if retained.phase() == vendor_host_tx::HostTxPhase::Scheduled
                && hardware.matches(retained.context().raw(), requeue)
        ) {
            crate::halt_always!();
        }
        let Some(HostTxState::Owned { mut retained, .. }) = self.states[index].take() else {
            crate::halt_always!();
        };
        let mut guard = mac_domain.enter();
        if unsafe { vendor_host_tx::requeue_scheduled_retry(&mut guard, &mut retained) }.is_err() {
            crate::halt_always!();
        }
        self.states[index] = Some(HostTxState::Owned {
            retained,
            wait_diagnostic: 0,
            hardware: None,
        });
    }

    #[cfg(all(
        target_arch = "arm",
        feature = "experimental-four-slot-ordinary",
        feature = "experimental-depth-four-ampdu"
    ))]
    #[inline(never)]
    unsafe fn publish_depth_four_reserved(
        &mut self,
        guard: &mut crate::mac_domain::MacDomainGuard<'_>,
        members: &mut [Option<ReservedBatchMember>; 4],
        member_count: usize,
    ) {
        let mut retained = [core::ptr::null_mut(); 4];
        let mut reservations = [core::ptr::null(); 4];
        for position in 0..member_count {
            let Some(member) = members[position].as_mut() else {
                crate::halt_always!();
            };
            retained[position] = &mut member.retained;
            reservations[position] = &member.reservation;
        }
        match unsafe {
            vendor_host_tx::publish_planned_ampdu(retained, reservations, member_count)
        } {
            Ok((aggregate_pipe, aggregate_slot)) => {
                let Some(aggregate_head) = members[0].as_ref().map(|member| member.frame_node) else {
                    crate::halt_always!();
                };
                for position in 0..member_count {
                    let Some(member) = members[position].take() else {
                        crate::halt_always!();
                    };
                    self.states[member.index] = Some(HostTxState::Owned {
                        retained: member.retained,
                        wait_diagnostic: 3,
                        hardware: Some(HardwareOwner::aggregate(
                            aggregate_pipe,
                            aggregate_slot,
                            member.frame_node,
                            aggregate_head,
                            member_count as u8,
                        )),
                    });
                }
            }
            Err(vendor_host_tx::AmpduPublishError::Ownership) => crate::halt_always!(),
            Err(_) => {
                for position in (0..member_count).rev() {
                    let Some(mut member) = members[position].take() else {
                        crate::halt_always!();
                    };
                    let _ = unsafe { member.reservation.cancel(guard, &mut member.retained) };
                    self.states[member.index] = Some(HostTxState::Owned {
                        retained: member.retained,
                        wait_diagnostic: member.wait_diagnostic,
                        hardware: None,
                    });
                }
            }
        }
    }

    /// Publish one or two ready PAS contexts onto a pipe without a retained
    /// runtime owner. A pair is staged into consecutive slots of the same pipe
    /// and crosses the MAC trigger boundary once.
    #[cfg(feature = "experimental-four-slot-ordinary")]
    unsafe fn publish_ready_batch(
        &mut self,
        mac_domain: &mut crate::mac_domain::MacDomain,
        owners: host_tx_policy::Class0RuntimeOwners,
    ) {
        let mut candidate_pipes = [None; HOST_CONTEXT_COUNT];
        let mut first_index = None;
        let mut ready_count = 0;
        for (index, state) in self.states.iter().enumerate() {
            let Some(HostTxState::Owned {
                retained,
                hardware: None,
                ..
            }) = state
            else {
                continue;
            };
            if retained.phase() != vendor_host_tx::HostTxPhase::PasQueued {
                continue;
            }
            let pipe = unsafe { vendor_host_tx::scheduler_live_diagnostic(retained) }.pipe;
            if pipe >= 4 {
                crate::halt_always!();
            }
            if owners.contains_pipe(pipe) {
                continue;
            }
            candidate_pipes[index] = Some(pipe);
            if first_index.is_none() {
                first_index = Some(index);
            }
            ready_count += 1;
        }
        #[cfg(feature = "experimental-member-requeue")]
        if let Some(index) = unsafe { vendor_host_tx::first_live_pas_frame() }.and_then(|frame_node| {
            self.states.iter().position(|state| {
                matches!(
                    state,
                    Some(HostTxState::Owned {
                        retained,
                        hardware: None,
                        ..
                    }) if retained.phase() == vendor_host_tx::HostTxPhase::PasQueued
                        && retained.context().frame_node().raw() == frame_node
                        && candidate_pipes[retained.context().index()].is_some()
                        && unsafe { vendor_host_tx::retry_attempted(retained) }
                )
            })
        }) {
            candidate_pipes[..index].fill(None);
            first_index = Some(index);
        }
        if !cfg!(feature = "experimental-fast-loop")
            && ready_count != 0
            && ready_count < host_tx_policy::MAX_ORDINARY_BATCH_DEPTH
            && self.scheduler_single_wait == 0
        {
            // Give the software lane one additional service pass to fill the
            // vendor's four-slot transaction. The next pass publishes whatever
            // is ready, preserving the existing bounded latency fallback.
            self.scheduler_single_wait = 1;
            return;
        }
        self.scheduler_single_wait = 0;
        let Some(first_index) = first_index else {
            return;
        };

        #[cfg(feature = "experimental-list-first-depth-four-ampdu")]
        {
            let Some(first_candidate) = self.states[first_index].as_ref().and_then(|state| {
                let HostTxState::Owned { retained, hardware: None, .. } = state else {
                    return None;
                };
                Some(unsafe { vendor_host_tx::ampdu_candidate(retained) })
            }) else {
                return;
            };
            let mut indices = [usize::MAX; 4];
            indices[0] = first_index;
            let mut member_count = 1;
            for (index, state) in self.states.iter().enumerate() {
                if index == first_index || candidate_pipes[index] != candidate_pipes[first_index] {
                    continue;
                }
                let Some(HostTxState::Owned { retained, hardware: None, .. }) = state else {
                    continue;
                };
                if vendor_host_tx::can_form_ampdu_pair(
                    first_candidate,
                    unsafe { vendor_host_tx::ampdu_candidate(retained) },
                ) {
                    indices[member_count] = index;
                    member_count += 1;
                    if member_count == 4 {
                        break;
                    }
                }
            }
            if member_count == 4 {
                let mut retained = [core::ptr::null_mut(); 4];
                let mut frames = [0_u32; 4];
                for position in 0..4 {
                    let Some(HostTxState::Owned { retained: member, hardware: None, .. }) =
                        self.states[indices[position]].as_mut()
                    else {
                        crate::halt_always!();
                    };
                    frames[position] = member.context().frame_node().raw();
                    retained[position] = member;
                }
                let _guard = mac_domain.enter();
                if let Ok((pipe, slot)) =
                    unsafe { vendor_host_tx::publish_list_first_depth_four(retained) }
                {
                    for position in 0..4 {
                        let Some(HostTxState::Owned {
                            hardware,
                            wait_diagnostic,
                            ..
                        }) = self.states[indices[position]].as_mut()
                        else {
                            crate::halt_always!();
                        };
                        *wait_diagnostic = 3;
                        *hardware = Some(HardwareOwner::aggregate(
                            pipe,
                            slot,
                            frames[position],
                            frames[0],
                            4,
                        ));
                    }
                }
                return;
            }
        }

        let Some(HostTxState::Owned {
            retained: mut first,
            wait_diagnostic: first_wait,
            hardware: None,
        }) = self.states[first_index].take()
        else {
            return;
        };

        let mut guard = mac_domain.enter();
        if !self.scheduler_phy_started_this_pass {
            let _ = unsafe { tx::start_phy_operation_1() };
            self.scheduler_phy_started_this_pass = true;
        }
        let first_reservation = match unsafe {
            vendor_host_tx::reserve_non_aggregate_scheduler(&mut guard, &mut first)
        } {
            Ok(reservation) => reservation,
            Err(vendor_host_tx::SchedulerReserveError::Expired) => {
                if unsafe { vendor_host_tx::reject_unscheduled_pas(&mut guard, &mut first) }.is_ok()
                {
                    let _ = first.transition(vendor_host_tx::HostTxPhase::Completing);
                    let completion_order = self.allocate_confirmation_order();
                    self.states[first_index] = Some(Self::confirmation_state(
                        first,
                        tx::wsm_status_from_internal(10),
                        0,
                        None,
                        completion_order,
                    ));
                } else {
                    self.states[first_index] = Some(HostTxState::Owned {
                        retained: first,
                        wait_diagnostic: first_wait,
                        hardware: None,
                    });
                }
                return;
            }
            Err(_) => {
                self.states[first_index] = Some(HostTxState::Owned {
                    retained: first,
                    wait_diagnostic: first_wait,
                    hardware: None,
                });
                return;
            }
        };
        let pipe = first_reservation.pipe();
        let first_slot = first_reservation.slot();
        let Some(occupied_slots) = owners.occupied_slots(pipe) else {
            crate::halt_always!();
        };
        let Some(plan) = host_tx_policy::plan_ordinary_batch(
            &candidate_pipes,
            occupied_slots,
            first_index,
            pipe,
            first_slot,
        ) else {
            crate::halt_always!();
        };

        let first_ampdu = unsafe { vendor_host_tx::ampdu_candidate(&first) };
        let second_ampdu = plan.index(1).and_then(|index| {
            let Some(HostTxState::Owned {
                retained,
                hardware: None,
                ..
            }) = self.states[index].as_ref()
            else {
                return None;
            };
            Some(unsafe { vendor_host_tx::ampdu_candidate(retained) })
        });
        let aggregate_pair = second_ampdu
            .is_some_and(|second| vendor_host_tx::can_form_ampdu_pair(first_ampdu, second));
        #[cfg(feature = "experimental-depth-four-ampdu")]
        let aggregate_len = {
            let mut candidates = [None; host_tx_policy::MAX_EXPERIMENTAL_AMPDU_DEPTH];
            for (position, candidate) in candidates.iter_mut().enumerate().take(plan.len()) {
                if position == 0 {
                    *candidate = Some(unsafe { vendor_host_tx::ampdu_plan_candidate(&first) });
                    continue;
                }
                let Some(index) = plan.index(position) else {
                    break;
                };
                let Some(HostTxState::Owned {
                    retained,
                    hardware: None,
                    ..
                }) = self.states[index].as_ref()
                else {
                    break;
                };
                *candidate = Some(unsafe { vendor_host_tx::ampdu_plan_candidate(retained) });
            }
            let (tx_ba_tids, _) = crate::configuration::block_ack_policy();
            let operational_tx_ba_tids = crate::configuration::operational_tx_ba_tids();
            #[cfg(feature = "experimental-aggregate-grouping-telemetry")]
            unsafe {
                host_tx_diagnostics::record_ampdu_outcome(
                    host_tx_diagnostics::ampdu_outcome::GROUPING_HEAD,
                );
                if first_ampdu.key.tid < 8
                    && operational_tx_ba_tids & (1 << first_ampdu.key.tid) != 0
                {
                    host_tx_diagnostics::record_ampdu_outcome(
                        host_tx_diagnostics::ampdu_outcome::GROUPING_SESSION,
                    );
                }
            }
            host_tx_policy::plan_ampdu_group(
                &candidates,
                tx_ba_tids,
                operational_tx_ba_tids,
                0,
            )
            .map_or(1, host_tx_policy::AmpduGroupPlan::len)
        };
        #[cfg(not(feature = "experimental-depth-four-ampdu"))]
        let aggregate_len = if aggregate_pair { 2 } else { 1 };
        if aggregate_len >= 2 {
            unsafe {
                #[cfg(feature = "experimental-aggregate-grouping-telemetry")]
                host_tx_diagnostics::record_ampdu_outcome(if aggregate_len == 2 {
                    host_tx_diagnostics::ampdu_outcome::GROUPING_DEPTH_TWO
                } else {
                    host_tx_diagnostics::ampdu_outcome::GROUPING_DEEP
                });
                host_tx_diagnostics::record_ampdu_candidate(
                    first_ampdu.key.tid,
                    first_ampdu.key.rate,
                );
            }
        }

        let mut members: [Option<ReservedBatchMember>; host_tx_policy::MAX_ORDINARY_BATCH_DEPTH] =
            [const { None }; host_tx_policy::MAX_ORDINARY_BATCH_DEPTH];
        let first_frame_node = first.context().frame_node().raw();
        members[0] = Some(ReservedBatchMember {
            index: first_index,
            retained: first,
            wait_diagnostic: first_wait,
            reservation: first_reservation,
            frame_node: first_frame_node,
        });
        let ordinary_len = if cfg!(feature = "experimental-four-slot-ordinary") {
            plan.len()
        } else {
            plan.len().min(2)
        };
        let target_len = if cfg!(feature = "experimental-depth-two-ampdu")
            && aggregate_len >= 2
        {
            aggregate_len
        } else {
            ordinary_len
        };
        let mut member_count = 1;
        for position in 1..target_len {
            let (Some(index), Some(slot)) = (plan.index(position), plan.slot(position)) else {
                crate::halt_always!();
            };
            let Some(HostTxState::Owned {
                retained: mut retained_member,
                wait_diagnostic,
                hardware: None,
            }) = self.states[index].take()
            else {
                crate::halt_always!();
            };
            let reservation = match unsafe {
                vendor_host_tx::reserve_non_aggregate_scheduler_in_batch(
                    &mut guard,
                    &mut retained_member,
                    pipe,
                    slot,
                    position as u8,
                )
            } {
                Ok(reservation) => reservation,
                Err(_) => {
                    self.states[index] = Some(HostTxState::Owned {
                        retained: retained_member,
                        wait_diagnostic,
                        hardware: None,
                    });
                    break;
                }
            };
            let frame_node = retained_member.context().frame_node().raw();
            members[position] = Some(ReservedBatchMember {
                index,
                retained: retained_member,
                wait_diagnostic,
                reservation,
                frame_node,
            });
            member_count += 1;
        }

        if member_count == 1 {
            let Some(member) = members[0].take() else {
                crate::halt_always!();
            };
            let ReservedBatchMember {
                index,
                mut retained,
                wait_diagnostic,
                reservation,
                frame_node,
            } = member;
            match unsafe { reservation.publish(&mut guard, &mut retained) } {
                Ok(()) => {
                    self.states[index] = Some(HostTxState::Owned {
                        hardware: Some(HardwareOwner::single(pipe, first_slot, frame_node)),
                        retained,
                        wait_diagnostic: 3,
                    });
                }
                Err((reservation, _)) => {
                    self.states[index] = Some(HostTxState::Reserved {
                        retained,
                        reservation,
                        wait_diagnostic,
                    });
                }
            }
            return;
        }

        #[cfg(all(
            feature = "experimental-depth-two-ampdu",
            not(feature = "experimental-list-first-depth-four-ampdu")
        ))]
        if aggregate_pair && member_count == 2 {
            let Some(first_member) = members[0].take() else {
                crate::halt_always!();
            };
            let Some(second_member) = members[1].take() else {
                crate::halt_always!();
            };
            let ReservedBatchMember {
                index: first_index,
                retained: mut first,
                wait_diagnostic: first_wait,
                reservation: first_reservation,
                frame_node: first_frame_node,
            } = first_member;
            let ReservedBatchMember {
                index: second_index,
                retained: mut second,
                wait_diagnostic: second_wait,
                reservation: second_reservation,
                frame_node: second_frame_node,
            } = second_member;
            match unsafe {
                vendor_host_tx::publish_depth_two_ampdu(
                    &mut guard,
                    &mut first,
                    &mut second,
                    first_reservation,
                    second_reservation,
                )
            } {
                Ok((aggregate_pipe, aggregate_slot)) => {
                    self.states[first_index] = Some(HostTxState::Owned {
                        retained: first,
                        wait_diagnostic: 3,
                        hardware: Some(HardwareOwner::aggregate(
                            aggregate_pipe,
                            aggregate_slot,
                            first_frame_node,
                            first_frame_node,
                            2,
                        )),
                    });
                    self.states[second_index] = Some(HostTxState::Owned {
                        retained: second,
                        wait_diagnostic: 3,
                        hardware: Some(HardwareOwner::aggregate(
                            aggregate_pipe,
                            aggregate_slot,
                            second_frame_node,
                            first_frame_node,
                            2,
                        )),
                    });
                    return;
                }
                Err(vendor_host_tx::AmpduPublishError::Ownership) => crate::halt_always!(),
                Err(_) => {
                    self.states[first_index] = Some(HostTxState::Owned {
                        retained: first,
                        wait_diagnostic: first_wait,
                        hardware: None,
                    });
                    self.states[second_index] = Some(HostTxState::Owned {
                        retained: second,
                        wait_diagnostic: second_wait,
                        hardware: None,
                    });
                    return;
                }
            }
        }

        #[cfg(all(
            feature = "experimental-depth-four-ampdu",
            not(feature = "experimental-list-first-depth-four-ampdu")
        ))]
        if aggregate_len >= 3 && member_count == aggregate_len {
            unsafe {
                host_tx_diagnostics::record_ampdu_depth(0, member_count as u8);
                self.publish_depth_four_reserved(&mut guard, &mut members, member_count)
            };
            return;
        }

        struct PublishedBatchMember {
            index: usize,
            retained: vendor_host_tx::RetainedHostTx,
            slot: u8,
            frame_node: u32,
        }
        let mut published: [Option<PublishedBatchMember>;
            host_tx_policy::MAX_ORDINARY_BATCH_DEPTH] =
            [const { None }; host_tx_policy::MAX_ORDINARY_BATCH_DEPTH];
        for position in 0..member_count {
            let Some(member) = members[position].take() else {
                crate::halt_always!();
            };
            let ReservedBatchMember {
                index,
                mut retained,
                reservation,
                frame_node,
                ..
            } = member;
            let batch = if position == 0 {
                tx::BatchPosition::First
            } else if position + 1 == member_count {
                tx::BatchPosition::Last
            } else {
                tx::BatchPosition::Middle
            };
            if unsafe { reservation.publish_in_batch(&mut guard, &mut retained, batch) }.is_err() {
                crate::halt_always!();
            }
            let Some(slot) = plan.slot(position) else {
                crate::halt_always!();
            };
            published[position] = Some(PublishedBatchMember {
                index,
                retained,
                slot,
                frame_node,
            });
        }
        let Some(last_slot) = plan.slot(member_count - 1) else {
            crate::halt_always!();
        };
        unsafe {
            tx::finalize_staged_host_class0_pipe(&mut guard, pipe, first_slot, last_slot);
            host_tx_diagnostics::record_batch_publication(member_count as u8);
        }
        for position in 0..member_count {
            let Some(member) = published[position].take() else {
                crate::halt_always!();
            };
            self.states[member.index] = Some(HostTxState::Owned {
                retained: member.retained,
                wait_diagnostic: 3,
                hardware: Some(HardwareOwner::single(pipe, member.slot, member.frame_node)),
            });
        }
    }

    #[cfg(not(feature = "experimental-four-slot-ordinary"))]
    unsafe fn publish_ready_batch(
        &mut self,
        mac_domain: &mut crate::mac_domain::MacDomain,
        owners: host_tx_policy::Class0RuntimeOwners,
    ) {
        let mut first_index = None;
        let mut ready_count = 0;
        for (index, state) in self.states.iter().enumerate() {
            let Some(HostTxState::Owned {
                retained,
                hardware: None,
                ..
            }) = state
            else {
                continue;
            };
            if retained.phase() != vendor_host_tx::HostTxPhase::PasQueued {
                continue;
            }
            let pipe = unsafe { vendor_host_tx::scheduler_live_diagnostic(retained) }.pipe;
            if pipe >= 4 {
                crate::halt_always!();
            }
            if owners.contains_pipe(pipe) {
                continue;
            }
            if first_index.is_none() {
                first_index = Some(index);
            }
            ready_count += 1;
            if ready_count == 2 {
                break;
            }
        }
        if !cfg!(feature = "experimental-fast-loop")
            && ready_count == 1
            && self.scheduler_single_wait == 0
        {
            // The command lane admits at most one request after this service
            // pass. Give it one pass to supply a partner before falling back
            // to the latency-safe single-frame path.
            self.scheduler_single_wait = 1;
            return;
        }
        self.scheduler_single_wait = 0;
        let Some(first_index) = first_index else {
            return;
        };

        #[cfg(feature = "experimental-list-first-ampdu")]
        {
            let Some((first_candidate, first_pipe)) = self.states[first_index].as_ref().and_then(|state| {
                let HostTxState::Owned { retained, hardware: None, .. } = state else {
                    return None;
                };
                Some((
                    unsafe { vendor_host_tx::ampdu_candidate(retained) },
                    unsafe { vendor_host_tx::scheduler_live_diagnostic(retained) }.pipe,
                ))
            }) else {
                return;
            };
            let second_index = self.states.iter().enumerate().find_map(|(index, state)| {
                if index == first_index {
                    return None;
                }
                let HostTxState::Owned { retained, hardware: None, .. } = state.as_ref()? else {
                    return None;
                };
                (unsafe { vendor_host_tx::scheduler_live_diagnostic(retained) }.pipe == first_pipe
                    && vendor_host_tx::can_form_ampdu_pair(
                        first_candidate,
                        unsafe { vendor_host_tx::ampdu_candidate(retained) },
                    ))
                .then_some(index)
            });
            if let Some(second_index) = second_index {
                let Some(HostTxState::Owned {
                    retained: mut first,
                    wait_diagnostic: first_wait,
                    hardware: None,
                }) = self.states[first_index].take()
                else {
                    crate::halt_always!();
                };
                let Some(HostTxState::Owned {
                    retained: mut second,
                    wait_diagnostic: second_wait,
                    hardware: None,
                }) = self.states[second_index].take()
                else {
                    crate::halt_always!();
                };
                let first_frame = first.context().frame_node().raw();
                let second_frame = second.context().frame_node().raw();
                let _guard = mac_domain.enter();
                match unsafe {
                    vendor_host_tx::publish_list_first_depth_two(&mut first, &mut second)
                } {
                    Ok((pipe, slot)) => {
                        self.states[first_index] = Some(HostTxState::Owned {
                            retained: first,
                            wait_diagnostic: 3,
                            hardware: Some(HardwareOwner::aggregate(
                                pipe, slot, first_frame, first_frame, 2,
                            )),
                        });
                        self.states[second_index] = Some(HostTxState::Owned {
                            retained: second,
                            wait_diagnostic: 3,
                            hardware: Some(HardwareOwner::aggregate(
                                pipe, slot, second_frame, first_frame, 2,
                            )),
                        });
                    }
                    Err(_) => {
                        self.states[first_index] = Some(HostTxState::Owned {
                            retained: first,
                            wait_diagnostic: first_wait,
                            hardware: None,
                        });
                        self.states[second_index] = Some(HostTxState::Owned {
                            retained: second,
                            wait_diagnostic: second_wait,
                            hardware: None,
                        });
                    }
                }
                return;
            }
        }

        let Some(HostTxState::Owned {
            retained: mut first,
            wait_diagnostic: first_wait,
            hardware: None,
        }) = self.states[first_index].take()
        else {
            return;
        };

        let mut guard = mac_domain.enter();
        if !self.scheduler_phy_started_this_pass {
            let _ = unsafe { tx::start_phy_operation_1() };
            self.scheduler_phy_started_this_pass = true;
        }
        let first_reservation = match unsafe {
            vendor_host_tx::reserve_non_aggregate_scheduler(&mut guard, &mut first)
        } {
            Ok(reservation) => reservation,
            Err(vendor_host_tx::SchedulerReserveError::Expired) => {
                if unsafe { vendor_host_tx::reject_unscheduled_pas(&mut guard, &mut first) }
                    .is_ok()
                {
                    let _ = first.transition(vendor_host_tx::HostTxPhase::Completing);
                    let completion_order = self.allocate_confirmation_order();
                    self.states[first_index] = Some(Self::confirmation_state(
                        first,
                        tx::wsm_status_from_internal(10),
                        0,
                        None,
                        completion_order,
                    ));
                } else {
                    self.states[first_index] = Some(HostTxState::Owned {
                        retained: first,
                        wait_diagnostic: first_wait,
                        hardware: None,
                    });
                }
                return;
            }
            Err(_) => {
                self.states[first_index] = Some(HostTxState::Owned {
                    retained: first,
                    wait_diagnostic: first_wait,
                    hardware: None,
                });
                return;
            }
        };
        let pipe = first_reservation.pipe();
        let first_slot = first_reservation.slot();

        let second_index = self.states.iter().position(|state| {
            matches!(
                state,
                Some(HostTxState::Owned { retained, hardware: None, .. })
                    if retained.phase() == vendor_host_tx::HostTxPhase::PasQueued
                        && unsafe { vendor_host_tx::scheduler_live_diagnostic(retained) }.pipe == pipe
            )
        });
        let Some(second_index) = second_index else {
            match unsafe { first_reservation.publish(&mut guard, &mut first) } {
                Ok(()) => {
                    self.states[first_index] = Some(HostTxState::Owned {
                        hardware: Some(HardwareOwner::single(
                            pipe,
                            first_slot,
                            first.context().frame_node().raw(),
                        )),
                        retained: first,
                        wait_diagnostic: 3,
                    });
                }
                Err((reservation, _)) => {
                    self.states[first_index] = Some(HostTxState::Reserved {
                        retained: first,
                        reservation,
                        wait_diagnostic: first_wait,
                    });
                }
            }
            return;
        };
        let Some(HostTxState::Owned {
            retained: mut second,
            wait_diagnostic: second_wait,
            hardware: None,
        }) = self.states[second_index].take()
        else {
            let _ = unsafe { first_reservation.cancel(&mut guard, &mut first) };
            self.states[first_index] = Some(HostTxState::Owned {
                retained: first,
                wait_diagnostic: first_wait,
                hardware: None,
            });
            return;
        };
        let first_ampdu = unsafe { vendor_host_tx::ampdu_candidate(&first) };
        let second_ampdu = unsafe { vendor_host_tx::ampdu_candidate(&second) };
        if vendor_host_tx::can_form_ampdu_pair(first_ampdu, second_ampdu) {
            unsafe {
                host_tx_diagnostics::record_ampdu_candidate(
                    first_ampdu.key.tid,
                    first_ampdu.key.rate,
                );
            }
        }
        let second_slot = first_slot.wrapping_add(1) & 3;
        let second_reservation = match unsafe {
            vendor_host_tx::reserve_non_aggregate_scheduler_in_batch(
                &mut guard,
                &mut second,
                pipe,
                second_slot,
                1,
            )
        } {
            Ok(reservation) => reservation,
            Err(_) => {
                self.states[second_index] = Some(HostTxState::Owned {
                    retained: second,
                    wait_diagnostic: second_wait,
                    hardware: None,
                });
                match unsafe { first_reservation.publish(&mut guard, &mut first) } {
                    Ok(()) => {
                        self.states[first_index] = Some(HostTxState::Owned {
                            hardware: Some(HardwareOwner::single(
                                pipe,
                                first_slot,
                                first.context().frame_node().raw(),
                            )),
                            retained: first,
                            wait_diagnostic: 3,
                        });
                    }
                    Err((reservation, _)) => {
                        self.states[first_index] = Some(HostTxState::Reserved {
                            retained: first,
                            reservation,
                            wait_diagnostic: first_wait,
                        });
                    }
                }
                return;
            }
        };

        let first_frame_node = first.context().frame_node().raw();
        let second_frame_node = second.context().frame_node().raw();
        #[cfg(feature = "experimental-depth-two-ampdu")]
        if vendor_host_tx::can_form_ampdu_pair(first_ampdu, second_ampdu) {
            match unsafe {
                vendor_host_tx::publish_depth_two_ampdu(
                    &mut guard,
                    &mut first,
                    &mut second,
                    first_reservation,
                    second_reservation,
                )
            } {
                Ok((aggregate_pipe, aggregate_slot)) => {
                    self.states[first_index] = Some(HostTxState::Owned {
                        retained: first,
                        wait_diagnostic: 3,
                        hardware: Some(HardwareOwner::aggregate(
                            aggregate_pipe,
                            aggregate_slot,
                            first_frame_node,
                            first_frame_node,
                            2,
                        )),
                    });
                    self.states[second_index] = Some(HostTxState::Owned {
                        retained: second,
                        wait_diagnostic: 3,
                        hardware: Some(HardwareOwner::aggregate(
                            aggregate_pipe,
                            aggregate_slot,
                            second_frame_node,
                            first_frame_node,
                            2,
                        )),
                    });
                    return;
                }
                Err(vendor_host_tx::AmpduPublishError::Ownership) => crate::halt_always!(),
                Err(_) => {
                    self.states[first_index] = Some(HostTxState::Owned {
                        retained: first,
                        wait_diagnostic: first_wait,
                        hardware: None,
                    });
                    self.states[second_index] = Some(HostTxState::Owned {
                        retained: second,
                        wait_diagnostic: second_wait,
                        hardware: None,
                    });
                    return;
                }
            }
        }
        let first_result = unsafe {
            first_reservation.publish_in_batch(&mut guard, &mut first, tx::BatchPosition::First)
        };
        let Err((_reservation, _)) = first_result else {
            let second_result = unsafe {
                second_reservation.publish_in_batch(&mut guard, &mut second, tx::BatchPosition::Last)
            };
            if second_result.is_err() {
                crate::halt_always!();
            }
            unsafe {
                tx::finalize_staged_host_class0_pipe(
                    &mut guard,
                    pipe,
                    first_slot,
                    second_slot,
                );
                host_tx_diagnostics::record_batch_publication(2);
            }
            self.states[first_index] = Some(HostTxState::Owned {
                retained: first,
                wait_diagnostic: 3,
                hardware: Some(HardwareOwner::single(pipe, first_slot, first_frame_node)),
            });
            self.states[second_index] = Some(HostTxState::Owned {
                retained: second,
                wait_diagnostic: 3,
                hardware: Some(HardwareOwner::single(pipe, second_slot, second_frame_node)),
            });
            return;
        };
        crate::halt_always!();
    }

    unsafe fn service_index(
        &mut self,
        index: usize,
        _events: &mut tx::MacEventQueue,
        mac_domain: &mut crate::mac_domain::MacDomain,
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
                let mut guard = mac_domain.enter();
                match unsafe { reservation.publish(&mut guard, &mut retained) } {
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
                        let hardware = HardwareOwner::single(pipe, slot, retained.context().frame_node().raw());
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
                    match unsafe { vendor_host_tx::service_pending(mac_domain, &mut retained) } {
                        Ok(vendor_host_tx::PendingServiceReport::Complete(status)) => {
                            let _ = retained.transition(vendor_host_tx::HostTxPhase::Completing);
                            let completion_order = self.allocate_confirmation_order();
                            self.states[index] = Some(Self::confirmation_state(
                                retained,
                                tx::wsm_status_from_internal(status),
                                0,
                                None,
                                completion_order,
                            ));
                            return None;
                        }
                        Ok(vendor_host_tx::PendingServiceReport::PasQueued) => {
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
                                        | u32::from(diagnostic.vif_mode_byte)
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

                let Some(hardware_owners) = self.hardware_runtime_owners() else {
                    crate::halt_always!();
                };
                if retained.phase() == vendor_host_tx::HostTxPhase::PasQueued
                    && allow_hardware_publication
                    && hardware_owners.is_empty()
                {
                    let mut guard = mac_domain.enter();
                    if !self.scheduler_phy_started_this_pass {
                        // Vendor starts PHY command 1 once before each
                        // non-empty scheduler pass.
                        let _ = unsafe { tx::start_phy_operation_1() };
                        self.scheduler_phy_started_this_pass = true;
                    }
                    match unsafe {
                        vendor_host_tx::reserve_non_aggregate_scheduler(&mut guard, &mut retained)
                    } {
                        Ok(reservation) => {
                            let pipe = reservation.pipe();
                            let slot = reservation.slot();
                            match unsafe { reservation.publish(&mut guard, &mut retained) } {
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
                                    hardware = Some(HardwareOwner::single(pipe, slot, retained.context().frame_node().raw()));
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
                            if unsafe {
                                vendor_host_tx::reject_unscheduled_pas(&mut guard, &mut retained)
                            }
                            .is_ok()
                            {
                                let _ =
                                    retained.transition(vendor_host_tx::HostTxPhase::Completing);
                                let completion_order = self.allocate_confirmation_order();
                                self.states[index] = Some(Self::confirmation_state(
                                    retained,
                                    tx::wsm_status_from_internal(10),
                                    0,
                                    None,
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

    // The context normally contains exact per-rate failure nibbles. The
    // host-testable fallback reconstructs them from ack_failures only when an
    // older completion path did not populate those words.
    fn confirmation_state(
        retained: vendor_host_tx::RetainedHostTx,
        status: u32,
        ack_failures: u8,
        hardware: Option<HardwareOwner>,
        completion_order: u32,
    ) -> HostTxState {
        let context = retained.context();
        let fields = unsafe { vendor_host_tx::confirmation_fields(context) };
        let tx_rate = fields.tx_rate;
        let hardware = hardware
            .unwrap_or_else(|| HardwareOwner::single(0, 0, retained.context().frame_node().raw()));
        unsafe {
            host_tx_diagnostics::capture_retry_feedback(context, status, tx_rate, ack_failures);
            host_tx_diagnostics::record_rate_feedback(ack_failures, fields.rate_try);
        }
        HostTxState::Confirming {
            confirmation: HostTxConfirmation {
                packet_id: retained.packet_id(),
                context: context.raw(),
                status,
                tx_rate,
                ack_failures,
                flags: fields.flags,
                rate_try: {
                    let reported = fields.rate_try;
                    if reported == [0; 3] {
                        host_tx_policy::rate_try_for_single_rate(tx_rate, ack_failures)
                    } else {
                        reported
                    }
                },
                #[cfg(feature = "experimental-aggregate-rate-feedback")]
                aggregate_head: hardware.aggregate_head,
                #[cfg(feature = "experimental-aggregate-rate-feedback")]
                aggregate_len: hardware.aggregate_len,
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
        let Some(HostTxState::Confirming {
            mut confirmation, ..
        }) = self.states[index]
        else {
            return None;
        };
        #[cfg(feature = "experimental-aggregate-rate-feedback")]
        {
            const AGGREGATE_METADATA: u16 = 1 << 7;
            const AGGREGATE_HEAD: u16 = 1 << 8;
            if confirmation.aggregate_len == 1 {
                return Some(confirmation);
            }
            confirmation.flags |= AGGREGATE_METADATA;
            let mut acknowledged = 0_u8;
            for state in &self.states {
                let Some(HostTxState::Confirming {
                    confirmation: member,
                    ..
                }) = state
                else {
                    continue;
                };
                if member.aggregate_head != confirmation.aggregate_head {
                    continue;
                }
                if member.status == 0 {
                    acknowledged = acknowledged.saturating_add(1);
                }
            }
            confirmation.flags |= AGGREGATE_HEAD
                | (u16::from(confirmation.aggregate_len.saturating_sub(1) & 0x7) << 9)
                | (u16::from(acknowledged.min(0xf)) << 12);
        }
        Some(confirmation)
    }

    pub fn confirmation_count(&self, limit: usize) -> usize {
        self.states
            .iter()
            .filter(|state| matches!(state, Some(HostTxState::Confirming { .. })))
            .count()
            .min(limit)
    }

    /// Finish the same first confirmation returned by `confirmation()` at the
    /// inherited HIF handoff point. `publish_request_in_place` subsequently
    /// returns request credit before enqueuing its copied output buffer.
    ///
    /// # Safety
    /// Completion accounting must already have removed hardware ownership.
    pub unsafe fn finish_confirmation(&mut self) -> Option<hif::RequestReleaseToken> {
        let index = self.confirmation_index()?;
        #[cfg(feature = "experimental-aggregate-rate-feedback")]
        let reported_group = match self.states[index] {
            Some(HostTxState::Confirming { confirmation, .. })
                if confirmation.aggregate_len >= 2 => Some(confirmation.aggregate_head),
            _ => None,
        };
        let HostTxState::Confirming { owner, .. } = self.states[index].take()? else {
            return None;
        };
        #[cfg(feature = "experimental-aggregate-rate-feedback")]
        if let Some(aggregate_head) = reported_group {
            for state in &mut self.states {
                if let Some(HostTxState::Confirming { confirmation, .. }) = state
                    && confirmation.aggregate_head == aggregate_head
                {
                    confirmation.aggregate_len = 1;
                }
            }
        }
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
    pub unsafe fn reset(&mut self, mac_domain: &mut crate::mac_domain::MacDomain) {
        self.scheduler_single_wait = 0;
        let mut guard = mac_domain.enter();
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
                    let _ = unsafe { reservation.cancel(&mut guard, &mut retained) };
                    let completion_order = self.allocate_confirmation_order();
                    unsafe { Self::cancelled_confirmation(&mut guard, retained, completion_order) }
                }
                HostTxState::Owned { retained, .. }
                    if retained.phase() != vendor_host_tx::HostTxPhase::Scheduled =>
                {
                    let completion_order = self.allocate_confirmation_order();
                    unsafe { Self::cancelled_confirmation(&mut guard, retained, completion_order) }
                }
                state @ HostTxState::Owned { .. } => state,
            });
        }
    }

    unsafe fn cancelled_confirmation(
        guard: &mut crate::mac_domain::MacDomainGuard<'_>,
        retained: vendor_host_tx::RetainedHostTx,
        completion_order: u32,
    ) -> HostTxState {
        let context = retained.context();
        let packet_id = retained.packet_id();
        let fields = unsafe { vendor_host_tx::confirmation_fields(context) };
        let tx_rate = fields.tx_rate;
        let rate_try = fields.rate_try;
        let release = unsafe {
            retained
                .cancel_before_pas(guard)
                .expect("reversible host TX cancellation")
        };
        HostTxState::Confirming {
            owner: ConfirmationOwner::Release(release),
            confirmation: HostTxConfirmation {
                packet_id,
                context: context.raw(),
                status: 1,
                tx_rate,
                ack_failures: 0,
                flags: 0,
                rate_try,
                #[cfg(feature = "experimental-aggregate-rate-feedback")]
                aggregate_head: context.frame_node().raw(),
                #[cfg(feature = "experimental-aggregate-rate-feedback")]
                aggregate_len: 1,
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
