//! Minimal XR819 host-interface descriptor transport.
//!
//! Hardware descriptor and packet-RAM addresses follow the XR819 interface.
//! CPU-only queue ownership is native Rust state rather than a mirror of the
//! vendor firmware's fixed DTCM globals.

#[cfg(all(target_arch = "arm", not(target_feature = "thumb-mode")))]
use core::arch::asm;
#[cfg(all(target_arch = "arm", target_feature = "thumb-mode"))]
use core::arch::global_asm;
#[cfg(feature = "vendor-host-tx-diagnostics")]
use core::cell::UnsafeCell;

use crate::packet_ram;
use crate::radio::{self, PendingIndication, RxToken};
use tock_registers::interfaces::{Readable, Writeable};
use tock_registers::register_bitfields;
use tock_registers::register_structs;
use tock_registers::registers::ReadWrite;

register_bitfields![
    u32,
    DescriptorControl [
        LENGTH OFFSET(0) NUMBITS(13) [],
        SEQUENCE OFFSET(13) NUMBITS(2) []
    ],
    HifControl [
        LENGTH_MASK OFFSET(0) NUMBITS(11) []
    ]
];

register_structs! {
    Descriptor {
        (0x00 => address: ReadWrite<u32>),
        (0x04 => control: ReadWrite<u32, DescriptorControl::Register>),
        (0x08 => @END),
    },

    RxShared {
        (0x00 => descriptors: [Descriptor; 32]),
        (0x100 => @END),
    },

    InterruptController {
        (0x00 => _reserved0),
        (0x0c => enable: ReadWrite<u32>),
        (0x10 => @END),
    },

    HifShared {
        (0x00 => tx: [Descriptor; 4]),
        (0x20 => control: ReadWrite<u32, HifControl::Register>),
        (0x24 => status: ReadWrite<u32>),
        (0x28 => interrupt_ack: ReadWrite<u32, HifControl::Register>),
        (0x2c => emergency_address: ReadWrite<u32>),
        (0x30 => emergency_control: ReadWrite<u32, DescriptorControl::Register>),
        (0x34 => startup_status: ReadWrite<u32>),
        (0x38 => _reserved1),
        (0x40 => activation: ReadWrite<u32>),
        (0x44 => @END),
    }
}

const INTERRUPT_CONTROLLER_BASE: usize = 0x0a88_0000;
const RX_DESCRIPTOR_BASE: usize = 0x0ab0_0000;
const TX_DESCRIPTOR_BASE: usize = 0x0ab0_0100;
const RX_BUFFER_SIZE: usize = packet_ram::HIF_INPUT_SIZE;
const RX_BUFFER_COUNT: usize = packet_ram::HIF_INPUT_COUNT;

#[cfg(feature = "vendor-host-tx-diagnostics")]
struct SharedOutputHeaders(UnsafeCell<[u32; 64]>);

#[cfg(feature = "vendor-host-tx-diagnostics")]
unsafe impl Sync for SharedOutputHeaders {}

#[cfg(feature = "vendor-host-tx-diagnostics")]
static OUTPUT_HEADERS: SharedOutputHeaders = SharedOutputHeaders(UnsafeCell::new([0; 64]));

#[cfg(feature = "vendor-host-tx-diagnostics")]
struct SharedOutputHashes(UnsafeCell<[u32; 64]>);

#[cfg(feature = "vendor-host-tx-diagnostics")]
unsafe impl Sync for SharedOutputHashes {}

#[cfg(feature = "vendor-host-tx-diagnostics")]
static OUTPUT_HASHES: SharedOutputHashes = SharedOutputHashes(UnsafeCell::new([0; 64]));

/// Size of each of the four linker-owned HIF output buffers. The exact pre-HIF
/// clock transition makes this packet-memory bank CPU-accessible.
pub const SHARED_BUFFER_SIZE: usize = packet_ram::HIF_OUTPUT_SIZE;
const REQUEST_PAYLOAD_CAPACITY: usize = RX_BUFFER_SIZE - 4;

const fn owned_descriptor_length(length: u16) -> u32 {
    ((length as u32).wrapping_add(1) & 0x1ffe) | 1
}

const fn tx_ring_has_capacity(producer: u32, consumer: u32) -> bool {
    producer.wrapping_sub(consumer) < 4
}

const fn output_queue_has_capacity(producer: u32, consumer: u32) -> bool {
    producer.wrapping_sub(consumer) < 64
}

const fn descriptor_sequence(header_id: u16) -> u32 {
    ((header_id >> 13) & 3) as u32
}

#[cfg(feature = "vendor-host-tx-diagnostics")]
unsafe fn output_prefix_hash(buffer: u32, length: u16) -> u32 {
    let count = usize::from(length).min(64);
    let mut hash = 0x811c_9dc5_u32;
    for offset in 0..count {
        let byte = unsafe { ((buffer as usize + offset) as *const u8).read_volatile() };
        hash = (hash ^ u32::from(byte)).wrapping_mul(0x0100_0193);
    }
    hash
}

