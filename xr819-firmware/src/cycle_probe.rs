//! Temporary per-batch cycle decomposition probe (counters MIB 0x100c, CYC5).
//!
//! Splits the publish cycle at every boundary the firmware can see, so the
//! per-batch cost can be attributed instead of inferred from throughput:
//!
//! * GO -> first drain (chip airtime plus completion latency for the head);
//! * consecutive drains inside one batch (the per-member inter-drain gap: the
//!   discriminator between airtime and per-member dead time);
//! * last drain -> confirmation (routing, encode, HIF publish);
//! * confirmation -> first admission (host round trip: confirm read, driver
//!   wake, SDIO write, command-lane poll);
//! * last admission -> GO (firmware reservation and publication).
//!
//! Cycle totals are accumulated as GO -> next GO so `cycle` can be regressed on
//! members per batch directly. Pipe 0 carries all measured traffic; other
//! pipes only increment one witness word so an unexpected second pipe cannot
//! hide inside the pipe-0 means.
//!
//! The confirm -> admit span is armed by each confirmation and closed by the
//! next admission. It measures the host response only when the host is not
//! already ahead of the firmware; when the host has pipelined more frames the
//! same batch closes the span immediately, which reads as a fast round trip and
//! is indistinguishable here from a genuinely fast host.
//!
//! MIB words (CYC5 `0x43594335`): 0 marker, 1 loops, 2 batches, 3 GO->GO sum,
//! 4 members, 5 intra-batch drain-gap sum, 6 its count, 7 GO->first-drain sum,
//! 8 last-drain->confirm sum, 9 its count, 10 confirm->admit sum, 11 its count,
//! 12 last-admit->GO sum, 13 its count, 14 other-pipe events, 15 pipe-0 idle
//! starved passes, 16 pipe-0 idle-blocked passes, 17..=21 reserved. This schema
//! supersedes CYC4 words; the features are never enabled in the same build. No
//! timers of its own: the existing vendor microsecond timer is the only clock,
//! and there is no scheduling, ownership or lifecycle change. All words are
//! `u32` and wrap modulo 2^32.

use core::cell::UnsafeCell;

/// Counters MIB schema marker, word 0.
pub const MAGIC: u32 = 0x4359_4335;

/// Per-batch latch state; never exported.
#[derive(Clone, Copy, Default)]
struct BatchState {
    go_stamp: u32,
    last_drain_stamp: u32,
    last_admit_stamp: u32,
    awaiting_admit_stamp: u32,
    go_valid: bool,
    drain_valid: bool,
    admit_valid: bool,
    awaiting_admit: bool,
}

#[derive(Clone, Copy, Default)]
struct Counters {
    loops: u32,
    batches: u32,
    cycle_sum: u32,
    members: u32,
    interdrain_sum: u32,
    interdrain_count: u32,
    go_firstdrain_sum: u32,
    drain_confirm_sum: u32,
    drain_confirm_count: u32,
    confirm_admit_sum: u32,
    confirm_admit_count: u32,
    admit_go_sum: u32,
    admit_go_count: u32,
    other_pipe_events: u32,
    pipe0_idle_starved: u32,
    pipe0_idle_blocked: u32,
    batch: BatchState,
}

struct Shared(UnsafeCell<Counters>);
// SAFETY: only the single cooperative foreground thread touches these counters;
// GO, drain, admission and confirmation hooks all run in that context, never in
// an IRQ.
unsafe impl Sync for Shared {}
static COUNTERS: Shared = Shared(UnsafeCell::new(Counters {
    loops: 0,
    batches: 0,
    cycle_sum: 0,
    members: 0,
    interdrain_sum: 0,
    interdrain_count: 0,
    go_firstdrain_sum: 0,
    drain_confirm_sum: 0,
    drain_confirm_count: 0,
    confirm_admit_sum: 0,
    confirm_admit_count: 0,
    admit_go_sum: 0,
    admit_go_count: 0,
    other_pipe_events: 0,
    pipe0_idle_starved: 0,
    pipe0_idle_blocked: 0,
    batch: BatchState {
        go_stamp: 0,
        last_drain_stamp: 0,
        last_admit_stamp: 0,
        awaiting_admit_stamp: 0,
        go_valid: false,
        drain_valid: false,
        admit_valid: false,
        awaiting_admit: false,
    },
}));

#[inline(always)]
fn counters() -> &'static mut Counters {
    unsafe { &mut *COUNTERS.0.get() }
}

#[inline(always)]
#[cfg(target_arch = "arm")]
fn now() -> u32 {
    unsafe { crate::vendor_host_tx::vendor_timer_now() }
}

#[inline(always)]
#[cfg(not(target_arch = "arm"))]
fn now() -> u32 {
    0
}

/// One cooperative service pass.
#[inline(always)]
pub fn observe_loop() {
    counters().loops = counters().loops.wrapping_add(1);
}

