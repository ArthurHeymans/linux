//! Temporary cooperative-service probe (counters MIB 0x100c, words 0..=10).
//!
//! Counts main-loop passes, TX hardware ownership and completions, and the RX
//! admission boundary: published indications, drops at the 24 host-transfer cap,
//! passes blocked on output descriptors, and passes where the RX FIFO is nearly
//! full. It exists to localize the downlink collapse: the AP records failed
//! transmissions against this station, so the board is not acknowledging frames
//! the AP sends.
//!
//! No timers, no scheduling change, and no lifecycle change. The RX counters are
//! fed from the `experimental-rx-path-diagnostics` observation points, so a
//! probe build enables both features.

use core::cell::UnsafeCell;

/// Counters MIB schema marker, word 0.
pub const MAGIC: u32 = 0x5258_5031;

#[derive(Clone, Copy, Default)]
pub struct Counters {
    pub passes: u32,
    pub busy_passes: u32,
    pub completions: u32,
    pub rx_indications: u32,
    pub rx_host_transfer_drops: u32,
    pub rx_blocked_by_request: u32,
    pub rx_blocked_by_control: u32,
    pub rx_blocked_by_descriptor: u32,
    pub rx_near_full_passes: u32,
    pub rx_host_transfers_max: u32,
    pub rx_pending_bytes_max: u32,
    pub rx_pending_passes: u32,
}

struct Shared(UnsafeCell<Counters>);
// SAFETY: only the single cooperative foreground thread touches these counters;
// no interrupt handler calls into this module.
unsafe impl Sync for Shared {}
static COUNTERS: Shared = Shared(UnsafeCell::new(Counters {
    passes: 0,
    busy_passes: 0,
    completions: 0,
    rx_indications: 0,
    rx_host_transfer_drops: 0,
    rx_blocked_by_request: 0,
    rx_blocked_by_control: 0,
    rx_blocked_by_descriptor: 0,
    rx_near_full_passes: 0,
    rx_host_transfers_max: 0,
    rx_pending_bytes_max: 0,
    rx_pending_passes: 0,
}));

#[inline(always)]
fn counters() -> &'static mut Counters {
    unsafe { &mut *COUNTERS.0.get() }
}

/// One cooperative service pass. `busy` is true when at least one class-0 slot
/// was hardware-owned at the start of the pass.
#[inline(always)]
pub fn observe_pass(busy: bool) {
    let counters = counters();
    counters.passes = counters.passes.wrapping_add(1);
    if busy {
        counters.busy_passes = counters.busy_passes.wrapping_add(1);
    }
}

/// One completion drained from the class-0 MAC executor.
#[inline(always)]
pub fn note_completion() {
    let counters = counters();
    counters.completions = counters.completions.wrapping_add(1);
}

/// One receive indication published to the host.
#[inline(always)]
pub fn note_rx_indication() {
    let counters = counters();
    counters.rx_indications = counters.rx_indications.wrapping_add(1);
}

/// One frame released without publication because the host-transfer budget was
/// exhausted.
#[inline(always)]
pub fn note_rx_host_transfer_drop() {
    let counters = counters();
    counters.rx_host_transfer_drops = counters.rx_host_transfer_drops.wrapping_add(1);
}

/// One observation of the joined-RX admission boundary. Mirrors the existing
/// rx-path-diagnostics attribution exactly, including the call site that
/// hardcodes `publication_available = false` because a host request or control
/// indication is already pending.
#[inline(always)]
pub fn observe_rx_pending(
    pending_bytes: u32,
    host_request_waiting: bool,
    control_pending: bool,
    publication_available: bool,
    host_transfers: u32,
    near_full: bool,
) {
    let counters = counters();
    counters.rx_host_transfers_max = counters.rx_host_transfers_max.max(host_transfers);
    counters.rx_pending_bytes_max = counters.rx_pending_bytes_max.max(pending_bytes);
    if pending_bytes == 0 {
        return;
    }
    counters.rx_pending_passes = counters.rx_pending_passes.wrapping_add(1);
    if host_request_waiting {
        counters.rx_blocked_by_request = counters.rx_blocked_by_request.wrapping_add(1);
    } else if control_pending {
        counters.rx_blocked_by_control = counters.rx_blocked_by_control.wrapping_add(1);
    } else if !publication_available {
        counters.rx_blocked_by_descriptor = counters.rx_blocked_by_descriptor.wrapping_add(1);
    }
    if near_full {
        counters.rx_near_full_passes = counters.rx_near_full_passes.wrapping_add(1);
    }
}

pub fn snapshot() -> Counters {
    unsafe { *COUNTERS.0.get() }
}

/// Words 0..=12. Words 13..=21 keep the existing RX-path diagnostic meanings.
pub fn populate(values: &mut [u32; 22]) {
    let counters = snapshot();
    values[0] = MAGIC;
    values[1] = counters.passes;
    values[2] = counters.busy_passes;
    values[3] = counters.completions;
    values[4] = counters.rx_indications;
    values[5] = counters.rx_host_transfer_drops;
    values[6] = counters.rx_blocked_by_request;
    values[7] = counters.rx_blocked_by_control;
    values[8] = counters.rx_blocked_by_descriptor;
    values[9] = counters.rx_host_transfers_max;
    values[10] = counters.rx_pending_bytes_max;
    values[11] = counters.rx_near_full_passes;
    values[12] = counters.rx_pending_passes;
}
