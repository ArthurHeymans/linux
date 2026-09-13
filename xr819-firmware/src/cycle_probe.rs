//! Temporary per-batch cycle probe bucketed by batch depth (MIB 0x100c, CYC9).
//!
//! CYC8 refuted the cooperative-loop stall hypothesis: during traffic no pass
//! interval exceeds 1 ms, so the ~1.2 ms excess a supply-clean batch carries
//! between GO and its first completion is not a scheduling artifact. What is
//! left is either a per-batch fixed cost or a per-member cost, and the two can
//! be separated from the within-run spread of batch depth instead of from a
//! rate sweep: CYC9 accumulates GO->first-drain and its batch count per member
//! count (1..4), so a least-squares fit over depth gives the intercept and the
//! per-member slope directly. Supply class and loop regularity are retained.
//!
//! MIB words (CYC8 `0x43594338`): 0 marker, 1 loops, 2 batches, 3 GO->GO sum,
//! 4 members, 5 GO->first-drain sum, 6 clean cycle sum, 7 clean GO->first-drain
//! sum, 8 clean batches, 9 starved cycle sum, 10 starved GO->first-drain sum,
//! 11 starved batches, 12 loop-interval sum, 13 loop-interval count,
//! 14 max loop interval, 15..=18 loop-interval histogram (<=250, <=500, <=1000,
//! >1000 us), 19 pipe-0 idle-starved passes, 20 pipe-0 idle-blocked passes,
//! > 21 reserved. This schema supersedes CYC7 words. No timers of its own beyond
//! > reading the existing vendor microsecond counter, and no scheduling,
//! > ownership or lifecycle change.

use core::cell::UnsafeCell;

/// Counters MIB schema marker, word 0.
pub const MAGIC: u32 = 0x4359_4339;

#[derive(Clone, Copy, Default)]
struct BatchState {
    go_stamp: u32,
    starved_passes: u32,
    members: u32,
    firstdrain_span: u32,
    go_valid: bool,
    drain_valid: bool,
    /// Starvation seen before the current batch's GO; classifies the batch.
    current_starved: bool,
    /// Classification of the batch whose cycle is still open.
    open_starved: bool,
}

#[derive(Clone, Copy, Default)]
struct Counters {
    loops: u32,
    batches: u32,
    cycle_sum: u32,
    members: u32,
    go_firstdrain_sum: u32,
    clean_batches: u32,
    starved_batches: u32,
    clean_cycle_sum: u32,
    starved_cycle_sum: u32,
    depth_batches: [u32; 4],
    depth_firstdrain: [u32; 4],
    loop_delta_sum: u32,
    loop_delta_count: u32,
    loop_slow: u32,
    pipe0_idle_starved: u32,
    pipe0_idle_blocked: u32,
    last_loop_stamp: u32,
    loop_stamp_valid: bool,
    batch: BatchState,
}

