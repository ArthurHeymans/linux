//! Optional fixed-record flight recorder for the vendor-shaped host TX runtime.
//!
//! The recorder is ordinary firmware BSS, not a hard-coded packet-RAM window.
//! Its fixed-size records remain independently decodable after a partial write:
//! the sequence word is committed last. The hardware lifecycle must never
//! depend on this module; without `vendor-host-tx-diagnostics` every writer is
//! an inline no-op.

#[cfg(feature = "vendor-host-tx-diagnostics")]
use core::cell::UnsafeCell;

pub const EVENT_REQUEST_SEEN: u16 = 0x0101;
pub const EVENT_REQUEST_DETACHED: u16 = 0x0102;
pub const EVENT_REQUEST_CREDIT: u16 = 0x0103;
pub const EVENT_OUTPUT_ENQUEUE: u16 = 0x0201;
pub const EVENT_DESCRIPTOR_STAGE: u16 = 0x0202;
pub const EVENT_DESCRIPTOR_RECLAIM: u16 = 0x0203;
pub const EVENT_MESSAGE_RELEASE: u16 = 0x0204;
pub const EVENT_RX_CLAIM: u16 = 0x0301;
pub const EVENT_RX_RELEASE: u16 = 0x0302;
pub const EVENT_TX_LIFECYCLE: u16 = 0x0401;
pub const EVENT_MAC_STATUS_2: u16 = 0x0502;
pub const EVENT_INVARIANT_FAILURE: u16 = 0xff01;

pub const STATUS2_SNAPSHOT_WORDS: usize = 28;
pub const PRE_GO_SNAPSHOT_WORDS: usize = 38;

#[cfg(all(
    feature = "vendor-host-tx-diagnostics",
    feature = "experimental-member-requeue"
))]
const STATUS2_RECORD_COUNT: usize = 1;
#[cfg(all(
    feature = "vendor-host-tx-diagnostics",
    not(feature = "experimental-member-requeue")
))]
const STATUS2_RECORD_COUNT: usize = 4;

#[cfg(all(
    feature = "vendor-host-tx-diagnostics",
    feature = "experimental-rate-feedback-telemetry"
))]
const FLIGHT_RECORD_COUNT: usize = 1;
#[cfg(all(
    feature = "vendor-host-tx-diagnostics",
    feature = "experimental-member-requeue",
    not(feature = "experimental-rate-feedback-telemetry")
))]
const FLIGHT_RECORD_COUNT: usize = 4;
#[cfg(all(
    feature = "vendor-host-tx-diagnostics",
    feature = "experimental-ampdu-outcome-telemetry",
    not(feature = "experimental-member-requeue")
))]
const FLIGHT_RECORD_COUNT: usize = 32;
#[cfg(all(
    feature = "vendor-host-tx-diagnostics",
    feature = "experimental-depth-four-ampdu",
    not(feature = "experimental-ampdu-outcome-telemetry")
))]
const FLIGHT_RECORD_COUNT: usize = 64;
#[cfg(all(
    feature = "vendor-host-tx-diagnostics",
    not(feature = "experimental-depth-four-ampdu")
))]
const FLIGHT_RECORD_COUNT: usize = 256;

#[cfg(feature = "vendor-host-tx-diagnostics")]
#[repr(C)]
#[derive(Clone, Copy)]
struct FlightRecord {
    sequence: u32,
    timestamp: u32,
    event_flags: u32,
    arg0: u32,
    arg1: u32,
    state: u32,
}

#[cfg(feature = "vendor-host-tx-diagnostics")]
const EMPTY_RECORD: FlightRecord = FlightRecord {
    sequence: 0,
    timestamp: 0,
    event_flags: 0,
    arg0: 0,
    arg1: 0,
    state: 0,
};

#[cfg(feature = "vendor-host-tx-diagnostics")]
#[repr(C)]
struct FlightRecorder {
    cursor: u32,
    frozen: u32,
    freeze_reason: u32,
    records: [FlightRecord; FLIGHT_RECORD_COUNT],
}

#[cfg(feature = "vendor-host-tx-diagnostics")]
struct SharedRecorder(UnsafeCell<FlightRecorder>);

#[cfg(feature = "vendor-host-tx-diagnostics")]
unsafe impl Sync for SharedRecorder {}

#[cfg(feature = "vendor-host-tx-diagnostics")]
static FLIGHT_RECORDER: SharedRecorder = SharedRecorder(UnsafeCell::new(FlightRecorder {
    cursor: 0,
    frozen: 0,
    freeze_reason: 0,
    records: [EMPTY_RECORD; FLIGHT_RECORD_COUNT],
}));

#[cfg(feature = "vendor-host-tx-diagnostics")]
struct SharedTxIdentity(UnsafeCell<[u32; 22]>);

#[cfg(feature = "vendor-host-tx-diagnostics")]
unsafe impl Sync for SharedTxIdentity {}

#[cfg(feature = "vendor-host-tx-diagnostics")]
static TX_IDENTITY: SharedTxIdentity = SharedTxIdentity(UnsafeCell::new([0; 22]));

