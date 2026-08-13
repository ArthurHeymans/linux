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
pub const EVENT_INVARIANT_FAILURE: u16 = 0xff01;

#[cfg(feature = "vendor-host-tx-diagnostics")]
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
pub unsafe fn capture_retry_feedback(context: u32, status: u32, tx_rate: u8, ack_failures: u8) {
    #[cfg(feature = "vendor-host-tx-diagnostics")]
    unsafe {
        trace(
            0x4854_5200 | u32::from(tx_rate),
            (context.wrapping_add(0x28) as *const u32).read_volatile(),
            (context.wrapping_add(0x2c) as *const u32).read_volatile(),
        );
        let snapshot = &mut *LEGACY_SNAPSHOT.0.get();
        snapshot.frame_address = (context.wrapping_add(0x30) as *const u32).read_volatile();
        snapshot.frame_metadata =
            (status & 0xffff) | (u32::from(ack_failures) << 16) | (u32::from(tx_rate) << 24);
    }
    #[cfg(not(feature = "vendor-host-tx-diagnostics"))]
    let _ = (context, status, tx_rate, ack_failures);
}

pub unsafe fn capture_completion(context: u32, status: u16, retries: u8) {
    #[cfg(feature = "vendor-host-tx-diagnostics")]
    unsafe {
        trace(
            0x4854_3000,
            context,
            u32::from(status) | (u32::from(retries) << 16),
        );
        let snapshot = &mut *LEGACY_SNAPSHOT.0.get();
        let frame = (context.wrapping_add(0x54) as *const u32).read_volatile();
        snapshot.frame_address = frame;
        snapshot.frame_metadata = (context.wrapping_add(0x5c) as *const u32).read_volatile();
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
    #[cfg(feature = "vendor-host-tx-diagnostics")]
    unsafe {
        let _ = transport;
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
    }
}
