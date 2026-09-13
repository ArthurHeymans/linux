//! Temporary post-GO TX-start decomposition probe (MIB 0x100c, CYC11).
//!
//! CYC9/CYC10 plus a monitor capture left one unexplained term: ~0.8-1.0 ms
//! between the firmware's GO register write and the first frame on air. The
//! static comparison of the vendor TX-start path found no missing early PHY
//! start, and identified the only post-GO firmware steps: the MAC raises a
//! type-`0x37` phase-2 event, we service it in a cooperative pass, and the
//! handler runs `txp_pipe_tx_start`, which dispatches PHY command 2 when the
//! retained operation state is 3.
//!
//! CYC11 timestamps exactly those boundaries, so one run splits the term:
//!
//! * GO -> phase-2 dequeue: MAC-side latency before it even tells us to start;
//! * phase-2 dequeue -> first completion: handler, command 2, air, report;
//! * command-2 entry -> exit and its event count, plus the retained operation
//!   state seen at TX-start (3 means command 2 runs this batch, 5 means it is
//!   skipped).
//!
//! MIB words (CYC11 `0x43594342`): 0 marker, 1 loops, 2 batches, 3 members,
//! 4 GO->first-drain sum, 5 GO->phase-2 sum, 6 its count, 7 phase-2->first-drain
//! sum, 8 its count, 9 command-2 events, 10 command-2 duration sum, 11 its
//! count, 12 TX-start events, 13 TX-starts with operation state 3, 14 with
//! state 5, 15 TX-start->first-drain sum, 16 its count, 17 pipe-0 idle-starved
//! passes, 18 pipe-0 idle-blocked passes, 19..=21 reserved. This schema
//! supersedes CYC9/CYC10 words. No timers of its own: the existing vendor
//! microsecond counter is the only clock.

use core::cell::UnsafeCell;

/// Counters MIB schema marker, word 0.
pub const MAGIC: u32 = 0x4359_4342;

#[derive(Clone, Copy, Default)]
struct BatchState {
    go_stamp: u32,
    phase2_stamp: u32,
    txstart_stamp: u32,
    go_valid: bool,
    phase2_valid: bool,
    txstart_valid: bool,
    drain_valid: bool,
}

#[derive(Clone, Copy, Default)]
struct Counters {
    loops: u32,
    batches: u32,
    members: u32,
    go_firstdrain_sum: u32,
    go_phase2_sum: u32,
    go_phase2_count: u32,
    phase2_firstdrain_sum: u32,
    phase2_firstdrain_count: u32,
    cmd2_events: u32,
    cmd2_duration_sum: u32,
    cmd2_duration_count: u32,
    txstart_events: u32,
    txstart_state3: u32,
    txstart_state5: u32,
    txstart_firstdrain_sum: u32,
    txstart_firstdrain_count: u32,
    pipe0_idle_starved: u32,
    pipe0_idle_blocked: u32,
    batch: BatchState,
}