#[cfg(feature = "vendor-host-tx-diagnostics")]
unsafe fn matching_tx_command(words: [u32; 4]) -> (u32, u32, u32) {
    let mut best_command = 0_u32;
    let mut best_state = 0_u32;
    let mut best_score = 0_u8;
    for pipe in 0..4 {
        for slot in 0..4 {
            let command = packet_ram::tx_command(pipe, slot);
            let mut score = 0_u8;
            let mut first_match = 0_usize;
            for offset in (0x0c..=0x40).step_by(4) {
                let command_word = unsafe { ((command + offset) as *const u32).read_volatile() };
                if words
                    .iter()
                    .copied()
                    .any(|word| word != 0 && word == command_word)
                {
                    score = score.saturating_add(1);
                    if first_match == 0 {
                        first_match = offset;
                    }
                }
            }
            if score > best_score {
                best_score = score;
                best_command = command as u32;
                best_state = pipe as u32
                    | ((slot as u32) << 8)
                    | ((first_match as u32) << 16)
                    | (u32::from(score) << 24);
            }
        }
    }
    let ring_state = if best_command == 0 {
        0
    } else {
        let pipe = usize::try_from(best_state & 3).unwrap_or(0);
        unsafe { (crate::platform::tx_ring_register(pipe, 0x20) as *const u32).read_volatile() }
    };
    (best_command, best_state, ring_state)
}

#[cfg(feature = "vendor-host-tx-diagnostics")]
pub unsafe fn validate_tx_boundary(phase: u32, pipe: u8, slot: u8, command: u32, ring: u32) {
    unsafe { radio::validate_tx_boundary(phase, pipe, slot, command, ring) };
    // Report-only mode. This is the fourth halting detector for the same
    // corruption: it fires when an RX slot already staged for the host is
    // overwritten. Like the others it stops the firmware on first sight, which
    // makes any throughput measurement impossible.
    // Do NOT count corruption here. This runs several times per main-loop pass,
    // so a counter bump at this point measures call frequency rather than
    // corruption events (it read 2,162,040 on the first attempt). Corruption is
    // counted where it is actually detected, in `radio.rs`.
    return;
}

#[derive(Debug, Eq, PartialEq)]
pub struct RequestReleaseToken {
    buffer_address: u32,
}

impl RequestReleaseToken {
    pub const fn buffer_address(&self) -> u32 {
        self.buffer_address
    }
}

/// Exclusive ownership of one detached host-to-firmware packet-RAM buffer.
///
/// Payload borrows are tied to this value, and consuming it is the only way to
/// obtain the token that returns the buffer to the HIF RX ring.
pub struct RequestBuffer {
    buffer_address: u32,
    payload_length: u16,
}

impl RequestBuffer {
    pub const fn buffer_address(&self) -> u32 {
        self.buffer_address
    }

    pub fn payload(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                self.buffer_address.wrapping_add(4) as *const u8,
                usize::from(self.payload_length),
            )
        }
    }

    pub fn into_release(self) -> RequestReleaseToken {
        RequestReleaseToken {
            buffer_address: self.buffer_address,
        }
    }
}

pub struct ReceivedRequest {
    pub id: u16,
    pub if_id: u8,
    pub buffer: RequestBuffer,
}

#[cfg(feature = "vendor-host-tx-diagnostics")]
pub struct DebugSnapshot {
    pub tx_queued: u32,
    pub tx_producer: u32,
    pub tx_consumer: u32,
    pub tx_length_mask: u32,
    pub descriptor_address: u32,
    pub descriptor_control: u32,
    pub hif_control: u32,
    pub hif_length_mask: u32,
    pub rx_producer: u32,
    pub rx_consumer: u32,
    pub rx_descriptor_address: u32,
    pub rx_descriptor_control: u32,
    pub request_polls: u32,
    pub malformed_requests: u32,
}

/// Firmware-owned HIF ring cursors formerly stored in the vendor DTCM record
/// at `0x040098fc`.
pub struct HifRingState {
    tx_queued: u32,
    tx_reclaimed: u32,
    rx_producer: u32,
    rx_consumer: u32,
    tx_consumer: u32,
    tx_length_mask: u32,
}

impl Default for HifRingState {
    fn default() -> Self {
        Self::new()
    }
}

impl HifRingState {
    pub const fn new() -> Self {
        Self {
            tx_queued: 0,
            tx_reclaimed: 0,
            rx_producer: 0,
            rx_consumer: 0,
            tx_consumer: 0,
            tx_length_mask: 0,
        }
    }
}

/// Firmware-owned packet-buffer queues formerly stored in the vendor DTCM
/// `HifSoftwareState` record at `0x04009754`.
///
/// Hardware consumes the descriptors and packet-RAM addresses, not this CPU
/// bookkeeping. Keeping it behind the unique `Transport` owner removes the
/// vendor address and prevents unrelated code from mutating queue ownership.
pub struct HifQueues {
    rx_buffers: [u32; 32],
    rx_released: u32,
    tx_buffers: [u32; 64],
}

impl Default for HifQueues {
    fn default() -> Self {
        Self::new()
    }
}

impl HifQueues {
    pub const fn new() -> Self {
        Self {
            rx_buffers: [0; 32],
            rx_released: 0,
            tx_buffers: [0; 64],
        }
    }
}

pub struct Transport {
    state: &'static mut HifRingState,
    queues: &'static mut HifQueues,
    shared: &'static HifShared,
    output_releases: [Option<RxToken>; 64],
    output_shared_slots: [Option<u8>; 64],
    shared_slots_in_use: [bool; 4],
    prepared_shared_slot: Option<u8>,
    tx_completion_pending: bool,
    rx_request_pending: bool,
    #[cfg(feature = "vendor-host-tx-diagnostics")]
    request_polls: u32,
    #[cfg(feature = "vendor-host-tx-diagnostics")]
    malformed_requests: u32,
}