/// Class-0 lifecycle counter slots.
///
/// Throughput measured over the air spans 36-95 TXed for identical firmware,
/// so it cannot resolve anything below a ~3x effect. These count state
/// transitions instead: how far each frame travels along the class-0 path.
/// A frame that is published but never started, or whose status never matches,
/// shows up as an exact count rather than as lost throughput.
pub mod counter {
    /// Host TX request admitted and classified.
    pub const ADMITTED: usize = 0;
    /// Slot published to a pipe (GO written).
    pub const PUBLISHED: usize = 1;
    /// `txp_pipe_tx_start` ran for a published slot.
    pub const TX_START: usize = 2;
    /// A pipe TX status was delivered by the MAC.
    pub const STATUS: usize = 3;
    /// The status gate refused the delivered status (the defect-A signature).
    pub const STATUS_INELIGIBLE: usize = 4;
    /// A slot completed normally.
    pub const COMPLETED: usize = 5;
    /// A refused slot was retired against the hardware ring cursor. Measures 0
    /// since retirement was restricted, so it now carries RX valid slots.
    pub const RETIRED: usize = 6;
    pub const RX_VALID_SLOTS: usize = RETIRED;
    /// A TX confirmation was handed back to the host.
    pub const CONFIRMED: usize = 7;
    /// Most recent delivered status, and the status the slot expected. Both
    /// measured a constant 0x11, so they now carry the RX side: frames dropped
    /// at the host-transfer limit, and indications published to the host.
    pub const LAST_STATUS: usize = 8;
    pub const LAST_EXPECTED: usize = 9;
    pub const RX_FILTERED: usize = LAST_STATUS;
    pub const RX_INDICATIONS: usize = LAST_EXPECTED;
    /// Pipes recovered by the 200 ms watchdog after being armed without
    /// completing for the whole window.
    /// Measured 0 in every run, because a frame that never publishes never
    /// arms a pipe, so this slot now carries the admit-to-publish split:
    /// low = admission to PAS release (pending gating), high = PAS release to
    /// publication (scheduler reservation).
    pub const WATCHDOG_RECOVERED: usize = 10;
    pub const STAGE_PENDING_SPLIT: usize = WATCHDOG_RECOVERED;
    /// RX consumer resynchronisations, i.e. observed recoverable RX FIFO
    /// corruption.
    pub const RX_RESYNC: usize = 11;
    /// Microseconds from host admission to confirmation for the most recent
    /// frame, and the worst seen. TCP moves about one segment per 9.3 ms RTT
    /// while 1500-byte airtime at 9 Mbit/s is ~1.3 ms, so most of each frame's
    /// time is unexplained; this says whether it is spent inside our pipeline.
    /// Retry-exhaustion give-ups (internal status `0x0b`) — the dominant TX
    /// outcome, and uncounted until now. `COMPLETED` counts only matched
    /// successes and the give-up path bumped nothing, so a run whose frames
    /// mostly failed looked like a run whose frames vanished. That gap produced
    /// a confident wrong diagnosis: `published - completed` was read as frames
    /// stuck in a pipe the watchdog could not see, when it was failed TX being
    /// reported correctly to the host.
    ///
    /// This takes over slot 12 from `LATENCY_LAST`, whose question is answered
    /// (firmware latency is ~1355us of a ~10.8ms per-frame budget). There is no
    /// spare slot to expand into: `wsm_mib_counters_table` in the host driver is
    /// exactly 22 words, all of which we use, so counters must be budgeted
    /// rather than added. Retire a finished one to fund a new one.
    pub const GIVE_UP: usize = 12;
    pub const LATENCY_MAX: usize = 13;
    /// Retirements split by the slot state the MAC left behind. `retired` is
    /// ~1% under ping flood but ~10% during iperf, and the split says which
    /// fault that is:
    ///
    /// - `UNSTARTED` (slot state 1): published, never transmitted. The MAC
    ///   genuinely refused the frame.
    /// - `STARTED` (state 2): transmission began but no matching status.
    /// - `TX_SUCCESS` (state 3): the frame transmitted successfully and only
    ///   its status failed to match. Retiring it reports give-up `0x0b` to the
    ///   host for a frame that was actually delivered, so mac80211 retransmits
    ///   and backs off - a self-inflicted throughput loss rather than a
    ///   radio problem.
    pub const RETIRED_UNSTARTED: usize = 14;
    /// Retirement no longer fires, so this carries RX slots released back, in
    /// the low 16 bits, and host transfers outstanding in the high 16.
    pub const RX_RELEASED: usize = RETIRED_UNSTARTED;
    /// Retirements at slot state 2 and 3 were measured at exactly 0 of 297, so
    /// these two slots now carry the latency breakdown instead; the MIB has no
    /// free fields and the report array is exactly 22 words.
    pub const RETIRED_STARTED: usize = 15;
    pub const RETIRED_TX_SUCCESS: usize = 16;
    /// Delivered status of the most recent retirement, against `LAST_EXPECTED`.
    /// Retirement now measures 0, so this slot carries the pending-gate reason:
    /// low 16 = `LeaveQueued` decisions for the frame in flight, high 16 = which
    /// gates refused, as bits 0 !active_link, 1 global_blocked, 2 !vif_operating,
    /// 3 !pipe_allowed, 4 expired.
    ///
    /// `admission -> PAS release` is 1170-3679 us of a ~4 ms frame while the
    /// scheduler reservation after it is 32 us, so this gate is the throughput
    /// ceiling.
    pub const RETIRED_LAST_STATUS: usize = 17;
    pub const PENDING_GATE: usize = RETIRED_LAST_STATUS;
    /// Retirements declined because the slot was published too recently to be
    /// stuck. These are the frames the unaged rule was destroying.
    pub const RETIREMENT_DEFERRED: usize = 18;
    /// Staged HIF output buffers whose header no longer matches what we wrote,
    /// i.e. the corruption reaching the TX output ring. Counted rather than
    /// published: `publish_terminal_exception` emits WSM id 0x0800 with no
    /// sequence bits, and the host validates the
    /// sequence before special-casing exceptions, so reporting one during live
    /// operation is itself fatal (`BH RX diag ... seq=0/2 ... result=-5`).
    pub const OUTPUT_CORRUPTION: usize = 19;
    /// Terminal exceptions suppressed instead of published. WSM id 0x0800 is
    /// the firmware-exception indication and cw1200 tears the link down when it
    /// arrives, by design, so a report-and-continue build must never send one.
    pub const SUPPRESSED_EXCEPTION: usize = 20;
    /// Where a frame's admission-to-confirmation time actually goes, for the
    /// most recent frame, packed as two 16-bit microsecond fields per counter
    /// because the counters MIB has no room left. Saturating at 65 ms.
    ///
    /// `STAGE_ADMIT_PUBLISH`: low = admission to publication (our driver
    /// deciding to send), high = publication to `tx_start` (MAC picking it up).
    /// `STAGE_START_CONFIRM`: low = `tx_start` to completion (airtime, retries,
    /// medium), high = completion to confirmation (our driver handing it back).
    ///
    /// Only the middle stage is airtime. If the time is in the first or last
    /// field, deeper TX pipelining cannot help and the fix is in the driver's
    /// service loop instead.
    pub const STAGE_ADMIT_PUBLISH: usize = RETIRED_STARTED;
    pub const STAGE_START_CONFIRM: usize = RETIRED_TX_SUCCESS;

