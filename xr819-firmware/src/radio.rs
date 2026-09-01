//! Polling receive path for the XR819 packet-DMA RX FIFO.
//!
//! This follows vendor `rxfifo_next_frame` (`0x08af2`),
//! `rx_indication_build_and_send` (`0xb054`), and `rxfifo_release_slot`
//! (`0x08c22`). Accepted scan frames are published zero-copy: the 16-byte WSM
//! receive header is written into the FIFO slot's existing headroom and the slot
//! remains owned by firmware until the corresponding HIF TX descriptor is
//! reclaimed.

use core::cell::UnsafeCell;

use crate::packet_ram;
pub use crate::rx_model::RxToken;
use crate::rx_model::{ReleaseAction, RxRing};

const FIFO_SIZE: u32 = packet_ram::RX_FIFO_LOGICAL_SIZE as u32;
// Vendor `rxfifo_off_to_addr()` wraps slot starts at 0x7000, but the current
// slot remains linearly addressable in the following packet-RAM spill area.
// The ring keeps 0x1000 bytes outside its logical cursor range for this.
const FIFO_STORAGE_SIZE: usize = packet_ram::RX_FIFO_STORAGE_SIZE;
const FIFO_MASK: u32 = 0x0001_fffc;
const FIFO_MAGIC: u32 = 0x00aa_55ff;
const FIFO_RELEASED: u32 = 0xcccc_cc00;
const DMA_PRODUCER: *const u32 = crate::platform::mac_register(0x0604) as *const u32;
const DMA_CONSUMER: *mut u32 = crate::platform::mac_register(0x0608) as *mut u32;

#[inline(always)]
fn fifo_base() -> usize {
    packet_ram::rx_fifo_base()
}
const WSM_RX_HEADROOM: usize = 16;
// The vendor accepts any complete FIFO slot whose length fits the HIF
// descriptor. A 1600-byte cap incorrectly rejected valid 1840-byte slots and
// resynchronized the release cursor across host-owned zero-copy indications.
const MAX_WSM_RX_MESSAGE_LEN: usize = 0x1ffe;
const MAX_FRAME_LEN: usize = MAX_WSM_RX_MESSAGE_LEN - WSM_RX_HEADROOM;

// A resync may move the parser past corrupt bytes while older zero-copy slots
// remain host-owned. One owner keeps release/claim cursors, live-token count,
// and the deferred hardware-consumer update coherent.
struct SharedRxRing(UnsafeCell<RxRing>);

unsafe impl Sync for SharedRxRing {}

static RX_RING: SharedRxRing = SharedRxRing(UnsafeCell::new(RxRing::new()));
const MAX_HOST_TRANSFERS: u32 = 24;

/// # Safety
/// The cooperative firmware loop must be the sole RX-ring mutator.
unsafe fn rx_ring() -> &'static mut RxRing {
    unsafe { &mut *RX_RING.0.get() }
}

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
    pub last_slot_length: u16,
    pub last_frame_control: u16,
    pub last_channel: u16,
    pub last_active_channel: u16,
    pub last_trailer_word: u32,
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
    last_slot_length: 0,
    last_frame_control: 0,
    last_channel: 0,
    last_active_channel: 0,
    last_trailer_word: 0,
}));

#[cfg(feature = "vendor-host-tx-diagnostics")]
struct SharedRxTxBoundaryWatch(UnsafeCell<[u32; 7]>);

#[cfg(feature = "vendor-host-tx-diagnostics")]
unsafe impl Sync for SharedRxTxBoundaryWatch {}

#[cfg(feature = "vendor-host-tx-diagnostics")]
static RX_TX_BOUNDARY_WATCH: SharedRxTxBoundaryWatch =
    SharedRxTxBoundaryWatch(UnsafeCell::new([0; 7]));

#[cfg(feature = "vendor-host-tx-diagnostics")]
struct SharedRxTxProducerWatch(UnsafeCell<[u32; 2]>);

#[cfg(feature = "vendor-host-tx-diagnostics")]
unsafe impl Sync for SharedRxTxProducerWatch {}

#[cfg(feature = "vendor-host-tx-diagnostics")]
static RX_TX_PRODUCER_WATCH: SharedRxTxProducerWatch =
    SharedRxTxProducerWatch(UnsafeCell::new([0; 2]));

#[cfg(feature = "vendor-host-tx-diagnostics")]
struct SharedTxCommandSignatures(UnsafeCell<[u32; 51]>);

#[cfg(feature = "vendor-host-tx-diagnostics")]
unsafe impl Sync for SharedTxCommandSignatures {}

#[cfg(feature = "vendor-host-tx-diagnostics")]
static TX_COMMAND_SIGNATURES: SharedTxCommandSignatures =
    SharedTxCommandSignatures(UnsafeCell::new([0; 51]));

#[cfg(feature = "vendor-host-tx-diagnostics")]
struct SharedClass0CommandHistory(UnsafeCell<[u32; 186]>);

#[cfg(feature = "vendor-host-tx-diagnostics")]
unsafe impl Sync for SharedClass0CommandHistory {}

#[cfg(feature = "vendor-host-tx-diagnostics")]
static CLASS0_COMMAND_HISTORY: SharedClass0CommandHistory =
    SharedClass0CommandHistory(UnsafeCell::new([0; 186]));

pub fn diagnostics() -> ReceiveDiagnostics {
    unsafe { *DIAGNOSTICS.0.get() }
}

/// Compact scan-completion telemetry: producer changes, valid slots,
/// indications, and malformed/bad-magic flags in four nibbles.
pub fn diagnostic_word() -> u16 {
    let value = diagnostics();
    let errors = u16::from(value.malformed_slots != 0)
        | (u16::from(value.bad_magic != 0) << 1)
        | (u16::from(value.oversized_frames != 0) << 2);
    (value.producer_changes.min(15) as u16)
        | ((value.valid_slots.min(15) as u16) << 4)
        | ((value.indications.min(15) as u16) << 8)
        | (errors << 12)
}

