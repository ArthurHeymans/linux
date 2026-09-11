//! Temporary batch-cycle timing probe (counters MIB 0x100c, CYC4 schema).
//!
//! Splits the ~8 ms publish cycle into three spans so cycle surgery can target
//! the dominant hop instead of guessing:
//!
//! * GO -> drain per pipe: airtime plus completion-drain latency;
//! * drain -> confirm, global: completion routing, encoding and HIF publish;
//! * confirm -> GO per pipe: credit return, admission and reservation.
//!
//! Words 20..=21 partition every pass for pipe 0 (the only pipe our traffic
//! uses; multi-pipe generalization is a follow-up): idle with no waiting
//! work (supply-starved) vs idle with waiting work (mechanics-blocked). Busy
//! passes are `loops - starved - blocked`. A pass counts blocked when pipe 0
//! has PasQueued-but-unowned candidates or Reserved-but-untriggered batches
//! while no hardware owner exists; multi-dispatch's null result makes this
//! partition, not finer spans, the next discriminator.
//!
//! Under the single-owner gate each pipe cycles GO -> drain -> confirm -> GO
//! in order; a drain with no preceding GO (e.g. a management publication,
//! which is not instrumented) only refreshes the drain stamp without opening
//! a GO -> drain span. Sums and counts are exported so means survive sampling
//! anywhere in the phase; timer wrap is handled by `wrapping_sub`. All words
//! are `u32` and wrap modulo 2^32.
//!
//! MIB words (CYC4 `0x43594334`): 0 marker, 1 cooperative loops, 2..=5
//! GO->drain sums per pipe, 6..=9 their counts, 10..=13 confirm->GO sums,
//! 14..=17 their counts, 18 drain->confirm sum, 19 its count, 20 pipe-0
//! idle-starved passes, 21 pipe-0 idle-blocked passes. This schema supersedes
//! RXP1/service-probe and CYC3 words; the features are never enabled in the
//! same build. No timers of its own: the existing vendor microsecond timer
//! is the only clock, and there is no scheduling, ownership or lifecycle
//! change.

use core::cell::UnsafeCell;

/// Counters MIB schema marker, word 0.
pub const MAGIC: u32 = 0x4359_4334;

const PIPES: usize = 4;

#[derive(Clone, Copy, Default)]
struct PipeState {
    go_stamp: u32,
    drain_stamp: u32,
    go_pending: bool,
    go_drain_sum: u32,
    go_drain_count: u32,
    confirm_go_sum: u32,
    confirm_go_count: u32,
}

#[derive(Clone, Copy, Default)]
struct Counters {
    loops: u32,
    pipes: [PipeState; PIPES],
    last_drain_stamp: u32,
    last_drain_valid: bool,
    last_confirm_stamp: u32,
    last_confirm_valid: bool,
    drain_confirm_sum: u32,
    drain_confirm_count: u32,
    pipe0_idle_starved: u32,
    pipe0_idle_blocked: u32,
}