    pub(super) const COUNT: usize = 21;
}

#[cfg(all(
    feature = "vendor-host-tx-diagnostics",
    feature = "experimental-depth-four-ampdu"
))]
struct SharedAmpduDepthTelemetry(UnsafeCell<[u32; 4]>);

#[cfg(all(
    feature = "vendor-host-tx-diagnostics",
    feature = "experimental-depth-four-ampdu"
))]
unsafe impl Sync for SharedAmpduDepthTelemetry {}

#[cfg(all(
    feature = "vendor-host-tx-diagnostics",
    feature = "experimental-depth-four-ampdu"
))]
static AMPDU_DEPTH_TELEMETRY: SharedAmpduDepthTelemetry =
    SharedAmpduDepthTelemetry(UnsafeCell::new([0; 4]));

pub mod ampdu_outcome {
    pub const BA_ALL_ACK_RX: usize = 0;
    pub const BA_PARTIAL_RX: usize = 1;
    pub const DEEP_PLAN_REARM: usize = 2;
    pub const DEEP_PLAN_EMPTY: usize = 3;
    pub const DEEP_PLAN_INVALID: usize = 4;
    pub const DEEP_WHOLE_REARM: usize = 5;
    pub const DEEP_PLAN_NO_SESSION: usize = 6;
    pub const FEEDBACK_ACK_FAILURES: usize = DEEP_PLAN_NO_SESSION;
    pub const RETRY_EVENT_REARM: usize = 7;
    pub const RETRY_EVENT_GIVE_UP: usize = 8;
    pub const RETRY_EVENT_COMPLETE: usize = 9;
    pub const WATCHDOG_REARM: usize = 10;
    pub const WATCHDOG_NO_REARM: usize = 11;
    pub const DEEP_PLAN_MIXED_RATE: usize = WATCHDOG_NO_REARM;
    pub const FEEDBACK_RATE_TRY: usize = WATCHDOG_NO_REARM;
    pub const RETIRED_UNMATCHED: usize = 12;
    pub const PARTIAL_GIVE_UP: usize = 13;
    pub const DEEP_PLAN_OUTSIDE_WINDOW: usize = PARTIAL_GIVE_UP;
    pub const FEEDBACK_ACK_WITHOUT_RATE_TRY: usize = PARTIAL_GIVE_UP;
    pub const DEEP_PLAN_NO_RATE: usize = 14;
    pub const FEEDBACK_COUNT_MISMATCH: usize = DEEP_PLAN_NO_RATE;
    pub const DEPTH_TWO_WHOLE: usize = 15;
}

#[cfg(feature = "vendor-host-tx-diagnostics")]
struct SharedLifecycleCounters(UnsafeCell<[u32; counter::COUNT]>);

#[cfg(feature = "vendor-host-tx-diagnostics")]
unsafe impl Sync for SharedLifecycleCounters {}

#[cfg(feature = "vendor-host-tx-diagnostics")]
static LIFECYCLE_COUNTERS: SharedLifecycleCounters =
    SharedLifecycleCounters(UnsafeCell::new([0; counter::COUNT]));

#[cfg(feature = "vendor-host-tx-diagnostics")]
struct SharedBatchCounter(UnsafeCell<u32>);

#[cfg(feature = "vendor-host-tx-diagnostics")]
unsafe impl Sync for SharedBatchCounter {}

#[cfg(feature = "vendor-host-tx-diagnostics")]
static BATCH_PUBLICATIONS: SharedBatchCounter = SharedBatchCounter(UnsafeCell::new(0));

#[cfg(feature = "vendor-host-tx-diagnostics")]
static AMPDU_CANDIDATES: SharedBatchCounter = SharedBatchCounter(UnsafeCell::new(0));

/// Record one same-link/TID/rate pair accepted by the A-MPDU grouping gate.
#[inline]
pub unsafe fn record_ampdu_candidate(tid: u8, rate: u8) {
    #[cfg(feature = "vendor-host-tx-diagnostics")]
    unsafe {
        let counter = AMPDU_CANDIDATES.0.get();
        let count = (counter.read_volatile() & 0xffff).wrapping_add(1) & 0xffff;
        counter.write_volatile((u32::from(rate) << 24) | (u32::from(tid) << 16) | count);
    }
    #[cfg(not(feature = "vendor-host-tx-diagnostics"))]
    let _ = (tid, rate);
}

/// Record one batch crossing the shared MAC trigger boundary.
#[inline]
pub unsafe fn record_batch_publication(depth: u8) {
    #[cfg(feature = "vendor-host-tx-diagnostics")]
    unsafe {
        let counter = BATCH_PUBLICATIONS.0.get();
        let current = counter.read_volatile();
        let count = (current & 0x00ff_ffff).wrapping_add(1) & 0x00ff_ffff;
        counter.write_volatile((u32::from(depth) << 24) | count);
    }
    #[cfg(not(feature = "vendor-host-tx-diagnostics"))]
    let _ = depth;
}

/// Records bounded depth-three/four counts for one aggregate lifecycle stage.
#[inline(always)]
pub unsafe fn record_ampdu_depth(stage: usize, depth: u8) {
    #[cfg(all(
        feature = "vendor-host-tx-diagnostics",
        feature = "experimental-depth-four-ampdu",
        not(feature = "experimental-ampdu-outcome-telemetry")
    ))]
    unsafe {
        if stage < 4 && matches!(depth, 3 | 4) {
            let shift = u32::from(depth - 3) * 16;
            let word = AMPDU_DEPTH_TELEMETRY.0.get().cast::<u32>().add(stage);
            let current = word.read_volatile();
            let count = ((current >> shift) & 0xffff).saturating_add(1);
            word.write_volatile((current & !(0xffff << shift)) | (count << shift));
        }
    }
    #[cfg(not(all(
        feature = "vendor-host-tx-diagnostics",
        feature = "experimental-depth-four-ampdu",
        not(feature = "experimental-ampdu-outcome-telemetry")
    )))]
    let _ = (stage, depth);
}