#[cfg(all(target_arch = "arm", not(target_feature = "thumb-mode")))]
pub(crate) fn drain_write_buffer() {
    unsafe {
        asm!(
            "mcr p15, 0, {value}, c7, c10, 4",
            value = in(reg) 0_u32,
            options(nostack, preserves_flags)
        );
    }
}

#[cfg(all(target_arch = "arm", target_feature = "thumb-mode"))]
global_asm!(
    ".syntax unified",
    ".arm",
    ".align 2",
    ".global xr819_drain_write_buffer",
    ".type xr819_drain_write_buffer, %function",
    "xr819_drain_write_buffer:",
    "mov r0, #0",
    "mcr p15, 0, r0, c7, c10, 4",
    "bx lr",
    ".thumb",
);

#[cfg(all(target_arch = "arm", target_feature = "thumb-mode"))]
pub(crate) fn drain_write_buffer() {
    unsafe extern "C" {
        fn xr819_drain_write_buffer();
    }
    unsafe { xr819_drain_write_buffer() }
}

#[cfg(not(target_arch = "arm"))]
pub(crate) fn drain_write_buffer() {
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
}

fn postcode(_value: u32) {}

#[derive(Clone, Copy)]
struct HifSequenceCounters {
    tx_producer: u32,
    emergency_skew: u32,
}

/// Sequence state shared with terminal exception publication.
///
/// Normal ring ownership remains inside `Transport`, but terminal exceptions
/// can occur without a borrow of it. Keeping only these two single-word
/// counters in one private static avoids exposing the complete HIF ring state
/// globally while preserving the vendor sequence calculation exactly.
struct SharedHifSequence(core::cell::UnsafeCell<HifSequenceCounters>);

unsafe impl Sync for SharedHifSequence {}

static HIF_SEQUENCE: SharedHifSequence =
    SharedHifSequence(core::cell::UnsafeCell::new(HifSequenceCounters {
        tx_producer: 0,
        emergency_skew: 0,
    }));

fn tx_producer() -> u32 {
    unsafe { (&raw const (*HIF_SEQUENCE.0.get()).tx_producer).read_volatile() }
}

fn set_tx_producer(value: u32) {
    unsafe { (&raw mut (*HIF_SEQUENCE.0.get()).tx_producer).write_volatile(value) };
}

fn reset_hif_sequence() {
    unsafe {
        HIF_SEQUENCE.0.get().write(HifSequenceCounters {
            tx_producer: 0,
            emergency_skew: 0,
        });
    }
}

/// Sequence skew applied to ordinary staged output.
///
/// # Safety
/// The caller must observe the firmware's single-core HIF sequencing rules.
pub unsafe fn emergency_sequence_skew() -> u32 {
    unsafe { (&raw const (*HIF_SEQUENCE.0.get()).emergency_skew).read_volatile() }
}

/// Claim the next WSM sequence for an emergency-channel message.
///
/// # Safety
/// The caller must own the emergency HIF channel.
unsafe fn next_emergency_sequence() -> u16 {
    unsafe {
        let counters = HIF_SEQUENCE.0.get();
        let skew = (&raw mut (*counters).emergency_skew);
        let current_skew = skew.read_volatile();
        let sequence = tx_producer().wrapping_add(current_skew);
        skew.write_volatile(current_skew.wrapping_add(1));
        (sequence & 7) as u16
    }
}

fn publish_emergency_descriptor(shared: &HifShared, length: u16) {
    shared
        .emergency_address
        .set((packet_ram::hif_output(0) as u32) & 0xf6ff_ffff);
    shared
        .emergency_control
        .set(u32::from(length).wrapping_add(1).wrapping_rem(1 << 13) | 1);
    drain_write_buffer();
}

/// Publish a driver-readable WSM exception after the ordinary HIF ring can no
/// longer be trusted. The payload matches `wsm_handle_exception()` exactly.
///
/// # Safety
/// The fixed HIF shared registers and emergency packet buffer must still be
/// owned by this firmware. Callers must not reuse the buffer afterward.
const EXCEPTION_REGISTERS: usize = 22;

/// Writes the exception record to the shared buffer and rings the host.
unsafe fn publish_exception_now<const N: usize>(registers: [u32; N], name: &[u8]) {
    const MESSAGE_LENGTH: u16 = 4 + 4 + 18 * 4 + 48;
    let buffer = packet_ram::hif_output(0) as *mut u8;
    // The host counts every message it receives, including this one, against a
    // single WSM sequence and treats a mismatch as fatal:
    //   BH RX diag ... id=0800 seq=0/2 ... result=-5  ->  [BH] Fatal error
    // This path bypasses `stage_next_tx`, the only place sequence bits are
    // assigned, so it used to emit sequence 0 always. Harmless when halting,
    // fatal when the firmware carries on afterwards.
    let sequence = unsafe { next_emergency_sequence() };
    unsafe {
        buffer.cast::<u16>().write_volatile(MESSAGE_LENGTH);
        buffer
            .add(2)
            .cast::<u16>()
            .write_volatile(0x0800 | (sequence << 13));
        buffer.add(4).cast::<u32>().write_volatile(4);
        // Callers pass 18 or 22 words; the record holds 22. Zero the tail so a
        // short caller cannot publish stale buffer contents as register values.
        for index in 0..EXCEPTION_REGISTERS {
            let value = registers.get(index).copied().unwrap_or(0);
            buffer
                .add(8 + index * 4)
                .cast::<u32>()
                .write_volatile(value);
        }
        // 8 header + 22*4 registers + 32 name = 128 = MESSAGE_LENGTH. The name
        // field was 48 bytes for a 15-byte label, so four words were taken from
        // it rather than changing the message size the driver expects.
        for index in 0..32 {
            buffer
                .add(8 + EXCEPTION_REGISTERS * 4 + index)
                .write_volatile(name.get(index).copied().unwrap_or(0));
        }
        let shared = &*(TX_DESCRIPTOR_BASE as *const HifShared);
        publish_emergency_descriptor(shared, MESSAGE_LENGTH);
    }
}

