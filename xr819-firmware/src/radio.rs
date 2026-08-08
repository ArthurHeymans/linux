//! Polling receive path for the XR819 packet-DMA RX FIFO.
//!
//! This follows vendor `rxfifo_next_frame` (`0x08af2`),
//! `rx_indication_build_and_send` (`0xb054`), and `rxfifo_release_slot`
//! (`0x08c22`). Accepted scan frames are published zero-copy: the 16-byte WSM
//! receive header is written into the FIFO slot's existing headroom and the slot
//! remains owned by firmware until the corresponding HIF TX descriptor is
//! reclaimed.

use core::cell::UnsafeCell;

const FIFO_BASE: usize = 0x0940_0000;
const FIFO_SIZE: u32 = 0x7000;
const FIFO_MASK: u32 = 0x0001_fffc;
const FIFO_MAGIC: u32 = 0x00aa_55ff;
const FIFO_RELEASED: u32 = 0xcccc_cc00;
const DMA_PRODUCER: *const u32 = 0x09c0_0604 as *const u32;
const DMA_CONSUMER: *mut u32 = 0x09c0_0608 as *mut u32;
const VENDOR_FIFO_STATE: usize = 0x0400_1680;
const MAX_FRAME_LEN: usize = 1600;
const WSM_RX_HEADROOM: usize = 16;

static mut CONSUMER_OFFSET: u32 = 0;
static mut HOST_TRANSFER_OUTSTANDING: bool = false;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ReceiveDiagnostics {
    pub producer_changes: u32,
    pub valid_slots: u32,
    pub bad_magic: u32,
    pub malformed_slots: u32,
    pub filtered_frames: u32,
    pub oversized_frames: u32,
    pub indications: u32,
    pub released_slots: u32,
    pub last_producer: u32,
}

struct SharedDiagnostics(UnsafeCell<ReceiveDiagnostics>);

unsafe impl Sync for SharedDiagnostics {}

static DIAGNOSTICS: SharedDiagnostics = SharedDiagnostics(UnsafeCell::new(ReceiveDiagnostics {
    producer_changes: 0,
    valid_slots: 0,
    bad_magic: 0,
    malformed_slots: 0,
    filtered_frames: 0,
    oversized_frames: 0,
    indications: 0,
    released_slots: 0,
    last_producer: 0,
}));