/// Record one feature-gated A-MPDU outcome in four packed saturating words.
/// Each outcome owns one byte.
pub unsafe fn record_ampdu_outcome(outcome: usize) {
    #[cfg(all(
        feature = "vendor-host-tx-diagnostics",
        feature = "experimental-ampdu-outcome-telemetry"
    ))]
    unsafe {
        if outcome < 16 {
            let word = AMPDU_DEPTH_TELEMETRY
                .0
                .get()
                .cast::<u32>()
                .add(outcome >> 2);
            let shift = ((outcome & 3) * 8) as u32;
            let current = word.read_volatile();
            let count = ((current >> shift) & 0xff).saturating_add(1);
            word.write_volatile((current & !(0xff << shift)) | (count << shift));
        }
    }
    #[cfg(not(all(
        feature = "vendor-host-tx-diagnostics",
        feature = "experimental-ampdu-outcome-telemetry"
    )))]
    let _ = outcome;
}

#[cfg(any(test, feature = "experimental-rate-feedback-telemetry"))]
fn rate_try_failure_count(rate_try: [u32; 3]) -> u8 {
    let mut failures = 0_u8;
    for word in rate_try {
        for nibble in 0..8 {
            failures = failures.saturating_add(((word >> (nibble * 4)) & 0xf) as u8);
        }
    }
    failures
}

/// Compare raw per-rate failure nibbles with the confirmation retry count.
pub unsafe fn record_rate_feedback(ack_failures: u8, rate_try: [u32; 3]) {
    #[cfg(feature = "experimental-rate-feedback-telemetry")]
    unsafe {
        let failures = rate_try_failure_count(rate_try);
        if ack_failures != 0 {
            record_ampdu_outcome(ampdu_outcome::FEEDBACK_ACK_FAILURES);
        }
        if failures != 0 {
            record_ampdu_outcome(ampdu_outcome::FEEDBACK_RATE_TRY);
        }
        if ack_failures != 0 && failures == 0 {
            record_ampdu_outcome(ampdu_outcome::FEEDBACK_ACK_WITHOUT_RATE_TRY);
        }
        if failures != ack_failures {
            record_ampdu_outcome(ampdu_outcome::FEEDBACK_COUNT_MISMATCH);
        }
    }
    #[cfg(not(feature = "experimental-rate-feedback-telemetry"))]
    let _ = (ack_failures, rate_try);
}

/// Increments one class-0 lifecycle counter.
///
/// # Safety
/// Single-threaded firmware context; the counter block has no other writer.
#[inline]
pub unsafe fn bump(counter: usize) {
    #[cfg(feature = "vendor-host-tx-diagnostics")]
    unsafe {
        if counter < counter::COUNT {
            let slot = LIFECYCLE_COUNTERS.0.get().cast::<u32>().add(counter);
            slot.write_volatile(slot.read_volatile().wrapping_add(1));
        }
    }
    #[cfg(not(feature = "vendor-host-tx-diagnostics"))]
    let _ = counter;
}

/// Overwrites a counter outright. Used to latch a one-shot snapshot of other
/// counters at the instant of an event, rather than accumulating.
pub unsafe fn set(counter: usize, value: u32) {
    #[cfg(feature = "vendor-host-tx-diagnostics")]
    unsafe {
        if counter < counter::COUNT {
            LIFECYCLE_COUNTERS
                .0
                .get()
                .cast::<u32>()
                .add(counter)
                .write_volatile(value);
        }
    }
    #[cfg(not(feature = "vendor-host-tx-diagnostics"))]
    let _ = (counter, value);
}

/// Copies the class-0 lifecycle counters.
///
/// # Safety
/// Single-threaded firmware context.
#[cfg(all(feature = "vendor-host-tx-diagnostics", target_arch = "arm"))]
pub unsafe fn counters_snapshot() -> [u32; counter::COUNT] {
    let mut values = [0_u32; counter::COUNT];
    unsafe {
        let counters = LIFECYCLE_COUNTERS.0.get().cast::<u32>();
        for (index, value) in values.iter_mut().enumerate() {
            *value = counters.add(index).read_volatile();
        }
    }
    values
}

/// Records an observed value (rather than a count) in a counter slot.
///
/// # Safety
/// Single-threaded firmware context; the counter block has no other writer.
#[inline]
/// Read one counter, for read-modify-write of packed fields.
///
/// # Safety
/// Single-threaded firmware context.
pub unsafe fn read(counter: usize) -> u32 {
    #[cfg(feature = "vendor-host-tx-diagnostics")]
    unsafe {
        if counter < counter::COUNT {
            return LIFECYCLE_COUNTERS
                .0
                .get()
                .cast::<u32>()
                .add(counter)
                .read_volatile();
        }
        0
    }
    #[cfg(not(feature = "vendor-host-tx-diagnostics"))]
    {
        let _ = counter;
        0
    }
}

pub unsafe fn observe(counter: usize, value: u32) {
    #[cfg(feature = "vendor-host-tx-diagnostics")]
    unsafe {
        if counter < counter::COUNT {
            LIFECYCLE_COUNTERS
                .0
                .get()
                .cast::<u32>()
                .add(counter)
                .write_volatile(value);
        }
    }
    #[cfg(not(feature = "vendor-host-tx-diagnostics"))]
    let _ = (counter, value);
}

#[cfg(feature = "vendor-host-tx-diagnostics")]
#[repr(C)]
#[derive(Clone, Copy)]
struct Status2Record {
    sequence: u32,
    phase: u32,
    words: [u32; STATUS2_SNAPSHOT_WORDS],
}

#[cfg(feature = "vendor-host-tx-diagnostics")]
const EMPTY_STATUS2_RECORD: Status2Record = Status2Record {
    sequence: 0,
    phase: 0,
    words: [0; STATUS2_SNAPSHOT_WORDS],
};

