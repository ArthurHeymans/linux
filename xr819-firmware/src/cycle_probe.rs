//! Temporary per-batch cycle decomposition probe (counters MIB 0x100c, CYC6).
//!
//! Splits the publish cycle at every boundary the firmware can see and now
//! exports distributions rather than only means, because a bimodal or quantized
//! GO -> first-drain distribution would mean the "fixed excess" is a mixture of
//! prompt and stalled batches rather than one constant delay:
//!
//! * GO -> first drain, histogrammed (chip airtime plus completion delivery);
//! * consecutive drains inside one batch (the per-member inter-drain gap);
//! * cooperative passes and MAC event-FIFO services inside the same window;
//! * confirmation -> first admission, histogrammed (host refill round trip).
//!
//! There is deliberately no "early completion visibility" counter: in this
//! firmware a class-0 completion is *produced* by processing a MAC event
//! (`service_single_probe_runtime_inactive` pops the event, the handler
//! enqueues the completion into the firmware ring, and the scheduler lane drains
//! it), and the pipe words a pass could sample instead are firmware-written
//! (cursor, completion word). The firmware therefore cannot observe a completion
//! before the event that carries it, and one poll of `mac_service_pending` bounds
//! its own contribution to one pass.
//!
//! MIB words (CYC6 `0x43594336`): 0 marker, 1 loops, 2 batches, 3 GO->GO sum,
//! 4 members, 5 intra-batch drain-gap sum, 6 its count, 7 GO->first-drain sum,
//! 8 passes in that window, 9 MAC event services in that window, 10 confirm->
//! admit sum, 11..=15 GO->first-drain histogram (<=250, <=500, <=1000, <=2000,
//! >2000 us), 16..=18 confirm->admit histogram (<=500, <=2000, >2000 us),
//! > 19 pipe-0 idle-starved passes, 20 pipe-0 idle-blocked passes, 21 reserved.
//! > This schema supersedes CYC5 words. No timers of its own: the existing vendor
//! > microsecond timer is the only clock, and there is no scheduling, ownership or
//! > lifecycle change. All words are `u32` and wrap modulo 2^32.

use core::cell::UnsafeCell;

/// Counters MIB schema marker, word 0.
pub const MAGIC: u32 = 0x4359_4336;

/// GO -> first-drain histogram bounds in microseconds, excluding the final bin.
const FIRST_DRAIN_BOUNDS: [u32; 4] = [250, 500, 1000, 2000];
/// Confirmation -> admission histogram bounds, excluding the final bin.
const CONFIRM_ADMIT_BOUNDS: [u32; 2] = [500, 2000];

#[derive(Clone, Copy, Default)]
struct BatchState {
    go_stamp: u32,
    last_drain_stamp: u32,
    awaiting_admit_stamp: u32,
    go_valid: bool,
    drain_valid: bool,
    awaiting_admit: bool,
    passes_pending: u32,
    mac_services_pending: u32,
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
    passes_to_first_sum: u32,
    mac_services_to_first_sum: u32,
    confirm_admit_sum: u32,
    first_drain_bins: [u32; 5],
    confirm_admit_bins: [u32; 3],
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
    passes_to_first_sum: 0,
    mac_services_to_first_sum: 0,
    confirm_admit_sum: 0,
    first_drain_bins: [0; 5],
    confirm_admit_bins: [0; 3],
    pipe0_idle_starved: 0,
    pipe0_idle_blocked: 0,
    batch: BatchState {
        go_stamp: 0,
        last_drain_stamp: 0,
        awaiting_admit_stamp: 0,
        go_valid: false,
        drain_valid: false,
        awaiting_admit: false,
        passes_pending: 0,
        mac_services_pending: 0,
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
    let counters = counters();
    counters.loops = counters.loops.wrapping_add(1);
    // Passes between a GO and its first drain: a late completion can only be
    // attributed to the MAC if the firmware actually kept polling.
    if counters.batch.go_valid && !counters.batch.drain_valid {
        counters.batch.passes_pending = counters.batch.passes_pending.wrapping_add(1);
    }
}

/// One MAC event-FIFO service admission.
#[inline(always)]
pub fn note_mac_service() {
    let counters = counters();
    if counters.batch.go_valid && !counters.batch.drain_valid {
        counters.batch.mac_services_pending = counters.batch.mac_services_pending.wrapping_add(1);
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
        counters.cycle_sum = counters
            .cycle_sum
            .wrapping_add(stamp.wrapping_sub(counters.batch.go_stamp));
        counters.batches = counters.batches.wrapping_add(1);
    }
    counters.batch.go_stamp = stamp;
    counters.batch.go_valid = true;
    counters.batch.drain_valid = false;
    counters.batch.passes_pending = 0;
    counters.batch.mac_services_pending = 0;
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
        counters.first_drain_bins[bin(&FIRST_DRAIN_BOUNDS, first_drain)] += 1;
        counters.passes_to_first_sum = counters
            .passes_to_first_sum
            .wrapping_add(counters.batch.passes_pending);
        counters.mac_services_to_first_sum = counters
            .mac_services_to_first_sum
            .wrapping_add(counters.batch.mac_services_pending);
        counters.batch.drain_valid = true;
    }
    counters.batch.last_drain_stamp = stamp;
}