pub fn diagnostics() -> ReceiveDiagnostics {
    unsafe { *DIAGNOSTICS.0.get() }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReleaseToken {
    slot: u32,
    next: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PendingIndication {
    pub address: u32,
    pub length: u16,
    pub release: ReleaseToken,
}

/// Initializes the software side of the circular RX FIFO before RX is enabled.
///
/// # Safety
/// Packet DMA must already be initialized and must not have an active consumer.
pub unsafe fn initialize() {
    unsafe {
        CONSUMER_OFFSET = 0;
        HOST_TRANSFER_OUTSTANDING = false;
        ((VENDOR_FIFO_STATE + 0x10) as *mut u32).write_volatile(0);
        ((VENDOR_FIFO_STATE + 0x14) as *mut u32).write_volatile(0);
        ((VENDOR_FIFO_STATE + 0x18) as *mut u32).write_volatile(0);
        DMA_CONSUMER.write_volatile(0);
        *DIAGNOSTICS.0.get() = ReceiveDiagnostics::default();
    }
}

fn scan_frame_flags(frame_control: u16) -> Option<u32> {
    match frame_control & 0x00fc {
        0x0080 => Some(1 << 7),
        0x0050 => Some(0),
        _ => None,
    }
}

fn next_offset(offset: u32, slot_length: u16) -> u32 {
    let mut next = offset
        .wrapping_add(u32::from(slot_length))
        .wrapping_add(0x2f)
        & FIFO_MASK;
    if next >= FIFO_SIZE {
        next -= FIFO_SIZE;
    }
    next
}

unsafe fn release(token: ReleaseToken) {
    unsafe {
        let slot = token.slot as usize;
        let state = (slot + 8) as *mut u32;
        let low = state.read_volatile() & 0xff;
        state.write_volatile(FIFO_RELEASED | low);
        (slot as *mut u32).write_volatile(0);
        DMA_CONSUMER.write_volatile(token.next);
        CONSUMER_OFFSET = token.next;
        ((VENDOR_FIFO_STATE + 0x10) as *mut u32).write_volatile(token.next);
        ((VENDOR_FIFO_STATE + 0x14) as *mut u32).write_volatile(token.next);
        let diagnostics = &mut *DIAGNOSTICS.0.get();
        diagnostics.released_slots = diagnostics.released_slots.wrapping_add(1);
    }
}

/// Releases a zero-copy FIFO indication after HIF confirms descriptor ownership
/// has returned to firmware.
///
/// # Safety
/// `token` must be the currently outstanding radio transfer token.
pub unsafe fn complete_host_transfer(token: ReleaseToken) {
    unsafe {
        if HOST_TRANSFER_OUTSTANDING {
            HOST_TRANSFER_OUTSTANDING = false;
            release(token);
        }
    }
}

unsafe fn write_u16(address: usize, value: u16) {
    unsafe { (address as *mut u16).write_volatile(value) };
}

unsafe fn write_u32(address: usize, value: u32) {
    unsafe { (address as *mut u32).write_volatile(value) };
}

/// Polls one packet-DMA frame and prepares a zero-copy WSM receive indication.
///
/// Only one FIFO-backed host transfer is allowed at a time. This preserves the
/// sequential release rule without needing the vendor's general out-of-order
/// reference-count machinery. Unwanted frames are released immediately.
///
/// # Safety
/// The fixed packet-memory window and packet-DMA registers must be accessible.
pub unsafe fn poll_scan_indication(active_channel: u16) -> Option<PendingIndication> {
    if unsafe { HOST_TRANSFER_OUTSTANDING } {
        return None;
    }

    let consumer = unsafe { CONSUMER_OFFSET };
    let producer = unsafe { DMA_PRODUCER.read_volatile() } & FIFO_MASK;
    if consumer == producer {
        return None;
    }
    unsafe {
        let diagnostics = &mut *DIAGNOSTICS.0.get();
        if diagnostics.last_producer != producer {
            diagnostics.last_producer = producer;
            diagnostics.producer_changes = diagnostics.producer_changes.wrapping_add(1);
        }
    }

    let slot = FIFO_BASE + consumer as usize;
    if unsafe { (slot as *const u32).read_volatile() } != FIFO_MAGIC {
        unsafe {
            let diagnostics = &mut *DIAGNOSTICS.0.get();
            diagnostics.bad_magic = diagnostics.bad_magic.wrapping_add(1);
        }
        return None;
    }
    unsafe {
        let diagnostics = &mut *DIAGNOSTICS.0.get();
        diagnostics.valid_slots = diagnostics.valid_slots.wrapping_add(1);
    }

    let slot_length = unsafe { ((slot + 0x18) as *const u16).read_volatile() };
    if slot_length < 4 || usize::from(slot_length) > MAX_FRAME_LEN + 4 {
        unsafe {
            let diagnostics = &mut *DIAGNOSTICS.0.get();
            diagnostics.malformed_slots = diagnostics.malformed_slots.wrapping_add(1);
            if usize::from(slot_length) > MAX_FRAME_LEN + 4 {
                diagnostics.oversized_frames = diagnostics.oversized_frames.wrapping_add(1);
            }
        }
        return None;
    }

    let frame_len = usize::from(slot_length) - 4;
    if consumer as usize + 0x20 + usize::from(slot_length) + 8 > FIFO_SIZE as usize {
        unsafe {
            let diagnostics = &mut *DIAGNOSTICS.0.get();
            diagnostics.malformed_slots = diagnostics.malformed_slots.wrapping_add(1);
        }
        return None;
    }

    let next = next_offset(consumer, slot_length);
    unsafe { ((slot + 4) as *mut u32).write_volatile(next) };
    let token = ReleaseToken {
        slot: slot as u32,
        next,
    };

    let frame_address = slot + 0x20;
    let trailer = (frame_address + usize::from(slot_length) + 3) & !3;
    let channel = unsafe { ((trailer + 2) as *const u16).read_volatile() } & 0x03ff;
    let rcpi = unsafe { ((trailer + 7) as *const u8).read_volatile() }.max(1);
    let frame_control = if frame_len >= 2 {
        unsafe { (frame_address as *const u16).read_volatile() }
    } else {
        0xffff
    };
    let indication_flags = scan_frame_flags(frame_control);

    if frame_len < 24 || channel != active_channel || indication_flags.is_none() {
        unsafe {
            let diagnostics = &mut *DIAGNOSTICS.0.get();
            diagnostics.filtered_frames = diagnostics.filtered_frames.wrapping_add(1);
            release(token);
        }
        return None;
    }

    let message_address = frame_address - WSM_RX_HEADROOM;
    let message_length = frame_len + WSM_RX_HEADROOM;
    unsafe {
        write_u16(message_address, message_length as u16);
        write_u16(message_address + 2, 0x0804);
        write_u32(message_address + 4, 0);
        write_u16(message_address + 8, channel);
        ((message_address + 10) as *mut u8).write_volatile(0);
        ((message_address + 11) as *mut u8).write_volatile(rcpi);
        write_u32(message_address + 12, indication_flags.unwrap_or(0));
        HOST_TRANSFER_OUTSTANDING = true;
        let diagnostics = &mut *DIAGNOSTICS.0.get();
        diagnostics.indications = diagnostics.indications.wrapping_add(1);
    }

    Some(PendingIndication {
        address: message_address as u32,
        length: message_length as u16,
        release: token,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vendor_fifo_advance_alignment_and_wrap() {
        assert_eq!(next_offset(0, 100), 0x90);
        assert_eq!(next_offset(0x6fc0, 64), 0x2c);
    }

    #[test]
    fn scan_filter_accepts_only_beacons_and_probe_responses() {
        assert_eq!(scan_frame_flags(0x0080), Some(1 << 7));
        assert_eq!(scan_frame_flags(0x0050), Some(0));
        assert_eq!(scan_frame_flags(0x0008), None);
        assert_eq!(scan_frame_flags(0x00d0), None);
    }

    #[test]
    fn zero_copy_indication_fits_advertised_hif_buffer() {
        assert_eq!(MAX_FRAME_LEN + WSM_RX_HEADROOM, 1616);
        assert!(MAX_FRAME_LEN + WSM_RX_HEADROOM <= 1632);
    }
}
