//! Minimal XR819 host-interface descriptor transport.
//!
//! The addresses and descriptor layout are intentionally kept close to the
//! vendor firmware while the transport is being reconstructed.

#[cfg(all(target_arch = "arm", not(target_feature = "thumb-mode")))]
use core::arch::asm;
#[cfg(all(target_arch = "arm", target_feature = "thumb-mode"))]
use core::arch::global_asm;

use crate::radio::{self, PendingIndication, ReleaseToken};
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

    HifState {
        (0x00 => tx_queued: ReadWrite<u32>),
        (0x04 => tx_reclaimed: ReadWrite<u32>),
        (0x08 => rx_producer: ReadWrite<u32>),
        (0x0c => rx_consumer: ReadWrite<u32>),
        (0x10 => rx_mask: ReadWrite<u32>),
        (0x14 => rx_descriptors: ReadWrite<u32>),
        (0x18 => tx_producer: ReadWrite<u32>),
        (0x1c => tx_consumer: ReadWrite<u32>),
        (0x20 => tx_mask: ReadWrite<u32>),
        (0x24 => tx_descriptors: ReadWrite<u32>),
        (0x28 => tx_length_mask: ReadWrite<u32, HifControl::Register>),
        (0x2c => @END),
    },

    ControlShadow {
        (0x00 => rx_available: ReadWrite<u32>),
        (0x04 => tx_pending: ReadWrite<u32>),
        (0x08 => @END),
    },

    HifSoftwareState {
        (0x00 => mode: ReadWrite<u32>),
        (0x04 => rx_credit: ReadWrite<u16>),
        (0x06 => _reserved0),
        (0x08 => activity_timestamp: ReadWrite<u32>),
        (0x0c => _reserved1),
        (0x20 => rx_buffers: [ReadWrite<u32>; 32]),
        (0xa0 => rx_released: ReadWrite<u32>),
        (0xa4 => buffer_size: ReadWrite<u16>),
        (0xa6 => _reserved2),
        (0xa8 => tx_buffers: [ReadWrite<u32>; 64]),
        (0x1a8 => @END),
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

const STATE_BASE: usize = 0x0400_98fc;
const INTERRUPT_CONTROLLER_BASE: usize = 0x0a88_0000;
const IRQ_CALLBACK_TABLE: usize = 0x0400_11bc;
const RX_DESCRIPTOR_BASE: usize = 0x0ab0_0000;
const TX_DESCRIPTOR_BASE: usize = 0x0ab0_0100;
const CONTROL_SHADOW_BASE: usize = 0x0400_11ac;
const HIF_SOFTWARE_STATE_BASE: usize = 0x0400_9754;
const RX_BUFFER_BASE: usize = 0x0900_8a68;
const RX_BUFFER_SIZE: usize = 1632;
const RX_BUFFER_COUNT: usize = 30;

/// First of the four vendor TX buffers allocated by `0x0000094c`. The exact
/// pre-HIF clock transition makes this packet-memory bank CPU-accessible.
pub const SHARED_BUFFER_BASE: usize = 0x0901_49a8;
pub const SHARED_BUFFER_SIZE: usize = 384;
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

pub struct Transport {
    state: &'static HifState,
    software_state: &'static HifSoftwareState,
    shared: &'static HifShared,
    output_releases: [Option<ReleaseToken>; 64],
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
fn drain_write_buffer() {
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
fn drain_write_buffer() {
    unsafe extern "C" {
        fn xr819_drain_write_buffer();
    }
    unsafe { xr819_drain_write_buffer() }
}

#[cfg(not(target_arch = "arm"))]
fn drain_write_buffer() {
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
}

fn postcode(_value: u32) {}

fn publish_emergency_descriptor(shared: &HifShared, length: u16) {
    shared
        .emergency_address
        .set((SHARED_BUFFER_BASE as u32) & 0xf6ff_ffff);
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
pub unsafe fn publish_terminal_exception(registers: [u32; 18], name: &[u8]) {
    const MESSAGE_LENGTH: u16 = 4 + 4 + 18 * 4 + 48;
    let buffer = SHARED_BUFFER_BASE as *mut u8;
    unsafe {
        buffer.cast::<u16>().write_volatile(MESSAGE_LENGTH);
        buffer.add(2).cast::<u16>().write_volatile(0x0800);
        buffer.add(4).cast::<u32>().write_volatile(4);
        for (index, value) in registers.into_iter().enumerate() {
            buffer
                .add(8 + index * 4)
                .cast::<u32>()
                .write_volatile(value);
        }
        for index in 0..48 {
            buffer
                .add(8 + 18 * 4 + index)
                .write_volatile(name.get(index).copied().unwrap_or(0));
        }
        let shared = &*(TX_DESCRIPTOR_BASE as *const HifShared);
        publish_emergency_descriptor(shared, MESSAGE_LENGTH);
    }
}

/// Publish the existing terminal MAC-event diagnostic shape.
///
/// # Safety
/// The fixed emergency HIF resources must still be owned by this firmware.
pub unsafe fn publish_mac_fatal_exception(registers: [u32; 18]) {
    unsafe { publish_terminal_exception(registers, b"xr819-mac-event") };
}

extern "C" fn diagnostic_hif_irq_stub() {}

impl Transport {
    /// Initializes the fixed descriptor rings used by the XR819 HIF block.
    ///
    /// # Safety
    ///
    /// No other firmware may own the HIF or the fixed shared-memory regions.
    pub unsafe fn initialize() -> Self {
        postcode(0x4849_4e00);
        let state = unsafe { &*(STATE_BASE as *const HifState) };
        let interrupt_controller =
            unsafe { &*(INTERRUPT_CONTROLLER_BASE as *const InterruptController) };
        let rx_shared = unsafe { &*(RX_DESCRIPTOR_BASE as *const RxShared) };
        let control_shadow = unsafe { &*(CONTROL_SHADOW_BASE as *const ControlShadow) };
        let software_state = unsafe { &*(HIF_SOFTWARE_STATE_BASE as *const HifSoftwareState) };
        let shared = unsafe { &*(TX_DESCRIPTOR_BASE as *const HifShared) };

        state.tx_queued.set(0);
        state.tx_reclaimed.set(0);
        state.rx_producer.set(RX_BUFFER_COUNT as u32);
        state.rx_consumer.set(0);
        state.rx_mask.set(31);
        state.rx_descriptors.set(RX_DESCRIPTOR_BASE as u32);
        state.tx_producer.set(0);
        state.tx_consumer.set(0);
        state.tx_mask.set(3);
        state.tx_descriptors.set(TX_DESCRIPTOR_BASE as u32);
        postcode(0x4849_4e01);

        for (index, descriptor) in rx_shared.descriptors.iter().enumerate() {
            postcode(0x4849_5100 | index as u32);
            if index < RX_BUFFER_COUNT {
                let address = RX_BUFFER_BASE + index * RX_BUFFER_SIZE;
                software_state.rx_buffers[index].set(address as u32);
                descriptor.address.set((address as u32) & 0xf6ff_ffff);
                descriptor
                    .control
                    .write(DescriptorControl::LENGTH.val((RX_BUFFER_SIZE as u32 + 1) & 0x1fff));
            } else {
                software_state.rx_buffers[index].set(0);
                descriptor.address.set(0);
                descriptor.control.set(0);
            }
        }
        postcode(0x4849_4e02);
        software_state.buffer_size.set(RX_BUFFER_SIZE as u16);
        control_shadow.rx_available.set(RX_BUFFER_COUNT as u32);
        control_shadow.tx_pending.set(0);
        postcode(0x4849_4e03);

        // Literal callback-table and source-enable effects of the vendor's
        // register_irq(13, callback) call. CPU IRQs remain masked until the
        // real handler and scheduler have been translated.
        let callback = diagnostic_hif_irq_stub as *const () as usize as u32 | 1;
        unsafe {
            ((IRQ_CALLBACK_TABLE + (31 - 13) * 4) as *mut u32).write_volatile(callback);
        }
        interrupt_controller
            .enable
            .set(interrupt_controller.enable.get() | (1 << 13));
        postcode(0x4849_4e04);

        shared
            .interrupt_ack
            .write(HifControl::LENGTH_MASK.val(0x07ff));
        postcode(0x4849_4e05);
        let control = if software_state.mode.get() == 1 && shared.control.get() & (1 << 10) != 0 {
            0x07f7
        } else {
            0x07ff
        };
        state
            .tx_length_mask
            .write(HifControl::LENGTH_MASK.val(control & 0x07f8));
        shared.control.write(HifControl::LENGTH_MASK.val(control));
        postcode(0x4849_4e06);

        Self {
            state,
            software_state,
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

    /// Polls and acknowledges HIF status like the reference IRQ 13 handler
    /// before it dispatches the corresponding software events.
    pub fn service_interrupt(&mut self) -> u32 {
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
        let rx_consumer = self.state.rx_consumer.get();
        let rx_descriptor = unsafe {
            &(*(RX_DESCRIPTOR_BASE as *const RxShared)).descriptors[(rx_consumer & 31) as usize]
        };
        DebugSnapshot {
            tx_queued: self.state.tx_queued.get(),
            tx_producer: self.state.tx_producer.get(),
            tx_consumer: self.state.tx_consumer.get(),
            tx_length_mask: self.state.tx_length_mask.get(),
            descriptor_address: self.shared.tx[0].address.get(),
            descriptor_control: self.shared.tx[0].control.get(),
            hif_control: self.shared.control.get(),
            hif_length_mask: self.shared.interrupt_ack.get(),
            rx_producer: self.state.rx_producer.get(),
            rx_consumer,
            rx_descriptor_address: rx_descriptor.address.get(),
            rx_descriptor_control: rx_descriptor.control.get(),
            request_polls: self.request_polls,
            malformed_requests: self.malformed_requests,
        }
    }

    fn stage_next_tx(&mut self) {
        let staged = self.state.tx_producer.get();
        let reclaimed = self.state.tx_consumer.get();
        let queued = self.state.tx_queued.get();
        if staged == queued || !tx_ring_has_capacity(staged, reclaimed) {
            return;
        }

        let queue_slot = (staged & 63) as usize;
        let buffer_address = self.software_state.tx_buffers[queue_slot].get() as usize;
        // Vendor staging reads MsgLen from the queued buffer rather than
        // retaining a separate length snapshot. This keeps descriptor and
        // mutable in-place response header ownership inseparable.
        let length = unsafe { (buffer_address as *const u16).read_volatile() };
        let header_id = unsafe { ((buffer_address + 2) as *const u16).read_volatile() };
        let sequence = ((staged as u16) & 7) << 13;
        let sequenced_id = (header_id & 0x1fff) | sequence;
        unsafe { ((buffer_address + 2) as *mut u16).write_volatile(sequenced_id) };

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
        self.state.tx_producer.set(staged.wrapping_add(1));
    }

    fn reclaim_tx(&mut self) {
        if !self.tx_completion_pending {
            return;
        }
        self.tx_completion_pending = false;
        let producer = self.state.tx_producer.get();
        let mut consumer = self.state.tx_consumer.get();
        let mut queue_consumer = self.state.tx_reclaimed.get();
        while consumer != producer {
            let descriptor_slot = (consumer & 3) as usize;
            if self.shared.tx[descriptor_slot].control.get() & 1 != 0 {
                break;
            }

            let queue_slot = (queue_consumer & 63) as usize;
            let buffer_address = self.software_state.tx_buffers[queue_slot].get();
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
            self.state.tx_consumer.set(consumer);
            // Preserve vendor `hif_tx_confirm_drain()` ordering exactly: make
            // the descriptor slot reusable and push its successor before
            // releasing the completed message's backing storage.
            self.stage_next_tx();

            queue_consumer = queue_consumer.wrapping_add(1);
            self.state.tx_reclaimed.set(queue_consumer);
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
            self.software_state.tx_buffers[queue_slot].set(0);
        }
        drain_write_buffer();
    }

    pub fn publication_available(&mut self) -> bool {
        self.reclaim_tx();
        output_queue_has_capacity(self.state.tx_queued.get(), self.state.tx_reclaimed.get())
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
                (SHARED_BUFFER_BASE + slot * SHARED_BUFFER_SIZE) as *mut u8,
                SHARED_BUFFER_SIZE,
            )
        }
    }

    fn detach_rx_buffer(&mut self, consumer: u32) {
        self.state.rx_consumer.set(consumer.wrapping_add(1));
        drain_write_buffer();
    }

    fn append_request_credit(&mut self, token: RequestReleaseToken) {
        let buffer_address = token.buffer_address as usize;
        if buffer_address == 0 {
            return;
        }
        self.software_state
            .rx_released
            .set(self.software_state.rx_released.get().wrapping_add(1));
        let producer = self.state.rx_producer.get();
        let producer_slot = (producer & 31) as usize;
        let producer_descriptor =
            unsafe { &(*(RX_DESCRIPTOR_BASE as *const RxShared)).descriptors[producer_slot] };
        self.software_state.rx_buffers[producer_slot].set(buffer_address as u32);
        producer_descriptor
            .address
            .set((buffer_address as u32) & 0xf6ff_ffff);
        producer_descriptor
            .control
            .write(DescriptorControl::LENGTH.val((RX_BUFFER_SIZE as u32 + 1) & 0x1fff));
        self.state.rx_producer.set(producer.wrapping_add(1));
        unsafe {
            crate::host_tx_diagnostics::record(
                crate::host_tx_diagnostics::EVENT_REQUEST_CREDIT,
                producer_slot as u16,
                buffer_address as u32,
                producer.wrapping_add(1),
                self.state.rx_consumer.get(),
            );
        }
    }

    pub fn release_request(&mut self, token: RequestReleaseToken) {
        self.append_request_credit(token);
        drain_write_buffer();
    }

    fn next_request_descriptor_ready(&self) -> bool {
        let consumer = self.state.rx_consumer.get();
        if consumer == self.state.rx_producer.get() {
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
        self.rx_request_pending
            && self.next_request_descriptor_ready()
            && self.response_available()
    }

    /// Whether one command response can be copied into independent output
    /// storage without consuming a request buffer first.
    pub fn response_available(&self) -> bool {
        output_queue_has_capacity(self.state.tx_queued.get(), self.state.tx_reclaimed.get())
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
        let consumer = self.state.rx_consumer.get();
        if consumer == self.state.rx_producer.get() {
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
        let buffer_address = self.software_state.rx_buffers[slot].get() as usize;
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
                let word = |offset: usize| {
                    ((buffer_address + offset) as *const u32).read_volatile()
                };
                publish_mac_fatal_exception([
                    buffer_address as u32,
                    consumer,
                    self.state.rx_producer.get(),
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
                    self.state.tx_producer.get(),
                    self.state.tx_consumer.get(),
                    self.state.tx_queued.get(),
                ]);
            }
            loop {
                core::hint::spin_loop();
            }
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
        release: Option<ReleaseToken>,
        shared_slot: Option<u8>,
    ) {
        let queued = self.state.tx_queued.get();
        assert!(output_queue_has_capacity(
            queued,
            self.state.tx_reclaimed.get()
        ));
        let queue_slot = (queued & 63) as usize;
        self.software_state.tx_buffers[queue_slot].set(buffer_address as u32);
        debug_assert_eq!(
            unsafe { (buffer_address as *const u16).read_volatile() },
            length
        );
        self.output_releases[queue_slot] = release;
        self.output_shared_slots[queue_slot] = shared_slot;
        self.state.tx_queued.set(queued.wrapping_add(1));
        let header = unsafe { (buffer_address as *const u32).read_volatile() };
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
        let buffer_address = SHARED_BUFFER_BASE + usize::from(shared_slot) * SHARED_BUFFER_SIZE;
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
        let shared_slot = self
            .shared_slots_in_use
            .iter()
            .position(|used| !*used)
            .expect("response availability guarantees a shared slot") as u8;
        self.shared_slots_in_use[usize::from(shared_slot)] = true;
        let buffer_address = SHARED_BUFFER_BASE + usize::from(shared_slot) * SHARED_BUFFER_SIZE;
        let count = usize::from(length).min(source.len()).min(SHARED_BUFFER_SIZE);
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