struct Shared(UnsafeCell<Counters>);
// SAFETY: only the single cooperative foreground thread touches these counters;
// GO, drain and confirmation hooks all run in that context, never in an IRQ.
unsafe impl Sync for Shared {}
static COUNTERS: Shared = Shared(UnsafeCell::new(Counters {
    loops: 0,
    pipes: [PipeState {
        go_stamp: 0,
        drain_stamp: 0,
        go_pending: false,
        go_drain_sum: 0,
        go_drain_count: 0,
        confirm_go_sum: 0,
        confirm_go_count: 0,
    }; PIPES],
    last_drain_stamp: 0,
    last_drain_valid: false,
    last_confirm_stamp: 0,
    last_confirm_valid: false,
    drain_confirm_sum: 0,
    drain_confirm_count: 0,
    pipe0_idle_starved: 0,
    pipe0_idle_blocked: 0,
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
    if pipe as usize >= PIPES {
        return;
    }
    let counters = counters();
    let pipe_state = &mut counters.pipes[pipe as usize];
    // Confirm -> GO span against the latest global confirmation. Under
    // single-owner ordering a confirmation precedes the next GO; a GO with
    // no preceding confirmation (boot) is counted without a span.
    if counters.last_confirm_valid {
        pipe_state.confirm_go_sum = pipe_state
            .confirm_go_sum
            .wrapping_add(stamp.wrapping_sub(counters.last_confirm_stamp));
        pipe_state.confirm_go_count = pipe_state.confirm_go_count.wrapping_add(1);
    }
    pipe_state.go_stamp = stamp;
    pipe_state.go_pending = true;
}

/// A hardware completion drained on `pipe` at `stamp`. Test-visible core.
#[inline(always)]
pub fn note_drain_at(pipe: u8, stamp: u32) {
    if pipe as usize >= PIPES {
        return;
    }
    let counters = counters();
    let pipe_state = &mut counters.pipes[pipe as usize];
    // Only the first drain after a GO opens a GO -> drain span; later drains
    // of the same batch still refresh the stamp for drain -> confirm.
    if pipe_state.go_pending {
        pipe_state.go_drain_sum = pipe_state
            .go_drain_sum
            .wrapping_add(stamp.wrapping_sub(pipe_state.go_stamp));
        pipe_state.go_drain_count = pipe_state.go_drain_count.wrapping_add(1);
        pipe_state.go_pending = false;
    }
    pipe_state.drain_stamp = stamp;
    counters.last_drain_stamp = stamp;
    counters.last_drain_valid = true;
}

/// A TX confirmation published to the host at `stamp`. Test-visible core.
#[inline(always)]
pub fn note_confirm_at(stamp: u32) {
    let counters = counters();
    if counters.last_drain_valid {
        counters.drain_confirm_sum = counters
            .drain_confirm_sum
            .wrapping_add(stamp.wrapping_sub(counters.last_drain_stamp));
        counters.drain_confirm_count = counters.drain_confirm_count.wrapping_add(1);
    }
    counters.last_confirm_stamp = stamp;
    counters.last_confirm_valid = true;
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

/// A TX confirmation published to the host now.
#[inline(always)]
pub fn note_confirm() {
    note_confirm_at(now());
}

pub fn snapshot() -> [u32; 22] {
    let counters = unsafe { *COUNTERS.0.get() };
    let mut values = [0_u32; 22];
    values[0] = MAGIC;
    values[1] = counters.loops;
    for pipe in 0..PIPES {
        let state = counters.pipes[pipe];
        values[2 + pipe] = state.go_drain_sum;
        values[6 + pipe] = state.go_drain_count;
        values[10 + pipe] = state.confirm_go_sum;
        values[14 + pipe] = state.confirm_go_count;
    }
    values[18] = counters.drain_confirm_sum;
    values[19] = counters.drain_confirm_count;
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
                pipes: [PipeState {
                    go_stamp: 0,
                    drain_stamp: 0,
                    go_pending: false,
                    go_drain_sum: 0,
                    go_drain_count: 0,
                    confirm_go_sum: 0,
                    confirm_go_count: 0,
                }; PIPES],
                last_drain_stamp: 0,
                last_drain_valid: false,
                last_confirm_stamp: 0,
                last_confirm_valid: false,
                drain_confirm_sum: 0,
                drain_confirm_count: 0,
                pipe0_idle_starved: 0,
                pipe0_idle_blocked: 0,
            };
        }
    }

    #[test]
    fn ordered_cycle_records_all_three_spans() {
        reset();
        note_go_at(0, 1000);
        note_drain_at(0, 2000);
        note_confirm_at(2500);
        note_go_at(0, 9000);
        let values = snapshot();
        assert_eq!(values[0], MAGIC);
        assert_eq!(values[2], 1000); // GO -> drain
        assert_eq!(values[6], 1);
        assert_eq!(values[18], 500); // drain -> confirm
        assert_eq!(values[19], 1);
        assert_eq!(values[10], 6500); // confirm -> GO
        assert_eq!(values[14], 1);
        assert_eq!(values[20], 0); // no pipe-0 pass outcomes recorded
        assert_eq!(values[21], 0);
    }

    #[test]
    fn drain_without_go_refreshes_without_span() {
        reset();
        note_drain_at(1, 500);
        note_confirm_at(700);
        let values = snapshot();
        assert_eq!(values[3], 0); // no GO -> drain span on pipe 1
        assert_eq!(values[7], 0);
        assert_eq!(values[18], 200); // drain -> confirm still recorded
        assert_eq!(values[19], 1);
    }

    #[test]
    fn second_drain_of_batch_does_not_reopen_span() {
        reset();
        note_go_at(2, 1000);
        note_drain_at(2, 1500);
        note_drain_at(2, 1600);
        let values = snapshot();
        assert_eq!(values[4], 500);
        assert_eq!(values[8], 1);
    }

    #[test]
    fn timer_wrap_uses_wrapping_arithmetic() {
        reset();
        note_go_at(3, u32::MAX - 100);
        note_drain_at(3, 200);
        let values = snapshot();
        assert_eq!(values[5], 301);
        assert_eq!(values[9], 1);
    }

    #[test]
    fn pipes_stay_independent() {
        reset();
        note_go_at(0, 100);
        note_go_at(1, 200);
        note_drain_at(1, 500);
        let values = snapshot();
        assert_eq!(values[6], 0); // pipe 0 GO still pending, no span
        assert_eq!(values[3], 300); // pipe 1 span
        assert_eq!(values[7], 1);
    }

    #[test]
    fn out_of_range_pipe_is_ignored() {
        reset();
        note_go_at(4, 100);
        note_drain_at(9, 200);
        note_pipe0_pass(false, false);
        note_pipe0_pass(false, true);
        note_pipe0_pass(true, true);
        let values = snapshot();
        assert_eq!(values[20], 1); // one starved pass
        assert_eq!(values[21], 1); // one blocked pass, busy pass uncounted
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
        assert_eq!(values[1], 3); // loops
        assert_eq!(values[20] + values[21], 2); // two idle passes classified
    }
}
