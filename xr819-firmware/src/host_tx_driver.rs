//! Concurrent retained host-TX ownership with a pipe-indexed class-0 MAC executor.
//!
//! Each of the 30 host contexts owns its original HIF request until the inherited
//! confirmation handoff point. The current HIF transport then returns request
//! credit before it enqueues the separately copied confirmation; this module does
//! not redesign that parent ordering. Pending contexts advance independently.
//! Retained hardware owners are discovered independently per MAC pipe;
//! publication admits at most one new batch per pass and never targets a pipe
//! that still has a retained runtime owner. Already-scheduled owners advance
//! from MAC completion events, so cooperative service budget is reserved for
//! software contexts and reservations that can still make progress.

use crate::{hif, host_tx_policy, tx, vendor_host_tx};

/// Host TX contexts in the fixed vendor arena; hardware context addresses
/// define slot identity.
const HOST_CONTEXT_COUNT: usize = 30;
/// Contexts advanced per service pass.
///
/// At 4, reservations that still need publication consume the front of the
/// budget and the remainder advances software owners. Already-scheduled slots
/// are completion-driven and are deliberately skipped instead of consuming a
/// no-op service turn. Queued contexts can still wait many passes before
/// publication. Keep the bounded vendor-shaped service loop until hardware
/// concurrency is made ownership-safe.
const SERVICE_BUDGET: usize = 4;

pub struct HostTxDriver {
    states: [Option<HostTxState>; HOST_CONTEXT_COUNT],
    admission_orders: [u32; HOST_CONTEXT_COUNT],
    next_admission_order: u32,
    hardware_service_cursor: u8,
    next_confirmation_order: u32,
    scheduler_phy_started_this_pass: bool,
}

