//! Temporary per-batch cycle probe with supply classification (MIB 0x100c, CYC7).
//!
//! Every earlier probe measured batches whose supply state was unknown, so a
//! long GO->first-drain or a long cycle could always be blamed on the host not
//! having fed the pipe. CYC7 classifies each batch by whether the window before
//! its GO contained any idle-starved cooperative pass (pipe 0 idle with no
//! admitted work at all) and reports cycle and GO->first-drain separately for
//! supply-clean and supply-starved batches, with a three-bin histogram each.
//!
//! A supply-clean batch is one the firmware could publish the moment the pipe
//! retired, so its cycle is the MAC/air term with host refill removed; a
//! starved batch measures the host. If the two classes differ only in their
//! cycle tails, host supply is the lever; if supply-clean batches still show a
//! long GO->first drain, the term is below the firmware.
//!
//! MIB words (CYC7 `0x43594337`): 0 marker, 1 loops, 2 batches, 3 GO->GO sum,
//! 4 members, 5 intra-batch drain-gap sum, 6 its count, 7 GO->first-drain sum,
//! 8 clean cycle sum, 9 clean GO->first-drain sum, 10 clean batches,
//! 11 starved cycle sum, 12 starved GO->first-drain sum, 13 starved batches,
//! 14..=16 clean GO->first-drain histogram (<=1000, <=2000, >2000 us),
//! 17..=19 starved histogram (same bounds), 20 pipe-0 idle-starved passes,
//! 21 pipe-0 idle-blocked passes. This schema supersedes CYC6 words. No timers
//! of its own: the existing vendor microsecond timer is the only clock, and
//! there is no scheduling, ownership or lifecycle change.

use core::cell::UnsafeCell;

/// Counters MIB schema marker, word 0.
pub const MAGIC: u32 = 0x4359_4337;

/// GO -> first-drain histogram bounds in microseconds, excluding the tail bin.
const FIRST_DRAIN_BOUNDS: [u32; 2] = [1000, 2000];

#[derive(Clone, Copy, Default)]
struct BatchState {
    go_stamp: u32,
    last_drain_stamp: u32,
    starved_passes: u32,
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
    interdrain_sum: u32,
    interdrain_count: u32,
    go_firstdrain_sum: u32,
    clean_cycle_sum: u32,
    clean_firstdrain_sum: u32,
    clean_batches: u32,
    starved_cycle_sum: u32,
    starved_firstdrain_sum: u32,
    starved_batches: u32,
    clean_bins: [u32; 3],
    starved_bins: [u32; 3],
    pipe0_idle_starved: u32,
    pipe0_idle_blocked: u32,
    batch: BatchState,
}