struct Shared(UnsafeCell<Counters>);
// SAFETY: only the single cooperative foreground thread touches these counters;
// every hook runs in that context, never in an IRQ.
unsafe impl Sync for Shared {}
static COUNTERS: Shared = Shared(UnsafeCell::new(Counters {
    loops: 0,
    batches: 0,
    cycle_sum: 0,
    members: 0,
    go_firstdrain_sum: 0,
    clean_batches: 0,
    starved_batches: 0,
    clean_cycle_sum: 0,
    starved_cycle_sum: 0,
    depth_batches: [0; 4],
    depth_firstdrain: [0; 4],
    loop_delta_sum: 0,
    loop_delta_count: 0,
    loop_slow: 0,
    pipe0_idle_starved: 0,
    pipe0_idle_blocked: 0,
    last_loop_stamp: 0,
    loop_stamp_valid: false,
    batch: BatchState {
        go_stamp: 0,
        starved_passes: 0,
        members: 0,
        firstdrain_span: 0,
        go_valid: false,
        drain_valid: false,
        current_starved: false,
        open_starved: false,
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

/// Bin index for `value` against ascending bounds; the last bin is the tail.
const fn bin(bounds: &[u32], value: u32) -> usize {
    let mut index = 0;
    while index < bounds.len() {
        if value <= bounds[index] {
            return index;
        }
        index += 1;
    }
    bounds.len()
}

/// One cooperative service pass, timestamped so the loop's own regularity is
/// accounted rather than assumed. Test-visible core.
#[inline(always)]
pub fn observe_loop_at(stamp: u32) {
    let counters = counters();
    counters.loops = counters.loops.wrapping_add(1);
    if counters.loop_stamp_valid {
        let delta = stamp.wrapping_sub(counters.last_loop_stamp);
        counters.loop_delta_sum = counters.loop_delta_sum.wrapping_add(delta);
        counters.loop_delta_count = counters.loop_delta_count.wrapping_add(1);
        if delta > 1_000 {
            counters.loop_slow = counters.loop_slow.wrapping_add(1);
        }
    }
    counters.last_loop_stamp = stamp;
    counters.loop_stamp_valid = true;
}

/// One cooperative service pass.
#[inline(always)]
pub fn observe_loop() {
    observe_loop_at(now());
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
        counters.batch.starved_passes = counters.batch.starved_passes.wrapping_add(1);
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
        let cycle = stamp.wrapping_sub(counters.batch.go_stamp);
        counters.cycle_sum = counters.cycle_sum.wrapping_add(cycle);
        counters.batches = counters.batches.wrapping_add(1);
        if counters.batch.open_starved {
            counters.starved_cycle_sum = counters.starved_cycle_sum.wrapping_add(cycle);
            counters.starved_batches = counters.starved_batches.wrapping_add(1);
        } else {
            counters.clean_cycle_sum = counters.clean_cycle_sum.wrapping_add(cycle);
            counters.clean_batches = counters.clean_batches.wrapping_add(1);
        }
        // Depth is known only once the batch has drained, so the bucket is
        // filled here, at the GO that closes it.
        let index = counters.batch.members.clamp(1, 4) as usize - 1;
        counters.depth_batches[index] = counters.depth_batches[index].wrapping_add(1);
        counters.depth_firstdrain[index] = counters
            .depth_firstdrain[index]
            .wrapping_add(counters.batch.firstdrain_span);
    }
    counters.batch.go_stamp = stamp;
    counters.batch.go_valid = true;
    counters.batch.drain_valid = false;
    counters.batch.current_starved = counters.batch.starved_passes > 0;
    counters.batch.open_starved = counters.batch.current_starved;
    counters.batch.starved_passes = 0;
    counters.batch.members = 0;
    counters.batch.firstdrain_span = 0;
}

/// A hardware completion drained on `pipe` at `stamp`. Test-visible core.
#[inline(always)]
pub fn note_drain_at(pipe: u8, stamp: u32) {
    if pipe != 0 {
        return;
    }
    let counters = counters();
    counters.members = counters.members.wrapping_add(1);
    counters.batch.members = counters.batch.members.wrapping_add(1);
    if !counters.batch.drain_valid && counters.batch.go_valid {
        let first_drain = stamp.wrapping_sub(counters.batch.go_stamp);
        counters.go_firstdrain_sum = counters.go_firstdrain_sum.wrapping_add(first_drain);
        counters.batch.firstdrain_span = first_drain;
        counters.batch.drain_valid = true;
    }
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

/// Confirmation and admission hooks are retained for wiring parity; CYC8 does
/// not decompose the refill span.
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
    values[3] = counters.cycle_sum;
    values[4] = counters.members;
    values[5] = counters.go_firstdrain_sum;
    values[6] = counters.depth_batches[0];
    values[7] = counters.depth_batches[1];
    values[8] = counters.depth_batches[2];
    values[9] = counters.depth_batches[3];
    values[10] = counters.depth_firstdrain[0];
    values[11] = counters.depth_firstdrain[1];
    values[12] = counters.depth_firstdrain[2];
    values[13] = counters.depth_firstdrain[3];
    values[14] = counters.clean_batches;
    values[15] = counters.starved_batches;
    values[16] = counters.clean_cycle_sum;
    values[17] = counters.starved_cycle_sum;
    values[18] = counters.loop_delta_sum;
    values[19] = counters.loop_slow;
    values[20] = counters.loop_delta_count;
    values[21] = counters.pipe0_idle_starved;
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

    /// The probe counters are one global, and cargo runs tests in parallel, so
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
            *COUNTERS.0.get() = Counters {
                loops: 0,
                batches: 0,
                cycle_sum: 0,
                members: 0,
                go_firstdrain_sum: 0,
                clean_batches: 0,
                starved_batches: 0,
                clean_cycle_sum: 0,
                starved_cycle_sum: 0,
                depth_batches: [0; 4],
                depth_firstdrain: [0; 4],
                loop_delta_sum: 0,
                loop_delta_count: 0,
                loop_slow: 0,
                pipe0_idle_starved: 0,
                pipe0_idle_blocked: 0,
                last_loop_stamp: 0,
                loop_stamp_valid: false,
                batch: BatchState {
                    go_stamp: 0,
                    starved_passes: 0,
                    members: 0,
                    firstdrain_span: 0,
                    go_valid: false,
                    drain_valid: false,
                    current_starved: false,
                    open_starved: false,
                },
            };
        }
    }

    #[test]
    fn slow_loop_intervals_are_counted() {
        let _guard = Guard::acquire();
        reset();
        observe_loop_at(1_000);
        observe_loop_at(1_200); // 200, fast
        observe_loop_at(3_000); // 1800, slow
        let values = snapshot();
        assert_eq!(values[0], MAGIC);
        assert_eq!(values[1], 3);
        assert_eq!(values[20], 2); // two intervals
        assert_eq!(values[18], 2_000);
        assert_eq!(values[19], 1); // one above 1 ms
    }

    #[test]
    fn batches_are_bucketed_by_their_own_depth() {
        let _guard = Guard::acquire();
        reset();
        // Depth 2: two drains, first at 400 us after the GO.
        note_go_at(0, 1_000);
        note_drain_at(0, 1_400);
        note_drain_at(0, 1_500);
        note_go_at(0, 2_000);
        // Depth 1: single member, first drain 300 us after its GO.
        note_drain_at(0, 2_300);
        note_go_at(0, 3_000);
        let values = snapshot();
        assert_eq!(values[7], 1); // one depth-2 batch
        assert_eq!(values[6], 1); // one depth-1 batch
        assert_eq!(values[11], 400); // depth-2 first-drain sum
        assert_eq!(values[10], 300); // depth-1 first-drain sum
    }

    #[test]
    fn first_loop_opens_no_interval() {
        let _guard = Guard::acquire();
        reset();
        observe_loop_at(500);
        let values = snapshot();
        assert_eq!(values[20], 0);
        assert_eq!(values[19], 0);
    }

    #[test]
    fn loop_wrap_is_handled() {
        let _guard = Guard::acquire();
        reset();
        observe_loop_at(u32::MAX - 50);
        observe_loop_at(50);
        let values = snapshot();
        assert_eq!(values[18], 101);
        assert_eq!(values[19], 0);
    }

    #[test]
    fn supply_clean_and_starved_batches_are_counted_separately() {
        let _guard = Guard::acquire();
        reset();
        note_go_at(0, 1_000);
        note_drain_at(0, 2_000);
        note_go_at(0, 11_000); // clean cycle 10_000
        note_drain_at(0, 12_000);
        note_pipe0_pass(false, false);
        note_go_at(0, 21_000); // second cycle still clean
        note_drain_at(0, 25_000);
        note_go_at(0, 31_000); // third cycle starved
        let values = snapshot();
        assert_eq!(values[2], 3);
        assert_eq!(values[14], 2);
        assert_eq!(values[15], 1);
        assert_eq!(values[16], 20_000);
        assert_eq!(values[17], 10_000);
    }

    #[test]
    fn blocked_work_is_not_classified_as_starvation() {
        let _guard = Guard::acquire();
        reset();
        note_pipe0_pass(false, true);
        note_go_at(0, 100);
        note_drain_at(0, 200);
        note_go_at(0, 1_100);
        let values = snapshot();
        assert_eq!(values[14], 1); // the batch is clean
        assert_eq!(values[15], 0); // none starved
        assert_eq!(values[21], 0); // no idle-starved pass counted
        assert_eq!(values[6], 1); // bucketed as depth 1
    }

    #[test]
    fn other_pipes_are_ignored() {
        let _guard = Guard::acquire();
        reset();
        note_go_at(0, 100);
        note_drain_at(1, 150);
        note_go_at(2, 160);
        note_drain_at(0, 200);
        let values = snapshot();
        assert_eq!(values[4], 1);
        assert_eq!(values[5], 100);
        assert_eq!(values[2], 0);
    }
}