#[cfg(feature = "vendor-host-tx-diagnostics")]
#[repr(C)]
struct Status2Recorder {
    cursor: u32,
    frozen: u32,
    records: [Status2Record; STATUS2_RECORD_COUNT],
}

#[cfg(feature = "vendor-host-tx-diagnostics")]
struct SharedStatus2Recorder(UnsafeCell<Status2Recorder>);

#[cfg(feature = "vendor-host-tx-diagnostics")]
unsafe impl Sync for SharedStatus2Recorder {}

#[cfg(feature = "vendor-host-tx-diagnostics")]
static STATUS2_RECORDER: SharedStatus2Recorder =
    SharedStatus2Recorder(UnsafeCell::new(Status2Recorder {
        cursor: 0,
        frozen: 0,
        records: [EMPTY_STATUS2_RECORD; STATUS2_RECORD_COUNT],
    }));

#[cfg(feature = "vendor-host-tx-diagnostics")]
#[repr(C)]
struct PreGoSnapshot {
    sequence: u32,
    words: [u32; PRE_GO_SNAPSHOT_WORDS],
}

#[cfg(feature = "vendor-host-tx-diagnostics")]
struct SharedPreGoSnapshot(UnsafeCell<PreGoSnapshot>);

#[cfg(feature = "vendor-host-tx-diagnostics")]
unsafe impl Sync for SharedPreGoSnapshot {}

#[cfg(feature = "vendor-host-tx-diagnostics")]
static PRE_GO_SNAPSHOT: SharedPreGoSnapshot = SharedPreGoSnapshot(UnsafeCell::new(PreGoSnapshot {
    sequence: 0,
    words: [0; PRE_GO_SNAPSHOT_WORDS],
}));

#[cfg(feature = "vendor-host-tx-diagnostics")]
struct SharedPreGoSequence(UnsafeCell<u32>);

#[cfg(feature = "vendor-host-tx-diagnostics")]
unsafe impl Sync for SharedPreGoSequence {}

#[cfg(feature = "vendor-host-tx-diagnostics")]
static PRE_GO_PENDING_SEQUENCE: SharedPreGoSequence = SharedPreGoSequence(UnsafeCell::new(0));

#[cfg(feature = "vendor-host-tx-diagnostics")]
#[derive(Clone, Copy)]
struct LegacySnapshot {
    stage: u32,
    value0: u32,
    value1: u32,
    frame_address: u32,
    frame_metadata: u32,
    frame_words: [u32; 8],
    descriptor_length: u32,
}

#[cfg(feature = "vendor-host-tx-diagnostics")]
struct SharedLegacySnapshot(UnsafeCell<LegacySnapshot>);

#[cfg(feature = "vendor-host-tx-diagnostics")]
unsafe impl Sync for SharedLegacySnapshot {}

#[cfg(feature = "vendor-host-tx-diagnostics")]
static LEGACY_SNAPSHOT: SharedLegacySnapshot =
    SharedLegacySnapshot(UnsafeCell::new(LegacySnapshot {
        stage: 0,
        value0: 0,
        value1: 0,
        frame_address: 0,
        frame_metadata: 0,
        frame_words: [0; 8],
        descriptor_length: 0,
    }));

#[cfg(feature = "vendor-host-tx-diagnostics")]
#[inline(always)]
fn timestamp() -> u32 {
    #[cfg(target_arch = "arm")]
    unsafe {
        (0x0ac0_0004 as *const u32).read_volatile()
    }
    #[cfg(not(target_arch = "arm"))]
    unsafe {
        (*FLIGHT_RECORDER.0.get()).cursor
    }
}

/// Appends one fixed-size record. `sequence` is written last as the commit word.
#[inline(always)]
pub unsafe fn record(event: u16, flags: u16, arg0: u32, arg1: u32, state: u32) {
    #[cfg(feature = "vendor-host-tx-diagnostics")]
    unsafe {
        let recorder = FLIGHT_RECORDER.0.get();
        if core::ptr::addr_of!((*recorder).frozen).read_volatile() != 0 {
            return;
        }
        let sequence = core::ptr::addr_of!((*recorder).cursor).read_volatile();
        let slot = sequence as usize % FLIGHT_RECORD_COUNT;
        let destination = core::ptr::addr_of_mut!((*recorder).records[slot]);
        core::ptr::addr_of_mut!((*destination).timestamp).write_volatile(timestamp());
        core::ptr::addr_of_mut!((*destination).event_flags)
            .write_volatile((u32::from(event) << 16) | u32::from(flags));
        core::ptr::addr_of_mut!((*destination).arg0).write_volatile(arg0);
        core::ptr::addr_of_mut!((*destination).arg1).write_volatile(arg1);
        core::ptr::addr_of_mut!((*destination).state).write_volatile(state);
        // Volatile payload stores precede the volatile sequence commit store.
        core::ptr::addr_of_mut!((*destination).sequence).write_volatile(sequence);
        core::ptr::addr_of_mut!((*recorder).cursor).write_volatile(sequence.wrapping_add(1));
    }
    #[cfg(not(feature = "vendor-host-tx-diagnostics"))]
    let _ = (event, flags, arg0, arg1, state);
}

/// Captures one ownership snapshot around vendor type-0x39/status-2 dispatch.
/// The payload is committed by writing `sequence` last.
#[inline(always)]
pub unsafe fn record_status2_snapshot(phase: u32, words: &[u32; STATUS2_SNAPSHOT_WORDS]) {
    #[cfg(feature = "vendor-host-tx-diagnostics")]
    unsafe {
        let recorder = STATUS2_RECORDER.0.get();
        if core::ptr::addr_of!((*recorder).frozen).read_volatile() != 0 {
            return;
        }
        let cursor = core::ptr::addr_of!((*recorder).cursor).read_volatile();
        let sequence = cursor.wrapping_add(1);
        let destination =
            core::ptr::addr_of_mut!((*recorder).records[cursor as usize % STATUS2_RECORD_COUNT]);
        core::ptr::addr_of_mut!((*destination).phase).write_volatile(phase);
        for (index, value) in words.iter().copied().enumerate() {
            core::ptr::addr_of_mut!((*destination).words[index]).write_volatile(value);
        }
        core::ptr::addr_of_mut!((*destination).sequence).write_volatile(sequence);
        core::ptr::addr_of_mut!((*recorder).cursor).write_volatile(sequence);
        record(
            EVENT_MAC_STATUS_2,
            phase as u16,
            words[0],
            words[7],
            words[8],
        );
    }
    #[cfg(not(feature = "vendor-host-tx-diagnostics"))]
    let _ = (phase, words);
}