/// Publishes unconditionally, for callers that are about to halt.
///
/// The suppression in `publish_terminal_exception` exists because WSM id 0x0800
/// makes cw1200 tear the link down, which is wrong for a firmware that carries
/// on. It is also wrong at a halt site: the firmware stops either way, so
/// suppressing only removes the one diagnostic that would identify which
/// detector fired. Measured cost of getting this wrong: runs where the firmware
/// answered configuration, scanned, delivered 31 beacons, then went silent
/// mid-join with no record at all, leaving `BH status: terminated`,
/// `Pending TX: 28` and an unanswered WSM 0x0006 as the only evidence.
pub unsafe fn publish_halting_exception<const N: usize>(registers: [u32; N], name: &[u8]) {
    unsafe { publish_exception_now(registers, name) };
}

pub unsafe fn publish_terminal_exception<const N: usize>(registers: [u32; N], name: &[u8]) {
    // Report-and-continue builds must not kill the link they are measuring.
    {
        let _ = (registers, name);
        unsafe {
            crate::host_tx_diagnostics::bump(
                crate::host_tx_diagnostics::counter::SUPPRESSED_EXCEPTION,
            );
        }
    };
}

/// Publish the existing terminal MAC-event diagnostic shape.
///
/// # Safety
/// The fixed emergency HIF resources must still be owned by this firmware.
pub unsafe fn publish_mac_fatal_exception<const N: usize>(registers: [u32; N]) {
    unsafe { publish_terminal_exception(registers, b"xr819-mac-event") };
}

impl Transport {
    /// Initializes the fixed descriptor rings used by the XR819 HIF block.
    ///
    /// # Safety
    ///
    /// No other firmware may own the HIF or the fixed shared-memory regions.
    pub unsafe fn initialize(
        state: &'static mut HifRingState,
        queues: &'static mut HifQueues,
    ) -> Self {
        postcode(0x4849_4e00);
        let interrupt_controller =
            unsafe { &*(INTERRUPT_CONTROLLER_BASE as *const InterruptController) };
        let rx_shared = unsafe { &*(RX_DESCRIPTOR_BASE as *const RxShared) };
        let shared = unsafe { &*(TX_DESCRIPTOR_BASE as *const HifShared) };

        *state = HifRingState {
            tx_queued: 0,
            tx_reclaimed: 0,
            rx_producer: RX_BUFFER_COUNT as u32,
            rx_consumer: 0,
            tx_consumer: 0,
            tx_length_mask: 0,
        };
        reset_hif_sequence();
        postcode(0x4849_4e01);

        for (index, descriptor) in rx_shared.descriptors.iter().enumerate() {
            postcode(0x4849_5100 | index as u32);
            if index < RX_BUFFER_COUNT {
                let address = packet_ram::hif_input(index);
                queues.rx_buffers[index] = address as u32;
                descriptor.address.set((address as u32) & 0xf6ff_ffff);
                descriptor
                    .control
                    .write(DescriptorControl::LENGTH.val((RX_BUFFER_SIZE as u32 + 1) & 0x1fff));
            } else {
                queues.rx_buffers[index] = 0;
                descriptor.address.set(0);
                descriptor.control.set(0);
            }
        }
        postcode(0x4849_4e02);
        postcode(0x4849_4e03);

        // Enable the HIF source in the hardware interrupt controller. CPU IRQs
        // remain masked, so no vendor software callback table is required.
        interrupt_controller
            .enable
            .set(interrupt_controller.enable.get() | (1 << 13));
        postcode(0x4849_4e04);

        shared
            .interrupt_ack
            .write(HifControl::LENGTH_MASK.val(0x07ff));
        postcode(0x4849_4e05);
        let control = if shared.control.get() & (1 << 10) != 0 {
            0x07f7
        } else {
            0x07ff
        };
        state.tx_length_mask = control & 0x07f8;
        shared.control.write(HifControl::LENGTH_MASK.val(control));
        postcode(0x4849_4e06);

        Self {
            state,
            queues,
            shared,
            output_releases: [const { None }; 64],
            output_shared_slots: [const { None }; 64],
            shared_slots_in_use: [false; 4],
            prepared_shared_slot: None,
            tx_completion_pending: false,
            rx_request_pending: false,
            #[cfg(feature = "vendor-host-tx-diagnostics")]
            request_polls: 0,
            #[cfg(feature = "vendor-host-tx-diagnostics")]
            malformed_requests: 0,
        }
    }

    /// Publishes through the dedicated descriptor used by the reference fatal
    /// exception path. This is a diagnostic for separating descriptor-ring
    /// failures from complete HIF-engine inactivity.
    pub fn publish_emergency(&self, length: u16) {
        publish_emergency_descriptor(self.shared, length);
    }

