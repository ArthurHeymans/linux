//! Single-outstanding ordinary host-TX ownership driver.
//!
//! The state enum makes retained-context, scheduler-reservation, completion,
//! and confirmation ownership mutually exclusive. This is the executable
//! boundary that serializes the shared MAC event backend with management TX.

use crate::{hif, host_tx_diagnostics, tx, vendor_host_tx};

pub struct HostTxDriver {
    state: HostTxState,
}

enum HostTxState {
    Idle,
    Owned {
        retained: vendor_host_tx::RetainedHostTx,
        wait_diagnostic: u8,
    },
    Reserved {
        retained: vendor_host_tx::RetainedHostTx,
        reservation: vendor_host_tx::HostSchedulerReservation,
        wait_diagnostic: u8,
    },
    Confirming {
        retained: vendor_host_tx::RetainedHostTx,
        status: u32,
        ack_failures: u8,
    },
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
            state: HostTxState::Idle,
        }
    }

    pub const fn management_runtime_available(&self) -> bool {
        matches!(self.state, HostTxState::Idle)
    }

    /// Admit, classify, encrypt, and queue one ordinary host request.
    ///
    /// Failure paths return the packet-RAM buffer to the HIF ring before
    /// returning, so ownership never disappears through an error value.
    ///
    /// # Safety
    /// The caller must serialize host-context and HIF-ring mutation.
    pub unsafe fn admit(
        &mut self,
        buffer: hif::RequestBuffer,
        interface: u8,
        transport: &mut hif::Transport,
    ) -> bool {
        if !matches!(self.state, HostTxState::Idle) {
            transport.release_request(buffer.into_release());
            return false;
        }
        let mut retained = match unsafe { vendor_host_tx::admit_host_tx(buffer, interface) } {
            Ok(retained) => retained,
            Err((buffer, _)) => {
                transport.release_request(buffer.into_release());
                return false;
            }
        };
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
        self.state = HostTxState::Owned {
            retained,
            wait_diagnostic: 0,
        };
        true
    }

    /// Advance the bounded lifecycle by one cooperative main-loop pass.
    ///
    /// The optional result is a host-visible diagnostic event. Retained trace
    /// publication remains internal to the diagnostics module.
    ///
    /// # Safety
    /// This driver must be the sole class-0 owner of the MAC backend.
    pub unsafe fn service(
        &mut self,
        events: &mut tx::MacEventQueue,
        allow_debug_event: bool,
    ) -> Option<(u32, u32)> {
        let state = core::mem::replace(&mut self.state, HostTxState::Idle);
        self.state = match state {
            HostTxState::Idle => HostTxState::Idle,
            HostTxState::Confirming {
                retained,
                status,
                ack_failures,
            } => HostTxState::Confirming {
                retained,
                status,
                ack_failures,
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
                        }
                        HostTxState::Owned {
                            retained,
                            wait_diagnostic: 3,
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
                            unsafe {
                                host_tx_diagnostics::trace(
                                    0x4854_1100 | u32::from(status),
                                    retained.context().raw(),
                                    0,
                                );
                            }
                            let _ = retained.transition(vendor_host_tx::HostTxPhase::Completing);
                            return self.replace_with_confirmation(
                                retained,
                                tx::wsm_status_from_internal(status),
                                0,
                            );
                        }
                        Ok(vendor_host_tx::PendingServiceReport::PasQueued) => {
                            unsafe {
                                host_tx_diagnostics::trace(
                                    0x4854_1200,
                                    retained.context().raw(),
                                    0,
                                );
                            }
                            wait_diagnostic = 0;
                        }
                        Ok(vendor_host_tx::PendingServiceReport::LeaveQueued) => {
                            if cfg!(feature = "vendor-host-tx-diagnostics")
                                && allow_debug_event
                                && wait_diagnostic == 0
                            {
                                let diagnostic =
                                    unsafe { vendor_host_tx::pending_live_diagnostic(&retained) };
                                unsafe {
                                    host_tx_diagnostics::trace(
                                        0x4854_1300,
                                        retained.context().raw(),
                                        0,
                                    );
                                }
                                let event = (
                                    0x4854_6000
                                        | u32::from(diagnostic.vif_state)
                                        | (u32::from(diagnostic.pipe_allowed) << 8)
                                        | ((diagnostic.global & 0xff) << 16),
                                    u32::from(diagnostic.active_mask)
                                        | (u32::from(diagnostic.effective_mask) << 16),
                                );
                                wait_diagnostic = 1;
                                self.state = HostTxState::Owned {
                                    retained,
                                    wait_diagnostic,
                                };
                                return Some(event);
                            }
                        }
                        Err(error) => {
                            unsafe {
                                host_tx_diagnostics::trace(
                                    0x4854_1f00 | u32::from(error.diagnostic_code()),
                                    retained.context().raw(),
                                    u32::from(retained.phase() as u8),
                                );
                            }
                            if cfg!(feature = "vendor-host-tx-diagnostics") && allow_debug_event {
                                let event = (
                                    0x4854_5f00 | u32::from(error.diagnostic_code()),
                                    retained.context().raw()
                                        | (u32::from(retained.phase() as u8) << 24),
                                );
                                self.state = HostTxState::Owned {
                                    retained,
                                    wait_diagnostic,
                                };
                                return Some(event);
                            }
                        }
                    }
                }

                if retained.phase() == vendor_host_tx::HostTxPhase::PasQueued {
                    match unsafe { vendor_host_tx::reserve_non_aggregate_scheduler(&mut retained) }
                    {
                        Ok(reservation) => {
                            unsafe {
                                host_tx_diagnostics::trace(
                                    0x4854_2000,
                                    retained.context().raw(),
                                    0,
                                );
                            }
                            return self.replace_with_reserved(retained, reservation);
                        }
                        Err(vendor_host_tx::SchedulerReserveError::Expired) => {
                            if unsafe { vendor_host_tx::reject_unscheduled_pas(&mut retained) }
                                .is_ok()
                            {
                                let _ =
                                    retained.transition(vendor_host_tx::HostTxPhase::Completing);
                                return self.replace_with_confirmation(
                                    retained,
                                    tx::wsm_status_from_internal(10),
                                    0,
                                );
                            }
                        }
                        Err(error) => {
                            unsafe {
                                host_tx_diagnostics::trace(
                                    0x4854_2f00 | u32::from(error.diagnostic_code()),
                                    retained.context().raw(),
                                    0,
                                );
                            }
                            if cfg!(feature = "vendor-host-tx-diagnostics")
                                && allow_debug_event
                                && wait_diagnostic < 2
                            {
                                let diagnostic =
                                    unsafe { vendor_host_tx::scheduler_live_diagnostic(&retained) };
                                let gates = u32::from(diagnostic.pipe)
                                    | (u32::from(diagnostic.idle_pipe_mask) << 8)
                                    | (u32::from(diagnostic.retry_gate) << 16)
                                    | (u32::from(diagnostic.receive_gate) << 24);
                                let flags = u32::from(diagnostic.ring_contains_frame)
                                    | (u32::from(diagnostic.pipe_allowed) << 1)
                                    | (u32::from(diagnostic.ring_head) << 2)
                                    | (u32::from(diagnostic.ring_tail) << 8);
                                wait_diagnostic += 1;
                                let event = (
                                    0x4854_5810 | u32::from(error.diagnostic_code()) | (flags << 8),
                                    gates,
                                );
                                self.state = HostTxState::Owned {
                                    retained,
                                    wait_diagnostic,
                                };
                                return Some(event);
                            }
                        }
                    }
                }

                if retained.phase() == vendor_host_tx::HostTxPhase::Scheduled
                    && let Some((context, status, ack_failures)) =
                        unsafe { tx::service_host_class0_runtime(events, 32) }
                {
                    unsafe {
                        host_tx_diagnostics::capture_completion(context, status, ack_failures);
                    }
                    if context == retained.context().raw() {
                        let _ = retained.transition(vendor_host_tx::HostTxPhase::Completing);
                        return self.replace_with_confirmation(
                            retained,
                            tx::wsm_status_from_internal(status),
                            ack_failures,
                        );
                    }
                    unsafe {
                        host_tx_diagnostics::trace(0x4854_3f00, context, retained.context().raw());
                    }
                }

                HostTxState::Owned {
                    retained,
                    wait_diagnostic,
                }
            }
        };
        None
    }

    fn replace_with_reserved(
        &mut self,
        retained: vendor_host_tx::RetainedHostTx,
        reservation: vendor_host_tx::HostSchedulerReservation,
    ) -> Option<(u32, u32)> {
        self.state = HostTxState::Reserved {
            retained,
            reservation,
            wait_diagnostic: 0,
        };
        None
    }

    fn replace_with_confirmation(
        &mut self,
        retained: vendor_host_tx::RetainedHostTx,
        status: u32,
        ack_failures: u8,
    ) -> Option<(u32, u32)> {
        self.state = HostTxState::Confirming {
            retained,
            status,
            ack_failures,
        };
        None
    }

    pub fn confirmation(&self) -> Option<HostTxConfirmation> {
        match &self.state {
            HostTxState::Confirming {
                retained,
                status,
                ack_failures,
            } => {
                let context = retained.context().raw();
                let tx_rate = unsafe { (context.wrapping_add(0x63) as *const u8).read_volatile() };
                unsafe {
                    host_tx_diagnostics::capture_retry_feedback(
                        context,
                        *status,
                        tx_rate,
                        *ack_failures,
                    );
                }
                Some(HostTxConfirmation {
                    packet_id: retained.packet_id(),
                    context,
                    status: *status,
                    tx_rate,
                    ack_failures: *ack_failures,
                    rate_try: unsafe {
                        [
                            (context.wrapping_add(0x28) as *const u32).read_volatile(),
                            (context.wrapping_add(0x2c) as *const u32).read_volatile(),
                            (context.wrapping_add(0x30) as *const u32).read_volatile(),
                        ]
                    },
                })
            }
            _ => None,
        }
    }

    /// Finish a confirmation only after its HIF descriptor was published.
    ///
    /// # Safety
    /// Completion accounting must already have removed hardware ownership.
    pub unsafe fn finish_confirmation(&mut self) -> Option<hif::RequestReleaseToken> {
        let state = core::mem::replace(&mut self.state, HostTxState::Idle);
        match state {
            HostTxState::Confirming { retained, .. } => Some(unsafe { retained.finish() }),
            state => {
                self.state = state;
                None
            }
        }
    }

    /// Cancel every reversible state while preserving scheduled hardware
    /// ownership until its normal completion.
    ///
    /// # Safety
    /// The caller must serialize scheduler, pending-list, and HIF mutation.
    pub unsafe fn reset(&mut self) -> Option<hif::RequestReleaseToken> {
        let state = core::mem::replace(&mut self.state, HostTxState::Idle);
        match state {
            HostTxState::Idle => None,
            HostTxState::Confirming { retained, .. } => Some(unsafe { retained.finish() }),
            HostTxState::Reserved {
                mut retained,
                reservation,
                ..
            } => {
                let _ = unsafe { reservation.cancel(&mut retained) };
                unsafe { retained.cancel_before_pas().ok() }
            }
            HostTxState::Owned { retained, .. }
                if retained.phase() != vendor_host_tx::HostTxPhase::Scheduled =>
            unsafe { retained.cancel_before_pas().ok() },
            HostTxState::Owned {
                retained,
                wait_diagnostic,
            } => {
                self.state = HostTxState::Owned {
                    retained,
                    wait_diagnostic,
                };
                None
            }
        }
    }
}

impl Default for HostTxDriver {
    fn default() -> Self {
        Self::new()
    }
}