/// Freezes status-2 ownership history on the first terminal MAC event.
#[inline(always)]
pub unsafe fn freeze_status2_snapshots() {
    #[cfg(feature = "vendor-host-tx-diagnostics")]
    unsafe {
        core::ptr::addr_of_mut!((*STATUS2_RECORDER.0.get()).frozen).write_volatile(1);
    }
}

/// Returns the newest complete status-2 snapshot and its commit metadata.
#[inline(always)]
pub fn latest_status2_snapshot() -> (u32, u32, [u32; STATUS2_SNAPSHOT_WORDS]) {
    #[cfg(feature = "vendor-host-tx-diagnostics")]
    unsafe {
        let recorder = STATUS2_RECORDER.0.get();
        let cursor = core::ptr::addr_of!((*recorder).cursor).read_volatile();
        if cursor == 0 {
            return (0, 0, [0; STATUS2_SNAPSHOT_WORDS]);
        }
        let source = core::ptr::addr_of!(
            (*recorder).records[cursor.wrapping_sub(1) as usize % STATUS2_RECORD_COUNT]
        );
        if core::ptr::addr_of!((*source).sequence).read_volatile() != cursor {
            return (0, 0, [0; STATUS2_SNAPSHOT_WORDS]);
        }
        let phase = core::ptr::addr_of!((*source).phase).read_volatile();
        let mut words = [0; STATUS2_SNAPSHOT_WORDS];
        for (index, destination) in words.iter_mut().enumerate() {
            *destination = core::ptr::addr_of!((*source).words[index]).read_volatile();
        }
        (cursor, phase, words)
    }
    #[cfg(not(feature = "vendor-host-tx-diagnostics"))]
    {
        (0, 0, [0; STATUS2_SNAPSHOT_WORDS])
    }
}

/// Replaces the latest pre-GO image. The sequence is committed after every
/// slot, PAS, command, ring, and pipe-state word has been copied into BSS.
#[inline(always)]
pub unsafe fn begin_pre_go_snapshot() {
    #[cfg(feature = "vendor-host-tx-diagnostics")]
    unsafe {
        let snapshot = PRE_GO_SNAPSHOT.0.get();
        let sequence = core::ptr::addr_of!((*snapshot).sequence)
            .read_volatile()
            .wrapping_add(1)
            .max(1);
        core::ptr::addr_of_mut!((*snapshot).sequence).write_volatile(0);
        PRE_GO_PENDING_SEQUENCE.0.get().write_volatile(sequence);
    }
}

#[inline(always)]
pub unsafe fn write_pre_go_snapshot_word(index: usize, value: u32) {
    #[cfg(feature = "vendor-host-tx-diagnostics")]
    unsafe {
        let words = core::ptr::addr_of_mut!((*PRE_GO_SNAPSHOT.0.get()).words).cast::<u32>();
        words.add(index).write_volatile(value);
    }
    #[cfg(not(feature = "vendor-host-tx-diagnostics"))]
    let _ = (index, value);
}

#[inline(always)]
pub unsafe fn commit_pre_go_snapshot() {
    #[cfg(feature = "vendor-host-tx-diagnostics")]
    unsafe {
        let sequence = PRE_GO_PENDING_SEQUENCE.0.get().read_volatile();
        core::ptr::addr_of_mut!((*PRE_GO_SNAPSHOT.0.get()).sequence).write_volatile(sequence);
    }
}

/// Returns the latest complete pre-GO image.
#[inline(always)]
pub fn latest_pre_go_snapshot() -> (u32, [u32; PRE_GO_SNAPSHOT_WORDS]) {
    #[cfg(feature = "vendor-host-tx-diagnostics")]
    unsafe {
        let snapshot = PRE_GO_SNAPSHOT.0.get();
        let sequence = core::ptr::addr_of!((*snapshot).sequence).read_volatile();
        if sequence == 0 {
            return (0, [0; PRE_GO_SNAPSHOT_WORDS]);
        }
        let mut words = [0; PRE_GO_SNAPSHOT_WORDS];
        for (index, destination) in words.iter_mut().enumerate() {
            *destination = core::ptr::addr_of!((*snapshot).words[index]).read_volatile();
        }
        if core::ptr::addr_of!((*snapshot).sequence).read_volatile() != sequence {
            return (0, [0; PRE_GO_SNAPSHOT_WORDS]);
        }
        (sequence, words)
    }
    #[cfg(not(feature = "vendor-host-tx-diagnostics"))]
    {
        (0, [0; PRE_GO_SNAPSHOT_WORDS])
    }
}

/// Records and freezes the first firmware-visible invariant failure.
#[inline(always)]
pub unsafe fn freeze(reason: u16, arg0: u32, arg1: u32, state: u32) {
    #[cfg(feature = "vendor-host-tx-diagnostics")]
    unsafe {
        let recorder = FLIGHT_RECORDER.0.get();
        if core::ptr::addr_of!((*recorder).frozen).read_volatile() == 0 {
            record(EVENT_INVARIANT_FAILURE, reason, arg0, arg1, state);
            core::ptr::addr_of_mut!((*recorder).freeze_reason).write_volatile(u32::from(reason));
            core::ptr::addr_of_mut!((*recorder).frozen).write_volatile(1);
        }
    }
    #[cfg(not(feature = "vendor-host-tx-diagnostics"))]
    let _ = (reason, arg0, arg1, state);
}

#[inline(always)]
pub unsafe fn record_hif_event(event: u8, id: u16, detail: u8) {
    unsafe {
        record(
            EVENT_TX_LIFECYCLE,
            u16::from(event),
            u32::from(id),
            u32::from(detail),
            0,
        )
    };
}