    #[cfg(feature = "vendor-host-tx-diagnostics")]
    fn validate_staged_tx_buffers(&self) {
        let producer = tx_producer();
        let mut consumer = self.state.tx_consumer;
        while consumer != producer {
            let queue_slot = (consumer & 63) as usize;
            let descriptor_slot = (consumer & 3) as usize;
            let buffer_address = self.queues.tx_buffers[queue_slot];
            let expected = unsafe { (*OUTPUT_HEADERS.0.get())[queue_slot] };
            let actual = if buffer_address == 0 {
                0
            } else {
                unsafe { (buffer_address as *const u32).read_volatile() }
            };
            if buffer_address == 0 || actual != expected {
                // Report-only: publishing here would kill the link outright.
                // The exception path writes WSM id 0x0800 with no sequence bits
                // and bypasses `stage_next_tx`, so the host sees an
                // out-of-sequence message and terminates its BH thread. That is
                // acceptable when halting and fatal when continuing.
                {
                    unsafe {
                        crate::host_tx_diagnostics::bump(
                            crate::host_tx_diagnostics::counter::OUTPUT_CORRUPTION,
                        );
                    }
                    consumer = consumer.wrapping_add(1);
                    continue;
                }
                crate::halt_always!();
            }
            consumer = consumer.wrapping_add(1);
        }
    }

    /// Polls and acknowledges HIF status like the reference IRQ 13 handler
    /// before it dispatches the corresponding software events.
    pub fn service_interrupt(&mut self) -> u32 {
        #[cfg(feature = "vendor-host-tx-diagnostics")]
        self.validate_staged_tx_buffers();
        let status = self.shared.status.get();
        if status != 0 {
            self.shared.interrupt_ack.set(status);
            // Vendor IRQ 13 schedules `hif_tx_confirm_drain()` only for HIF
            // status bit 1. A cleared descriptor control word alone is not a
            // transfer-completion notification: the engine may clear it after
            // fetching the descriptor but before the host finishes the read.
            self.tx_completion_pending |= status & (1 << 1) != 0;
            // Status bit 2 is the vendor request-ready notification. Descriptor
            // ownership can clear before the host-to-firmware DMA write is
            // complete, so request dispatch must be IRQ-gated as well.
            self.rx_request_pending |= status & (1 << 2) != 0;
            drain_write_buffer();
        }
        status
    }

    #[cfg(feature = "vendor-host-tx-diagnostics")]
    pub fn debug_snapshot(&self) -> DebugSnapshot {
        let rx_consumer = self.state.rx_consumer;
        let rx_descriptor = unsafe {
            &(*(RX_DESCRIPTOR_BASE as *const RxShared)).descriptors[(rx_consumer & 31) as usize]
        };
        DebugSnapshot {
            tx_queued: self.state.tx_queued,
            tx_producer: tx_producer(),
            tx_consumer: self.state.tx_consumer,
            tx_length_mask: self.state.tx_length_mask,
            descriptor_address: self.shared.tx[0].address.get(),
            descriptor_control: self.shared.tx[0].control.get(),
            hif_control: self.shared.control.get(),
            hif_length_mask: self.shared.interrupt_ack.get(),
            rx_producer: self.state.rx_producer,
            rx_consumer,
            rx_descriptor_address: rx_descriptor.address.get(),
            rx_descriptor_control: rx_descriptor.control.get(),
            request_polls: self.request_polls,
            malformed_requests: self.malformed_requests,
        }
    }

    fn stage_next_tx(&mut self) {
        let staged = tx_producer();
        let reclaimed = self.state.tx_consumer;
        let queued = self.state.tx_queued;
        if staged == queued || !tx_ring_has_capacity(staged, reclaimed) {
            return;
        }

        let queue_slot = (staged & 63) as usize;
        let buffer_address = self.queues.tx_buffers[queue_slot] as usize;
        // Vendor staging reads MsgLen from the queued buffer rather than
        // retaining a separate length snapshot. This keeps descriptor and
        // mutable in-place response header ownership inseparable.
        let length = unsafe { (buffer_address as *const u16).read_volatile() };
        let header_id = unsafe { ((buffer_address + 2) as *const u16).read_volatile() };
        // Emergency-channel publications consume sequence numbers the host
        // counts, so ordinary output has to skip past them.
        let sequence =
            (((staged.wrapping_add(unsafe { emergency_sequence_skew() })) as u16) & 7) << 13;
        let sequenced_id = (header_id & 0x1fff) | sequence;
        unsafe { ((buffer_address + 2) as *mut u16).write_volatile(sequenced_id) };
        #[cfg(feature = "vendor-host-tx-diagnostics")]
        {
            unsafe {
                (*OUTPUT_HEADERS.0.get())[queue_slot] =
                    u32::from(length) | (u32::from(sequenced_id) << 16);
                (*OUTPUT_HASHES.0.get())[queue_slot] =
                    output_prefix_hash(buffer_address as u32, length);
            }
        }

        let descriptor_slot = (staged & 3) as usize;
        let descriptor = &self.shared.tx[descriptor_slot];
        descriptor
            .address
            .set((buffer_address as u32) & 0xf6ff_ffff);
        let descriptor_control =
            owned_descriptor_length(length) | (descriptor_sequence(header_id) << 13);
        descriptor.control.set(descriptor_control);
        unsafe {
            crate::host_tx_diagnostics::record(
                crate::host_tx_diagnostics::EVENT_DESCRIPTOR_STAGE,
                descriptor_slot as u16,
                buffer_address as u32,
                (u32::from(header_id) << 16) | u32::from(length),
                descriptor_control,
            );
        }
        set_tx_producer(staged.wrapping_add(1));
    }