pub fn host_transfer_outstanding() -> bool {
    unsafe { rx_ring().host_transfer_count() != 0 }
}

pub fn fifo_quiescent() -> bool {
    unsafe {
        let ring = rx_ring();
        ring.host_transfer_count() == 0
            && ring.release_offset() == ring.claim_offset()
            && ring.claim_offset() == DMA_PRODUCER.read_volatile() & FIFO_MASK
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct PendingIndication {
    pub address: u32,
    pub length: u16,
    pub release: RxToken,
}

/// Initializes the software side of the circular RX FIFO before RX is enabled.
///
/// # Safety
/// Packet DMA must already be initialized and must not have an active consumer.
pub unsafe fn initialize() {
    unsafe {
        crate::dtcm::shared_ptr::<u32>(crate::dtcm::RX_FIFO_STATE.deferred_consumer()).write_volatile(0);
        synchronize_after_wake(0);
        *DIAGNOSTICS.0.get() = ReceiveDiagnostics::default();
    }
}

/// Synchronizes the polling consumer with the live packet-DMA producer like
/// vendor `sync_regs_10` (`0x10024`).
///
/// # Safety
/// No FIFO-backed HIF transfer may remain outstanding across the wake reset.
pub unsafe fn synchronize_after_wake(producer: u32) {
    let producer = producer & FIFO_MASK;
    unsafe {
        rx_ring().synchronize(producer);
        crate::dtcm::shared_ptr::<u32>(crate::dtcm::RX_FIFO_STATE.release_cursor())
            .write_volatile(producer);
        crate::dtcm::shared_ptr::<u32>(crate::dtcm::RX_FIFO_STATE.claim_cursor())
            .write_volatile(producer);
        DMA_CONSUMER.write_volatile(producer);
        let control = (crate::platform::mac_register(0x0600) as *mut u32).read_volatile();
        (crate::platform::mac_register(0x0600) as *mut u32).write_volatile(control);
    }
}

fn receive_indication_id(if_id: u8) -> u16 {
    0x0804 | (u16::from(if_id.min(2)) << 6)
}

fn scan_frame_flags(frame_control: u16) -> Option<u32> {
    match frame_control & 0x00fc {
        0x0080 => Some(1 << 7),
        0x0050 => Some(0),
        _ => None,
    }
}

unsafe fn management_ds_channel(frame: usize, frame_len: usize) -> Option<u8> {
    // Beacon and probe-response bodies both begin with twelve fixed bytes after
    // the 24-byte management header.
    let mut offset = 36;
    while offset + 2 <= frame_len {
        let id = unsafe { ((frame + offset) as *const u8).read_volatile() };
        let length = usize::from(unsafe { ((frame + offset + 1) as *const u8).read_volatile() });
        let next = offset.checked_add(2 + length)?;
        if next > frame_len {
            return None;
        }
        if id == 3 && length >= 1 {
            return Some(unsafe { ((frame + offset + 2) as *const u8).read_volatile() });
        }
        offset = next;
    }
    None
}

fn normalize_offset(mut offset: u32) -> u32 {
    offset &= FIFO_MASK;
    if offset >= FIFO_SIZE {
        offset -= FIFO_SIZE;
    }
    offset
}

fn available_bytes(consumer: u32, producer: u32) -> u32 {
    if producer < consumer {
        producer.wrapping_add(FIFO_SIZE).wrapping_sub(consumer)
    } else {
        producer - consumer
    }
}

fn next_offset(offset: u32, slot_length: u16) -> u32 {
    normalize_offset(
        offset
            .wrapping_add(u32::from(slot_length))
            .wrapping_add(0x2f),
    )
}

fn repairable_tail_slot(
    slot_length: u16,
    available: u32,
    next: u32,
    producer: u32,
    next_has_magic: bool,
) -> bool {
    u32::from(slot_length) <= available && next == producer && !next_has_magic
}

fn slot_data_fits_packet_ram(offset: u32, slot_length: u16) -> bool {
    let frame_end = offset as usize + 0x20 + usize::from(slot_length);
    let trailer = (frame_end + 3) & !3;
    trailer + 8 <= FIFO_STORAGE_SIZE
}

const fn claimed_slot_state(value: u32) -> u32 {
    value & 0xff
}

const fn pending_release_state(value: u32) -> u32 {
    claimed_slot_state(value).wrapping_sub(0x100)
}

unsafe fn set_release_offset(ring: &mut RxRing, next: u32) {
    unsafe {
        ring.set_release_offset(next);
        crate::dtcm::shared_ptr::<u32>(crate::dtcm::RX_FIFO_STATE.release_cursor()).write_volatile(next);
        DMA_CONSUMER.write_volatile(next);
    }
}

unsafe fn set_claim_offset(ring: &mut RxRing, next: u32) {
    unsafe {
        ring.set_claim_offset(next);
        crate::dtcm::shared_ptr::<u32>(crate::dtcm::RX_FIFO_STATE.claim_cursor())
            .write_volatile(next);
    }
}

/// Vendor-style recovery for a corrupt current header: walk four-byte-aligned
/// candidates up to the producer and adopt the next magic slot, or discard the
/// unread region if no valid slot remains.
/// Vendor `rxfifo_next_frame` (`0x8af2`, `annotated-main.c:10331`) scan path.
///
/// Vendor does not simply take the first word matching the magic. Having found
/// a candidate it also requires the slot's declared length to fit the remaining
/// span AND the following slot to carry the magic too, and keeps scanning when
/// either fails. Without that two-slot lookahead any payload word that happens
/// to equal `0x00aa55ff` resynchronises us onto garbage, which desynchronises
/// the consumer further and costs another resync — a plausible amplifier for
/// runs where resyncs climb from 1 to 21.
///
/// The total flush when the span is exhausted IS vendor behaviour
/// (`annotated-main.c:10373-10380` sets release, consumer and the DMA consumer
/// at `0x09c00608` all to the producer), so it is kept as-is.
///
/// `RX_RESYNC` now packs three fields, since resync counts are small:
/// bits 0..9 accepted, bits 10..19 flushed, bits 20..31 candidates rejected by
/// the lookahead.
unsafe fn apply_resynchronized_offset(ring: &mut RxRing, previous_claim: u32, target: u32) {
    unsafe {
        let release = ring.resynchronize(previous_claim, target);
        crate::dtcm::shared_ptr::<u32>(crate::dtcm::RX_FIFO_STATE.claim_cursor())
            .write_volatile(target);
        if let Some(target) = release {
            set_release_offset(ring, target);
        }
    }
}

unsafe fn resynchronize_consumer(
    ring: &mut RxRing,
    consumer: u32,
    producer: u32,
    raw_producer: u32,
) {
    unsafe {
        let mut remaining = available_bytes(consumer, producer);
        let mut candidate = consumer;
        loop {
            // Vendor's inner loop: step one word at a time to the next slot
            // that carries the magic, flushing if the span runs out.
            loop {
                if remaining < 4 || candidate == producer {
                    resync_report(ring, raw_producer, consumer, producer, RESYNC_FLUSHED);
                    apply_resynchronized_offset(ring, consumer, producer);
                    return;
                }
                remaining -= 4;
                candidate = normalize_offset(candidate.wrapping_add(4));
                if fifo_word(candidate) == FIFO_MAGIC {
                    break;
                }
            }
            let slot_length =
                ((fifo_base() + candidate as usize + 0x18) as *const u16).read_volatile();
            let next = next_offset(candidate, slot_length);
            if u32::from(slot_length) <= remaining
                && slot_length >= 4
                && fifo_word(next) == FIFO_MAGIC
            {
                resync_report(ring, raw_producer, consumer, producer, RESYNC_ACCEPTED);
                apply_resynchronized_offset(ring, consumer, candidate);
                return;
            }
            // Candidate carried the magic but failed vendor's validation: a
            // false positive we would previously have resynchronised onto.
            bump_resync_field(RESYNC_REJECTED);
        }
    }
}

const RESYNC_ACCEPTED: u32 = 0;
const RESYNC_FLUSHED: u32 = 10;
const RESYNC_REJECTED: u32 = 20;

/// Bump one packed field of the `RX_RESYNC` counter, saturating so a busy field
/// cannot carry into its neighbour.
unsafe fn bump_resync_field(shift: u32) {
    unsafe {
        let width = if shift == RESYNC_REJECTED { 12 } else { 10 };
        let mask = ((1_u32 << width) - 1) << shift;
        let current =
            crate::host_tx_diagnostics::read(crate::host_tx_diagnostics::counter::RX_RESYNC);
        let field = (current & mask) >> shift;
        if field + 1 < (1 << width) {
            crate::host_tx_diagnostics::observe(
                crate::host_tx_diagnostics::counter::RX_RESYNC,
                (current & !mask) | ((field + 1) << shift),
            );
        }
    }
}

unsafe fn resync_report(
    ring: &RxRing,
    raw_producer: u32,
    consumer: u32,
    producer: u32,
    shift: u32,
) {
    unsafe {
        if ring.host_transfer_count() != 0 {
            {
                let _ = (raw_producer, consumer, producer);
            }
        }
        bump_resync_field(shift);
    }
}

unsafe fn matching_tx_command(slot: usize) -> (usize, usize, u32) {
    let slot_words = unsafe {
        [
            (slot as *const u32).read_volatile(),
            ((slot + 4) as *const u32).read_volatile(),
            ((slot + 8) as *const u32).read_volatile(),
            ((slot + 0x18) as *const u32).read_volatile(),
        ]
    };
    let mut best = (0, 0, 0);
    let mut best_score = 0_u8;
    for pipe in 0..4 {
        for tx_slot in 0..4 {
            let command = packet_ram::tx_command(pipe, tx_slot);
            let mut score = 0_u8;
            let mut first_match = 0_usize;
            for offset in (0x0c..=0x40).step_by(4) {
                let word = unsafe { ((command + offset) as *const u32).read_volatile() };
                if slot_words
                    .iter()
                    .copied()
                    .any(|slot_word| slot_word != 0 && slot_word != FIFO_MAGIC && slot_word == word)
                {
                    score = score.saturating_add(1);
                    if first_match == 0 {
                        first_match = offset;
                    }
                }
            }
            if score > best_score {
                best_score = score;
                best = (
                    command,
                    command + first_match,
                    u32::try_from(pipe).unwrap_or(0)
                        | (u32::try_from(tx_slot).unwrap_or(0) << 8)
                        | (u32::try_from(first_match).unwrap_or(0) << 16)
                        | (u32::from(score) << 24),
                );
            }
        }
    }
    best
}

#[cfg(feature = "vendor-host-tx-diagnostics")]
fn encode_snapshot_name(words: [u32; 6]) -> [u8; 48] {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = [0_u8; 48];
    for (word_index, word) in words.into_iter().enumerate() {
        for nibble in 0..8 {
            let shift = 28 - nibble * 4;
            output[word_index * 8 + nibble] = HEX[((word >> shift) & 0x0f) as usize];
        }
    }
    output
}

#[cfg(feature = "vendor-host-tx-diagnostics")]
pub unsafe fn record_tx_command_signature(class: u8, command: u32) {
    let (valid_bit, pointer_index) = match class {
        0 => (2_u32, 2_usize),
        6 => (1_u32, 1_usize),
        _ => return,
    };
    let signatures = unsafe { &mut *TX_COMMAND_SIGNATURES.0.get() };
    signatures[0] = (signatures[0] & 3) | valid_bit | (u32::from(class) << 8);
    signatures[pointer_index] = command;
}

#[cfg(feature = "vendor-host-tx-diagnostics")]
unsafe fn capture_final_tx_command_signature(command: u32) {
    let signatures = unsafe { &mut *TX_COMMAND_SIGNATURES.0.get() };
    let class = (signatures[0] >> 8) as u8;
    let words_start = match class {
        0 if signatures[2] == command => 30,
        6 if signatures[1] == command => 9,
        _ => return,
    };
    for word in 0..21 {
        signatures[words_start + word] =
            unsafe { ((command as usize + word * 4) as *const u32).read_volatile() };
    }
    if class == 0 {
        let history = unsafe { &mut *CLASS0_COMMAND_HISTORY.0.get() };
        let index = history[0] as usize & 7;
        let generation = history[1].wrapping_add(1);
        let entry = 2 + index * 23;
        history[0] = ((index + 1) & 7) as u32;
        history[1] = generation;
        history[entry] = generation;
        history[entry + 1] = command;
        history[entry + 2..entry + 23].copy_from_slice(&signatures[30..51]);
    }
}

// Used by the vendor-shaped resync scan in every build, not just diagnostics.
unsafe fn fifo_word(offset: u32) -> u32 {
    unsafe { ((fifo_base() + normalize_offset(offset) as usize) as *const u32).read_volatile() }
}

#[cfg(feature = "vendor-host-tx-diagnostics")]
unsafe fn longest_signature_match_in_producer_delta(
    start: u32,
    end: u32,
    signature: &[u32; 21],
) -> Option<(usize, usize, usize)> {
    let mut best = None;
    let mut candidate = normalize_offset(start);
    let mut remaining = available_bytes(candidate, normalize_offset(end));
    while remaining >= 12 {
        for command_word in 0..21 {
            let mut matched = 0;
            while command_word + matched < signature.len()
                && (matched + 1) * 4 <= remaining as usize
                && unsafe { fifo_word(candidate.wrapping_add((matched * 4) as u32)) }
                    == signature[command_word + matched]
            {
                matched += 1;
            }
            if matched >= 3 && best.is_none_or(|(_, _, best_words)| matched > best_words) {
                best = Some((fifo_base() + candidate as usize, command_word * 4, matched));
            }
        }
        candidate = normalize_offset(candidate.wrapping_add(4));
        remaining -= 4;
    }
    best
}

#[cfg(feature = "vendor-host-tx-diagnostics")]
unsafe fn best_class0_history_match(
    start: u32,
    end: u32,
) -> Option<(usize, usize, usize, u32, u32)> {
    let history = unsafe { &*CLASS0_COMMAND_HISTORY.0.get() };
    let latest_generation = history[1];
    let mut best = None;
    for index in 0..8 {
        let entry = 2 + index * 23;
        let generation = history[entry];
        if generation == 0 {
            continue;
        }
        let mut signature = [0_u32; 21];
        signature.copy_from_slice(&history[entry + 2..entry + 23]);
        if let Some((address, command_offset, matched_words)) =
            unsafe { longest_signature_match_in_producer_delta(start, end, &signature) }
        {
            let replace = best.is_none_or(|(_, _, best_words, _, best_generation)| {
                matched_words > best_words
                    || (matched_words == best_words
                        && latest_generation.wrapping_sub(generation)
                            < latest_generation.wrapping_sub(best_generation))
            });
            if replace {
                best = Some((
                    address,
                    command_offset,
                    matched_words,
                    history[entry + 1],
                    generation,
                ));
            }
        }
    }
    best
}

#[cfg(feature = "vendor-host-tx-diagnostics")]
unsafe fn locate_live_command_word(value: u32) -> (u32, u32, u32) {
    let mut first = 0_u32;
    let mut first_pointer = 0_u32;
    let mut count = 0_u32;
    for pipe in 0..4_u32 {
        for slot in 0..4_u32 {
            let command = packet_ram::tx_command(pipe as usize, slot as usize) as u32;
            for word in 0..21_u32 {
                if unsafe { ((command + word * 4) as *const u32).read_volatile() } == value {
                    if count == 0 {
                        first = (pipe << 28) | (slot << 24) | (word * 4);
                        first_pointer = command;
                    }
                    count = count.wrapping_add(1);
                }
            }
        }
    }
    (first, first_pointer, count)
}

#[cfg(feature = "vendor-host-tx-diagnostics")]
unsafe fn locate_history_command_word(value: u32) -> (u32, u32, u32) {
    let history = unsafe { &*CLASS0_COMMAND_HISTORY.0.get() };
    let latest = history[1];
    let mut best = None;
    let mut count = 0_u32;
    for index in 0..8 {
        let entry = 2 + index * 23;
        let generation = history[entry];
        if generation == 0 {
            continue;
        }
        for word in 0..21 {
            if history[entry + 2 + word] == value {
                count = count.wrapping_add(1);
                if best.is_none_or(|(best_generation, _, _)| {
                    latest.wrapping_sub(generation) < latest.wrapping_sub(best_generation)
                }) {
                    best = Some((generation, (word * 4) as u32, history[entry + 1]));
                }
            }
        }
    }
    best.map_or((0, 0, count), |(generation, offset, pointer)| {
        (generation, (offset << 16) | (count & 0xffff), pointer)
    })
}

#[cfg(feature = "vendor-host-tx-diagnostics")]
unsafe fn locate_class6_command_word(value: u32) -> u32 {
    let signatures = unsafe { &*TX_COMMAND_SIGNATURES.0.get() };
    (0..21)
        .find(|word| signatures[9 + word] == value)
        .map_or(u32::MAX, |word| (word * 4) as u32)
}

#[cfg(feature = "vendor-host-tx-diagnostics")]
unsafe fn describe_matching_slot(start: u32, end: u32, address: usize) -> [u32; 6] {
    let target = normalize_offset((address - fifo_base()) as u32);
    let end = normalize_offset(end);
    let mut cursor = normalize_offset(start);
    for _ in 0..16 {
        if cursor == end {
            break;
        }
        let slot = fifo_base() + cursor as usize;
        if unsafe { (slot as *const u32).read_volatile() } != FIFO_MAGIC {
            break;
        }
        let slot_length = unsafe { ((slot + 0x18) as *const u16).read_volatile() };
        let next = next_offset(cursor, slot_length);
        let stride = available_bytes(cursor, next);
        if slot_length < 4 || stride == 0 || stride > available_bytes(cursor, end) {
            break;
        }
        let relative = available_bytes(cursor, target);
        if relative < stride {
            let frame = slot + 0x20;
            return unsafe {
                [
                    slot as u32,
                    (u32::from(slot_length) << 16) | relative,
                    (frame as *const u32).read_volatile(),
                    ((frame + 4) as *const u32).read_volatile(),
                    ((frame + 8) as *const u32).read_volatile(),
                    ((frame + 12) as *const u32).read_volatile(),
                ]
            };
        }
        cursor = next;
    }
    [0; 6]
}

#[cfg(feature = "vendor-host-tx-diagnostics")]
pub unsafe fn fatal_command_snapshot() -> [u32; 11] {
    unsafe {
        let watch = &*RX_TX_PRODUCER_WATCH.0.get();
        let start = watch[1];
        let end = DMA_PRODUCER.read_volatile();
        let mut result = [0_u32; 11];
        result[0] = start;
        result[1] = end;
        result[8] = DMA_CONSUMER.read_volatile();
        let ring = rx_ring();
        result[9] = ring.claim_offset();
        result[10] = ring.release_offset();
        if watch[0] != 0
            && normalize_offset(start) != normalize_offset(end)
            && let Some((address, command_offset, matched_words, pointer, generation)) =
                best_class0_history_match(start, end)
        {
            result[2] = address as u32;
            result[3] = pointer;
            result[4] = command_offset as u32;
            result[5] = (matched_words * 4) as u32;
            result[6] = generation;
            result[7] = fifo_word(start);
        }
        result
    }
}

#[cfg(feature = "vendor-host-tx-diagnostics")]
pub fn validate_tx_boundary(phase: u32, _pipe: u8, _tx_slot: u8, command: u32, ring: u32) {
    // Report-only mode. This validator halts the firmware the instant it sees a
    // TX command signature inside the RX producer delta, so every run so far has
    // stopped itself at the first corruption and we have never observed whether
    // the system continues to work through it. Skip the check entirely here; the
    // RX consumer resynchronisation counter still records that corruption
    // happened.
    {
        let _ = (phase, command, ring);
        return;
    }
}

unsafe fn publish_owned_resynchronization(consumer: u32, producer: u32, raw_producer: u32) -> ! {
    unsafe {
        let current = fifo_base() + consumer as usize;
        let (command, match_address, match_state) = matching_tx_command(current);
        let (match_words, ring_state) = if command == 0 {
            ([0; 4], 0)
        } else {
            let pipe = usize::try_from(match_state & 3).unwrap_or(0);
            let hardware_ring = crate::platform::tx_ring_register(pipe, 0);
            (
                [
                    (match_address as *const u32).read_volatile(),
                    ((match_address + 4) as *const u32).read_volatile(),
                    ((match_address + 8) as *const u32).read_volatile(),
                    ((match_address + 0x0c) as *const u32).read_volatile(),
                ],
                ((hardware_ring + 0x20) as *const u32).read_volatile(),
            )
        };
        let ring = rx_ring();
        crate::hif::publish_halting_exception(
            [
                0x5258_5253,
                raw_producer,
                producer,
                ring.claim_offset(),
                ring.release_offset(),
                DMA_CONSUMER.read_volatile(),
                current as u32,
                (current as *const u32).read_volatile(),
                ((current + 4) as *const u32).read_volatile(),
                ((current + 8) as *const u32).read_volatile(),
                ((current + 0x18) as *const u32).read_volatile(),
                command as u32,
                match_words[0],
                match_words[1],
                match_words[2],
                match_words[3],
                ring_state,
                match_state,
            ],
            b"xr819-rx-resync-owned",
        );
        crate::halt_always!();
    }
}

unsafe fn release_head_slot(ring: &mut RxRing, slot: usize, next: u32, low_state: u32) {
    unsafe {
        // Preserve vendor `rxfifo_release_slot()` ordering: advance the
        // software release cursor, mark and clear the slot, then expose the
        // new consumer pointer to packet DMA.
        ring.set_release_offset(next);
        crate::dtcm::shared_ptr::<u32>(crate::dtcm::RX_FIFO_STATE.release_cursor()).write_volatile(next);
        ((slot + 8) as *mut u32).write_volatile(FIFO_RELEASED | low_state);
        (slot as *mut u32).write_volatile(0);
        DMA_CONSUMER.write_volatile(next);
        crate::host_tx_diagnostics::record(
            crate::host_tx_diagnostics::EVENT_RX_RELEASE,
            0,
            slot as u32,
            next,
            low_state,
        );
        let diagnostics = &mut *DIAGNOSTICS.0.get();
        diagnostics.released_slots = diagnostics.released_slots.wrapping_add(1);
    }
}

unsafe fn release(ring: &mut RxRing, token: RxToken) {
    unsafe {
        let slot_offset = token.slot_offset();
        let slot = fifo_base() + slot_offset as usize;
        if slot_offset >= FIFO_SIZE {
            crate::hif::publish_halting_exception(
                [
                    0x5258_524c,
                    slot as u32,
                    token.next(),
                    ring.release_offset(),
                    ring.claim_offset(),
                    DMA_PRODUCER.read_volatile(),
                    ring.host_transfer_count(),
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    fifo_base() as u32,
                    FIFO_SIZE,
                ],
                b"xr819-rx-release-address",
            );
            // Do NOT convert this into a return: skipping the release stalls
            // the release head permanently, the FIFO never drains, and every
            // run dies harder than the hang it was meant to avoid (measured:
            // 3 of 4 runs failed to associate at all).
            crate::halt_always!();
        }
        let state = (slot + 8) as *mut u32;
        let value = state.read_volatile();
        let low = match ring.classify_release(&token, value) {
            ReleaseAction::AlreadyReleased | ReleaseAction::Pending => return,
            ReleaseAction::MarkPending { corrupt, .. } => {
                if corrupt {
                    crate::host_tx_diagnostics::bump(
                        crate::host_tx_diagnostics::counter::RX_RESYNC,
                    );
                }
                state.write_volatile(pending_release_state(value));
                return;
            }
            ReleaseAction::ReleaseHead {
                low_state,
                normalize_first,
                corrupt,
            } => {
                if corrupt {
                    crate::host_tx_diagnostics::bump(
                        crate::host_tx_diagnostics::counter::RX_RESYNC,
                    );
                }
                if normalize_first {
                    state.write_volatile(pending_release_state(value));
                }
                low_state
            }
        };

        release_head_slot(ring, slot, token.next(), low);
        while ring.release_offset() != ring.claim_offset() {
            let next_slot = fifo_base() + ring.release_offset() as usize;
            let next_state = ((next_slot + 8) as *const u32).read_volatile();
            if next_state & 0xffff_ff00 != 0xffff_ff00 {
                break;
            }
            let next = ((next_slot + 4) as *const u32).read_volatile();
            release_head_slot(ring, next_slot, normalize_offset(next), next_state & 0xff);
        }
        if let Some(target) = ring.finish_deferred_resync() {
            // `finish_deferred_resync` already updates the software cursor;
            // preserve the existing hardware write order here.
            crate::dtcm::shared_ptr::<u32>(crate::dtcm::RX_FIFO_STATE.release_cursor())
                .write_volatile(target);
            DMA_CONSUMER.write_volatile(target);
        }
    }
}

/// Releases a zero-copy FIFO indication after HIF confirms descriptor ownership
/// has returned to firmware.
///
/// # Safety
/// `token` must be the currently outstanding radio transfer token.
/// RX counters for the lifecycle report: frames seen, frames dropped at the
/// host-transfer limit, indications published, and slots released.
///
/// The radio diagnostics MIB and the class-0 lifecycle MIB share one report
/// array, so without this the RX and TX sides can never be read in the same
/// run. Ping shows 2-5% loss with duplicates, which means retransmission, so
/// whether we are discarding received frames matters.
///
/// # Safety
/// Single-threaded firmware context.
pub unsafe fn rx_diagnostic_counters() -> (u32, u32, u32, u32) {
    unsafe {
        let diagnostics = &*DIAGNOSTICS.0.get();
        (
            diagnostics.valid_slots,
            diagnostics.filtered_frames,
            diagnostics.indications,
            diagnostics.released_slots,
        )
    }
}

/// Receive indications currently owned by the host, against `MAX_HOST_TRANSFERS`.
/// At the limit every further frame is dropped.
///
/// # Safety
/// Single-threaded firmware context.
pub unsafe fn host_transfers_outstanding() -> u32 {
    unsafe { rx_ring().host_transfer_count() }
}

pub unsafe fn complete_host_transfer(token: RxToken) {
    unsafe {
        let ring = rx_ring();
        if ring.complete_host_transfer() {
            release(ring, token);
        }
    }
}

/// Continuously consumes receive FIFO entries when no host scan owns them.
/// Vendor `rx_handler_main_loop` runs independently of scan state; without
/// this path, old beacons accumulate and contaminate the next scan dwell.
pub unsafe fn discard_one_idle(channel: u16) {
    if let Some(indication) = unsafe { poll_scan_indication(0, channel) } {
        unsafe { complete_host_transfer(indication.release) };
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
pub unsafe fn poll_scan_indication(if_id: u8, active_channel: u16) -> Option<PendingIndication> {
    unsafe { poll_indication(if_id, active_channel, true) }
}

pub unsafe fn poll_joined_indication(if_id: u8, active_channel: u16) -> Option<PendingIndication> {
    unsafe { poll_indication(if_id, active_channel, false) }
}

/// Exact cursor normalization shared by vendor `rxfifo_off_to_addr()` and
/// `rxfifo_wrap_sub()` for the 0x7000-byte logical packet-RAM ring.
pub const fn vendor_rx_offset(mut offset: u32) -> u32 {
    offset &= FIFO_MASK;
    if offset >= FIFO_SIZE {
        offset -= FIFO_SIZE;
    }
    offset
}

/// Exact vendor `rxfifo_advance(offset, slot_length)` arithmetic.
pub const fn vendor_rx_advance(offset: u32, slot_length: u16) -> u32 {
    vendor_rx_offset(
        offset
            .wrapping_add(slot_length as u32)
            .wrapping_add(0x2f),
    )
}

unsafe fn poll_indication(
    if_id: u8,
    active_channel: u16,
    scan_only: bool,
) -> Option<PendingIndication> {
    unsafe {
        let ring = rx_ring();
        let consumer = ring.claim_offset();
        let raw_producer = DMA_PRODUCER.read_volatile();
        let producer = normalize_offset(raw_producer);
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

        let slot = fifo_base() + consumer as usize;
        let slot_length = unsafe { ((slot + 0x18) as *const u16).read_volatile() };
        if unsafe { (slot as *const u32).read_volatile() } != FIFO_MAGIC {
            let available = available_bytes(consumer, producer);
            let next = next_offset(consumer, slot_length);
            let next_has_magic = fifo_word(next) == FIFO_MAGIC;
            if repairable_tail_slot(slot_length, available, next, producer, next_has_magic) {
                unsafe {
                    // Vendor `rxfifo_next_frame` repair branch: the current slot's
                    // header marker was damaged, but its bounded length lands
                    // exactly on the producer. Preserve that frame, rebuild the
                    // producer sentinel, and advance ownership normally instead of
                    // destructively scanning or flushing the unread span.
                    ((slot + 4) as *mut u32).write_volatile(next);
                    let state = (slot + 8) as *mut u32;
                    state.write_volatile(claimed_slot_state(state.read_volatile()));
                    set_claim_offset(ring, next);
                    ((fifo_base() + next as usize) as *mut u32).write_volatile(FIFO_MAGIC);
                    bump_resync_field(RESYNC_ACCEPTED);
                }
            } else {
                unsafe {
                    let diagnostics = &mut *DIAGNOSTICS.0.get();
                    diagnostics.bad_magic = diagnostics.bad_magic.wrapping_add(1);
                    resynchronize_consumer(ring, consumer, producer, raw_producer);
                }
                return None;
            }
        } else {
            unsafe {
                let diagnostics = &mut *DIAGNOSTICS.0.get();
                diagnostics.valid_slots = diagnostics.valid_slots.wrapping_add(1);
            }
        }

        let available = available_bytes(consumer, producer);
        if slot_length < 4
            || usize::from(slot_length) > MAX_FRAME_LEN + 4
            || available < u32::from(slot_length)
        {
            unsafe {
                let diagnostics = &mut *DIAGNOSTICS.0.get();
                diagnostics.malformed_slots = diagnostics.malformed_slots.wrapping_add(1);
                if usize::from(slot_length) > MAX_FRAME_LEN + 4 {
                    diagnostics.oversized_frames = diagnostics.oversized_frames.wrapping_add(1);
                }
                resynchronize_consumer(ring, consumer, producer, raw_producer);
            }
            return None;
        }

        let frame_len = usize::from(slot_length) - 4;
        let next = next_offset(consumer, slot_length);
        let token = unsafe {
            ((slot + 4) as *mut u32).write_volatile(next);
            let state = (slot + 8) as *mut u32;
            let slot_state = claimed_slot_state(state.read_volatile());
            state.write_volatile(slot_state);
            let token = ring.claim(next);
            crate::dtcm::shared_ptr::<u32>(crate::dtcm::RX_FIFO_STATE.claim_cursor())
                .write_volatile(next);
            crate::host_tx_diagnostics::record(
                crate::host_tx_diagnostics::EVENT_RX_CLAIM,
                0,
                slot as u32,
                (u32::from(slot_length) << 16) | next,
                slot_state,
            );
            token
        };

        let frame_address = slot + 0x20;
        let trailer = (frame_address + usize::from(slot_length) + 3) & !3;
        if !slot_data_fits_packet_ram(consumer, slot_length) {
            unsafe {
                let diagnostics = &mut *DIAGNOSTICS.0.get();
                diagnostics.malformed_slots = diagnostics.malformed_slots.wrapping_add(1);
                release(ring, token);
            }
            return None;
        }

        // Vendor `rx_handler_main_loop` reads the complete halfword at trailer+2
        // and masks it to ten bits for the low PHY classes.
        let channel = unsafe { ((trailer + 2) as *const u16).read_volatile() } & 0x03ff;
        let rcpi = unsafe { ((trailer + 7) as *const u8).read_volatile() }.max(1);
        // Vendor copies two adjacent descriptor bytes into the indication's rate
        // and RCPI fields (`annotated-main.c:13085-13125`: `param_1+0xe` -> +10,
        // `param_1+0xf` -> +11). Our RCPI is `trailer+7`, matching vendor's +0xf,
        // so the rate byte is `trailer+6`. We previously hard-coded 0 here, and
        // `txrx.c:1241-1247` maps that straight to rate_idx 0, so every received
        // frame was reported to mac80211 as 1 Mbit/s regardless of its real rate.
        // Clamp before reporting. `txrx.c:1241-1247` treats >=14 as HT with
        // `rate_idx = rx_rate - 14` and mac80211 then *drops the frame* with a
        // WARN when that exceeds MCS 76, so one stray descriptor byte costs a
        // received frame. This chip is 1x1 802.11n, so the legal space is 0..=21:
        // 0..=3 legacy direct, 4..=13 legacy `rate_idx - 2`, 14..=21 HT MCS0..7.
        // Anything else is not a rate and is reported as unknown rather than
        // silently discarding the frame. Measured cost of passing it through raw:
        // a continuous WARN storm from ieee80211_rx_list and association failing
        // at AUTHENTICATING because the unicast auth response never survived RX.
        let rx_rate = match unsafe { ((trailer + 6) as *const u8).read_volatile() } {
            rate @ 0..=21 => rate,
            _ => 0,
        };
        let frame_control = if frame_len >= 2 {
            unsafe { (frame_address as *const u16).read_volatile() }
        } else {
            0xffff
        };
        let mut indication_flags = scan_frame_flags(frame_control).unwrap_or(0);
        unsafe {
            let diagnostics = &mut *DIAGNOSTICS.0.get();
            diagnostics.last_slot_length = slot_length;
            diagnostics.last_frame_control = frame_control;
            diagnostics.last_channel = channel;
            diagnostics.last_active_channel = active_channel;
            diagnostics.last_trailer_word = (trailer as *const u32).read_volatile();
        }

        #[cfg(all(target_arch = "arm", feature = "experimental-depth-two-ampdu"))]
        if !scan_only
            && unsafe { crate::tx::consume_depth_two_block_ack(frame_address, frame_len) }
        {
            unsafe { release(ring, token) };
            return None;
        }

        // The vendor drains old frames before retuning. Reject a management frame
        // whose on-air DS element proves it belongs to a previous channel rather
        // than relabeling it with the active CW1200 dwell.
        let wrong_ds_channel = scan_only
            && unsafe { management_ds_channel(frame_address, frame_len) }
                .is_some_and(|channel| u16::from(channel) != active_channel);
        if frame_len < 24
            || (scan_only && scan_frame_flags(frame_control).is_none())
            || wrong_ds_channel
        {
            unsafe {
                let diagnostics = &mut *DIAGNOSTICS.0.get();
                diagnostics.filtered_frames = diagnostics.filtered_frames.wrapping_add(1);
                release(ring, token);
            }
            return None;
        }

        if !scan_only {
            let frame =
                unsafe { core::slice::from_raw_parts_mut(frame_address as *mut u8, frame_len) };
            match crate::crypto::decrypt_rx_frame(frame, if_id) {
                Ok(true) => indication_flags |= 3,
                Ok(false) => {}
                Err(_) => {
                    unsafe {
                        let diagnostics = &mut *DIAGNOSTICS.0.get();
                        diagnostics.filtered_frames = diagnostics.filtered_frames.wrapping_add(1);
                        release(ring, token);
                    }
                    return None;
                }
            }
        }

        // Vendor keeps draining and filtering RX FIFO slots while 24 receive
        // indications are host-owned; only an otherwise publishable frame is
        // dropped at this admission boundary.
        if ring.host_transfer_count() >= MAX_HOST_TRANSFERS {
            unsafe {
                let diagnostics = &mut *DIAGNOSTICS.0.get();
                diagnostics.filtered_frames = diagnostics.filtered_frames.wrapping_add(1);
                release(ring, token);
            }
            return None;
        }

        let message_address = frame_address - WSM_RX_HEADROOM;
        let message_length = frame_len + WSM_RX_HEADROOM;
        unsafe {
            write_u16(message_address, message_length as u16);
            write_u16(message_address + 2, receive_indication_id(if_id));
            write_u32(message_address + 4, 0);
            // XR819 trailer channel metadata includes PHY status bits (for example
            // channel 11 appears as 0x010b). CW1200 WSM requires the plain channel
            // number, which is authoritative from the active scan request.
            write_u16(message_address + 8, active_channel);
            ((message_address + 10) as *mut u8).write_volatile(rx_rate);
            ((message_address + 11) as *mut u8).write_volatile(rcpi);
            write_u32(message_address + 12, indication_flags);
            let published = ring.publish_host_transfer(MAX_HOST_TRANSFERS);
            debug_assert!(published);
            let diagnostics = &mut *DIAGNOSTICS.0.get();
            diagnostics.indications = diagnostics.indications.wrapping_add(1);
        }

        Some(PendingIndication {
            address: message_address as u32,
            length: message_length as u16,
            release: token,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vendor_fifo_advance_alignment_and_wrap() {
        assert_eq!(next_offset(0, 100), 0x90);
        assert_eq!(next_offset(0x6fc0, 64), 0x2c);
        assert_eq!(vendor_rx_advance(0, 100), 0x90);
        assert_eq!(vendor_rx_advance(0x6fc0, 64), 0x2c);
        assert_eq!(vendor_rx_advance(0x100, 1), 0x130);
        assert_eq!(normalize_offset(0x7000), 0);
        assert_eq!(vendor_rx_offset(0x7000), 0);
        assert_eq!(available_bytes(0x6ff0, 0x20), 0x30);
        assert!(slot_data_fits_packet_ram(0x6f00, 100));
        assert!(slot_data_fits_packet_ram(0x6fc0, 64));
        assert!(slot_data_fits_packet_ram(0x6a00, 1552));
        assert!(!slot_data_fits_packet_ram(0x6fc0, 0x1100));
    }

    #[test]
    fn vendor_tail_repair_requires_exact_producer_boundary() {
        assert!(repairable_tail_slot(100, 100, 0x240, 0x240, false));
        assert!(!repairable_tail_slot(101, 100, 0x240, 0x240, false));
        assert!(!repairable_tail_slot(100, 100, 0x23c, 0x240, false));
        assert!(!repairable_tail_slot(100, 100, 0x240, 0x240, true));
    }

    #[test]
    fn scan_filter_accepts_only_beacons_and_probe_responses() {
        assert_eq!(scan_frame_flags(0x0080), Some(1 << 7));
        assert_eq!(scan_frame_flags(0x0050), Some(0));
        assert_eq!(scan_frame_flags(0x0008), None);
        assert_eq!(scan_frame_flags(0x00d0), None);
    }

    #[test]
    fn receive_indication_preserves_xr819_interface_bits() {
        assert_eq!(receive_indication_id(0), 0x0804);
        assert_eq!(receive_indication_id(1), 0x0844);
        assert_eq!(receive_indication_id(2), 0x0884);
    }

    #[test]
    fn zero_copy_indication_fits_hif_descriptor_length() {
        assert_eq!(MAX_FRAME_LEN + WSM_RX_HEADROOM, 0x1ffe);
        assert!(MAX_FRAME_LEN > 1840);
    }

    #[test]
    fn vendor_slot_ownership_preserves_low_metadata_byte() {
        assert_eq!(claimed_slot_state(0x1234_56a5), 0x0000_00a5);
        assert_eq!(pending_release_state(0x1234_56a5), 0xffff_ffa5);
        assert_eq!(FIFO_RELEASED | claimed_slot_state(0x1234_56a5), 0xcccc_cca5);
    }
}