#[inline(always)]
pub unsafe fn trace(stage: u32, value0: u32, value1: u32) {
    #[cfg(feature = "vendor-host-tx-diagnostics")]
    unsafe {
        let snapshot = &mut *LEGACY_SNAPSHOT.0.get();
        snapshot.stage = stage;
        snapshot.value0 = value0;
        snapshot.value1 = value1;
        record(EVENT_TX_LIFECYCLE, 0, stage, value0, value1);
    }
    #[cfg(not(feature = "vendor-host-tx-diagnostics"))]
    let _ = (stage, value0, value1);
}

/// Preserve the completed frame header before its borrowed HIF buffer returns.
#[inline(always)]
pub unsafe fn capture_retry_feedback(
    context: crate::dtcm::HostContextAddress,
    status: u32,
    tx_rate: u8,
    ack_failures: u8,
) {
    #[cfg(feature = "vendor-host-tx-diagnostics")]
    unsafe {
        trace(
            0x4854_5200 | u32::from(tx_rate),
            crate::dtcm::shared_ptr::<u32>(context.rate_try(0).unwrap()).read_volatile(),
            crate::dtcm::shared_ptr::<u32>(context.rate_try(1).unwrap()).read_volatile(),
        );
        let snapshot = &mut *LEGACY_SNAPSHOT.0.get();
        snapshot.frame_address =
            crate::dtcm::shared_ptr::<u32>(context.rate_try(2).unwrap()).read_volatile();
        snapshot.frame_metadata =
            (status & 0xffff) | (u32::from(ack_failures) << 16) | (u32::from(tx_rate) << 24);
    }
    #[cfg(not(feature = "vendor-host-tx-diagnostics"))]
    let _ = (context, status, tx_rate, ack_failures);
}

/// Records the latest host submission identity for counters-MIB diagnostics.
///
/// # Safety
/// The caller must serialize access with the foreground host-TX runtime.
pub unsafe fn capture_submission_identity(
    packet_id: u32,
    context: crate::dtcm::HostContextAddress,
    request_flags: u8,
    priority: u8,
    ac: u8,
    tx_flags: u32,
) {
    #[cfg(feature = "vendor-host-tx-diagnostics")]
    unsafe {
        let values = &mut *TX_IDENTITY.0.get();
        values[0] = 0x5854_4944; // "XTID"
        values[1] = values[1].wrapping_add(1);
        values[2] = packet_id;
        values[3] = context.raw();
        values[4] = u32::from(request_flags) | (u32::from(priority) << 8) | (u32::from(ac) << 16);
        values[5] = tx_flags;
    }
    #[cfg(not(feature = "vendor-host-tx-diagnostics"))]
    let _ = (packet_id, context, request_flags, priority, ac, tx_flags);
}

/// Records the exact context and hardware slot crossing publication.
///
/// # Safety
/// The caller must own the retained context and scheduler reservation.
pub unsafe fn capture_publication_identity(packet_id: u32, context: u32, pipe: u8, slot: u8) {
    #[cfg(feature = "vendor-host-tx-diagnostics")]
    unsafe {
        let values = &mut *TX_IDENTITY.0.get();
        values[6] = values[6].wrapping_add(1);
        values[7] = packet_id;
        values[8] = context;
        values[9] = u32::from(pipe) | (u32::from(slot) << 8);
    }
    #[cfg(not(feature = "vendor-host-tx-diagnostics"))]
    let _ = (packet_id, context, pipe, slot);
}

/// Records the MAC completion resolved for one retained host identity.
///
/// # Safety
/// `context` must remain live while the completion is recorded.
pub unsafe fn capture_completion_identity(packet_id: u32, context: u32, status: u16, retries: u8) {
    #[cfg(feature = "vendor-host-tx-diagnostics")]
    unsafe {
        let values = &mut *TX_IDENTITY.0.get();
        values[10] = values[10].wrapping_add(1);
        values[11] = packet_id;
        values[12] = context;
        values[13] = u32::from(status) | (u32::from(retries) << 16);
    }
    #[cfg(not(feature = "vendor-host-tx-diagnostics"))]
    let _ = (packet_id, context, status, retries);
}

/// Records one truthful HIF confirmation publication attempt.
///
/// # Safety
/// The caller must still own `context` and its retained request token.
pub unsafe fn capture_confirmation_identity(
    packet_id: u32,
    context: u32,
    status: u32,
    retries: u8,
) {
    #[cfg(feature = "vendor-host-tx-diagnostics")]
    unsafe {
        let values = &mut *TX_IDENTITY.0.get();
        values[18] = if values[15] == packet_id && values[16] == context {
            values[18].wrapping_add(1)
        } else {
            0
        };
        values[19] = values[15];
        values[20] = values[16];
        values[14] = values[14].wrapping_add(1);
        values[15] = packet_id;
        values[16] = context;
        values[17] = (status & 0xffff) | (u32::from(retries) << 16);
        // values[21] carries the pipe cursor invariant; see `capture_pipe_cursor`.
    }
    #[cfg(not(feature = "vendor-host-tx-diagnostics"))]
    let _ = (packet_id, context, status, retries);
}

/// Records the vendor `txp_fn_4425` cursor invariant for the most recent
/// class-0 publication, in identity slot 21.
///
/// Nibbles 0..6 hold the packed pipe/cursor state from
/// `tx::pipe_cursor_diagnostic`; the top nibble is a saturating count of
/// observed producer/ring-cursor divergences. A non-zero top nibble means the
/// software producer and the hardware ring cursor disagreed at least once.
///
/// # Safety
/// The caller must serialize access with the foreground host-TX runtime.
#[inline(always)]
pub unsafe fn capture_pipe_cursor(packed: u32, diverged: bool) {
    #[cfg(feature = "vendor-host-tx-diagnostics")]
    unsafe {
        let values = &mut *TX_IDENTITY.0.get();
        let seen = values[21] >> 28;
        let seen = if diverged { (seen + 1).min(0x0f) } else { seen };
        values[21] = (packed & 0x0fff_ffff) | (seen << 28);
    }
    #[cfg(not(feature = "vendor-host-tx-diagnostics"))]
    let _ = (packed, diverged);
}