/// A TX request admitted from the command lane at `stamp`. Test-visible core.
#[inline(always)]
pub fn note_admit_at(stamp: u32) {
    let counters = counters();
    if counters.batch.awaiting_admit {
        let span = stamp.wrapping_sub(counters.batch.awaiting_admit_stamp);
        counters.confirm_admit_sum = counters.confirm_admit_sum.wrapping_add(span);
        counters.confirm_admit_bins[bin(&CONFIRM_ADMIT_BOUNDS, span)] += 1;
        counters.batch.awaiting_admit = false;
    }
}

/// A TX confirmation published to the host at `stamp`. Test-visible core.
#[inline(always)]
pub fn note_confirm_at(stamp: u32) {
    let counters = counters();
    counters.batch.awaiting_admit = true;
    counters.batch.awaiting_admit_stamp = stamp;
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
    values[8] = counters.passes_to_first_sum;
    values[9] = counters.mac_services_to_first_sum;
    values[10] = counters.confirm_admit_sum;
    values[11] = counters.first_drain_bins[0];
    values[12] = counters.first_drain_bins[1];
    values[13] = counters.first_drain_bins[2];
    values[14] = counters.first_drain_bins[3];
    values[15] = counters.first_drain_bins[4];
    values[16] = counters.confirm_admit_bins[0];
    values[17] = counters.confirm_admit_bins[1];
    values[18] = counters.confirm_admit_bins[2];
    values[19] = counters.pipe0_idle_starved;
    values[20] = counters.pipe0_idle_blocked;
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
                passes_to_first_sum: 0,
                mac_services_to_first_sum: 0,
                confirm_admit_sum: 0,
                first_drain_bins: [0; 5],
                confirm_admit_bins: [0; 3],
                pipe0_idle_starved: 0,
                pipe0_idle_blocked: 0,
                batch: BatchState {
                    go_stamp: 0,
                    last_drain_stamp: 0,
                    awaiting_admit_stamp: 0,
                    go_valid: false,
                    drain_valid: false,
                    awaiting_admit: false,
                    passes_pending: 0,
                    mac_services_pending: 0,
                },
            };
        }
    }

    #[test]
    fn full_batch_records_every_span() {
        reset();
        note_go_at(0, 1_000);
        note_drain_at(0, 3_000);
        note_drain_at(0, 4_000);
        note_confirm_at(4_500);
        note_admit_at(6_000);
        note_go_at(0, 11_000);
        let values = snapshot();
        assert_eq!(values[0], MAGIC);
        assert_eq!(values[2], 1); // one completed batch interval
        assert_eq!(values[3], 10_000); // GO -> GO
        assert_eq!(values[4], 2); // members
        assert_eq!(values[5], 1_000); // first -> second drain
        assert_eq!(values[6], 1);
        assert_eq!(values[7], 2_000); // GO -> first drain
        assert_eq!(values[10], 1_500); // confirm -> admit
    }

    #[test]
    fn first_drain_histogram_places_every_batch_in_one_bin() {
        reset();
        note_go_at(0, 0);
        note_drain_at(0, 100);
        note_go_at(0, 10_000); // 100 us -> bin 0
        note_drain_at(0, 10_300);
        note_go_at(0, 20_000); // 300 us -> bin 1
        note_drain_at(0, 21_200);
        note_go_at(0, 30_000); // 1200 us -> bin 3
        note_drain_at(0, 33_000);
        note_go_at(0, 40_000); // 3000 us -> bin 4
        let values = snapshot();
        assert_eq!(values[11], 1);
        assert_eq!(values[12], 1);
        assert_eq!(values[13], 0);
        assert_eq!(values[14], 1);
        assert_eq!(values[15], 1);
        let total: u32 = values[11..16].iter().sum();
        assert_eq!(total, values[2]); // one histogram sample per completed batch
    }

    #[test]
    fn confirm_admit_histogram_bins_by_span() {
        reset();
        note_confirm_at(0);
        note_admit_at(100); // bin 0
        note_confirm_at(1_000);
        note_admit_at(2_000); // 1000 us -> bin 1
        note_confirm_at(3_000);
        note_admit_at(9_000); // 6000 us -> bin 2
        let values = snapshot();
        assert_eq!(values[16], 1);
        assert_eq!(values[17], 1);
        assert_eq!(values[18], 1);
        assert_eq!(values[10], 7_100); // 100 + 1000 + 6000
    }

    #[test]
    fn passes_and_mac_services_are_attributed_to_the_first_drain() {
        reset();
        note_go_at(0, 1_000);
        observe_loop();
        observe_loop();
        note_mac_service();
        observe_loop();
        note_drain_at(0, 2_000);
        observe_loop(); // after the first drain: outside this batch's window
        note_go_at(0, 3_000);
        let values = snapshot();
        assert_eq!(values[8], 3); // passes in the window
        assert_eq!(values[9], 1); // MAC services in the window
    }

    #[test]
    fn scheduling_window_resets_at_each_go() {
        reset();
        note_go_at(0, 100);
        observe_loop();
        note_drain_at(0, 200);
        note_go_at(0, 300);
        observe_loop();
        observe_loop();
        note_drain_at(0, 400);
        let values = snapshot();
        assert_eq!(values[8], 3); // 1 from the first batch, 2 from the second
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
        assert_eq!(values[10], 100); // confirm -> admit across the wrap
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
        assert_eq!(values[19] + values[20], 2);
    }
}