/// A host batch GO triggered on `pipe` at `stamp`. Test-visible core.
#[inline(always)]
pub fn note_go_at(pipe: u8, stamp: u32) {
    if pipe != 0 {
        let counters = counters();
        counters.other_pipe_events = counters.other_pipe_events.wrapping_add(1);
        return;
    }
    let counters = counters();
    // The previous batch is only complete once its successor GOes, so the
    // GO -> GO distance is the cycle the throughput is limited by.
    if counters.batch.go_valid {
        counters.cycle_sum = counters
            .cycle_sum
            .wrapping_add(stamp.wrapping_sub(counters.batch.go_stamp));
        counters.batches = counters.batches.wrapping_add(1);
    }
    // Admission -> GO is the firmware's own staging latency for the members the
    // host supplied since the last trigger.
    if counters.batch.admit_valid {
        counters.admit_go_sum = counters
            .admit_go_sum
            .wrapping_add(stamp.wrapping_sub(counters.batch.last_admit_stamp));
        counters.admit_go_count = counters.admit_go_count.wrapping_add(1);
    }
    counters.batch.go_stamp = stamp;
    counters.batch.go_valid = true;
    counters.batch.drain_valid = false;
    counters.batch.admit_valid = false;
}

/// A hardware completion drained on `pipe` at `stamp`. Test-visible core.
#[inline(always)]
pub fn note_drain_at(pipe: u8, stamp: u32) {
    if pipe != 0 {
        let counters = counters();
        counters.other_pipe_events = counters.other_pipe_events.wrapping_add(1);
        return;
    }
    let counters = counters();
    counters.members = counters.members.wrapping_add(1);
    if counters.batch.drain_valid {
        // Consecutive members of one batch: this gap is where per-member dead
        // time would show up, because the aggregate's members share one GO.
        counters.interdrain_sum = counters
            .interdrain_sum
            .wrapping_add(stamp.wrapping_sub(counters.batch.last_drain_stamp));
        counters.interdrain_count = counters.interdrain_count.wrapping_add(1);
    } else if counters.batch.go_valid {
        counters.go_firstdrain_sum = counters
            .go_firstdrain_sum
            .wrapping_add(stamp.wrapping_sub(counters.batch.go_stamp));
    }
    counters.batch.last_drain_stamp = stamp;
    counters.batch.drain_valid = true;
}

/// A TX confirmation published to the host at `stamp`. Test-visible core.
#[inline(always)]
pub fn note_confirm_at(stamp: u32) {
    let counters = counters();
    if counters.batch.drain_valid {
        counters.drain_confirm_sum = counters
            .drain_confirm_sum
            .wrapping_add(stamp.wrapping_sub(counters.batch.last_drain_stamp));
        counters.drain_confirm_count = counters.drain_confirm_count.wrapping_add(1);
    }
    counters.batch.awaiting_admit = true;
    counters.batch.awaiting_admit_stamp = stamp;
}

/// A TX request admitted from the command lane at `stamp`. Test-visible core.
#[inline(always)]
pub fn note_admit_at(stamp: u32) {
    let counters = counters();
    if counters.batch.awaiting_admit {
        counters.confirm_admit_sum = counters
            .confirm_admit_sum
            .wrapping_add(stamp.wrapping_sub(counters.batch.awaiting_admit_stamp));
        counters.confirm_admit_count = counters.confirm_admit_count.wrapping_add(1);
        counters.batch.awaiting_admit = false;
    }
    counters.batch.last_admit_stamp = stamp;
    counters.batch.admit_valid = true;
}

/// A host batch GO triggered on `pipe` now.
#[inline(always)]
pub fn note_go(pipe: u8) {
    note_go_at(pipe, now());
}

/// A hardware completion drained on `pipe` now.
#[inline(always)]
pub fn note_drain(pipe: u8) {
    note_drain_at(pipe, now());
}

/// A TX confirmation published to the host now.
#[inline(always)]
pub fn note_confirm() {
    note_confirm_at(now());
}

/// A TX request admitted from the command lane now.
#[inline(always)]
pub fn note_admit() {
    note_admit_at(now());
}

/// End-of-pass pipe-0 stall flavor. `hardware_owned` means a software owner
/// still references pipe 0; `work_present` means a PasQueued-but-unowned
/// candidate or a Reserved-but-untriggered batch waits on pipe 0. Busy passes
/// are derived as `loops - starved - blocked`. Test-visible core.
#[inline(always)]
pub fn note_pipe0_pass(hardware_owned: bool, work_present: bool) {
    if hardware_owned {
        return;
    }
    let counters = counters();
    if work_present {
        counters.pipe0_idle_blocked = counters.pipe0_idle_blocked.wrapping_add(1);
    } else {
        counters.pipe0_idle_starved = counters.pipe0_idle_starved.wrapping_add(1);
    }
}