pub unsafe fn capture_completion(
    context: crate::dtcm::HostContextAddress,
    status: u16,
    retries: u8,
) {
    #[cfg(feature = "vendor-host-tx-diagnostics")]
    unsafe {
        trace(
            0x4854_3000,
            context.raw(),
            u32::from(status) | (u32::from(retries) << 16),
        );
        let snapshot = &mut *LEGACY_SNAPSHOT.0.get();
        let frame = crate::dtcm::shared_ptr::<u32>(context.frame_address()).read_volatile();
        snapshot.frame_address = frame;
        snapshot.frame_metadata =
            crate::dtcm::shared_ptr::<u32>(context.frame_length()).read_volatile();
        for (index, destination) in snapshot.frame_words.iter_mut().enumerate() {
            *destination = (frame.wrapping_add(index as u32 * 4) as *const u32).read_volatile();
        }
    }
    #[cfg(not(feature = "vendor-host-tx-diagnostics"))]
    let _ = (context, status, retries);
}

#[inline(always)]
pub unsafe fn capture_descriptor_length(word: u32) {
    #[cfg(feature = "vendor-host-tx-diagnostics")]
    unsafe {
        (*LEGACY_SNAPSHOT.0.get()).descriptor_length = word;
    }
    #[cfg(not(feature = "vendor-host-tx-diagnostics"))]
    let _ = word;
}

/// Exposes recorder metadata and the newest three complete records through the
/// existing counters MIB until a paginated diagnostic transport is added.
#[inline(always)]
pub fn populate_counters(values: &mut [u32; 22], transport: &crate::hif::Transport) {
    {
        let snapshot = crate::crypto::hardware_ccmp_selftest_snapshot();
        if snapshot[0] == 0x4857_434b {
            let _ = transport;
            values.fill(0);
            values[..snapshot.len()].copy_from_slice(&snapshot);
            #[cfg(feature = "vendor-host-tx-diagnostics")]
            unsafe {
                #[cfg(feature = "experimental-depth-two-ampdu")]
                {
                    let counters = LIFECYCLE_COUNTERS.0.get().cast::<u32>();
                    values[14] = counters.add(counter::PUBLISHED).read_volatile();
                    values[15] = counters.add(counter::TX_START).read_volatile();
                    values[16] = counters.add(counter::STATUS).read_volatile();
                    values[17] = counters.add(counter::COMPLETED).read_volatile();
                    #[cfg(not(feature = "experimental-depth-four-ampdu"))]
                    {
                        values[18] = (*LEGACY_SNAPSHOT.0.get()).descriptor_length;
                    }
                }
                #[cfg(all(
                    feature = "experimental-depth-two-ampdu",
                    not(feature = "experimental-depth-four-ampdu")
                ))]
                {
                    let (count, last, seen) = crate::tx::ineligible_tx_status_snapshot();
                    values[19] = count;
                    values[20] = last;
                    values[21] = seen;
                }
                #[cfg(feature = "experimental-depth-four-ampdu")]
                {
                    let telemetry = AMPDU_DEPTH_TELEMETRY.0.get().cast::<u32>();
                    for index in 0..4 {
                        values[18 + index] = telemetry.add(index).read_volatile();
                    }
                }
                #[cfg(not(feature = "experimental-depth-two-ampdu"))]
                {
                    values[19] = AMPDU_CANDIDATES.0.get().read_volatile();
                    values[20] = BATCH_PUBLICATIONS.0.get().read_volatile();
                }
            }
            return;
        }
    }
    #[cfg(feature = "vendor-host-tx-diagnostics")]
    unsafe {
        let _ = transport;
        let identity = &*TX_IDENTITY.0.get();
        if identity[0] == 0x5854_4944 {
            values.copy_from_slice(identity);
            return;
        }
        let recorder = FLIGHT_RECORDER.0.get();
        let cursor = core::ptr::addr_of!((*recorder).cursor).read_volatile();
        values.fill(0);
        values[0] = cursor;
        values[1] = core::ptr::addr_of!((*recorder).frozen).read_volatile();
        values[2] = core::ptr::addr_of!((*recorder).freeze_reason).read_volatile();
        values[3] = FLIGHT_RECORD_COUNT as u32;
        let available = cursor.min(3);
        for record_index in 0..available {
            let sequence = cursor.wrapping_sub(record_index + 1);
            let source =
                core::ptr::addr_of!((*recorder).records[sequence as usize % FLIGHT_RECORD_COUNT]);
            let base = 4 + record_index as usize * 6;
            values[base] = core::ptr::addr_of!((*source).sequence).read_volatile();
            values[base + 1] = core::ptr::addr_of!((*source).timestamp).read_volatile();
            values[base + 2] = core::ptr::addr_of!((*source).event_flags).read_volatile();
            values[base + 3] = core::ptr::addr_of!((*source).arg0).read_volatile();
            values[base + 4] = core::ptr::addr_of!((*source).arg1).read_volatile();
            values[base + 5] = core::ptr::addr_of!((*source).state).read_volatile();
        }
    }
    #[cfg(not(feature = "vendor-host-tx-diagnostics"))]
    let _ = (values, transport);
}

#[cfg(all(test, feature = "vendor-host-tx-diagnostics"))]
mod tests {
    #[test]
    fn fixed_record_layout_is_six_words() {
        assert_eq!(core::mem::size_of::<super::FlightRecord>(), 24);
        assert_eq!(core::mem::size_of::<super::Status2Record>(), 120);
        assert_eq!(core::mem::size_of::<super::PreGoSnapshot>(), 156);
    }

    #[test]
    fn retry_feedback_sums_packed_failure_nibbles() {
        assert_eq!(super::rate_try_failure_count([0, 0, 0]), 0);
        assert_eq!(super::rate_try_failure_count([0x1000_0021, 0x0000_3000, 0]), 7);
        assert_eq!(super::rate_try_failure_count([u32::MAX; 3]), u8::MAX);
    }
}