struct Shared(UnsafeCell<Counters>);
// SAFETY: only the single cooperative foreground thread touches these counters;
// GO, drain, admission, confirmation and pass hooks all run in that context,
// never in an IRQ.
unsafe impl Sync for Shared {}
static COUNTERS: Shared = Shared(UnsafeCell::new(Counters {
    loops: 0,
    batches: 0,
    cycle_sum: 0,
    members: 0,
    interdrain_sum: 0,
    interdrain_count: 0,
    go_firstdrain_sum: 0,
    clean_cycle_sum: 0,
    clean_firstdrain_sum: 0,
    clean_batches: 0,
    starved_cycle_sum: 0,
    starved_firstdrain_sum: 0,
    starved_batches: 0,
    clean_bins: [0; 3],
    starved_bins: [0; 3],
    pipe0_idle_starved: 0,
    pipe0_idle_blocked: 0,
    batch: BatchState {
        go_stamp: 0,
        last_drain_stamp: 0,
        starved_passes: 0,
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

/// One cooperative service pass.
#[inline(always)]
pub fn observe_loop() {
    counters().loops = counters().loops.wrapping_add(1);
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
        // No admitted work at all during the refill window: the next batch is
        // host-limited rather than firmware- or MAC-limited.
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
    }
    counters.batch.go_stamp = stamp;
    counters.batch.go_valid = true;
    counters.batch.drain_valid = false;
    counters.batch.current_starved = counters.batch.starved_passes > 0;
    counters.batch.open_starved = counters.batch.current_starved;
    counters.batch.starved_passes = 0;
}

/// A hardware completion drained on `pipe` at `stamp`. Test-visible core.
#[inline(always)]
pub fn note_drain_at(pipe: u8, stamp: u32) {
    if pipe != 0 {
        return;
    }
    let counters = counters();
    counters.members = counters.members.wrapping_add(1);
    if counters.batch.drain_valid {
        // Consecutive members of one batch: a per-member dead time would show up
        // here, because the members share a single GO.
        counters.interdrain_sum = counters
            .interdrain_sum
            .wrapping_add(stamp.wrapping_sub(counters.batch.last_drain_stamp));
        counters.interdrain_count = counters.interdrain_count.wrapping_add(1);
    } else if counters.batch.go_valid {
        let first_drain = stamp.wrapping_sub(counters.batch.go_stamp);
        counters.go_firstdrain_sum = counters.go_firstdrain_sum.wrapping_add(first_drain);
        if counters.batch.current_starved {
            counters.starved_firstdrain_sum =
                counters.starved_firstdrain_sum.wrapping_add(first_drain);
            counters.starved_bins[bin(&FIRST_DRAIN_BOUNDS, first_drain)] += 1;
        } else {
            counters.clean_firstdrain_sum =
                counters.clean_firstdrain_sum.wrapping_add(first_drain);
            counters.clean_bins[bin(&FIRST_DRAIN_BOUNDS, first_drain)] += 1;
        }
        counters.batch.drain_valid = true;
    }
    counters.batch.last_drain_stamp = stamp;
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

/// A TX confirmation published to the host now. Retained so the probe keeps the
/// same hook set as CYC6 even though CYC7 only classifies supply.
#[inline(always)]
pub fn note_confirm() {}

/// A TX request admitted from the command lane now. Retained for hook parity.
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
    values[5] = counters.interdrain_sum;
    values[6] = counters.interdrain_count;
    values[7] = counters.go_firstdrain_sum;
    values[8] = counters.clean_cycle_sum;
    values[9] = counters.clean_firstdrain_sum;
    values[10] = counters.clean_batches;
    values[11] = counters.starved_cycle_sum;
    values[12] = counters.starved_firstdrain_sum;
    values[13] = counters.starved_batches;
    values[14] = counters.clean_bins[0];
    values[15] = counters.clean_bins[1];
    values[16] = counters.clean_bins[2];
    values[17] = counters.starved_bins[0];
    values[18] = counters.starved_bins[1];
    values[19] = counters.starved_bins[2];
    values[20] = counters.pipe0_idle_starved;
    values[21] = counters.pipe0_idle_blocked;
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
                clean_cycle_sum: 0,
                clean_firstdrain_sum: 0,
                clean_batches: 0,
                starved_cycle_sum: 0,
                starved_firstdrain_sum: 0,
                starved_batches: 0,
                clean_bins: [0; 3],
                starved_bins: [0; 3],
                pipe0_idle_starved: 0,
                pipe0_idle_blocked: 0,
                batch: BatchState {
                    go_stamp: 0,
                    last_drain_stamp: 0,
                    starved_passes: 0,
                    go_valid: false,
                    drain_valid: false,
                    current_starved: false,
                    open_starved: false,
                },
            };
        }
    }

    #[test]
    fn supply_clean_and_starved_batches_are_counted_separately() {
        reset();
        // Batch A: clean refill, closes at the second GO.
        note_go_at(0, 1_000);
        note_drain_at(0, 2_000);
        note_go_at(0, 11_000); // A cycle 10_000, clean
        // Batch B: also clean; then a starved refill before C.
        note_drain_at(0, 12_000);
        note_pipe0_pass(false, false);
        note_go_at(0, 21_000); // B cycle 10_000, clean; C opens starved
        note_drain_at(0, 25_000); // C first drain 4_000
        note_go_at(0, 31_000); // C cycle 10_000, starved
        let values = snapshot();
        assert_eq!(values[0], MAGIC);
        assert_eq!(values[2], 3); // three completed cycles
        assert_eq!(values[10], 2); // clean batches
        assert_eq!(values[13], 1); // starved batch
        assert_eq!(values[8], 20_000); // clean cycle sum
        assert_eq!(values[11], 10_000); // starved cycle sum
        assert_eq!(values[9], 2_000); // clean GO -> first drain sum
        assert_eq!(values[12], 4_000); // starved GO -> first drain
        assert_eq!(values[14], 2); // both clean drains <= 1000 us
        assert_eq!(values[19], 1); // starved drain > 2000 us
    }

    #[test]
    fn histogram_bins_follow_the_classification() {
        reset();
        note_go_at(0, 0);
        note_drain_at(0, 500); // clean, <=1000
        note_go_at(0, 10_000);
        note_pipe0_pass(false, false);
        note_go_at(0, 20_000);
        note_drain_at(0, 23_000); // starved, >2000
        let values = snapshot();
        assert_eq!(values[14], 1); // clean <=1000
        assert_eq!(values[16], 0);
        assert_eq!(values[17], 0);
        assert_eq!(values[19], 1); // starved >2000
    }

    #[test]
    fn blocked_work_is_not_classified_as_starvation() {
        reset();
        note_pipe0_pass(false, true); // work waiting: blocked, not starved
        note_go_at(0, 100);
        note_drain_at(0, 200);
        note_go_at(0, 1_100);
        let values = snapshot();
        assert_eq!(values[10], 1); // clean
        assert_eq!(values[13], 0);
        assert_eq!(values[21], 1); // blocked recorded, starved not
        assert_eq!(values[20], 0);
    }

    #[test]
    fn starvation_after_the_go_does_not_reclassify_the_batch() {
        reset();
        note_go_at(0, 100);
        note_pipe0_pass(false, false); // during the batch's own airtime window
        note_drain_at(0, 200);
        note_go_at(0, 1_100);
        let values = snapshot();
        assert_eq!(values[10], 1); // still clean
        assert_eq!(values[13], 0);
    }

    #[test]
    fn timer_wrap_uses_wrapping_arithmetic() {
        reset();
        note_go_at(0, u32::MAX - 100);
        note_drain_at(0, 200);
        note_go_at(0, 500);
        let values = snapshot();
        assert_eq!(values[3], 601);
        assert_eq!(values[7], 301);
    }

    #[test]
    fn other_pipes_are_ignored() {
        reset();
        note_go_at(0, 100);
        note_drain_at(1, 150);
        note_go_at(2, 160);
        note_drain_at(0, 200);
        let values = snapshot();
        assert_eq!(values[4], 1);
        assert_eq!(values[7], 100);
        assert_eq!(values[2], 0);
    }
}