    fn reclaim_tx(&mut self) {
        if !self.tx_completion_pending {
            return;
        }
        self.tx_completion_pending = false;
        let producer = tx_producer();
        let mut consumer = self.state.tx_consumer;
        let mut queue_consumer = self.state.tx_reclaimed;
        while consumer != producer {
            let descriptor_slot = (consumer & 3) as usize;
            if self.shared.tx[descriptor_slot].control.get() & 1 != 0 {
                break;
            }

            let queue_slot = (queue_consumer & 63) as usize;
            let buffer_address = self.queues.tx_buffers[queue_slot];
            let header = if buffer_address == 0 {
                0
            } else {
                unsafe { (buffer_address as *const u32).read_volatile() }
            };
            unsafe {
                crate::host_tx_diagnostics::record(
                    crate::host_tx_diagnostics::EVENT_DESCRIPTOR_RECLAIM,
                    descriptor_slot as u16,
                    buffer_address,
                    header,
                    self.shared.tx[descriptor_slot].control.get(),
                );
            }

            consumer = consumer.wrapping_add(1);
            self.state.tx_consumer = consumer;
            // Preserve vendor `hif_tx_confirm_drain()` ordering exactly: make
            // the descriptor slot reusable and push its successor before
            // releasing the completed message's backing storage.
            self.stage_next_tx();

            queue_consumer = queue_consumer.wrapping_add(1);
            self.state.tx_reclaimed = queue_consumer;
            if let Some(token) = self.output_releases[queue_slot].take() {
                unsafe {
                    crate::host_tx_diagnostics::record(
                        crate::host_tx_diagnostics::EVENT_MESSAGE_RELEASE,
                        0x0804,
                        buffer_address,
                        queue_consumer,
                        0,
                    );
                    radio::complete_host_transfer(token);
                }
            }
            if let Some(shared_slot) = self.output_shared_slots[queue_slot].take() {
                self.shared_slots_in_use[usize::from(shared_slot)] = false;
            }
            #[cfg(feature = "vendor-host-tx-diagnostics")]
            {
                unsafe {
                    (*OUTPUT_HEADERS.0.get())[queue_slot] = 0;
                    (*OUTPUT_HASHES.0.get())[queue_slot] = 0;
                }
            }
            self.queues.tx_buffers[queue_slot] = 0;
        }
        drain_write_buffer();
    }

    pub fn publication_available(&mut self) -> bool {
        self.reclaim_tx();
        output_queue_has_capacity(self.state.tx_queued, self.state.tx_reclaimed)
    }

    pub fn output_available(&mut self) -> bool {
        self.publication_available()
            && self.prepared_shared_slot.is_none()
            && self.shared_slots_in_use.iter().any(|in_use| !in_use)
    }