enum HostTxState {
    Owned {
        retained: vendor_host_tx::RetainedHostTx,
        hardware: Option<HardwareOwner>,
    },
    Reserved {
        retained: vendor_host_tx::RetainedHostTx,
        reservation: vendor_host_tx::HostSchedulerReservation,
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

#[cfg(target_arch = "arm")]
struct ReservedBatchMember {
    index: usize,
    retained: vendor_host_tx::RetainedHostTx,
    reservation: vendor_host_tx::HostSchedulerReservation,
    frame_node: u32,
}

impl HardwareOwner {
    const fn single(pipe: u8, slot: u8, frame_node: u32) -> Self {
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
}

impl HostTxDriver {
    pub const fn new() -> Self {
        Self {
            states: [const { None }; HOST_CONTEXT_COUNT],
            admission_orders: [0; HOST_CONTEXT_COUNT],
            next_admission_order: 0,
            hardware_service_cursor: 0,
            next_confirmation_order: 0,
            scheduler_phy_started_this_pass: false,
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
            let release = unsafe { retained.abort() };
            transport.release_request(release);
            return false;
        }
        if unsafe { vendor_host_tx::classify_and_encrypt(&mut retained) }.is_err() {
            let release = unsafe { retained.abort() };
            transport.release_request(release);
            return false;
        }
        if unsafe { vendor_host_tx::enqueue_post_crypto(&mut retained) }.is_err() {
            let release = unsafe { retained.abort() };
            transport.release_request(release);
            return false;
        }
        self.admission_orders[index] = self.next_admission_order;
        self.next_admission_order = self.next_admission_order.wrapping_add(1);
        self.states[index] = Some(HostTxState::Owned {
            retained,
            hardware: None,
        });
        true
    }

    /// Advance a bounded subset of contexts. A reserved/scheduled context is
    /// always serviced first; remaining budget visits pending contexts in
    /// admission order, at most once each per pass. Arena reuse is not FIFO.
    ///
    /// # Safety
    /// This driver must be the sole class-0 owner of pending/PAS/scheduler and
    /// MAC-event mutation.
    pub unsafe fn service(
        &mut self,
        events: &mut tx::MacEventQueue,
        mac_domain: &mut crate::mac_domain::MacDomain,
        allow_hardware_publication: bool,
    ) {
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
            // Scheduled contexts cannot advance here: their only transitions
            // are driven by the completion/requeue drain above. Do not let up
            // to four inert physical slots starve pending software contexts.
            if !matches!(self.states[index], Some(HostTxState::Reserved { .. })) {
                continue;
            }
            unsafe {
                self.service_index(index, events, mac_domain, allow_hardware_publication);
            }
            budget -= 1;
        }

        let mut software_visited = [false; HOST_CONTEXT_COUNT];
        while budget != 0 {
            let Some(index) = self.next_software_owner(&software_visited) else {
                break;
            };
            software_visited[index] = true;
            unsafe {
                self.service_index(index, events, mac_domain, false);
            }
            budget -= 1;
        }
        let Some(owners) = self.hardware_runtime_owners() else {
            crate::halt_always!();
        };
        if allow_hardware_publication && owners.is_empty() {
            unsafe { self.publish_ready_batch(mac_domain, owners) };
        }
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

    fn next_software_owner(&self, visited: &[bool; HOST_CONTEXT_COUNT]) -> Option<usize> {
        let pending = self.states.iter().enumerate().filter_map(|(index, state)| {
            let Some(HostTxState::Owned { retained, hardware: None, .. }) = state else {
                return None;
            };
            (!visited[index] && matches!(retained.phase(),
                vendor_host_tx::HostTxPhase::PostCryptoQueued
                    | vendor_host_tx::HostTxPhase::PendingEligible
            )).then_some((index, self.admission_orders[index]))
        });
        host_tx_policy::oldest_pending_context(pending, self.next_admission_order)
    }

    fn route_hardware_completion(&mut self, completion: tx::HostClass0Completion) {
        let Some(owners) = self.hardware_runtime_owners() else {
            crate::halt_always!();
        };
        if owners
            .slot_owner(completion.pipe, completion.slot)
            .is_none()
        {
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
            return;
        };
        let Some(HostTxState::Owned {
            mut retained,
            hardware: Some(_),
            ..
        }) = self.states[index].take()
        else {
            return;
        };
        let _ = retained.transition(vendor_host_tx::HostTxPhase::Completing);
        let completion_order = self.allocate_confirmation_order();
        self.states[index] = Some(Self::confirmation_state(
            retained,
            tx::wsm_status_from_internal(completion.status),
            completion.ack_failures,
            completion_order,
        ));
    }

    /// Publish one or two ready PAS contexts onto a pipe without a retained
    /// runtime owner. A pair is staged into consecutive slots of the same pipe
    /// and crosses the MAC trigger boundary once.
    unsafe fn publish_ready_batch(
        &mut self,
        mac_domain: &mut crate::mac_domain::MacDomain,
        owners: host_tx_policy::Class0RuntimeOwners,
    ) {
        let mut candidate_pipes = [None; HOST_CONTEXT_COUNT];
        let mut fifo_distances = [u8::MAX; HOST_CONTEXT_COUNT];
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
            let Some(distance) = (unsafe { vendor_host_tx::queued_pas_ring_distance(retained) }) else {
                continue;
            };
            candidate_pipes[index] = Some(pipe);
            fifo_distances[index] = distance;
            ready_count += 1;
        }
        // Context indices are reusable storage, not transmission order. Preserve
        // PAS order for the head and every member, including ordinary batches.
        let candidate_order: [usize; HOST_CONTEXT_COUNT] = core::array::from_fn(|_| {
            let next = fifo_distances.iter().enumerate()
                .filter(|(_, distance)| **distance != u8::MAX)
                .min_by_key(|(_, distance)| **distance)
                .map(|(index, _)| index);
            if let Some(index) = next {
                fifo_distances[index] = u8::MAX;
            }
            next.unwrap_or(usize::MAX)
        });
        let Some(&first_index) = candidate_order[..ready_count].first() else {
            return;
        };

        let Some(HostTxState::Owned {
            retained: mut first,
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
                        completion_order,
                    ));
                } else {
                    self.states[first_index] = Some(HostTxState::Owned {
                        retained: first,
                        hardware: None,
                    });
                }
                return;
            }
            Err(_) => {
                self.states[first_index] = Some(HostTxState::Owned {
                    retained: first,
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
            &candidate_order[..ready_count],
            occupied_slots,
            first_index,
            pipe,
            first_slot,
        ) else {
            crate::halt_always!();
        };

        let mut members: [Option<ReservedBatchMember>; host_tx_policy::MAX_ORDINARY_BATCH_DEPTH] =
            [const { None }; host_tx_policy::MAX_ORDINARY_BATCH_DEPTH];
        let first_frame_node = first.context().frame_node().raw();
        members[0] = Some(ReservedBatchMember {
            index: first_index,
            retained: first,
            reservation: first_reservation,
            frame_node: first_frame_node,
        });
        // Publish one bounded FIFO cohort, then let the service-level owner
        // barrier retire every member before another cohort reaches hardware.
        // Overtaking is therefore bounded by this four-slot transaction rather
        // than growing without limit across repeated slot reuse.
        let mut member_count = 1;
        for position in 1..plan.len() {
            let (Some(index), Some(slot)) = (plan.index(position), plan.slot(position)) else {
                crate::halt_always!();
            };
            let Some(HostTxState::Owned {
                retained: mut retained_member,
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
                        hardware: None,
                    });
                    break;
                }
            };
            let frame_node = retained_member.context().frame_node().raw();
            members[position] = Some(ReservedBatchMember {
                index,
                retained: retained_member,
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
                reservation,
                frame_node,
            } = member;
            match unsafe { reservation.publish(&mut guard, &mut retained) } {
                Ok(()) => {
                    self.states[index] = Some(HostTxState::Owned {
                        hardware: Some(HardwareOwner::single(pipe, first_slot, frame_node)),
                        retained,
                    });
                }
                Err((reservation, _)) => {
                    self.states[index] = Some(HostTxState::Reserved {
                        retained,
                        reservation,
                    });
                }
            }
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
        }
        for position in 0..member_count {
            let Some(member) = published[position].take() else {
                crate::halt_always!();
            };
            self.states[member.index] = Some(HostTxState::Owned {
                retained: member.retained,
                hardware: Some(HardwareOwner::single(pipe, member.slot, member.frame_node)),
            });
        }
    }

    unsafe fn service_index(
        &mut self,
        index: usize,
        _events: &mut tx::MacEventQueue,
        mac_domain: &mut crate::mac_domain::MacDomain,
        allow_hardware_publication: bool,
    ) {
        let Some(state) = self.states[index].take() else {
            return;
        };
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
            } => {
                let pipe = reservation.pipe();
                let slot = reservation.slot();
                let mut guard = mac_domain.enter();
                match unsafe { reservation.publish(&mut guard, &mut retained) } {
                    Ok(()) => {
                        let hardware = HardwareOwner::single(pipe, slot, retained.context().frame_node().raw());
                        HostTxState::Owned {
                            retained,
                            hardware: Some(hardware),
                        }
                    }
                    Err((reservation, _)) => {
                        HostTxState::Reserved {
                            retained,
                            reservation,
                        }
                    }
                }
            }
            HostTxState::Owned {
                mut retained,
                mut hardware,
            } => {
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
                                completion_order,
                            ));
                            return;
                        }
                        Ok(
                            vendor_host_tx::PendingServiceReport::PasQueued
                            | vendor_host_tx::PendingServiceReport::LeaveQueued,
                        )
                        | Err(_) => {}
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
                                    hardware = Some(HardwareOwner::single(pipe, slot, retained.context().frame_node().raw()));
                                    self.states[index] = Some(HostTxState::Owned {
                                        retained,
                                        hardware,
                                    });
                                }
                                Err((reservation, _)) => {
                                    self.states[index] = Some(HostTxState::Reserved {
                                        retained,
                                        reservation,
                                    });
                                }
                            }
                            return;
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
                                    completion_order,
                                ));
                                return;
                            }
                        }
                        Err(_) => {}
                    }
                }

                HostTxState::Owned {
                    retained,
                    hardware,
                }
            }
        };
        self.states[index] = Some(next);
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
        completion_order: u32,
    ) -> HostTxState {
        let context = retained.context();
        let fields = unsafe { vendor_host_tx::confirmation_fields(context) };
        let tx_rate = fields.tx_rate;
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
            confirmation, ..
        }) = self.states[index]
        else {
            return None;
        };
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
        let HostTxState::Confirming { owner, .. } = self.states[index].take()? else {
            return None;
        };
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
