//! Optional retained diagnostics for the vendor-shaped host TX runtime.
//!
//! The hardware lifecycle must not depend on this module. With
//! `vendor-host-tx-diagnostics` disabled every writer is an inline no-op and
//! the counters MIB retains its ordinary radio/scan contents.

#[cfg(feature = "vendor-host-tx-diagnostics")]
const TRACE_STAGE: usize = 0x0900_ff60;
#[cfg(feature = "vendor-host-tx-diagnostics")]
const TRACE_VALUE0: usize = 0x0900_ff64;
#[cfg(feature = "vendor-host-tx-diagnostics")]
const TRACE_VALUE1: usize = 0x0900_ff68;
#[cfg(feature = "vendor-host-tx-diagnostics")]
const FRAME_ADDRESS: usize = 0x0900_ff6c;
#[cfg(feature = "vendor-host-tx-diagnostics")]
const FRAME_METADATA: usize = 0x0900_ff70;
#[cfg(feature = "vendor-host-tx-diagnostics")]
const FRAME_WORDS: usize = 0x0900_ff74;
#[cfg(feature = "vendor-host-tx-diagnostics")]
const DESCRIPTOR_LENGTH_WORD: usize = 0x0900_ff94;

#[inline(always)]
pub unsafe fn trace(stage: u32, value0: u32, value1: u32) {
    #[cfg(feature = "vendor-host-tx-diagnostics")]
    unsafe {
        (TRACE_STAGE as *mut u32).write_volatile(stage);
        (TRACE_VALUE0 as *mut u32).write_volatile(value0);
        (TRACE_VALUE1 as *mut u32).write_volatile(value1);
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
        (FRAME_ADDRESS as *mut u32)
            .write_volatile((context.wrapping_add(0x30) as *const u32).read_volatile());
        (FRAME_METADATA as *mut u32).write_volatile(
            (status & 0xffff) | (u32::from(ack_failures) << 16) | (u32::from(tx_rate) << 24),
        );
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
        let frame = (context.wrapping_add(0x54) as *const u32).read_volatile();
        (FRAME_ADDRESS as *mut u32).write_volatile(frame);
        (FRAME_METADATA as *mut u32)
            .write_volatile((context.wrapping_add(0x5c) as *const u32).read_volatile());
        for index in 0..8_u32 {
            let offset = index * 4;
            let word = u32::from((frame.wrapping_add(offset) as *const u8).read_volatile())
                | (u32::from((frame.wrapping_add(offset + 1) as *const u8).read_volatile()) << 8)
                | (u32::from((frame.wrapping_add(offset + 2) as *const u8).read_volatile()) << 16)
                | (u32::from((frame.wrapping_add(offset + 3) as *const u8).read_volatile()) << 24);
            (FRAME_WORDS.wrapping_add(offset as usize) as *mut u32).write_volatile(word);
        }
    }
    #[cfg(not(feature = "vendor-host-tx-diagnostics"))]
    let _ = (context, status, retries);
}

#[inline(always)]
pub unsafe fn capture_descriptor_length(word: u32) {
    #[cfg(feature = "vendor-host-tx-diagnostics")]
    unsafe {
        (DESCRIPTOR_LENGTH_WORD as *mut u32).write_volatile(word);
    }
    #[cfg(not(feature = "vendor-host-tx-diagnostics"))]
    let _ = word;
}

/// Overlay the optional host-TX trace onto the established counters MIB.
#[inline(always)]
pub fn populate_counters(values: &mut [u32; 22], transport: &crate::hif::Transport) {
    #[cfg(feature = "vendor-host-tx-diagnostics")]
    unsafe {
        let _ = transport;
        values[8] = (TRACE_STAGE as *const u32).read_volatile();
        values[9] = (TRACE_VALUE0 as *const u32).read_volatile();
        values[10] = (TRACE_VALUE1 as *const u32).read_volatile();
        values[11] = (FRAME_ADDRESS as *const u32).read_volatile();
        values[12] = (FRAME_METADATA as *const u32).read_volatile();
        for index in 0..8_usize {
            values[13 + index] =
                (FRAME_WORDS.wrapping_add(index * 4) as *const u32).read_volatile();
        }
        values[21] = (DESCRIPTOR_LENGTH_WORD as *const u32).read_volatile();
    }
    #[cfg(not(feature = "vendor-host-tx-diagnostics"))]
    let _ = (values, transport);
}