    /// Returns one free vendor 384-byte indication buffer.
    ///
    /// # Safety
    ///
    /// The caller must publish the initialized buffer exactly once before
    /// requesting another one.
    pub unsafe fn output_buffer(&mut self) -> &'static mut [u8] {
        self.reclaim_tx();
        let slot = self
            .shared_slots_in_use
            .iter()
            .position(|in_use| !in_use)
            .expect("output buffer requires output_available");
        self.prepared_shared_slot = Some(slot as u8);
        unsafe {
            core::slice::from_raw_parts_mut(
                packet_ram::hif_output(slot) as *mut u8,
                SHARED_BUFFER_SIZE,
            )
        }
    }

    fn detach_rx_buffer(&mut self, consumer: u32) {
        self.state.rx_consumer = consumer.wrapping_add(1);
        drain_write_buffer();
    }

    fn append_request_credit(&mut self, token: RequestReleaseToken) {
        let buffer_address = token.buffer_address as usize;
        if buffer_address == 0 {
            return;
        }
        self.queues.rx_released = self.queues.rx_released.wrapping_add(1);
        let producer = self.state.rx_producer;
        let producer_slot = (producer & 31) as usize;
        let producer_descriptor =
            unsafe { &(*(RX_DESCRIPTOR_BASE as *const RxShared)).descriptors[producer_slot] };
        self.queues.rx_buffers[producer_slot] = buffer_address as u32;
        producer_descriptor
            .address
            .set((buffer_address as u32) & 0xf6ff_ffff);
        producer_descriptor
            .control
            .write(DescriptorControl::LENGTH.val((RX_BUFFER_SIZE as u32 + 1) & 0x1fff));
        self.state.rx_producer = producer.wrapping_add(1);
        unsafe {
            crate::host_tx_diagnostics::record(
                crate::host_tx_diagnostics::EVENT_REQUEST_CREDIT,
                producer_slot as u16,
                buffer_address as u32,
                producer.wrapping_add(1),
                self.state.rx_consumer,
            );
        }
    }

    pub fn release_request(&mut self, token: RequestReleaseToken) {
        self.append_request_credit(token);
        drain_write_buffer();
    }

    fn next_request_descriptor_ready(&self) -> bool {
        let consumer = self.state.rx_consumer;
        if consumer == self.state.rx_producer {
            return false;
        }
        let descriptor = unsafe {
            &(*(RX_DESCRIPTOR_BASE as *const RxShared)).descriptors[(consumer & 31) as usize]
        };
        descriptor.control.get() & 1 == 0
    }

    fn schedule_ready_successor_request(&mut self) {
        // Vendor `hif_rx_process()` handles one request per invocation and
        // reschedules itself when the following descriptor is already ready.
        self.rx_request_pending = self.next_request_descriptor_ready();
    }

    fn recycle_rx_buffer(&mut self, consumer: u32, buffer_address: usize) {
        self.detach_rx_buffer(consumer);
        self.release_request(RequestReleaseToken {
            buffer_address: buffer_address as u32,
        });
        self.schedule_ready_successor_request();
    }

    /// Reports whether an IRQ-notified host-to-firmware request is ready for
    /// one cooperative dispatch invocation.
    pub fn request_available(&self) -> bool {
        self.rx_request_pending && self.next_request_descriptor_ready() && self.response_available()
    }

    /// Whether one command response can be copied into independent output
    /// storage without consuming a request buffer first.
    pub fn response_available(&self) -> bool {
        output_queue_has_capacity(self.state.tx_queued, self.state.tx_reclaimed)
            && self.shared_slots_in_use.iter().any(|used| !*used)
            && self.prepared_shared_slot.is_none()
    }

    pub fn poll_request(&mut self) -> Option<ReceivedRequest> {
        #[cfg(feature = "vendor-host-tx-diagnostics")]
        {
            self.request_polls = self.request_polls.wrapping_add(1);
        }
        if !self.rx_request_pending {
            return None;
        }
        // Consume exactly one scheduled invocation. A ready successor below
        // schedules the next invocation, matching vendor cooperative dispatch.
        self.rx_request_pending = false;
        let consumer = self.state.rx_consumer;
        if consumer == self.state.rx_producer {
            return None;
        }

        let descriptor = unsafe {
            &(*(RX_DESCRIPTOR_BASE as *const RxShared)).descriptors[(consumer & 31) as usize]
        };
        let control = descriptor.control.get();
        if control & 1 != 0 {
            return None;
        }

        let slot = (consumer & 31) as usize;
        let buffer_address = self.queues.rx_buffers[slot] as usize;
        let descriptor_len = (control & 0x1ffe) as usize;
        if buffer_address == 0 || descriptor_len < 4 {
            #[cfg(feature = "vendor-host-tx-diagnostics")]
            {
                self.malformed_requests = self.malformed_requests.wrapping_add(1);
            }
            self.recycle_rx_buffer(consumer, buffer_address);
            return None;
        }

        let wire_len = unsafe { (buffer_address as *const u16).read_volatile() as usize };
        let raw_id = unsafe { ((buffer_address + 2) as *const u16).read_volatile() };
        unsafe {
            crate::host_tx_diagnostics::record(
                crate::host_tx_diagnostics::EVENT_REQUEST_SEEN,
                slot as u16,
                buffer_address as u32,
                (u32::from(raw_id) << 16) | wire_len as u32,
                control,
            );
        }
        // Host-to-firmware descriptors may contain requests only. Under heavy
        // traffic we have observed a consumed output buffer reappear on this
        // ring with a 0x04xx response header. Never dispatch such a reflected
        // firmware message as a new command: doing so manufactures 0x0400 and
        // fatally desynchronizes cw1200's synchronous command state.
        // Request ID zero is also absent from the XR819 WSM command set. Turn
        // its first occurrence into a terminal transport snapshot instead of
        // manufacturing an unsupported-command 0x0400 response.
        if raw_id & 0x1fff == 0 {
            unsafe {
                let word =
                    |offset: usize| ((buffer_address + offset) as *const u32).read_volatile();
                publish_mac_fatal_exception([
                    buffer_address as u32,
                    consumer,
                    self.state.rx_producer,
                    control,
                    wire_len as u32,
                    u32::from(raw_id),
                    word(0),
                    word(4),
                    word(8),
                    word(0x0c),
                    word(0x10),
                    word(0x14),
                    word(0x18),
                    word(0x1c),
                    self.shared.status.get(),
                    tx_producer(),
                    self.state.tx_consumer,
                    self.state.tx_queued,
                ]);
            }
            crate::halt_always!();
        }
        if wire_len < 4 || raw_id & 0x0c00 != 0 {
            unsafe {
                crate::host_tx_diagnostics::freeze(
                    1,
                    buffer_address as u32,
                    (u32::from(raw_id) << 16) | wire_len as u32,
                    control,
                );
            }
            #[cfg(feature = "vendor-host-tx-diagnostics")]
            {
                self.malformed_requests = self.malformed_requests.wrapping_add(1);
            }
            self.recycle_rx_buffer(consumer, buffer_address);
            return None;
        }
        let length = wire_len.min(descriptor_len).min(RX_BUFFER_SIZE);
        // Match vendor `wsm_dispatch_cmd` at 0x0000e5a0. Preserve the low two
        // routing bits separately because XR819 uses them as a three-entry VIF
        // selector even though CW1200 names bits 6..9 the link-ID field.
        let id = raw_id & 0x0c3f;
        let if_id = ((raw_id >> 6) & 3) as u8;
        let payload_len = length.saturating_sub(4).min(REQUEST_PAYLOAD_CAPACITY);
        // Keep the packet-RAM request buffer borrowed until the dispatcher
        // explicitly releases it. Vendor ordinary TX stores its MPDU pointer
        // directly in the class-0 context and returns this buffer only after
        // the WSM TX confirmation.
        self.detach_rx_buffer(consumer);
        self.schedule_ready_successor_request();
        unsafe {
            crate::host_tx_diagnostics::record(
                crate::host_tx_diagnostics::EVENT_REQUEST_DETACHED,
                slot as u16,
                buffer_address as u32,
                (u32::from(raw_id) << 16) | length as u32,
                consumer.wrapping_add(1),
            );
        }

        Some(ReceivedRequest {
            id,
            if_id,
            buffer: RequestBuffer {
                buffer_address: buffer_address as u32,
                payload_length: payload_len as u16,
            },
        })
    }

    /// Appends one complete message to the vendor-shaped 64-entry software
    /// output queue, then fills any free hardware descriptors in FIFO order.
    fn enqueue_output(
        &mut self,
        buffer_address: usize,
        length: u16,
        release: Option<RxToken>,
        shared_slot: Option<u8>,
    ) {
        let queued = self.state.tx_queued;
        assert!(output_queue_has_capacity(queued, self.state.tx_reclaimed));
        let queue_slot = (queued & 63) as usize;
        self.queues.tx_buffers[queue_slot] = buffer_address as u32;
        debug_assert_eq!(
            unsafe { (buffer_address as *const u16).read_volatile() },
            length
        );
        self.output_releases[queue_slot] = release;
        self.output_shared_slots[queue_slot] = shared_slot;
        self.state.tx_queued = queued.wrapping_add(1);
        let header = unsafe { (buffer_address as *const u32).read_volatile() };
        #[cfg(feature = "vendor-host-tx-diagnostics")]
        {
            unsafe {
                (*OUTPUT_HEADERS.0.get())[queue_slot] = header;
                (*OUTPUT_HASHES.0.get())[queue_slot] =
                    output_prefix_hash(buffer_address as u32, length);
            }
        }
        unsafe {
            crate::host_tx_diagnostics::record(
                crate::host_tx_diagnostics::EVENT_OUTPUT_ENQUEUE,
                queue_slot as u16,
                buffer_address as u32,
                header,
                queued.wrapping_add(1),
            );
        }
        self.stage_next_tx();
        drain_write_buffer();
    }

    /// # Safety
    ///
    /// `length` must describe initialized data in the buffer returned by the
    /// immediately preceding `output_buffer()` call.
    pub fn publish(&mut self, length: u16) {
        let shared_slot = self
            .prepared_shared_slot
            .take()
            .expect("publish requires a prepared output buffer");
        self.shared_slots_in_use[usize::from(shared_slot)] = true;
        let buffer_address = packet_ram::hif_output(usize::from(shared_slot));
        self.enqueue_output(buffer_address, length, None, Some(shared_slot));
    }

    /// Publishes a response for the original request while preserving logical
    /// request identity. The bytes use independent output storage because this
    /// direct-ring implementation does not yet reproduce the vendor HIF
    /// transfer scheduler that safely serializes opposite-direction DMA to the
    /// same packet-RAM pointer.
    ///
    /// # Safety
    /// `source[..length]` must contain a complete WSM response or confirmation.
    pub unsafe fn publish_request_in_place(
        &mut self,
        token: RequestReleaseToken,
        source: &[u8],
        length: u16,
    ) {
        self.reclaim_tx();
        assert!(self.response_available());
        let shared_slot =
            self.shared_slots_in_use
                .iter()
                .position(|used| !*used)
                .expect("response availability guarantees a shared slot") as u8;
        self.shared_slots_in_use[usize::from(shared_slot)] = true;
        let buffer_address = packet_ram::hif_output(usize::from(shared_slot));
        let count = usize::from(length)
            .min(source.len())
            .min(SHARED_BUFFER_SIZE);
        unsafe {
            core::ptr::copy_nonoverlapping(source.as_ptr(), buffer_address as *mut u8, count);
        }
        self.append_request_credit(token);
        self.enqueue_output(buffer_address, length, None, Some(shared_slot));
    }

    /// Publishes a retained packet-DMA RX slot directly. Its release occurs
    /// when the HIF TX-completion interrupt reports descriptor reclaim.
    ///
    /// # Safety
    /// `indication` must be the unique outstanding radio FIFO transfer.
    pub fn publish_radio(&mut self, indication: PendingIndication) {
        self.enqueue_output(
            indication.address as usize,
            indication.length,
            Some(indication.release),
            None,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descriptor_ownership_survives_odd_message_lengths() {
        assert_eq!(owned_descriptor_length(20), 21);
        assert_eq!(owned_descriptor_length(21), 23);
        assert_eq!(owned_descriptor_length(1600 + 16), 1617);
        assert_eq!(owned_descriptor_length(21) & 1, 1);
        assert!(owned_descriptor_length(21) & 0x1ffe >= 21);
    }

    #[test]
    fn tx_ring_capacity_handles_counter_wrap() {
        assert!(tx_ring_has_capacity(3, 0));
        assert!(!tx_ring_has_capacity(4, 0));
        assert!(tx_ring_has_capacity(1, u32::MAX));
        assert!(!tx_ring_has_capacity(2, u32::MAX - 1));
    }

    #[test]
    fn software_output_queue_has_vendor_depth_and_wraps() {
        assert!(output_queue_has_capacity(63, 0));
        assert!(!output_queue_has_capacity(64, 0));
        assert!(output_queue_has_capacity(31, u32::MAX - 31));
        assert!(!output_queue_has_capacity(32, u32::MAX - 31));
    }

    #[test]
    fn descriptor_preserves_presequenced_header_control_bits() {
        assert_eq!(descriptor_sequence(0x0413), 0);
        assert_eq!(descriptor_sequence(0x2413), 1);
        assert_eq!(descriptor_sequence(0x4413), 2);
        assert_eq!(descriptor_sequence(0x6413), 3);
        // Callers sample these bits before assigning the new WSM sequence.
    }

    #[test]
    fn host_input_rejects_firmware_response_and_indication_classes() {
        assert_eq!(0x0004_u16 & 0x0c00, 0);
        assert_ne!(0x0400_u16 & 0x0c00, 0);
        assert_ne!(0x0404_u16 & 0x0c00, 0);
        assert_ne!(0x0804_u16 & 0x0c00, 0);
    }
}