pub fn snapshot() -> [u32; 22] {
    let counters = unsafe { *COUNTERS.0.get() };
    let mut values = [0_u32; 22];
    values[0] = MAGIC;
    values[1] = counters.loops;
    values[2] = counters.batches;
    values[3] = counters.cycle_sum;
    values[4] = counters.members;
    values[5] = counters.interdrain_sum;
    values[6] = counters.interdrain_count;
    values[7] = counters.go_firstdrain_sum;
    values[8] = counters.drain_confirm_sum;
    values[9] = counters.drain_confirm_count;
    values[10] = counters.confirm_admit_sum;
    values[11] = counters.confirm_admit_count;
    values[12] = counters.admit_go_sum;
    values[13] = counters.admit_go_count;
    values[14] = counters.other_pipe_events;
    values[15] = counters.pipe0_idle_starved;
    values[16] = counters.pipe0_idle_blocked;
    values
}

/// Export into the shared MIB word array.
pub fn populate(values: &mut [u32; 22]) {
    *values = snapshot();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reset() {
        unsafe {
            *COUNTERS.0.get() = Counters {
                loops: 0,
                batches: 0,
                cycle_sum: 0,
                members: 0,
                interdrain_sum: 0,
                interdrain_count: 0,
                go_firstdrain_sum: 0,
                drain_confirm_sum: 0,
                drain_confirm_count: 0,
                confirm_admit_sum: 0,
                confirm_admit_count: 0,
                admit_go_sum: 0,
                admit_go_count: 0,
                other_pipe_events: 0,
                pipe0_idle_starved: 0,
                pipe0_idle_blocked: 0,
                batch: BatchState {
                    go_stamp: 0,
                    last_drain_stamp: 0,
                    last_admit_stamp: 0,
                    awaiting_admit_stamp: 0,
                    go_valid: false,
                    drain_valid: false,
                    admit_valid: false,
                    awaiting_admit: false,
                },
            };
        }
    }

    #[test]
    fn full_cycle_splits_at_every_boundary() {
        reset();
        note_go_at(0, 1_000);
        note_drain_at(0, 3_000);
        note_drain_at(0, 4_000);
        note_confirm_at(4_500);
        note_admit_at(6_000);
        note_go_at(0, 11_000);
        let values = snapshot();
        assert_eq!(values[0], MAGIC);
        assert_eq!(values[2], 1); // one completed cycle
        assert_eq!(values[3], 10_000); // GO -> GO
        assert_eq!(values[4], 2); // members
        assert_eq!(values[5], 1_000); // first -> second drain
        assert_eq!(values[6], 1);
        assert_eq!(values[7], 2_000); // GO -> first drain
        assert_eq!(values[8], 500); // last drain -> confirm
        assert_eq!(values[9], 1);
        assert_eq!(values[10], 1_500); // confirm -> admit
        assert_eq!(values[11], 1);
        assert_eq!(values[12], 5_000); // last admit -> GO
        assert_eq!(values[13], 1);
    }

    #[test]
    fn single_member_batch_has_no_interdrain_gap() {
        reset();
        note_go_at(0, 100);
        note_drain_at(0, 900);
        note_confirm_at(1_000);
        note_go_at(0, 2_000);
        let values = snapshot();
        assert_eq!(values[5], 0);
        assert_eq!(values[6], 0);
        assert_eq!(values[7], 800);
        assert_eq!(values[4], 1);
        assert_eq!(values[2], 1);
    }

    #[test]
    fn first_go_does_not_open_a_cycle() {
        reset();
        note_go_at(0, 500);
        note_drain_at(0, 900);
        let values = snapshot();
        assert_eq!(values[2], 0); // no predecessor GO
        assert_eq!(values[3], 0);
        assert_eq!(values[7], 400); // but the batch span is still measured
    }

    #[test]
    fn confirm_admit_span_closes_once_per_confirmation() {
        reset();
        note_go_at(0, 100);
        note_drain_at(0, 200);
        note_confirm_at(300);
        note_admit_at(400);
        note_admit_at(500);
        note_admit_at(600);
        let values = snapshot();
        assert_eq!(values[10], 100); // 300 -> 400 only
        assert_eq!(values[11], 1);
        assert_eq!(values[12], 0); // no GO yet
    }

    #[test]
    fn other_pipes_only_touch_the_witness_word() {
        reset();
        note_go_at(0, 100);
        note_drain_at(1, 150);
        note_go_at(2, 160);
        note_drain_at(0, 200);
        let values = snapshot();
        assert_eq!(values[14], 2);
        assert_eq!(values[4], 1);
        assert_eq!(values[7], 100);
        assert_eq!(values[2], 0);
    }

    #[test]
    fn timer_wrap_uses_wrapping_arithmetic() {
        reset();
        note_go_at(0, u32::MAX - 100);
        note_drain_at(0, 200);
        note_confirm_at(300);
        note_admit_at(400);
        note_go_at(0, 500);
        let values = snapshot();
        assert_eq!(values[3], 601); // GO -> GO across the wrap
        assert_eq!(values[7], 301); // GO -> first drain across the wrap
    }

    #[test]
    fn pipe0_partition_covers_every_idle_pass() {
        reset();
        observe_loop();
        note_pipe0_pass(false, false);
        observe_loop();
        note_pipe0_pass(false, true);
        observe_loop();
        note_pipe0_pass(true, false);
        let values = snapshot();
        assert_eq!(values[1], 3);
        assert_eq!(values[15] + values[16], 2);
    }
}