struct Shared(UnsafeCell<Counters>);
// SAFETY: only the single cooperative foreground thread touches these counters;
// every hook runs in that context, never in an IRQ.
unsafe impl Sync for Shared {}
static COUNTERS: Shared = Shared(UnsafeCell::new(Counters {
    loops: 0,
    batches: 0,
    members: 0,
    go_firstdrain_sum: 0,
    go_phase2_sum: 0,
    go_phase2_count: 0,
    phase2_firstdrain_sum: 0,
    phase2_firstdrain_count: 0,
    cmd2_events: 0,
    cmd2_duration_sum: 0,
    cmd2_duration_count: 0,
    txstart_events: 0,
    txstart_state3: 0,
    txstart_state5: 0,
    txstart_firstdrain_sum: 0,
    txstart_firstdrain_count: 0,
    pipe0_idle_starved: 0,
    pipe0_idle_blocked: 0,
    batch: BatchState {
        go_stamp: 0,
        phase2_stamp: 0,
        txstart_stamp: 0,
        go_valid: false,
        phase2_valid: false,
        txstart_valid: false,
        drain_valid: false,
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

/// End-of-pass pipe-0 stall flavor (retained from the earlier probes).
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

/// A host batch GO triggered on `pipe` at `stamp`. Test-visible core.
#[inline(always)]
pub fn note_go_at(pipe: u8, stamp: u32) {
    if pipe != 0 {
        return;
    }
    let counters = counters();
    if counters.batch.go_valid {
        counters.batches = counters.batches.wrapping_add(1);
    }
    counters.batch.go_stamp = stamp;
    counters.batch.go_valid = true;
    counters.batch.phase2_valid = false;
    counters.batch.txstart_valid = false;
    counters.batch.drain_valid = false;
}

/// The MAC's type-`0x37` phase-2 event for `pipe` was dequeued at `stamp`.
#[inline(always)]
pub fn note_phase2_at(pipe: u8, stamp: u32) {
    if pipe != 0 {
        return;
    }
    let counters = counters();
    if !counters.batch.go_valid || counters.batch.phase2_valid {
        return;
    }
    counters.go_phase2_sum = counters
        .go_phase2_sum
        .wrapping_add(stamp.wrapping_sub(counters.batch.go_stamp));
    counters.go_phase2_count = counters.go_phase2_count.wrapping_add(1);
    counters.batch.phase2_stamp = stamp;
    counters.batch.phase2_valid = true;
}

/// `txp_pipe_tx_start` was entered at `stamp`, with the retained PHY operation
/// state the handler will branch on.
#[inline(always)]
pub fn note_tx_start_at(stamp: u32, operation_state: u32) {
    let counters = counters();
    counters.txstart_events = counters.txstart_events.wrapping_add(1);
    match operation_state {
        3 => counters.txstart_state3 = counters.txstart_state3.wrapping_add(1),
        5 => counters.txstart_state5 = counters.txstart_state5.wrapping_add(1),
        _ => {}
    }
    counters.batch.txstart_stamp = stamp;
    counters.batch.txstart_valid = true;
}

/// PHY command 2 was dispatched for `duration` microseconds.
#[inline(always)]
pub fn note_command2(duration: u32) {
    let counters = counters();
    counters.cmd2_events = counters.cmd2_events.wrapping_add(1);
    counters.cmd2_duration_sum = counters.cmd2_duration_sum.wrapping_add(duration);
    counters.cmd2_duration_count = counters.cmd2_duration_count.wrapping_add(1);
}

/// A hardware completion drained on `pipe` at `stamp`. Test-visible core.
#[inline(always)]
pub fn note_drain_at(pipe: u8, stamp: u32) {
    if pipe != 0 {
        return;
    }
    let counters = counters();
    counters.members = counters.members.wrapping_add(1);
    if counters.batch.drain_valid || !counters.batch.go_valid {
        return;
    }
    counters.go_firstdrain_sum = counters
        .go_firstdrain_sum
        .wrapping_add(stamp.wrapping_sub(counters.batch.go_stamp));
    if counters.batch.phase2_valid {
        counters.phase2_firstdrain_sum = counters
            .phase2_firstdrain_sum
            .wrapping_add(stamp.wrapping_sub(counters.batch.phase2_stamp));
        counters.phase2_firstdrain_count = counters.phase2_firstdrain_count.wrapping_add(1);
    }
    if counters.batch.txstart_valid {
        counters.txstart_firstdrain_sum = counters
            .txstart_firstdrain_sum
            .wrapping_add(stamp.wrapping_sub(counters.batch.txstart_stamp));
        counters.txstart_firstdrain_count = counters.txstart_firstdrain_count.wrapping_add(1);
    }
    counters.batch.drain_valid = true;
}

/// A host batch GO triggered on `pipe` now.
#[inline(always)]
pub fn note_go(pipe: u8) {
    note_go_at(pipe, now());
}

/// The MAC phase-2 start event for `pipe` was dequeued now.
#[inline(always)]
pub fn note_phase2(pipe: u8) {
    note_phase2_at(pipe, now());
}

/// `txp_pipe_tx_start` was entered now.
#[inline(always)]
pub fn note_tx_start(operation_state: u32) {
    note_tx_start_at(now(), operation_state);
}

/// PHY command 2 is about to be dispatched; returns the entry stamp.
#[inline(always)]
pub fn command2_enter() -> u32 {
    now()
}

/// PHY command 2 finished; records its duration.
#[inline(always)]
pub fn command2_exit(enter_stamp: u32) {
    note_command2(now().wrapping_sub(enter_stamp));
}

/// A hardware completion drained on `pipe` now.
#[inline(always)]
pub fn note_drain(pipe: u8) {
    note_drain_at(pipe, now());
}

/// Confirmation and admission hooks are retained for wiring parity.
#[inline(always)]
pub fn note_confirm() {}

#[inline(always)]
pub fn note_admit() {}

pub fn snapshot() -> [u32; 22] {
    let counters = unsafe { *COUNTERS.0.get() };
    let mut values = [0_u32; 22];
    values[0] = MAGIC;
    values[1] = counters.loops;
    values[2] = counters.batches;
    values[3] = counters.members;
    values[4] = counters.go_firstdrain_sum;
    values[5] = counters.go_phase2_sum;
    values[6] = counters.go_phase2_count;
    values[7] = counters.phase2_firstdrain_sum;
    values[8] = counters.phase2_firstdrain_count;
    values[9] = counters.cmd2_events;
    values[10] = counters.cmd2_duration_sum;
    values[11] = counters.cmd2_duration_count;
    values[12] = counters.txstart_events;
    values[13] = counters.txstart_state3;
    values[14] = counters.txstart_state5;
    values[15] = counters.txstart_firstdrain_sum;
    values[16] = counters.txstart_firstdrain_count;
    values[17] = counters.pipe0_idle_starved;
    values[18] = counters.pipe0_idle_blocked;
    values
}

/// Export into the shared MIB word array.
pub fn populate(values: &mut [u32; 22]) {
    *values = snapshot();
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::sync::atomic::{AtomicBool, Ordering};

    /// The probe counters are one global and cargo runs tests in parallel, so
    /// every test that resets and reads them must hold this guard.
    static TEST_LOCK: AtomicBool = AtomicBool::new(false);

    struct Guard;

    impl Guard {
        fn acquire() -> Self {
            while TEST_LOCK
                .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
                .is_err()
            {
                core::hint::spin_loop();
            }
            Guard
        }
    }

    impl Drop for Guard {
        fn drop(&mut self) {
            TEST_LOCK.store(false, Ordering::Release);
        }
    }

    fn reset() {
        unsafe {
            *COUNTERS.0.get() = Counters::default();
        }
    }

    #[test]
    fn the_post_go_chain_is_split_at_every_boundary() {
        let _guard = Guard::acquire();
        reset();
        note_go_at(0, 1_000);
        note_phase2_at(0, 1_500); // 500 us from the GO
        note_tx_start_at(1_550, 3);
        note_command2(120);
        note_drain_at(0, 3_000);
        note_go_at(0, 10_000);
        let values = snapshot();
        assert_eq!(values[0], MAGIC);
        assert_eq!(values[2], 1); // one completed batch
        assert_eq!(values[4], 2_000); // GO -> first drain
        assert_eq!(values[5], 500); // GO -> phase 2
        assert_eq!(values[6], 1);
        assert_eq!(values[7], 1_500); // phase 2 -> first drain
        assert_eq!(values[9], 1); // command 2 dispatched
        assert_eq!(values[10], 120);
        assert_eq!(values[12], 1);
        assert_eq!(values[13], 1); // state 3
        assert_eq!(values[15], 1_450); // TX-start -> first drain
    }

    #[test]
    fn state_five_batches_skip_command_two() {
        let _guard = Guard::acquire();
        reset();
        note_go_at(0, 0);
        note_phase2_at(0, 100);
        note_tx_start_at(120, 5);
        note_drain_at(0, 900);
        let values = snapshot();
        assert_eq!(values[9], 0); // no command 2
        assert_eq!(values[14], 1); // state 5 seen
        assert_eq!(values[13], 0);
    }

    #[test]
    fn a_second_phase2_in_one_batch_does_not_recount() {
        let _guard = Guard::acquire();
        reset();
        note_go_at(0, 0);
        note_phase2_at(0, 100);
        note_phase2_at(0, 200);
        let values = snapshot();
        assert_eq!(values[6], 1);
        assert_eq!(values[5], 100);
    }

    #[test]
    fn other_pipes_are_ignored() {
        let _guard = Guard::acquire();
        reset();
        note_go_at(0, 100);
        note_drain_at(1, 150);
        note_drain_at(0, 200);
        let values = snapshot();
        assert_eq!(values[3], 1);
        assert_eq!(values[4], 100);
    }
}
