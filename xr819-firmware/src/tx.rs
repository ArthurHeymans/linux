//! Internal management-frame TX preparation.
//!
//! This module intentionally stops before hardware publication. The vendor
//! queue/pipe ownership and IRQ completion path must be translated before a
//! prepared frame can become DMA-owned.

use core::cell::UnsafeCell;

use crate::configuration::MAX_TEMPLATE_FRAME_LEN;
use crate::packet_ram;

const TX_CONTEXT_SIZE: usize = crate::dtcm::INTERNAL_TX_CONTEXT_SIZE;
const TX_CONTEXT_COUNT: usize = crate::dtcm::INTERNAL_TX_CONTEXT_COUNT;
const TX_BUFFER_SIZE: usize = packet_ram::INTERNAL_TX_BUFFER_SIZE;
const FRAME_NODE_OFFSET: u32 = 0x54;
// The class-0 allocation counter remains in the untranslated vendor
// accounting record. Its independently decoded non-class-0 counter, probe
// sequence, PAS accounting, and completed-frame FIFO are native Rust state.
const COMPLETION_RING_CAPACITY: usize = 64;

#[inline(always)]
fn internal_context_free_head() -> *mut u32 {
    crate::dtcm::internal_context_free_head_ptr()
}

#[inline(always)]
fn internal_context_address(index: usize) -> crate::dtcm::InternalContextAddress {
    crate::dtcm::InternalContextAddress::from_index(index).unwrap_or_else(|| unreachable!())
}

#[repr(C)]
struct CompletionRingState {
    consumer: u32,
    producer: u32,
    frame_nodes: [u32; COMPLETION_RING_CAPACITY],
}

struct SharedCompletionRing(UnsafeCell<CompletionRingState>);

unsafe impl Sync for SharedCompletionRing {}

static COMPLETION_RING: SharedCompletionRing =
    SharedCompletionRing(UnsafeCell::new(CompletionRingState {
        consumer: 0,
        producer: 0,
        frame_nodes: [0; COMPLETION_RING_CAPACITY],
    }));

struct SharedProbeContextSequence(UnsafeCell<u16>);

unsafe impl Sync for SharedProbeContextSequence {}

static PROBE_CONTEXT_SEQUENCE: SharedProbeContextSequence =
    SharedProbeContextSequence(UnsafeCell::new(0));

unsafe fn probe_context_sequence() -> u16 {
    unsafe { PROBE_CONTEXT_SEQUENCE.0.get().read_volatile() }
}

unsafe fn set_probe_context_sequence(value: u16) {
    unsafe { PROBE_CONTEXT_SEQUENCE.0.get().write_volatile(value) };
}

struct SharedRetryRandomState(UnsafeCell<u32>);

unsafe impl Sync for SharedRetryRandomState {}

static RETRY_RANDOM_STATE: SharedRetryRandomState = SharedRetryRandomState(UnsafeCell::new(0));

pub(crate) unsafe fn initialize_retry_random_state() {
    unsafe { RETRY_RANDOM_STATE.0.get().write_volatile(0x1234_5678) };
}

struct SharedInternalContextCount(UnsafeCell<u8>);

unsafe impl Sync for SharedInternalContextCount {}

static INTERNAL_CONTEXT_COUNT: SharedInternalContextCount =
    SharedInternalContextCount(UnsafeCell::new(0));

#[inline(always)]
unsafe fn active_internal_contexts() -> u8 {
    unsafe { INTERNAL_CONTEXT_COUNT.0.get().read_volatile() }
}

#[inline(always)]
unsafe fn set_active_internal_contexts(value: u8) {
    unsafe { INTERNAL_CONTEXT_COUNT.0.get().write_volatile(value) };
}

struct SharedPasAccounting(UnsafeCell<u16>);

unsafe impl Sync for SharedPasAccounting {}

static PAS_ACCOUNTING: SharedPasAccounting = SharedPasAccounting(UnsafeCell::new(0));

#[inline(always)]
pub(crate) unsafe fn active_pas_contexts() -> u16 {
    unsafe { PAS_ACCOUNTING.0.get().read_volatile() }
}

#[inline(always)]
pub(crate) unsafe fn set_active_pas_contexts(value: u16) {
    unsafe { PAS_ACCOUNTING.0.get().write_volatile(value) };
}

impl SharedCompletionRing {
    unsafe fn cursors(&self) -> (u32, u32) {
        let state = self.0.get();
        unsafe {
            (
                (&raw const (*state).consumer).read_volatile(),
                (&raw const (*state).producer).read_volatile(),
            )
        }
    }

    unsafe fn frame_node(&self, index: u32) -> u32 {
        let state = self.0.get();
        unsafe { (&raw const (*state).frame_nodes[index as usize]).read_volatile() }
    }

    unsafe fn enqueue(&self, frame_node: FrameNodeAddress) {
        let state = self.0.get();
        unsafe {
            let producer = (&raw const (*state).producer).read_volatile();
            (&raw mut (*state).frame_nodes[producer as usize]).write_volatile(frame_node.raw());
            (&raw mut (*state).producer).write_volatile(producer.wrapping_add(1) & 0x3f);
        }
    }

    unsafe fn pop(&self, consumer: u32) -> (FrameNodeAddress, u32) {
        let state = self.0.get();
        unsafe {
            let slot = &raw mut (*state).frame_nodes[consumer as usize];
            let frame_node = slot.read_volatile();
            slot.write_volatile(0);
            let next = consumer.wrapping_add(1) & 0x3f;
            (&raw mut (*state).consumer).write_volatile(next);
            (FrameNodeAddress::new(frame_node), next)
        }
    }
}

const PIPE_IRQ_PENDING: u32 = crate::platform::mac_register(0x0e84) as u32;
const PIPE_IRQ_TRIGGER: u32 = crate::platform::mac_register(0x0e98) as u32;
const PIPE_QUANTUM: u32 = 0x0000_0fff;
const PIPE_STATUS_COUNTER: u32 = 0xfff0_1aa4;
const PIPE_RETRY_INACTIVE_SENTINEL: u32 = 0xff00_ffff;
// `txp_pipe_advance_slot` acknowledges with `-((0x1110 << pipe) + 0x10)`, which
// is a different lane from the `0x100 << pipe` publication ownership mask.
const PIPE_ADVANCE_ACK_BASE: u32 = 0x0000_1110;
const PIPE_RETRY_SPECIAL_ACK: u32 = 0x0000_f010;
const PIPE_RETRY_RANDOM_STATS: u32 = 0xfff0_2e7c;
const MAC_EVENT_READINESS: u32 = crate::platform::mac_register(0x0a24) as u32;
#[cfg(target_arch = "arm")]
const INTERRUPT_PENDING: usize = 0x0a88_0020;
// `tsf_timer_reload` writes interrupt-controller configuration 0x1600a037.
// Its high source byte selects hardware source 0x16 for the direct ARM FIQ
// vector, whose body is exactly `mac_irq_handler`.
const MAC_FIQ_SOURCE: u32 = 0x16;
const TX_TRACE_MAGIC: u32 = 0x5458_4558; // "TXEX"
const TX_TRACE_PUBLISHED: u32 = 1 << 0;
const TX_TRACE_GO: u32 = 1 << 1;
const TX_TRACE_FIQ: u32 = 1 << 2;
const TX_TRACE_POP: u32 = 1 << 3;
const TX_TRACE_BIT23: u32 = 1 << 4;
const TX_TRACE_PHASE2: u32 = 1 << 5;
const TX_TRACE_START: u32 = 1 << 6;
const TX_TRACE_PHY2: u32 = 1 << 7;
const TX_TRACE_SUCCESS: u32 = 1 << 8;
const TX_PUBLICATION_BISECT_STAGE: u8 = env!("XR819_TX_BISECT_STAGE").as_bytes()[0] - b'0';
const TX_PUBLICATION_BISECT_SUBTYPE: u8 = parse_decimal_u8(env!("XR819_TX_BISECT_SUBTYPE"));
const DATA_DIAGNOSTIC_LENGTH: usize =
    parse_decimal_u16(env!("XR819_DATA_DIAGNOSTIC_LENGTH")) as usize;
#[cfg(target_arch = "arm")]
const MAC_FATAL_MAGIC: u32 = 0x5852_4651;

const fn parse_decimal_u8(value: &str) -> u8 {
    parse_decimal_u16(value) as u8
}

const fn parse_decimal_u16(value: &str) -> u16 {
    let bytes = value.as_bytes();
    let mut result = 0_u16;
    let mut index = 0;
    while index < bytes.len() {
        result = result * 10 + (bytes[index] - b'0') as u16;
        index += 1;
    }
    result
}

const fn publication_bisect_matches(configured: u8, reached: u8) -> bool {
    configured != 0 && configured == reached
}

struct SharedPublicationBisectStage(UnsafeCell<u8>);

unsafe impl Sync for SharedPublicationBisectStage {}

static ACTIVE_PUBLICATION_BISECT_STAGE: SharedPublicationBisectStage =
    SharedPublicationBisectStage(UnsafeCell::new(0));

fn select_publication_bisect(frame: &[u8]) {
    let frame_control = frame
        .get(..2)
        .map(|value| u16::from_le_bytes([value[0], value[1]]))
        .unwrap_or(0);
    let subtype = (frame_control >> 4) as u8 & 0x0f;
    let protected_data = frame_control & 0x400c == 0x4008;
    let stage = if TX_PUBLICATION_BISECT_SUBTYPE == 0xff
        || (TX_PUBLICATION_BISECT_SUBTYPE == 0xfe && subtype != 11)
        || (TX_PUBLICATION_BISECT_SUBTYPE == 0xfd && protected_data)
        || TX_PUBLICATION_BISECT_SUBTYPE == subtype
    {
        TX_PUBLICATION_BISECT_STAGE
    } else {
        0
    };
    unsafe { *ACTIVE_PUBLICATION_BISECT_STAGE.0.get() = stage };
}

fn publication_bisect_reached(reached: u8) -> bool {
    let configured = unsafe { *ACTIVE_PUBLICATION_BISECT_STAGE.0.get() };
    publication_bisect_matches(configured, reached)
}
const MAC_BEACON_TIMER: u32 = crate::platform::mac_register(0x0e00) as u32;

unsafe fn read_u8(address: usize) -> u8 {
    unsafe { (address as *const u8).read_volatile() }
}

unsafe fn read_u16(address: usize) -> u16 {
    unsafe { (address as *const u16).read_volatile() }
}

unsafe fn read_u32(address: usize) -> u32 {
    unsafe { (address as *const u32).read_volatile() }
}

unsafe fn write_u8(address: usize, value: u8) {
    unsafe { (address as *mut u8).write_volatile(value) }
}

unsafe fn write_u16(address: usize, value: u16) {
    unsafe { (address as *mut u16).write_volatile(value) }
}

unsafe fn write_u32(address: usize, value: u32) {
    unsafe { (address as *mut u32).write_volatile(value) }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ContextAddress(u32);

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrameNodeAddress(u32);

impl ContextAddress {
    pub const fn new(address: u32) -> Self {
        Self(address)
    }

    pub const fn from_raw(address: u32) -> Option<Self> {
        if crate::dtcm::host_context_from_raw(address).is_some()
            || crate::dtcm::InternalContextAddress::from_raw(address).is_some()
        {
            Some(Self(address))
        } else {
            None
        }
    }

    pub const fn raw(self) -> u32 {
        self.0
    }

    pub const fn frame_node(self) -> FrameNodeAddress {
        FrameNodeAddress(self.0.wrapping_add(FRAME_NODE_OFFSET))
    }

    #[inline(always)]
    fn host(self) -> Option<crate::dtcm::HostContextAddress> {
        crate::dtcm::host_context_from_raw(self.0)
    }

    #[inline(always)]
    fn field_address(
        self,
        host_field: impl FnOnce(crate::dtcm::HostContextAddress) -> crate::dtcm::DtcmAddress,
        internal_field: impl FnOnce(crate::dtcm::InternalContextAddress) -> crate::dtcm::DtcmAddress,
    ) -> usize {
        if let Some(host) = self.host() {
            host_field(host).get()
        } else {
            internal_field(crate::dtcm::InternalContextAddress::from_non_host_raw(self.0)).get()
        }
    }

    fn intrusive_next_address(self) -> usize { self.field_address(|c| c.intrusive_next(), |c| c.intrusive_next()) }
    fn requested_rate_address(self) -> usize { self.field_address(|c| c.requested_rate(), |c| c.requested_rate()) }
    fn queue_id_address(self) -> usize { self.field_address(|c| c.queue_id(), |c| c.queue_id()) }
    fn request_flags_address(self) -> usize { self.field_address(|c| c.request_flags(), |c| c.request_flags()) }
    fn borrowed_frame_address_address(self) -> usize { self.field_address(|c| c.borrowed_frame_address(), |c| c.borrowed_frame_address()) }
    fn completion_status_address(self) -> usize { self.field_address(|c| c.completion_status(), |c| c.completion_status()) }
    fn saved_status_address(self) -> usize { self.field_address(|c| c.saved_status(), |c| c.saved_status()) }
    fn completion_flags_address(self) -> usize { self.field_address(|c| c.completion_flags(), |c| c.completion_flags()) }
    fn header_length_address(self) -> usize { self.field_address(|c| c.header_length(), |c| c.header_length()) }
    fn payload_length_address(self) -> usize { self.field_address(|c| c.payload_length(), |c| c.payload_length()) }
    fn optional_pipe_object_address(self) -> usize { self.field_address(|c| c.optional_pipe_object(), |c| c.optional_pipe_object()) }
    fn sequence_or_callback_state_address(self) -> usize { self.field_address(|c| c.sequence_or_callback_state(), |c| c.sequence_or_callback_state()) }
    fn submit_state_address(self) -> usize { self.field_address(|c| c.submit_state(), |c| c.submit_state()) }
    fn completion_class_address(self) -> usize { self.field_address(|c| c.completion_class(), |c| c.completion_class()) }
    fn frame_address_address(self) -> usize { self.field_address(|c| c.frame_address(), |c| c.frame_address()) }
    fn control_bits_address(self) -> usize { self.field_address(|c| c.control_bits(), |c| c.control_bits()) }
    fn frame_length_address(self) -> usize { self.field_address(|c| c.frame_length(), |c| c.frame_length()) }
    fn frame_control_address(self) -> usize { self.field_address(|c| c.frame_control(), |c| c.frame_control()) }
    fn access_category_address(self) -> usize { self.field_address(|c| c.access_category(), |c| c.access_category()) }
    fn request_flag_rate_bits_address(self) -> usize { self.field_address(|c| c.request_flag_rate_bits(), |c| c.request_flag_rate_bits()) }
    fn retry_policy_address(self) -> usize { self.field_address(|c| c.retry_policy(), |c| c.retry_policy()) }
    fn tx_rate_address(self) -> usize { self.field_address(|c| c.tx_rate(), |c| c.tx_rate()) }
    fn expiry_time_address(self) -> usize { self.field_address(|c| c.pas_expiry_time(), |c| c.expiry_time()) }
    fn completion_timestamp_address(self) -> usize { self.field_address(|c| c.completion_timestamp(), |c| c.completion_timestamp()) }
    fn scheduler_timestamp_address(self) -> usize { self.field_address(|c| c.scheduler_timestamp(), |c| c.scheduler_timestamp()) }
    fn terminal_status_address(self) -> usize { self.field_address(|c| c.terminal_status(), |c| c.terminal_status()) }
    fn try_count_address(self) -> usize { self.field_address(|c| c.try_count(), |c| c.try_count()) }
    fn ownership_bits_address(self) -> usize { self.field_address(|c| c.ownership_bits(), |c| c.ownership_bits()) }
    fn timing_reset_32_address(self) -> usize { self.field_address(|c| c.timing_reset_32(), |c| c.timing_reset_32()) }
    fn timing_reset_34_address(self) -> usize { self.field_address(|c| c.timing_reset_34(), |c| c.timing_reset_34()) }
    fn duration_address(self) -> usize { self.field_address(|c| c.duration(), |c| c.duration()) }
    fn payload_extended_address(self) -> usize { self.field_address(|c| c.payload_extended(), |c| c.payload_extended()) }
    fn payload_base_address(self) -> usize { self.field_address(|c| c.payload_base(), |c| c.payload_base()) }
    fn next_in_ampdu_address(self) -> usize { self.field_address(|c| c.next_in_ampdu(), |c| c.next_in_ampdu()) }
    fn word_48_address(self) -> usize { self.field_address(|c| c.word_48(), |c| c.word_48()) }
    fn frame_state_address_address(self) -> usize { self.field_address(|c| c.frame_state_address(), |c| c.frame_state_address()) }
    fn auxiliary_state_address(self) -> usize { self.field_address(|c| c.auxiliary_state(), |c| c.auxiliary_state()) }
    fn tid_address(self) -> usize { self.field_address(|c| c.tid(), |c| c.tid()) }
    fn insertion_mode_address(self) -> usize { self.field_address(|c| c.insertion_mode(), |c| c.insertion_mode()) }
    fn sequence_number_address(self) -> usize { self.field_address(|c| c.sequence_number(), |c| c.sequence_number()) }
    fn retry_rate_address(self) -> usize { self.field_address(|c| c.retry_rate(), |c| c.retry_rate()) }
    fn byte_57_address(self) -> usize { self.field_address(|c| c.byte_57(), |c| c.byte_57()) }
    fn retry_random_address(self) -> usize { self.field_address(|c| c.retry_random(), |c| c.retry_random()) }
    fn interface_address(self) -> usize { self.field_address(|c| c.interface(), |c| c.interface()) }
    fn duration_slot_address(self) -> usize { self.field_address(|c| c.duration_slot(), |c| c.duration_slot()) }
    fn host_link_address(self) -> usize { self.field_address(|c| c.host_link(), |c| c.host_link()) }
    fn link_id_address(self) -> usize { self.field_address(|c| c.link_id(), |c| c.link_id()) }
    fn qos_control_address(self) -> usize { self.field_address(|c| c.qos_control(), |c| c.qos_control()) }
    fn cipher_class_address(self) -> usize { self.field_address(|c| c.cipher_class(), |c| c.cipher_class()) }
    fn word_7c_address(self) -> usize { self.field_address(|c| c.word_7c(), |c| c.word_7c()) }
}

impl FrameNodeAddress {
    pub const fn new(address: u32) -> Self {
        Self(address)
    }

    pub const fn from_raw(address: u32) -> Option<Self> {
        let context = address.wrapping_sub(FRAME_NODE_OFFSET);
        match ContextAddress::from_raw(context) {
            Some(context) if context.frame_node().raw() == address => Some(Self(address)),
            _ => None,
        }
    }

    pub const fn raw(self) -> u32 {
        self.0
    }

    pub const fn context(self) -> ContextAddress {
        ContextAddress(self.0.wrapping_sub(FRAME_NODE_OFFSET))
    }

    fn frame_address(self) -> u32 { self.context().frame_address_address() as u32 }
    fn control_bits(self) -> u32 { self.context().control_bits_address() as u32 }
    fn frame_length(self) -> u32 { self.context().frame_length_address() as u32 }
    fn frame_control(self) -> u32 { self.context().frame_control_address() as u32 }
    fn access_category(self) -> u32 { self.context().access_category_address() as u32 }
    fn request_flag_rate_bits(self) -> u32 { self.context().request_flag_rate_bits_address() as u32 }
    fn tx_rate(self) -> u32 { self.context().tx_rate_address() as u32 }
    fn scheduler_timestamp(self) -> u32 { self.context().scheduler_timestamp_address() as u32 }
    fn ownership_bits(self) -> u32 { self.context().ownership_bits_address() as u32 }
    fn timing_reset_32(self) -> u32 { self.context().timing_reset_32_address() as u32 }
    fn timing_reset_34(self) -> u32 { self.context().timing_reset_34_address() as u32 }
    fn duration(self) -> u32 { self.context().duration_address() as u32 }
    fn payload_extended(self) -> u32 { self.context().payload_extended_address() as u32 }
    fn payload_base(self) -> u32 { self.context().payload_base_address() as u32 }
    fn next_in_ampdu(self) -> u32 { self.context().next_in_ampdu_address() as u32 }
    fn total_airtime(self) -> u32 { self.context().word_48_address() as u32 }
    fn auxiliary_state(self) -> u32 { self.context().auxiliary_state_address() as u32 }
    fn frame_kind(self) -> u32 { self.context().retry_rate_address() as u32 }
    fn retry_random(self) -> u32 { self.context().retry_random_address() as u32 }
    fn interface(self) -> u32 { self.context().interface_address() as u32 }
    fn duration_slot(self) -> u32 { self.context().duration_slot_address() as u32 }
}

/// A class-0 completion tied to the exact published MAC slot that owned it.
///
/// Context pointers alone are insufficient in a batch: the completion drain
/// may return several contexts while another slot in the same pipe remains
/// live. The publication registry records the pipe/slot/frame-node identity
/// before GO, and the completion drain consumes that identity only after
/// `complete_tx_pipe_slot` has enqueued the corresponding frame node.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HostClass0Completion {
    pub context: u32,
    pub frame_node: u32,
    pub pipe: u8,
    pub slot: u8,
    pub status: u16,
    pub ack_failures: u8,
}

/// One service pass can retire the configured ordinary batch depth on each of
/// four MAC pipes before the host driver starts draining copied identities.
#[cfg(feature = "experimental-four-slot-ordinary")]
const HOST_CLASS0_COMPLETION_CAPACITY: usize =
    4 * crate::host_tx_policy::MAX_ORDINARY_BATCH_DEPTH;
#[cfg(not(feature = "experimental-four-slot-ordinary"))]
const HOST_CLASS0_COMPLETION_CAPACITY: usize = 4 * 2;

struct BoundedCompletionQueue<T, const N: usize> {
    entries: [Option<T>; N],
}

impl<T, const N: usize> BoundedCompletionQueue<T, N> {
    const fn new() -> Self {
        Self {
            entries: [const { None }; N],
        }
    }

    fn push(&mut self, value: T) -> Result<(), T> {
        let Some(entry) = self.entries.iter_mut().find(|entry| entry.is_none()) else {
            return Err(value);
        };
        *entry = Some(value);
        Ok(())
    }

    fn take(&mut self) -> Option<T> {
        self.entries.iter_mut().find_map(Option::take)
    }

    fn first(&self) -> Option<&T> {
        self.entries.iter().find_map(Option::as_ref)
    }
}

#[derive(Clone, Copy)]
struct PublishedSlotIdentity {
    context: ContextAddress,
    frame_node: FrameNodeAddress,
    command: u32,
    pipe: u8,
    slot: u8,
}

#[derive(Clone, Copy)]
struct LiveTxSlot {
    address: crate::dtcm::MacPipeSlotAddress,
    frame_node: FrameNodeAddress,
    command: u32,
}

impl LiveTxSlot {
    fn from_mmio<M: MacPipeMmio>(
        mmio: &mut M,
        pipe: u8,
        slot: u8,
        retained_slot: u32,
        retained_frame: Option<FrameNodeAddress>,
    ) -> Option<Self> {
        if pipe >= 4 || slot >= 4 {
            return None;
        }
        let address = pipe_record_address(pipe).slot_unchecked(usize::from(slot));
        if retained_slot != address.raw() {
            return None;
        }
        let frame_node = FrameNodeAddress::from_raw(mmio.read_u32(address.frame().get() as u32))?;
        if retained_frame.is_some_and(|retained| retained != frame_node) {
            return None;
        }
        let command = mmio.read_u32(address.command().get() as u32);
        (command
            == packet_ram::tx_command(usize::from(pipe), usize::from(slot)) as u32)
        .then_some(Self {
            address,
            frame_node,
            command,
        })
    }

    unsafe fn from_pipe_slot(pipe: u8, slot: u8) -> Option<Self> {
        let retained_slot = (pipe < 4 && slot < 4)
            .then(|| pipe_record_address(pipe).slot_unchecked(usize::from(slot)).raw())?;
        Self::from_mmio(
            &mut VolatileMacPipeMmio,
            pipe,
            slot,
            retained_slot,
            None,
        )
    }
}

#[derive(Clone, Copy)]
enum RetirableTxSlot {
    Empty(crate::dtcm::MacPipeSlotAddress),
    Live(LiveTxSlot),
}

impl RetirableTxSlot {
    unsafe fn from_pipe_slot(pipe: u8, slot: u8) -> Option<Self> {
        if pipe >= 4 || slot >= 4 {
            return None;
        }
        unsafe {
            let address = pipe_record_address(pipe).slot_unchecked(usize::from(slot));
            let frame_node = read_u32(address.frame().get());
            if frame_node == 0 {
                return empty_retirement_allowed(
                    frame_node,
                    read_u8(address.kind().get()),
                    read_u8(address.state().get()),
                )
                .then_some(Self::Empty(address));
            }
            LiveTxSlot::from_pipe_slot(pipe, slot).map(Self::Live)
        }
    }
}

impl PublishedSlotIdentity {
    fn new(context: ContextAddress, pipe: u8, slot: u8) -> Option<Self> {
        if pipe >= 4 || slot >= 4 || ContextAddress::from_raw(context.raw()) != Some(context) {
            return None;
        }
        Some(Self {
            context,
            frame_node: context.frame_node(),
            command: packet_ram::tx_command(usize::from(pipe), usize::from(slot)) as u32,
            pipe,
            slot,
        })
    }

    #[cfg(target_arch = "arm")]
    unsafe fn live_slot(self) -> Option<crate::dtcm::MacPipeSlotAddress> {
        unsafe {
            let slot = pipe_record_address(self.pipe).slot_unchecked(usize::from(self.slot));
            (read_u32(slot.frame().get()) == self.frame_node.raw()
                && read_u32(slot.command().get()) == self.command
                && packet_ram::tx_command_index(self.command as usize)
                    == Some((usize::from(self.pipe), usize::from(self.slot))))
            .then_some(slot)
        }
    }
}

struct ProbeChecksum(UnsafeCell<u32>);

unsafe impl Sync for ProbeChecksum {}

static PROBE_CHECKSUM: ProbeChecksum = ProbeChecksum(UnsafeCell::new(0));

#[derive(Clone, Copy)]
struct TxDebugSnapshot {
    values: [u32; 8],
    next: u8,
    valid: bool,
}

struct SharedTxDebugSnapshot(UnsafeCell<TxDebugSnapshot>);

unsafe impl Sync for SharedTxDebugSnapshot {}

static TX_DEBUG_SNAPSHOT: SharedTxDebugSnapshot =
    SharedTxDebugSnapshot(UnsafeCell::new(TxDebugSnapshot {
        values: [0; 8],
        next: 0,
        valid: false,
    }));

struct SharedTxExecTrace(UnsafeCell<[u32; 12]>);

unsafe impl Sync for SharedTxExecTrace {}

static TX_EXEC_TRACE: SharedTxExecTrace = SharedTxExecTrace(UnsafeCell::new([0; 12]));

pub fn take_tx_debug_event() -> Option<(u32, u32)> {
    let snapshot = unsafe { &mut *TX_DEBUG_SNAPSHOT.0.get() };
    if !snapshot.valid {
        return None;
    }
    let index = usize::from(snapshot.next);
    let value = snapshot.values[index];
    snapshot.next = snapshot.next.wrapping_add(1);
    if usize::from(snapshot.next) == snapshot.values.len() {
        snapshot.valid = false;
        snapshot.next = 0;
    }
    Some((0x5852_0000 | index as u32, value))
}

unsafe fn copy_to_packet_ram(destination: u32, source: &[u8]) {
    let mut offset = 0;
    while offset + 4 <= source.len() {
        let word = u32::from_le_bytes([
            source[offset],
            source[offset + 1],
            source[offset + 2],
            source[offset + 3],
        ]);
        unsafe { ((destination as usize + offset) as *mut u32).write_volatile(word) };
        offset += 4;
    }
    if offset < source.len() {
        let pointer = (destination as usize + offset) as *mut u32;
        let mut bytes = unsafe { pointer.read_volatile() }.to_le_bytes();
        let remaining = source.len() - offset;
        bytes[..remaining].copy_from_slice(&source[offset..]);
        unsafe { pointer.write_volatile(u32::from_le_bytes(bytes)) };
    }
}

unsafe fn packet_ram_matches(destination: u32, source: &[u8]) -> bool {
    let mut offset = 0;
    while offset + 4 <= source.len() {
        let observed = unsafe {
            ((destination as usize + offset) as *const u32)
                .read_volatile()
                .to_le_bytes()
        };
        if observed != source[offset..offset + 4] {
            return false;
        }
        offset += 4;
    }
    if offset < source.len() {
        let observed = unsafe {
            ((destination as usize + offset) as *const u32)
                .read_volatile()
                .to_le_bytes()
        };
        if observed[..source.len() - offset] != source[offset..] {
            return false;
        }
    }
    true
}

unsafe fn release_context_address(context: u32) {
    unsafe {
        let address = crate::dtcm::InternalContextAddress::from_raw(context)
            .unwrap_or_else(|| unreachable!());
        crate::dtcm::shared_ptr::<u16>(address.terminal_status()).write_volatile(0x00ff);
        let flags = crate::dtcm::shared_ptr::<u32>(address.ownership_bits());
        flags.write_volatile(flags.read_volatile() | 0x0002_0000);
        let free_head = internal_context_free_head();
        let old_head = free_head.read_volatile();
        crate::dtcm::shared_ptr::<u32>(address.intrusive_next()).write_volatile(old_head);
        free_head.write_volatile(address.raw());
        set_active_internal_contexts(active_internal_contexts().wrapping_sub(1));
    }
}

#[cfg(target_arch = "arm")]
unsafe fn release_wsm_context_address(context: crate::dtcm::HostContextAddress) {
    unsafe {
        let header = crate::dtcm::shared_ptr::<u32>(context.borrowed_frame_address()).read_volatile();
        let backing = (0..TX_CONTEXT_COUNT)
            .map(internal_context_address)
            .find(|candidate| expected_header_address(candidate.raw()) == Some(header));
        crate::vendor_host_tx::free_host_context(context);
        if let Some(backing) = backing {
            release_context_address(backing.raw());
        }
    }
}

#[cfg(not(target_arch = "arm"))]
unsafe fn release_wsm_context_address(_context: crate::dtcm::HostContextAddress) {
    unsafe { core::arch::asm!("") };
}

fn expected_header_address(context: u32) -> Option<u32> {
    crate::dtcm::InternalContextAddress::from_raw(context)
        .map(|address| (packet_ram::internal_tx_buffer(address.index()) + 0x40) as u32)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProbeBuildError {
    MissingTemplate,
    WrongTemplateType,
    MalformedTemplate,
    SsidTooLong,
    FrameTooLarge,
    ContextPoolEmpty,
    InvalidInterface,
    InvalidContextPointer,
    PacketRamMismatch,
    PipeStateUnavailable,
    PipeSlotBusy,
    PipeSlotOwnershipMismatch,
    DescriptorReadbackMismatch,
    UnsupportedPublicationShape,
    CryptoFailure,
}

pub struct PreparedProbe {
    bytes: [u8; MAX_TEMPLATE_FRAME_LEN],
    length: usize,
    rate: u8,
}

struct PreparedProbeScratch(UnsafeCell<PreparedProbe>);

unsafe impl Sync for PreparedProbeScratch {}

static PREPARED_PROBE_SCRATCH: PreparedProbeScratch =
    PreparedProbeScratch(UnsafeCell::new(PreparedProbe {
        bytes: [0; MAX_TEMPLATE_FRAME_LEN],
        length: 0,
        rate: 0,
    }));

static TRANSFORMED_HOST_SCRATCH: PreparedProbeScratch =
    PreparedProbeScratch(UnsafeCell::new(PreparedProbe {
        bytes: [0; MAX_TEMPLATE_FRAME_LEN],
        length: 0,
        rate: 0,
    }));

static LAST_EAPOL_SCRATCH: PreparedProbeScratch =
    PreparedProbeScratch(UnsafeCell::new(PreparedProbe {
        bytes: [0; MAX_TEMPLATE_FRAME_LEN],
        length: 0,
        rate: 0,
    }));

#[derive(Clone, Copy)]
struct RetainedEapolMetadata {
    queue_id: u8,
    more: bool,
    flags: u8,
    expire_time: u32,
    ht_tx_parameters: u32,
}

struct SharedRetainedEapolMetadata(UnsafeCell<RetainedEapolMetadata>);

unsafe impl Sync for SharedRetainedEapolMetadata {}

static LAST_EAPOL_METADATA: SharedRetainedEapolMetadata =
    SharedRetainedEapolMetadata(UnsafeCell::new(RetainedEapolMetadata {
        queue_id: 0,
        more: false,
        flags: 0,
        expire_time: 0,
        ht_tx_parameters: 0,
    }));

struct SharedReplayDataPathGuard(UnsafeCell<bool>);

unsafe impl Sync for SharedReplayDataPathGuard {}

static REPLAY_DATA_PATH_GUARD: SharedReplayDataPathGuard =
    SharedReplayDataPathGuard(UnsafeCell::new(false));

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PhyRateWords {
    pub control: u32,
    pub rate: u32,
}

/// Pure arithmetic from vendor `pas_build_phy_rate_words` (`0x834e`).
pub fn build_phy_rate_words(
    rate_index: u8,
    legacy_mode: u8,
    tx_flags: u32,
    request_flag_rate_bits: u8,
    rate_attribute: u8,
) -> PhyRateWords {
    let mut control = 2;
    let class = if rate_index < 4 {
        if legacy_mode == 0 || rate_index == 0 || (legacy_mode == 2 && rate_index == 1) {
            0x0400
        } else {
            0
        }
    } else if rate_index < 0x0e {
        0x0800
    } else {
        if tx_flags & 0x20 != 0 {
            control = 6;
        }
        if tx_flags & 0x08 != 0 { 0x1400 } else { 0x1000 }
    };
    PhyRateWords {
        control,
        rate: class
            | u32::from(rate_attribute & 0x0f)
            | (u32::from(request_flag_rate_bits & 7) << 16),
    }
}

/// Applies the HT mixed-mode duration field added by vendor
/// `txp_submit_to_pipe` (`0xadd0`) after building the base PHY words.
fn finalize_phy_control(phy: PhyRateWords, rate_index: u8, frame_length: u16) -> u32 {
    if (phy.rate & 0x1fff) >> 10 == 5 {
        phy.control
            | (u32::from(crate::mac::ofdm_duration(
                rate_index,
                frame_length.wrapping_add(4),
            )) << 12)
    } else {
        phy.control
    }
}

fn single_frame_secondary_command(tx_flags: u32, header_duration: u16, duration_slot: u8) -> u32 {
    if tx_flags & 1 == 0 {
        0x2100_0000
            | unsafe {
                packet_ram::mac_packet_offset_unchecked(packet_ram::duration_word(usize::from(
                    duration_slot,
                )))
            }
    } else {
        0x3200_0000 | header_duration as u32
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SingleFramePasTiming {
    pub payload_extended: u16,
    pub payload_base: u16,
    pub ack: u16,
    pub total_airtime: u32,
    pub frame_kind: u8,
}

/// Single legacy-frame subset of vendor `pas_compute_tx_timing` (`0x7fa6`).
/// Protection/preamble modes are deliberately rejected by the caller; probes
/// and ordinary host management frames both use the direct payload branch.
pub fn compute_single_frame_pas_timing(
    phy_config: u16,
    rate: u8,
    frame_length: u16,
    flags: u32,
    ack_duration: u16,
    special_peer: bool,
) -> Option<SingleFramePasTiming> {
    if flags & 0x0c00 != 0 {
        return None;
    }
    let stream = flags & 8 != 0;
    let payload_length = frame_length.wrapping_add(4);
    let payload_base = crate::mac::base_airtime(phy_config, rate, payload_length, stream);
    let payload_extended = crate::mac::extended_airtime(phy_config, rate, payload_length, stream);
    let (ack, frame_kind) = if flags & 0x0200 != 0 {
        (0, 0xff)
    } else if flags & 0x4000 != 0 {
        (ack_duration, 0x0c)
    } else {
        (ack_duration, if special_peer { 0x0e } else { 0x11 })
    };
    Some(SingleFramePasTiming {
        payload_extended,
        payload_base,
        ack,
        total_airtime: u32::from(payload_base) + u32::from(ack),
        frame_kind,
    })
}

pub const fn single_frame_slot_duration(timing: SingleFramePasTiming) -> u32 {
    (timing.payload_base as u32).wrapping_mul(0x8000)
        + if timing.frame_kind == 0xff {
            0
        } else {
            0x2000 + timing.ack as u32
        }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SingleFramePipeInput {
    pub phy_rate_word: u32,
    pub phy_control_word: u32,
    pub frame_length: u16,
    pub hardware_rate: u8,
    pub frame_control: u16,
    pub retry_flag: bool,
    pub metadata_address: packet_ram::RuntimePacketAddress,
    pub duration: u16,
    pub header_address: packet_ram::RuntimePacketAddress,
    pub secondary_command: u32,
    pub address_mask: u32,
    pub terminal_command: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SingleFramePipeDescriptor {
    words: [u32; 13],
    length: u8,
}

impl SingleFramePipeDescriptor {
    pub fn words(&self) -> &[u32] {
        &self.words[..usize::from(self.length)]
    }
}

/// Exact non-control, non-aggregate branch of vendor
/// `txp_submit_to_pipe` (`0xadd0`). Unknown policy-derived values are explicit
/// inputs so descriptor publication cannot silently invent them.
pub fn build_single_frame_pipe_descriptor(
    input: SingleFramePipeInput,
) -> Result<SingleFramePipeDescriptor, ProbeBuildError> {
    let header = validated_tx_frame(input.header_address.raw(), input.frame_length)
        .ok_or(ProbeBuildError::PacketRamMismatch)?;
    if packet_ram::RuntimePacketAddress::new(input.metadata_address.raw(), 1).is_none() {
        return Err(ProbeBuildError::PacketRamMismatch);
    }
    let mut words = [0_u32; 13];
    let frame_control = u32::from(input.frame_control) | if input.retry_flag { 0x0800 } else { 0 };
    words[0] = 0x5100_0000 | (input.phy_rate_word & 0x00ff_ffff);
    words[1] = 0x5000_0000 | (input.phy_control_word & 0x00ff_ffff);
    words[2] = 0x5200_0000
        | (u32::from(input.hardware_rate) << 16)
        | u32::from(input.frame_length.wrapping_add(4));
    words[3] = 0x3100_0000 + frame_control;
    words[4] = 0x4700_0000 + (frame_control >> 8);
    words[5] = 0x2080_0000 | input.metadata_address.mac_offset().raw();
    words[6] = 0x3200_0000 | u32::from(input.duration);
    words[7] =
        0x2900_0000 | packet_ram::encode_mac_packet_offset_u32(header.descriptor_tail());
    words[8] = input.secondary_command;
    let mut length = 9;
    if input.frame_length > DOT11_FIXED_HEADER_LENGTH {
        let payload = packet_ram::RuntimePacketAddress::new(
            header.payload_after_fixed_header(),
            usize::from(input.frame_length - DOT11_FIXED_HEADER_LENGTH),
        )
        .ok_or(ProbeBuildError::PacketRamMismatch)?;
        words[9] = 0x4000_0000 | payload.tx_payload_bus_address(input.address_mask).raw();
        words[10] = (u32::from(input.frame_length - DOT11_FIXED_HEADER_LENGTH) & 0x0fff) << 12
            | (payload.raw() & 3);
        length = 11;
    }
    words[length] = input.terminal_command;
    words[length + 1] = 0xf000_0000;
    Ok(SingleFramePipeDescriptor {
        words,
        length: (length + 2) as u8,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DepthTwoAmpduInput {
    pub first_frame_state: u32,
    pub second_frame_state: u32,
    pub first_frame_length: u16,
    pub second_frame_length: u16,
    pub phy_rate_word: u32,
    pub phy_control_word: u32,
    pub hardware_rate: u8,
    /// Vendor spacing selector used by opcode 4. Zero emits no spacing word.
    pub spacing_selector: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlockAckMemberState {
    Acknowledged,
    Missing,
    OutsideWindow,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlockAckMemberAction {
    Confirm,
    Retry,
    GiveUp,
}

pub fn classify_depth_two_block_ack(
    start_sequence: u16,
    bitmap: u64,
    sequences: [u16; 2],
) -> [BlockAckMemberState; 2] {
    sequences.map(|sequence| {
        let delta = (sequence & 0x0fff).wrapping_sub(start_sequence & 0x0fff) & 0x0fff;
        if delta >= 64 {
            BlockAckMemberState::OutsideWindow
        } else if bitmap & (1_u64 << delta) != 0 {
            BlockAckMemberState::Acknowledged
        } else {
            BlockAckMemberState::Missing
        }
    })
}

pub fn merge_depth_two_block_ack_states(
    previous: Option<[BlockAckMemberState; 2]>,
    current: [BlockAckMemberState; 2],
) -> [BlockAckMemberState; 2] {
    core::array::from_fn(|index| match (previous.map(|states| states[index]), current[index]) {
        (Some(BlockAckMemberState::Acknowledged), _)
        | (_, BlockAckMemberState::Acknowledged) => BlockAckMemberState::Acknowledged,
        (Some(BlockAckMemberState::Missing), _)
        | (_, BlockAckMemberState::Missing) => BlockAckMemberState::Missing,
        _ => BlockAckMemberState::OutsideWindow,
    })
}

pub fn plan_depth_two_block_ack_actions(
    members: [BlockAckMemberState; 2],
    retry_allowed: [bool; 2],
    session_active: bool,
) -> [BlockAckMemberAction; 2] {
    core::array::from_fn(|index| match members[index] {
        BlockAckMemberState::Acknowledged => BlockAckMemberAction::Confirm,
        BlockAckMemberState::Missing if session_active && retry_allowed[index] => {
            BlockAckMemberAction::Retry
        }
        BlockAckMemberState::Missing | BlockAckMemberState::OutsideWindow => {
            BlockAckMemberAction::GiveUp
        }
    })
}

pub fn depth_two_whole_retry_allowed(
    retry_allowed: [bool; 2],
    session_active: bool,
    next_rates: [u8; 2],
) -> bool {
    plan_depth_two_block_ack_actions(
        [BlockAckMemberState::Missing; 2],
        retry_allowed,
        session_active,
    ) == [BlockAckMemberAction::Retry; 2]
        && next_rates[0] == next_rates[1]
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DepthTwoAmpduDescriptor {
    /// Auxiliary stream reached by the top-level opcode-0 transfer.
    pub words: [u32; 6],
    pub length: u8,
    /// Shared PHY words emitted at top-level command offsets `+0x0c..+0x14`.
    pub phy_words: [u32; 3],
    pub aggregate_length: u16,
}

fn ampdu_transfer_word(cpu_address: usize) -> u32 {
    #[cfg(target_arch = "arm")]
    unsafe {
        packet_ram::ampdu_transfer_word_unchecked(cpu_address)
    }
    #[cfg(not(target_arch = "arm"))]
    {
        packet_ram::ampdu_transfer_word(cpu_address)
            .expect("A-MPDU transfer requires an aligned CPU-form runtime packet-RAM address")
    }
}

const AMPDU_SPACING_SELECTORS: [[u8; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [1, 1, 1, 1, 1, 1, 1, 1],
    [1, 1, 1, 1, 1, 1, 1, 2],
    [1, 1, 1, 1, 2, 2, 2, 3],
    [1, 1, 2, 2, 3, 4, 4, 5],
    [1, 2, 3, 4, 5, 7, 8, 9],
    [2, 4, 5, 7, 10, 13, 15, 17],
    [4, 7, 10, 13, 20, 26, 30, 33],
];

pub(crate) const fn ampdu_spacing_selector(density: u8, rate: u8) -> u8 {
    if density < 8 && rate >= 14 && rate < 22 {
        AMPDU_SPACING_SELECTORS[density as usize][(rate - 14) as usize]
    } else {
        0
    }
}

fn ampdu_spacing_word(selector: u8) -> Option<u32> {
    if selector == 0 {
        None
    } else {
        Some(ampdu_transfer_word(packet_ram::ampdu_spacing_word_address(selector)))
    }
}

/// Build the vendor opcode stream for exactly two MPDUs.
///
/// This is the body reached through the slot command's initial opcode-0 jump;
/// reservation and publication own that outer word separately. The first
/// subframe includes delimiter/alignment overhead while the last contributes
/// its frame and FCS length directly.
pub fn build_depth_two_ampdu_descriptor(input: DepthTwoAmpduInput) -> DepthTwoAmpduDescriptor {
    let mut words = [0_u32; 6];
    let mut length = 0_usize;
    words[length] = ampdu_transfer_word(input.first_frame_state.wrapping_add(8) as usize);
    length += 1;
    words[length] = 0x6600_0000;
    length += 1;
    if let Some(spacing) = ampdu_spacing_word(input.spacing_selector) {
        words[length] = spacing;
        length += 1;
    }
    words[length] = ampdu_transfer_word(input.second_frame_state.wrapping_add(8) as usize);
    length += 1;
    words[length] = 0xe400_0000;
    length += 1;
    let aggregate_length = (u32::from(input.first_frame_length).wrapping_add(0x0b) & !3)
        .wrapping_add(u32::from(input.spacing_selector) * 4)
        .wrapping_add(u32::from(input.second_frame_length))
        .wrapping_add(8) as u16;
    let phy_words = [
        0x5100_0000 | (input.phy_rate_word & 0x00ff_ffff),
        0x5000_0000 | (input.phy_control_word & 0x00ff_ffff),
        0x5200_0000
            | (u32::from(input.hardware_rate) << 16)
            | u32::from(aggregate_length),
    ];
    DepthTwoAmpduDescriptor {
        words,
        length: length as u8,
        phy_words,
        aggregate_length,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MacEvent {
    pub raw: u32,
    pub event_type: u8,
    pub pipe: u8,
    pub phase: u8,
    pub status: u8,
    pub pipe_marker: bool,
    pub completion_marker: bool,
    pub fatal_marker: bool,
    pub pipe_service_marker: bool,
    pub beacon_marker: bool,
    pub sideband_marker: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MacEventEffect {
    Trace,
    Fatal,
    PipePhase {
        event_type: u8,
        phase: u8,
        latch_index: Option<u8>,
    },
    PipeService,
    TxStatus {
        event_type: u8,
        status: u8,
        pipe_service_escalation: bool,
    },
    Beacon,
    Sideband,
    Archive,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MacEventDispatchPlan {
    effects: [Option<MacEventEffect>; 8],
    len: u8,
}

/// Deterministic Rust postmortem record for a terminal MAC event. Vendor
/// assertion arguments for this path are `line = 222`, `code = 0x29`; the
/// additional words retain ownership-relevant state for reset-time recovery.
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MacFatalPostmortem {
    pub valid: u32,
    pub version: u32,
    pub line: u32,
    pub code: u32,
    pub event: u32,
    pub saved_scheduler_word: u32,
    pub cpsr: u32,
    pub current_pipe: u32,
    pub current_pipe_record: u32,
    pub current_slot: u32,
    pub pipe_busy: u32,
    pub event_readiness: u32,
    pub pipe_irq_pending: u32,
    pub pipe_irq_trigger: u32,
}

impl MacFatalPostmortem {
    pub const fn empty() -> Self {
        Self {
            valid: 0,
            version: 1,
            line: 222,
            code: 0x29,
            event: 0,
            saved_scheduler_word: 0,
            cpsr: 0,
            current_pipe: 0,
            current_pipe_record: 0,
            current_slot: 0,
            pipe_busy: 0,
            event_readiness: 0,
            pipe_irq_pending: 0,
            pipe_irq_trigger: 0,
        }
    }
}

struct MacFatalStorage(UnsafeCell<MacFatalPostmortem>);

// Fatal publication occurs after IRQ/FIQ masking and never returns. Reset-time
// readers must inspect `valid` before the remaining words.
unsafe impl Sync for MacFatalStorage {}

static MAC_FATAL_POSTMORTEM: MacFatalStorage =
    MacFatalStorage(UnsafeCell::new(MacFatalPostmortem::empty()));

pub fn capture_mac_fatal_postmortem<M: MacPipeMmio>(
    mmio: &mut M,
    event: MacEvent,
    saved_scheduler_word: SchedulerWord,
    cpsr: u32,
    output: &mut MacFatalPostmortem,
) {
    output.valid = 0;
    output.version = 1;
    output.line = 222;
    output.code = 0x29;
    output.event = event.raw;
    output.saved_scheduler_word = saved_scheduler_word.raw();
    output.cpsr = cpsr;
    output.current_pipe = u32::from(mmio.read_u8(crate::dtcm::MAC_CURRENT_PIPE.get() as u32));
    output.current_pipe_record = mmio.read_u32(crate::dtcm::MAC_CURRENT_PIPE_RECORD.get() as u32);
    output.current_slot = mmio.read_u32(crate::dtcm::MAC_CURRENT_SLOT.get() as u32);
    output.pipe_busy = u32::from(mmio.read_u8(crate::dtcm::LOW_MAC_PIPE_BUSY.get() as u32));
    output.event_readiness = mmio.read_u32(MAC_EVENT_READINESS);
    output.pipe_irq_pending = mmio.read_u32(PIPE_IRQ_PENDING);
    output.pipe_irq_trigger = mmio.read_u32(PIPE_IRQ_TRIGGER);
}

impl MacEventDispatchPlan {
    pub fn effects(&self) -> &[Option<MacEventEffect>] {
        &self.effects[..usize::from(self.len)]
    }

    fn push(&mut self, effect: MacEventEffect) {
        self.effects[usize::from(self.len)] = Some(effect);
        self.len += 1;
    }
}

impl MacEvent {
    /// Decodes the fields consumed by vendor `mac_irq_handler` (`0x9eb4`).
    /// Bit 31 is the event-FIFO empty sentinel.
    pub fn decode(raw: u32) -> Option<Self> {
        (raw & 0x8000_0000 == 0).then_some(Self {
            raw,
            event_type: ((raw & 0x3fff) >> 8) as u8,
            pipe: ((raw >> 18) & 3) as u8,
            phase: ((raw >> 16) & 3) as u8,
            status: (raw & 0x3f) as u8,
            pipe_marker: raw & (1 << 25) != 0,
            completion_marker: raw & (1 << 24) != 0,
            fatal_marker: raw & (1 << 30) != 0,
            pipe_service_marker: raw & (1 << 23) != 0,
            beacon_marker: raw & (1 << 26) != 0,
            sideband_marker: raw & (1 << 7) != 0,
        })
    }

    /// Index that vendor `0x9ee4..0x9ee8` is allowed to latch globally.
    pub fn pipe_index(self) -> Option<u8> {
        (self.pipe_marker && self.event_type == 0x37).then_some(self.pipe)
    }

    /// Status consumed by the bit-24 dispatch path. Low bits in other event
    /// classes are not terminal TX status.
    pub fn completion_status(self) -> Option<u8> {
        self.completion_marker.then_some(self.status)
    }

    pub fn is_pipe_start(self) -> bool {
        self.pipe_index().is_some() && self.phase == 2
    }

    pub fn is_pipe_success(self) -> bool {
        self.pipe_index().is_some() && self.phase == 3
    }

    /// Preserves the exact independent-marker order in vendor FIQ handler
    /// `0x9e90..0x9ff8`. Fatal assertion handling never returns, so no later
    /// marker or archive effect is reachable for a bit-30 event.
    pub fn dispatch_plan(self) -> MacEventDispatchPlan {
        let mut plan = MacEventDispatchPlan {
            effects: [None; 8],
            len: 0,
        };
        plan.push(MacEventEffect::Trace);
        if self.fatal_marker {
            plan.push(MacEventEffect::Fatal);
            return plan;
        }
        if self.pipe_marker {
            plan.push(MacEventEffect::PipePhase {
                event_type: self.event_type,
                phase: self.phase,
                latch_index: self.pipe_index(),
            });
        }
        if self.pipe_service_marker {
            plan.push(MacEventEffect::PipeService);
        }
        if let Some(status) = self.completion_status() {
            plan.push(MacEventEffect::TxStatus {
                event_type: self.event_type,
                status,
                pipe_service_escalation: self.pipe_service_marker,
            });
        }
        if self.beacon_marker {
            plan.push(MacEventEffect::Beacon);
        }
        if self.sideband_marker {
            plan.push(MacEventEffect::Sideband);
        }
        plan.push(MacEventEffect::Archive);
        plan
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProbeTxOwnership {
    Idle,
    Prepared {
        context: u32,
    },
    PipeOwned {
        context: u32,
        pipe: u8,
        slot: u8,
    },
    Started {
        context: u32,
        pipe: u8,
        slot: u8,
    },
    RetryRequired {
        context: u32,
        pipe: u8,
        slot: u8,
        status: u8,
    },
    TerminalObserved {
        context: u32,
        pipe: u8,
        slot: u8,
        status: u8,
    },
    CompletionQueued {
        context: u32,
        pipe: u8,
        slot: u8,
        status: u8,
    },
    CallbackRunning {
        context: u32,
        pipe: u8,
        slot: u8,
        status: u8,
    },
    FatalQuiesced {
        context: Option<u32>,
        pipe: Option<u8>,
        slot: Option<u8>,
    },
    Returned,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProbeTxIdentity {
    pub context: u32,
    pub pipe: Option<u8>,
    pub slot: Option<u8>,
    pub generation: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProbeTxTransitionError {
    Busy,
    NotPrepared,
    WrongPipe,
    CompletionBeforeStart,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProbeTxTracker {
    ownership: ProbeTxOwnership,
    latched_pipe: Option<u8>,
    observed_status: Option<u8>,
    generation: u32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MacEventServiceReport {
    pub drained: u32,
    pub handled: u32,
    pub unhandled: u32,
    pub blocked: Option<u32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MacEventStopReason {
    Empty,
    BudgetExhausted,
    Fatal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MacEventLoopReport {
    pub processed: u32,
    pub stop: MacEventStopReason,
    pub reschedule_required: bool,
}

/// One immutable snapshot of `0x09c00e84`, taken before bit-23 service.
/// Its value must be reused for the later bit-24 pending decision even though
/// the bit-23 acknowledgement can change the hardware register.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SchedulerWord(u32);

impl SchedulerWord {
    pub const fn new(raw: u32) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MacStatusSnapshot {
    pub pre_service_scheduler_word: SchedulerWord,
    pub latched_pipe: u8,
    pub pipe_active: bool,
    pub slot_expected_status: u8,
    pub slot_state: u8,
    pub global_busy: bool,
    pub mismatch_count: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MacStatusResolution {
    pub status: u8,
    pub pending_mask: u32,
    pub dispatch_ordinary: bool,
    pub ordinary_completion_eligible: bool,
    pub direct_retry: bool,
    pub next_mismatch_count: u8,
    pub escalation_retry: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MacPipeServicePlan {
    None,
    LowNibble {
        asserted: u8,
        backoff_pipe_mask: u8,
        trigger: u32,
        acknowledgement: u32,
    },
    MiddleNibble {
        selected_pipe: u8,
        quantum_pointer_address: u32,
        trigger: u32,
        acknowledgement: u32,
    },
    HighNibble {
        selected_pipe: u8,
        acknowledgement: u32,
    },
}

/// Pure selection/acknowledgement plan for `mac_pipe_irq_service`
/// (`0x9b6e..0x9c28`). Hardware backoff programming remains an executor leaf.
pub fn plan_mac_pipe_service(
    scheduler_word: SchedulerWord,
    latched_pipe: u8,
) -> MacPipeServicePlan {
    let scheduler_word = scheduler_word.raw();
    let low = (scheduler_word & 0x0f) as u8;
    if low != 0 {
        return MacPipeServicePlan::LowNibble {
            asserted: low,
            backoff_pipe_mask: low & 0x07,
            trigger: u32::from(low) << 25,
            acknowledgement: u32::from(low).wrapping_add(1).wrapping_neg(),
        };
    }
    let middle = ((scheduler_word >> 4) & 0x0f) as u8;
    if middle != 0 {
        let selected_pipe = middle.trailing_zeros() as u8;
        return MacPipeServicePlan::MiddleNibble {
            selected_pipe,
            quantum_pointer_address: crate::dtcm::duration_quantum_pointer_unchecked(
                usize::from(selected_pipe),
            )
            .get() as u32,
            trigger: 1_u32 << (u32::from(selected_pipe) + 25),
            acknowledgement: (0x10_u32 << selected_pipe).wrapping_add(1).wrapping_neg(),
        };
    }
    if scheduler_word & 0xf000 != 0 {
        let selected_pipe = latched_pipe & 3;
        return MacPipeServicePlan::HighNibble {
            selected_pipe,
            acknowledgement: (0x1000_u32 << selected_pipe).wrapping_add(1).wrapping_neg(),
        };
    }
    MacPipeServicePlan::None
}

/// Infallible MMIO access used by the inactive MAC-pipe service executor.
/// Keeping this boundary injectable makes the exact write order testable
/// without mapping XR819 hardware into a host test process.
pub trait MacPipeMmio {
    fn read_u8(&mut self, address: u32) -> u8;
    fn read_u16(&mut self, address: u32) -> u16;
    fn read_u32(&mut self, address: u32) -> u32;
    fn write_u8(&mut self, address: u32, value: u8);
    fn write_u16(&mut self, address: u32, value: u16);
    fn write_u32(&mut self, address: u32, value: u32);
}

/// Random backoff is policy-owned and is deliberately not guessed here.
/// Implementations are infallible because this executor is reached after the
/// MAC event has already been consumed.
pub trait TxPolicy {
    fn program_random_backoff(&mut self, pipe: u8, backoff_word: u32, pas: u32);
}

/// Captures the sole pre-service scheduler word for one event. Call this once,
/// before bit-23 service, and pass the returned value to both service planning
/// and bit-24 status resolution.
pub fn capture_pipe_scheduler_word<M: MacPipeMmio>(mmio: &mut M) -> SchedulerWord {
    SchedulerWord::new(mmio.read_u32(PIPE_IRQ_PENDING))
}

#[repr(C)]
struct TxHardwareRingLayout {
    duration_fifo: u32,
    opaque_04: [u8; 0x10],
    go: u32,
    inactive_sentinel: u32,
    completion_word: u32,
    cursor_and_pending_mask: u32,
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TxHardwareRingAddress(u32);

impl TxHardwareRingAddress {
    #[cfg(test)]
    const fn new(address: u32) -> Self { Self(address) }
    const fn for_pipe(pipe: u8, address: u32) -> Option<Self> {
        if pipe < 4
            && address
                == crate::platform::tx_ring_register(pipe as usize, 0) as u32
        {
            Some(Self(address))
        } else {
            None
        }
    }
    const fn raw(self) -> u32 { self.0 }
    const fn field(self, offset: usize) -> u32 { self.0.wrapping_add(offset as u32) }
    const fn duration_fifo(self) -> u32 { self.field(core::mem::offset_of!(TxHardwareRingLayout, duration_fifo)) }
    const fn diagnostic_word_0c(self) -> u32 { self.field(0x0c) }
    const fn diagnostic_word_10(self) -> u32 { self.field(0x10) }
    const fn go(self) -> u32 { self.field(core::mem::offset_of!(TxHardwareRingLayout, go)) }
    const fn inactive_sentinel(self) -> u32 { self.field(core::mem::offset_of!(TxHardwareRingLayout, inactive_sentinel)) }
    const fn completion_word(self) -> u32 { self.field(core::mem::offset_of!(TxHardwareRingLayout, completion_word)) }
    const fn cursor_and_pending_mask(self) -> u32 { self.field(core::mem::offset_of!(TxHardwareRingLayout, cursor_and_pending_mask)) }
}

fn retained_hardware_ring<M: MacPipeMmio>(
    mmio: &mut M,
    pipe: u8,
) -> Option<TxHardwareRingAddress> {
    if pipe >= 4 {
        return None;
    }
    let record = pipe_record_address(pipe);
    TxHardwareRingAddress::for_pipe(
        pipe,
        mmio.read_u32(record.hardware_ring().get() as u32),
    )
}

const DOT11_FIXED_HEADER_LENGTH: u16 = 24;

#[repr(C, packed)]
struct Dot11FixedHeaderLayout {
    frame_control: u16,
    duration: u16,
    address_1: [u8; 6],
    address_2: [u8; 6],
    address_3: [u8; 6],
    sequence_control: u16,
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TxFrameAddress(u32);

impl TxFrameAddress {
    const fn new(address: u32) -> Self { Self(address) }
    const fn raw(self) -> u32 { self.0 }
    const fn field(self, offset: usize) -> u32 { self.0.wrapping_add(offset as u32) }
    const fn frame_control(self) -> u32 { self.field(core::mem::offset_of!(Dot11FixedHeaderLayout, frame_control)) }
    const fn address_1(self) -> u32 { self.field(core::mem::offset_of!(Dot11FixedHeaderLayout, address_1)) }
    const fn address_1_halfword_unchecked(self, word: u32) -> u32 { self.address_1().wrapping_add(word * 2) }
    const fn address_2_byte_unchecked(self, byte: u32) -> u32 {
        self.field(core::mem::offset_of!(Dot11FixedHeaderLayout, address_2)).wrapping_add(byte)
    }
    const fn address_2_halfword_unchecked(self, word: u32) -> u32 {
        self.address_2_byte_unchecked(word * 2)
    }
    const fn sequence_control(self) -> u32 { self.field(core::mem::offset_of!(Dot11FixedHeaderLayout, sequence_control)) }
    const fn descriptor_tail(self) -> u32 { self.address_1() }
    const fn payload_after_fixed_header(self) -> u32 {
        self.0.wrapping_add(DOT11_FIXED_HEADER_LENGTH as u32)
    }
}

fn validated_tx_frame(address: u32, length: u16) -> Option<TxFrameAddress> {
    (length >= DOT11_FIXED_HEADER_LENGTH)
        .then(|| packet_ram::RuntimePacketAddress::new(address, usize::from(length)))
        .flatten()
        .map(|address| TxFrameAddress::new(address.raw()))
}

unsafe fn validated_context_tx_frame(context: ContextAddress) -> Option<TxFrameAddress> {
    unsafe {
        validated_tx_frame(
            read_u32(context.frame_address_address()),
            read_u16(context.frame_length_address()),
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Dot11HeaderShape {
    length: u32,
    qos_data: bool,
}

fn classify_dot11_header(frame_control: u16, legacy_eapol: bool) -> Dot11HeaderShape {
    let four_address = frame_control & 0x0300 == 0x0300;
    let qos_data = !legacy_eapol && frame_control & 0x008f == 0x0088;
    let base = if four_address { 30 } else { u32::from(DOT11_FIXED_HEADER_LENGTH) };
    let qos_extension = if qos_data {
        if frame_control & 0x8000 != 0 { 6 } else { 2 }
    } else {
        0
    };
    Dot11HeaderShape {
        length: base + qos_extension,
        qos_data,
    }
}

#[repr(C)]
struct TxDescriptorLayout {
    command: u32,
    flags: u32,
    duration: u32,
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TxDescriptorAddress(u32);

impl TxDescriptorAddress {
    const fn new(address: u32) -> Self { Self(address) }
    const fn raw(self) -> u32 { self.0 }
    const fn command(self) -> u32 {
        self.0.wrapping_add(core::mem::offset_of!(TxDescriptorLayout, command) as u32)
    }
    const fn flags(self) -> u32 {
        self.0.wrapping_add(core::mem::offset_of!(TxDescriptorLayout, flags) as u32)
    }
    const fn duration(self) -> u32 {
        self.0.wrapping_add(core::mem::offset_of!(TxDescriptorLayout, duration) as u32)
    }
    const fn word_unchecked(self, index: u32) -> u32 { self.0.wrapping_add(index * 4) }
}

fn pipe_record_address(pipe: u8) -> crate::dtcm::MacPipeRecordAddress {
    crate::dtcm::MacPipeRecordAddress::from_index_unchecked(usize::from(pipe & 3))
}

fn pipe_state_address(pipe: u8) -> u32 {
    pipe_record_address(pipe).raw()
}

/// Exact translation of vendor `txp_pipe_advance_slot` (`0xa9f2`).
///
/// Retires one pipe's hardware ring state: the inactive command sentinel at
/// `ring + 0x18`, the pipe acknowledgement, and — the part the packet
/// controller actually consumes — both cursor fields of `ring + 0x20`
/// (bits 26:24 and 29:27), which are set to the retired slot.
///
/// The caller must hold IRQ/FIQ off across the pipe-state read/modify, matching
/// the vendor `irq_fiq_disable_save()`/`irq_fiq_restore()` pair; the ring and
/// acknowledgement writes are outside that section in the vendor as well.
///
/// Returns true when the pipe was still armed, which selects the vendor
/// slot-record cleanup sweep in `txp_fn_4425` (`0x38c`).
pub fn advance_pipe_slot<M: MacPipeMmio>(mmio: &mut M, pipe: u8) -> Option<bool> {
    let ring = retained_hardware_ring(mmio, pipe)?;
    let record = pipe_record_address(pipe);
    let armed = mmio.read_u8(record.state().get() as u32) != 0;
    let cursor = if armed {
        mmio.write_u8(record.state().get() as u32, 0);
        // Vendor saturates this abort counter at 0xff rather than wrapping.
        let aborts = mmio.read_u8(record.abort_status().get() as u32);
        if aborts != 0xff {
            mmio.write_u8(record.abort_status().get() as u32, aborts.wrapping_add(1));
        }
        mmio.read_u8(record.last_slot().get() as u32).wrapping_add(1) & 3
    } else {
        mmio.read_u8(record.producer_slot().get() as u32) & 3
    };
    mmio.write_u32(ring.inactive_sentinel(), PIPE_RETRY_INACTIVE_SENTINEL);
    mmio.write_u32(
        PIPE_IRQ_PENDING,
        0_u32.wrapping_sub((PIPE_ADVANCE_ACK_BASE << (pipe & 3)).wrapping_add(0x10)),
    );
    resync_pipe_ring_cursor(mmio, ring, cursor);
    Some(armed)
}

/// Writes both `ring + 0x20` cursor fields, preserving the pending-slot mask in
/// bits 23:0 and the two hardware-owned high bits. This is the only part of
/// `txp_pipe_advance_slot` that the packet controller reads back, and it is the
/// half the open firmware has never performed.
fn resync_pipe_ring_cursor<M: MacPipeMmio>(
    mmio: &mut M,
    ring: TxHardwareRingAddress,
    cursor: u8,
) {
    let cursor = u32::from(cursor & 3);
    let word = mmio.read_u32(ring.cursor_and_pending_mask());
    mmio.write_u32(
        ring.cursor_and_pending_mask(),
        (word & 0xc0ff_ffff) | (cursor << 24) | (cursor << 27),
    );
}

/// Restores the vendor invariant asserted at the end of `txp_fn_4425`
/// (`0x38c`): the software producer `pipe_state + 0` equals the hardware ring
/// cursor `(ring[0x20] & 0x3fffffff) >> 27`. Unlike `advance_pipe_slot` this
/// touches neither the pipe acknowledgement nor the command sentinel, so it is
/// safe to call from the completion handler that already retired the burst.
pub fn resync_pipe_cursor<M: MacPipeMmio>(mmio: &mut M, pipe: u8) -> bool {
    let Some(ring) = retained_hardware_ring(mmio, pipe) else {
        return false;
    };
    let record = pipe_record_address(pipe);
    let producer = mmio.read_u8(record.producer_slot().get() as u32);
    resync_pipe_ring_cursor(mmio, ring, producer);
    true
}

/// The invariant asserted at the end of vendor `txp_fn_4425` (`0x38c`): the
/// software producer equals the hardware ring cursor field in bits 29:27.
pub fn pipe_cursor_invariant_holds(packed: u32) -> bool {
    (packed >> 4) & 0x0f == (packed >> 24) & 0x0f
}

/// Packs the vendor cursor invariant inputs for one pipe into one nibble-coded
/// word plus the raw ring word:
///
/// ```text
/// [3:0] pipe  [7:4] producer  [11:8] last  [15:12] current
/// [19:16] armed  [23:20] ring cursor 26:24  [27:24] ring cursor 29:27
/// ```
///
/// The invariant holds when nibble 1 equals nibble 6.
pub fn pipe_cursor_diagnostic<M: MacPipeMmio>(mmio: &mut M, pipe: u8) -> (u32, u32) {
    if pipe >= 4 {
        return (u32::from(pipe), 0);
    }
    let record = pipe_record_address(pipe);
    let ring_word = retained_hardware_ring(mmio, pipe)
        .map(|ring| mmio.read_u32(ring.cursor_and_pending_mask()))
        .unwrap_or(0);
    let packed = u32::from(pipe & 3)
        | (u32::from(mmio.read_u8(record.producer_slot().get() as u32) & 0x0f) << 4)
        | (u32::from(mmio.read_u8(record.last_slot().get() as u32) & 0x0f) << 8)
        | (u32::from(mmio.read_u8(record.current_slot().get() as u32) & 0x0f) << 12)
        | (u32::from(mmio.read_u8(record.state().get() as u32) & 0x0f) << 16)
        | (((ring_word >> 24) & 7) << 20)
        | (((ring_word >> 27) & 7) << 24);
    (packed, ring_word)
}

#[cfg(all(target_arch = "arm", feature = "vendor-host-tx-diagnostics"))]
unsafe fn capture_status2_ownership(
    event: MacEvent,
    saved_scheduler_word: SchedulerWord,
    mismatch_count: u8,
    phase: u32,
) {
    let pipe = unsafe { read_u8(crate::dtcm::MAC_CURRENT_PIPE.get()) } & 3;
    let pipe_state = pipe_state_address(pipe) as usize;
    let slot_index = unsafe { read_u8(pipe_state + 2) };
    let slot = (slot_index < 4)
        .then_some(pipe_state + 0x0c + usize::from(slot_index) * 0x18)
        .unwrap_or(0);
    let slot_word = |offset: usize| {
        if slot != 0 {
            unsafe { read_u32(slot + offset) }
        } else {
            0
        }
    };
    let command = slot_word(0x14) as usize;
    let hardware_ring = unsafe { read_u32(pipe_state + 8) } as usize;
    let diagnostic_pointer_word = |address: usize, offset: usize| {
        let Some(word) = address.checked_add(offset) else { return 0 };
        if packet_ram::contains_owned_range(word, core::mem::size_of::<u32>()) {
            unsafe { read_u32(word) }
        } else {
            0
        }
    };
    let (completion_consumer, completion_producer) = unsafe { COMPLETION_RING.cursors() };
    let words = [
        event.raw,
        saved_scheduler_word.raw(),
        phase << 24 | u32::from(mismatch_count) << 8 | u32::from(pipe),
        unsafe { read_u32(crate::dtcm::MAC_CURRENT_SLOT.get()) },
        pipe_state as u32,
        unsafe { read_u32(pipe_state) },
        unsafe { read_u32(pipe_state + 4) },
        slot as u32,
        slot_word(0),
        slot_word(8),
        slot_word(0x0c),
        command as u32,
        hardware_ring as u32,
        diagnostic_pointer_word(hardware_ring, 0),
        diagnostic_pointer_word(hardware_ring, 0x14),
        diagnostic_pointer_word(command, 0),
        diagnostic_pointer_word(command, 4),
        diagnostic_pointer_word(command, 8),
        diagnostic_pointer_word(command, 0x14),
        diagnostic_pointer_word(command, 0x18),
        diagnostic_pointer_word(command, 0x1c),
        u32::from(unsafe { read_u8(crate::dtcm::LOW_MAC_PIPE_BUSY.get()) }),
        u32::from(unsafe { read_u8(crate::dtcm::LOW_MAC_ACTIVE_TX_COUNT.get()) }),
        u32::from(unsafe { active_pas_contexts() }),
        completion_consumer,
        completion_producer,
        unsafe { read_u32(PIPE_IRQ_PENDING as usize) },
        unsafe { read_u32(MAC_EVENT_READINESS as usize) },
    ];
    unsafe { crate::host_tx_diagnostics::record_status2_snapshot(phase, &words) };
}

fn current_slot<M: MacPipeMmio>(
    mmio: &mut M,
    pipe: u8,
) -> crate::dtcm::MacPipeSlotAddress {
    let record = pipe_record_address(pipe);
    let slot = usize::from(mmio.read_u8(record.current_slot().get() as u32));
    record.slot_unchecked(slot)
}

fn current_slot_address<M: MacPipeMmio>(mmio: &mut M, pipe: u8) -> u32 {
    current_slot(mmio, pipe).raw()
}

fn publish_current_pipe_slot<M: MacPipeMmio>(mmio: &mut M, pipe: u8) -> (u32, u32) {
    let pipe_state = pipe_record_address(pipe).raw();
    let slot = current_slot_address(mmio, pipe);
    mmio.write_u32(crate::dtcm::MAC_CURRENT_PIPE_RECORD.get() as u32, pipe_state);
    mmio.write_u32(crate::dtcm::MAC_CURRENT_SLOT.get() as u32, slot);
    (pipe_state, slot)
}

/// Executes one already-snapshotted `mac_pipe_irq_service` word.
///
/// This is intentionally not connected to the destructive event FIFO or to
/// any event dispatcher. The caller must read `0x09c00e84` once before bit-23
/// service and retain that word for the later bit-24 decision. The executor
/// performs only the translated service MMIO sequence; random backoff is an
/// explicit infallible policy callback.
pub fn execute_mac_pipe_service<M: MacPipeMmio, P: TxPolicy>(
    mmio: &mut M,
    scheduler_word: SchedulerWord,
    policy: &mut P,
) {
    let latched_pipe = mmio.read_u8(crate::dtcm::MAC_CURRENT_PIPE.get() as u32) & 3;
    let (_, _) = publish_current_pipe_slot(mmio, latched_pipe);
    let plan = plan_mac_pipe_service(scheduler_word, latched_pipe);

    match plan {
        MacPipeServicePlan::None => {}
        MacPipeServicePlan::LowNibble {
            asserted,
            trigger,
            acknowledgement,
            ..
        } => {
            for pipe in 0..3_u8 {
                if asserted & (1 << pipe) != 0 {
                    let slot = current_slot(mmio, pipe);
                    let pas = mmio.read_u32(slot.frame().get() as u32);
                    let backoff_word = mmio.read_u32(slot.command().get() as u32);
                    policy.program_random_backoff(pipe, backoff_word, pas);
                }
            }
            mmio.write_u32(PIPE_IRQ_TRIGGER, trigger);
            mmio.write_u32(PIPE_IRQ_PENDING, acknowledgement);
        }
        MacPipeServicePlan::MiddleNibble {
            selected_pipe,
            quantum_pointer_address,
            trigger,
            acknowledgement,
        } => {
            let quantum_destination = mmio.read_u32(quantum_pointer_address);
            mmio.write_u32(quantum_destination, PIPE_QUANTUM);
            mmio.write_u32(PIPE_IRQ_TRIGGER, trigger);
            let (_, _) = publish_current_pipe_slot(mmio, selected_pipe);
            mmio.write_u32(PIPE_IRQ_PENDING, acknowledgement);
        }
        MacPipeServicePlan::HighNibble {
            acknowledgement, ..
        } => {
            mmio.write_u32(PIPE_IRQ_PENDING, acknowledgement);
        }
    }
}

struct VolatileMacPipeMmio;

impl MacPipeMmio for VolatileMacPipeMmio {
    fn read_u8(&mut self, address: u32) -> u8 {
        unsafe { read_u8(address as usize) }
    }

    fn read_u32(&mut self, address: u32) -> u32 {
        unsafe { read_u32(address as usize) }
    }

    fn read_u16(&mut self, address: u32) -> u16 {
        unsafe { read_u16(address as usize) }
    }

    fn write_u8(&mut self, address: u32, value: u8) {
        unsafe { write_u8(address as usize, value) }
    }

    fn write_u32(&mut self, address: u32, value: u32) {
        unsafe { write_u32(address as usize, value) }
    }

    fn write_u16(&mut self, address: u32, value: u16) {
        unsafe { write_u16(address as usize, value) }
    }
}

/// Inactive raw-MMIO entry point for the translated bit-23 service.
/// Nothing in the firmware calls this yet; in particular it cannot publish a
/// descriptor or raise a hardware trigger merely by being linked.
///
/// # Safety
/// XR819 MMIO and DTCM state must be mapped and exclusively owned.
pub unsafe fn service_mac_pipe_irq_inactive<P: TxPolicy>(
    scheduler_word: SchedulerWord,
    policy: &mut P,
) {
    let mut mmio = VolatileMacPipeMmio;
    execute_mac_pipe_service(&mut mmio, scheduler_word, policy);
}

#[cfg(all(target_arch = "arm", not(target_feature = "thumb-mode")))]
unsafe fn mask_irq_fiq_terminal() -> u32 {
    let previous: u32;
    unsafe {
        core::arch::asm!(
            "mrs {previous}, cpsr",
            "orr r1, {previous}, #0xc0",
            "msr cpsr_c, r1",
            previous = lateout(reg) previous,
            lateout("r1") _,
            options(nostack),
        );
    }
    previous
}

#[cfg(all(target_arch = "arm", not(target_feature = "thumb-mode")))]
unsafe fn restore_irq_fiq(previous: u32) {
    unsafe {
        core::arch::asm!(
            "mrs r1, cpsr",
            "bic r1, r1, #0xc0",
            "and {0}, {0}, #0xc0",
            "orr r1, r1, {0}",
            "msr cpsr_c, r1",
            inout(reg) previous => _,
            lateout("r1") _,
            options(nostack),
        )
    };
}

#[cfg(all(target_arch = "arm", target_feature = "thumb-mode"))]
core::arch::global_asm!(
    ".syntax unified",
    ".pushsection .text.xr819_mask_irq_fiq_terminal, \"ax\", %progbits",
    ".arm",
    ".align 2",
    ".global xr819_mask_irq_fiq_terminal",
    ".type xr819_mask_irq_fiq_terminal, %function",
    "xr819_mask_irq_fiq_terminal:",
    "mrs r0, cpsr",
    "orr r1, r0, #0xc0",
    "msr cpsr_c, r1",
    "bx lr",
    ".size xr819_mask_irq_fiq_terminal, .-xr819_mask_irq_fiq_terminal",
    ".popsection",
    ".thumb",
);

#[cfg(all(target_arch = "arm", target_feature = "thumb-mode"))]
unsafe fn mask_irq_fiq_terminal() -> u32 {
    unsafe extern "C" {
        fn xr819_mask_irq_fiq_terminal() -> u32;
    }
    unsafe { xr819_mask_irq_fiq_terminal() }
}

#[cfg(all(target_arch = "arm", target_feature = "thumb-mode"))]
core::arch::global_asm!(
    ".syntax unified",
    ".pushsection .text.xr819_restore_irq_fiq, \"ax\", %progbits",
    ".arm",
    ".align 2",
    ".global xr819_restore_irq_fiq",
    ".type xr819_restore_irq_fiq, %function",
    "xr819_restore_irq_fiq:",
    "mrs r1, cpsr",
    "bic r1, r1, #0xc0",
    "and r0, r0, #0xc0",
    "orr r1, r1, r0",
    "msr cpsr_c, r1",
    "bx lr",
    ".size xr819_restore_irq_fiq, .-xr819_restore_irq_fiq",
    ".popsection",
    ".thumb",
);

#[cfg(all(target_arch = "arm", target_feature = "thumb-mode"))]
unsafe fn restore_irq_fiq(previous: u32) {
    unsafe extern "C" {
        fn xr819_restore_irq_fiq(previous: u32);
    }
    unsafe { xr819_restore_irq_fiq(previous) };
}

pub fn mac_fatal_postmortem_address() -> *const MacFatalPostmortem {
    MAC_FATAL_POSTMORTEM.0.get().cast_const()
}

/// Terminal production path for MAC event bit 30. It publishes a bounded Rust
/// postmortem after permanently masking IRQ/FIQ, then spins until reset.
///
/// # Safety
/// The caller must own the destructive MAC event consumer. This function never
/// returns and the record may only be reused after a full firmware reset.
#[cfg(target_arch = "arm")]
pub unsafe fn enter_mac_fatal_quiescence(
    event: MacEvent,
    saved_scheduler_word: SchedulerWord,
) -> ! {
    let cpsr = unsafe { mask_irq_fiq_terminal() };
    let record = MAC_FATAL_POSTMORTEM.0.get();
    unsafe { core::ptr::addr_of_mut!((*record).valid).write_volatile(0) };
    unsafe {
        capture_mac_fatal_postmortem(
            &mut VolatileMacPipeMmio,
            event,
            saved_scheduler_word,
            cpsr,
            &mut *record,
        );
    }
    unsafe { core::arch::asm!("", options(nostack, preserves_flags)) };
    unsafe { core::ptr::addr_of_mut!((*record).valid).write_volatile(MAC_FATAL_MAGIC) };
    unsafe { core::arch::asm!("", options(nostack, preserves_flags)) };
    unsafe {
        crate::host_tx_diagnostics::freeze_status2_snapshots();
        #[cfg(feature = "vendor-host-tx-diagnostics")]
        let rx = crate::radio::fatal_command_snapshot();
        #[cfg(not(feature = "vendor-host-tx-diagnostics"))]
        let rx = [0_u32; 11];
        // The vendor traces the raw event before treating bit 30 as terminal.
        // Capture the still-live RX producer delta at that same boundary so a
        // MAC fatal cannot preempt the later cooperative command matcher.
        // The complete pre-GO and postmortem records remain committed in BSS.
        let postmortem = &*record;
        let pipe = postmortem.current_pipe as u8;
        let pipe_state = postmortem.current_pipe_record as usize;
        let slot = postmortem.current_slot as usize;
        let command = read_u32(slot.wrapping_add(0x14)) as usize;
        let expected_record = crate::dtcm::MacPipeRecordAddress::from_index(usize::from(pipe));
        let hardware_ring = expected_record
            .filter(|record| record.raw() == postmortem.current_pipe_record)
            .and_then(|record| {
                TxHardwareRingAddress::for_pipe(pipe, read_u32(record.hardware_ring().get()))
            });
        let ring_word = hardware_ring
            .map(|ring| read_u32(ring.cursor_and_pending_mask() as usize))
            .unwrap_or(0);
        // The slot the MAC itself is pointing at, which is not necessarily the
        // one we think is current.
        let cursor_slot = ((ring_word >> 27) & 3) as usize;
        let cursor_command = expected_record
            .map(|_| packet_ram::tx_command(usize::from(pipe), cursor_slot))
            .unwrap_or(0);
        let cursor_command_word_0 = if cursor_command != 0 { read_u32(cursor_command) } else { 0 };
        let cursor_command_word_14 = if cursor_command != 0 {
            read_u32(cursor_command.wrapping_add(0x14))
        } else {
            0
        };
        let trace_count = read_u32(0xfff0_3794);
        let prior_event = |back: u32| {
            let index = trace_count.wrapping_sub(back) & 0x1f;
            read_u32(0xfff0_3714 + index as usize * 4)
        };
        crate::hif::publish_mac_fatal_exception([
            // `trace_mac_event()` commits the raw event before dispatch. Keep
            // the preceding event sequence inside the driver's bounded trace
            // prefix: the fatal often arrives after the owning slot has
            // already been released, making postmortem frame pointers stale.
            event.raw,
            trace_count,
            prior_event(2),
            prior_event(3),
            prior_event(4),
            prior_event(5),
            command as u32,
            // The driver's diag buffer truncates at 80 bytes (8 header + 18
            // words), so anything past word 17 never reaches dmesg. These four
            // slots held the command setup words, which static analysis showed
            // are well-formed and constant across captures; the cursor/producer
            // comparison is what we actually lack. The displaced command words
            // move to the tail, still committed in BSS.
            // `current_pipe_record` is already the field-derived MAC pipe record root.
            // (a capture showed 0x04001720 = 0x04001680 + 0xa0 for pipe 0), and
            // `hardware_ring` is read from `+8`, matching vendor's `iVar4+0xa8`.
            // So the slot bytes are at +0..+3, not +0xa0..+0xa3.
            u32::from(read_u8(pipe_state))
                | (u32::from(read_u8(pipe_state.wrapping_add(1))) << 8)
                | (u32::from(read_u8(pipe_state.wrapping_add(2))) << 16)
                | (u32::from(read_u8(pipe_state.wrapping_add(3))) << 24),
            cursor_command as u32,
            cursor_command_word_0,
            cursor_command_word_14,
            rx[0],
            rx[1],
            postmortem.current_pipe,
            pipe_state as u32,
            hardware_ring.map(TxHardwareRingAddress::raw).unwrap_or(0),
            ring_word,
            postmortem.event_readiness,
            // Vendor asserts that the software slot pointer equals the hardware
            // ring cursor (`ring+0x20` bits 29:27); see tx_ptcs.c's check in
            // `annotated-main.c:657`. We have never recorded both sides at a
            // fault, so the comparison it panics on was never actually made.
            // A prior capture had producer 2 against cursor 3, with the cursor
            // slot holding a null frame and stale command storage, and seven
            // words of that same stale storage turned up in the RX FIFO.
            read_u32(command.wrapping_add(0x0c)),
            read_u32(command.wrapping_add(0x10)),
            read_u32(command.wrapping_add(0x14)),
            read_u32(command.wrapping_add(0x18)),
        ]);
    }
    loop {
        unsafe { core::arch::asm!("nop", options(nomem, nostack)) };
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OrdinaryTxPipeStatusInput {
    pub pipe_active: bool,
    pub expected_status: u8,
    pub slot_kind: u8,
    pub slot_state: u8,
    pub global_busy: bool,
    pub pipe_current: u8,
    pub pipe_last: u8,
    pub pipe_status: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrdinaryTxPipeCursorPlan {
    AdvanceCurrent { next_current: u8 },
    Recycle { next_head: u8 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrdinaryTxPipeStatusPlan {
    Ineligible,
    SuppressedAfterSlotMark,
    AdvanceWithoutCompletion {
        initialize_pipe_status: bool,
        next_current: u8,
    },
    Complete {
        initialize_pipe_status: bool,
        cursor: OrdinaryTxPipeCursorPlan,
    },
}

/// Pure mandatory part of `txp_pipe_tx_status` (`0x9a32`). The suppressed
/// result intentionally follows the vendor ordering: the slot has already
/// moved from state 3 to state 5 before the global busy byte is examined.
pub fn plan_ordinary_tx_pipe_status(
    input: OrdinaryTxPipeStatusInput,
    status: u8,
) -> OrdinaryTxPipeStatusPlan {
    if !input.pipe_active || input.expected_status != status || input.slot_state != 3 {
        return OrdinaryTxPipeStatusPlan::Ineligible;
    }
    if input.global_busy {
        return OrdinaryTxPipeStatusPlan::SuppressedAfterSlotMark;
    }

    let initialize_pipe_status = i8::from_ne_bytes([input.pipe_status]) < 1;
    let next_current = (input.pipe_current + 1) & 3;
    if input.slot_kind == 2 {
        return OrdinaryTxPipeStatusPlan::AdvanceWithoutCompletion {
            initialize_pipe_status,
            next_current,
        };
    }
    let cursor = if input.pipe_current == input.pipe_last {
        OrdinaryTxPipeCursorPlan::Recycle {
            next_head: (input.pipe_last + 1) & 3,
        }
    } else {
        OrdinaryTxPipeCursorPlan::AdvanceCurrent { next_current }
    };
    OrdinaryTxPipeStatusPlan::Complete {
        initialize_pipe_status,
        cursor,
    }
}

/// Sticky record of the last refused TX status on an armed pipe, plus how many
/// were refused. Read back through the cursor-divergence report, because the
/// counters MIB stops answering once the pipe wedges.
#[cfg(all(target_arch = "arm", feature = "vendor-host-tx-diagnostics"))]
struct IneligibleTxStatus {
    count: core::cell::UnsafeCell<u32>,
    last: core::cell::UnsafeCell<u32>,
    seen: core::cell::UnsafeCell<u32>,
}

#[cfg(all(target_arch = "arm", feature = "vendor-host-tx-diagnostics"))]
unsafe impl Sync for IneligibleTxStatus {}

#[cfg(all(target_arch = "arm", feature = "vendor-host-tx-diagnostics"))]
static INELIGIBLE_TX_STATUS: IneligibleTxStatus = IneligibleTxStatus {
    count: core::cell::UnsafeCell::new(0),
    last: core::cell::UnsafeCell::new(0),
    seen: core::cell::UnsafeCell::new(0),
};

#[cfg(all(target_arch = "arm", feature = "vendor-host-tx-diagnostics"))]
fn record_ineligible_tx_status(input: OrdinaryTxPipeStatusInput, status: u8) {
    unsafe {
        let count = INELIGIBLE_TX_STATUS.count.get();
        count.write_volatile(count.read_volatile().wrapping_add(1));
        INELIGIBLE_TX_STATUS.last.get().write_volatile(
            u32::from(status)
                | (u32::from(input.expected_status) << 8)
                | (u32::from(input.slot_state) << 16)
                | (u32::from(input.slot_kind) << 24),
        );
        // Bitmap of every distinct status value the MAC has delivered, so a
        // single report shows whether `expected_status` was ever offered.
        let seen = INELIGIBLE_TX_STATUS.seen.get();
        seen.write_volatile(seen.read_volatile() | (1_u32 << (status & 0x1f)));
    }
}

/// `(refused count, last refused word, delivered-status bitmap)`.
#[cfg(all(target_arch = "arm", feature = "vendor-host-tx-diagnostics"))]
pub(crate) fn ineligible_tx_status_snapshot() -> (u32, u32, u32) {
    unsafe {
        (
            INELIGIBLE_TX_STATUS.count.get().read_volatile(),
            INELIGIBLE_TX_STATUS.last.get().read_volatile(),
            INELIGIBLE_TX_STATUS.seen.get().read_volatile(),
        )
    }
}

const fn aggregate_retry_command_owned(slot_kind: u8, slot_state: u8) -> bool {
    slot_kind == 1 && slot_state == 4
}

const fn empty_retirement_allowed(frame_node: u32, slot_kind: u8, slot_state: u8) -> bool {
    frame_node == 0 && !aggregate_retry_command_owned(slot_kind, slot_state)
}

/// Release the current hardware command-mask owner for a retry-owned kind-1
/// slot only after validating its complete retained identity.
fn release_aggregate_retry_command_mask<M: MacPipeMmio>(
    mmio: &mut M,
    pipe: u8,
    retained_slot: u32,
    retained_frame: FrameNodeAddress,
) -> Option<bool> {
    if pipe >= 4 {
        return None;
    }
    let record = pipe_record_address(pipe);
    let current = mmio.read_u8(record.current_slot().get() as u32);
    let slot = LiveTxSlot::from_mmio(
        mmio,
        pipe,
        current,
        retained_slot,
        Some(retained_frame),
    )?;
    if !aggregate_retry_command_owned(
        mmio.read_u8(slot.address.kind().get() as u32),
        mmio.read_u8(slot.address.state().get() as u32),
    ) {
        return Some(false);
    }
    let ring = TxHardwareRingAddress::for_pipe(
        pipe,
        mmio.read_u32(record.hardware_ring().get() as u32),
    )?;
    let command_mask = mmio.read_u32(ring.cursor_and_pending_mask());
    mmio.write_u32(
        ring.cursor_and_pending_mask(),
        command_mask & !(1_u32 << current),
    );
    Some(true)
}

/// Release a retry-owned command-mask bit only after the slot's exact frame and
/// packet-RAM command identities have been validated.
#[cfg(target_arch = "arm")]
unsafe fn release_retiring_aggregate_command_mask(pipe: u8, slot: LiveTxSlot) -> Option<bool> {
    unsafe {
        if !aggregate_retry_command_owned(
            read_u8(slot.address.kind().get()),
            read_u8(slot.address.state().get()),
        ) {
            return Some(false);
        }
        let record = pipe_record_address(pipe);
        let ring = TxHardwareRingAddress::for_pipe(
            pipe,
            read_u32(record.hardware_ring().get()),
        )?;
        let current = read_u8(record.current_slot().get());
        if current >= 4 {
            return None;
        }
        write_u32(
            ring.cursor_and_pending_mask() as usize,
            read_u32(ring.cursor_and_pending_mask() as usize) & !(1_u32 << current),
        );
        Some(true)
    }
}

/// Completes one slot the hardware has already left behind, using the vendor
/// give-up status `0x0b` so the host receives a truthful TX failure instead of
/// silently losing the frame.
///
/// # Safety
/// The pipe must be armed, the slot started, and the caller must exclusively
/// own the pipe records and completion state.
#[cfg(target_arch = "arm")]
unsafe fn retire_unmatched_tx_slot<B: TxStatusPolicy>(
    pipe: u8,
    slot: u8,
    cursor: OrdinaryTxPipeCursorPlan,
    backend: &mut B,
) {
    unsafe {
        let record = pipe_record_address(pipe);
        let Some(slot) = RetirableTxSlot::from_pipe_slot(pipe, slot) else {
            crate::halt_always!();
        };
        crate::host_tx_diagnostics::bump(crate::host_tx_diagnostics::counter::RETIRED);
        let aggregate = match slot {
            RetirableTxSlot::Empty(_) => false,
            RetirableTxSlot::Live(slot) => {
                let Some(released) = release_retiring_aggregate_command_mask(pipe, slot) else {
                    crate::halt_always!();
                };
                released
            }
        };
        match slot {
            RetirableTxSlot::Empty(slot) => write_u32(slot.frame().get(), 0),
            RetirableTxSlot::Live(slot) => {
                complete_tx_pipe_slot(slot.frame_node, slot.address.raw(), 0x0b, backend)
            }
        }
        if aggregate {
            write_u8(crate::dtcm::LOW_MAC_PIPE_BUSY.get(), 0);
        }
        match cursor {
            OrdinaryTxPipeCursorPlan::AdvanceCurrent { next_current } => {
                write_u8(record.current_slot().get(), next_current);
                write_u8(record.producer_slot().get(), next_current);
            }
            OrdinaryTxPipeCursorPlan::Recycle { next_head } => {
                write_u8(record.producer_slot().get(), next_head);
                write_u8(record.state().get(), 0);
                write_u8(record.control().get(), 0);
                write_u8(record.watchdog().get(), 5);
            }
        }
    }
}

/// Number of cooperative main-loop passes between expensive hardware-timer
/// samples. The timer still determines elapsed time; this only gates MMIO.
pub const WATCHDOG_TIMER_SAMPLE_DIVIDER: u8 = 16;

pub const fn advance_watchdog_timer_divider(countdown: u8) -> (u8, bool) {
    if countdown == 0 {
        (WATCHDOG_TIMER_SAMPLE_DIVIDER - 1, true)
    } else {
        (countdown - 1, false)
    }
}

/// One pipe's watchdog decision, from vendor `FUN_00003bac` (the 200 ms timer
/// that Ghidra never turned into a function, recovered by disassembly).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PipeWatchdogAction {
    /// Not programmed, or not armed: nothing to supervise.
    Idle,
    /// Healthy; the counter was decremented.
    Tick,
    /// One or two ticks from expiry. Vendor nudges the radio to let a nearly
    /// stuck pipe drain (`mac_rx_pause_briefly`).
    Nudge,
    /// Expired: the pipe has been armed without completing for the whole
    /// watchdog window and must be recovered.
    Expired,
}

/// Pure per-pipe watchdog decision.
///
/// Vendor decrements a signed `pipe_state + 0xa5` every 200 ms for any pipe
/// whose `+0xa4` bit 0 ("programmed by the scheduler") is set, and recovers the
/// pipe once the counter reaches zero. Arming reloads the counter to 5, so a
/// pipe that never completes is recovered after roughly a second.
///
/// The open firmware already reloads `+0xa5` at every arm point but never
/// decremented it, so an armed pipe whose completion never arrives stayed armed
/// forever. Retirement could not rescue it either, because retirement only runs
/// from a delivered status and the MAC stops delivering statuses for a wedged
/// pipe.
pub const fn plan_pipe_watchdog(programmed: bool, armed: bool, counter: i8) -> PipeWatchdogAction {
    if !programmed || !armed {
        return PipeWatchdogAction::Idle;
    }
    if counter > 2 {
        PipeWatchdogAction::Tick
    } else if counter > 0 {
        PipeWatchdogAction::Nudge
    } else {
        PipeWatchdogAction::Expired
    }
}

/// Runs one 200 ms watchdog tick across all four pipes.
///
/// Recovery reuses the retirement path rather than inventing a second one: an
/// expired pipe retires its current slot exactly as an unmatched status would,
/// which also releases the host slot from `Owned { Scheduled }` and so restores
/// the host command lane.
///
/// # Safety
/// Must run from the single-threaded firmware context that owns pipe state.
#[cfg(target_arch = "arm")]
pub unsafe fn service_pipe_watchdog_tick_runtime() {
    let runtime = unsafe { &mut *PROBE_EXPERIMENT.0.get() };
    unsafe { service_pipe_watchdog_tick(&mut runtime.backend) };
}

/// See `service_pipe_watchdog_tick_runtime`.
///
/// # Safety
/// Must run from the single-threaded firmware context that owns pipe state.
#[cfg(target_arch = "arm")]
pub unsafe fn service_pipe_watchdog_tick<B: TxStatusPolicy>(backend: &mut B) {
    unsafe {
        for pipe in 0..4_u8 {
            let record = pipe_record_address(pipe);
            let programmed = read_u8(record.control().get()) & 1 != 0;
            let armed = read_u8(record.state().get()) == 1;
            let counter = read_u8(record.watchdog().get()) as i8;
            match plan_pipe_watchdog(programmed, armed, counter) {
                PipeWatchdogAction::Idle => (),
                PipeWatchdogAction::Tick | PipeWatchdogAction::Nudge => {
                    write_u8(record.watchdog().get(), counter.wrapping_sub(1) as u8);
                }
                PipeWatchdogAction::Expired => {
                    let current = read_u8(record.current_slot().get());
                    let cursor = match hardware_pipe_cursor(&mut VolatileMacPipeMmio, pipe) {
                        Ok(Some(cursor)) => OrdinaryTxPipeCursorPlan::Recycle {
                            next_head: cursor,
                        },
                        Ok(None) => OrdinaryTxPipeCursorPlan::Recycle {
                            next_head: read_u8(record.last_slot().get()).wrapping_add(1) & 3,
                        },
                        Err(()) => crate::halt_always!(),
                    };
                    crate::host_tx_diagnostics::bump(
                        crate::host_tx_diagnostics::counter::WATCHDOG_RECOVERED,
                    );
                    retire_unmatched_tx_slot(pipe, current, cursor, backend);
                    // Vendor `txp_fn_4155` clears the programmed/abort bits and
                    // reloads the counter after a recovery pass.
                    write_u8(record.control().get(), read_u8(record.control().get()) & 0xf6);
                    write_u8(record.watchdog().get(), 5);
                }
            }
        }
    }
}

/// Reads the hardware ring cursor (`ring + 0x20` bits 29:27) for one pipe.
/// `None` when the pipe has no programmed ring.
fn hardware_pipe_cursor<M: MacPipeMmio>(
    mmio: &mut M,
    pipe: u8,
) -> Result<Option<u8>, ()> {
    if pipe >= 4 {
        return Err(());
    }
    let record = pipe_record_address(pipe);
    let raw = mmio.read_u32(record.hardware_ring().get() as u32);
    if raw == 0 {
        return Ok(None);
    }
    let ring = TxHardwareRingAddress::for_pipe(pipe, raw).ok_or(())?;
    Ok(Some(
        ((mmio.read_u32(ring.cursor_and_pending_mask()) >> 27) & 3) as u8,
    ))
}

/// Policy boundary for the unresolved `pas_backoff_reset` in ordinary status.
/// It cannot reject, defer, or return an error after event FIFO consumption.
pub trait TxStatusPolicy: PipeSlotCompletionEffects {
    fn reset_status_backoff(&mut self, link: u8, queue: u8, pas_state: u32, queue_table: u32);
}

/// Exact inactive executor for ordinary `txp_pipe_tx_status`.
///
/// It performs the mandatory slot/pipe ownership transitions and delegates
/// the completion boundary to [`complete_tx_pipe_slot`]. It is not wired to
/// destructive event FIFO reads, descriptor publication, or hardware GO.
///
/// # Safety
/// Pipe records, frame nodes, completion state, and MMIO must be valid and
/// exclusively owned by the caller.
pub unsafe fn service_txp_pipe_tx_status<B: TxStatusPolicy>(status: u8, backend: &mut B) {
    let pipe = unsafe { read_u8(crate::dtcm::MAC_CURRENT_PIPE.get()) } & 3;
    let record = pipe_record_address(pipe);
    let current = unsafe { read_u8(record.current_slot().get()) };
    let slot = record.slot_unchecked(usize::from(current));
    unsafe {
        write_u32(crate::dtcm::MAC_CURRENT_PIPE_RECORD.get(), record.raw());
        write_u32(crate::dtcm::MAC_CURRENT_SLOT.get(), slot.raw());

        let input = OrdinaryTxPipeStatusInput {
            pipe_active: read_u8(record.state().get()) == 1,
            expected_status: read_u8(slot.expected_status().get()),
            slot_kind: read_u8(slot.kind().get()),
            slot_state: read_u8(slot.state().get()),
            global_busy: read_u8(crate::dtcm::LOW_MAC_PIPE_BUSY.get()) != 0,
            pipe_current: read_u8(record.current_slot().get()),
            pipe_last: read_u8(record.last_slot().get()),
            pipe_status: read_u8(record.watchdog().get()),
        };
        crate::host_tx_diagnostics::bump(crate::host_tx_diagnostics::counter::STATUS);
        crate::host_tx_diagnostics::observe(
            crate::host_tx_diagnostics::counter::LAST_STATUS,
            u32::from(status),
        );
        crate::host_tx_diagnostics::observe(
            crate::host_tx_diagnostics::counter::LAST_EXPECTED,
            u32::from(input.expected_status),
        );
        let plan = plan_ordinary_tx_pipe_status(input, status);
        if matches!(plan, OrdinaryTxPipeStatusPlan::Ineligible) {
            crate::host_tx_diagnostics::bump(
                crate::host_tx_diagnostics::counter::STATUS_INELIGIBLE,
            );
            // An armed pipe that keeps rejecting statuses never retires its
            // slot, so record why the gate refused. `expected_status` is the
            // slot's `frame + 0x56` byte; the pipe stalls when no delivered
            // status ever equals it.
            #[cfg(all(target_arch = "arm", feature = "vendor-host-tx-diagnostics"))]
            if input.pipe_active {
                record_ineligible_tx_status(input, status);
            }
            return;
        }

        // This write must precede the busy/suppression decision.
        write_u8(slot.state().get(), 5);
        match plan {
            OrdinaryTxPipeStatusPlan::Ineligible => unreachable!(),
            OrdinaryTxPipeStatusPlan::SuppressedAfterSlotMark => (),
            OrdinaryTxPipeStatusPlan::AdvanceWithoutCompletion {
                initialize_pipe_status,
                next_current,
            } => {
                if initialize_pipe_status {
                    write_u8(record.watchdog().get(), 1);
                }
                write_u32(
                    PIPE_STATUS_COUNTER as usize,
                    read_u32(PIPE_STATUS_COUNTER as usize).wrapping_add(1),
                );
                write_u32(
                    crate::dtcm::MAC_STATUS_ACCOUNTING.get(),
                    read_u32(crate::dtcm::MAC_STATUS_ACCOUNTING.get()).wrapping_add(1),
                );
                write_u8(record.current_slot().get(), next_current);
            }
            OrdinaryTxPipeStatusPlan::Complete {
                initialize_pipe_status,
                cursor,
            } => {
                crate::host_tx_diagnostics::bump(crate::host_tx_diagnostics::counter::COMPLETED);
                if initialize_pipe_status {
                    write_u8(record.watchdog().get(), 1);
                }
                let pas = read_u32(slot.frame().get());
                backend.reset_status_backoff(
                    read_u8(pas as usize + 0x69),
                    read_u8(
                        crate::dtcm::queue_to_access_category_unchecked(usize::from(pipe)).get(),
                    ),
                    pas.wrapping_add(0x60),
                    crate::dtcm::QUEUE_TO_ACCESS_CATEGORY.get() as u32,
                );
                let pas_address = pas as usize;
                write_u32(pas_address + 0x2c, read_u32(pas_address + 0x2c) | 0x200);
                complete_tx_pipe_slot(FrameNodeAddress::new(pas), slot.raw(), 0, backend);
                match cursor {
                    OrdinaryTxPipeCursorPlan::AdvanceCurrent { next_current } => {
                        write_u8(record.current_slot().get(), next_current);
                        write_u8(record.producer_slot().get(), next_current);
                    }
                    OrdinaryTxPipeCursorPlan::Recycle { next_head } => {
                        write_u8(record.producer_slot().get(), next_head);
                        write_u8(record.state().get(), 0);
                        write_u8(record.control().get(), 0);
                        write_u8(record.watchdog().get(), 5);
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MacTxStatusAccountingPlan {
    pub increment_range_counter: bool,
    pub specific_counter: Option<u32>,
    pub increment_total_counter: bool,
    pub service_status_0e_side_effects: bool,
}

pub fn plan_mac_tx_status_accounting(status: u8) -> MacTxStatusAccountingPlan {
    let (specific_counter, increment_total_counter) = match status {
        0 => (Some(0xfff0_2e40), false),
        3 => (Some(0xfff0_1a90), true),
        4 => (Some(0xfff0_1a94), true),
        _ => (None, false),
    };
    MacTxStatusAccountingPlan {
        increment_range_counter: (6..0x19).contains(&status),
        specific_counter,
        increment_total_counter,
        service_status_0e_side_effects: status == 0x0e,
    }
}

/// Exact `mac_irq_tx_status_dispatch` accounting and status-`0x0e` side
/// effects before ordinary `txp_pipe_tx_status` dispatch.
///
/// # Safety
/// Statistics aliases, MAC status state, and RX lookup state must be mapped and
/// exclusively owned by the popped-event executor.
pub unsafe fn service_mac_irq_tx_status_dispatch<B: TxStatusPolicy>(status: u8, backend: &mut B) {
    unsafe {
        let plan = plan_mac_tx_status_accounting(status);
        if plan.increment_range_counter {
            write_u32(0xfff0_1aac, read_u32(0xfff0_1aac).wrapping_add(1));
        }
        if let Some(counter) = plan.specific_counter {
            write_u32(counter as usize, read_u32(counter as usize).wrapping_add(1));
        }
        if plan.increment_total_counter {
            write_u32(0xfff0_1aa0, read_u32(0xfff0_1aa0).wrapping_add(1));
        }
        if plan.service_status_0e_side_effects {
            write_u32(
                crate::dtcm::MAC_PHY_COMPLETION_STATUS.get(),
                read_u32(crate::dtcm::RX_FIFO_STATE.claim_cursor().get()),
            );
            let _ = backend.find_rx_frame_by_subtype(0x80);
            if read_u32(crate::dtcm::MAC_WAKE_CONTROL.get()) != 0 {
                let control = read_u32(crate::dtcm::MAC_BEACON_CONTROL.get()) & !1;
                write_u32(crate::dtcm::MAC_BEACON_CONTROL.get(), control);
                write_u32(crate::platform::mac_register(0x0a00), control);
                write_u32(crate::dtcm::MAC_BEACON_CONTROL_STATE.get(), 1);
            }
        }
        service_txp_pipe_tx_status(status, backend);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SingleTxRetryDecision {
    /// The backend has selected another bounded attempt. `rearm_and_ack()` must
    /// rebuild the descriptor as needed, retain slot/context ownership, re-arm
    /// the MAC, and perform the matching acknowledgement before returning.
    Rearm,
    /// The received BlockAck confirms every tracked aggregate member.
    CompleteSuccess,
    /// A kind-1 transmission is waiting for its received BlockAck bitmap.
    AwaitBlockAck,
    /// Complete the current frame or aggregate with vendor status `0x0b`.
    GiveUp,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SingleTxRetryOutcome {
    MaskNotOwned,
    SlotNotStarted,
    InactivePipeAcknowledged,
    Rearmed,
    Completed,
    AwaitingBlockAck,
    GivenUp,
}

/// Replacement for vendor TALA/retry policy in the initial one-outstanding
/// low-MAC. `max_retries` counts re-arms after the original transmission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BoundedSingleTxRetry {
    attempts: u8,
    max_retries: u8,
}

impl BoundedSingleTxRetry {
    pub const fn new(max_retries: u8) -> Self {
        Self {
            attempts: 0,
            max_retries,
        }
    }

    pub const fn attempts(self) -> u8 {
        self.attempts
    }

    pub fn decide(&mut self) -> SingleTxRetryDecision {
        if self.attempts >= self.max_retries {
            SingleTxRetryDecision::GiveUp
        } else {
            self.record_rearm();
            SingleTxRetryDecision::Rearm
        }
    }

    fn record_rearm(&mut self) {
        self.attempts = self.attempts.saturating_add(1);
    }

    pub fn reset(&mut self) {
        self.attempts = 0;
    }
}

/// Policy and remaining descriptor-rebuild leaves for the deliberately small
/// one-outstanding, non-aggregate retry path. None of these operations may
/// reject an event after the destructive MAC FIFO read.
pub trait SingleTxRetryBackend {
    fn decide_retry(
        &mut self,
        pipe: u8,
        slot: u32,
        frame_node: FrameNodeAddress,
    ) -> SingleTxRetryDecision;

    fn rearm_and_ack(
        &mut self,
        pipe: u8,
        slot: u32,
        frame_node: FrameNodeAddress,
        pending_mask: u32,
    );

    fn complete_success(&mut self, pipe: u8, frame_node: FrameNodeAddress, slot: u32);

    fn complete_give_up(
        &mut self,
        pipe: u8,
        frame_node: FrameNodeAddress,
        slot: u32,
        status: u16,
    );

    /// Status class 6 enters the vendor multi-slot path and is outside the
    /// enabled single-frame subset. Production must enter fatal quiescence.
    fn fatal_unsupported_multi_slot_retry(
        &mut self,
        pipe: u8,
        slot: u32,
        frame_node: FrameNodeAddress,
    ) -> !;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SingleFrameRearmOutcome {
    CommandMaskAcknowledged,
    HardwareSentinelAcknowledged,
}

fn next_retry_random24<M: MacPipeMmio>(_mmio: &mut M) -> u32 {
    #[cfg(target_arch = "arm")]
    let state = unsafe { RETRY_RANDOM_STATE.0.get().read_volatile() };
    #[cfg(not(target_arch = "arm"))]
    let state = _mmio.read_u32(crate::dtcm::initialized_random_lfsr().get() as u32);

    let mixed = (state >> 4) ^ state;
    let next = (state << 27) | (mixed & 0x07ff_ffff);

    #[cfg(target_arch = "arm")]
    unsafe {
        RETRY_RANDOM_STATE.0.get().write_volatile(next);
    }
    #[cfg(not(target_arch = "arm"))]
    _mmio.write_u32(crate::dtcm::initialized_random_lfsr().get() as u32, next);

    next & 0x00ff_ffff
}

fn build_single_frame_duration<M: MacPipeMmio>(
    mmio: &mut M,
    descriptor: u32,
    frame_node: FrameNodeAddress,
    expects_ack: bool,
) {
    let descriptor = TxDescriptorAddress::new(descriptor);
    mmio.write_u32(descriptor.command(), 0);

    let interface = u32::from(mmio.read_u8(frame_node.interface()));
    let selector = u32::from(mmio.read_u8(frame_node.access_category()));
    let random_mask = mmio.read_u16(
        crate::dtcm::pas_stride_view_unchecked(interface as usize)
            .contention_window_unchecked(selector as usize)
            .get() as u32,
    );
    let random = (next_retry_random24(mmio) as u16) & random_mask;

    let maximum = mmio.read_u32(PIPE_RETRY_RANDOM_STATS);
    if maximum < u32::from(random) {
        mmio.write_u32(PIPE_RETRY_RANDOM_STATS, u32::from(random));
    }
    let bucket = match random {
        0..=7 => 1,
        8..=15 => 2,
        16..=31 => 3,
        32..=63 => 4,
        64..=127 => 5,
        128..=255 => 6,
        256..=511 => 7,
        _ => 8,
    };
    let bucket_address = PIPE_RETRY_RANDOM_STATS + bucket * 4;
    let bucket_count = mmio.read_u32(bucket_address).wrapping_add(1);
    mmio.write_u32(bucket_address, bucket_count);

    mmio.write_u16(frame_node.retry_random(), random);
    mmio.write_u32(descriptor.flags(), (u32::from(random) & 0x0fff) << 10);

    let duration_word = if expects_ack {
        let rate = u32::from(mmio.read_u8(frame_node.tx_rate()));
        let timing_index = u32::from(mmio.read_u8(
            crate::dtcm::mac_retry_rate_unchecked(rate as usize).get() as u32,
        ));
        let timing = u32::from(mmio.read_u16(
            crate::dtcm::tx_duration_timing_unchecked(timing_index as usize).get() as u32,
        ));
        let duration = mmio
            .read_u32(crate::dtcm::LOW_MAC_SLOT_TIME_INITIAL.get() as u32)
            .wrapping_add(
                mmio.read_u32(crate::dtcm::LOW_MAC_SLOT_TIME_BASE.get() as u32)
                    .wrapping_mul(2),
            )
            .wrapping_add(timing)
            & 0xffff;
        0xd800_0000 | (((duration & 0x03ff) * 8).wrapping_add(0x2000))
    } else {
        0xdc00_0000
    };
    mmio.write_u32(descriptor.duration(), duration_word);
}

/// Exact mode-1 `tx_build_duration_desc` used by a fixed-rate, non-aggregate
/// retry. The wider mode-2/mode-5 protection branches remain outside the
/// enabled one-frame subset.
pub fn build_fixed_rate_retry_duration<M: MacPipeMmio>(
    mmio: &mut M,
    descriptor: u32,
    frame_node: FrameNodeAddress,
) {
    build_single_frame_duration(mmio, descriptor, frame_node, true);
}

/// Exact mode-0 duration descriptor for the no-ACK broadcast probe.
pub fn build_no_ack_single_frame_duration<M: MacPipeMmio>(
    mmio: &mut M,
    descriptor: u32,
    frame_node: FrameNodeAddress,
) {
    build_single_frame_duration(mmio, descriptor, frame_node, false);
}

/// Fatal boundary for retry shapes outside the fixed-rate one-frame subset.
pub trait SingleFrameRearmBackend {
    fn rebuild_rate_descriptor(&mut self, _pipe: u8, _slot: u32, _frame_node: FrameNodeAddress) {}

    fn fatal_unsupported_rearm_shape(
        &mut self,
        pipe: u8,
        slot: u32,
        frame_node: FrameNodeAddress,
    ) -> !;
}

/// Matching-payload fixed-rate, one-slot tail of the successful re-arm branch
/// at `0x97de..0x9a2e`. Rate-change rebuilding, aggregation, and a nontrivial
/// ring cursor are deliberately fatal until their ownership effects exist.
pub fn execute_fixed_rate_single_frame_rearm<M, B>(
    mmio: &mut M,
    pipe: u8,
    slot: u32,
    frame_node: FrameNodeAddress,
    pending_mask: u32,
    backend: &mut B,
) -> SingleFrameRearmOutcome
where
    M: MacPipeMmio,
    B: SingleFrameRearmBackend,
{
    if pipe >= 4 {
        backend.fatal_unsupported_rearm_shape(pipe, slot, frame_node);
    }
    let Some(ring) = retained_hardware_ring(mmio, pipe) else {
        backend.fatal_unsupported_rearm_shape(pipe, slot, frame_node);
    };
    let record = pipe_record_address(pipe);
    let slot_address = crate::dtcm::MacPipeSlotAddress::from_raw_unchecked(slot);

    let flags = mmio.read_u32(frame_node.raw() + 4);
    if mmio.read_u8(slot_address.kind().get() as u32) == 1 {
        backend.fatal_unsupported_rearm_shape(pipe, slot, frame_node);
    }
    if flags & 0x0008_0000 != 0 {
        backend.rebuild_rate_descriptor(pipe, slot, frame_node);
    }

    let descriptor = TxDescriptorAddress::new(mmio.read_u32(slot_address.command().get() as u32));
    let status = mmio.read_u8(slot_address.expected_status().get() as u32);
    build_fixed_rate_retry_duration(mmio, descriptor.raw(), frame_node);
    let descriptor_flags = mmio.read_u32(descriptor.flags()) | u32::from(status).wrapping_add(0x80);
    mmio.write_u32(descriptor.flags(), descriptor_flags);

    mmio.write_u32(PIPE_IRQ_TRIGGER, (1_u32 << pipe) << 25);

    if mmio.read_u8(crate::dtcm::mac_retry_hardware_state_mmio_address()) & 2 != 0 {
        mmio.write_u32(ring.inactive_sentinel(), PIPE_RETRY_INACTIVE_SENTINEL);
        mmio.write_u32(
            PIPE_IRQ_PENDING,
            0_u32.wrapping_sub(pending_mask.wrapping_add(PIPE_RETRY_SPECIAL_ACK)),
        );
        return SingleFrameRearmOutcome::HardwareSentinelAcknowledged;
    }

    let producer = mmio.read_u8(record.producer_slot().get() as u32) & 3;
    let current = mmio.read_u8(record.current_slot().get() as u32) & 3;
    if producer != current {
        // Vendor's multi-slot branch first checks a mapped companion slot. For
        // an ordinary legacy batch that slot has kind 0 and a different PAS;
        // with bit 15 clear it takes the simple fallback: clear only the
        // current command-mask bit and acknowledge, without forcing producer
        // to follow the hardware cursor. The remaining remapped/A-MPDU shape
        // is still deliberately unsupported.
        if mmio.read_u8(slot_address.kind().get() as u32) == 0 && flags & (1 << 15) == 0 {
            let command_mask = mmio.read_u32(ring.cursor_and_pending_mask());
            mmio.write_u32(
                ring.cursor_and_pending_mask(),
                command_mask & !(1_u32 << current),
            );
            mmio.write_u32(
                PIPE_IRQ_PENDING,
                0_u32.wrapping_sub(pending_mask.wrapping_add(1)),
            );
            return SingleFrameRearmOutcome::CommandMaskAcknowledged;
        }
        backend.fatal_unsupported_rearm_shape(pipe, slot, frame_node);
    }

    // The vendor repeats descriptor-flag publication for a non-0xff frame
    // kind before clearing the command ownership bit.
    if mmio.read_u8(frame_node.raw() + 0x56) != 0xff {
        let descriptor_flags = mmio.read_u32(descriptor.flags()) | u32::from(status).wrapping_add(0x80);
        mmio.write_u32(descriptor.flags(), descriptor_flags);
    }
    let command_mask = mmio.read_u32(ring.cursor_and_pending_mask());
    mmio.write_u32(
        ring.cursor_and_pending_mask(),
        command_mask & !(1_u32 << current),
    );
    mmio.write_u32(
        PIPE_IRQ_PENDING,
        0_u32.wrapping_sub(pending_mask.wrapping_add(1)),
    );
    SingleFrameRearmOutcome::CommandMaskAcknowledged
}

/// Exact entry, inactive-pipe, and give-up portions of
/// `txp_pipe_tx_done_retry` (`0x9550`). The retry/rebuild tail is an infallible
/// backend leaf so a bounded Rust retry policy can replace vendor TALA without
/// changing hardware ownership transitions.
///
/// The supplied word is the scheduler value captured before bit-23 service.
/// This function remains inactive until its backend can execute every reachable
/// re-arm and fatal-quiescence operation.
pub fn execute_single_outstanding_tx_retry<M, B>(
    mmio: &mut M,
    scheduler_word: SchedulerWord,
    backend: &mut B,
) -> SingleTxRetryOutcome
where
    M: MacPipeMmio,
    B: SingleTxRetryBackend,
{
    let pipe = mmio.read_u8(crate::dtcm::MAC_CURRENT_PIPE.get() as u32) & 3;
    let record = pipe_record_address(pipe);
    mmio.write_u32(crate::dtcm::MAC_CURRENT_PIPE_RECORD.get() as u32, record.raw());
    let slot = current_slot(mmio, pipe);
    mmio.write_u32(crate::dtcm::MAC_CURRENT_SLOT.get() as u32, slot.raw());
    let owned_mask = 0x100_u32 << pipe;
    let pending = scheduler_word.raw();

    if pending & owned_mask == 0 {
        return SingleTxRetryOutcome::MaskNotOwned;
    }
    if mmio.read_u8(slot.state().get() as u32) != 3 {
        return SingleTxRetryOutcome::SlotNotStarted;
    }

    let frame_node = FrameNodeAddress::new(mmio.read_u32(slot.frame().get() as u32));
    let Some(ring) = retained_hardware_ring(mmio, pipe) else {
        backend.fatal_unsupported_multi_slot_retry(pipe, slot.raw(), frame_node);
    };
    // Vendor ownership transition: started -> retry-owned, then globally busy.
    mmio.write_u8(slot.state().get() as u32, 4);
    mmio.write_u8(crate::dtcm::LOW_MAC_PIPE_BUSY.get() as u32, 1);

    if mmio.read_u8(record.state().get() as u32) != 1 {
        for selected_pipe in 0..4_u8 {
            if pending & (0x100_u32 << selected_pipe) != 0 {
                let Some(selected_ring) = retained_hardware_ring(mmio, selected_pipe) else {
                    backend.fatal_unsupported_multi_slot_retry(pipe, slot.raw(), frame_node);
                };
                mmio.write_u32(
                    selected_ring.inactive_sentinel(),
                    PIPE_RETRY_INACTIVE_SENTINEL,
                );
            }
        }
        mmio.write_u32(
            PIPE_IRQ_PENDING,
            0_u32.wrapping_sub(owned_mask.wrapping_add(1)),
        );
        return SingleTxRetryOutcome::InactivePipeAcknowledged;
    }

    if mmio.read_u8(slot.expected_status().get() as u32) == 6 {
        backend.fatal_unsupported_multi_slot_retry(pipe, slot.raw(), frame_node);
    }
    if (mmio.read_u8(record.watchdog().get() as u32) as i8) < 1 {
        mmio.write_u8(record.watchdog().get() as u32, 1);
    }

    match backend.decide_retry(pipe, slot.raw(), frame_node) {
        SingleTxRetryDecision::Rearm => {
            backend.rearm_and_ack(pipe, slot.raw(), frame_node, owned_mask);
            SingleTxRetryOutcome::Rearmed
        }
        SingleTxRetryDecision::CompleteSuccess => {
            mmio.write_u32(PIPE_IRQ_TRIGGER, (1_u32 << pipe) << 25);
            let current = mmio.read_u8(record.current_slot().get() as u32);
            let last = mmio.read_u8(record.last_slot().get() as u32);
            backend.complete_success(pipe, frame_node, slot.raw());
            mmio.write_u32(ring.completion_word(), 1);
            let next = current.wrapping_add(1) & 3;
            mmio.write_u8(record.current_slot().get() as u32, next);
            mmio.write_u8(record.producer_slot().get() as u32, next);
            if current == last {
                mmio.write_u8(record.state().get() as u32, 0);
                mmio.write_u8(record.control().get() as u32, 0);
                mmio.write_u8(record.watchdog().get() as u32, 5);
            }
            mmio.write_u32(
                PIPE_IRQ_PENDING,
                0_u32.wrapping_sub(owned_mask.wrapping_add(0x10)),
            );
            SingleTxRetryOutcome::Completed
        }
        SingleTxRetryDecision::AwaitBlockAck => {
            // The aggregate remains owned in slot state 4 until the joined RX
            // lane consumes the matching BlockAck. Acknowledge only this retry
            // event; do not clear the hardware command or advance ring cursors.
            mmio.write_u32(
                PIPE_IRQ_PENDING,
                0_u32.wrapping_sub(owned_mask.wrapping_add(0x10)),
            );
            SingleTxRetryOutcome::AwaitingBlockAck
        }
        SingleTxRetryDecision::GiveUp => {
            // The trigger precedes completion and the command completion word
            // follows `txp_fn_2441`, exactly as in the non-status-6 branch.
            mmio.write_u32(PIPE_IRQ_TRIGGER, (1_u32 << pipe) << 25);
            let current = mmio.read_u8(record.current_slot().get() as u32);
            let last = mmio.read_u8(record.last_slot().get() as u32);
            backend.complete_give_up(pipe, frame_node, slot.raw(), 0x0b);
            mmio.write_u32(ring.completion_word(), 1);

            let next = current.wrapping_add(1) & 3;
            mmio.write_u8(record.current_slot().get() as u32, next);
            mmio.write_u8(record.producer_slot().get() as u32, next);
            if current == last {
                mmio.write_u8(record.state().get() as u32, 0);
                mmio.write_u8(record.control().get() as u32, 0);
                mmio.write_u8(record.watchdog().get() as u32, 5);
            }
            mmio.write_u32(
                PIPE_IRQ_PENDING,
                0_u32.wrapping_sub(owned_mask.wrapping_add(0x10)),
            );
            SingleTxRetryOutcome::GivenUp
        }
    }
}

/// Inactive volatile-MMIO wrapper for the bounded single-frame retry path.
///
/// # Safety
/// Pipe, slot, command, and frame-node ownership must match the saved word.
pub unsafe fn service_single_outstanding_tx_retry_inactive<B: SingleTxRetryBackend>(
    scheduler_word: SchedulerWord,
    backend: &mut B,
) -> SingleTxRetryOutcome {
    execute_single_outstanding_tx_retry(&mut VolatileMacPipeMmio, scheduler_word, backend)
}

/// Pure bit-24 decision model from `0x9f5c..0x9fc4`. It deliberately accepts
/// the scheduler word captured before bit-23 service; re-reading `0x09c00e84`
/// after service would change the vendor decision.
pub fn resolve_mac_status_event(
    event: MacEvent,
    snapshot: MacStatusSnapshot,
) -> Option<MacStatusResolution> {
    let status = event.completion_status()?;
    let pending_mask = snapshot.pre_service_scheduler_word.raw()
        & 0x100_u32.wrapping_shl(u32::from(snapshot.latched_pipe) & 0x1f);
    if pending_mask != 0 && matches!(status, 4 | 0x19) {
        return Some(MacStatusResolution {
            status,
            pending_mask,
            dispatch_ordinary: false,
            ordinary_completion_eligible: false,
            // The pending/status branch calls retry directly; unlike ordinary
            // status dispatch, it has no slot-state-three gate.
            direct_retry: true,
            next_mismatch_count: snapshot.mismatch_count,
            escalation_retry: false,
        });
    }

    let dispatch_ordinary = !(event.event_type == 0x39 && status == 6);
    let ordinary_completion_eligible = dispatch_ordinary
        && snapshot.pipe_active
        && snapshot.slot_expected_status == status
        && snapshot.slot_state == 3
        && !snapshot.global_busy;
    // This check follows ordinary dispatch and examines the active slot again,
    // while retaining the same pending mask derived from the pre-service word.
    let mismatch = event.pipe_service_marker
        && pending_mask != 0
        && snapshot.pipe_active
        && snapshot.slot_expected_status != status;
    let next_mismatch_count = if mismatch {
        snapshot.mismatch_count.wrapping_add(1)
    } else {
        snapshot.mismatch_count
    };
    Some(MacStatusResolution {
        status,
        pending_mask,
        dispatch_ordinary,
        ordinary_completion_eligible,
        direct_retry: false,
        next_mismatch_count,
        escalation_retry: mismatch && next_mismatch_count > 2,
    })
}

/// Infallible effect backend used after a destructive FIFO pop. A production
/// implementation may not reject or defer an effect once `pop_event` returned
/// a non-negative word. Its `Fatal` effect must enter the matching non-returning
/// postmortem path; `MacEventStopReason::Fatal` exists for host backends only.
pub trait MacEventBackend {
    fn pop_event(&mut self) -> i32;
    fn readiness(&mut self) -> i32;
    fn apply_effect(&mut self, event: MacEvent, effect: MacEventEffect);
    fn drain_tail(&mut self);
}

/// Infallible executor leaves for one event that has already been popped from
/// `0x09c00a20`. This is the production-shaped single-outstanding core: the
/// scheduler word is captured once after pipe-phase handling but before bit-23
/// service, then reused by bit-24 status/retry handling.
pub trait PoppedMacEventEffects {
    fn trace(&mut self, event: MacEvent);
    /// Bit 30 handling records the recoverable fault and returns so the caller
    /// keeps processing the event.
    ///
    /// This used to return `!` unconditionally, which meant a report-and-
    /// continue build suppressed the exception publish and then span forever in
    /// `enter_mac_fatal_quiescence`. The host saw no error at all while the
    /// firmware went silent: measured mid-traffic as `TXed` frozen with the
    /// driver holding buffers, and the counters MIB unreadable because the main
    /// loop was gone.
    fn fatal(&mut self, event: MacEvent);
    fn pipe_phase(&mut self, event: MacEvent, event_type: u8, phase: u8, latch: Option<u8>);
    fn capture_scheduler_word(&mut self) -> SchedulerWord;
    fn pipe_service(&mut self, event: MacEvent, saved_scheduler_word: SchedulerWord);
    fn tx_status(
        &mut self,
        event: MacEvent,
        status: u8,
        pipe_service_escalation: bool,
        saved_scheduler_word: SchedulerWord,
    );
    fn beacon(&mut self, event: MacEvent);
    fn sideband(&mut self, event: MacEvent);
    fn archive(&mut self, event: MacEvent);
}

/// Executes every independent effect of one destructively consumed event.
/// There is no rejection or deferral return path.
pub fn execute_popped_single_outstanding_event<B: PoppedMacEventEffects>(
    event: MacEvent,
    backend: &mut B,
) {
    backend.trace(event);
    if event.fatal_marker {
        backend.fatal(event);
    }
    if event.pipe_marker {
        backend.pipe_phase(event, event.event_type, event.phase, event.pipe_index());
    }

    let saved_scheduler_word = if event.pipe_service_marker || event.completion_marker {
        Some(backend.capture_scheduler_word())
    } else {
        None
    };
    if event.pipe_service_marker {
        backend.pipe_service(
            event,
            saved_scheduler_word.unwrap_or_else(|| unreachable!()),
        );
    }
    if let Some(status) = event.completion_status() {
        backend.tx_status(
            event,
            status,
            event.pipe_service_marker,
            saved_scheduler_word.unwrap_or_else(|| unreachable!()),
        );
    }
    if event.beacon_marker {
        backend.beacon(event);
    }
    if event.sideband_marker {
        backend.sideband(event);
    }
    backend.archive(event);
}

/// Concrete leaves required by the one-pipe, one-slot event adapter. Advanced
/// event types deliberately enter fatal quiescence instead of being ignored.
pub trait SingleOutstandingMacHardwareEffects:
    TxPolicy + TxStatusPolicy + SingleTxRetryBackend + PipeStartEffects + PipeSuccessEffects
{
    fn mismatch_count(&self, pipe: u8) -> u8;
    fn set_mismatch_count(&mut self, pipe: u8, value: u8);
    fn unsupported_mac_event(&mut self, event: MacEvent) -> core::convert::Infallible;
}

#[cfg(target_arch = "arm")]
struct InactiveSingleOutstandingEventAdapter<'a, B> {
    backend: &'a mut B,
}

#[cfg(target_arch = "arm")]
impl<B: SingleOutstandingMacHardwareEffects> PoppedMacEventEffects
    for InactiveSingleOutstandingEventAdapter<'_, B>
{
    fn trace(&mut self, event: MacEvent) {
        unsafe { trace_mac_event(event.raw) };
    }

    fn fatal(&mut self, event: MacEvent) {
        let saved = capture_pipe_scheduler_word(&mut VolatileMacPipeMmio);
        {
            let _ = saved;
            unsafe {
                crate::host_tx_diagnostics::bump(
                    crate::host_tx_diagnostics::counter::SUPPRESSED_EXCEPTION,
                );
            }
        }
    }

    #[allow(unreachable_code)]
    fn pipe_phase(&mut self, event: MacEvent, event_type: u8, phase: u8, latch: Option<u8>) {
        let selected_pipe = if event_type == 0x37 {
            let Some(pipe) = latch else {
                match self.backend.unsupported_mac_event(event) {}
            };
            unsafe { write_u8(crate::dtcm::MAC_CURRENT_PIPE.get(), pipe) };
            Some(pipe)
        } else {
            None
        };
        match phase {
            2 => {
                if event_type == 0x37 {
                    let pipe = selected_pipe.unwrap_or(0);
                    unsafe {
                        trace_tx_stage(TX_TRACE_PHASE2);
                        trace_tx_value(0x1c, event.raw);
                        write_u8(crate::dtcm::LOW_MAC_EVENT_PENDING.get(), 0);
                        service_pipe_tx_start(pipe, self.backend);
                    }
                } else {
                    unsafe { service_mac_irq_count_status(event_type) };
                }
            }
            3 | 1 if phase == 3 || event_type == 0x19 => unsafe {
                if read_u8(crate::dtcm::LOW_MAC_CONTROL_0A.get()) != 0 {
                    write_u8(crate::dtcm::LOW_MAC_CONTROL_0A.get(), 0);
                    let pending = crate::dtcm::scheduler_pending_events().get() as usize;
                    write_u32(pending, read_u32(pending) | 0x10);
                }
                if read_u8(crate::dtcm::LOW_MAC_EVENT_PENDING.get()) != 0 {
                    let index = usize::from(read_u8(crate::dtcm::LOW_MAC_SELECTED_RATE.get()));
                    let duration = packet_ram::duration_word(index);
                    write_u16(duration, read_u16(duration).wrapping_add(0x10) & !0x0f);
                }
                if event_type == 0x37 {
                    let pipe = selected_pipe.unwrap_or(0);
                    service_pipe_tx_success(pipe, self.backend);
                } else {
                    service_mac_nonpipe_completion_event(event_type);
                }
            },
            _ => {}
        }
    }

    fn capture_scheduler_word(&mut self) -> SchedulerWord {
        capture_pipe_scheduler_word(&mut VolatileMacPipeMmio)
    }

    fn pipe_service(&mut self, event: MacEvent, saved_scheduler_word: SchedulerWord) {
        unsafe {
            trace_tx_stage(TX_TRACE_BIT23);
            trace_tx_value(0x18, saved_scheduler_word.raw());
            trace_tx_value(0x20, event.raw);
        }
        if saved_scheduler_word.raw() & 0x0000_f0ff != 0 {
            unsafe { service_mac_pipe_irq_inactive(saved_scheduler_word, self.backend) };
        }
    }

    fn tx_status(
        &mut self,
        event: MacEvent,
        status: u8,
        pipe_service_escalation: bool,
        saved_scheduler_word: SchedulerWord,
    ) {
        let pipe = unsafe { read_u8(crate::dtcm::MAC_CURRENT_PIPE.get()) } & 3;
        let pending_mask = saved_scheduler_word.raw() & (0x100_u32 << pipe);
        if pending_mask != 0 && matches!(status, 4 | 0x19) {
            unsafe {
                service_single_outstanding_tx_retry_inactive(
                    SchedulerWord::new(pending_mask),
                    self.backend,
                )
            };
            return;
        }

        #[cfg(feature = "vendor-host-tx-diagnostics")]
        if event.event_type == 0x39 && status == 2 {
            unsafe {
                capture_status2_ownership(
                    event,
                    saved_scheduler_word,
                    self.backend.mismatch_count(pipe),
                    1,
                )
            };
        }
        if !(event.event_type == 0x39 && status == 6) {
            unsafe { service_mac_irq_tx_status_dispatch(status, self.backend) };
        }
        #[cfg(feature = "vendor-host-tx-diagnostics")]
        if event.event_type == 0x39 && status == 2 {
            unsafe {
                capture_status2_ownership(
                    event,
                    saved_scheduler_word,
                    self.backend.mismatch_count(pipe),
                    2,
                )
            };
        }
        if !pipe_service_escalation || pending_mask == 0 {
            return;
        }

        let record = pipe_record_address(pipe);
        if unsafe { read_u8(record.state().get()) } != 1 {
            return;
        }
        let current = unsafe { read_u8(record.current_slot().get()) };
        let slot = record.slot_unchecked(usize::from(current));
        if unsafe { read_u8(slot.expected_status().get()) } == status {
            return;
        }
        let mismatch = self.backend.mismatch_count(pipe).wrapping_add(1);
        self.backend.set_mismatch_count(pipe, mismatch);
        if mismatch > 2 {
            unsafe {
                service_single_outstanding_tx_retry_inactive(
                    SchedulerWord::new(pending_mask),
                    self.backend,
                )
            };
        }
    }

    fn beacon(&mut self, _event: MacEvent) {
        unsafe { service_mac_beacon_event() };
    }

    fn sideband(&mut self, _event: MacEvent) {
        unsafe { service_mac_sideband() };
    }

    fn archive(&mut self, event: MacEvent) {
        unsafe { archive_mac_event(event.raw) };
    }
}

/// Executes one already-popped event through the concrete one-pipe hardware
/// leaves. It remains disconnected from `0x09c00a20`.
///
/// # Safety
/// The caller must exclusively own all MAC event, pipe, and completion state.
#[cfg(target_arch = "arm")]
pub unsafe fn execute_popped_mac_event_inactive<B: SingleOutstandingMacHardwareEffects>(
    event: MacEvent,
    backend: &mut B,
) {
    execute_popped_single_outstanding_event(
        event,
        &mut InactiveSingleOutstandingEventAdapter { backend },
    );
}

#[cfg_attr(not(target_arch = "arm"), allow(dead_code))]
fn insert_scheduler_timer_list<M: MacPipeMmio>(mmio: &mut M, timer: u32, deadline: u32) -> bool {
    let mut previous_link = crate::dtcm::scheduler_timer_list_head().get() as u32;
    let mut next = mmio.read_u32(previous_link);
    while next != 0 && (mmio.read_u32(next + 8).wrapping_sub(deadline) as i32) <= 0 {
        previous_link = next;
        next = mmio.read_u32(next);
    }
    if next != 0 {
        mmio.write_u32(next + 4, timer);
    }
    mmio.write_u32(timer + 8, deadline);
    mmio.write_u32(previous_link, timer);
    mmio.write_u32(timer, next);
    mmio.write_u32(timer + 4, previous_link);
    mmio.read_u32(crate::dtcm::scheduler_timer_list_head().get() as u32) == timer
}

#[cfg_attr(not(target_arch = "arm"), allow(dead_code))]
fn unlink_scheduler_timer_list<M: MacPipeMmio>(mmio: &mut M, timer: u32) -> bool {
    let previous_link = mmio.read_u32(timer + 4);
    if previous_link == 0 {
        return false;
    }
    let next = mmio.read_u32(timer);
    mmio.write_u32(timer + 4, 0);
    mmio.write_u32(previous_link, next);
    if next != 0 {
        mmio.write_u32(next + 4, previous_link);
    }
    true
}

#[cfg_attr(not(target_arch = "arm"), allow(dead_code))]
fn claim_scheduler_mask<M: MacPipeMmio>(mmio: &mut M, mask: u32) -> u32 {
    let address = crate::dtcm::scheduler_pending_events().get() as u32; let pending = mmio.read_u32(address);
    let claimed = pending & mask;
    if claimed != 0 {
        mmio.write_u32(address, pending & !claimed);
    }
    claimed
}

#[cfg(target_arch = "arm")]
unsafe fn claim_scheduler_mask_atomic(mask: u32) -> u32 {
    let previous = unsafe { mask_irq_fiq_terminal() };
    let claimed = claim_scheduler_mask(&mut VolatileMacPipeMmio, mask);
    unsafe { restore_irq_fiq(previous) };
    claimed
}

#[cfg(target_arch = "arm")]
unsafe fn clear_scheduler_timer_event() {
    let previous = unsafe { mask_irq_fiq_terminal() };
    unsafe {
        let pending = read_u32(crate::dtcm::scheduler_runtime_flags().get()) & !0x10;
        write_u32(crate::dtcm::scheduler_runtime_flags().get(), pending);
        if pending == 0 {
            write_u32(crate::dtcm::scheduler_pending_events().get(), read_u32(crate::dtcm::scheduler_pending_events().get()) | 4);
        }
        restore_irq_fiq(previous);
    }
}

#[cfg(target_arch = "arm")]
unsafe fn cancel_scheduler_timer(timer: u32) -> bool {
    unsafe {
        let previous_link = read_u32(timer as usize + 4);
        if previous_link == 0 {
            return false;
        }
        if read_u32(crate::dtcm::scheduler_runtime_flags().get()) & 0x10 != 0 && read_u32(crate::dtcm::scheduler_timer_list_head().get()) == timer {
            clear_scheduler_timer_event();
        }

        let previous = mask_irq_fiq_terminal();
        let unlinked = unlink_scheduler_timer_list(&mut VolatileMacPipeMmio, timer);
        restore_irq_fiq(previous);
        unlinked
    }
}

/// Exact matching-payload `timer_start` (`0xf2aa`) for cooperative completion
/// timers.
#[cfg(target_arch = "arm")]
unsafe fn start_scheduler_timer(timer: u32, duration: u32) -> u8 {
    unsafe {
        if read_u32(timer as usize + 4) != 0 {
            let _ = cancel_scheduler_timer(timer);
        }
        let duration = if duration as i32 >= 0 { duration } else { 0 };
        let deadline = read_u32(0x0ac0_0004)
            .wrapping_add(crate::dtcm::initialized_timer_counter_ptr().read_volatile())
            .wrapping_add(duration);
        if publication_bisect_reached(5) {
            return 5;
        }

        if publication_bisect_reached(6) {
            let previous = mask_irq_fiq_terminal();
            restore_irq_fiq(previous);
            return 6;
        }

        let previous = mask_irq_fiq_terminal();
        let became_head = insert_scheduler_timer_list(&mut VolatileMacPipeMmio, timer, deadline);
        restore_irq_fiq(previous);
        if publication_bisect_reached(7) {
            return 7;
        }

        if became_head && read_u32(crate::dtcm::SCHEDULER_HARDWARE_TIMER_GUARD.get()) == 0 {
            write_u32(0x0ac0_001c, 0);
            write_u32(0x0ac0_0014, duration);
            write_u32(0x0ac0_001c, 0xc1);
        }
        if publication_bisect_reached(8) {
            return 8;
        }
        0
    }
}

const fn pipe_slot_state_index(pipe: u8, slot: u8) -> Option<usize> {
    if pipe < 4 && slot < 4 {
        Some(pipe as usize * 4 + slot as usize)
    } else {
        None
    }
}

fn retained_slot_state_index(pipe: u8, slot_raw: u32) -> Option<usize> {
    if pipe >= 4 {
        return None;
    }
    (0..4_u8).find_map(|slot| {
        let address = crate::dtcm::mac_pipe_slot_state_word_unchecked(
            usize::from(pipe),
            usize::from(slot),
        )
        .get() as u32;
        (address == slot_raw).then_some(usize::from(pipe) * 4 + usize::from(slot))
    })
}

fn ampdu_publication_indices(slot: u8, member_count: usize) -> Option<[usize; 4]> {
    if slot >= 4 || !(2..=4).contains(&member_count) {
        return None;
    }
    let mut indices = [usize::MAX; 4];
    indices[0] = usize::from(slot);
    let mut position = 1;
    for index in 0..4 {
        if index == usize::from(slot) {
            continue;
        }
        if position == member_count {
            break;
        }
        indices[position] = index;
        position += 1;
    }
    (position == member_count).then_some(indices)
}

fn publication_registration_allowed(
    occupied: [bool; 4],
    batch: BatchPosition,
    slot: u8,
) -> bool {
    if slot >= 4 || occupied[usize::from(slot)] {
        return false;
    }
    let count = occupied.into_iter().filter(|occupied| *occupied).count();
    let prior_slots_are_contiguous =
        (1..=count).all(|offset| occupied[(usize::from(slot) + 4 - offset) & 3]);
    match batch {
        BatchPosition::Only | BatchPosition::First => count == 0,
        BatchPosition::Middle => {
            cfg!(feature = "experimental-four-slot-ordinary")
                && (1..=2).contains(&count)
                && prior_slots_are_contiguous
        }
        BatchPosition::Last => {
            if cfg!(feature = "experimental-four-slot-ordinary") {
                (1..=3).contains(&count) && prior_slots_are_contiguous
            } else {
                count == 1
            }
        }
    }
}

#[cfg(all(target_arch = "arm", feature = "experimental-depth-two-ampdu"))]
#[derive(Clone, Copy)]
struct RetainedAmpduBlockAck {
    /// Exact CPU-form frame-node identities; zero terminates the member prefix.
    members: [u32; crate::host_tx_policy::MAX_EXPERIMENTAL_AMPDU_DEPTH],
    observation: PlannedBlockAck,
}

#[cfg(target_arch = "arm")]
pub struct SingleProbeMacBackend {
    retry: [BoundedSingleTxRetry; 16],
    mismatch: [u8; 4],
    selective_retry: [Option<FrameNodeAddress>; 16],
    partial_give_up: [Option<(FrameNodeAddress, FrameNodeAddress)>; 16],
    #[cfg(feature = "experimental-depth-two-ampdu")]
    depth_two_block_ack: [Option<RetainedAmpduBlockAck>; 16],
    publications: [Option<PublishedSlotIdentity>; 16],
    completed: BoundedCompletionQueue<HostClass0Completion, HOST_CLASS0_COMPLETION_CAPACITY>,
}

#[cfg(target_arch = "arm")]
impl SingleProbeMacBackend {
    pub const fn new(max_retries: u8) -> Self {
        Self {
            retry: [BoundedSingleTxRetry::new(max_retries); 16],
            mismatch: [0; 4],
            selective_retry: [None; 16],
            partial_give_up: [None; 16],
            #[cfg(feature = "experimental-depth-two-ampdu")]
            depth_two_block_ack: [None; 16],
            publications: [None; 16],
            completed: BoundedCompletionQueue::new(),
        }
    }

    fn register_publication(
        &mut self,
        batch: BatchPosition,
        context: ContextAddress,
        pipe: u8,
        slot: u8,
    ) -> bool {
        let Some(publication) = PublishedSlotIdentity::new(context, pipe, slot) else {
            return false;
        };
        let pipe_index = usize::from(pipe & 3);
        let Some(retry_index) = pipe_slot_state_index(pipe, slot) else {
            return false;
        };
        let occupied = core::array::from_fn(|slot| {
            self.publications[pipe_index * 4 + slot].is_some()
        });
        if !publication_registration_allowed(occupied, batch, slot) {
            return false;
        }
        self.retry[retry_index].reset();
        self.selective_retry[retry_index] = None;
        self.partial_give_up[retry_index] = None;
        #[cfg(feature = "experimental-depth-two-ampdu")]
        {
            self.depth_two_block_ack[retry_index] = None;
        }
        if matches!(batch, BatchPosition::Only | BatchPosition::First) {
            self.mismatch[pipe_index] = 0;
        }
        let publication_index = pipe_index * 4 + usize::from(slot & 3);
        self.publications[publication_index] = Some(publication);
        true
    }

    #[cfg(feature = "experimental-depth-two-ampdu")]
    fn register_ampdu_publications(
        &mut self,
        members: [Option<ContextAddress>; 4],
        pipe: u8,
        slot: u8,
    ) -> bool {
        let member_count = members.iter().take_while(|member| member.is_some()).count();
        if members[member_count..].iter().any(Option::is_some) {
            return false;
        }
        let Some(indices) = ampdu_publication_indices(slot, member_count) else {
            return false;
        };
        let pipe_index = usize::from(pipe & 3);
        let Some(retry_index) = pipe_slot_state_index(pipe, slot) else {
            return false;
        };
        if self.publications[pipe_index * 4..pipe_index * 4 + 4]
            .iter()
            .any(Option::is_some)
        {
            return false;
        }

        let mut publications = [None; 4];
        for position in 0..member_count {
            let Some(context) = members[position] else {
                return false;
            };
            let Some(publication) = PublishedSlotIdentity::new(context, pipe, slot) else {
                return false;
            };
            publications[position] = Some(publication);
        }

        self.retry[retry_index].reset();
        self.mismatch[pipe_index] = 0;
        self.selective_retry[retry_index] = None;
        self.partial_give_up[retry_index] = None;
        self.depth_two_block_ack[retry_index] = None;
        for position in 0..member_count {
            self.publications[pipe_index * 4 + indices[position]] = publications[position];
        }
        true
    }

    #[cfg(feature = "experimental-depth-two-ampdu")]
    fn register_ampdu_publication(
        &mut self,
        first: ContextAddress,
        second: ContextAddress,
        pipe: u8,
        slot: u8,
    ) -> bool {
        self.register_ampdu_publications(
            [Some(first), Some(second), None, None],
            pipe,
            slot,
        )
    }

    fn push_completion(&mut self, completion: HostClass0Completion) {
        if let Err(completion) = self.completed.push(completion) {
            terminal_probe_backend_fault(completion.pipe)
        }
    }

    pub fn take_completion(&mut self) -> Option<HostClass0Completion> {
        self.completed.take()
    }
}

#[cfg(target_arch = "arm")]
fn terminal_probe_backend_fault(pipe: u8) -> ! {
    let event = MacEvent {
        raw: (1 << 30) | (u32::from(pipe & 3) << 18),
        event_type: 0,
        pipe: pipe & 3,
        phase: 0,
        status: 0,
        pipe_marker: false,
        completion_marker: false,
        fatal_marker: true,
        pipe_service_marker: false,
        beacon_marker: false,
        sideband_marker: false,
    };
    let saved = capture_pipe_scheduler_word(&mut VolatileMacPipeMmio);
    unsafe { enter_mac_fatal_quiescence(event, saved) }
}

#[cfg(target_arch = "arm")]
impl TxPolicy for SingleProbeMacBackend {
    fn program_random_backoff(&mut self, _pipe: u8, descriptor: u32, frame_node: u32) {
        let mut mmio = VolatileMacPipeMmio;
        let context = FrameNodeAddress::new(frame_node).context();
        let interface = u32::from(mmio.read_u8(context.interface_address() as u32));
        let selector = u32::from(mmio.read_u8(context.access_category_address() as u32));
        let mask = mmio.read_u16(
            crate::dtcm::pas_stride_view_unchecked(interface as usize)
                .contention_window_unchecked(selector as usize)
                .get() as u32,
        );
        let random = (next_retry_random24(&mut mmio) as u16) & mask;
        let current = mmio.read_u32(descriptor + 4);
        mmio.write_u32(
            descriptor + 4,
            (u32::from(random) & 0x0fff) * 0x400 | (current & 0x01ff),
        );
    }
}

#[cfg(target_arch = "arm")]
impl PipeSlotCompletionEffects for SingleProbeMacBackend {
    fn link_set_state(&mut self, _link: u8, _state: u8) {}

    fn rate_recovery_on_success(&mut self, _frame_node: FrameNodeAddress, _status: u16) {}

    fn special_completion_gate(&mut self, _frame_node: FrameNodeAddress) -> bool {
        false
    }

    fn find_rx_frame_by_subtype(&mut self, _subtype: u8) -> Option<u32> {
        None
    }

    fn find_tx_pipe_by_mac_tid(&mut self, _interface: u8, _tid: u8, _mac: Option<u32>) -> u8 {
        8
    }

    fn process_ba_bitmap(&mut self, _pipe: u8) {}

    fn invalid_link_assertion(&mut self) -> core::convert::Infallible {
        terminal_probe_backend_fault(0)
    }
}

#[cfg(target_arch = "arm")]
impl TxStatusPolicy for SingleProbeMacBackend {
    fn reset_status_backoff(&mut self, _link: u8, _queue: u8, _pas_state: u32, _queue_table: u32) {}
}

#[cfg(target_arch = "arm")]
impl PipeSuccessEffects for SingleProbeMacBackend {
    fn set_frame_lifetime(&mut self, _frame_node: FrameNodeAddress) {}

    fn reset_backoff(&mut self, _link: u8, _queue: u8) {}
}

#[cfg(target_arch = "arm")]
fn record_nonfatal_backend_diagnostic(code: u32, pipe: u8) {
    unsafe { trace_tx_value(0x28, code | u32::from(pipe & 3)) };
}

#[cfg(target_arch = "arm")]
impl PipeStartEffects for SingleProbeMacBackend {
    fn start_without_pending_diagnostic(&mut self, pipe: u8) {
        // Vendor `txp_pipe_tx_start()` reports this through its trace helper
        // and returns. It is not an assertion or a terminal MAC fault.
        record_nonfatal_backend_diagnostic(0x5354_0000, pipe);
    }
}

#[cfg(target_arch = "arm")]
impl MessageCompletionEffects for SingleProbeMacBackend {
    fn completion_messages_enabled(&self) -> bool {
        // Vendor type-7 LMC messages feed the BA policy task on scheduler bit
        // 22. The single management-frame low-MAC intentionally has no BA
        // policy, so allocating them would fill the unserviced 16-entry ring
        // after fifteen otherwise legal completions.
        false
    }

    fn message_allocation_failed(&mut self) {
        // `lmc_msg_alloc()` records ring exhaustion and returns null. The
        // single-frame backend disables these messages, but retain the vendor
        // non-fatal behavior if that policy changes.
        record_nonfatal_backend_diagnostic(0x4c4d_0000, 0);
    }
}

#[cfg(target_arch = "arm")]
impl BaCompletionEffects for SingleProbeMacBackend {
    fn find_pipe_by_mac_upper(
        &mut self,
        _mac_upper: u32,
    ) -> Option<crate::dtcm::BaPipeObjectAddress> {
        None
    }
}

#[cfg(target_arch = "arm")]
impl RadioCompletionEffects for SingleProbeMacBackend {
    fn radio_owner_mismatch(&mut self) -> core::convert::Infallible {
        terminal_probe_backend_fault(0)
    }

    fn tbtt_post_process(&mut self, _interface: u8) {}
}

#[cfg(target_arch = "arm")]
impl PowerSaveCompletionEffects for SingleProbeMacBackend {
    fn completion_idle_policy_enabled(&self) -> bool {
        // Command 7 and radio-owner release depend on the vendor timer/task
        // scheduler. Leaving that policy half-executed poisons the next scan
        // generation, so the cooperative management-frame subset keeps the
        // already-active scan radio instead.
        false
    }

    fn timer_start(&mut self, timer: u32, duration: u32) {
        let _ = unsafe { start_scheduler_timer(timer, duration) };
    }

    fn send_pending_poll_or_qos_null(&mut self, _interface: u8) {}

    fn try_enter_sleep_all(&mut self) {}

    fn release_radio_if_all_idle(&mut self, _interface: u8) {}
}

#[cfg(target_arch = "arm")]
impl CompletionDrainEffects for SingleProbeMacBackend {
    fn complete_context(&mut self, context: ContextAddress, status: u16) {
        let dispatch = unsafe {
            dispatch_completed_context(
                context,
                status,
                |completion_class, context| {
                    if completion_class == 0 && true {
                        // Class-0 ownership lasts until inherited confirmation
                        // handoff; current HIF returns request credit before
                        // output enqueue. This does not match vendor lifetime.
                    } else if completion_class == 6 {
                        service_class6_probe_completion(context.raw());
                    } else {
                        terminal_probe_backend_fault(0);
                    }
                },
                || {},
            )
        };
        if dispatch == CompletedContextDispatch::Returned
            || ((true) && dispatch == CompletedContextDispatch::ClassZeroCallbackOwnsReturn)
        {
            let frame_node = context.frame_node();
            let Some((publication_index, publication)) = self
                .publications
                .iter()
                .enumerate()
                .find_map(|(index, publication)| {
                    publication
                        .filter(|publication| {
                            publication.context == context && publication.frame_node == frame_node
                        })
                        .map(|publication| (index, publication))
                })
            else {
                terminal_probe_backend_fault(0)
            };
            self.publications[publication_index] = None;
            let ack_failures = if let Some(host) = context.host() {
                unsafe { crate::dtcm::shared_ptr::<u16>(host.try_count()).read_volatile() }
                    .min(u16::from(u8::MAX)) as u8
            } else {
                let Some(retry_index) = pipe_slot_state_index(publication.pipe, publication.slot)
                else {
                    terminal_probe_backend_fault(publication.pipe)
                };
                self.retry[retry_index].attempts()
            };
            self.push_completion(HostClass0Completion {
                context: context.raw(),
                frame_node: frame_node.raw(),
                pipe: publication.pipe,
                slot: publication.slot,
                status,
                ack_failures,
            });
        } else {
            terminal_probe_backend_fault(0);
        }
    }
}

#[cfg(target_arch = "arm")]
impl SingleFrameRearmBackend for SingleProbeMacBackend {
    fn rebuild_rate_descriptor(&mut self, pipe: u8, slot: u32, frame_node: FrameNodeAddress) {
        let context = frame_node.context();
        if unsafe { prepare_host_frame_timing(context.raw()) }.is_err() {
            terminal_probe_backend_fault(pipe);
        }
        let command = unsafe { read_u32(slot as usize + 0x14) };
        if command == 0
            || unsafe { emit_host_frame_descriptor_at(context.raw(), command.wrapping_add(0x0c)) }
                .is_err()
        {
            terminal_probe_backend_fault(pipe);
        }
        unsafe {
            write_u32(
                context.control_bits_address(),
                read_u32(context.control_bits_address()) & !0x000c_0000,
            );
        }
    }

    fn fatal_unsupported_rearm_shape(
        &mut self,
        pipe: u8,
        _slot: u32,
        _frame_node: FrameNodeAddress,
    ) -> ! {
        terminal_probe_backend_fault(pipe)
    }
}

#[cfg(all(target_arch = "arm", feature = "experimental-depth-two-ampdu"))]
const WHOLE_AMPDU_RETRY_LIMIT: u16 = 2;

#[cfg(all(target_arch = "arm", feature = "experimental-depth-two-ampdu"))]
unsafe fn collect_ampdu_contexts(
    first_frame_node: FrameNodeAddress,
) -> Option<(
    [Option<ContextAddress>; crate::host_tx_policy::MAX_EXPERIMENTAL_AMPDU_DEPTH],
    usize,
)> {
    unsafe {
        let mut contexts: [
            Option<ContextAddress>;
            crate::host_tx_policy::MAX_EXPERIMENTAL_AMPDU_DEPTH
        ] = [None; crate::host_tx_policy::MAX_EXPERIMENTAL_AMPDU_DEPTH];
        let mut current = first_frame_node;
        let mut count = 0_usize;
        loop {
            if count == crate::host_tx_policy::MAX_EXPERIMENTAL_AMPDU_DEPTH
                || contexts[..count]
                    .iter()
                    .flatten()
                    .any(|context| context.frame_node() == current)
            {
                return None;
            }
            let context = current.context();
            contexts[count] = Some(context);
            count += 1;
            let next = read_u32(context.next_in_ampdu_address());
            if next == 0 {
                break;
            }
            current = FrameNodeAddress::from_raw(next)?;
        }
        (count >= 2).then_some((contexts, count))
    }
}

#[cfg(all(target_arch = "arm", feature = "experimental-depth-two-ampdu"))]
unsafe fn prepare_whole_ampdu_retry(
    first_frame_node: FrameNodeAddress,
) -> Option<[Option<ContextAddress>; crate::host_tx_policy::MAX_EXPERIMENTAL_AMPDU_DEPTH]> {
    unsafe {
        let (contexts, member_count) = collect_ampdu_contexts(first_frame_node)?;
        let first = contexts[0]?;
        let tid = read_u8(first.tid_address());
        let session_active = tid < 8
            && crate::configuration::operational_tx_ba_tids() & (1_u8 << tid) != 0;

        let mut next_rates = [None; crate::host_tx_policy::MAX_EXPERIMENTAL_AMPDU_DEPTH];
        for index in 0..member_count {
            let context = contexts[index]?;
            let host = context.host()?;
            let rate = read_u8(context.tx_rate_address());
            let policy = crate::rate_policy::get(read_u8(context.retry_policy_address()))?;
            let tries = read_u16(context.try_count_address());
            if tries >= WHOLE_AMPDU_RETRY_LIMIT {
                return None;
            }
            let control = read_u32(context.control_bits_address());
            let long_frame = (control & 0x7ff) >> 9 != 0;
            let crate::rate_policy::RetryStep::Rearm { rate: next_rate } =
                crate::rate_policy::retry_step(policy, rate, tries, long_frame)
            else {
                return None;
            };
            host.rate_try(usize::from(rate >> 3))?;
            next_rates[index] = Some(next_rate);
        }
        let shared_next_rate = next_rates[0]?;
        if !session_active
            || next_rates
                .iter()
                .take(member_count)
                .any(|rate| *rate != Some(shared_next_rate))
        {
            return None;
        }

        for index in 0..member_count {
            let context = contexts[index]?;
            let host = context.host()?;
            let rate = read_u8(context.tx_rate_address());
            let status_address = host.rate_try(usize::from(rate >> 3))?.get();
            let shift = u32::from((rate & 7) * 4);
            let status = read_u32(status_address);
            let attempts = (status >> shift) & 0x0f;
            if attempts < 0x0f {
                write_u32(
                    status_address,
                    (status & !(0x0f << shift)) | ((attempts + 1) << shift),
                );
            }
            let next_rate = next_rates[index]?;
            let flags = read_u32(context.control_bits_address());
            let rate_changed = next_rate != rate && (flags & 0x20 == 0 || next_rate > 13);
            write_u8(context.tx_rate_address(), if rate_changed { next_rate } else { rate });
            write_u32(
                context.control_bits_address(),
                flags
                    | 0x10
                    | 0x0008_0000
                    | if rate_changed && rate > 3 && next_rate < 4 {
                        0x0004_0000
                    } else {
                        0
                    },
            );
            write_u16(
                context.try_count_address(),
                read_u16(context.try_count_address()).wrapping_add(1),
            );
        }
        Some(contexts)
    }
}

#[cfg(all(target_arch = "arm", feature = "experimental-depth-two-ampdu"))]
unsafe fn rearm_whole_ampdu(
    pipe: u8,
    slot_raw: u32,
    first_frame_node: FrameNodeAddress,
    pending_mask: u32,
) -> Result<(), ProbeBuildError> {
    unsafe {
        if pipe >= 4 {
            return Err(ProbeBuildError::UnsupportedPublicationShape);
        }
        let record = pipe_record_address(pipe);
        let current = read_u8(record.current_slot().get());
        let Some(live_slot) = LiveTxSlot::from_mmio(
            &mut VolatileMacPipeMmio,
            pipe,
            current,
            slot_raw,
            Some(first_frame_node),
        ) else {
            return Err(ProbeBuildError::UnsupportedPublicationShape);
        };
        let slot = live_slot.address;
        let contexts = collect_ampdu_contexts(live_slot.frame_node)
            .map(|(contexts, _)| contexts)
            .ok_or(ProbeBuildError::UnsupportedPublicationShape)?;
        let command = live_slot.command;
        let descriptor_node = read_u32(slot.auxiliary().get());
        let Some(descriptor_index) = crate::dtcm::mac_software_record_node_index(descriptor_node)
        else {
            return Err(ProbeBuildError::UnsupportedPublicationShape);
        };
        // The transfer word at command +0x18 contains the MAC bus encoding,
        // not a CPU pointer. Keep the CPU-form packet-RAM address from the
        // exact software-record free-list node retained by this slot.
        let packet_record = read_u32(descriptor_node as usize + 4);
        if packet_ram::software_record_index(packet_record as usize) != Some(descriptor_index) {
            return Err(ProbeBuildError::UnsupportedPublicationShape);
        }
        prepare_host_ampdu(
            contexts.map(|context| context.map(ContextAddress::raw)),
            pipe,
            current,
            slot_raw,
            command,
            descriptor_node,
            packet_record,
        )?;

        let Some(ring) = TxHardwareRingAddress::for_pipe(
            pipe,
            read_u32(record.hardware_ring().get()),
        ) else {
            return Err(ProbeBuildError::UnsupportedPublicationShape);
        };
        // Retry ownership is already live in the hardware ring. Rebuild the
        // command in place, preserve slot state 4, trigger the pipe, then
        // release only this slot's command-mask bit. Re-running GO would
        // publish a second active owner.
        write_u8(slot.state().get(), 4);
        write_u32(PIPE_IRQ_TRIGGER as usize, (1_u32 << pipe) << 25);
        write_u32(
            ring.cursor_and_pending_mask() as usize,
            read_u32(ring.cursor_and_pending_mask() as usize) & !(1_u32 << current),
        );
        write_u32(
            PIPE_IRQ_PENDING as usize,
            0_u32.wrapping_sub(pending_mask.wrapping_add(1)),
        );
        Ok(())
    }
}

#[cfg(all(target_arch = "arm", feature = "experimental-depth-two-ampdu"))]
unsafe fn depth_two_block_ack_actions_for(
    first_frame_node: FrameNodeAddress,
    observation: RetainedAmpduBlockAck,
) -> Option<([BlockAckMemberAction; 2], [FrameNodeAddress; 2])> {
    unsafe {
        if observation.observation.member_count != 2 {
            return None;
        }
        let first = first_frame_node.context();
        let second_raw = read_u32(first.next_in_ampdu_address());
        if second_raw == 0 {
            return None;
        }
        let second_frame_node = FrameNodeAddress::from_raw(second_raw)?;
        let members = [first_frame_node, second_frame_node];
        if observation.members != [first_frame_node.raw(), second_frame_node.raw(), 0, 0] {
            return None;
        }
        let tid = read_u8(first.tid_address());
        let session_active = tid < 8
            && crate::configuration::operational_tx_ba_tids() & (1_u8 << tid) != 0;
        Some((
            plan_depth_two_block_ack_actions(
                [
                    observation.observation.states[0],
                    observation.observation.states[1],
                ],
                [true; 2],
                session_active,
            ),
            members,
        ))
    }
}

#[cfg(all(target_arch = "arm", feature = "experimental-depth-two-ampdu"))]
unsafe fn prepare_selective_member_retry(frame_node: FrameNodeAddress) -> bool {
    unsafe {
        let context = frame_node.context();
        let Some(host) = context.host() else { return false };
        let rate = read_u8(context.tx_rate_address());
        let Some(status_field) = host.rate_try(usize::from(rate >> 3)) else { return false };
        let Some(policy) = crate::rate_policy::get(read_u8(context.retry_policy_address())) else {
            return false;
        };
        let try_count = read_u16(context.try_count_address());
        let flags = read_u32(context.control_bits_address());
        let long_frame = (flags & 0x7ff) >> 9 != 0;
        let crate::rate_policy::RetryStep::Rearm { rate: next_rate } =
            crate::rate_policy::retry_step(policy, rate, try_count, long_frame)
        else {
            return false;
        };
        let shift = u32::from((rate & 7) * 4);
        let status = read_u32(status_field.get());
        let attempts = (status >> shift) & 0x0f;
        if attempts < 0x0f {
            write_u32(
                status_field.get(),
                (status & !(0x0f << shift)) | ((attempts + 1) << shift),
            );
        }
        let rate_changed = next_rate != rate && (flags & 0x20 == 0 || next_rate > 13);
        write_u8(context.tx_rate_address(), if rate_changed { next_rate } else { rate });
        write_u32(
            context.control_bits_address(),
            flags
                | 0x10
                | 0x0008_0000
                | if rate_changed && rate > 3 && next_rate < 4 {
                    0x0004_0000
                } else {
                    0
                },
        );
        write_u16(context.try_count_address(), try_count.wrapping_add(1));
        true
    }
}

#[cfg(all(target_arch = "arm", feature = "experimental-depth-two-ampdu"))]
unsafe fn enqueue_acknowledged_aggregate_member(frame_node: FrameNodeAddress) {
    unsafe {
        let context = frame_node.context();
        write_u32(
            context.ownership_bits_address(),
            read_u32(context.ownership_bits_address()) | 0x0c00,
        );
        let class_bits = ((read_u32(context.control_bits_address()) >> 18) & 0x0c) as u16;
        write_u16(
            context.auxiliary_state_address(),
            read_u16(context.auxiliary_state_address()) | 1 | class_bits,
        );
        write_u16(context.terminal_status_address(), 0);
        write_u32(context.completion_timestamp_address(), read_u32(0x0ac0_0004));
        enqueue_completion_frame_node(frame_node, || false);
    }
}

#[cfg(all(target_arch = "arm", feature = "experimental-depth-two-ampdu"))]
unsafe fn convert_depth_two_slot_to_selective_retry(
    link: u8,
    slot_raw: u32,
    missing: FrameNodeAddress,
) {
    unsafe {
        let slot = crate::dtcm::MacPipeSlotAddress::from_raw_unchecked(slot_raw);
        let descriptor = read_u32(slot.auxiliary().get());
        if descriptor != 0 {
            write_u32(descriptor as usize, read_u32(crate::dtcm::MAC_SOFTWARE_RECORDS.get()));
            write_u32(crate::dtcm::MAC_SOFTWARE_RECORDS.get(), descriptor);
        }
        write_u8(slot.kind().get(), 0);
        write_u8(slot.retry_rate().get(), read_u8(missing.context().retry_rate_address()));
        write_u8(slot.control_02().get(), 0);
        write_u32(slot.frame().get(), missing.raw());
        write_u32(slot.auxiliary().get(), 0);
        write_u32(missing.context().next_in_ampdu_address(), 0);
        if link < 8 {
            for member in 0..16 {
                write_u32(
                    crate::dtcm::MAC_AGGREGATE_SLOT_TABLES
                        .member_unchecked(usize::from(link), member)
                        .get(),
                    0,
                );
            }
        }
    }
}

#[cfg(target_arch = "arm")]
impl SingleTxRetryBackend for SingleProbeMacBackend {
    fn decide_retry(
        &mut self,
        pipe: u8,
        slot: u32,
        frame_node: FrameNodeAddress,
    ) -> SingleTxRetryDecision {
        let Some(retry_index) = retained_slot_state_index(pipe, slot) else {
            terminal_probe_backend_fault(pipe)
        };
        if unsafe {
            read_u8(
                crate::dtcm::MacPipeSlotAddress::from_raw_unchecked(slot)
                    .kind()
                    .get(),
            ) == 1
        } {
            #[cfg(feature = "experimental-depth-two-ampdu")]
            {
                let observation = self.depth_two_block_ack[retry_index].take();
                if let Some(observation) = observation
                    && observation.observation.member_count > 2
                {
                    let member_count = usize::from(observation.observation.member_count);
                    let total_miss = observation
                        .observation
                        .states
                        .iter()
                        .take(member_count)
                        .all(|state| *state == BlockAckMemberState::Missing);
                    return if total_miss
                        && unsafe { prepare_whole_ampdu_retry(frame_node) }.is_some()
                    {
                        self.retry[retry_index].record_rearm();
                        SingleTxRetryDecision::Rearm
                    } else {
                        SingleTxRetryDecision::GiveUp
                    };
                }
                if let Some((actions, members)) = observation.and_then(|observation| unsafe {
                    depth_two_block_ack_actions_for(frame_node, observation)
                }) {
                    if actions == [BlockAckMemberAction::Confirm; 2] {
                        return SingleTxRetryDecision::CompleteSuccess;
                    }
                    let member_retry_index = actions
                        .iter()
                        .position(|action| *action == BlockAckMemberAction::Retry);
                    let confirm_index = actions
                        .iter()
                        .position(|action| *action == BlockAckMemberAction::Confirm);
                    let give_up_index = actions
                        .iter()
                        .position(|action| *action == BlockAckMemberAction::GiveUp);
                    if let (Some(member_retry_index), Some(confirm_index)) =
                        (member_retry_index, confirm_index)
                        && actions.iter().all(|action| {
                            matches!(
                                action,
                                BlockAckMemberAction::Retry | BlockAckMemberAction::Confirm
                            )
                        })
                    {
                        if unsafe { prepare_selective_member_retry(members[member_retry_index]) } {
                            unsafe {
                                write_u32(members[confirm_index].context().next_in_ampdu_address(), 0);
                                enqueue_acknowledged_aggregate_member(members[confirm_index]);
                            }
                            self.selective_retry[retry_index] = Some(members[member_retry_index]);
                            self.retry[retry_index].record_rearm();
                            return SingleTxRetryDecision::Rearm;
                        }
                        self.partial_give_up[retry_index] =
                            Some((members[confirm_index], members[member_retry_index]));
                        return SingleTxRetryDecision::CompleteSuccess;
                    }
                    if let (Some(give_up_index), Some(confirm_index)) =
                        (give_up_index, confirm_index)
                        && actions.iter().all(|action| {
                            matches!(
                                action,
                                BlockAckMemberAction::GiveUp | BlockAckMemberAction::Confirm
                            )
                        })
                    {
                        self.partial_give_up[retry_index] =
                            Some((members[confirm_index], members[give_up_index]));
                        return SingleTxRetryDecision::CompleteSuccess;
                    }
                }
                // A missing or unusable bitmap retains the already-qualified
                // conservative whole-aggregate retry path.
                return if unsafe { prepare_whole_ampdu_retry(frame_node) }.is_some() {
                    self.retry[retry_index].record_rearm();
                    SingleTxRetryDecision::Rearm
                } else {
                    SingleTxRetryDecision::GiveUp
                };
            }
            #[cfg(not(feature = "experimental-depth-two-ampdu"))]
            return SingleTxRetryDecision::AwaitBlockAck;
        }
        let Some(context) = crate::dtcm::host_context_from_raw(
            frame_node.raw().wrapping_sub(FRAME_NODE_OFFSET),
        ) else {
            return self.retry[retry_index].decide();
        };

        let rate = unsafe { crate::dtcm::shared_ptr::<u8>(context.tx_rate()).read_volatile() };
        let Some(status_field) = context.rate_try(usize::from(rate >> 3)) else {
            return self.retry[retry_index].decide();
        };
        let status_address = status_field.get();
        let shift = u32::from((rate & 7) * 4);
        let status = unsafe { read_u32(status_address as usize) };
        let attempts = (status >> shift) & 0x0f;
        if attempts < 0x0f {
            unsafe {
                write_u32(
                    status_address as usize,
                    (status & !(0x0f << shift)) | ((attempts + 1) << shift),
                );
            }
        }

        let policy_index = unsafe {
            crate::dtcm::shared_ptr::<u8>(context.retry_policy()).read_volatile()
        };
        let Some(policy) = crate::rate_policy::get(policy_index) else {
            return self.retry[retry_index].decide();
        };
        let try_count = unsafe { crate::dtcm::shared_ptr::<u16>(context.try_count()).read_volatile() };
        let flags = unsafe { crate::dtcm::shared_ptr::<u32>(context.control_bits()).read_volatile() };
        let long_frame = (flags & 0x7ff) >> 9 != 0;
        match crate::rate_policy::retry_step(policy, rate, try_count, long_frame) {
            crate::rate_policy::RetryStep::GiveUp => SingleTxRetryDecision::GiveUp,
            crate::rate_policy::RetryStep::Rearm { rate: next_rate } => {
                let rate_changed = next_rate != rate && (flags & 0x20 == 0 || next_rate > 13);
                let first_retry = flags & 0x10 == 0;
                if first_retry || rate_changed {
                    unsafe {
                        if rate_changed {
                            crate::dtcm::shared_ptr::<u8>(context.tx_rate()).write_volatile(next_rate);
                        }
                        write_u32(
                            context.control_bits().get(),
                            flags
                                | 0x10
                                | 0x0008_0000
                                | if rate_changed && rate > 3 && next_rate < 4 {
                                    0x0004_0000
                                } else {
                                    0
                                },
                        );
                    }
                }
                unsafe {
                    crate::dtcm::shared_ptr::<u16>(context.try_count())
                        .write_volatile(try_count.wrapping_add(1));
                }
                self.retry[retry_index].record_rearm();
                SingleTxRetryDecision::Rearm
            }
        }
    }

    fn rearm_and_ack(
        &mut self,
        pipe: u8,
        slot: u32,
        frame_node: FrameNodeAddress,
        pending_mask: u32,
    ) {
        if pipe >= 4 {
            crate::halt_always!();
        }
        let record = pipe_record_address(pipe);
        let current = unsafe { read_u8(record.current_slot().get()) };
        let Some(live_slot) = LiveTxSlot::from_mmio(
            &mut VolatileMacPipeMmio,
            pipe,
            current,
            slot,
            Some(frame_node),
        ) else {
            terminal_probe_backend_fault(pipe);
        };
        #[cfg(feature = "experimental-depth-two-ampdu")]
        if unsafe { read_u8(live_slot.address.kind().get()) == 1 } {
            let Some(retry_index) = retained_slot_state_index(pipe, slot) else {
                terminal_probe_backend_fault(pipe)
            };
            if let Some(missing) = self.selective_retry[retry_index].take() {
                let link = unsafe { read_u8(missing.context().link_id_address()) };
                unsafe { convert_depth_two_slot_to_selective_retry(link, slot, missing) };
                execute_fixed_rate_single_frame_rearm(
                    &mut VolatileMacPipeMmio,
                    pipe,
                    slot,
                    missing,
                    pending_mask,
                    self,
                );
            } else if unsafe {
                rearm_whole_ampdu(pipe, slot, frame_node, pending_mask)
            }
            .is_err()
            {
                terminal_probe_backend_fault(pipe);
            }
            return;
        }
        execute_fixed_rate_single_frame_rearm(
            &mut VolatileMacPipeMmio,
            pipe,
            slot,
            frame_node,
            pending_mask,
            self,
        );
    }

    fn complete_success(&mut self, pipe: u8, frame_node: FrameNodeAddress, slot: u32) {
        if pipe >= 4 {
            crate::halt_always!();
        }
        let Some(aggregate) = release_aggregate_retry_command_mask(
            &mut VolatileMacPipeMmio,
            pipe,
            slot,
            frame_node,
        ) else {
            terminal_probe_backend_fault(pipe);
        };
        #[cfg(feature = "experimental-depth-two-ampdu")]
        let Some(retry_index) = retained_slot_state_index(pipe, slot) else {
            terminal_probe_backend_fault(pipe)
        };
        #[cfg(feature = "experimental-depth-two-ampdu")]
        if let Some((acknowledged, missing)) = self.partial_give_up[retry_index].take() {
            unsafe {
                crate::host_tx_diagnostics::bump(crate::host_tx_diagnostics::counter::GIVE_UP);
                write_u32(acknowledged.context().next_in_ampdu_address(), 0);
                enqueue_acknowledged_aggregate_member(acknowledged);
                let link = read_u8(missing.context().link_id_address());
                convert_depth_two_slot_to_selective_retry(link, slot, missing);
                complete_tx_pipe_slot(missing, slot, 0x0b, self);
            }
        } else {
            unsafe { complete_tx_pipe_slot(frame_node, slot, 0, self) };
        }
        #[cfg(not(feature = "experimental-depth-two-ampdu"))]
        unsafe {
            complete_tx_pipe_slot(frame_node, slot, 0, self);
        }
        if aggregate {
            unsafe { write_u8(crate::dtcm::LOW_MAC_PIPE_BUSY.get(), 0) };
        }
    }

    fn complete_give_up(
        &mut self,
        pipe: u8,
        frame_node: FrameNodeAddress,
        slot: u32,
        status: u16,
    ) {
        unsafe { crate::host_tx_diagnostics::bump(crate::host_tx_diagnostics::counter::GIVE_UP) };
        if pipe >= 4 {
            crate::halt_always!();
        }
        let Some(aggregate) = release_aggregate_retry_command_mask(
            &mut VolatileMacPipeMmio,
            pipe,
            slot,
            frame_node,
        ) else {
            terminal_probe_backend_fault(pipe);
        };
        unsafe { complete_tx_pipe_slot(frame_node, slot, status, self) };
        if aggregate {
            unsafe { write_u8(crate::dtcm::LOW_MAC_PIPE_BUSY.get(), 0) };
        }
    }

    fn fatal_unsupported_multi_slot_retry(
        &mut self,
        pipe: u8,
        _slot: u32,
        _frame_node: FrameNodeAddress,
    ) -> ! {
        terminal_probe_backend_fault(pipe)
    }
}

#[cfg(target_arch = "arm")]
impl SingleOutstandingMacHardwareEffects for SingleProbeMacBackend {
    fn mismatch_count(&self, pipe: u8) -> u8 {
        self.mismatch[usize::from(pipe & 3)]
    }

    fn set_mismatch_count(&mut self, pipe: u8, value: u8) {
        self.mismatch[usize::from(pipe & 3)] = value;
    }

    fn unsupported_mac_event(&mut self, event: MacEvent) -> core::convert::Infallible {
        let saved = capture_pipe_scheduler_word(&mut VolatileMacPipeMmio);
        unsafe { enter_mac_fatal_quiescence(event, saved) }
    }
}

/// Complete bounded destructive FIFO loop for the inactive one-probe backend.
/// Nothing in the firmware calls this function yet.
///
/// # Safety
/// The caller must exclusively own `0x09c00a20/24`, all MAC pipe state, and
/// completion processing for the entire call. A fatal event never returns.
#[cfg(target_arch = "arm")]
pub unsafe fn service_single_probe_mac_fifo_inactive(
    backend: &mut SingleProbeMacBackend,
    max_events: u32,
) -> MacEventLoopReport {
    if max_events == 0 {
        return MacEventLoopReport {
            processed: 0,
            stop: MacEventStopReason::BudgetExhausted,
            reschedule_required: true,
        };
    }

    let mut processed = 0_u32;
    let mut raw = unsafe { read_u32(crate::platform::mac_register(0x0a20)) } as i32;
    unsafe {
        trace_tx_value(0x0c, raw as u32);
        trace_tx_stage(TX_TRACE_POP);
    }
    loop {
        let Some(event) = MacEvent::decode(raw as u32) else {
            unsafe { service_mac_event_drain_tail() };
            return MacEventLoopReport {
                processed,
                stop: MacEventStopReason::Empty,
                reschedule_required: false,
            };
        };

        unsafe { execute_popped_mac_event_inactive(event, backend) };
        processed = processed.wrapping_add(1);
        if processed == max_events {
            return MacEventLoopReport {
                processed,
                stop: MacEventStopReason::BudgetExhausted,
                reschedule_required: true,
            };
        }
        if unsafe { read_u32(MAC_EVENT_READINESS as usize) as i32 } < 0 {
            unsafe { service_mac_event_drain_tail() };
            return MacEventLoopReport {
                processed,
                stop: MacEventStopReason::Empty,
                reschedule_required: false,
            };
        }
        raw = unsafe { read_u32(crate::platform::mac_register(0x0a20)) } as i32;
    }
}

/// Drains the snapshotted completion-ring prefix through class-6 callback and
/// context return for the concrete one-probe backend. Scheduler-bit ownership
/// remains with the future cooperative scheduler integration.
///
/// # Safety
/// Completion-ring and scheduler state must be exclusively owned.
#[cfg(target_arch = "arm")]
pub unsafe fn service_single_probe_completion_drain_inactive(backend: &mut SingleProbeMacBackend) {
    unsafe { service_completion_drain(backend) };
}

/// Claims and services scheduler bit 20 exactly once. Other scheduler tasks
/// remain untouched for the surrounding cooperative loop.
///
/// # Safety
/// Scheduler flags and completion state must be exclusively owned.
#[cfg(target_arch = "arm")]
pub unsafe fn service_single_probe_scheduler_inactive(backend: &mut SingleProbeMacBackend) -> bool {
    let claimed = unsafe { claim_scheduler_mask_atomic(1 << 20) } != 0;
    let queued = unsafe {
        let (consumer, producer) = COMPLETION_RING.cursors();
        consumer != producer
    };
    // The vendor scheduler dispatches bit 20 to drain this ring. In the
    // cooperative runtime, also treat a visibly non-empty ring as sufficient:
    // a concurrently raised scheduler bit can otherwise be consumed by an
    // unrelated translated task, stranding an already hardware-completed
    // class-0 context and its HIF token.
    if !claimed && !queued {
        return false;
    }
    unsafe { service_single_probe_completion_drain_inactive(backend) };
    true
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProbeTxRuntimeReport {
    pub events: Option<MacEventLoopReport>,
    pub completion_drained: bool,
    pub completion: Option<HostClass0Completion>,
}

/// One bounded cooperative service pass for an already-published probe. It
/// enters the destructive FIFO dispatcher only while the MAC FIQ source is
/// pending, then services completion bit 20 and reports a context only after
/// callback-driven return.
///
/// # Safety
/// The caller must exclusively own the active probe, MAC event FIFO, scheduler
/// bit 20, and completion ring.
pub const fn mac_fiq_pending(pending: u32) -> bool {
    pending & (1 << MAC_FIQ_SOURCE) != 0
}

const fn mac_service_pending(interrupt_pending: u32, fifo_readiness: u32) -> bool {
    mac_fiq_pending(interrupt_pending) || fifo_readiness as i32 >= 0
}

#[cfg(target_arch = "arm")]
unsafe fn reset_tx_trace(pipe: u8, slot: u8) {
    unsafe {
        *TX_EXEC_TRACE.0.get() = [
            TX_TRACE_MAGIC,
            1,
            TX_TRACE_PUBLISHED,
            u32::MAX,
            (u32::from(pipe) << 8) | u32::from(slot),
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ];
    }
}

#[cfg(target_arch = "arm")]
unsafe fn trace_tx_stage(stage: u32) {
    unsafe {
        let trace = &mut *TX_EXEC_TRACE.0.get();
        trace[2] |= stage;
        let debug = &mut *TX_DEBUG_SNAPSHOT.0.get();
        debug.values = [
            trace[2], trace[3], trace[5], trace[6], trace[7], trace[8], trace[9], trace[10],
        ];
        debug.next = 0;
        debug.valid = true;
    }
}

#[cfg(target_arch = "arm")]
unsafe fn trace_tx_value(offset: u32, value: u32) {
    unsafe {
        let index = usize::try_from(offset / 4).unwrap_or(0);
        if index < 12 {
            (*TX_EXEC_TRACE.0.get())[index] = value;
        }
    }
}

#[cfg(not(target_arch = "arm"))]
unsafe fn reset_tx_trace(_pipe: u8, _slot: u8) {}

#[cfg(not(target_arch = "arm"))]
unsafe fn trace_tx_stage(_stage: u32) {}

#[cfg(not(target_arch = "arm"))]
unsafe fn trace_tx_value(_offset: u32, _value: u32) {}

#[cfg(target_arch = "arm")]
pub unsafe fn service_single_probe_runtime_inactive(
    backend: &mut SingleProbeMacBackend,
    max_events: u32,
) -> ProbeTxRuntimeReport {
    // Vendor dispatcher `0x9e90` is entered through direct ARM FIQ source
    // 0x16. This firmware intentionally keeps CPU IRQ/FIQ masked and services
    // hardware cooperatively, so also admit a non-empty MAC event FIFO when
    // the interrupt-controller pending mirror does not expose the routed FIQ.
    // Once admitted, the first 0x09c00a20 pop remains unconditional and
    // 0x09c00a24 gates only subsequent pops, matching the vendor handler.
    let pending = unsafe { read_u32(INTERRUPT_PENDING) };
    let readiness = unsafe { read_u32(MAC_EVENT_READINESS as usize) };
    let events = if mac_service_pending(pending, readiness) {
        unsafe {
            trace_tx_value(0x14, pending);
            trace_tx_stage(TX_TRACE_FIQ);
        }
        Some(unsafe { service_single_probe_mac_fifo_inactive(backend, max_events) })
    } else {
        None
    };
    let completion_drained = unsafe { service_single_probe_scheduler_inactive(backend) };
    ProbeTxRuntimeReport {
        events,
        completion_drained,
        completion: backend.take_completion(),
    }
}

/// Irreversibly advance one reserved class-0 slot and trigger its MAC pipe.
/// All fallible descriptor/ring work must have completed before this boundary.
///
/// # Safety
/// The slot, command storage, pipe producer, and class-0 context must be
/// exclusively owned by the vendor-host runtime.
#[cfg(target_arch = "arm")]
pub unsafe fn publish_host_class0_slot(
    _guard: &mut crate::mac_domain::MacDomainGuard<'_>,
    context: u32,
    pipe: u8,
    slot: u8,
    slot_record: u32,
    command: u32,
    batch: BatchPosition,
) -> Result<(), ProbeBuildError> {
    let Some(host_context) = crate::dtcm::host_context_from_raw(context) else {
        return Err(ProbeBuildError::UnsupportedPublicationShape);
    };
    if pipe >= 4 || slot >= 4 {
        return Err(ProbeBuildError::UnsupportedPublicationShape);
    }
    let frame_node = FrameNodeAddress::new(host_context.frame_node().raw());
    if !single_frame_slot_matches(
        unsafe { read_u32(slot_record as usize + 0x0c) },
        frame_node.raw(),
        unsafe { read_u8(slot_record as usize) },
        unsafe { read_u8(slot_record as usize + 1) },
        unsafe { crate::dtcm::shared_ptr::<u8>(host_context.retry_rate()).read_volatile() },
    ) {
        return Err(ProbeBuildError::PipeSlotOwnershipMismatch);
    }
    let pipe_state = pipe_state_address(pipe);
    let hardware_ring = unsafe { read_u32(pipe_state as usize + 8) };
    let live_command = unsafe { read_u32(slot_record as usize + 0x14) };
    if command != live_command
        || packet_ram::tx_command_index(command as usize)
            != Some((usize::from(pipe), usize::from(slot)))
    {
        unsafe {
            crate::hif::publish_halting_exception(
                [
                    0x5458_4341,
                    u32::from(pipe),
                    u32::from(slot),
                    slot_record,
                    command,
                    live_command,
                    pipe_state,
                    hardware_ring,
                    read_u32(slot_record as usize),
                    read_u32(slot_record as usize + 8),
                    read_u32(slot_record as usize + 0x0c),
                    read_u32(slot_record as usize + 0x10),
                    read_u32(pipe_state as usize),
                    read_u32(pipe_state as usize + 4),
                    read_u32(pipe_state as usize + 8),
                    read_u32(pipe_state as usize + 0x20),
                    read_u32(pipe_state as usize + 0x38),
                    read_u32(pipe_state as usize + 0x50),
                ],
                b"xr819-tx-command-address",
            );
        }
        crate::halt_always!();
    }
    if TxHardwareRingAddress::for_pipe(pipe, hardware_ring).is_none() {
        return Err(ProbeBuildError::PipeStateUnavailable);
    }
    #[cfg(all(feature = "vendor-host-tx-diagnostics", target_arch = "arm"))]
    unsafe {
        crate::hif::validate_tx_boundary(0x12, pipe, slot, command, hardware_ring);
    }

    let runtime = unsafe { &mut *PROBE_EXPERIMENT.0.get() };
    if !runtime
        .backend
        .register_publication(batch, ContextAddress::new(context), pipe, slot)
    {
        return Err(ProbeBuildError::InvalidContextPointer);
    }
    unsafe {
        // `txq_build_aggregate_lists()` has already started command 1 before
        // scheduler reservation and descriptor construction. Publication must
        // not restart that asynchronous PHY transition at the GO boundary.
        // `tx_frame_done_release` claims global and per-VIF active-TX
        // accounting before PAS insertion. Completion drains the matching
        // counts after hardware ownership ends.
        // Retain the descriptor's PHY-length word for post-completion
        // diagnostics; the slot may be recycled before the host reads MIBs.
        crate::host_tx_diagnostics::capture_descriptor_length(read_u32(command as usize + 0x14));
        #[cfg(feature = "vendor-host-tx-diagnostics")]
        crate::radio::record_tx_command_signature(0, command);
        // Vendor `txp_fn_4425` asserts producer == hardware ring cursor. Record
        // both immediately before GO so a divergence is attributable to the
        // publication that consumed the stale slot.
        #[cfg(feature = "vendor-host-tx-diagnostics")]
        {
            let (packed, ring_word) = pipe_cursor_diagnostic(&mut VolatileMacPipeMmio, pipe);
            crate::host_tx_diagnostics::trace(0x4358_0001, packed, ring_word);
            crate::host_tx_diagnostics::capture_pipe_cursor(
                packed,
                !pipe_cursor_invariant_holds(packed),
            );
        }
        execute_single_probe_publication(
            &mut VolatileMacPipeMmio,
            SingleProbePublicationInput {
                pipe,
                slot,
                pipe_state,
                slot_record,
                command_storage: command,
                hardware_ring,
                frame_node,
                expects_ack: crate::dtcm::shared_ptr::<u8>(host_context.retry_rate())
                    .read_volatile()
                    != 0xff,
                batch,
            },
        );
    }
    Ok(())
}

#[cfg(all(target_arch = "arm", feature = "experimental-depth-two-ampdu"))]
pub unsafe fn prepare_host_ampdu(
    contexts: [Option<u32>; crate::host_tx_policy::MAX_EXPERIMENTAL_AMPDU_DEPTH],
    pipe: u8,
    slot: u8,
    slot_record: u32,
    command: u32,
    descriptor_node: u32,
    packet_record: u32,
) -> Result<(), ProbeBuildError> {
    let member_count = contexts.iter().take_while(|context| context.is_some()).count();
    if !(2..=crate::host_tx_policy::MAX_EXPERIMENTAL_AMPDU_DEPTH).contains(&member_count)
        || contexts[member_count..].iter().any(Option::is_some)
    {
        return Err(ProbeBuildError::UnsupportedPublicationShape);
    }

    let descriptor_index = crate::dtcm::mac_software_record_node_index(descriptor_node);
    if pipe >= 4
        || slot >= 4
        || packet_ram::tx_command_index(command as usize)
            != Some((usize::from(pipe), usize::from(slot)))
        || descriptor_index.is_none()
        || packet_ram::software_record_index(packet_record as usize) != descriptor_index
    {
        return Err(ProbeBuildError::UnsupportedPublicationShape);
    }

    let mut hosts = [None; crate::host_tx_policy::MAX_EXPERIMENTAL_AMPDU_DEPTH];
    let mut typed = [None; crate::host_tx_policy::MAX_EXPERIMENTAL_AMPDU_DEPTH];
    for index in 0..member_count {
        let raw = contexts[index].ok_or(ProbeBuildError::InvalidContextPointer)?;
        let host = crate::dtcm::host_context_from_raw(raw)
            .ok_or(ProbeBuildError::InvalidContextPointer)?;
        if host.expected_frame_state().raw()
            != unsafe { read_u32(host.frame_state_address().get()) }
        {
            return Err(ProbeBuildError::UnsupportedPublicationShape);
        }
        hosts[index] = Some(host);
        typed[index] = Some(ContextAddress::new(raw));
    }

    let first_host = hosts[0].ok_or(ProbeBuildError::UnsupportedPublicationShape)?;
    let first = typed[0].ok_or(ProbeBuildError::UnsupportedPublicationShape)?;
    let rate = unsafe { read_u8(first.tx_rate_address()) };
    let mut members = [None; crate::host_tx_policy::MAX_EXPERIMENTAL_AMPDU_DEPTH];
    let mut special_ack = false;
    for index in 0..member_count {
        let context = typed[index].ok_or(ProbeBuildError::UnsupportedPublicationShape)?;
        if unsafe { read_u8(context.tx_rate_address()) } != rate {
            return Err(ProbeBuildError::UnsupportedPublicationShape);
        }
        let frame_state = unsafe { read_u32(context.frame_state_address_address()) };
        let frame_length = unsafe { read_u16(context.frame_length_address()) };
        unsafe { emit_host_frame_descriptor_at(context.raw(), frame_state)? };
        special_ack |= unsafe { read_u32(context.control_bits_address()) } & 0x0030_0000 == 0;
        members[index] = Some(PlannedAmpduMember {
            frame_state,
            frame_length,
        });
    }

    let tx_flags = unsafe { read_u32(first.control_bits_address()) };
    let request_flag_rate_bits = unsafe { read_u8(first.request_flag_rate_bits_address()) };
    let legacy_mode = unsafe { read_u8(crate::dtcm::LOW_MAC_LEGACY_MODE.get()) };
    let rate_attribute = unsafe {
        crate::dtcm::rate_encoding_unchecked(usize::from(rate))
            .cast_mut::<u8>()
            .read_volatile()
    };
    let hardware_rate = unsafe {
        crate::dtcm::rate_attribute_unchecked(usize::from(rate))
            .cast_mut::<u8>()
            .read_volatile()
    };
    let phy = build_phy_rate_words(
        rate,
        legacy_mode,
        tx_flags,
        request_flag_rate_bits,
        rate_attribute,
    );
    let first_length = members[0]
        .ok_or(ProbeBuildError::UnsupportedPublicationShape)?
        .frame_length;
    let descriptor = build_planned_ampdu_descriptor(PlannedAmpduInput {
        members,
        phy_rate_word: phy.rate,
        phy_control_word: finalize_phy_control(phy, rate, first_length),
        hardware_rate,
        spacing_selector: ampdu_spacing_selector(
            crate::configuration::mpdu_start_spacing(),
            rate,
        ),
    })
    .ok_or(ProbeBuildError::UnsupportedPublicationShape)?;

    unsafe {
        for index in 0..16_u32 {
            write_u32(command as usize + index as usize * 4, 0);
        }
        build_single_frame_duration(
            &mut VolatileMacPipeMmio,
            command,
            first.frame_node(),
            special_ack,
        );
        if special_ack {
            write_u32(command as usize + 4, read_u32(command as usize + 4) | 0x8c);
        }
        for (index, word) in descriptor.phy_words.into_iter().enumerate() {
            write_u32(command as usize + 0x0c + index * 4, word);
        }
        write_u32(command as usize + 0x18, ampdu_transfer_word(packet_record as usize));
        for (index, word) in descriptor
            .words
            .iter()
            .take(usize::from(descriptor.length))
            .copied()
            .enumerate()
        {
            write_u32(packet_record as usize + index * 4, word);
        }
        let slot = crate::dtcm::MacPipeSlotAddress::from_raw_unchecked(slot_record);
        write_u8(slot.kind().get(), 1);
        write_u8(slot.retry_rate().get(), if special_ack { 0x0c } else { 0xff });
        write_u8(slot.control_02().get(), u8::from(special_ack));
        write_u8(slot.state().get(), 0);
        write_u32(slot.frame().get(), first_host.pas().raw());
        write_u32(slot.auxiliary().get(), descriptor_node);
        let aggregate_airtime = u32::from(read_u16(first.payload_extended_address()))
            .wrapping_add(0xb4)
            .wrapping_add(0x20);
        write_u32(
            slot.duration().get(),
            (u32::from(read_u16(first.payload_extended_address())) << 15)
                + if special_ack { 0x20b4 } else { 0 },
        );
        write_u32(first.word_48_address(), aggregate_airtime);
        for index in 0..member_count {
            let context = typed[index].ok_or(ProbeBuildError::UnsupportedPublicationShape)?;
            let next = if index + 1 == member_count {
                0
            } else {
                hosts[index + 1]
                    .ok_or(ProbeBuildError::UnsupportedPublicationShape)?
                    .pas()
                    .raw()
            };
            write_u32(context.next_in_ampdu_address(), next);
        }
    }
    Ok(())
}

#[cfg(all(target_arch = "arm", feature = "experimental-depth-two-ampdu"))]
pub unsafe fn prepare_depth_two_host_ampdu(
    first_context: u32,
    second_context: u32,
    pipe: u8,
    slot: u8,
    slot_record: u32,
    command: u32,
    descriptor_node: u32,
    packet_record: u32,
) -> Result<(), ProbeBuildError> {
    unsafe {
        prepare_host_ampdu(
            [Some(first_context), Some(second_context), None, None],
            pipe,
            slot,
            slot_record,
            command,
            descriptor_node,
            packet_record,
        )
    }
}

#[cfg(all(target_arch = "arm", feature = "experimental-depth-two-ampdu"))]
pub unsafe fn publish_depth_two_host_ampdu(
    first_context: u32,
    second_context: u32,
    pipe: u8,
    slot: u8,
) -> Result<(), ProbeBuildError> {
    if pipe >= 4 || slot >= 4 {
        return Err(ProbeBuildError::UnsupportedPublicationShape);
    }
    let pipe_state = pipe_state_address(pipe);
    let hardware_ring = unsafe { read_u32(pipe_state as usize + 8) };
    let Some(ring) = TxHardwareRingAddress::for_pipe(pipe, hardware_ring) else {
        return Err(ProbeBuildError::PipeStateUnavailable);
    };
    let runtime = unsafe { &mut *PROBE_EXPERIMENT.0.get() };
    if !runtime.backend.register_ampdu_publication(
        ContextAddress::new(first_context),
        ContextAddress::new(second_context),
        pipe,
        slot,
    ) {
        return Err(ProbeBuildError::UnsupportedPublicationShape);
    }
    unsafe {
        // Vendor `txq_build_aggregate_lists` enters PHY operation 1 before it
        // selects and publishes any aggregate-capable PAS contexts.
        if start_phy_operation_1() != 0 {
            return Err(ProbeBuildError::UnsupportedPublicationShape);
        }
        let timestamp = read_u32(0x0ac0_0004);
        for context in [ContextAddress::new(first_context), ContextAddress::new(second_context)] {
            write_u32(
                context.ownership_bits_address(),
                read_u32(context.ownership_bits_address()) | 0x180,
            );
            write_u32(context.scheduler_timestamp_address(), timestamp);
        }
        let first = ContextAddress::new(first_context);
        let record = crate::dtcm::MacPipeRecordAddress::from_raw_unchecked(pipe_state);
        write_u8(record.current_slot().get(), slot);
        write_u8(record.last_slot().get(), slot);
        write_u32(ring.go() as usize, 0);

        // Common tail of vendor `txp_scheduler_run` after every descriptor
        // shape: publish the interface EDCA timing and the selected pipe's
        // duration quantum before the trigger/GO boundary.
        let interface = usize::from(read_u8(first.interface_address()));
        let pas = crate::dtcm::pas_stride_view_unchecked(interface);
        let edca_slot_timing = read_u32(pas.packed_aifs().get());
        let edca_slot_timing_cache = crate::dtcm::mac_edca_slot_timing_ptr() as usize;
        if read_u32(edca_slot_timing_cache) != edca_slot_timing {
            write_u32(crate::platform::mac_register(0x0e64), edca_slot_timing);
            write_u32(edca_slot_timing_cache, edca_slot_timing);
        }
        let queue = usize::from(read_u8(
            crate::dtcm::queue_to_access_category_unchecked(usize::from(pipe)).get(),
        ));
        let mut quantum = u32::from(read_u16(pas.txop_limit_unchecked(queue).get()));
        let airtime = read_u32(first.word_48_address()) & 0xffff;
        if quantum == 0 {
            if (read_u32(first.control_bits_address()) & 0x0fff) >> 10 != 0 {
                quantum = airtime;
            }
        } else if quantum <= airtime {
            write_u16(
                first.auxiliary_state_address(),
                read_u16(first.auxiliary_state_address()) | 8,
            );
            quantum = airtime;
        }
        let quantum_destination = read_u32(
            crate::dtcm::duration_quantum_pointer_unchecked(usize::from(pipe)).get(),
        );
        write_u32(quantum_destination as usize, quantum.wrapping_add(0x1f) >> 5);
        if !finalize_staged_pipe(
            &mut VolatileMacPipeMmio,
            pipe,
            pipe_state,
            hardware_ring,
            slot,
            slot,
        ) {
            return Err(ProbeBuildError::PipeStateUnavailable);
        }
        crate::host_tx_diagnostics::bump(crate::host_tx_diagnostics::counter::PUBLISHED);
        crate::host_tx_diagnostics::bump(crate::host_tx_diagnostics::counter::PUBLISHED);
    }
    Ok(())
}

/// Exclusive capability for consuming the shared MAC event FIFO.
///
/// The startup runtime creates one token and passes it mutably to whichever
/// subsystem currently owns MAC servicing.
pub struct MacEventQueue {
    _private: (),
}

impl MacEventQueue {
    /// # Safety
    /// Exactly one token may exist for the live MAC event FIFO.
    pub const unsafe fn claim() -> Self {
        Self { _private: () }
    }
}

/// Cooperatively service MAC events and class-0 completion for the ordinary
/// vendor-host runtime.
#[cfg(target_arch = "arm")]
pub unsafe fn service_host_class0_runtime(
    _events: &mut MacEventQueue,
    max_events: u32,
) -> Option<HostClass0Completion> {
    let runtime = unsafe { &mut *PROBE_EXPERIMENT.0.get() };
    unsafe { service_single_probe_runtime_inactive(&mut runtime.backend, max_events) }.completion
}

/// Drain another completion produced by the most recent class-0 MAC service.
/// A single vendor completion-ring pass may retire every slot in a batch.
#[cfg(target_arch = "arm")]
pub unsafe fn take_host_class0_completion() -> Option<HostClass0Completion> {
    let runtime = unsafe { &mut *PROBE_EXPERIMENT.0.get() };
    runtime.backend.take_completion()
}

/// Compact read-only snapshot of the bounded class-0 MAC backend.
///
/// Bits 0..7 contain the maximum retry attempts across all pipes, bit 8
/// reports a queued completion, and bits 16..31 contain its internal status
/// when present.
#[cfg(target_arch = "arm")]
pub unsafe fn host_class0_runtime_diagnostic() -> u32 {
    let runtime = unsafe { &*PROBE_EXPERIMENT.0.get() };
    u32::from(
        runtime
            .backend
            .retry
            .iter()
            .map(|retry| retry.attempts())
            .max()
            .unwrap_or(0),
    )
        | runtime
            .backend
            .completed
            .first()
            .copied()
            .map(|completion| (1 << 8) | (u32::from(completion.status) << 16))
            .unwrap_or(0)
}

/// Exact bounded loop shape from vendor FIQ handler `0x9e90..0xa038`.
///
/// Budget exhaustion occurs before another readiness read or destructive pop,
/// and does not run the empty-FIFO drain tail. The production MMIO backend is
/// deliberately absent until every effect executor is translated.
pub fn service_mac_event_backend<B: MacEventBackend>(
    backend: &mut B,
    max_events: u32,
) -> MacEventLoopReport {
    if max_events == 0 {
        return MacEventLoopReport {
            processed: 0,
            stop: MacEventStopReason::BudgetExhausted,
            reschedule_required: true,
        };
    }

    let mut processed = 0_u32;
    let mut raw = backend.pop_event();
    loop {
        let Some(event) = MacEvent::decode(raw as u32) else {
            backend.drain_tail();
            return MacEventLoopReport {
                processed,
                stop: MacEventStopReason::Empty,
                reschedule_required: false,
            };
        };
        for effect in event.dispatch_plan().effects().iter().flatten().copied() {
            backend.apply_effect(event, effect);
            if effect == MacEventEffect::Fatal {
                return MacEventLoopReport {
                    processed: processed.wrapping_add(1),
                    stop: MacEventStopReason::Fatal,
                    reschedule_required: false,
                };
            }
        }
        processed = processed.wrapping_add(1);
        if processed == max_events {
            return MacEventLoopReport {
                processed,
                stop: MacEventStopReason::BudgetExhausted,
                reschedule_required: true,
            };
        }
        if backend.readiness() < 0 {
            backend.drain_tail();
            return MacEventLoopReport {
                processed,
                stop: MacEventStopReason::Empty,
                reschedule_required: false,
            };
        }
        raw = backend.pop_event();
    }
}

impl ProbeTxTracker {
    pub const fn new() -> Self {
        Self {
            ownership: ProbeTxOwnership::Idle,
            latched_pipe: None,
            observed_status: None,
            generation: 0,
        }
    }

    pub fn ownership(&self) -> ProbeTxOwnership {
        self.ownership
    }

    pub fn observed_status(&self) -> Option<u8> {
        self.observed_status
    }

    pub fn identity(&self) -> Option<ProbeTxIdentity> {
        let (context, pipe, slot) = match self.ownership {
            ProbeTxOwnership::Prepared { context } => (context, None, None),
            ProbeTxOwnership::PipeOwned {
                context,
                pipe,
                slot,
            }
            | ProbeTxOwnership::Started {
                context,
                pipe,
                slot,
            }
            | ProbeTxOwnership::RetryRequired {
                context,
                pipe,
                slot,
                ..
            }
            | ProbeTxOwnership::TerminalObserved {
                context,
                pipe,
                slot,
                ..
            }
            | ProbeTxOwnership::CompletionQueued {
                context,
                pipe,
                slot,
                ..
            }
            | ProbeTxOwnership::CallbackRunning {
                context,
                pipe,
                slot,
                ..
            } => (context, Some(pipe), Some(slot)),
            ProbeTxOwnership::FatalQuiesced {
                context: Some(context),
                pipe,
                slot,
            } => (context, pipe, slot),
            ProbeTxOwnership::Idle
            | ProbeTxOwnership::Returned
            | ProbeTxOwnership::FatalQuiesced { context: None, .. } => return None,
        };
        Some(ProbeTxIdentity {
            context,
            pipe,
            slot,
            generation: self.generation,
        })
    }

    pub fn owned_pipe(&self) -> Option<u8> {
        match self.ownership {
            ProbeTxOwnership::PipeOwned { pipe, .. }
            | ProbeTxOwnership::Started { pipe, .. }
            | ProbeTxOwnership::RetryRequired { pipe, .. }
            | ProbeTxOwnership::TerminalObserved { pipe, .. }
            | ProbeTxOwnership::CompletionQueued { pipe, .. }
            | ProbeTxOwnership::CallbackRunning { pipe, .. } => Some(pipe),
            ProbeTxOwnership::FatalQuiesced { pipe, .. } => pipe,
            ProbeTxOwnership::Idle
            | ProbeTxOwnership::Prepared { .. }
            | ProbeTxOwnership::Returned => None,
        }
    }

    pub fn prepare(&mut self, context: u32) -> Result<(), ProbeTxTransitionError> {
        if self.ownership != ProbeTxOwnership::Idle {
            return Err(ProbeTxTransitionError::Busy);
        }
        self.latched_pipe = None;
        self.observed_status = None;
        self.generation = self.generation.wrapping_add(1);
        self.ownership = ProbeTxOwnership::Prepared { context };
        Ok(())
    }

    pub fn publish(&mut self, pipe: u8, slot: u8) -> Result<(), ProbeTxTransitionError> {
        let ProbeTxOwnership::Prepared { context } = self.ownership else {
            return Err(ProbeTxTransitionError::NotPrepared);
        };
        self.ownership = ProbeTxOwnership::PipeOwned {
            context,
            pipe,
            slot,
        };
        Ok(())
    }

    /// Applies only the pipe events needed by the probe path. Other MAC events
    /// remain owned by the future complete `mac_irq_handler` translation.
    pub fn handle_pipe_event(
        &mut self,
        event: MacEvent,
        _pipe_pending: bool,
    ) -> Result<bool, ProbeTxTransitionError> {
        let (context, pipe, slot, started) = match self.ownership {
            ProbeTxOwnership::PipeOwned {
                context,
                pipe,
                slot,
            } => (context, pipe, slot, false),
            ProbeTxOwnership::Started {
                context,
                pipe,
                slot,
            } => (context, pipe, slot, true),
            _ => return Ok(false),
        };
        if (event.is_pipe_start() || event.is_pipe_success()) && event.pipe != pipe {
            return Err(ProbeTxTransitionError::WrongPipe);
        }
        if event.is_pipe_start() {
            self.latched_pipe = event.pipe_index();
            self.observed_status = None;
            self.ownership = ProbeTxOwnership::Started {
                context,
                pipe,
                slot,
            };
            return Ok(true);
        }
        if event.is_pipe_success() {
            if !started {
                return Err(ProbeTxTransitionError::CompletionBeforeStart);
            }
            self.ownership = ProbeTxOwnership::TerminalObserved {
                context,
                pipe,
                slot,
                status: 0,
            };
            self.observed_status = Some(0);
            return Ok(true);
        }
        if let Some(status) = event.completion_status() {
            if self.latched_pipe != Some(pipe) {
                return Err(ProbeTxTransitionError::WrongPipe);
            }
            self.observed_status = Some(status);
            return Ok(true);
        }
        Ok(false)
    }

    pub fn resolve_retry_status(&mut self, status: u8) -> bool {
        let ProbeTxOwnership::Started {
            context,
            pipe,
            slot,
        } = self.ownership
        else {
            return false;
        };
        if self.observed_status != Some(status) {
            return false;
        }
        self.ownership = ProbeTxOwnership::RetryRequired {
            context,
            pipe,
            slot,
            status,
        };
        true
    }

    pub fn resolve_terminal_status(&mut self, status: u8) -> bool {
        let ProbeTxOwnership::Started {
            context,
            pipe,
            slot,
        } = self.ownership
        else {
            return false;
        };
        if self.observed_status != Some(status) {
            return false;
        }
        self.ownership = ProbeTxOwnership::TerminalObserved {
            context,
            pipe,
            slot,
            status,
        };
        true
    }

    pub fn enter_fatal_quiescence(&mut self) {
        let identity = self.identity();
        self.observed_status = None;
        self.ownership = ProbeTxOwnership::FatalQuiesced {
            context: identity.map(|identity| identity.context),
            pipe: identity.and_then(|identity| identity.pipe),
            slot: identity.and_then(|identity| identity.slot),
        };
    }

    pub fn reset_fatal_quiescence(&mut self) -> bool {
        if !matches!(self.ownership, ProbeTxOwnership::FatalQuiesced { .. }) {
            return false;
        }
        self.latched_pipe = None;
        self.observed_status = None;
        self.ownership = ProbeTxOwnership::Idle;
        true
    }

    pub fn queue_terminal_completion(&mut self) -> bool {
        let ProbeTxOwnership::TerminalObserved {
            context,
            pipe,
            slot,
            status,
        } = self.ownership
        else {
            return false;
        };
        self.ownership = ProbeTxOwnership::CompletionQueued {
            context,
            pipe,
            slot,
            status,
        };
        true
    }

    pub fn begin_completion_callback(&mut self) -> bool {
        let ProbeTxOwnership::CompletionQueued {
            context,
            pipe,
            slot,
            status,
        } = self.ownership
        else {
            return false;
        };
        self.ownership = ProbeTxOwnership::CallbackRunning {
            context,
            pipe,
            slot,
            status,
        };
        true
    }

    /// Runs the final context-return operation exactly once and records the
    /// returned state only after the callback path has completed.
    pub fn finish_completion_callback<F>(&mut self, return_context: F) -> bool
    where
        F: FnOnce(u32),
    {
        self.try_finish_completion_callback(|context| {
            return_context(context);
            true
        })
    }

    pub fn try_finish_completion_callback<F>(&mut self, return_context: F) -> bool
    where
        F: FnOnce(u32) -> bool,
    {
        let ProbeTxOwnership::CallbackRunning { context, .. } = self.ownership else {
            return false;
        };
        if !return_context(context) {
            return false;
        }
        self.latched_pipe = None;
        self.observed_status = None;
        self.ownership = ProbeTxOwnership::Returned;
        true
    }

    pub fn reset_returned(&mut self) -> bool {
        if self.ownership != ProbeTxOwnership::Returned {
            return false;
        }
        self.ownership = ProbeTxOwnership::Idle;
        self.observed_status = None;
        true
    }
}

impl Default for ProbeTxTracker {
    fn default() -> Self {
        Self::new()
    }
}

fn completion_device_address(interface: u8) -> usize {
    crate::vif::mode_address(interface).unwrap_or(0)
}

fn completion_retry_limit_offset(alternate: bool) -> usize {
    if alternate { 0x115 } else { 0x114 }
}

fn completion_drain_required(
    context_completion_class: u8,
    frame_control: u16,
    special_gate_nonzero: bool,
) -> bool {
    context_completion_class != 0 || frame_control & 8 == 0 || !special_gate_nonzero
}

// `evt_flags_set` at `0xf034` masks both IRQ and FIQ around this exact
// read-modify-write, then restores the prior CPSR. A naked volatile RMW loses
// scheduler bits when foreground, IRQ, and FIQ paths interleave.
#[cfg(all(target_arch = "arm", not(target_feature = "thumb-mode")))]
unsafe fn raise_scheduler_bits(bits: u32) {
    unsafe {
        core::arch::asm!(
            "mrs r1, cpsr",
            "orr r2, r1, #0xc0",
            "msr cpsr_c, r2",
            "ldr r2, [r0]",
            "orr r2, r2, r3",
            "str r2, [r0]",
            "msr cpsr_c, r1",
            in("r0") crate::dtcm::scheduler_pending_events().get(),
            in("r3") bits,
            lateout("r1") _,
            lateout("r2") _,
            options(nostack),
        );
    }
}

#[cfg(all(target_arch = "arm", target_feature = "thumb-mode"))]
core::arch::global_asm!(
    ".syntax unified",
    ".pushsection .text.xr819_raise_scheduler_bits, \"ax\", %progbits",
    ".arm",
    ".align 2",
    ".global xr819_raise_scheduler_bits",
    ".type xr819_raise_scheduler_bits, %function",
    "xr819_raise_scheduler_bits:",
    "mrs r1, cpsr",
    "orr r2, r1, #0xc0",
    "msr cpsr_c, r2",
    "ldr r2, =0x04001fd4",
    "ldr r3, [r2]",
    "orr r3, r3, r0",
    "str r3, [r2]",
    "msr cpsr_c, r1",
    "bx lr",
    ".size xr819_raise_scheduler_bits, .-xr819_raise_scheduler_bits",
    ".popsection",
    ".thumb",
);

#[cfg(all(target_arch = "arm", target_feature = "thumb-mode"))]
unsafe fn raise_scheduler_bits(bits: u32) {
    unsafe extern "C" {
        fn xr819_raise_scheduler_bits(bits: u32);
    }
    unsafe { xr819_raise_scheduler_bits(bits) }
}

// Host tests cannot mask a CPU's interrupt state. Keep their MMIO model
// ordered, while target firmware always uses the CPSR-masked implementation.
#[cfg(not(target_arch = "arm"))]
unsafe fn raise_scheduler_bits(bits: u32) {
    use core::sync::atomic::{Ordering, compiler_fence};

    compiler_fence(Ordering::SeqCst);
    unsafe {
        let pending = crate::dtcm::shared_ptr::<u32>(crate::dtcm::scheduler_pending_events());
        pending.write_volatile(pending.read_volatile() | bits);
    }
    compiler_fence(Ordering::SeqCst);
}

/// Exact completion-ring enqueue at matching-payload `0xcfb8`.
///
/// `special_gate` represents vendor helper `0xebe8`, which is consulted only
/// for the rare `context+0x53 == 0 && frame-control bit 3` branch. The callback
/// returns the helper's nonzero/zero result. Probe contexts set `context+0x53`
/// to six and therefore never invoke it.
///
/// # Safety
/// The frame node must be valid, completion-ring state must be initialized,
/// and the caller must exclusively own completion enqueue and scheduler state.
pub unsafe fn enqueue_completion_frame_node<F>(frame_node: FrameNodeAddress, special_gate: F)
where
    F: FnOnce() -> bool,
{
    unsafe {
        let context = frame_node.context();
        COMPLETION_RING.enqueue(frame_node);

        let flags = context.ownership_bits_address() as *mut u32;
        flags.write_volatile(flags.read_volatile() | (1 << 14));
        if (context.terminal_status_address() as *const u16).read_volatile() == 0x16 {
            let completion_flags = context.auxiliary_state_address() as *mut u16;
            completion_flags.write_volatile(completion_flags.read_volatile() | 2);
        } else {
            raise_scheduler_bits(1 << 21);
        }

        let completion_class = (context.completion_class_address() as *const u8).read_volatile();
        let frame_control = (context.frame_control_address() as *const u16).read_volatile();
        let special_gate_nonzero = if completion_class == 0 && frame_control & 8 != 0 {
            special_gate()
        } else {
            false
        };
        let should_raise_drain =
            completion_drain_required(completion_class, frame_control, special_gate_nonzero);
        if should_raise_drain {
            raise_scheduler_bits(1 << 20);
        }
    }
}

/// Atomically couples the software ownership transition to the vendor
/// completion-ring enqueue. A terminal context cannot be enqueued twice.
///
/// # Safety
/// Same requirements as [`enqueue_completion_frame_node`].
pub unsafe fn enqueue_probe_terminal_completion<F>(
    tracker: &mut ProbeTxTracker,
    frame_node: FrameNodeAddress,
    special_gate: F,
) -> bool
where
    F: FnOnce() -> bool,
{
    let ProbeTxOwnership::TerminalObserved { context, .. } = tracker.ownership else {
        return false;
    };
    if frame_node.context().raw() != context {
        return false;
    }
    unsafe { enqueue_completion_frame_node(frame_node, special_gate) };
    tracker.queue_terminal_completion()
}

/// Snapshot cursor for the matching-payload `0xd1fc` completion drain. The
/// vendor snapshots the producer once and drains exactly that prefix; entries
/// enqueued concurrently are left for the next scheduler-bit-20 pass.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CompletionDrainCursor {
    next: u32,
    target: u32,
}

impl CompletionDrainCursor {
    /// # Safety
    /// Completion-ring state must be initialized and exclusively owned by the
    /// completion consumer.
    pub unsafe fn begin() -> Self {
        unsafe {
            let (next, target) = COMPLETION_RING.cursors();
            Self { next, target }
        }
    }

    pub const fn is_empty(self) -> bool {
        self.next == self.target
    }

    /// Destructively removes the next frame node from the snapshotted prefix.
    ///
    /// # Safety
    /// The same ownership requirements as [`Self::begin`] apply. Every
    /// returned frame node must run the complete callback and context-return
    /// chain before the cursor is discarded.
    pub unsafe fn pop(&mut self) -> Option<FrameNodeAddress> {
        if self.is_empty() {
            return None;
        }
        unsafe {
            let (frame_node, next) = COMPLETION_RING.pop(self.next);
            self.next = next;
            Some(frame_node)
        }
    }
}

pub trait CompletionDrainEffects {
    fn complete_context(&mut self, context: ContextAddress, status: u16);
}

pub trait MessageCompletionEffects {
    fn completion_messages_enabled(&self) -> bool;
    fn message_allocation_failed(&mut self);
}

pub trait BaCompletionEffects {
    fn find_pipe_by_mac_upper(
        &mut self,
        mac_upper: u32,
    ) -> Option<crate::dtcm::BaPipeObjectAddress>;
}

#[cfg(test)] fn lmc_message_address(index: u8) -> u32 {
    crate::dtcm::lmc_message_unchecked(usize::from(index)).raw()
}

fn lmc_vif_address(interface: usize) -> usize {
    crate::vif::record_address(interface as u8).unwrap_or(0)
}

fn infallible_to_never(value: core::convert::Infallible) -> ! {
    match value {}
}

/// Exact fixed-ring message allocation at matching-payload `0x5c5a`.
///
/// `allocation_failed` represents the vendor diagnostic call
/// `0xfff019c8(9, 0)`.
///
/// # Safety
/// FIQ/IRQ exclusion equivalent to the vendor save/restore pair must be held,
/// and the message ring at `0x04008ad8` must be initialized.
pub unsafe fn allocate_lmc_message<F>(allocation_failed: F) -> Option<crate::dtcm::LmcMessageAddress>
where
    F: FnOnce(),
{
    unsafe {
        let producer_address = crate::dtcm::lmc_message_producer().get();
        let producer = read_u8(producer_address).wrapping_add(1) & 0x0f;
        if read_u8(crate::dtcm::lmc_message_consumer().get()) == producer {
            allocation_failed();
            return None;
        }
        write_u8(producer_address, producer);
        Some(crate::dtcm::lmc_message_unchecked(usize::from(producer)))
    }
}

/// Exact status-`0x0b` BA transition at matching-payload `0x6dae`.
///
/// # Safety
/// `context` and the pointer returned by `find_pipe` must reference valid
/// vendor records.
pub unsafe fn mark_ba_session_state_5<F>(context: ContextAddress, find_pipe: F)
where
    F: FnOnce(u32) -> Option<crate::dtcm::BaPipeObjectAddress>,
{
    unsafe {
        if read_u16(crate::dtcm::LOW_MAC_OPTIONAL_PIPE_OBJECT_WORD.get()) == 0 {
            return;
        }
        let header = read_u32(context.frame_address_address());
        let Some(pipe) = find_pipe(header.wrapping_add(4)) else {
            return;
        };
        let state = read_u8(pipe.state().get());
        if state & 4 == 0 {
            write_u8(pipe.state().get(), 5);
        }
    }
}

/// Exact inter-VIF radio release at matching-payload `0x699a`.
///
/// The assertion callback must diverge, matching the vendor fatal path.
///
/// # Safety
/// `owner` and every scheduler/VIF record reached from it must be valid and
/// exclusively owned by the cooperative scheduler.
pub trait RadioCompletionEffects {
    fn radio_owner_mismatch(&mut self) -> core::convert::Infallible;
    fn tbtt_post_process(&mut self, interface: u8);
}

pub trait PipeSlotCompletionEffects {
    fn link_set_state(&mut self, link: u8, state: u8);
    fn rate_recovery_on_success(&mut self, frame_node: FrameNodeAddress, status: u16);
    fn special_completion_gate(&mut self, frame_node: FrameNodeAddress) -> bool;
    fn find_rx_frame_by_subtype(&mut self, subtype: u8) -> Option<u32>;
    fn find_tx_pipe_by_mac_tid(&mut self, interface: u8, tid: u8, mac: Option<u32>) -> u8;
    fn process_ba_bitmap(&mut self, pipe: u8);
    fn invalid_link_assertion(&mut self) -> core::convert::Infallible;
}

pub trait PipeSuccessEffects: PipeSlotCompletionEffects {
    fn set_frame_lifetime(&mut self, frame_node: FrameNodeAddress);
    fn reset_backoff(&mut self, link: u8, queue: u8);
}

/// Exact command-3 path through `phy_state_cmd_dispatch`. Inline switch entry
/// three (`0x0f`) targets Thumb `0x16fa0`.
///
/// # Safety
/// PHY command/output and global state records must be valid and exclusively
/// owned.
pub unsafe fn dispatch_phy_command_3() {
    unsafe {
        debug_assert_eq!(phy_dispatch_switch_target(3), 0x0001_6fa0);
        let output = crate::dtcm::MAC_PHY_DISPATCH_OUTPUT.get();
        let global_state = crate::dtcm::phy_retained_state().get();
        write_u8(crate::dtcm::MAC_PHY_DISPATCH_COMMAND.get(), 3);
        if read_u8(global_state) == 4 {
            write_u8(
                crate::dtcm::MAC_PHY_DISPATCH_OUTPUT_FLAGS.get(),
                read_u8(crate::dtcm::MAC_PHY_DISPATCH_OUTPUT_FLAGS.get()) | 2,
            );
            write_u8(global_state, 5);
        }
        write_u8(output, read_u8(global_state));
        write_u32(crate::dtcm::MAC_PHY_DISPATCH_OUTPUT_TIMEOUT.get(), 0x0098_9680);
    }
}

/// Exact command-2 path through `phy_state_cmd_dispatch`, including the
/// `0x19fc4 -> 0x19f90` PHY register sequence. Inline switch entry two
/// (`0x08`) targets Thumb `0x16f92`.
///
/// # Safety
/// PHY command/output, global state, and all listed MMIO blocks must be valid
/// and exclusively owned.
pub unsafe fn dispatch_phy_command_2(secondary: u8) {
    unsafe {
        trace_tx_stage(TX_TRACE_PHY2);
        trace_tx_value(0x28, u32::from(secondary));
        debug_assert_eq!(phy_dispatch_switch_target(2), 0x0001_6f92);
        let output = crate::dtcm::MAC_PHY_DISPATCH_OUTPUT.get();
        let global_state = crate::dtcm::phy_retained_state().get();
        write_u8(crate::dtcm::MAC_PHY_DISPATCH_COMMAND.get(), 2);
        write_u8(crate::dtcm::mac_phy_dispatch_command_byte_unchecked(1).get(), secondary);
        if read_u8(global_state) != 5 {
            write_u8(crate::dtcm::phy_measurement_control().get(), secondary);
            write_u32(0x0abb_8004, read_u32(0x0abb_8004) | 0x800);
            write_u32(0x0abb_8014, 0x0014_00c8);
            write_u32(0x0abb_8010, 0x0014_00c8);
            for address in [0x0abb_8440, 0x0abb_8444, 0x0abb_8448, 0x0abb_844c] {
                write_u32(address, 0x0014_00c8);
            }
            let control = read_u32(0x0abb_800c);
            write_u32(0x0abb_800c, control & !0x3040);
            write_u32(0x0abb_800c, control | 0x00e4_0e20);
            write_u8(global_state, 4);
        }
        write_u8(output, read_u8(global_state));
        write_u32(crate::dtcm::MAC_PHY_DISPATCH_OUTPUT_TIMEOUT.get(), 0x0098_9680);
    }
}

/// Exact command-1 path through `pac_phy_start_op` (`0x7f10`). Vendor TX
/// scheduling executes this before every queue/pipe pass so the pipe-start
/// event can enter command 2 and program the TX PHY.
///
/// # Safety
/// PHY operation state and the cooperative timer list must be exclusively
/// owned.
#[cfg(target_arch = "arm")]
pub unsafe fn start_phy_operation_1() -> u8 {
    unsafe {
        debug_assert_eq!(phy_dispatch_switch_target(1), 0x0001_6f8c);
        let output = crate::dtcm::MAC_PHY_OPERATION_OUTPUT.get();
        let global_state = crate::dtcm::phy_retained_state().get();
        write_u8(crate::dtcm::MAC_PHY_OPERATION_COMMAND.get(), 1);
        write_u8(crate::dtcm::mac_phy_dispatch_command_byte_unchecked(1).get(), 0);
        if read_u8(global_state) != 5 {
            write_u8(global_state, 3);
        }
        write_u8(output, read_u8(global_state));
        write_u32(crate::dtcm::MAC_PHY_OPERATION_TIMEOUT.get(), 0x0098_9680);
        write_u32(
            crate::dtcm::MAC_PHY_OPERATION_STATE.get(),
            u32::from(read_u8(output)),
        );
        if publication_bisect_reached(4) {
            return 4;
        }
        start_scheduler_timer(
            crate::dtcm::MAC_PHY_OPERATION_TIMER.get() as u32,
            read_u32(crate::dtcm::MAC_PHY_OPERATION_TIMEOUT.get()),
        )
    }
}

/// Exact matching-payload `txp_fn_2441` at Ghidra `0x91ec` (r2 `0x91c8`).
///
/// # Safety
/// `first_frame_node`, `slot`, every linked frame node, descriptor free-list,
/// completion ring, and BA/link records must be valid and exclusively owned.
#[allow(unreachable_code)]
pub unsafe fn complete_tx_pipe_slot<B: PipeSlotCompletionEffects>(
    first_frame_node: FrameNodeAddress,
    slot: u32,
    status: u16,
    backend: &mut B,
) {
    unsafe {
        let slot = crate::dtcm::MacPipeSlotAddress::from_raw_unchecked(slot);
        let timestamp = read_u32(0x0ac0_0004);
        let mut final_link_state = 10_u8;
        let link = read_u8(first_frame_node.context().link_id_address());
        let slot_kind = read_u8(slot.kind().get());

        if slot_kind != 0 {
            if slot_kind != 1 {
                write_u8(
                    crate::dtcm::LOW_MAC_ACTIVE_TX_COUNT.get(),
                    read_u8(crate::dtcm::LOW_MAC_ACTIVE_TX_COUNT.get()).wrapping_sub(1),
                );
                return;
            }
            if link < 8 {
                backend.link_set_state(link, 10);
            }
            let descriptor = read_u32(slot.auxiliary().get());
            write_u32(descriptor as usize, read_u32(crate::dtcm::MAC_SOFTWARE_RECORDS.get()));
            write_u32(crate::dtcm::MAC_SOFTWARE_RECORDS.get(), descriptor);
        }

        backend.rate_recovery_on_success(first_frame_node, status);
        write_u32(slot.frame().get(), 0);

        let mut frame_node = first_frame_node;
        loop {
            let context = frame_node.context();
            if slot_kind == 1 {
                write_u32(
                    context.ownership_bits_address(),
                    read_u32(context.ownership_bits_address()) | 0x400,
                );
                write_u16(
                    context.auxiliary_state_address(),
                    read_u16(context.auxiliary_state_address()) | 1,
                );
            }
            let class_bits = ((read_u32(context.control_bits_address()) >> 18) & 0x0c) as u16;
            write_u16(
                context.auxiliary_state_address(),
                read_u16(context.auxiliary_state_address()) | class_bits,
            );
            write_u16(context.terminal_status_address(), status);
            write_u32(context.completion_timestamp_address(), timestamp);

            if slot_kind == 1 {
                if status == 0 {
                    raise_scheduler_bits(1 << 21);
                } else if status == 0x0b {
                    final_link_state = 0x0b;
                }
                #[cfg(feature = "experimental-depth-two-ampdu")]
                if context.host().is_some() {
                    write_u32(
                        context.ownership_bits_address(),
                        read_u32(context.ownership_bits_address()) | 0x800,
                    );
                    enqueue_completion_frame_node(frame_node, || {
                        backend.special_completion_gate(frame_node)
                    });
                }
            } else {
                write_u32(
                    context.ownership_bits_address(),
                    read_u32(context.ownership_bits_address()) | 0x800,
                );
                enqueue_completion_frame_node(frame_node, || {
                    backend.special_completion_gate(frame_node)
                });
            }

            let next = read_u32(context.next_in_ampdu_address());
            if next == 0 {
                break;
            }
            frame_node = FrameNodeAddress::new(next);
        }

        #[cfg(feature = "experimental-depth-two-ampdu")]
        if slot_kind == 1 && first_frame_node.context().host().is_some() {
            if link < 8 {
                for member in 0..16 {
                    write_u32(
                        crate::dtcm::MAC_AGGREGATE_SLOT_TABLES
                            .member_unchecked(usize::from(link), member)
                            .get(),
                        0,
                    );
                }
            }
            write_u8(
                crate::dtcm::LOW_MAC_ACTIVE_TX_COUNT.get(),
                read_u8(crate::dtcm::LOW_MAC_ACTIVE_TX_COUNT.get()).wrapping_sub(1),
            );
            return;
        }

        if slot_kind == 1 {
            if link < 8 {
                backend.link_set_state(link, final_link_state);
            }
            if final_link_state != 0x0b {
                if let Some(frame) = backend.find_rx_frame_by_subtype(0x94) {
                    let frame = frame as usize;
                    let mut selected_pipe = 8_u8;
                    for interface in 0..2_u8 {
                        let record =
                            crate::dtcm::pas_stride_view_unchecked(usize::from(interface));
                        if read_u16(frame + 4) == read_u16(record.own_mac_byte_unchecked(0).get())
                            && read_u16(frame + 6)
                                == read_u16(record.own_mac_byte_unchecked(2).get())
                            && read_u16(frame + 8)
                                == read_u16(record.own_mac_byte_unchecked(4).get())
                        {
                            let device = lmc_vif_address(usize::from(interface));
                            let mac = if read_u32(device + 0x1c) & 4 != 0 {
                                Some((frame + 10) as u32)
                            } else {
                                None
                            };
                            selected_pipe = backend.find_tx_pipe_by_mac_tid(
                                interface,
                                (read_u16(frame + 0x10) >> 12) as u8,
                                mac,
                            );
                            break;
                        }
                    }
                    write_u32(0xfff0_2e48, read_u32(0xfff0_2e48).wrapping_add(1));
                    if selected_pipe < 8 && selected_pipe == link {
                        let selected_index = usize::from(selected_pipe);
                        if read_u8(crate::dtcm::ba_pipe_activity_unchecked(selected_index).get()) > 4 {
                            write_u32(
                                crate::dtcm::ba_pipe_bitmap_low_unchecked(selected_index).get(),
                                read_u32(frame + 0x14),
                            );
                            write_u32(
                                crate::dtcm::ba_pipe_bitmap_high_unchecked(selected_index).get(),
                                read_u32(frame + 0x18),
                            );
                            write_u16(
                                crate::dtcm::ba_pipe_sequence_unchecked(selected_index).get(),
                                read_u16(frame + 0x12) >> 4,
                            );
                            backend.process_ba_bitmap(selected_pipe);
                        }
                        write_u8(
                            crate::dtcm::LOW_MAC_ACTIVE_TX_COUNT.get(),
                            read_u8(crate::dtcm::LOW_MAC_ACTIVE_TX_COUNT.get()).wrapping_sub(1),
                        );
                        return;
                    }
                    if link > 7 {
                        infallible_to_never(backend.invalid_link_assertion());
                    }
                } else {
                    let record = crate::dtcm::MacPipeRecordAddress::from_raw_unchecked(
                        read_u32(crate::dtcm::MAC_CURRENT_PIPE_RECORD.get()),
                    );
                    if read_u8(record.current_slot().get()) == read_u8(record.last_slot().get())
                        && link < 8
                    {
                        let link_index = usize::from(link);
                        write_u32(crate::dtcm::ba_pipe_bitmap_low_unchecked(link_index).get(), 0);
                        write_u32(crate::dtcm::ba_pipe_bitmap_high_unchecked(link_index).get(), 0);
                    } else {
                        write_u8(
                            crate::dtcm::LOW_MAC_ACTIVE_TX_COUNT.get(),
                            read_u8(crate::dtcm::LOW_MAC_ACTIVE_TX_COUNT.get()).wrapping_sub(1),
                        );
                        return;
                    }
                }
                backend.link_set_state(link, 0x0b);
            }
            raise_scheduler_bits(1 << 21);
        }

        write_u8(
            crate::dtcm::LOW_MAC_ACTIVE_TX_COUNT.get(),
            read_u8(crate::dtcm::LOW_MAC_ACTIVE_TX_COUNT.get()).wrapping_sub(1),
        );
    }
}

#[cfg(all(target_arch = "arm", feature = "experimental-depth-two-ampdu"))]
/// Consume a received compressed BlockAck for the currently owned bounded
/// aggregate. Vendor `complete_tx_pipe_slot` copies the BA start sequence and
/// bitmap into per-link state before `bab_process_ba_bitmap`; the bounded
/// bring-up path classifies the exact retained two-to-four member chain.
///
/// Returns true when the frame is a matching BA and therefore belongs to the
/// low-MAC rather than the host RX indication path.
///
/// # Safety
/// `frame` must address a live RX FIFO frame for `length` bytes, and callers
/// must serialize this with TX publication/completion under the cooperative
/// reactor's existing IRQ/FIQ exclusion.
pub unsafe fn consume_depth_two_block_ack(frame: usize, length: usize) -> bool {
    unsafe {
        if length < 0x1c || read_u16(frame) & 0x00fc != 0x0094 {
            return false;
        }

        let tid = (read_u16(frame + 0x10) >> 12) as u8;
        let runtime = &mut *PROBE_EXPERIMENT.0.get();
        let candidate = runtime.backend.publications.iter().flatten().copied().find(|entry| {
            let Some(slot) = entry.live_slot() else {
                return false;
            };
            if read_u8(slot.kind().get()) != 1
                || read_u32(entry.context.next_in_ampdu_address()) == 0
                || read_u8(entry.context.tid_address()) != tid
            {
                return false;
            }
            let interface = read_u8(entry.context.interface_address());
            (0..3).all(|word| {
                crate::vif::own_mac_word(interface, word)
                    == Some(read_u16(frame + 4 + word * 2))
            })
        });
        let Some(publication) = candidate else {
            return false;
        };

        let mut member_nodes = [0_u32; crate::host_tx_policy::MAX_EXPERIMENTAL_AMPDU_DEPTH];
        let mut sequences = [None; crate::host_tx_policy::MAX_EXPERIMENTAL_AMPDU_DEPTH];
        let mut current = publication.frame_node;
        let mut member_count = 0_usize;
        loop {
            if member_count == crate::host_tx_policy::MAX_EXPERIMENTAL_AMPDU_DEPTH
                || member_nodes[..member_count].contains(&current.raw())
                || !runtime.backend.publications.iter().flatten().any(|entry| {
                    entry.pipe == publication.pipe
                        && entry.slot == publication.slot
                        && entry.context == current.context()
                        && entry.frame_node == current
                })
            {
                return true;
            }
            member_nodes[member_count] = current.raw();
            sequences[member_count] = Some(read_u16(current.context().sequence_number_address()));
            member_count += 1;

            let next_raw = read_u32(current.context().next_in_ampdu_address());
            if next_raw == 0 {
                break;
            }
            let Some(next) = FrameNodeAddress::from_raw(next_raw) else {
                return true;
            };
            current = next;
        }

        let start = read_u16(frame + 0x12) >> 4;
        let bitmap = u64::from(read_u32(frame + 0x14))
            | (u64::from(read_u32(frame + 0x18)) << 32);
        let Some(current) = classify_planned_block_ack(start, bitmap, sequences) else {
            return true;
        };
        if usize::from(current.member_count) != member_count {
            return true;
        }
        let pipe = publication.pipe;
        let Some(retry_index) = pipe_slot_state_index(pipe, publication.slot) else {
            crate::halt_always!();
        };
        let retained = &mut runtime.backend.depth_two_block_ack[retry_index];
        let previous = retained
            .filter(|observation| observation.members == member_nodes)
            .map(|observation| observation.observation);
        let Some(observation) = merge_planned_block_ack(previous, current) else {
            return true;
        };
        *retained = Some(RetainedAmpduBlockAck {
            members: member_nodes,
            observation,
        });

        // Compressed BA frames can arrive repeatedly with shifted or growing
        // windows. Acknowledgements are sticky for this exact aggregate; keep
        // ownership until every retained member has been observed as acknowledged.
        if observation.states[..member_count]
            .iter()
            .any(|state| *state != BlockAckMemberState::Acknowledged)
        {
            return true;
        }
        *retained = None;

        let record = pipe_record_address(pipe);
        let Some(slot) = publication.live_slot() else {
            return true;
        };
        if !matches!(read_u8(slot.state().get()), 3 | 4) {
            return true;
        }

        let Some(ring) = TxHardwareRingAddress::for_pipe(
            pipe,
            read_u32(record.hardware_ring().get()),
        ) else {
            crate::halt_always!();
        };
        write_u32(PIPE_IRQ_TRIGGER as usize, (1_u32 << pipe) << 25);
        complete_tx_pipe_slot(publication.frame_node, slot.raw(), 0, &mut runtime.backend);
        write_u32(ring.completion_word() as usize, 1);
        let next = publication.slot.wrapping_add(1) & 3;
        write_u8(record.current_slot().get(), next);
        write_u8(record.producer_slot().get(), next);
        write_u8(record.state().get(), 0);
        write_u8(record.control().get(), 0);
        write_u8(record.watchdog().get(), 5);
        write_u8(crate::dtcm::LOW_MAC_PIPE_BUSY.get(), 0);
        true
    }
}
/// Exact matching-payload `txp_pipe_tx_success` at Ghidra `0x9cdc`
/// (r2 `0x9cb8`).
///
/// # Safety
/// The selected pipe, all referenced slots/frame nodes, PHY command state,
/// and completion structures must be valid and exclusively owned.
pub unsafe fn service_pipe_tx_success<B: PipeSuccessEffects>(pipe: u8, backend: &mut B) {
    unsafe {
        trace_tx_stage(TX_TRACE_SUCCESS);
        trace_tx_value(0x2c, u32::from(pipe));
        if pipe >= 4 {
            crate::halt_always!();
        }
        let pipe_index = usize::from(pipe);
        let current = read_u8(crate::dtcm::mac_pipe_cursor_mirror_02_unchecked(pipe_index).get());
        if current >= 4 {
            crate::halt_always!();
        }
        let Some(current_slot) = LiveTxSlot::from_pipe_slot(pipe, current) else {
            crate::halt_always!();
        };
        let current_frame = current_slot.frame_node;

        write_u32(0xfff0_1a98, read_u32(0xfff0_1a98).wrapping_add(1));
        write_u8(crate::dtcm::LOW_MAC_PIPE_BUSY.get(), 0);
        write_u8(current_slot.address.state().get(), 3);

        if read_u8(crate::dtcm::mac_pipe_state_unchecked(pipe_index).get()) == 1
            && read_u8(current_slot.address.retry_rate().get()) == 0xff
        {
            let frame = current_frame.context();
            let flags = read_u32(frame.control_bits_address());
            if flags & (1 << 4) == 0 {
                backend.set_frame_lifetime(current_frame);
            }
            if flags & (1 << 15) == 0 {
                backend.reset_backoff(
                    read_u8(frame.interface_address()),
                    read_u8(
                        crate::dtcm::queue_to_access_category_unchecked(usize::from(pipe)).get(),
                    ),
                );
            }

            let last = read_u8(crate::dtcm::mac_pipe_cursor_mirror_01_unchecked(pipe_index).get());
            if current == last {
                let mut index = read_u8(crate::dtcm::mac_pipe_current_slot_unchecked(pipe_index).get());
                loop {
                    let Some(slot) = LiveTxSlot::from_pipe_slot(pipe, index) else {
                        crate::halt_always!();
                    };
                    complete_tx_pipe_slot(
                        slot.frame_node,
                        slot.address.raw(),
                        0,
                        backend,
                    );
                    if index == last {
                        break;
                    }
                    index = index.wrapping_add(1) & 3;
                }
                write_u8(crate::dtcm::mac_pipe_current_slot_unchecked(pipe_index).get(), last.wrapping_add(1) & 3);
                write_u8(crate::dtcm::mac_pipe_state_unchecked(pipe_index).get(), 0);
                write_u8(crate::dtcm::mac_pipe_control_byte_04_unchecked(pipe_index).get(), 0);
                write_u8(crate::dtcm::mac_pipe_control_byte_05_unchecked(pipe_index).get(), 5);
            } else {
                write_u8(
                    crate::dtcm::mac_pipe_cursor_mirror_02_unchecked(pipe_index).get(),
                    current.wrapping_add(1) & 3,
                );
            }
        }

        #[cfg(feature = "vendor-host-tx-diagnostics")]
        {
            let (packed, ring_word) = pipe_cursor_diagnostic(&mut VolatileMacPipeMmio, pipe);
            crate::host_tx_diagnostics::trace(0x4358_0002, packed, ring_word);
            crate::host_tx_diagnostics::capture_pipe_cursor(
                packed,
                !pipe_cursor_invariant_holds(packed),
            );
        }
        write_u8(crate::dtcm::mac_pipe_event_flag_unchecked(usize::from(pipe)).get(), 0);
        if read_u32(crate::dtcm::MAC_PHY_OPERATION_STATE.get()) == 4 {
            dispatch_phy_command_3();
            write_u32(crate::dtcm::MAC_PHY_OPERATION_STATE.get(), 2);
            raise_scheduler_bits(1 << 18);
        }
    }
}

pub trait PipeStartEffects {
    fn start_without_pending_diagnostic(&mut self, pipe: u8);
}

/// Exact matching-payload `txp_pipe_tx_start` at Ghidra `0x9dea`
/// (r2 `0x9dc6`).
///
/// # Safety
/// Pipe/slot/frame records, packet headers, duration tables, DMA producer and
/// PHY MMIO must be valid and exclusively owned.
pub unsafe fn service_pipe_tx_start<B: PipeStartEffects>(pipe: u8, backend: &mut B) {
    unsafe {
        trace_tx_stage(TX_TRACE_START);
        trace_tx_value(0x24, u32::from(pipe));
        write_u32(
            crate::dtcm::RX_FIFO_STATE.ba_scan_cursor().get(),
            read_u32(crate::platform::mac_register(0x0604)),
        );
        if pipe >= 4 {
            crate::halt_always!();
        }
        let pipe_index = usize::from(pipe);
        let current = read_u8(crate::dtcm::mac_pipe_cursor_mirror_02_unchecked(pipe_index).get());
        if current >= 4 {
            crate::halt_always!();
        }
        let current_index = usize::from(current);
        let slot = crate::dtcm::mac_pipe_slot_state_word_unchecked(pipe_index, current_index);
        if read_u8(crate::dtcm::mac_pipe_state_unchecked(pipe_index).get()) == 0 {
            backend.start_without_pending_diagnostic(pipe);
            return;
        }
        let Some(live_slot) = LiveTxSlot::from_pipe_slot(pipe, current) else {
            crate::halt_always!();
        };

        crate::host_tx_diagnostics::bump(crate::host_tx_diagnostics::counter::TX_START);
        write_u8(live_slot.address.state().get(), 2);
        let frame_node = live_slot.frame_node;
        let context = frame_node.context();
        if read_u32(crate::dtcm::MAC_PHY_OPERATION_STATE.get()) == 3 {
            let secondary = read_u8(
                crate::dtcm::rate_encoding_unchecked(usize::from(read_u8(
                    context.tx_rate_address(),
                )))
                .get(),
            );
            dispatch_phy_command_2(secondary);
            if read_u8(crate::dtcm::MAC_PHY_DISPATCH_OUTPUT.get()) == 4 {
                write_u32(crate::dtcm::MAC_PHY_OPERATION_STATE.get(), 4);
            } else {
                raise_scheduler_bits(1 << 18);
            }
        }

        if read_u8(crate::dtcm::mac_pipe_state_unchecked(pipe_index).get()) == 1
            && read_u8(slot.get()) == 0
            && read_u16(context.frame_control_address()) & 0x0f != 4
            && read_u32(context.control_bits_address()) & 1 == 0
        {
            let rate = read_u8(context.duration_slot_address());
            if usize::from(rate) >= packet_ram::DURATION_WORD_COUNT {
                crate::halt_always!();
            }
            let duration = read_u16(packet_ram::duration_word(usize::from(rate)));
            let Some(header) = validated_context_tx_frame(context) else {
                crate::halt_always!();
            };
            write_u16(header.sequence_control() as usize, duration);
            write_u16(context.sequence_number_address(), duration);
            write_u32(
                context.control_bits_address(),
                read_u32(context.control_bits_address()) | 1,
            );
            write_u8(crate::dtcm::LOW_MAC_EVENT_PENDING.get(), 1);
            write_u8(crate::dtcm::LOW_MAC_SELECTED_RATE.get(), rate);
        }

        let slot_state = live_slot.command;
        if read_u32(slot_state as usize + 8) & (1 << 27) == 0
            && current != read_u8(crate::dtcm::mac_pipe_cursor_mirror_01_unchecked(pipe_index).get())
        {
            write_u8(
                crate::dtcm::mac_pipe_cursor_mirror_02_unchecked(pipe_index).get(),
                current.wrapping_add(1) & 3,
            );
        }
    }
}

#[allow(unreachable_code)]
pub unsafe fn release_lmc_radio_scheduler<B: RadioCompletionEffects>(owner: u32, backend: &mut B) {
    unsafe {
        let owner_address = owner as usize;
        let interface = usize::from(read_u8(owner_address + 0x0d));
        let released_vif = lmc_vif_address(interface); let deferred_owner = crate::dtcm::deferred_radio_owner().get(); let radio_owner = crate::dtcm::radio_owner().get();
        if read_u16(released_vif + 0x30) != 0 {
            write_u32(deferred_owner, owner);
            return;
        }
        write_u32(deferred_owner, 0);
        if read_u32(radio_owner) != owner {
            infallible_to_never(backend.radio_owner_mismatch());
        }

        write_u32(radio_owner, 0);
        write_u8(owner_address + 0x22, 0);
        write_u8(
            crate::dtcm::LOW_MAC_RECEIVE_GATE_BITS.get(),
            read_u8(crate::dtcm::LOW_MAC_RECEIVE_GATE_BITS.get()) & 0xfd,
        );
        raise_scheduler_bits(1 << 21);
        write_u16(released_vif + 0x2e, 0);

        for interface in 0..3_usize {
            let vif = lmc_vif_address(interface);
            if read_u16(vif + 0x52) == read_u16(released_vif + 0x42) && read_u8(vif + 0x66) == 2 {
                write_u32(radio_owner, (vif + 0x44) as u32);
                write_u8(vif + 0x66, 3);
                return;
            }
        }

        let radio_wait_head = crate::dtcm::radio_wait_head().get(); let pending = read_u32(radio_wait_head);
        if pending == 0 {
            return;
        }
        write_u32(radio_owner, pending);
        write_u32(radio_wait_head, read_u32(pending as usize + 4));
        write_u8(pending as usize + 0x22, 3);
        backend.tbtt_post_process(read_u8(pending as usize + 0x0d));
    }
}

pub trait PowerSaveCompletionEffects {
    fn completion_idle_policy_enabled(&self) -> bool;
    fn timer_start(&mut self, timer: u32, duration: u32);
    fn send_pending_poll_or_qos_null(&mut self, interface: u8);
    fn try_enter_sleep_all(&mut self);
    fn release_radio_if_all_idle(&mut self, interface: u8);
}

fn phy_operation_7_timer() -> (u32, u32) {
    (crate::dtcm::MAC_PHY_OPERATION_TIMER.get() as u32, 0x0098_9680)
}

fn phy_dispatch_switch_target(command: u8) -> u32 {
    const OFFSETS: [u8; 9] = [0x17, 0x05, 0x08, 0x0f, 0x20, 0x0f, 0x1c, 0x1a, 0x07];
    let index = usize::from(command.min(8));
    (0x0001_6f83_u32 + u32::from(OFFSETS[index]) * 2) & !1
}

/// Exact command-7 path through `pac_phy_start_op` (`0x7f10`) and
/// `phy_state_cmd_dispatch` (`0x16f6c`). The inline switch table maps command
/// seven directly to the `0x16fb6` case: state 1 plus a 10,000,000-tick timer.
///
/// # Safety
/// PHY command state and timer storage must be initialized and exclusively
/// owned by the cooperative scheduler.
pub unsafe fn start_phy_operation_7<B: PowerSaveCompletionEffects>(backend: &mut B) {
    unsafe {
        debug_assert_eq!(phy_dispatch_switch_target(7), 0x0001_6fb6);
        write_u8(crate::dtcm::MAC_PHY_OPERATION_COMMAND.get(), 7);
        write_u8(crate::dtcm::mac_phy_dispatch_command_byte_unchecked(1).get(), 0);
        write_u8(crate::dtcm::MAC_PHY_OPERATION_OUTPUT.get(), 1);
        let (timer, duration) = phy_operation_7_timer();
        write_u32(crate::dtcm::MAC_PHY_OPERATION_TIMEOUT.get(), duration);
        write_u32(
            crate::dtcm::MAC_PHY_OPERATION_STATE.get(),
            u32::from(read_u8(crate::dtcm::MAC_PHY_OPERATION_OUTPUT.get())),
        );
        let timeout = read_u32(crate::dtcm::MAC_PHY_OPERATION_TIMEOUT.get());
        if timeout != 0 {
            backend.timer_start(timer, timeout);
        }
    }
}

/// Exact matching-payload `ps_timer_followup` at `0xda40`.
///
/// # Safety
/// `context`, per-interface power-save records, and timer objects must be
/// initialized and exclusively owned by the cooperative scheduler.
pub unsafe fn service_power_save_completion<B: PowerSaveCompletionEffects>(
    context: ContextAddress,
    backend: &mut B,
) {
    unsafe {
        if read_u16(context.terminal_status_address()) == 0x16 {
            return;
        }
        let interface = read_u8(context.interface_address());
        let state = crate::dtcm::power_save_observed_view(usize::from(interface))
            .map_or_else(|| unreachable!(), crate::dtcm::DtcmAddress::get);
        if read_u8(state + 0xfc) != 0
            && (read_u32(context.control_bits_address()) & 0x03ff_ffff) >> 24 == 0
            && read_u16(context.frame_control_address()) & 0x4f != 0x48
        {
            write_u16(state + 0x44, read_u16(state + 0x44) | 8);
            backend.timer_start((state + 0xe8) as u32, read_u32(state + 0x118));
        }

        if read_u32(crate::dtcm::MAC_WAKE_CONTROL.get()) != 0 {
            if read_u8(crate::dtcm::power_save_global_sleep_state().get()) < 3 {
                return;
            }
            backend.timer_start((state + 0xe8) as u32, 0x0000_1f40);
            if read_u8(crate::dtcm::power_save_global_sleep_state().get()) == 4 {
                backend.try_enter_sleep_all();
            }
            return;
        }

        if read_u16(context.terminal_status_address()) == 0 {
            write_u16(lmc_vif_address(usize::from(interface)) + 0x1e2, 0);
        }
        let mode = read_u8(state + 0x40);
        if mode != 1 {
            if mode == 0 && read_u8(crate::dtcm::power_save_global_sleep_state().get()) >= 3 {
                backend.try_enter_sleep_all();
            }
            return;
        }

        let threshold = read_u16(state + 0x136);
        if threshold != 0 && threshold < read_u16(state + 0x134) {
            write_u16(state + 0x134, 0);
            write_u16(state + 0x44, read_u16(state + 0x44) | 4);
            backend.timer_start((state + 0xac) as u32, read_u32(state + 0x120));
        }
        let context_flags = read_u32(context.control_bits_address());
        if context_flags & (1 << 25) != 0 {
            backend.timer_start((state + 0xc0) as u32, read_u32(crate::dtcm::power_save_global_timer_duration().get()));
            return;
        }

        let queue = read_u8(context.access_category_address());
        let power_save_mask = read_u16(state + 0x5a);
        let queue_mask = 1_u32.wrapping_shl(u32::from(queue) & 0x1f);
        if u32::from(power_save_mask) & queue_mask == 0 {
            let pending = read_u32(state + 0x54);
            let interval = read_u32(state + 0x128);
            if pending != 0 && pending >= interval.wrapping_mul(2) {
                backend.timer_start((state + 0x70) as u32, interval);
            }
        } else {
            if (power_save_mask & 0x3f) >> 4 == 1 {
                if read_u16(state + 0x44) & 1 == 0 {
                    write_u16(state + 0x44, read_u16(state + 0x44) | 1);
                    backend.send_pending_poll_or_qos_null(interface);
                }
            } else if context_flags & (1 << 24) != 0 {
                write_u16(state + 0x44, read_u16(state + 0x44) | 1);
            }
            if power_save_mask & (1 << 6) == 0 {
                backend.timer_start((state + 0x98) as u32, 0x0000_9470);
            }
        }
        backend.release_radio_if_all_idle(interface);
    }
}

fn tala_reduction(
    current: u32,
    weighted_total: u32,
    weighted_penalty: u32,
    parameter1: u32,
    minimum: u32,
) -> (u32, u32) {
    if weighted_total.wrapping_mul(parameter1 >> 24) <= weighted_penalty {
        (current >> 1, 0)
    } else if weighted_total.wrapping_mul((parameter1 >> 16) & 0xff) <= weighted_penalty {
        ((current >> 1).wrapping_add(current >> 3), 1)
    } else if weighted_total.wrapping_mul((parameter1 >> 8) & 0xff) <= weighted_penalty {
        ((current >> 1).wrapping_add(current >> 2), 1)
    } else if weighted_total.wrapping_mul(parameter1 & 0xff) <= weighted_penalty {
        (
            (current >> 1)
                .wrapping_add(current >> 2)
                .wrapping_add(current >> 3)
                & 0xff,
            minimum,
        )
    } else {
        (current, minimum)
    }
}

fn tala_reduction_at_decision(
    read_ampdu_length: impl FnOnce() -> u16,
    weighted_total: u32,
    weighted_penalty: u32,
    parameter1: u32,
    minimum: u32,
) -> (u32, u32) {
    tala_reduction(
        u32::from(read_ampdu_length() & 0xff),
        weighted_total,
        weighted_penalty,
        parameter1,
        minimum,
    )
}

unsafe fn update_tala_for_completion(frame_node: FrameNodeAddress) {
    unsafe {
        let context = frame_node.context();
        let interface = usize::from(read_u8(context.interface_address()));
        let override_value = read_u16(0xfff0_1a7c + interface * 2);
        if override_value != 0 && crate::vif::rts_threshold(interface as u8).unwrap_or(0) >= 0x660 {
            return;
        }

        let status = read_u16(context.terminal_status_address());
        let success = crate::dtcm::tala_success_unchecked(interface).get();
        let failure = crate::dtcm::tala_failure_unchecked(interface).get();
        let tries_total = crate::dtcm::tala_cumulative_tries_unchecked(interface).get();
        let penalty = crate::dtcm::tala_weighted_penalty_unchecked(interface).get();
        if status == 0 {
            write_u32(success, read_u32(success).wrapping_add(1));
        } else if status == 0x0b {
            write_u32(failure, read_u32(failure).wrapping_add(1));
        }

        let completed = read_u32(success).wrapping_add(read_u32(failure));
        let policy = usize::from(read_u8(context.retry_policy_address()));
        let short_retries = u32::from(read_u8(
            crate::dtcm::rate_policy_short_retry_limit_compat(policy).get(),
        ));
        let expected = short_retries.wrapping_mul(15).wrapping_add(99) / 100;
        let tries = u32::from(read_u16(context.try_count_address()));
        write_u32(tries_total, read_u32(tries_total).wrapping_add(tries));
        let addition = if tries > (expected & 0xffff) {
            1_u32 << (tries.wrapping_sub(expected & 0xffff).wrapping_add(1) & 0x0f)
        } else {
            tries
        };
        write_u32(penalty, read_u32(penalty).wrapping_add(addition));

        let parameter0 = read_u32(crate::dtcm::MAC_ACCOUNTING_PARAMETER0.get());
        let evaluation_window = ((parameter0 >> 16) & 0xff).wrapping_mul(200);
        if completed < evaluation_window && read_u32(penalty) <= 0x400 {
            return;
        }

        let parameter1 = read_u32(crate::dtcm::MAC_ACCOUNTING_PARAMETER1.get());
        let weighted_penalty = read_u32(penalty).wrapping_mul(100);
        let weighted_total = short_retries.wrapping_mul(completed);
        let (mut next, minimum) = tala_reduction_at_decision(
            || crate::vif::ampdu_length(interface as u8).unwrap_or(0),
            weighted_total,
            weighted_penalty,
            parameter1,
            (parameter0 >> 8) & 0xff,
        );

        let sample_count = read_u32(crate::dtcm::MAC_SAMPLE_COUNT.get());
        let sample_divisor = read_u32(crate::dtcm::MAC_STATUS_ACCOUNTING.get());
        if sample_count.wrapping_add(sample_divisor) == 0 {
            if read_u8(crate::dtcm::mac_current_pipe_state_byte().get()) != 0
                && read_u16(crate::dtcm::MAC_ACCOUNTING_AVERAGE.get()) < 50
                && read_u32(tries_total)
                    .wrapping_add(read_u32(success))
                    .wrapping_mul(80)
                    < read_u32(tries_total).wrapping_mul(100)
            {
                write_u8(crate::dtcm::mac_current_pipe_state_byte().get(), 0);
            }
        } else {
            let sample = sample_count.wrapping_mul(600) / sample_count.wrapping_add(sample_divisor);
            let average =
                sample.wrapping_add(u32::from(read_u16(crate::dtcm::MAC_ACCOUNTING_AVERAGE.get())).wrapping_mul(4)) / 10;
            write_u16(crate::dtcm::MAC_ACCOUNTING_AVERAGE.get(), average as u16);
            if average > 75 {
                write_u8(crate::dtcm::mac_current_pipe_state_byte().get(), 1);
            }
            write_u32(crate::dtcm::MAC_SAMPLE_COUNT.get(), 0);
            write_u32(crate::dtcm::MAC_STATUS_ACCOUNTING.get(), 0);
        }

        let control = read_u8(crate::dtcm::MAC_SILICON_CONTROL.get());
        if weighted_total.wrapping_mul(parameter1 & 0xff) >> 1 < weighted_penalty {
            write_u8(crate::dtcm::tala_growth_streak_unchecked(interface).get(), 0);
            if control & 2 == 0 {
                write_u8(crate::dtcm::MAC_SILICON_CONTROL.get(), control | 1);
            }
        } else {
            let streak_address = crate::dtcm::tala_growth_streak_unchecked(interface).get();
            let streak = read_u8(streak_address).wrapping_add(1);
            write_u8(streak_address, streak);
            if ((parameter0 >> 24) & 0x0f) <= u32::from(streak) {
                write_u8(crate::dtcm::mac_current_pipe_state_byte().get(), 1);
                let average = read_u16(crate::dtcm::MAC_ACCOUNTING_AVERAGE.get());
                write_u16(crate::dtcm::MAC_ACCOUNTING_AVERAGE.get(), average.wrapping_sub(average >> 5));
                next = next.wrapping_add(1) & 0xff;
                if control & 2 != 0 {
                    write_u8(crate::dtcm::MAC_SILICON_CONTROL.get(), control | 1);
                }
            }
        }

        if override_value == 0 {
            let maximum = parameter0 & 0xff;
            let clamped = if next < minimum {
                minimum
            } else if next > maximum {
                maximum
            } else {
                next
            };
            if crate::vif::set_ampdu_length(interface as u8, clamped as u16).is_err() {
                crate::halt_always!();
            }
        }
        write_u32(tries_total, 0);
        write_u32(penalty, 0);
        write_u32(failure, 0);
        write_u32(success, 0);
        write_u32(
            0xfff0_2e4c,
            u32::from(crate::vif::ampdu_length(interface as u8).unwrap_or(0)),
        );
    }
}

/// Exact matching-payload `tx_complete_tala_adapt` (`0xd254`, r2 `0xd1fc`).
/// The producer is snapshotted once and every frame in that prefix is fully
/// processed before this function returns.
///
/// # Safety
/// Completion-ring, TX context, statistics, scheduler, and message storage
/// must be initialized and exclusively owned by the cooperative dispatcher.
pub unsafe fn service_completion_drain<B>(backend: &mut B)
where
    B: CompletionDrainEffects
        + PowerSaveCompletionEffects
        + BaCompletionEffects
        + MessageCompletionEffects
        + RadioCompletionEffects,
{
    unsafe {
        let mut cursor = CompletionDrainCursor::begin();
        let mut zero_class_pending = 0_u8;
        if crate::dtcm::ampdu_completion_control_ptr().read_volatile() != 0 {
            let mut index = cursor.next;
            while index != cursor.target {
                let node = COMPLETION_RING.frame_node(index);
                if read_u8(FrameNodeAddress::new(node).context().completion_class_address()) == 0 {
                    zero_class_pending = zero_class_pending.wrapping_add(1);
                }
                index = index.wrapping_add(1) & 0x3f;
            }
        }

        while let Some(frame_node) = cursor.pop() {
            let context = frame_node.context();
            let status = read_u16(context.terminal_status_address());
            let interface = usize::from(read_u8(context.interface_address()));

            update_tala_for_completion(frame_node);
            write_u16(
                context.completion_flags_address(),
                read_u32(context.auxiliary_state_address()) as u16,
            );

            let flags = read_u32(context.control_bits_address());
            if flags & (1 << 5) != 0 {
                let accumulated = u64::from(
                    crate::dtcm::ampdu_tx_duration_low_ptr().read_volatile(),
                ) | (u64::from(
                    crate::dtcm::ampdu_tx_duration_high_ptr().read_volatile(),
                ) << 32);
                let accumulated = accumulated.wrapping_add(u64::from(read_u16(context.frame_length_address())));
                crate::dtcm::ampdu_tx_duration_low_ptr().write_volatile(accumulated as u32);
                crate::dtcm::ampdu_tx_duration_high_ptr()
                    .write_volatile((accumulated >> 32) as u32);
                let counted_frames = crate::dtcm::ampdu_tx_counted_frames_ptr();
                counted_frames.write_volatile(counted_frames.read_volatile().wrapping_add(1));

                if flags & (1 << 6) != 0 {
                    let error_frames = crate::dtcm::ampdu_tx_error_frames_ptr();
                    error_frames.write_volatile(error_frames.read_volatile().wrapping_add(1));
                    if backend.completion_messages_enabled()
                        && (flags >> 20) & 3 != 0
                        && read_u8(crate::dtcm::lmc_message_control().get()) & 1 != 0
                        && let Some(header) = validated_context_tx_frame(context)
                            && let Some(message) =
                                allocate_lmc_message(|| backend.message_allocation_failed())
                        {
                            // The allocator returns the typed record selected by the producer cursor.
                            write_u8(message.kind().get(), 7);
                            write_u8(message.interface().get(), interface as u8);
                            write_u8(message.completion_tid().get(), read_u8(context.tid_address()));
                            write_u8(message.completion_state().get(), read_u8(context.link_id_address()));
                            let queue = usize::from(read_u8(context.access_category_address()));
                            write_u8(
                                message.completion_queue().get(),
                                read_u8(crate::dtcm::access_category_to_queue_unchecked(queue).get()),
                            );
                            write_u16(message.completion_sequence().get(), read_u16(context.sequence_number_address()) << 4);
                            write_u16(
                                message.completion_mac_word(0).unwrap().get(),
                                read_u16(header.address_1_halfword_unchecked(0) as usize),
                            );
                            write_u16(
                                message.completion_mac_word(1).unwrap().get(),
                                read_u16(header.address_1_halfword_unchecked(1) as usize),
                            );
                            write_u16(
                                message.completion_mac_word(2).unwrap().get(),
                                read_u16(header.address_1_halfword_unchecked(2) as usize),
                            );
                            raise_scheduler_bits(1 << 22);
                        }
                }
            }

            if status == 0x0b
                && read_u8(
                    crate::dtcm::pas_stride_view_unchecked(interface)
                        .mode_byte()
                        .get(),
                ) == 2
            {
                mark_ba_session_state_5(context, |mac_upper| {
                    backend.find_pipe_by_mac_upper(mac_upper)
                });
            }
            set_active_pas_contexts(active_pas_contexts().wrapping_sub(1));
            if interface < 3
                && crate::vif::adjust_tx_busy(interface as u8, -1).is_err() {
                    crate::halt_always!();
                }
            if interface < 2 {
                service_power_save_completion(context, backend);
            }
            if zero_class_pending > 1 && read_u8(context.completion_class_address()) == 0 {
                write_u16(
                    context.completion_flags_address(),
                    read_u16(context.completion_flags_address()) | 0x20,
                );
                zero_class_pending = zero_class_pending.wrapping_sub(1);
            }
            write_u32(
                context.ownership_bits_address(),
                read_u32(context.ownership_bits_address()) | 0x8000,
            );
            backend.complete_context(context, status);
        }

        if active_pas_contexts() == 0 && backend.completion_idle_policy_enabled() {
            start_phy_operation_7(backend);
            let owner = read_u32(crate::dtcm::deferred_radio_owner().get());
            if owner != 0 {
                release_lmc_radio_scheduler(owner, backend);
            }
        }
    }
}

/// Couples a completion-ring frame node to callback entry for the tracked
/// probe. A node for another context is not consumed by this tracker and must
/// be dispatched by the general class callback path.
pub fn begin_probe_completion_callback(
    tracker: &mut ProbeTxTracker,
    frame_node: FrameNodeAddress,
) -> bool {
    let ProbeTxOwnership::CompletionQueued { context, .. } = tracker.ownership else {
        return false;
    };
    frame_node.context().raw() == context && tracker.begin_completion_callback()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompletedContextDispatch {
    NoCallback,
    ClassZeroCallbackOwnsReturn,
    Returned,
}

/// Exact callback wrapper at matching-payload `0xb6f4`.
///
/// `callback` must execute the translated callback for the selected class.
/// The vendor callback table is consulted only for presence; its legacy code
/// pointer is never invoked. Class zero owns its own return path. Every other
/// callback class flows through exact `0xd0a8` context return.
///
/// # Safety
/// The context and all referenced per-interface/accounting records must be
/// valid. Completion callback and free-list state must be exclusively owned.
/// Publish the translated completion callback classes before any context can
/// complete. The retained words were vendor function pointers; translated
/// dispatch uses them only as class-presence gates and invokes Rust closures.
pub(crate) unsafe fn initialize_completion_callback_presence() {
    let words = crate::dtcm::visible_completion_words_ptr();
    for class in 0..10 {
        unsafe { words.add(class).write_volatile(1) };
    }
}

pub unsafe fn dispatch_completed_context<F, G>(
    context: ContextAddress,
    completion_status: u16,
    callback: F,
    service_pending_queue: G,
) -> CompletedContextDispatch
where
    F: FnOnce(u8, ContextAddress),
    G: FnOnce(),
{
    unsafe {
        if (context.terminal_status_address() as *const u16).read_volatile() == 0x00ff {
            return CompletedContextDispatch::NoCallback;
        }

        let saved_status = (context.try_count_address() as *const u16).read_volatile();
        (context.saved_status_address() as *mut u8).write_volatile(saved_status as u8);
        (context.terminal_status_address() as *mut u16).write_volatile(completion_status);
        (context.completion_status_address() as *mut u32)
            .write_volatile(u32::from(completion_status));

        let aggregate = (context.optional_pipe_object_address() as *const u32).read_volatile();
        if aggregate != 0 {
            let budget = (aggregate as usize + 7) as *mut u8;
            let budget_value = budget.read_volatile() as i8;
            if budget_value > 0
                && (context.completion_class_address() as *const u8).read_volatile() == 0
            {
                budget.write_volatile((budget_value - 1) as u8);
            }
        }

        let interface = (context.interface_address() as *const u8).read_volatile();
        if interface < 2 {
            let device = completion_device_address(interface);
            let retry_class = if completion_status == 0
                && (context.frame_control_address() as *const u16).read_volatile() & 0x0f == 8
            {
                1
            } else {
                0
            };
            let retry_flags = (device + 0x3ac) as *mut u8;
            retry_flags.write_volatile(retry_flags.read_volatile() | retry_class);

            let device_flags = (device + 4) as *mut u32;
            let flags = device_flags.read_volatile();
            if (flags >> 1) & 3 == 3 {
                let countdown = (device + 0x14b) as *mut u8;
                let countdown_value = countdown.read_volatile();
                let peer = validated_context_tx_frame(context);
                if countdown_value != 0
                    && peer.is_some_and(|peer| {
                        ((peer.raw() as usize + 4) as *const u8).read_volatile() & 1 != 0
                    })
                {
                    let next = countdown_value.wrapping_sub(1);
                    countdown.write_volatile(next);
                    if next == 0 {
                        let mut updated = device_flags.read_volatile();
                        if updated & 0x8000_0000 != 0 {
                            updated |= 1 << 29;
                        }
                        device_flags.write_volatile(updated & 0x7fff_ffff);
                    }
                }
            }
        }

        let completion_class = (context.completion_class_address() as *const u8).read_volatile();
        let callback_address = crate::dtcm::visible_completion_words_ptr()
            .add(usize::from(completion_class)) as *const u32;
        if callback_address.read_volatile() == 0 {
            return CompletedContextDispatch::NoCallback;
        }
        let ownership_flags = context.ownership_bits_address() as *mut u32;
        ownership_flags.write_volatile(ownership_flags.read_volatile() | (1 << 16));
        callback(completion_class, context);
        raise_scheduler_bits(1 << 21);

        if completion_class == 0 {
            return CompletedContextDispatch::ClassZeroCallbackOwnsReturn;
        }

        if interface < 2 {
            let device = completion_device_address(interface);
            let alternate = ((device + 0x0e) as *const u8).read_volatile() != 0;
            let retry_limit =
                ((device + completion_retry_limit_offset(alternate)) as *const u8).read_volatile();
            if retry_limit != 0xff
                && (context.tx_rate_address() as *const u8).read_volatile() <= retry_limit
            {
                let retry_count = (device + 0x0f) as *mut u8;
                let mut retries = retry_count.read_volatile();
                if completion_status == 0 && retries < 5 {
                    retries = retries.wrapping_add(1);
                    retry_count.write_volatile(retries);
                }
                let consumed = (context.saved_status_address() as *const u8).read_volatile();
                if consumed != 0 {
                    retry_count.write_volatile(retries.saturating_sub(consumed));
                }
            }
        }

        return_completed_context(context, service_pending_queue);
        CompletedContextDispatch::Returned
    }
}

/// Runs the exact general callback wrapper for a tracked probe and records
/// `Returned` only when the wrapper actually reaches `0xd0a8`.
///
/// # Safety
/// Same requirements as [`dispatch_completed_context`].
pub unsafe fn dispatch_probe_completed_context<F, G>(
    tracker: &mut ProbeTxTracker,
    completion_status: u16,
    callback: F,
    service_pending_queue: G,
) -> bool
where
    F: FnOnce(u8, ContextAddress),
    G: FnOnce(),
{
    let mut callback = Some(callback);
    let mut pending = Some(service_pending_queue);
    tracker.try_finish_completion_callback(|context| {
        let result = unsafe {
            dispatch_completed_context(
                ContextAddress::new(context),
                completion_status,
                |class, address| {
                    if let Some(callback) = callback.take() {
                        callback(class, address);
                    }
                },
                || {
                    if let Some(pending) = pending.take() {
                        pending();
                    }
                },
            )
        };
        result == CompletedContextDispatch::Returned
    })
}

/// Executes the translated class-6 callback wrapper for a tracked probe.
/// Other classes are rejected before any callback/accounting mutation.
///
/// # Safety
/// Same requirements as [`dispatch_completed_context`]. The downstream
/// scheduler-bit-10 task raised by class 6 must be serviced separately.
pub unsafe fn dispatch_class6_probe_completion<G>(
    tracker: &mut ProbeTxTracker,
    completion_status: u16,
    service_pending_queue: G,
) -> bool
where
    G: FnOnce(),
{
    let ProbeTxOwnership::CallbackRunning { context, .. } = tracker.ownership else {
        return false;
    };
    if unsafe { ((context as usize + 0x53) as *const u8).read_volatile() } != 6 {
        return false;
    }
    unsafe {
        dispatch_probe_completed_context(
            tracker,
            completion_status,
            |class, address| {
                if class == 6 {
                    service_class6_probe_completion(address.raw());
                }
            },
            service_pending_queue,
        )
    }
}

/// Exact context return at matching-payload `0xd0a8`.
///
/// `service_pending_queue` represents vendor helper `0x4f58` and is invoked
/// only when `0x04008ad8+0x0b` is nonzero.
///
/// # Safety
/// `context` must have completed callback dispatch and must still be owned by
/// the completion path. Free-list, accounting, and scheduler state must be
/// exclusively owned by the caller.
pub unsafe fn return_completed_context<F>(context: ContextAddress, service_pending_queue: F)
where
    F: FnOnce(),
{
    unsafe {
        let free_head = internal_context_free_head();
        write_u32(context.intrusive_next_address(), free_head.read_volatile());
        write_u16(context.terminal_status_address(), 0x00ff);
        write_u32(
            context.ownership_bits_address(),
            read_u32(context.ownership_bits_address()) | (1 << 17),
        );
        free_head.write_volatile(context.raw());

        let class = read_u8(context.completion_class_address());
        if class == 0 {
            let counter = crate::dtcm::shared_ptr::<u8>(
                crate::dtcm::class0_internal_context_count(),
            );
            counter.write_volatile(counter.read_volatile().wrapping_sub(1));
        } else {
            set_active_internal_contexts(active_internal_contexts().wrapping_sub(1));
        }

        let pending_service = crate::dtcm::shared_ptr::<u8>(crate::dtcm::pending_service_needed());
        if pending_service.read_volatile() != 0 {
            service_pending_queue();
        }
        let control = crate::dtcm::shared_ptr::<u8>(crate::dtcm::lmc_message_control());
        let value = control.read_volatile();
        if value & 4 != 0 {
            control.write_volatile(value & 0xfb);
            raise_scheduler_bits(1 << 22);
        }
    }
}

/// Couples final callback completion to exact vendor context return. The
/// tracker changes to `Returned` only after all MMIO effects complete.
///
/// # Safety
/// Same requirements as [`return_completed_context`].
pub unsafe fn return_probe_context_after_callback<F>(
    tracker: &mut ProbeTxTracker,
    service_pending_queue: F,
) -> bool
where
    F: FnOnce(),
{
    let ProbeTxOwnership::CallbackRunning { .. } = tracker.ownership else {
        return false;
    };
    let mut callback = Some(service_pending_queue);
    tracker.finish_completion_callback(|returned| unsafe {
        return_completed_context(ContextAddress::new(returned), || {
            if let Some(callback) = callback.take() {
                callback();
            }
        });
    })
}

/// Matching-payload event trace helper at `0x164f8`.
///
/// # Safety
/// The vendor high-alias trace RAM must be mapped and writable.
pub unsafe fn trace_mac_event(raw: u32) {
    unsafe {
        let counter = 0xfff0_3794 as *mut u32;
        let index = counter.read_volatile();
        let slot = 0xfff0_3714_u32.wrapping_add((index & 0x1f) * 4);
        (slot as *mut u32).write_volatile(raw);
        counter.write_volatile(index.wrapping_add(1));
    }
}

/// Exact beacon-marker effect from `0x9fea -> 0x81c4 -> 0x9038`.
pub fn execute_mac_beacon_event<M, F>(mmio: &mut M, raise_scheduler: F)
where
    M: MacPipeMmio,
    F: FnOnce(u32),
{
    mmio.write_u32(crate::dtcm::MAC_BEACON_CONTROL_STATE.get() as u32, 5);
    if mmio.read_u16(crate::dtcm::LOW_MAC_OPTIONAL_PIPE_OBJECT_WORD.get() as u32) != 0 {
        mmio.write_u16(crate::dtcm::LOW_MAC_CONTROLLER_CONFIG.get() as u32, 0x2000);
        let timer = mmio
            .read_u32(crate::dtcm::MAC_BEACON_SECONDARY_COMMAND.get() as u32)
            .wrapping_add(
                u32::from(
                    mmio.read_u16(crate::dtcm::LOW_MAC_OPTIONAL_PIPE_OBJECT_WORD.get() as u32),
                ) * 0x400,
            )
            | 0x8000_0000;
        mmio.write_u32(MAC_BEACON_TIMER + 0x14, timer);
    }
    raise_scheduler(1 << 24);
}

/// Production MMIO wrapper for the beacon-marker effect.
///
/// # Safety
/// MAC/DTCM state must be mapped and exclusively owned by event servicing.
pub unsafe fn service_mac_beacon_event() {
    execute_mac_beacon_event(&mut VolatileMacPipeMmio, |bits| unsafe {
        let pending = crate::dtcm::scheduler_pending_events().get() as *mut u32;
        pending.write_volatile(pending.read_volatile() | bits);
    });
}

/// Non-type-`0x37` phase-2 accounting at matching `0x9d9e`.
///
/// # Safety
/// MAC scheduler, statistics, and retained interface state must be mapped.
pub unsafe fn service_mac_irq_count_status(event_type: u8) {
    unsafe {
        if matches!(event_type, 7 | 8 | 0x13 | 0x14) {
            write_u32(0xfff0_1a98, read_u32(0xfff0_1a98).wrapping_add(1));
            return;
        }
        if event_type != 0x19 {
            return;
        }
        if read_u32(crate::dtcm::MAC_WAKE_CONTROL.get()) != 0 {
            write_u32(crate::dtcm::LOW_MAC_BAND_BITS.get(), 1);
        }
        write_u8(crate::dtcm::LOW_MAC_EVENT_PENDING.get(), 1);
        let interface = usize::from(read_u8(crate::dtcm::MAC_PHY_INTERFACE.get()));
        write_u8(
            crate::dtcm::LOW_MAC_SELECTED_RATE.get(),
            read_u8(
                crate::dtcm::pas_stride_view_unchecked(interface)
                    .path_selector_byte()
                    .get(),
            ),
        );
    }
}

/// Non-type-`0x37` phase-3/phase-1-type-`0x19` dispatch at matching `0x9c4e`.
///
/// # Safety
/// MAC, radio-scheduler, beacon, and scheduler state must be mapped.
pub unsafe fn service_mac_nonpipe_completion_event(event_type: u8) {
    unsafe {
        match event_type {
            0x19 => {
                let state = read_u32(crate::dtcm::MAC_BEACON_CONTROL_STATE.get());
                if state == 4 {
                    let pending = crate::dtcm::scheduler_pending_events().get();
                    write_u32(pending, read_u32(pending) | (1 << 24));
                } else if state != 5 {
                    service_mac_beacon_event();
                }
                write_u32(crate::dtcm::MAC_BEACON_CONTROL_STATE.get(), 1);
            }
            0x35 if read_u8(crate::dtcm::radio_timer_state().get()) == 2 => {
                write_u8(crate::dtcm::radio_timer_state().get(), 4);
                let pending = crate::dtcm::scheduler_pending_events().get();
                write_u32(pending, read_u32(pending) | (1 << 31));
            }
            _ => {}
        }
    }
}

/// Direct bit-7 sideband effects from matching FIQ handler `0x9ff8..0xa016`.
///
/// # Safety
/// MAC/DTCM and the vendor statistics alias must be mapped and exclusively
/// owned by event servicing.
pub unsafe fn service_mac_sideband() {
    unsafe {
        let captured = (0x0ab8_0c50 as *const u32).read_volatile();
        (crate::dtcm::MAC_SIDEBAND_CAPTURE.get() as *mut u32).write_volatile(captured);
        let counter = 0xfff0_1a9c as *mut u32;
        counter.write_volatile(counter.read_volatile().wrapping_add(1));
        raise_scheduler_bits(1 << 19);
    }
}

/// Archives the last fully processed non-fatal event.
///
/// # Safety
/// DTCM event state must be mapped.
pub unsafe fn archive_mac_event(raw: u32) {
    unsafe { (crate::dtcm::MAC_BEACON_COMPLETION_WORD.get() as *mut u32).write_volatile(raw) }
}

/// Vendor `0x9070` hardware-idle predicate used only by the event-drain tail.
/// It is diagnostic state and does not release a TX context by itself.
///
/// # Safety
/// MAC MMIO must be mapped.
pub unsafe fn mac_hardware_idle() -> bool {
    unsafe {
        ((crate::platform::mac_register(0x0a28) as *const u32).read_volatile() >> 8) & 0x1f != 0x12
            && ((crate::platform::mac_register(0x0e90) as *const u32).read_volatile() >> 4) & 0xff
                == 0
            && ((crate::platform::mac_register(0x0ea0) as *const u32).read_volatile() >> 24) & 0x1f
                == 0
    }
}

/// Vendor `0x9038` predicate: every software TX pipe record has state byte
/// `pipe+3 == 0`. Like `mac_hardware_idle`, this is only a drain diagnostic.
///
/// # Safety
/// DTCM pipe records must be initialized and exclusively coherent with MAC TX
/// servicing.
pub unsafe fn mac_pipe_records_idle() -> bool {
    for pipe in 0..4_usize {
        if unsafe {
            (crate::dtcm::mac_pipe_state_unchecked(pipe).get() as *const u8).read_volatile()
        } != 0
        {
            return false;
        }
    }
    true
}

fn mac_drain_tail_transition(control: u8, hardware_idle: bool, pipes_idle: bool) -> Option<u8> {
    (control & 2 != 0 && (hardware_idle || pipes_idle)).then_some((control & 0xfd) | 4)
}

/// Exact empty-FIFO tail at matching `0xa02e..0xa05c`.
///
/// # Safety
/// Event servicing must own DTCM scheduler state and MAC MMIO.
pub unsafe fn service_mac_event_drain_tail() {
    unsafe {
        let control = crate::dtcm::mac_retry_hardware_state_ptr()
            .cast::<u8>()
            .read_volatile();
        let next = mac_drain_tail_transition(control, mac_hardware_idle(), mac_pipe_records_idle());
        if let Some(next) = next {
            crate::dtcm::mac_retry_hardware_state_ptr()
                .cast::<u8>()
                .write_volatile(next);
            raise_scheduler_bits(1 << 31);
        }
    }
}

/// Matching-payload class-6 callback at Thumb `0x15426`.
///
/// The callback ignores control subtype `0x50`; otherwise it raises scheduler
/// event bit 10 only while vendor scan-completion state is active. This helper
/// is translated but deliberately not called until the bit-10 task effects are
/// translated as well.
///
/// # Safety
/// The context and scheduler DTCM state must be valid and exclusively owned.
pub unsafe fn service_class6_probe_completion(context: u32) {
    unsafe {
        let header = ((context as usize + 0x1c) as *const u32).read_volatile();
        let frame_type = (header as *const u16).read_volatile() as u8;
        if frame_type != 0x50 && crate::dtcm::shared_ptr::<u8>(crate::dtcm::vendor_scan_state()).read_volatile() != 0 {
            raise_scheduler_bits(1 << 10);
        }
    }
}

/// Observes whether the MAC event FIFO is non-empty without consuming it.
///
/// Vendor FIQ handler `0x9e90` treats `0x09c00a24` only as a signed
/// empty/readiness value and obtains the event from destructive register
/// `0x09c00a20`. Therefore no event can be classified safely at this boundary.
///
/// # Safety
/// The MAC event registers must be mapped.
pub unsafe fn service_probe_mac_events(
    _tracker: &mut ProbeTxTracker,
    max_events: u32,
) -> Result<MacEventServiceReport, ProbeTxTransitionError> {
    let mut report = MacEventServiceReport::default();
    if max_events != 0 {
        let readiness =
            unsafe { (crate::platform::mac_register(0x0a24) as *const i32).read_volatile() };
        if readiness >= 0 {
            report.blocked = Some(readiness as u32);
        }
    }
    Ok(report)
}

impl PreparedProbe {
    pub fn bytes(&self) -> &[u8] {
        &self.bytes[..self.length]
    }

    pub fn rate(&self) -> u8 {
        self.rate
    }
}

pub struct PreparedProbeContext {
    context: u32,
    header: u32,
    length: u16,
    rate: u8,
    expects_ack: bool,
}

impl PreparedProbeContext {
    pub fn context_address(&self) -> u32 {
        self.context
    }

    pub fn header_address(&self) -> u32 {
        self.header
    }

    pub fn frame_length(&self) -> u16 {
        self.length
    }

    pub fn rate(&self) -> u8 {
        self.rate
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SingleProbePublicationInput {
    pub pipe: u8,
    pub slot: u8,
    pub pipe_state: u32,
    pub slot_record: u32,
    pub command_storage: u32,
    pub hardware_ring: u32,
    pub frame_node: FrameNodeAddress,
    pub expects_ack: bool,
    /// Batch position. `txp_scheduler_run` sets `current` to the producer once,
    /// then loops slot state and ring duration from there to `last`, and only
    /// afterwards arms the pipe and writes GO. Staging a second frame must
    /// therefore leave `current` alone (it tracks hardware progress) and must
    /// not arm; the final frame of a batch arms once for all of them.
    pub batch: BatchPosition,
}

/// Where a staged frame sits in a pipe batch. `Only` reproduces the exact
/// single-frame sequence, so a depth-1 build is unchanged.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BatchPosition {
    Only,
    First,
    Middle,
    Last,
}

impl BatchPosition {
    /// Vendor writes `current = producer` once, before the publish loop.
    pub const fn sets_current(self) -> bool {
        matches!(self, Self::Only | Self::First)
    }

    /// Vendor arms and writes GO once, after the publish loop.
    pub const fn arms(self) -> bool {
        matches!(self, Self::Only | Self::Last)
    }
}

/// Final matching-payload publication sequence for one kind-0 no-ACK frame.
/// The caller must validate software ownership before entering this function;
/// after `PIPE_IRQ_TRIGGER` is written, no recoverable error is permitted.
#[cfg(any(target_arch = "arm", test))]
fn single_frame_slot_matches(
    slot_frame_node: u32,
    expected_frame_node: u32,
    slot_kind: u8,
    slot_frame_kind: u8,
    frame_kind: u8,
) -> bool {
    slot_frame_node == expected_frame_node && slot_kind == 0 && slot_frame_kind == frame_kind
}

#[cfg(feature = "vendor-host-tx-diagnostics")]
fn prepare_pre_go_publication<M: MacPipeMmio>(
    mmio: &mut M,
    input: SingleProbePublicationInput,
    ring: TxHardwareRingAddress,
    duration: u32,
) {
    let frame = input.frame_node;
    let descriptor = TxDescriptorAddress::new(input.command_storage);
    let record = crate::dtcm::MacPipeRecordAddress::from_raw_unchecked(input.pipe_state);
    let slot = crate::dtcm::MacPipeSlotAddress::from_raw_unchecked(input.slot_record);
    // Vendor txp_scheduler_run performs no descriptor readback between its
    // trigger and GO writes. Gather the expensive image directly into BSS
    // while the pipe is inactive, avoiding a 152-byte firmware stack copy.
    unsafe {
        crate::host_tx_diagnostics::begin_pre_go_snapshot();
        crate::host_tx_diagnostics::write_pre_go_snapshot_word(
            0,
            mmio.read_u32(slot.state_word().get() as u32) & 0x00ff_ffff | 0x0100_0000,
        );
        crate::host_tx_diagnostics::write_pre_go_snapshot_word(1, duration);
        crate::host_tx_diagnostics::write_pre_go_snapshot_word(
            2,
            mmio.read_u32(slot.frame().get() as u32),
        );
        crate::host_tx_diagnostics::write_pre_go_snapshot_word(
            3,
            mmio.read_u32(slot.auxiliary().get() as u32),
        );
        crate::host_tx_diagnostics::write_pre_go_snapshot_word(
            4,
            mmio.read_u32(slot.command().get() as u32),
        );
        crate::host_tx_diagnostics::write_pre_go_snapshot_word(5, mmio.read_u32(frame.control_bits()));
        crate::host_tx_diagnostics::write_pre_go_snapshot_word(
            6,
            u32::from(mmio.read_u16(frame.frame_length())),
        );
        crate::host_tx_diagnostics::write_pre_go_snapshot_word(
            7,
            u32::from(mmio.read_u16(frame.frame_control())),
        );
        crate::host_tx_diagnostics::write_pre_go_snapshot_word(
            8,
            u32::from(mmio.read_u8(frame.request_flag_rate_bits())),
        );
        crate::host_tx_diagnostics::write_pre_go_snapshot_word(
            9,
            u32::from(mmio.read_u8(frame.tx_rate())),
        );
        crate::host_tx_diagnostics::write_pre_go_snapshot_word(
            10,
            u32::from(mmio.read_u16(frame.duration())),
        );
        crate::host_tx_diagnostics::write_pre_go_snapshot_word(
            11,
            u32::from(mmio.read_u16(frame.payload_extended())),
        );
        crate::host_tx_diagnostics::write_pre_go_snapshot_word(
            12,
            u32::from(mmio.read_u16(frame.payload_base())),
        );
        crate::host_tx_diagnostics::write_pre_go_snapshot_word(13, mmio.read_u32(frame.total_airtime()));
        crate::host_tx_diagnostics::write_pre_go_snapshot_word(
            14,
            u32::from(mmio.read_u8(frame.frame_kind())),
        );
        crate::host_tx_diagnostics::write_pre_go_snapshot_word(
            15,
            u32::from(mmio.read_u8(frame.interface())),
        );
        crate::host_tx_diagnostics::write_pre_go_snapshot_word(
            16,
            u32::from(mmio.read_u8(frame.duration_slot())),
        );
        for index in 0..16_u32 {
            crate::host_tx_diagnostics::write_pre_go_snapshot_word(
                17 + index as usize,
                mmio.read_u32(descriptor.word_unchecked(index)),
            );
        }
        crate::host_tx_diagnostics::write_pre_go_snapshot_word(33, duration);
        crate::host_tx_diagnostics::write_pre_go_snapshot_word(
            34,
            mmio.read_u32(ring.diagnostic_word_0c()),
        );
        crate::host_tx_diagnostics::write_pre_go_snapshot_word(
            35,
            mmio.read_u32(ring.diagnostic_word_10()),
        );
        crate::host_tx_diagnostics::write_pre_go_snapshot_word(36, 0);
        crate::host_tx_diagnostics::write_pre_go_snapshot_word(
            37,
            mmio.read_u32(record.producer_slot().get() as u32) & 0x00ff_ffff | 0x0100_0000,
        );
    }
}

pub fn execute_single_probe_publication<M: MacPipeMmio>(
    mmio: &mut M,
    input: SingleProbePublicationInput,
) -> u8 {
    if input.pipe >= 4 || input.slot >= 4 {
        return u8::MAX;
    }
    let pipe = input.pipe;
    let slot_index = input.slot;
    let record = crate::dtcm::MacPipeRecordAddress::from_raw_unchecked(input.pipe_state);
    let slot = crate::dtcm::MacPipeSlotAddress::from_raw_unchecked(input.slot_record);
    let Some(ring) = TxHardwareRingAddress::for_pipe(pipe, input.hardware_ring) else {
        return u8::MAX;
    };
    let descriptor = TxDescriptorAddress::new(input.command_storage);
    let frame = input.frame_node;

    // `current` tracks hardware progress through the batch, so only the first
    // staged frame sets it (vendor: `*(byte *)(iVar4 + 0xa2) = *pbVar8`, once,
    // before the publish loop).
    if input.batch.sets_current() {
        mmio.write_u8(record.current_slot().get() as u32, slot_index);
    }
    // The inactive first-submission branch of `txp_scheduler_run` does not
    // call `txp_pipe_advance_slot`; startup already synchronized the ring and
    // software cursors. Slot-advance publication belongs only to cleanup/rearm
    // paths where pipe state +3 was already active.
    let ownership_flags = mmio.read_u32(frame.ownership_bits()) | 0x100;
    mmio.write_u32(frame.ownership_bits(), ownership_flags);
    let timestamp = mmio.read_u32(0x0ac0_0004);
    mmio.write_u32(frame.scheduler_timestamp(), timestamp);
    mmio.write_u32(frame.next_in_ampdu(), 0);
    if publication_bisect_reached(4) {
        return 4;
    }

    let timing = SingleFramePasTiming {
        payload_extended: mmio.read_u16(frame.payload_extended()),
        payload_base: mmio.read_u16(frame.payload_base()),
        ack: mmio.read_u16(frame.duration()),
        total_airtime: mmio.read_u32(frame.total_airtime()),
        frame_kind: mmio.read_u8(frame.frame_kind()),
    };
    let expects_ack = timing.frame_kind != 0xff;
    debug_assert_eq!(input.expects_ack, expects_ack);
    mmio.write_u32(slot.duration().get() as u32, single_frame_slot_duration(timing));
    build_single_frame_duration(mmio, descriptor.raw(), input.frame_node, expects_ack);
    if expects_ack {
        let flags = mmio.read_u32(descriptor.flags())
            | u32::from(timing.frame_kind).wrapping_add(0x80);
        mmio.write_u32(descriptor.flags(), flags);
    }
    mmio.write_u8(record.last_slot().get() as u32, slot_index);
    mmio.write_u32(ring.go(), 0);
    #[cfg(feature = "vendor-host-tx-diagnostics")]
    {
        let diagnostic_duration = mmio.read_u32(slot.duration().get() as u32);
        prepare_pre_go_publication(mmio, input, ring, diagnostic_duration);
    }
    if publication_bisect_reached(5) {
        return 5;
    }

    let interface = u32::from(mmio.read_u8(frame.interface()));
    let pas = crate::dtcm::pas_stride_view_unchecked(interface as usize);
    let edca_slot_timing = mmio.read_u32(pas.packed_aifs().get() as u32);
    let edca_slot_timing_cache = crate::dtcm::mac_edca_slot_timing_mmio_address();
    if mmio.read_u32(edca_slot_timing_cache) != edca_slot_timing {
        mmio.write_u32(
            crate::platform::mac_register(0x0e64) as u32,
            edca_slot_timing,
        );
        mmio.write_u32(edca_slot_timing_cache, edca_slot_timing);
    }
    if publication_bisect_reached(6) {
        return 6;
    }

    let queue = u32::from(mmio.read_u8(
        crate::dtcm::queue_to_access_category_unchecked(usize::from(pipe)).get() as u32,
    ));
    let mut quantum =
        u32::from(mmio.read_u16(pas.txop_limit_unchecked(queue as usize).get() as u32));
    let airtime = mmio.read_u32(frame.total_airtime()) & 0xffff;
    if quantum == 0 {
        if (mmio.read_u32(frame.control_bits()) & 0x0fff) >> 10 != 0 {
            quantum = airtime;
        }
    } else if quantum <= airtime {
        let frame_policy = mmio.read_u16(frame.auxiliary_state()) | 8;
        mmio.write_u16(frame.auxiliary_state(), frame_policy);
        quantum = airtime;
    }
    let quantum_destination = mmio.read_u32(
        crate::dtcm::duration_quantum_pointer_unchecked(usize::from(pipe)).get() as u32,
    );
    mmio.write_u32(quantum_destination, quantum.wrapping_add(0x1f) >> 5);
    if publication_bisect_reached(7) {
        return 7;
    }

    if !matches!(input.batch, BatchPosition::Only) {
        // A staged batch remains entirely software-owned until
        // `finalize_staged_pipe`: no trigger, published slot state, duration
        // FIFO write, arm, or GO is allowed per descriptor.
        return 0;
    }

    #[cfg(all(feature = "vendor-host-tx-diagnostics", target_arch = "arm"))]
    unsafe {
        crate::hif::validate_tx_boundary(
            0x13,
            pipe,
            slot_index,
            input.command_storage,
            input.hardware_ring,
        );
    }
    mmio.write_u32(PIPE_IRQ_TRIGGER, (1_u32 << pipe) << 25);
    if publication_bisect_reached(8) {
        return 8;
    }

    let active_count = mmio
        .read_u8(crate::dtcm::LOW_MAC_ACTIVE_TX_COUNT.get() as u32)
        .wrapping_add(1);
    mmio.write_u8(
        crate::dtcm::LOW_MAC_ACTIVE_TX_COUNT.get() as u32,
        active_count,
    );
    // Vendor's publish loop body: mark the slot published and push its duration
    // into the ring, once per slot from producer to `last`.
    mmio.write_u8(slot.state().get() as u32, 1);
    let duration = mmio.read_u32(slot.duration().get() as u32);
    mmio.write_u32(ring.duration_fifo(), duration);
    mmio.write_u8(record.state().get() as u32, 1);
    let pipe_flags = mmio.read_u8(record.control().get() as u32) | 1;
    mmio.write_u8(record.control().get() as u32, pipe_flags);
    mmio.write_u8(record.watchdog().get() as u32, 5);
    #[cfg(feature = "vendor-host-tx-diagnostics")]
    unsafe {
        let actual_ring_duration = mmio.read_u32(ring.duration_fifo());
        crate::host_tx_diagnostics::write_pre_go_snapshot_word(33, actual_ring_duration);
        crate::host_tx_diagnostics::commit_pre_go_snapshot();
    }
    #[cfg(all(feature = "vendor-host-tx-diagnostics", target_arch = "arm"))]
    unsafe {
        crate::hif::validate_tx_boundary(
            0x14,
            pipe,
            slot_index,
            input.command_storage,
            input.hardware_ring,
        );
    }
    mmio.write_u32(ring.go(), 1);
    #[cfg(all(feature = "vendor-host-tx-diagnostics", target_arch = "arm"))]
    unsafe {
        crate::hif::validate_tx_boundary(
            0x15,
            pipe,
            slot_index,
            input.command_storage,
            input.hardware_ring,
        );
    }
    0
}

/// Publish every slot staged by a non-aggregate scheduler batch and arm the
/// pipe once. Descriptor construction and all fallible ownership checks must
/// complete before this boundary.
#[cfg(any(target_arch = "arm", test))]
fn finalize_staged_pipe<M: MacPipeMmio>(
    mmio: &mut M,
    pipe: u8,
    pipe_state: u32,
    hardware_ring: u32,
    first_slot: u8,
    last_slot: u8,
) -> bool {
    if pipe >= 4 || first_slot >= 4 || last_slot >= 4 {
        return false;
    }
    let record = pipe_record_address(pipe);
    let Some(ring) = TxHardwareRingAddress::for_pipe(pipe, hardware_ring) else {
        return false;
    };
    if pipe_state != record.raw() {
        return false;
    }
    mmio.write_u32(PIPE_IRQ_TRIGGER, (1_u32 << pipe) << 25);

    let mut slot_index = first_slot;
    loop {
        let slot = record.slot_unchecked(usize::from(slot_index));
        let active_count = mmio
            .read_u8(crate::dtcm::LOW_MAC_ACTIVE_TX_COUNT.get() as u32)
            .wrapping_add(1);
        mmio.write_u8(
            crate::dtcm::LOW_MAC_ACTIVE_TX_COUNT.get() as u32,
            active_count,
        );
        mmio.write_u8(slot.state().get() as u32, 1);
        let duration = mmio.read_u32(slot.duration().get() as u32);
        mmio.write_u32(ring.duration_fifo(), duration);
        if slot_index == last_slot {
            break;
        }
        slot_index = slot_index.wrapping_add(1) & 3;
    }

    mmio.write_u8(record.state().get() as u32, 1);
    let pipe_flags = mmio.read_u8(record.control().get() as u32) | 1;
    mmio.write_u8(record.control().get() as u32, pipe_flags);
    mmio.write_u8(record.watchdog().get() as u32, 5);
    mmio.write_u32(ring.go(), 1);
    true
}

/// Cross the shared MAC trigger boundary for a fully staged class-0 batch.
///
/// # Safety
/// Every slot from `first_slot` through `last_slot` must be reserved by the
/// caller in this pipe, and no other owner may mutate the pipe or ring.
#[cfg(target_arch = "arm")]
pub unsafe fn finalize_staged_host_class0_pipe(
    _guard: &mut crate::mac_domain::MacDomainGuard<'_>,
    pipe: u8,
    first_slot: u8,
    last_slot: u8,
) {
    if pipe >= 4 {
        crate::halt_always!();
    }
    let pipe_state = pipe_state_address(pipe);
    let hardware_ring = unsafe { read_u32(pipe_state as usize + 8) };
    if !finalize_staged_pipe(
        &mut VolatileMacPipeMmio,
        pipe,
        pipe_state,
        hardware_ring,
        first_slot,
        last_slot,
    ) {
        crate::halt_always!();
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PublishedProbePublication {
    pub context: ContextAddress,
    pub pipe: u8,
    pub slot: u8,
    pub checksum: u32,
    pub bisect_stage: u8,
}

/// Software-owned pipe reservation with a fully written command list.
///
/// The slot points at the context, but neither the hardware ring nor the MAC
/// trigger has been published. Consuming this value through `cancel` restores
/// the slot and returns the context exactly once.
pub struct PreparedProbePublication {
    context: PreparedProbeContext,
    pipe: u8,
    slot: u8,
    slot_record: crate::dtcm::MacPipeSlotAddress,
    command: TxDescriptorAddress,
    original_slot_header: u32,
    original_slot_frame: u32,
    original_command: [u32; 16],
    checksum: u32,
    start_phy: bool,
}

impl PreparedProbePublication {
    pub fn context_address(&self) -> u32 {
        self.context.context
    }

    pub fn pipe(&self) -> u8 {
        self.pipe
    }

    pub fn slot(&self) -> u8 {
        self.slot
    }

    pub fn checksum(&self) -> u32 {
        self.checksum
    }

    /// Transfers the prepared single probe to MAC ownership. This is target-
    /// only and remains uncalled by the firmware main loop.
    ///
    /// # Safety
    /// The reservation and all pipe/MMIO state must remain exclusively owned;
    /// the concrete event and completion services must run after publication.
    #[cfg(target_arch = "arm")]
    pub unsafe fn publish(
        self,
        backend: &mut SingleProbeMacBackend,
    ) -> Result<PublishedProbePublication, ProbeBuildError> {
        unsafe {
            let frame_node = FrameNodeAddress::new(self.context.context + FRAME_NODE_OFFSET);
            let context_address = frame_node.context();
            let valid = single_frame_slot_matches(
                read_u32(self.slot_record.frame().get()),
                frame_node.raw(),
                read_u8(self.slot_record.kind().get()),
                read_u8(self.slot_record.retry_rate().get()),
                read_u8(context_address.retry_rate_address()),
            );
            if !valid {
                let cancellation = self.cancel();
                return Err(cancellation
                    .err()
                    .unwrap_or(ProbeBuildError::UnsupportedPublicationShape));
            }

            let publication = |bisect_stage| PublishedProbePublication {
                context: ContextAddress::new(self.context.context),
                pipe: self.pipe,
                slot: self.slot,
                checksum: self.checksum,
                bisect_stage,
            };

            if !backend.register_publication(
                BatchPosition::Only,
                ContextAddress::new(self.context.context),
                self.pipe,
                self.slot,
            ) {
                let cancellation = self.cancel();
                return Err(cancellation
                    .err()
                    .unwrap_or(ProbeBuildError::InvalidContextPointer));
            }
            if publication_bisect_reached(3) {
                return Ok(publication(3));
            }
            if self.start_phy {
                let phy_bisect_stage = start_phy_operation_1();
                if phy_bisect_stage != 0 {
                    return Ok(publication(phy_bisect_stage));
                }
            }
            let pipe_state = pipe_state_address(self.pipe);
            let hardware_ring = read_u32(pipe_state as usize + 8);
            let live_command = read_u32(self.slot_record.command().get());
            if self.command.raw() != live_command
                || packet_ram::tx_command_index(self.command.raw() as usize)
                    != Some((usize::from(self.pipe), usize::from(self.slot)))
            {
                crate::hif::publish_halting_exception(
                    [
                        0x5458_4341,
                        u32::from(self.pipe),
                        u32::from(self.slot),
                        self.slot_record.raw(),
                        self.command.raw(),
                        live_command,
                        pipe_state,
                        hardware_ring,
                        read_u32(self.slot_record.state_word().get()),
                        read_u32(self.slot_record.duration().get()),
                        read_u32(self.slot_record.frame().get()),
                        read_u32(self.slot_record.auxiliary().get()),
                        read_u32(pipe_state as usize),
                        read_u32(pipe_state as usize + 4),
                        read_u32(pipe_state as usize + 8),
                        read_u32(pipe_state as usize + 0x0c + 0x14),
                        read_u32(pipe_state as usize + 0x24 + 0x14),
                        read_u32(pipe_state as usize + 0x3c + 0x14),
                    ],
                    b"xr819-tx-command-address",
                );
                crate::halt_always!();
            }
            if TxHardwareRingAddress::for_pipe(self.pipe, hardware_ring).is_none() {
                let cancellation = self.cancel();
                return Err(cancellation
                    .err()
                    .unwrap_or(ProbeBuildError::PipeStateUnavailable));
            }
            if publication_bisect_reached(5) {
                return Ok(publication(5));
            }
            if publication_bisect_reached(6) {
                return Ok(publication(6));
            }
            // Vendor queue accounting increments the global active-completion
            // count before hardware ownership. `service_completion_drain`
            // performs the matching decrement before callback return.
            set_active_pas_contexts(active_pas_contexts().wrapping_add(1));
            if publication_bisect_reached(7) {
                return Ok(publication(7));
            }
            reset_tx_trace(self.pipe, self.slot);
            if publication_bisect_reached(8) {
                return Ok(publication(8));
            }
            #[cfg(feature = "vendor-host-tx-diagnostics")]
            crate::radio::record_tx_command_signature(6, self.command.raw());
            execute_single_probe_publication(
                &mut VolatileMacPipeMmio,
                SingleProbePublicationInput {
                    pipe: self.pipe,
                    slot: self.slot,
                    pipe_state,
                    slot_record: self.slot_record.raw(),
                    command_storage: self.command.raw(),
                    hardware_ring,
                    frame_node,
                    expects_ack: self.context.expects_ack,
                    batch: BatchPosition::Only,
                },
            );
            // Mirror the execution record through ordinary WSM event
            // indications so the host can dump it without resetting the core.
            trace_tx_stage(TX_TRACE_GO);
            Ok(publication(0))
        }
    }

    /// Cancels a reservation that has not been transferred to hardware.
    ///
    /// # Safety
    /// The reservation must still be software-owned and its pipe must not have
    /// been published or triggered.
    pub unsafe fn cancel(self) -> Result<u32, ProbeBuildError> {
        unsafe {
            let expected = ContextAddress::new(self.context.context).frame_node().raw();
            if (self.slot_record.frame().get() as *const u32).read_volatile() != expected {
                return Err(ProbeBuildError::PipeSlotOwnershipMismatch);
            }
            self.slot_record
                .frame()
                .cast_mut::<u32>()
                .write_volatile(self.original_slot_frame);
            self.slot_record
                .state_word()
                .cast_mut::<u32>()
                .write_volatile(self.original_slot_header);
            for (index, word) in self.original_command.into_iter().enumerate() {
                (self.command.word_unchecked(index as u32) as *mut u32).write_volatile(word);
            }
            if let Some(context) = crate::dtcm::host_context_from_raw(self.context.context) {
                release_wsm_context_address(context);
            } else {
                release_context_address(self.context.context);
            }
        }
        Ok(self.checksum)
    }
}

unsafe fn prepare_single_frame_pas_timing(
    context: &mut PreparedProbeContext,
) -> Result<(), ProbeBuildError> {
    unsafe {
        let frame = FrameNodeAddress::new(context.context.wrapping_add(FRAME_NODE_OFFSET));
        let interface = usize::from(read_u8(frame.interface() as usize));
        if interface > 2 {
            return Err(ProbeBuildError::InvalidInterface);
        }
        let flags = read_u32(frame.control_bits() as usize);
        let rate = read_u8(frame.tx_rate() as usize);
        let pas = crate::dtcm::pas_stride_view_unchecked(interface);
        let rate_map = pas.rate_map_unchecked(usize::from(rate));
        let timing_index = usize::from(read_u8(rate_map.get()));
        // Preserve frame+0x0d exactly as initialized by the vendor HIF path
        // from `(wsm_tx_flags & 0x0f) >> 1`. `txp_submit_to_pipe()` passes this
        // rate attribute directly to `pas_build_phy_rate_words()`. The rate
        // map index below selects ACK timing only; writing it into frame+0x0d
        // changed healthy vendor `0x5104....` words into `0x5107....`.
        let ack_duration = read_u16(if flags & 0x4000 != 0 {
            crate::dtcm::low_mac_long_airtime_unchecked(timing_index).get()
        } else {
            crate::dtcm::low_mac_short_airtime_unchecked(timing_index).get()
        });
        let mode = read_u8(pas.mode_byte().get());
        let header = validated_context_tx_frame(frame.context())
            .ok_or(ProbeBuildError::PacketRamMismatch)?;
        let special_peer = (mode == 5 || mode == 6)
            && (0..6).all(|offset| {
                read_u8(header.address_2_byte_unchecked(offset as u32) as usize)
                    == read_u8(
                        crate::dtcm::low_mac_peer_address_byte_unchecked(interface, offset).get(),
                    )
            });
        let timing = compute_single_frame_pas_timing(
            read_u16(crate::dtcm::LOW_MAC_RATE_CONFIG.get()),
            rate,
            read_u16(frame.frame_length() as usize),
            flags,
            ack_duration,
            special_peer,
        )
        .ok_or(ProbeBuildError::UnsupportedPublicationShape)?;

        write_u16(frame.timing_reset_32() as usize, 0);
        write_u16(frame.timing_reset_34() as usize, 0);
        write_u16(frame.duration() as usize, timing.ack);
        write_u16(frame.payload_extended() as usize, timing.payload_extended);
        write_u16(frame.payload_base() as usize, timing.payload_base);
        write_u32(frame.total_airtime() as usize, timing.total_airtime);
        write_u8(frame.frame_kind() as usize, timing.frame_kind);
        context.expects_ack = timing.frame_kind != 0xff;
    }
    Ok(())
}

/// Pops and initializes one internal management context using the field order
/// from `tx_ctx_alloc_init(6, 0, 1)` and the probe-specific tail of `0x141b0`.
/// No queue or hardware ownership is transferred.
///
/// # Safety
/// The internal pool must have been initialized and exclusively owned by the
/// firmware runtime.
pub unsafe fn prepare_probe_context(
    probe: &PreparedProbe,
    if_id: u8,
) -> Result<PreparedProbeContext, ProbeBuildError> {
    if probe.length > TX_BUFFER_SIZE - 0x40 || probe.length > usize::from(u16::MAX) {
        return Err(ProbeBuildError::FrameTooLarge);
    }
    unsafe {
        let free_head = internal_context_free_head();
        let context = free_head.read_volatile();
        if context == 0 {
            return Err(ProbeBuildError::ContextPoolEmpty);
        }
        let context_address = ContextAddress::new(context);
        free_head.write_volatile(read_u32(context_address.intrusive_next_address()));

        set_active_internal_contexts(active_internal_contexts().wrapping_add(1));
        write_u8(context_address.queue_id_address(), 0);
        write_u32(context_address.optional_pipe_object_address(), 0);
        write_u8(context_address.completion_class_address(), 6);
        write_u8(context_address.submit_state_address(), 1);
        let sequence = probe_context_sequence();
        write_u16(context_address.sequence_or_callback_state_address(), sequence);
        set_probe_context_sequence(sequence.wrapping_add(1));
        write_u8(context_address.request_flags_address(), 0);
        write_u8(context_address.insertion_mode_address(), 1);
        write_u32(context_address.control_bits_address(), 0);
        write_u16(context_address.terminal_status_address(), 0x00fe);
        write_u16(context_address.try_count_address(), 0);
        write_u16(context_address.auxiliary_state_address(), 0);
        write_u32(context_address.ownership_bits_address(), 1);
        write_u32(context_address.next_in_ampdu_address(), 0);
        // Vendor sources `ctx+0x98` from a ROM-owned pointer. Preserve the
        // pool value until that ROM/global state is translated; zero is not a
        // reference-faithful substitute once the context becomes live.
        write_u8(
            context_address.access_category_address(),
            crate::dtcm::shared_ptr::<u8>(crate::dtcm::queue_to_access_category_unchecked(0))
                .read_volatile(),
        );
        write_u8(context_address.request_flag_rate_bits_address(), 0);

        let header = read_u32(context_address.borrowed_frame_address_address());
        let frame = TxFrameAddress::new(header);
        if expected_header_address(context) != Some(header) {
            release_context_address(context);
            return Err(ProbeBuildError::InvalidContextPointer);
        }
        copy_to_packet_ram(frame.raw(), probe.bytes());
        if !packet_ram_matches(frame.raw(), probe.bytes()) {
            release_context_address(context);
            return Err(ProbeBuildError::PacketRamMismatch);
        }
        let if_id = if_id.min(2);
        for word in 0..3 {
            let Some(value) = crate::vif::own_mac_word(if_id, word) else {
                release_context_address(context);
                return Err(ProbeBuildError::InvalidContextPointer);
            };
            (frame.address_2_halfword_unchecked(word as u32) as *mut u16).write_volatile(value);
        }
        write_u16(context_address.frame_length_address(), probe.length as u16);
        write_u8(context_address.host_link_address(), 0x0f);
        write_u8(context_address.interface_address(), if_id);

        // Ordinary foreground scans store explicit rate 0xff, after which
        // `lmc_tx_assign_default_rate` resolves the VIF default.
        let default_index = usize::from(crate::vif::rate_byte(if_id, 6).unwrap_or(0) != 0);
        let mut rate = crate::vif::default_rate(if_id, default_index).unwrap_or(0);
        if rate == 0xff || crate::vif::rate_byte(if_id, 7).unwrap_or(0) < 3 {
            rate = crate::vif::rate_byte(if_id, 4).unwrap_or(0);
        }
        let mut flags = 0_u32;
        if rate & 0x80 != 0 {
            rate &= 0x7f;
        } else {
            flags |= 8;
        }
        write_u8(context_address.requested_rate_address(), rate);
        write_u8(context_address.tx_rate_address(), rate);
        write_u8(context_address.retry_policy_address(), 0x0f);
        write_u8(context_address.byte_57_address(), 0xff);
        write_u32(context_address.expiry_time_address(), 0);
        write_u32(context_address.frame_address_address(), frame.raw());
        flags |= 0x1000;
        if (frame.address_1() as *const u32).read_volatile() & 1 != 0 {
            flags |= 0x300;
        }
        write_u32(context_address.control_bits_address(), flags);
        write_u32(context_address.ownership_bits_address(), 3);

        let frame_control = (frame.frame_control() as *const u16).read_volatile();
        write_u16(context_address.frame_control_address(), frame_control);
        write_u32(
            context_address.header_length_address(),
            u32::from(DOT11_FIXED_HEADER_LENGTH),
        );
        write_u32(
            context_address.payload_length_address(),
            (probe.length as u32).saturating_sub(u32::from(DOT11_FIXED_HEADER_LENGTH)),
        );
        let duration_slot = (crate::dtcm::pas_stride_view_unchecked(usize::from(if_id))
            .slot_bits()
            .get() as *const u8)
            .read_volatile()
            & 1;
        write_u8(context_address.duration_slot_address(), duration_slot);
        write_u8(context_address.retry_rate_address(), 0xff);
        write_u16(context_address.qos_control_address(), 0);
        write_u8(context_address.cipher_class_address(), 9);
        write_u16(context_address.word_7c_address(), 0x10);
        let mut prepared = PreparedProbeContext {
            context,
            header: frame.raw(),
            length: probe.length as u16,
            rate,
            expects_ack: false,
        };
        if let Err(error) = prepare_single_frame_pas_timing(&mut prepared) {
            release_context_address(context);
            return Err(error);
        }
        Ok(prepared)
    }
}

#[cfg(target_arch = "arm")]
unsafe fn move_to_wsm_class0_context(
    source: PreparedProbeContext,
) -> Result<PreparedProbeContext, ProbeBuildError> {
    unsafe {
        let destination = match crate::vendor_host_tx::allocate_host_context() {
            Ok(context) => context,
            Err(_) => {
                release_context_address(source.context);
                return Err(ProbeBuildError::ContextPoolEmpty);
            }
        };
        let destination_address = destination.raw() as usize;
        let destination_request =
            crate::dtcm::shared_ptr::<u32>(destination.request_buffer()).read_volatile();
        let destination_frame_state =
            crate::dtcm::shared_ptr::<u32>(destination.frame_state_address()).read_volatile();
        if packet_ram::host_frame_state_index(destination_frame_state as usize).is_none() {
            release_context_address(source.context);
            release_wsm_context_address(destination);
            return Err(ProbeBuildError::InvalidContextPointer);
        }
        for offset in (0..TX_CONTEXT_SIZE).step_by(4) {
            ((destination_address + offset) as *mut u32)
                .write_volatile(((source.context as usize + offset) as *const u32).read_volatile());
        }

        // The vendor host path borrows the HIF frame until class-0 completion.
        // Our transformed frame lives in the internal context's packet-RAM
        // buffer, so retain that context as the host descriptor's backing
        // owner instead of guessing a nonexistent host-pool buffer mapping.
        crate::dtcm::shared_ptr::<u32>(destination.request_buffer()).write_volatile(destination_request);
        crate::dtcm::shared_ptr::<u8>(destination.request_flags()).write_volatile(0);
        crate::dtcm::shared_ptr::<u32>(destination.borrowed_frame_address()).write_volatile(source.header);
        crate::dtcm::shared_ptr::<u32>(destination.completion_status()).write_volatile(0xfe);
        crate::dtcm::shared_ptr::<u32>(destination.frame_address()).write_volatile(source.header);
        crate::dtcm::shared_ptr::<u16>(destination.terminal_status()).write_volatile(0xfe);
        crate::dtcm::shared_ptr::<u32>(destination.frame_state_address()).write_volatile(destination_frame_state);
        crate::dtcm::shared_ptr::<u8>(destination.completion_class()).write_volatile(0);
        crate::dtcm::shared_ptr::<u8>(destination.host_link()).write_volatile(0);
        crate::dtcm::shared_ptr::<u32>(destination.ownership_bits()).write_volatile(3);

        Ok(PreparedProbeContext {
            context: destination.raw(),
            ..source
        })
    }
}

/// Returns a context that has never been queued or published.
///
/// # Safety
/// `context` must still be software-owned and must not have entered a queue.
pub unsafe fn release_unpublished_probe_context(context: PreparedProbeContext) {
    unsafe {
        if let Some(host) = crate::dtcm::host_context_from_raw(context.context) {
            release_wsm_context_address(host);
        } else {
            release_context_address(context.context);
        }
    }
}

/// Builds the single-frame command list from a prepared context using the live
/// vendor rate tables and PAS fields. The returned words are still
/// software-owned and are not copied into a pipe descriptor.
///
/// # Safety
/// `context` must remain prepared and software-owned; its DTCM and packet-RAM
/// pointers must be valid.
pub unsafe fn build_prepared_probe_descriptor(
    context: &PreparedProbeContext,
) -> Result<SingleFramePipeDescriptor, ProbeBuildError> {
    unsafe {
        let address = ContextAddress::new(context.context);
        let rate = (address.tx_rate_address() as *const u8).read_volatile();
        let tx_flags = (address.control_bits_address() as *const u32).read_volatile();
        let request_flag_rate_bits =
            (address.request_flag_rate_bits_address() as *const u8).read_volatile();
        let legacy_mode = (crate::dtcm::LOW_MAC_LEGACY_MODE.get() as *const u8).read_volatile();
        let rate_attribute = crate::dtcm::rate_encoding_unchecked(usize::from(rate))
            .cast_mut::<u8>()
            .read_volatile();
        let hardware_rate = crate::dtcm::rate_attribute_unchecked(usize::from(rate))
            .cast_mut::<u8>()
            .read_volatile();
        let phy = build_phy_rate_words(
            rate,
            legacy_mode,
            tx_flags,
            request_flag_rate_bits,
            rate_attribute,
        );
        let if_id = (address.interface_address() as *const u8).read_volatile();
        if usize::from(if_id) >= packet_ram::INTERFACE_METADATA_SIZE {
            return Err(ProbeBuildError::PacketRamMismatch);
        }
        let metadata_address = packet_ram::RuntimePacketAddress::new(
            packet_ram::interface_metadata_byte(usize::from(if_id)) as u32,
            1,
        )
        .ok_or(ProbeBuildError::PacketRamMismatch)?;
        let header_address = packet_ram::RuntimePacketAddress::new(
            context.header,
            usize::from(context.length),
        )
        .ok_or(ProbeBuildError::PacketRamMismatch)?;
        let duration_slot = (address.duration_slot_address() as *const u8).read_volatile();
        if usize::from(duration_slot) >= packet_ram::DURATION_WORD_COUNT {
            return Err(ProbeBuildError::PacketRamMismatch);
        }
        // `txp_submit_to_pipe` uses PAS `bVifSlot` (`ctx+0xbe`) here when
        // flags bit 0 is clear. Both internal and host contexts use this
        // selector; the TX rate indexes different PHY tables.
        let secondary_address = packet_ram::duration_word(usize::from(duration_slot)) as u32;
        build_single_frame_pipe_descriptor(SingleFramePipeInput {
            phy_rate_word: phy.rate,
            phy_control_word: finalize_phy_control(phy, rate, context.length),
            frame_length: context.length,
            hardware_rate,
            frame_control: (address.frame_control_address() as *const u16).read_volatile(),
            retry_flag: (address.control_bits_address() as *const u32).read_volatile() & 0x10 != 0,
            metadata_address,
            duration: (address.duration_address() as *const u16).read_volatile(),
            header_address,
            secondary_command: 0x2100_0000
                | packet_ram::encode_mac_packet_offset_u32(secondary_address),
            address_mask: 0x007f_fffc,
            terminal_command: 0x0700_4600,
        })
    }
}

unsafe fn emit_prepared_probe_descriptor(
    context: &PreparedProbeContext,
    destination: u32,
) -> Result<u32, ProbeBuildError> {
    unsafe {
        let address = ContextAddress::new(context.context);
        let frame = validated_tx_frame(context.header, context.length)
            .ok_or(ProbeBuildError::PacketRamMismatch)?;
        let descriptor_address = packet_ram::RuntimePacketAddress::new(
            destination,
            core::mem::size_of::<[u32; 13]>(),
        )
        .ok_or(ProbeBuildError::PacketRamMismatch)?;
        let descriptor = TxDescriptorAddress::new(descriptor_address.raw());
        let rate = (address.tx_rate_address() as *const u8).read_volatile();
        let tx_flags = (address.control_bits_address() as *const u32).read_volatile();
        let request_flag_rate_bits =
            (address.request_flag_rate_bits_address() as *const u8).read_volatile();
        let legacy_mode = (crate::dtcm::LOW_MAC_LEGACY_MODE.get() as *const u8).read_volatile();
        let rate_attribute = crate::dtcm::rate_encoding_unchecked(usize::from(rate))
            .cast_mut::<u8>()
            .read_volatile();
        let hardware_rate = crate::dtcm::rate_attribute_unchecked(usize::from(rate))
            .cast_mut::<u8>()
            .read_volatile();
        let phy = build_phy_rate_words(
            rate,
            legacy_mode,
            tx_flags,
            request_flag_rate_bits,
            rate_attribute,
        );
        let if_id = (address.interface_address() as *const u8).read_volatile();
        if usize::from(if_id) >= packet_ram::INTERFACE_METADATA_SIZE {
            return Err(ProbeBuildError::PacketRamMismatch);
        }
        let metadata_address = packet_ram::RuntimePacketAddress::new(
            packet_ram::interface_metadata_byte(usize::from(if_id)) as u32,
            1,
        )
        .ok_or(ProbeBuildError::PacketRamMismatch)?;
        let duration_slot = (address.duration_slot_address() as *const u8).read_volatile();
        if usize::from(duration_slot) >= packet_ram::DURATION_WORD_COUNT {
            return Err(ProbeBuildError::PacketRamMismatch);
        }
        let frame_control = u32::from((address.frame_control_address() as *const u16).read_volatile())
            | if (address.control_bits_address() as *const u32).read_volatile() & 0x10 != 0 {
                0x0800
            } else {
                0
            };
        let mut checksum = 0_u32;
        let mut word_index = 0_u32;
        let mut mismatch = false;
        let mut add = |word: u32| {
            let pointer = descriptor.word_unchecked(word_index) as *mut u32;
            pointer.write_volatile(word);
            mismatch |= pointer.read_volatile() != word;
            word_index += 1;
            checksum = checksum.rotate_left(5).wrapping_add(word);
        };
        add(0x5100_0000 | (phy.rate & 0x00ff_ffff));
        add(0x5000_0000 | (finalize_phy_control(phy, rate, context.length) & 0x00ff_ffff));
        add(0x5200_0000 | (u32::from(hardware_rate) << 16) | u32::from(context.length + 4));
        add(0x3100_0000 + frame_control);
        add(0x4700_0000 + (frame_control >> 8));
        add(0x2080_0000 | metadata_address.mac_offset().raw());
        add(0x3200_0000 | u32::from((address.duration_address() as *const u16).read_volatile()));
        add(0x2900_0000 | packet_ram::encode_mac_packet_offset_u32(frame.descriptor_tail()));
        add(single_frame_secondary_command(
            tx_flags,
            (frame.sequence_control() as *const u16).read_volatile(),
            duration_slot,
        ));
        // The hardware descriptor always splits after the fixed three-address
        // 24-byte prefix. Optional address-4, QoS, and HT-control bytes remain
        // in the payload segment; this is independent of the context's parsed
        // 24/26/30/32/36-byte software header length.
        if context.length > DOT11_FIXED_HEADER_LENGTH {
            let payload = packet_ram::RuntimePacketAddress::new(
                frame.payload_after_fixed_header(),
                usize::from(context.length - DOT11_FIXED_HEADER_LENGTH),
            )
            .ok_or(ProbeBuildError::PacketRamMismatch)?;
            add(0x4000_0000 | payload.tx_payload_bus_address(0x007f_fffc).raw());
            add(
                (u32::from(context.length - DOT11_FIXED_HEADER_LENGTH) & 0x0fff) << 12
                    | (payload.raw() & 3),
            );
        }
        add(0x0700_4600);
        add(0xf000_0000);
        if mismatch {
            Err(ProbeBuildError::DescriptorReadbackMismatch)
        } else {
            Ok(checksum)
        }
    }
}

/// Build the reusable descriptor image produced by vendor
/// `txp_submit_to_pipe(ctx+0xa0, ctx+0x54, ctx+0x8a)` for one real host
/// context. This does not publish a pipe slot or transfer scheduler ownership.
///
/// # Safety
/// `context` must be an exclusively owned class-0 host context whose header,
/// length, rate, classification, and duration fields are initialized.
#[inline(always)]
fn validated_host_context(context: u32) -> Option<crate::dtcm::HostContextAddress> {
    crate::dtcm::host_context_from_raw(context)
}

#[inline(never)]
fn host_prepared_context(context: u32) -> Result<PreparedProbeContext, ProbeBuildError> {
    let address = crate::dtcm::host_context_from_raw(context)
        .ok_or(ProbeBuildError::InvalidContextPointer)?;
    Ok(PreparedProbeContext {
        context,
        header: unsafe { crate::dtcm::shared_ptr::<u32>(address.frame_address()).read_volatile() },
        length: unsafe { crate::dtcm::shared_ptr::<u16>(address.frame_length()).read_volatile() },
        rate: unsafe { crate::dtcm::shared_ptr::<u8>(address.tx_rate()).read_volatile() },
        expects_ack: unsafe {
            crate::dtcm::shared_ptr::<u32>(address.control_bits()).read_volatile() & 0x300 == 0
        },
    })
}

/// Compute the PAS timing image after classification and software crypto.
///
/// # Safety
/// `context` must be exclusively owned and not yet linked into the pending
/// list or global PAS ring.
pub unsafe fn prepare_host_frame_timing(context: u32) -> Result<(), ProbeBuildError> {
    let mut prepared = host_prepared_context(context)?;
    unsafe { prepare_single_frame_pas_timing(&mut prepared) }
}

pub unsafe fn emit_host_frame_descriptor_at(
    context: u32,
    destination: u32,
) -> Result<u32, ProbeBuildError> {
    let prepared = host_prepared_context(context)?;
    unsafe { emit_prepared_probe_descriptor(&prepared, destination) }
}

pub unsafe fn build_host_frame_descriptor(context: u32) -> Result<u32, ProbeBuildError> {
    let address = validated_host_context(context)
        .ok_or(ProbeBuildError::InvalidContextPointer)?;
    let destination = unsafe {
        crate::dtcm::shared_ptr::<u32>(address.frame_state_address()).read_volatile()
    };
    if packet_ram::host_frame_state_index(destination as usize).is_none() {
        return Err(ProbeBuildError::InvalidContextPointer);
    }
    unsafe { emit_host_frame_descriptor_at(context, destination) }
}

/// Builds a software-owned pipe reservation without publishing hardware state.
///
/// # Safety
/// The internal context pool, retained template storage, and selected TX pipe
/// must be exclusively owned by the firmware runtime.
pub unsafe fn prepare_probe_publication(
    template: Option<&[u8]>,
    ssid: &[u8],
    channel: u8,
    if_id: u8,
) -> Result<PreparedProbePublication, ProbeBuildError> {
    let probe = unsafe {
        prepare_probe_into(
            &mut *PREPARED_PROBE_SCRATCH.0.get(),
            template,
            ssid,
            channel,
        )?
    };
    let context = unsafe { prepare_probe_context(probe, if_id) }?;
    unsafe { prepare_context_publication(context) }
}

/// Reproduce the vendor host-frame ownership handoff up to scheduler selection:
/// post-crypto ready bit, pending-list insertion/removal by `task_b88e`, then
/// `tx_frame_done_release -> pas_retime_and_kick` through the 64-entry PAS ring.
/// The caller still owns the context after this diagnostic round trip.
#[cfg(target_arch = "arm")]
unsafe fn vendor_queue_handoff_before_direct_publication(
    context: u32,
) -> Result<(), ProbeBuildError> {
    unsafe {
        let Some(host) = crate::dtcm::host_context_from_raw(context) else {
            return Err(ProbeBuildError::InvalidContextPointer);
        };
        let previous = mask_irq_fiq_terminal();
        let (pending_head, pending_tail) = (crate::dtcm::pending_tx_head().get(), crate::dtcm::pending_tx_tail().get());
        let old_head = read_u32(pending_head);
        let old_tail = read_u32(pending_tail);

        // `txq_list_insert(context, queue, 2)` prepends to the pending list.
        write_u32(host.intrusive_next().get(), old_head);
        if old_tail == 0 {
            write_u32(pending_tail, context);
        }
        write_u32(pending_head, context);
        write_u32(
            host.ownership_bits().get(),
            read_u32(host.ownership_bits().get()) | 0x20,
        );

        // The joined/active task accepts this frame, removes the same head,
        // and passes it through `tx_frame_done_release`.
        let next = read_u32(host.intrusive_next().get());
        write_u32(pending_head, next);
        if read_u32(pending_tail) == context {
            write_u32(pending_tail, if next == 0 { 0 } else { old_tail });
        }
        write_u32(host.intrusive_next().get(), 0);
        write_u32(
            host.ownership_bits().get(),
            read_u32(host.ownership_bits().get()) | 0x40,
        );

        // `pas_txq_push_global(context + 0x54)` appends class-0 host frames.
        let head = read_u32(crate::dtcm::HOST_PAS_RING_HEAD.get()) as u8 & 0x3f;
        let tail = read_u32(crate::dtcm::HOST_PAS_RING_TAIL.get()) as u8 & 0x3f;
        let following = tail.wrapping_add(1) & 0x3f;
        if head != tail || following == head {
            restore_irq_fiq(previous);
            return Err(ProbeBuildError::PipeSlotBusy);
        }
        let frame_node = host.frame_node().raw();
        write_u32(
            crate::dtcm::host_pas_ring_slot_unchecked(usize::from(tail)).get(),
            frame_node,
        );
        write_u32(crate::dtcm::HOST_PAS_RING_TAIL.get(), u32::from(following));

        // The minimum scheduler diagnostic consumes exactly the frame it just
        // enqueued, preserving an otherwise-empty ring for the direct backend.
        let selected = read_u32(
            crate::dtcm::host_pas_ring_slot_unchecked(usize::from(head)).get(),
        );
        if selected != frame_node {
            restore_irq_fiq(previous);
            return Err(ProbeBuildError::UnsupportedPublicationShape);
        }
        write_u32(
            crate::dtcm::host_pas_ring_slot_unchecked(usize::from(head)).get(),
            0,
        );
        write_u32(
            crate::dtcm::HOST_PAS_RING_HEAD.get(),
            u32::from(head.wrapping_add(1) & 0x3f),
        );
        // The non-aggregate scheduler branch marks the selected frame before
        // `txp_build_pipe_descriptor(..., 0)`.
        write_u32(
            host.control_bits().get(),
            read_u32(host.control_bits().get()) | (1 << 26),
        );
        restore_irq_fiq(previous);
    }
    Ok(())
}

unsafe fn prepare_context_publication(
    context: PreparedProbeContext,
) -> Result<PreparedProbePublication, ProbeBuildError> {
    unsafe {
        let context_address = ContextAddress::new(context.context);
        let queue = read_u8(context_address.access_category_address());
        let pipe =
            crate::dtcm::shared_ptr::<u8>(crate::dtcm::access_category_to_queue_unchecked(
                usize::from(queue),
            ))
            .read_volatile();
        if pipe >= 4 {
            release_unpublished_probe_context(context);
            return Err(ProbeBuildError::PipeStateUnavailable);
        }
        let pipe_index = usize::from(pipe);
        let hardware = crate::dtcm::shared_ptr::<u32>(
            crate::dtcm::mac_pipe_hardware_ring_unchecked(pipe_index),
        )
        .read_volatile();
        let slot = crate::dtcm::shared_ptr::<u8>(
            crate::dtcm::mac_pipe_current_slot_unchecked(pipe_index),
        )
        .read_volatile()
            & 3;
        let slot_index = usize::from(slot);
        let slot_record = crate::dtcm::MacPipeSlotAddress::from_raw_unchecked(
            crate::dtcm::mac_pipe_slot_state_word_unchecked(pipe_index, slot_index).get() as u32,
        );
        let command = crate::dtcm::shared_ptr::<u32>(
            crate::dtcm::mac_pipe_slot_command_unchecked(pipe_index, slot_index),
        )
        .read_volatile();
        if hardware == 0 || command == 0 {
            release_unpublished_probe_context(context);
            return Err(ProbeBuildError::PipeStateUnavailable);
        }
        // Startup imports persistent slot tails from vendor state. They are not
        // zero-valued idle sentinels, so detached reservation must preserve and
        // restore them rather than infer ownership from their contents.
        let original_slot_header = read_u32(slot_record.state_word().get());
        let original_slot_frame = crate::dtcm::shared_ptr::<u32>(
            crate::dtcm::mac_pipe_slot_frame_unchecked(pipe_index, slot_index),
        )
        .read_volatile();
        let command = TxDescriptorAddress::new(command);
        let mut original_command = [0_u32; 16];
        for (index, word) in original_command.iter_mut().enumerate() {
            *word = (command.word_unchecked(index as u32) as *const u32).read_volatile();
        }
        write_u32(
            context_address.ownership_bits_address(),
            read_u32(context_address.ownership_bits_address()) | 0x100,
        );
        // `0xa712` receives the frame node at `context+0x54`, not the context
        // base. Its kind-0 branch stores frame-node `+0x56` (context `+0xaa`)
        // in `slot+1`; success handler `0x9cdc` requires this marker to be
        // `0xff` before entering the release loop.
        slot_record.kind().cast_mut::<u8>().write_volatile(0);
        crate::dtcm::shared_ptr::<u8>(
            crate::dtcm::mac_pipe_slot_retry_rate_unchecked(pipe_index, slot_index),
        )
        .write_volatile(read_u8(context_address.retry_rate_address()));
        crate::dtcm::shared_ptr::<u8>(
            crate::dtcm::mac_pipe_slot_control_02_unchecked(pipe_index, slot_index),
        )
        .write_volatile(0);
        crate::dtcm::shared_ptr::<u8>(
            crate::dtcm::mac_pipe_slot_control_03_unchecked(pipe_index, slot_index),
        )
        .write_volatile(0);
        crate::dtcm::shared_ptr::<u32>(
            crate::dtcm::mac_pipe_slot_frame_unchecked(pipe_index, slot_index),
        )
        .write_volatile(context_address.frame_node().raw());
        (command.word_unchecked(0) as *mut u32).write_volatile(0);
        (command.word_unchecked(1) as *mut u32).write_volatile(0);
        (command.word_unchecked(2) as *mut u32).write_volatile(0xdc00_0000);
        let checksum = match emit_prepared_probe_descriptor(&context, command.word_unchecked(3)) {
            Ok(checksum) => checksum,
            Err(error) => {
                crate::dtcm::shared_ptr::<u32>(
                    crate::dtcm::mac_pipe_slot_frame_unchecked(pipe_index, slot_index),
                )
                .write_volatile(original_slot_frame);
                slot_record
                    .state_word()
                    .cast_mut::<u32>()
                    .write_volatile(original_slot_header);
                for (index, word) in original_command.into_iter().enumerate() {
                    (command.word_unchecked(index as u32) as *mut u32).write_volatile(word);
                }
                release_unpublished_probe_context(context);
                return Err(error);
            }
        };
        Ok(PreparedProbePublication {
            context,
            pipe,
            slot,
            slot_record,
            command,
            original_slot_header,
            original_slot_frame,
            original_command,
            checksum,
            start_phy: true,
        })
    }
}

/// Prepare one host-supplied unicast management or data frame for the proven
/// single-context MAC publication path.
///
/// # Safety
/// JOIN must own the selected VIF/channel and no scan probe or host frame may
/// currently own the shared context/pipe backend.
#[cfg(target_arch = "arm")]
unsafe fn prepare_legacy_control_publication(
    request: &crate::wsm::TxRequest<'_>,
    if_id: u8,
) -> Result<PreparedProbePublication, ProbeBuildError> {
    if if_id > 1
        || !crate::vif::is_active(if_id)
        || (!request.is_unicast_management() && !request.is_unicast_eapol())
    {
        return Err(ProbeBuildError::InvalidInterface);
    }
    if request.frame.len() > MAX_TEMPLATE_FRAME_LEN {
        return Err(ProbeBuildError::FrameTooLarge);
    }

    if request.is_unicast_eapol() {
        let retained = unsafe { &mut *LAST_EAPOL_SCRATCH.0.get() };
        retained.bytes[..request.frame.len()].copy_from_slice(request.frame);
        retained.length = request.frame.len();
        retained.rate = request.max_tx_rate;
        *unsafe { &mut *LAST_EAPOL_METADATA.0.get() } = RetainedEapolMetadata {
            queue_id: request.queue_id,
            more: request.more,
            flags: request.flags,
            expire_time: request.expire_time,
            ht_tx_parameters: request.ht_tx_parameters,
        };
    }

    let scratch = unsafe { &mut *PREPARED_PROBE_SCRATCH.0.get() };
    scratch.bytes[..request.frame.len()].copy_from_slice(request.frame);
    scratch.length = request.frame.len();
    scratch.rate = request.max_tx_rate;

    let mut context = unsafe { prepare_probe_context(scratch, if_id) }?;
    let address = ContextAddress::new(context.context);
    unsafe { copy_to_packet_ram(context.header, request.frame) };
    if !unsafe { packet_ram_matches(context.header, request.frame) } {
        unsafe { release_context_address(context.context) };
        return Err(ProbeBuildError::PacketRamMismatch);
    }
    let queue = request.queue_id.min(3);
    unsafe {
        let ac = crate::dtcm::shared_ptr::<u8>(
            crate::dtcm::queue_to_access_category_unchecked(usize::from(queue)),
        )
        .read_volatile();
        write_u8(address.access_category_address(), ac);
        write_u8(address.request_flag_rate_bits_address(), (request.flags & 0x0f) >> 1);
        write_u8(address.retry_policy_address(), (request.flags & 0x7f) >> 4);
        write_u8(address.host_link_address(), 1);
    }
    if request.max_tx_rate < 22 {
        context.rate = request.max_tx_rate;
        unsafe {
            write_u8(address.requested_rate_address(), context.rate);
            write_u8(address.tx_rate_address(), context.rate);
        }
    }
    let mut tx_flags = 0x0080_1000_u32;
    if request.frame.get(4).is_some_and(|octet| octet & 1 != 0) {
        tx_flags |= 0x300;
    }
    if request.ht_tx_parameters & 3 == 1 {
        tx_flags |= 8;
    }
    if request.flags & 1 != 0 {
        tx_flags |= 0x0001_0000;
    }
    tx_flags |= (request.ht_tx_parameters >> 11) & 0xe0;
    unsafe {
        write_u32(address.control_bits_address(), tx_flags);
        write_u32(address.expiry_time_address(), request.expire_time);
    }
    if let Err(error) = unsafe { prepare_single_frame_pas_timing(&mut context) } {
        unsafe { release_context_address(context.context) };
        return Err(error);
    }
    unsafe { prepare_context_publication(context) }
}

#[cfg(target_arch = "arm")]
unsafe fn prepare_host_management_publication(
    request: &crate::wsm::TxRequest<'_>,
    if_id: u8,
) -> Result<PreparedProbePublication, ProbeBuildError> {
    let force_ordinary_replay = unsafe { *REPLAY_DATA_PATH_GUARD.0.get() };
    if !force_ordinary_replay && (request.is_unicast_management() || request.is_unicast_eapol()) {
        return unsafe { prepare_legacy_control_publication(request, if_id) };
    }
    if if_id > 1 || !crate::vif::is_active(if_id) {
        return Err(ProbeBuildError::InvalidInterface);
    }
    if !request.is_unicast_management() && !request.is_unicast_data() {
        return Err(ProbeBuildError::WrongTemplateType);
    }
    if request.frame.len() > MAX_TEMPLATE_FRAME_LEN {
        return Err(ProbeBuildError::FrameTooLarge);
    }

    let scratch = unsafe { &mut *PREPARED_PROBE_SCRATCH.0.get() };
    scratch.bytes[..request.frame.len()].copy_from_slice(request.frame);
    scratch.length = request.frame.len();
    scratch.rate = request.max_tx_rate;
    let legacy_eapol = request.is_unicast_eapol() && !force_ordinary_replay;
    if !legacy_eapol {
        crate::crypto::encrypt_tx_frame(&mut scratch.bytes[..scratch.length], if_id)
            .map_err(|_| ProbeBuildError::CryptoFailure)?;
    }
    let host_queue = request.queue_id & 3;

    let mut context = unsafe { prepare_probe_context(scratch, if_id) }?;
    let address = ContextAddress::new(context.context);
    // Probe templates deliberately patch address fields at +0x0a/+0x0c/+0x0e.
    // Host WSM frames already contain their final DA/SA/BSSID and must be
    // restored byte-for-byte after reusing the probe context initializer.
    let prepared_frame = &scratch.bytes[..scratch.length];
    unsafe { copy_to_packet_ram(context.header, prepared_frame) };
    if !unsafe { packet_ram_matches(context.header, prepared_frame) } {
        unsafe { release_context_address(context.context) };
        return Err(ProbeBuildError::PacketRamMismatch);
    }
    let frame_control = u16::from_le_bytes([prepared_frame[0], prepared_frame[1]]);
    let header = classify_dot11_header(frame_control, legacy_eapol);
    let header_length = header.length;
    let qos_data = header.qos_data;
    let queue = host_queue;
    unsafe {
        if !legacy_eapol {
            write_u32(address.header_length_address(), header_length);
            write_u32(
                address.payload_length_address(),
                (scratch.length as u32).saturating_sub(header_length),
            );
            // RustCrypto has already filled the host-reserved CCMP header and
            // MIC space. Mark ordinary data as having no pending hardware
            // crypto work. The validated EAPOL compatibility path preserves
            // the original class-6 context status unchanged.
            if frame_control & 0x400c == 0x4008 {
                write_u16(address.terminal_status_address(), 0x0010);
            }
        }
        // `ctx+0x60` is the vendor AC selected through the four-entry WSM
        // queue map, not the raw WSM queue ID. The adjacent bytes retain the
        // PTA priority and retry-policy selector packed in WSM TX flags.
        let ac = crate::dtcm::shared_ptr::<u8>(
            crate::dtcm::queue_to_access_category_unchecked(usize::from(queue)),
        )
        .read_volatile();
        write_u8(address.access_category_address(), ac);
        write_u8(
            address.request_flag_rate_bits_address(),
            (request.flags & 0x0f) >> 1,
        );
        write_u8(address.retry_policy_address(), (request.flags & 0x7f) >> 4);
        // This direct cooperative publisher still uses an internal class-6
        // context rather than the vendor WSM class-0 pool. Link slot 1 is the
        // validated internal slot for that temporary path.
        write_u8(address.host_link_address(), 1);
    }
    if request.max_tx_rate < 22 {
        context.rate = request.max_tx_rate;
        unsafe {
            write_u8(address.requested_rate_address(), context.rate);
            write_u8(address.tx_rate_address(), context.rate);
        }
    }
    // `tx_lmac_req_submit` seeds bit 23, then `tx_classify_hdr_len` adds the
    // direct-frame marker and multicast/no-ACK classification.
    let mut tx_flags = 0x0080_1000_u32;
    if qos_data {
        tx_flags |= 0x0040_0000;
    }
    if request.frame.get(4).is_some_and(|octet| octet & 1 != 0) {
        tx_flags |= 0x300;
    }
    if request.ht_tx_parameters & 3 == 1 {
        tx_flags |= 8;
    }
    if request.flags & 1 != 0 {
        tx_flags |= 0x0001_0000;
    }
    tx_flags |= (request.ht_tx_parameters >> 11) & 0xe0;
    unsafe {
        write_u32(address.control_bits_address(), tx_flags);
        write_u32(address.expiry_time_address(), request.expire_time);
    }
    // Recompute the complete PAS timing image after replacing the probe
    // template's rate, flags, and header with the host-supplied frame.
    if let Err(error) = unsafe { prepare_single_frame_pas_timing(&mut context) } {
        unsafe { release_context_address(context.context) };
        return Err(error);
    }
    let mut publication = unsafe { prepare_context_publication(context) }?;
    Ok(publication)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProbeExperimentReport {
    Idle,
    Published {
        opportunity: crate::scan::ProbeOpportunity,
        publication: PublishedProbePublication,
    },
    Servicing,
    Completed {
        opportunity: crate::scan::ProbeOpportunity,
        context: ContextAddress,
        status: u16,
    },
    Failed {
        opportunity: crate::scan::ProbeOpportunity,
        error: ProbeBuildError,
    },
}

#[cfg(target_arch = "arm")]
struct ProbeExperimentRuntime {
    backend: SingleProbeMacBackend,
    published: Option<PublishedProbePublication>,
    opportunity: Option<crate::scan::ProbeOpportunity>,
    host_published: Option<PublishedProbePublication>,
    host_packet_id: u32,
    host_rate: u8,
    completed_count: u32,
    diagnostic: u16,
}

#[cfg(target_arch = "arm")]
impl ProbeExperimentRuntime {
    const fn new() -> Self {
        Self {
            backend: SingleProbeMacBackend::new(2),
            published: None,
            opportunity: None,
            host_published: None,
            host_packet_id: 0,
            host_rate: 0,
            completed_count: 0,
            diagnostic: 0,
        }
    }
}

#[cfg(target_arch = "arm")]
struct SharedProbeExperiment(UnsafeCell<ProbeExperimentRuntime>);

#[cfg(target_arch = "arm")]
unsafe impl Sync for SharedProbeExperiment {}

#[cfg(target_arch = "arm")]
static PROBE_EXPERIMENT: SharedProbeExperiment =
    SharedProbeExperiment(UnsafeCell::new(ProbeExperimentRuntime::new()));

/// Guarded scan-owned probe experiment. Publications are serialized through
/// complete hardware return before another scan opportunity can publish.
///
/// # Safety
/// The caller must own probe preparation, event FIFO and completion servicing.
#[cfg(target_arch = "arm")]
pub unsafe fn service_guarded_probe_experiment(
    _events: &mut MacEventQueue,
    template: Option<&[u8]>,
    opportunity: Option<crate::scan::ProbeOpportunity>,
    ssid: &[u8],
    max_events: u32,
) -> ProbeExperimentReport {
    let runtime = unsafe { &mut *PROBE_EXPERIMENT.0.get() };
    if runtime.host_published.is_some() {
        return ProbeExperimentReport::Servicing;
    }
    if let Some(published) = runtime.published {
        let report =
            unsafe { service_single_probe_runtime_inactive(&mut runtime.backend, max_events) };
        let Some(completion) = report.completion else {
            runtime.diagnostic = 0x2000;
            return ProbeExperimentReport::Servicing;
        };
        let context = ContextAddress::new(completion.context);
        let status = completion.status;
        if context != published.context {
            terminal_probe_backend_fault(published.pipe);
        }
        let Some(opportunity) = runtime.opportunity.take() else {
            terminal_probe_backend_fault(published.pipe);
        };
        runtime.published = None;
        runtime.completed_count = runtime.completed_count.saturating_add(1);

        runtime.diagnostic =
            0x3000 | ((runtime.completed_count.min(0x0f) as u16) << 8) | (status & 0x00ff);
        return ProbeExperimentReport::Completed {
            opportunity,
            context,
            status,
        };
    }

    let Some(opportunity) = opportunity else {
        return ProbeExperimentReport::Idle;
    };

    let prepared = match unsafe {
        prepare_probe_publication(template, ssid, opportunity.channel, opportunity.if_id)
    } {
        Ok(prepared) => prepared,
        Err(error) => {
            runtime.diagnostic = 0xf000 | error as u16;
            return ProbeExperimentReport::Failed { opportunity, error };
        }
    };
    let published = match unsafe { prepared.publish(&mut runtime.backend) } {
        Ok(published) => published,
        Err(error) => {
            runtime.diagnostic = 0xf100 | error as u16;
            return ProbeExperimentReport::Failed { opportunity, error };
        }
    };
    runtime.published = Some(published);
    runtime.opportunity = Some(opportunity);
    runtime.diagnostic = 0x1000 | u16::from(published.pipe) | (u16::from(published.slot) << 4);
    ProbeExperimentReport::Published {
        opportunity,
        publication: published,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostManagementTxReport {
    Idle,
    Published {
        packet_id: u32,
        bisect_stage: u8,
    },
    Servicing,
    Completed {
        packet_id: u32,
        status: u32,
        tx_rate: u8,
        ack_failures: u8,
    },
    Failed {
        packet_id: u32,
        error: ProbeBuildError,
    },
}

/// Exact `wsm_status_from_internal` switch table at vendor `0x0000aff8`.
pub const fn wsm_status_from_internal(status: u16) -> u32 {
    match status {
        0 => 0,
        1 => 1,
        2 => 2,
        8 => 13,
        10 => 7,
        11 => 6,
        12 => 8,
        16 => 9,
        17 => 10,
        19 => 5,
        21 => 4,
        22 => 11,
        23 => 14,
        24 => 15,
        _ => 1,
    }
}

/// Publish or service one host-supplied authentication/association frame.
/// The shared backend admits exactly one hardware-owned context at a time.
///
/// # Safety
/// The caller must exclusively service the MAC event FIFO and completion ring.
#[cfg(target_arch = "arm")]
pub unsafe fn host_management_runtime_active() -> bool {
    let runtime = unsafe { &*PROBE_EXPERIMENT.0.get() };
    runtime.host_published.is_some() || runtime.published.is_some()
}

#[cfg(target_arch = "arm")]
pub unsafe fn service_host_management_tx(
    _events: &mut MacEventQueue,
    request: Option<(&crate::wsm::TxRequest<'_>, u8)>,
    max_events: u32,
) -> HostManagementTxReport {
    let runtime = unsafe { &mut *PROBE_EXPERIMENT.0.get() };
    if let Some(published) = runtime.host_published {
        if let Some((request, _)) = request {
            return HostManagementTxReport::Failed {
                packet_id: request.packet_id,
                error: ProbeBuildError::PipeSlotBusy,
            };
        }
        let report =
            unsafe { service_single_probe_runtime_inactive(&mut runtime.backend, max_events) };
        let Some(completion) = report.completion else {
            return HostManagementTxReport::Servicing;
        };
        let context = ContextAddress::new(completion.context);
        let completion_status = completion.status;
        if context != published.context {
            terminal_probe_backend_fault(published.pipe);
        }
        runtime.host_published = None;
        // Class-6 callback return raises scheduler bit 21 for its owning task.
        // Host-management TX has no separate vendor task, so consume it only
        // after complete context/ring return before admitting RESET or scan.
        let _ = unsafe { claim_scheduler_mask_atomic(1 << 21) };
        let status = wsm_status_from_internal(completion_status);
        return HostManagementTxReport::Completed {
            packet_id: runtime.host_packet_id,
            status,
            tx_rate: runtime.host_rate,
            ack_failures: completion.ack_failures,
        };
    }

    let Some((request, if_id)) = request else {
        if runtime.published.is_some() {
            return HostManagementTxReport::Servicing;
        }
        // Vendor source 0x16 services `mac_irq_handler()` continuously from
        // FIQ, including between host transmissions. Cooperatively drain the
        // FIFO and completion task here as well, otherwise beacon/radio/pipe
        // events remain stale until the next class-6 publication.
        let report =
            unsafe { service_single_probe_runtime_inactive(&mut runtime.backend, max_events) };
        if let Some(completion) = report.completion {
            unsafe {
                trace_tx_value(0x28, 0x4944_0000 | u32::from(completion.status));
                trace_tx_value(0x2c, completion.context);
            }
        }
        return HostManagementTxReport::Idle;
    };
    if runtime.published.is_some() {
        return HostManagementTxReport::Failed {
            packet_id: request.packet_id,
            error: ProbeBuildError::PipeSlotBusy,
        };
    }
    select_publication_bisect(request.frame);
    if publication_bisect_reached(1) {
        return HostManagementTxReport::Published {
            packet_id: request.packet_id,
            bisect_stage: 1,
        };
    }
    let prepared = match unsafe { prepare_host_management_publication(request, if_id) } {
        Ok(prepared) => prepared,
        Err(error) => {
            return HostManagementTxReport::Failed {
                packet_id: request.packet_id,
                error,
            };
        }
    };
    if publication_bisect_reached(2) {
        return HostManagementTxReport::Published {
            packet_id: request.packet_id,
            bisect_stage: 2,
        };
    }
    let published = match unsafe { prepared.publish(&mut runtime.backend) } {
        Ok(published) => published,
        Err(error) => {
            return HostManagementTxReport::Failed {
                packet_id: request.packet_id,
                error,
            };
        }
    };
    if published.bisect_stage == 0 {
        runtime.host_published = Some(published);
        runtime.host_packet_id = request.packet_id;
        runtime.host_rate = request.max_tx_rate;
    }
    HostManagementTxReport::Published {
        packet_id: request.packet_id,
        bisect_stage: published.bisect_stage,
    }
}

#[cfg(target_arch = "arm")]
pub fn probe_runtime_quiescent() -> bool {
    let runtime = unsafe { &*PROBE_EXPERIMENT.0.get() };
    runtime.published.is_none()
        && runtime.host_published.is_none()
        && unsafe { active_internal_contexts() } == 0
        && unsafe { active_pas_contexts() } == 0
        && unsafe {
            let (consumer, producer) = COMPLETION_RING.cursors();
            consumer == producer
        }
        && unsafe { read_u8(crate::dtcm::LOW_MAC_ACTIVE_TX_COUNT.get()) } == 0
}

/// Exact `pac_phy_stop_op`: enter command 7, arm its vendor timer, then cancel
/// that timer immediately.
///
/// # Safety
/// PHY command and scheduler-timer state must be exclusively owned.
#[cfg(target_arch = "arm")]
pub unsafe fn stop_phy_operation_7() {
    unsafe {
        write_u8(crate::dtcm::MAC_PHY_OPERATION_COMMAND.get(), 7);
        write_u8(crate::dtcm::mac_phy_dispatch_command_byte_unchecked(1).get(), 0);
        write_u8(crate::dtcm::MAC_PHY_OPERATION_OUTPUT.get(), 1);
        let (timer, duration) = phy_operation_7_timer();
        write_u32(crate::dtcm::MAC_PHY_OPERATION_TIMEOUT.get(), duration);
        write_u32(
            crate::dtcm::MAC_PHY_OPERATION_STATE.get(),
            u32::from(read_u8(crate::dtcm::MAC_PHY_OPERATION_OUTPUT.get())),
        );
        // `pac_phy_stop_op` cancels this timer immediately. The detached
        // low-MAC does not install the vendor callback/list ownership for this
        // object, so execute the identical stable end state without briefly
        // publishing an unreachable timer callback.
        let _ = cancel_scheduler_timer(timer);
    }
}

#[cfg(target_arch = "arm")]
pub unsafe fn fatal_scan_stop_timeout() -> ! {
    terminal_probe_backend_fault(0)
}

#[cfg(target_arch = "arm")]
pub unsafe fn set_scheduler_bits(mask: u32) {
    unsafe { raise_scheduler_bits(mask) };
}

#[cfg(target_arch = "arm")]
pub unsafe fn clear_scheduler_bits(mask: u32) {
    let _ = unsafe { claim_scheduler_mask_atomic(mask) };
}

#[cfg(target_arch = "arm")]
pub unsafe fn disable_irq_fiq_save() -> u32 {
    unsafe { mask_irq_fiq_terminal() }
}

#[cfg(target_arch = "arm")]
pub unsafe fn restore_irq_fiq_saved(previous: u32) {
    unsafe { restore_irq_fiq(previous) };
}

#[cfg(not(target_arch = "arm"))]
pub unsafe fn disable_irq_fiq_save() -> u32 {
    0
}

#[cfg(not(target_arch = "arm"))]
pub unsafe fn restore_irq_fiq_saved(_previous: u32) {}

#[cfg(target_arch = "arm")]
pub fn probe_experiment_diagnostic_word() -> u16 {
    unsafe { (*PROBE_EXPERIMENT.0.get()).diagnostic }
}

#[cfg(target_arch = "arm")]
pub fn probe_experiment_diagnostic_value() -> u32 {
    unsafe {
        let runtime = &*PROBE_EXPERIMENT.0.get();
        u32::from(runtime.diagnostic) | (runtime.completed_count << 16)
    }
}

#[cfg(not(target_arch = "arm"))]
pub const fn probe_experiment_diagnostic_word() -> u16 {
    0
}

#[cfg(not(target_arch = "arm"))]
pub const fn probe_experiment_diagnostic_value() -> u32 {
    0
}

/// Exercises complete detached preparation and cancellation without publishing.
///
/// # Safety
/// The internal context pool and retained template storage must be exclusively
/// owned by the firmware runtime.
pub unsafe fn validate_probe_preparation(
    template: Option<&[u8]>,
    ssid: &[u8],
    channel: u8,
    if_id: u8,
) -> Result<u32, ProbeBuildError> {
    let publication = unsafe { prepare_probe_publication(template, ssid, channel, if_id) }?;
    let checksum = publication.checksum();
    unsafe {
        (&raw mut *PROBE_CHECKSUM.0.get()).write_volatile(checksum);
        publication.cancel()?;
    }
    Ok(checksum)
}

/// Initializes the three-entry internal management TX pool from vendor
/// `tx_ctx_pool_init` (`0x12574`). This does not publish anything to hardware.
///
/// # Safety
/// DTCM and packet RAM must be mapped and no internal TX context may be owned.
pub unsafe fn initialize_internal_pool() {
    unsafe {
        let boundary = packet_ram::internal_tx_buffers_end() as u32;
        (crate::dtcm::INTERNAL_BUFFER_END_PRIMARY.get() as *mut u32).write_volatile(boundary);
        (crate::dtcm::INTERNAL_BUFFER_END_MIRROR.get() as *mut u32).write_volatile(boundary);

        let mut previous = 0_u32;
        for index in 0..TX_CONTEXT_COUNT {
            let context = internal_context_address(index);
            let buffer = packet_ram::internal_tx_buffer(index);
            crate::dtcm::shared_ptr::<u32>(context.borrowed_frame_address())
                .write_volatile((buffer + 0x40) as u32);
            crate::dtcm::shared_ptr::<u32>(context.cipher_buffer())
                .write_volatile((buffer + 0x20) as u32);
            crate::dtcm::shared_ptr::<u16>(context.terminal_status()).write_volatile(0x00ff);
            crate::dtcm::shared_ptr::<u32>(context.intrusive_next()).write_volatile(previous);
            previous = context.raw();
        }
        internal_context_free_head().write_volatile(previous);
    }
}

/// Builds the 802.11 probe request portion of vendor
/// `syn_scan_build_probe_req` (`0x141b0`). The four-byte WSM template header is
/// consumed, the requested SSID is substituted, and every DS Parameter Set IE
/// is rewritten for the active channel.
fn prepare_probe_into<'a>(
    prepared: &'a mut PreparedProbe,
    template: Option<&[u8]>,
    ssid: &[u8],
    channel: u8,
) -> Result<&'a PreparedProbe, ProbeBuildError> {
    if ssid.len() > 32 {
        return Err(ProbeBuildError::SsidTooLong);
    }
    let template = template.ok_or(ProbeBuildError::MissingTemplate)?;
    if template.len() < 4 {
        return Err(ProbeBuildError::MalformedTemplate);
    }
    if template[0] != 0 {
        return Err(ProbeBuildError::WrongTemplateType);
    }
    let rate = template[1];
    let declared = usize::from(u16::from_le_bytes([template[2], template[3]]));
    if declared < 24 || declared != template.len() - 4 {
        return Err(ProbeBuildError::MalformedTemplate);
    }
    let frame = &template[4..];
    let bytes = &mut prepared.bytes;
    bytes[..24].copy_from_slice(&frame[..24]);
    let mut output = 24;
    let mut input = 24;
    let mut wrote_ssid = false;
    while input < frame.len() {
        if input + 2 > frame.len() {
            return Err(ProbeBuildError::MalformedTemplate);
        }
        let id = frame[input];
        let length = usize::from(frame[input + 1]);
        let next = input
            .checked_add(2 + length)
            .ok_or(ProbeBuildError::MalformedTemplate)?;
        if next > frame.len() {
            return Err(ProbeBuildError::MalformedTemplate);
        }
        if id == 0 {
            if !wrote_ssid {
                let end = output + 2 + ssid.len();
                if end > bytes.len() {
                    return Err(ProbeBuildError::FrameTooLarge);
                }
                bytes[output] = 0;
                bytes[output + 1] = ssid.len() as u8;
                bytes[output + 2..end].copy_from_slice(ssid);
                output = end;
                wrote_ssid = true;
            }
        } else if id == 3 {
            if output + 3 > bytes.len() {
                return Err(ProbeBuildError::FrameTooLarge);
            }
            bytes[output..output + 3].copy_from_slice(&[3, 1, channel]);
            output += 3;
        } else {
            let encoded = &frame[input..next];
            let end = output + encoded.len();
            if end > bytes.len() {
                return Err(ProbeBuildError::FrameTooLarge);
            }
            bytes[output..end].copy_from_slice(encoded);
            output = end;
        }
        input = next;
    }
    if !wrote_ssid {
        let end = output + 2 + ssid.len();
        if end > bytes.len() {
            return Err(ProbeBuildError::FrameTooLarge);
        }
        bytes[output] = 0;
        bytes[output + 1] = ssid.len() as u8;
        bytes[output + 2..end].copy_from_slice(ssid);
        output = end;
    }
    prepared.length = output;
    prepared.rate = rate;
    Ok(prepared)
}

pub fn prepare_probe(
    template: Option<&[u8]>,
    ssid: &[u8],
    channel: u8,
) -> Result<PreparedProbe, ProbeBuildError> {
    let mut prepared = PreparedProbe {
        bytes: [0; MAX_TEMPLATE_FRAME_LEN],
        length: 0,
        rate: 0,
    };
    prepare_probe_into(&mut prepared, template, ssid, channel)?;
    Ok(prepared)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlannedAmpduMember {
    pub frame_state: u32,
    pub frame_length: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlannedAmpduInput {
    pub members: [Option<PlannedAmpduMember>;
        crate::host_tx_policy::MAX_EXPERIMENTAL_AMPDU_DEPTH],
    pub phy_rate_word: u32,
    pub phy_control_word: u32,
    pub hardware_rate: u8,
    pub spacing_selector: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlannedAmpduDescriptor {
    pub words: [u32; 11],
    pub length: u8,
    pub phy_words: [u32; 3],
    pub aggregate_length: u16,
    pub member_count: u8,
}

/// Build a bounded vendor opcode stream without publishing it to hardware.
///
/// Every non-final member contributes its padded delimiter length and optional
/// spacing transfer. Members must form one contiguous prefix with depth 2..=4.
pub(crate) fn build_planned_ampdu_descriptor(
    input: PlannedAmpduInput,
) -> Option<PlannedAmpduDescriptor> {
    let member_count = input.members.iter().take_while(|member| member.is_some()).count();
    if !(2..=crate::host_tx_policy::MAX_EXPERIMENTAL_AMPDU_DEPTH).contains(&member_count)
        || input.members[member_count..].iter().any(Option::is_some)
    {
        return None;
    }

    let mut words = [0_u32; 11];
    let mut word_count = 0_usize;
    let mut aggregate_length = 0_u32;
    for index in 0..member_count {
        let member = input.members.get(index).copied().flatten()?;
        *words.get_mut(word_count)? =
            ampdu_transfer_word(member.frame_state.wrapping_add(8) as usize);
        word_count += 1;
        if index + 1 == member_count {
            aggregate_length = aggregate_length.checked_add(u32::from(member.frame_length) + 8)?;
        } else {
            *words.get_mut(word_count)? = 0x6600_0000;
            word_count += 1;
            if let Some(spacing) = ampdu_spacing_word(input.spacing_selector) {
                *words.get_mut(word_count)? = spacing;
                word_count += 1;
            }
            let padded = u32::from(member.frame_length).checked_add(0x0b)? & !3;
            aggregate_length = aggregate_length
                .checked_add(padded)?
                .checked_add(u32::from(input.spacing_selector) * 4)?;
        }
    }
    *words.get_mut(word_count)? = 0xe400_0000;
    word_count += 1;
    let aggregate_length = u16::try_from(aggregate_length).ok()?;
    let phy_words = [
        0x5100_0000 | (input.phy_rate_word & 0x00ff_ffff),
        0x5000_0000 | (input.phy_control_word & 0x00ff_ffff),
        0x5200_0000
            | (u32::from(input.hardware_rate) << 16)
            | u32::from(aggregate_length),
    ];
    Some(PlannedAmpduDescriptor {
        words,
        length: word_count as u8,
        phy_words,
        aggregate_length,
        member_count: member_count as u8,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlannedBlockAck {
    pub states: [BlockAckMemberState;
        crate::host_tx_policy::MAX_EXPERIMENTAL_AMPDU_DEPTH],
    pub member_count: u8,
}

fn planned_member_count<T>(members: &[Option<T>]) -> Option<usize> {
    let count = members.iter().take_while(|member| member.is_some()).count();
    if !(2..=crate::host_tx_policy::MAX_EXPERIMENTAL_AMPDU_DEPTH).contains(&count)
        || members[count..].iter().any(Option::is_some)
    {
        None
    } else {
        Some(count)
    }
}

pub(crate) fn classify_planned_block_ack(
    start_sequence: u16,
    bitmap: u64,
    sequences: [Option<u16>; crate::host_tx_policy::MAX_EXPERIMENTAL_AMPDU_DEPTH],
) -> Option<PlannedBlockAck> {
    let member_count = planned_member_count(&sequences)?;
    let mut states = [BlockAckMemberState::OutsideWindow;
        crate::host_tx_policy::MAX_EXPERIMENTAL_AMPDU_DEPTH];
    for index in 0..member_count {
        let sequence = sequences[index]?;
        let delta = (sequence & 0x0fff).wrapping_sub(start_sequence & 0x0fff) & 0x0fff;
        states[index] = if delta >= 64 {
            BlockAckMemberState::OutsideWindow
        } else if bitmap & (1_u64 << delta) != 0 {
            BlockAckMemberState::Acknowledged
        } else {
            BlockAckMemberState::Missing
        };
    }
    Some(PlannedBlockAck {
        states,
        member_count: member_count as u8,
    })
}

pub(crate) fn merge_planned_block_ack(
    previous: Option<PlannedBlockAck>,
    current: PlannedBlockAck,
) -> Option<PlannedBlockAck> {
    if previous.is_some_and(|previous| previous.member_count != current.member_count) {
        return None;
    }
    let mut merged = current;
    for index in 0..usize::from(current.member_count) {
        merged.states[index] = match (
            previous.map(|previous| previous.states[index]),
            current.states[index],
        ) {
            (Some(BlockAckMemberState::Acknowledged), _)
            | (_, BlockAckMemberState::Acknowledged) => BlockAckMemberState::Acknowledged,
            (Some(BlockAckMemberState::Missing), _) | (_, BlockAckMemberState::Missing) => {
                BlockAckMemberState::Missing
            }
            _ => BlockAckMemberState::OutsideWindow,
        };
    }
    Some(merged)
}

pub(crate) fn plan_planned_block_ack_actions(
    observation: PlannedBlockAck,
    retry_allowed: [bool; crate::host_tx_policy::MAX_EXPERIMENTAL_AMPDU_DEPTH],
    session_active: bool,
) -> [Option<BlockAckMemberAction>; crate::host_tx_policy::MAX_EXPERIMENTAL_AMPDU_DEPTH] {
    core::array::from_fn(|index| {
        if index >= usize::from(observation.member_count) {
            return None;
        }
        Some(match observation.states[index] {
            BlockAckMemberState::Acknowledged => BlockAckMemberAction::Confirm,
            BlockAckMemberState::Missing if session_active && retry_allowed[index] => {
                BlockAckMemberAction::Retry
            }
            BlockAckMemberState::Missing | BlockAckMemberState::OutsideWindow => {
                BlockAckMemberAction::GiveUp
            }
        })
    })
}

pub(crate) fn planned_whole_retry_allowed(
    observation: PlannedBlockAck,
    retry_allowed: [bool; crate::host_tx_policy::MAX_EXPERIMENTAL_AMPDU_DEPTH],
    session_active: bool,
    next_rates: [Option<u8>; crate::host_tx_policy::MAX_EXPERIMENTAL_AMPDU_DEPTH],
) -> bool {
    let actions = plan_planned_block_ack_actions(observation, retry_allowed, session_active);
    let count = usize::from(observation.member_count);
    if count < 2
        || actions[..count]
            .iter()
            .any(|action| *action != Some(BlockAckMemberAction::Retry))
    {
        return false;
    }
    let Some(first_rate) = next_rates[0] else {
        return false;
    };
    next_rates[..count]
        .iter()
        .all(|rate| *rate == Some(first_rate))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_state_indices_require_an_exact_pipe_slot_identity() {
        assert_eq!(pipe_slot_state_index(2, 3), Some(11));
        assert_eq!(pipe_slot_state_index(4, 0), None);
        assert_eq!(pipe_slot_state_index(0, 4), None);

        let slot = crate::dtcm::mac_pipe_slot_state_word_unchecked(2, 3).get() as u32;
        assert_eq!(retained_slot_state_index(2, slot), Some(11));
        assert_eq!(retained_slot_state_index(1, slot), None);
        assert_eq!(retained_slot_state_index(2, slot + 1), None);
        assert_eq!(retained_slot_state_index(4, slot), None);
    }

    #[test]
    fn ampdu_publication_indices_preserve_head_slot_and_bound_members() {
        assert_eq!(
            ampdu_publication_indices(3, 2),
            Some([3, 0, usize::MAX, usize::MAX]),
        );
        assert_eq!(ampdu_publication_indices(2, 4), Some([2, 0, 1, 3]));
        assert_eq!(ampdu_publication_indices(0, 1), None);
        assert_eq!(ampdu_publication_indices(0, 5), None);
        assert_eq!(ampdu_publication_indices(4, 2), None);
    }

    #[test]
    fn publication_registration_rejects_overlapping_pipe_owners() {
        assert!(publication_registration_allowed(
            [false; 4],
            BatchPosition::Only,
            2,
        ));
        assert!(publication_registration_allowed(
            [false; 4],
            BatchPosition::First,
            3,
        ));
        assert!(!publication_registration_allowed(
            [false; 4],
            BatchPosition::Last,
            1,
        ));
        assert!(publication_registration_allowed(
            [true, false, false, false],
            BatchPosition::Last,
            1,
        ));
        assert!(!publication_registration_allowed(
            [true, false, false, false],
            BatchPosition::Only,
            1,
        ));
        assert!(!publication_registration_allowed(
            [true, false, false, false],
            BatchPosition::Last,
            0,
        ));
        #[cfg(not(feature = "experimental-four-slot-ordinary"))]
        assert!(!publication_registration_allowed(
            [true, false, false, false],
            BatchPosition::Middle,
            1,
        ));
        #[cfg(feature = "experimental-four-slot-ordinary")]
        {
            assert!(publication_registration_allowed(
                [true, false, false, false],
                BatchPosition::Middle,
                1,
            ));
            assert!(publication_registration_allowed(
                [true, true, false, false],
                BatchPosition::Middle,
                2,
            ));
            assert!(publication_registration_allowed(
                [true, true, true, false],
                BatchPosition::Last,
                3,
            ));
            assert!(publication_registration_allowed(
                [true, false, false, true],
                BatchPosition::Middle,
                1,
            ));
        }
        assert!(!publication_registration_allowed(
            [true, false, true, false],
            BatchPosition::Last,
            3,
        ));
        assert!(!publication_registration_allowed(
            [false; 4],
            BatchPosition::Only,
            4,
        ));
    }

    #[test]
    fn class_zero_completion_queue_holds_the_configured_pipe_depth() {
        let mut completed = BoundedCompletionQueue::<u8, HOST_CLASS0_COMPLETION_CAPACITY>::new();
        for value in 0..HOST_CLASS0_COMPLETION_CAPACITY as u8 {
            assert_eq!(completed.push(value), Ok(()));
        }
        assert_eq!(completed.push(0xff), Err(0xff));
        for value in 0..HOST_CLASS0_COMPLETION_CAPACITY as u8 {
            assert_eq!(completed.take(), Some(value));
        }
        assert_eq!(completed.take(), None);
        assert_eq!(completed.push(0xfe), Ok(()));
        assert_eq!(completed.take(), Some(0xfe));
    }

    struct MockMacEventBackend {
        events: [i32; 4],
        event_count: usize,
        event_index: usize,
        readiness: i32,
        readiness_reads: u32,
        effects: [Option<MacEventEffect>; 16],
        effect_count: usize,
        drain_tails: u32,
    }

    impl MockMacEventBackend {
        fn new(events: &[i32], readiness: i32) -> Self {
            let mut stored = [-1; 4];
            stored[..events.len()].copy_from_slice(events);
            Self {
                events: stored,
                event_count: events.len(),
                event_index: 0,
                readiness,
                readiness_reads: 0,
                effects: [None; 16],
                effect_count: 0,
                drain_tails: 0,
            }
        }
    }

    impl MacEventBackend for MockMacEventBackend {
        fn pop_event(&mut self) -> i32 {
            let event = if self.event_index < self.event_count {
                self.events[self.event_index]
            } else {
                -1
            };
            self.event_index += 1;
            event
        }

        fn readiness(&mut self) -> i32 {
            self.readiness_reads += 1;
            self.readiness
        }

        fn apply_effect(&mut self, _event: MacEvent, effect: MacEventEffect) {
            self.effects[self.effect_count] = Some(effect);
            self.effect_count += 1;
        }

        fn drain_tail(&mut self) {
            self.drain_tails += 1;
        }
    }

    struct MockPipeMmio {
        values: [(u32, u32); 64],
        value_count: usize,
        writes: [(u32, u32); 64],
        write_count: usize,
    }

    impl MockPipeMmio {
        fn new() -> Self {
            Self {
                values: [(0, 0); 64],
                value_count: 0,
                writes: [(0, 0); 64],
                write_count: 0,
            }
        }

        fn set(&mut self, address: u32, value: u32) {
            self.values[self.value_count] = (address, value);
            self.value_count += 1;
        }

        fn get(&self, address: u32) -> u32 {
            self.values[..self.value_count]
                .iter()
                .rev()
                .find_map(|(stored, value)| (*stored == address).then_some(*value))
                .unwrap_or(0)
        }
    }

    impl MacPipeMmio for MockPipeMmio {
        fn read_u8(&mut self, address: u32) -> u8 {
            self.get(address) as u8
        }

        fn read_u16(&mut self, address: u32) -> u16 {
            self.get(address) as u16
        }

        fn read_u32(&mut self, address: u32) -> u32 {
            self.get(address)
        }

        fn write_u8(&mut self, address: u32, value: u8) {
            self.set(address, u32::from(value));
        }

        fn write_u16(&mut self, address: u32, value: u16) {
            self.set(address, u32::from(value));
        }

        fn write_u32(&mut self, address: u32, value: u32) {
            self.writes[self.write_count] = (address, value);
            self.write_count += 1;
            self.set(address, value);
        }
    }

    struct MockTxPolicy {
        calls: [(u8, u32, u32); 3],
        call_count: usize,
    }

    impl MockTxPolicy {
        fn new() -> Self {
            Self {
                calls: [(0, 0, 0); 3],
                call_count: 0,
            }
        }
    }

    impl TxPolicy for MockTxPolicy {
        fn program_random_backoff(&mut self, pipe: u8, backoff_word: u32, pas: u32) {
            self.calls[self.call_count] = (pipe, backoff_word, pas);
            self.call_count += 1;
        }
    }

    struct MockRetryBackend {
        decision: SingleTxRetryDecision,
        decided: Option<(u8, u32, FrameNodeAddress)>,
        rearmed: Option<(u8, u32, FrameNodeAddress, u32)>,
        completed: Option<(u8, FrameNodeAddress, u32, u16)>,
    }

    impl MockRetryBackend {
        fn new(decision: SingleTxRetryDecision) -> Self {
            Self {
                decision,
                decided: None,
                rearmed: None,
                completed: None,
            }
        }
    }

    impl SingleTxRetryBackend for MockRetryBackend {
        fn decide_retry(
            &mut self,
            pipe: u8,
            slot: u32,
            frame_node: FrameNodeAddress,
        ) -> SingleTxRetryDecision {
            self.decided = Some((pipe, slot, frame_node));
            self.decision
        }

        fn rearm_and_ack(
            &mut self,
            pipe: u8,
            slot: u32,
            frame_node: FrameNodeAddress,
            pending_mask: u32,
        ) {
            self.rearmed = Some((pipe, slot, frame_node, pending_mask));
        }

        fn complete_success(&mut self, pipe: u8, frame_node: FrameNodeAddress, slot: u32) {
            self.completed = Some((pipe, frame_node, slot, 0));
        }

        fn complete_give_up(
            &mut self,
            pipe: u8,
            frame_node: FrameNodeAddress,
            slot: u32,
            status: u16,
        ) {
            self.completed = Some((pipe, frame_node, slot, status));
        }

        fn fatal_unsupported_multi_slot_retry(
            &mut self,
            _pipe: u8,
            _slot: u32,
            _frame_node: FrameNodeAddress,
        ) -> ! {
            panic!("unexpected multi-slot retry")
        }
    }

    struct MockRearmBackend {
        rebuilt: bool,
    }

    impl MockRearmBackend {
        fn new() -> Self {
            Self { rebuilt: false }
        }
    }

    impl SingleFrameRearmBackend for MockRearmBackend {
        fn rebuild_rate_descriptor(
            &mut self,
            _pipe: u8,
            _slot: u32,
            _frame_node: FrameNodeAddress,
        ) {
            self.rebuilt = true;
        }

        fn fatal_unsupported_rearm_shape(
            &mut self,
            _pipe: u8,
            _slot: u32,
            _frame_node: FrameNodeAddress,
        ) -> ! {
            panic!("unsupported rearm shape")
        }
    }

    struct MockPoppedEventEffects {
        log: [u8; 9],
        len: usize,
        service_scheduler: Option<SchedulerWord>,
        status_scheduler: Option<SchedulerWord>,
    }

    impl MockPoppedEventEffects {
        fn new() -> Self {
            Self {
                log: [0; 9],
                len: 0,
                service_scheduler: None,
                status_scheduler: None,
            }
        }

        fn push(&mut self, value: u8) {
            self.log[self.len] = value;
            self.len += 1;
        }
    }

    impl PoppedMacEventEffects for MockPoppedEventEffects {
        fn trace(&mut self, _event: MacEvent) {
            self.push(1);
        }

        fn fatal(&mut self, _event: MacEvent) {
            self.push(0xfa);
        }

        fn pipe_phase(
            &mut self,
            _event: MacEvent,
            _event_type: u8,
            _phase: u8,
            _latch: Option<u8>,
        ) {
            self.push(2);
        }

        fn capture_scheduler_word(&mut self) -> SchedulerWord {
            self.push(3);
            SchedulerWord::new(0x1234_5678)
        }

        fn pipe_service(&mut self, _event: MacEvent, saved_scheduler_word: SchedulerWord) {
            self.push(4);
            self.service_scheduler = Some(saved_scheduler_word);
        }

        fn tx_status(
            &mut self,
            _event: MacEvent,
            _status: u8,
            _pipe_service_escalation: bool,
            saved_scheduler_word: SchedulerWord,
        ) {
            self.push(5);
            self.status_scheduler = Some(saved_scheduler_word);
        }

        fn beacon(&mut self, _event: MacEvent) {
            self.push(6);
        }

        fn sideband(&mut self, _event: MacEvent) {
            self.push(7);
        }

        fn archive(&mut self, _event: MacEvent) {
            self.push(8);
        }
    }

    #[test]
    fn mac_pipe_service_executor_preserves_pointer_and_low_nibble_order() {
        let mut mmio = MockPipeMmio::new();
        mmio.set(crate::dtcm::MAC_CURRENT_PIPE.get() as u32, 2);
        mmio.set(pipe_state_address(2) + 2, 1);
        let current_slot = current_slot_address(&mut mmio, 2);
        mmio.set(current_slot + 0x0c, 0x1234);
        mmio.set(current_slot + 0x14, 0x5678);
        let mut policy = MockTxPolicy::new();

        execute_mac_pipe_service(&mut mmio, SchedulerWord::new(0x0f), &mut policy);

        assert_eq!(mmio.writes[0], (crate::dtcm::MAC_CURRENT_PIPE_RECORD.get() as u32, pipe_state_address(2)));
        assert_eq!(
            mmio.writes[1],
            (crate::dtcm::MAC_CURRENT_SLOT.get() as u32, pipe_state_address(2) + 0x0c + 0x18)
        );
        assert_eq!(policy.call_count, 3);
        assert_eq!(policy.calls[0].0, 0);
        assert_eq!(policy.calls[1].0, 1);
        assert_eq!(policy.calls[2].0, 2);
        assert_eq!(policy.calls[2], (2, 0x5678, 0x1234));
        assert_eq!(mmio.writes[2], (PIPE_IRQ_TRIGGER, 0x1e00_0000));
        assert_eq!(mmio.writes[3], (PIPE_IRQ_PENDING, !0x0f_u32));
    }

    #[test]
    fn mac_pipe_service_executor_middle_publishes_quantum_before_ack() {
        let mut mmio = MockPipeMmio::new();
        mmio.set(crate::dtcm::MAC_CURRENT_PIPE.get() as u32, 0);
        let selected = 3;
        let selected_state = pipe_state_address(selected);
        mmio.set(selected_state + 2, 2);
        mmio.set(crate::dtcm::duration_quantum_pointer_unchecked(3).get() as u32, 0x1234);
        let mut policy = MockTxPolicy::new();

        execute_mac_pipe_service(&mut mmio, SchedulerWord::new(0x0080), &mut policy);

        assert_eq!(mmio.writes[2], (0x1234, PIPE_QUANTUM));
        assert_eq!(mmio.writes[3], (PIPE_IRQ_TRIGGER, 1 << 28));
        assert_eq!(mmio.writes[4], (crate::dtcm::MAC_CURRENT_PIPE_RECORD.get() as u32, selected_state));
        assert_eq!(
            mmio.writes[5],
            (crate::dtcm::MAC_CURRENT_SLOT.get() as u32, selected_state + 0x0c + 2 * 0x18)
        );
        assert_eq!(mmio.writes[6], (PIPE_IRQ_PENDING, !0x80_u32));
        assert_eq!(policy.call_count, 0);
    }

    fn unmatched_input(current: u8, last: u8) -> OrdinaryTxPipeStatusInput {
        OrdinaryTxPipeStatusInput {
            pipe_active: true,
            expected_status: 0x11,
            slot_kind: 0,
            slot_state: 3,
            global_busy: false,
            pipe_current: current,
            pipe_last: last,
            pipe_status: 5,
        }
    }

    #[test]
    fn matching_status_still_uses_the_vendor_plan() {
        // The recovery path must never shadow a real completion.
        assert!(matches!(
            plan_ordinary_tx_pipe_status(unmatched_input(2, 2), 0x11),
            OrdinaryTxPipeStatusPlan::Complete { .. }
        ));
    }

    #[test]
    fn advance_pipe_slot_retires_armed_pipe_to_last_plus_one() {
        let mut mmio = MockPipeMmio::new();
        let pipe_state = pipe_state_address(2);
        mmio.set(pipe_state, 1); // producer
        mmio.set(pipe_state + 1, 2); // last consumed
        mmio.set(pipe_state + 3, 1); // armed
        mmio.set(pipe_state + 6, 7); // abort counter
        let ring = crate::platform::tx_ring_register(2, 0) as u32;
        mmio.set(pipe_state + 8, ring);
        // Pending-slot mask and hardware-owned high bits must survive.
        mmio.set(ring + 0x20, 0x8000_000f | (1 << 24) | (1 << 27));

        assert_eq!(advance_pipe_slot(&mut mmio, 2), Some(true));

        assert_eq!(mmio.get(pipe_state + 3), 0);
        assert_eq!(mmio.get(pipe_state + 6), 8);
        assert_eq!(mmio.get(ring + 0x18), PIPE_RETRY_INACTIVE_SENTINEL);
        assert_eq!(mmio.get(PIPE_IRQ_PENDING), 0_u32.wrapping_sub(0x4450));
        // cursor = (last + 1) & 3 = 3, written to bits 26:24 and 29:27.
        assert_eq!(mmio.get(ring + 0x20), 0x8000_000f | (3 << 24) | (3 << 27));
    }

    #[test]
    fn watchdog_timer_divider_samples_once_per_sixteen_passes() {
        let mut countdown = 0;
        let mut samples = 0;
        for _ in 0..64 {
            let (next, due) = advance_watchdog_timer_divider(countdown);
            countdown = next;
            samples += usize::from(due);
        }
        assert_eq!(samples, 4);
        assert_eq!(countdown, 0);
    }

    #[test]
    fn pipe_watchdog_ignores_unprogrammed_or_idle_pipes() {
        assert_eq!(plan_pipe_watchdog(false, true, 5), PipeWatchdogAction::Idle);
        assert_eq!(plan_pipe_watchdog(true, false, 5), PipeWatchdogAction::Idle);
    }

    #[test]
    fn empty_retirement_rejects_retry_owned_aggregate_slots() {
        assert!(empty_retirement_allowed(0, 0, 3));
        assert!(empty_retirement_allowed(0, 1, 3));
        assert!(!empty_retirement_allowed(0, 1, 4));
        assert!(!empty_retirement_allowed(0x0400_5ad8, 0, 3));
    }

    #[test]
    fn retry_command_release_requires_complete_live_slot_identity() {
        let mut mmio = MockPipeMmio::new();
        let pipe = 2_u8;
        let current = 1_u8;
        let record = pipe_record_address(pipe);
        let slot = record.slot_unchecked(usize::from(current));
        let frame_node = FrameNodeAddress::from_raw(0x0400_9248).unwrap();
        let ring = crate::platform::tx_ring_register(usize::from(pipe), 0) as u32;
        mmio.set(record.current_slot().get() as u32, u32::from(current));
        mmio.set(record.hardware_ring().get() as u32, ring);
        mmio.set(slot.kind().get() as u32, 1);
        mmio.set(slot.state().get() as u32, 4);
        mmio.set(slot.frame().get() as u32, frame_node.raw());
        mmio.set(
            slot.command().get() as u32,
            packet_ram::tx_command(usize::from(pipe), usize::from(current)) as u32,
        );
        mmio.set(ring + 0x20, 0x8000_000f);

        assert_eq!(
            mmio.get(slot.command().get() as u32),
            packet_ram::tx_command(usize::from(pipe), usize::from(current)) as u32,
        );
        assert_eq!(
            FrameNodeAddress::from_raw(mmio.get(slot.frame().get() as u32)),
            Some(frame_node),
        );
        assert!(LiveTxSlot::from_mmio(
            &mut mmio,
            pipe,
            current,
            slot.raw(),
            Some(frame_node),
        )
        .is_some());
        assert_eq!(
            release_aggregate_retry_command_mask(
                &mut mmio,
                pipe,
                slot.raw(),
                frame_node,
            ),
            Some(true),
        );
        assert_eq!(mmio.get(ring + 0x20), 0x8000_000d);

        mmio.set(ring + 0x20, 0x8000_000f);
        assert_eq!(
            release_aggregate_retry_command_mask(
                &mut mmio,
                pipe,
                slot.raw() + 0x18,
                frame_node,
            ),
            None,
        );
        assert_eq!(mmio.get(ring + 0x20), 0x8000_000f);

        mmio.set(slot.command().get() as u32, packet_ram::tx_command(0, 0) as u32);
        assert_eq!(
            release_aggregate_retry_command_mask(
                &mut mmio,
                pipe,
                slot.raw(),
                frame_node,
            ),
            None,
        );
        assert_eq!(mmio.get(ring + 0x20), 0x8000_000f);
    }

    #[test]
    fn pipe_watchdog_counts_down_then_nudges_then_expires() {
        // Arming reloads the counter to 5, so an armed pipe that never
        // completes walks 5 -> 3 healthy, 2 -> 1 nudging, then expires.
        assert_eq!(plan_pipe_watchdog(true, true, 5), PipeWatchdogAction::Tick);
        assert_eq!(plan_pipe_watchdog(true, true, 3), PipeWatchdogAction::Tick);
        assert_eq!(plan_pipe_watchdog(true, true, 2), PipeWatchdogAction::Nudge);
        assert_eq!(plan_pipe_watchdog(true, true, 1), PipeWatchdogAction::Nudge);
        assert_eq!(
            plan_pipe_watchdog(true, true, 0),
            PipeWatchdogAction::Expired
        );
        // The vendor read is signed, so an overshoot stays expired rather than
        // wrapping back into the healthy range.
        assert_eq!(
            plan_pipe_watchdog(true, true, -1),
            PipeWatchdogAction::Expired
        );
    }

    #[test]
    fn advance_pipe_slot_on_idle_pipe_republishes_producer() {
        let mut mmio = MockPipeMmio::new();
        let pipe_state = pipe_state_address(0);
        mmio.set(pipe_state, 2);
        mmio.set(pipe_state + 1, 3);
        mmio.set(pipe_state + 3, 0);
        mmio.set(pipe_state + 6, 0xff);
        let ring = crate::platform::tx_ring_register(0, 0) as u32;
        mmio.set(pipe_state + 8, ring);

        assert_eq!(advance_pipe_slot(&mut mmio, 0), Some(false));

        // A saturated abort counter is left alone, and the idle branch mirrors
        // the producer rather than `last + 1`.
        assert_eq!(mmio.get(pipe_state + 6), 0xff);
        assert_eq!(mmio.get(ring + 0x20), (2 << 24) | (2 << 27));
    }

    #[test]
    fn hardware_ring_view_matches_cursor_and_sentinel_offsets() {
        let ring = TxHardwareRingAddress::new(0x9000);
        assert_eq!(core::mem::size_of::<TxHardwareRingLayout>(), 0x24);
        assert_eq!(ring.raw(), 0x9000);
        assert_eq!(ring.duration_fifo(), 0x9000);
        assert_eq!(ring.diagnostic_word_0c(), 0x900c);
        assert_eq!(ring.diagnostic_word_10(), 0x9010);
        assert_eq!(ring.go(), 0x9014);
        assert_eq!(ring.inactive_sentinel(), 0x9018);
        assert_eq!(ring.completion_word(), 0x901c);
        assert_eq!(ring.cursor_and_pending_mask(), 0x9020);
        let descriptor = TxDescriptorAddress::new(0xa000);
        assert_eq!(core::mem::size_of::<TxDescriptorLayout>(), 12);
        assert_eq!(descriptor.raw(), 0xa000);
        assert_eq!(descriptor.command(), 0xa000);
        assert_eq!(descriptor.flags(), 0xa004);
        assert_eq!(descriptor.duration(), 0xa008);
    }

    #[test]
    fn hardware_ring_identity_is_exact_for_each_pipe() {
        for pipe in 0..4 {
            let expected = crate::platform::tx_ring_register(usize::from(pipe), 0) as u32;
            assert_eq!(
                TxHardwareRingAddress::for_pipe(pipe, expected),
                Some(TxHardwareRingAddress::new(expected)),
            );
            assert_eq!(TxHardwareRingAddress::for_pipe(pipe, expected + 0x80), None);
        }
        assert_eq!(
            TxHardwareRingAddress::for_pipe(
                4,
                crate::platform::tx_ring_register(0, 0) as u32,
            ),
            None,
        );
    }

    #[test]
    fn retained_hardware_ring_consumers_reject_mismatched_roots_before_mmio() {
        let mut mmio = MockPipeMmio::new();
        let pipe = 2_u8;
        let record = pipe_record_address(pipe);
        let wrong_ring = crate::platform::tx_ring_register(1, 0) as u32;
        mmio.set(record.hardware_ring().get() as u32, wrong_ring);

        assert_eq!(advance_pipe_slot(&mut mmio, pipe), None);
        assert!(!resync_pipe_cursor(&mut mmio, pipe));
        assert_eq!(hardware_pipe_cursor(&mut mmio, pipe), Err(()));
        assert_eq!(mmio.write_count, 0);

        let input = SingleProbePublicationInput {
            pipe,
            slot: 0,
            pipe_state: record.raw(),
            slot_record: record.slot_unchecked(0).raw(),
            command_storage: packet_ram::tx_command(usize::from(pipe), 0) as u32,
            hardware_ring: wrong_ring,
            frame_node: FrameNodeAddress::from_raw(0x0400_9248).unwrap(),
            expects_ack: false,
            batch: BatchPosition::Only,
        };
        assert_eq!(execute_single_probe_publication(&mut mmio, input), u8::MAX);
        assert!(!finalize_staged_pipe(
            &mut mmio,
            pipe,
            record.raw(),
            wrong_ring,
            0,
            0,
        ));
        assert_eq!(mmio.write_count, 0);
    }

    #[test]
    fn resync_pipe_cursor_matches_vendor_invariant_and_keeps_mask() {
        let mut mmio = MockPipeMmio::new();
        let pipe_state = pipe_state_address(1);
        mmio.set(pipe_state, 3);
        let ring = crate::platform::tx_ring_register(1, 0) as u32;
        mmio.set(pipe_state + 8, ring);
        mmio.set(ring + 0x20, 0xc000_00aa | (1 << 24) | (1 << 27));

        assert!(resync_pipe_cursor(&mut mmio, 1));

        let word = mmio.get(ring + 0x20);
        assert_eq!(word & 0x00ff_ffff, 0xaa);
        assert_eq!(word & 0xc000_0000, 0xc000_0000);
        // The invariant asserted by vendor `txp_fn_4425`.
        assert_eq!((word & 0x3fff_ffff) >> 27, u32::from(mmio.get(pipe_state)));
        assert_eq!((word >> 24) & 7, 3);
    }

    #[test]
    fn resync_pipe_cursor_ignores_unprogrammed_ring() {
        let mut mmio = MockPipeMmio::new();
        resync_pipe_cursor(&mut mmio, 3);
        assert_eq!(mmio.write_count, 0);
    }

    #[test]
    fn pipe_cursor_diagnostic_exposes_divergence() {
        let mut mmio = MockPipeMmio::new();
        let pipe_state = pipe_state_address(2);
        mmio.set(pipe_state, 3); // producer
        mmio.set(pipe_state + 1, 2); // last
        mmio.set(pipe_state + 2, 2); // current
        mmio.set(pipe_state + 3, 1); // armed
        let ring = crate::platform::tx_ring_register(2, 0) as u32;
        mmio.set(pipe_state + 8, ring);
        mmio.set(ring + 0x20, (1 << 24) | (1 << 27));

        let (packed, ring_word) = pipe_cursor_diagnostic(&mut mmio, 2);

        assert_eq!(ring_word, (1 << 24) | (1 << 27));
        assert_eq!(packed & 0x0f, 2);
        assert_eq!((packed >> 4) & 0x0f, 3);
        assert_eq!((packed >> 8) & 0x0f, 2);
        assert_eq!((packed >> 12) & 0x0f, 2);
        assert_eq!((packed >> 16) & 0x0f, 1);
        // Producer nibble 3 versus hardware cursor nibble 1: diverged.
        assert_eq!((packed >> 20) & 0x0f, 1);
        assert_eq!((packed >> 24) & 0x0f, 1);
        assert_ne!((packed >> 4) & 0x0f, (packed >> 24) & 0x0f);
    }

    #[test]
    fn single_retry_inactive_pipe_marks_selected_commands_before_ack() {
        let mut mmio = MockPipeMmio::new();
        mmio.set(crate::dtcm::MAC_CURRENT_PIPE.get() as u32, 1);
        mmio.set(pipe_state_address(1) + 2, 0);
        let slot = current_slot_address(&mut mmio, 1);
        mmio.set(slot + 3, 3);
        let ring_1 = crate::platform::tx_ring_register(1, 0) as u32;
        let ring_3 = crate::platform::tx_ring_register(3, 0) as u32;
        mmio.set(pipe_state_address(1) + 8, ring_1);
        mmio.set(pipe_state_address(3) + 8, ring_3);
        let mut backend = MockRetryBackend::new(SingleTxRetryDecision::GiveUp);

        let outcome = execute_single_outstanding_tx_retry(
            &mut mmio,
            SchedulerWord::new(0x0a00),
            &mut backend,
        );

        assert_eq!(outcome, SingleTxRetryOutcome::InactivePipeAcknowledged);
        assert_eq!(mmio.writes[0], (crate::dtcm::MAC_CURRENT_PIPE_RECORD.get() as u32, pipe_state_address(1)));
        assert_eq!(mmio.writes[1], (crate::dtcm::MAC_CURRENT_SLOT.get() as u32, slot));
        assert_eq!(mmio.get(slot + 3), 4);
        assert_eq!(mmio.get(crate::dtcm::LOW_MAC_PIPE_BUSY.get() as u32), 1);
        assert_eq!(mmio.writes[2], (ring_1 + 0x18, PIPE_RETRY_INACTIVE_SENTINEL));
        assert_eq!(mmio.writes[3], (ring_3 + 0x18, PIPE_RETRY_INACTIVE_SENTINEL));
        assert_eq!(mmio.writes[4], (PIPE_IRQ_PENDING, 0xffff_fdff));
        assert_eq!(backend.decided, None);
    }

    #[test]
    fn single_retry_give_up_completes_before_command_and_ack() {
        let mut mmio = MockPipeMmio::new();
        mmio.set(crate::dtcm::MAC_CURRENT_PIPE.get() as u32, 0);
        let pipe_state = pipe_state_address(0);
        mmio.set(pipe_state + 1, 0);
        mmio.set(pipe_state + 2, 0);
        mmio.set(pipe_state + 3, 1);
        mmio.set(pipe_state + 5, 0);
        let ring = crate::platform::tx_ring_register(0, 0) as u32;
        mmio.set(pipe_state + 8, ring);
        let slot = current_slot_address(&mut mmio, 0);
        mmio.set(slot + 1, 4);
        mmio.set(slot + 3, 3);
        mmio.set(slot + 0x0c, 0x0400_90d8);
        let mut backend = MockRetryBackend::new(SingleTxRetryDecision::GiveUp);

        let outcome = execute_single_outstanding_tx_retry(
            &mut mmio,
            SchedulerWord::new(0x0100),
            &mut backend,
        );

        assert_eq!(outcome, SingleTxRetryOutcome::GivenUp);
        assert_eq!(
            backend.completed,
            Some((0, FrameNodeAddress::new(0x0400_90d8), slot, 0x0b))
        );
        assert_eq!(mmio.get(PIPE_IRQ_TRIGGER), 1 << 25);
        assert_eq!(mmio.get(ring + 0x1c), 1);
        assert_eq!(mmio.get(pipe_state), 1);
        assert_eq!(mmio.get(pipe_state + 2), 1);
        assert_eq!(mmio.get(pipe_state + 3), 0);
        assert_eq!(mmio.get(pipe_state + 4), 0);
        assert_eq!(mmio.get(pipe_state + 5), 5);
        assert_eq!(mmio.get(PIPE_IRQ_PENDING), 0xffff_fef0);
    }

    #[test]
    fn block_ack_success_keeps_the_selected_pipe_through_completion() {
        let mut mmio = MockPipeMmio::new();
        mmio.set(crate::dtcm::MAC_CURRENT_PIPE.get() as u32, 2);
        let pipe_state = pipe_state_address(2);
        mmio.set(pipe_state + 1, 0);
        mmio.set(pipe_state + 2, 0);
        mmio.set(pipe_state + 3, 1);
        mmio.set(pipe_state + 5, 0);
        let ring = crate::platform::tx_ring_register(2, 0) as u32;
        mmio.set(pipe_state + 8, ring);
        let slot = current_slot_address(&mut mmio, 2);
        mmio.set(slot + 1, 4);
        mmio.set(slot + 3, 3);
        mmio.set(slot + 0x0c, 0x0400_90d8);
        let mut backend = MockRetryBackend::new(SingleTxRetryDecision::CompleteSuccess);

        let outcome = execute_single_outstanding_tx_retry(
            &mut mmio,
            SchedulerWord::new(0x0400),
            &mut backend,
        );

        assert_eq!(outcome, SingleTxRetryOutcome::Completed);
        assert_eq!(
            backend.completed,
            Some((2, FrameNodeAddress::new(0x0400_90d8), slot, 0))
        );
        assert_eq!(mmio.get(PIPE_IRQ_TRIGGER), 1 << 27);
        assert_eq!(mmio.get(ring + 0x1c), 1);
        assert_eq!(mmio.get(pipe_state + 2), 1);
        assert_eq!(mmio.get(pipe_state + 3), 0);
        assert_eq!(mmio.get(pipe_state + 4), 0);
        assert_eq!(mmio.get(pipe_state + 5), 5);
        assert_eq!(mmio.get(PIPE_IRQ_PENDING), 0xffff_fbf0);
    }

    #[test]
    fn single_retry_rearm_retains_slot_ownership_for_backend() {
        let mut mmio = MockPipeMmio::new();
        mmio.set(crate::dtcm::MAC_CURRENT_PIPE.get() as u32, 2);
        let pipe_state = pipe_state_address(2);
        mmio.set(pipe_state + 2, 1);
        mmio.set(pipe_state + 3, 1);
        mmio.set(pipe_state + 5, 1);
        mmio.set(
            pipe_state + 8,
            crate::platform::tx_ring_register(2, 0) as u32,
        );
        let slot = current_slot_address(&mut mmio, 2);
        mmio.set(slot + 1, 4);
        mmio.set(slot + 3, 3);
        mmio.set(slot + 0x0c, 0x0400_9248);
        let mut backend = MockRetryBackend::new(SingleTxRetryDecision::Rearm);

        let outcome = execute_single_outstanding_tx_retry(
            &mut mmio,
            SchedulerWord::new(0x0400),
            &mut backend,
        );

        assert_eq!(outcome, SingleTxRetryOutcome::Rearmed);
        assert_eq!(mmio.get(slot + 3), 4);
        assert_eq!(mmio.get(crate::dtcm::LOW_MAC_PIPE_BUSY.get() as u32), 1);
        assert_eq!(
            backend.rearmed,
            Some((2, slot, FrameNodeAddress::new(0x0400_9248), 0x400))
        );
        assert_eq!(backend.completed, None);
    }

    #[test]
    fn bounded_single_retry_counts_only_hardware_rearms() {
        let mut retry = BoundedSingleTxRetry::new(2);
        assert_eq!(retry.attempts(), 0);
        assert_eq!(retry.decide(), SingleTxRetryDecision::Rearm);
        assert_eq!(retry.attempts(), 1);
        assert_eq!(retry.decide(), SingleTxRetryDecision::Rearm);
        assert_eq!(retry.attempts(), 2);
        assert_eq!(retry.decide(), SingleTxRetryDecision::GiveUp);
        assert_eq!(retry.attempts(), 2);
        retry.reset();
        assert_eq!(retry.attempts(), 0);

        let mut no_retry = BoundedSingleTxRetry::new(0);
        assert_eq!(no_retry.decide(), SingleTxRetryDecision::GiveUp);
    }

    #[test]
    fn scheduler_timer_list_preserves_deadline_order_and_backlinks() {
        let mut mmio = MockPipeMmio::new();
        let first = 0x1000;
        let second = 0x2000;
        let inserted = 0x1800;
        mmio.set(0x0400_2014, first);
        mmio.set(first, second);
        mmio.set(first + 4, 0x0400_2014);
        mmio.set(first + 8, 100);
        mmio.set(second, 0);
        mmio.set(second + 4, first);
        mmio.set(second + 8, 300);

        assert!(!insert_scheduler_timer_list(&mut mmio, inserted, 200));
        assert_eq!(mmio.get(first), inserted);
        assert_eq!(mmio.get(inserted), second);
        assert_eq!(mmio.get(inserted + 4), first);
        assert_eq!(mmio.get(inserted + 8), 200);
        assert_eq!(mmio.get(second + 4), inserted);

        assert!(unlink_scheduler_timer_list(&mut mmio, inserted));
        assert_eq!(mmio.get(first), second);
        assert_eq!(mmio.get(second + 4), first);
        assert_eq!(mmio.get(inserted + 4), 0);
        assert!(!unlink_scheduler_timer_list(&mut mmio, inserted));
    }

    #[test]
    fn scheduler_claim_clears_only_requested_pending_bits() {
        let mut mmio = MockPipeMmio::new();
        mmio.set(0x0400_1fd4, (1 << 24) | (1 << 20) | (1 << 10));

        assert_eq!(claim_scheduler_mask(&mut mmio, 1 << 20), 1 << 20);
        assert_eq!(mmio.get(0x0400_1fd4), (1 << 24) | (1 << 10));
        assert_eq!(claim_scheduler_mask(&mut mmio, 1 << 20), 0);
        assert_eq!(mmio.get(0x0400_1fd4), (1 << 24) | (1 << 10));
    }

    #[test]
    fn fixed_rate_rearm_rebuilds_before_clearing_command_mask_and_ack() {
        let mut mmio = MockPipeMmio::new();
        let pipe = 0;
        let pipe_state = pipe_state_address(pipe);
        mmio.set(pipe_state, 0);
        mmio.set(pipe_state + 2, 0);
        let ring = crate::platform::tx_ring_register(usize::from(pipe), 0) as u32;
        mmio.set(pipe_state + 8, ring);
        mmio.set(ring + 0x20, 0x0f);
        let slot = pipe_state + 0x0c;
        mmio.set(slot, 0);
        mmio.set(slot + 1, 0xff);
        mmio.set(slot + 0x14, 0xa000);
        mmio.set(0xa004, 0x20);
        let frame = FrameNodeAddress::new(0x0400_90d8);
        mmio.set(frame.raw() + 4, 0x1018);
        mmio.set(frame.raw() + 0x0c, 0);
        mmio.set(frame.raw() + 0x0f, 2);
        mmio.set(frame.raw() + 0x69, 0);
        mmio.set(frame.raw() + 0x56, 0xff);
        mmio.set(crate::dtcm::initialized_random_lfsr().get() as u32, 0x0012_3456);
        mmio.set(
            crate::dtcm::pas_stride_view(0)
                .unwrap()
                .contention_window(0)
                .unwrap()
                .get() as u32,
            0x001f,
        );
        mmio.set(crate::dtcm::mac_retry_rate_unchecked(2).get() as u32, 3);
        mmio.set(crate::dtcm::tx_duration_timing_unchecked(3).get() as u32, 0x20);
        mmio.set(crate::dtcm::LOW_MAC_SLOT_TIME_BASE.get() as u32, 2);
        mmio.set(crate::dtcm::LOW_MAC_SLOT_TIME_INITIAL.get() as u32, 3);
        mmio.set(crate::dtcm::MAC_RETRY_HARDWARE_STATE.get() as u32, 0);
        let mut backend = MockRearmBackend::new();

        let outcome = execute_fixed_rate_single_frame_rearm(
            &mut mmio,
            pipe,
            slot,
            frame,
            0x100,
            &mut backend,
        );

        assert_eq!(outcome, SingleFrameRearmOutcome::CommandMaskAcknowledged);
        let trigger_index = mmio
            .writes
            .iter()
            .position(|write| *write == (PIPE_IRQ_TRIGGER, 1 << 25))
            .unwrap_or_else(|| panic!("missing retry trigger write"));
        let descriptor_index = mmio
            .writes
            .iter()
            .rposition(|(address, _)| *address == 0xa004)
            .unwrap_or_else(|| panic!("missing rebuilt descriptor write"));
        let command_mask_index = mmio
            .writes
            .iter()
            .position(|(address, _)| *address == ring + 0x20)
            .unwrap_or_else(|| panic!("missing retry command-mask write"));
        assert!(descriptor_index < trigger_index);
        assert!(trigger_index < command_mask_index);
        assert_eq!(mmio.get(0xa000), 0);
        assert_eq!(mmio.get(crate::dtcm::initialized_random_lfsr().get() as u32), 0xb013_1713);
        assert_eq!(mmio.get(PIPE_RETRY_RANDOM_STATS), 0x13);
        assert_eq!(mmio.get(PIPE_RETRY_RANDOM_STATS + 12), 1);
        assert_eq!(mmio.get(frame.raw() + 0x5a), 0x13);
        assert_eq!(mmio.get(0xa004), 0x4d7f);
        assert_eq!(mmio.get(0xa008), 0xd800_2138);
        assert_eq!(mmio.get(ring + 0x20), 0x0e);
        assert_eq!(mmio.get(PIPE_IRQ_PENDING), 0xffff_feff);
        assert!(!backend.rebuilt);
    }

    #[test]
    fn ordinary_batch_retry_keeps_producer_and_clears_only_current_mask_bit() {
        let mut mmio = MockPipeMmio::new();
        let pipe = 0;
        let pipe_state = pipe_state_address(pipe);
        mmio.set(pipe_state, 0);
        mmio.set(pipe_state + 2, 1);
        let ring = crate::platform::tx_ring_register(usize::from(pipe), 0) as u32;
        mmio.set(pipe_state + 8, ring);
        mmio.set(ring + 0x20, 0x0f);
        let slot = pipe_state + 0x0c + 0x18;
        mmio.set(slot, 0);
        mmio.set(slot + 1, 0x11);
        mmio.set(slot + 0x14, 0xa000);
        mmio.set(0xa004, 0x20);
        let frame = FrameNodeAddress::new(0x0400_90d8);
        mmio.set(frame.raw() + 4, 0);
        mmio.set(frame.raw() + 0x0c, 0);
        mmio.set(frame.raw() + 0x0f, 2);
        mmio.set(frame.raw() + 0x69, 0);
        mmio.set(frame.raw() + 0x56, 0x11);
        mmio.set(crate::dtcm::initialized_random_lfsr().get() as u32, 0x0012_3456);
        mmio.set(
            crate::dtcm::pas_stride_view(0)
                .unwrap()
                .contention_window(0)
                .unwrap()
                .get() as u32,
            0x001f,
        );
        mmio.set(crate::dtcm::mac_retry_rate_unchecked(2).get() as u32, 3);
        mmio.set(crate::dtcm::tx_duration_timing_unchecked(3).get() as u32, 0x20);
        mmio.set(crate::dtcm::LOW_MAC_SLOT_TIME_BASE.get() as u32, 2);
        mmio.set(crate::dtcm::LOW_MAC_SLOT_TIME_INITIAL.get() as u32, 3);
        mmio.set(crate::dtcm::MAC_RETRY_HARDWARE_STATE.get() as u32, 0);
        let mut backend = MockRearmBackend::new();

        let outcome = execute_fixed_rate_single_frame_rearm(
            &mut mmio,
            pipe,
            slot,
            frame,
            0x100,
            &mut backend,
        );

        assert_eq!(outcome, SingleFrameRearmOutcome::CommandMaskAcknowledged);
        assert_eq!(mmio.get(pipe_state), 0);
        assert_eq!(mmio.get(pipe_state + 2), 1);
        assert_eq!(mmio.get(ring + 0x20), 0x0d);
        assert_eq!(mmio.get(PIPE_IRQ_PENDING), 0xffff_feff);
        assert!(!backend.rebuilt);
    }

    #[test]
    fn rate_change_rebuilds_phy_descriptor_before_retry_duration() {
        let mut mmio = MockPipeMmio::new();
        let pipe = 0;
        let pipe_state = pipe_state_address(pipe);
        mmio.set(pipe_state, 0);
        mmio.set(pipe_state + 2, 0);
        let ring = crate::platform::tx_ring_register(usize::from(pipe), 0) as u32;
        mmio.set(pipe_state + 8, ring);
        mmio.set(ring + 0x20, 1);
        let slot = pipe_state + 0x0c;
        mmio.set(slot, 0);
        mmio.set(slot + 1, 0xff);
        mmio.set(slot + 0x14, 0xa000);
        mmio.set(0xa004, 0);
        let frame = FrameNodeAddress::new(0x0400_90d8);
        mmio.set(frame.raw() + 4, 0x0008_1018);
        mmio.set(frame.raw() + 0x0c, 0);
        mmio.set(frame.raw() + 0x0f, 2);
        mmio.set(frame.raw() + 0x69, 0);
        mmio.set(frame.raw() + 0x56, 0xff);
        mmio.set(crate::dtcm::initialized_random_lfsr().get() as u32, 1);
        mmio.set(
            crate::dtcm::pas_stride_view(0)
                .unwrap()
                .contention_window(0)
                .unwrap()
                .get() as u32,
            0,
        );
        mmio.set(crate::dtcm::mac_retry_rate_unchecked(2).get() as u32, 0);
        mmio.set(crate::dtcm::tx_duration_timing_unchecked(0).get() as u32, 0);
        mmio.set(crate::dtcm::MAC_RETRY_HARDWARE_STATE.get() as u32, 0);
        let mut backend = MockRearmBackend::new();

        let outcome = execute_fixed_rate_single_frame_rearm(
            &mut mmio,
            pipe,
            slot,
            frame,
            0x100,
            &mut backend,
        );

        assert_eq!(outcome, SingleFrameRearmOutcome::CommandMaskAcknowledged);
        assert!(backend.rebuilt);
    }

    #[test]
    fn fixed_rate_rearm_hardware_state_uses_matching_payload_sentinel() {
        let mut mmio = MockPipeMmio::new();
        let pipe = 1;
        let pipe_state = pipe_state_address(pipe);
        mmio.set(pipe_state, 3);
        mmio.set(pipe_state + 2, 3);
        let ring = crate::platform::tx_ring_register(usize::from(pipe), 0) as u32;
        mmio.set(pipe_state + 8, ring);
        let slot = pipe_state + 0x0c + 3 * 0x18;
        mmio.set(slot, 0);
        mmio.set(slot + 1, 4);
        mmio.set(slot + 0x14, 0xa000);
        let frame = FrameNodeAddress::new(0x0400_9248);
        mmio.set(frame.raw() + 4, 0x1018);
        mmio.set(frame.raw() + 0x0c, 0);
        mmio.set(frame.raw() + 0x0f, 1);
        mmio.set(frame.raw() + 0x69, 0);
        mmio.set(frame.raw() + 0x56, 0xff);
        mmio.set(crate::dtcm::initialized_random_lfsr().get() as u32, 1);
        mmio.set(
            crate::dtcm::pas_stride_view(0)
                .unwrap()
                .contention_window(0)
                .unwrap()
                .get() as u32,
            0,
        );
        mmio.set(crate::dtcm::mac_retry_rate_unchecked(1).get() as u32, 0);
        mmio.set(crate::dtcm::tx_duration_timing_unchecked(0).get() as u32, 0);
        mmio.set(crate::dtcm::MAC_RETRY_HARDWARE_STATE.get() as u32, 2);
        let mut backend = MockRearmBackend::new();

        let outcome = execute_fixed_rate_single_frame_rearm(
            &mut mmio,
            pipe,
            slot,
            frame,
            0x200,
            &mut backend,
        );

        assert_eq!(
            outcome,
            SingleFrameRearmOutcome::HardwareSentinelAcknowledged
        );
        assert_eq!(mmio.get(ring + 0x18), 0xff00_ffff);
        assert_eq!(mmio.get(PIPE_IRQ_PENDING), 0xffff_0df0);
    }

    #[test]
    fn single_probe_publication_preserves_vendor_trigger_and_go_order() {
        let mut mmio = MockPipeMmio::new();
        let pipe_state = pipe_state_address(0);
        let slot = pipe_state + 0x0c;
        let hardware_ring = crate::platform::tx_ring_register(0, 0) as u32;
        let command = packet_ram::tx_command(0, 0) as u32;
        let frame = FrameNodeAddress::new(0x0400_90d8);
        mmio.set(frame.raw() + 0x2c, 3);
        mmio.set(frame.raw() + 0x3a, 0x20);
        mmio.set(frame.raw() + 0x56, 0xff);
        mmio.set(frame.raw() + 0x69, 0);
        mmio.set(frame.raw() + 0x0c, 0);
        mmio.set(0x0ac0_0004, 0x1234);
        mmio.set(crate::dtcm::initialized_random_lfsr().get() as u32, 1);
        mmio.set(
            crate::dtcm::pas_stride_view(0)
                .unwrap()
                .contention_window(0)
                .unwrap()
                .get() as u32,
            0,
        );
        mmio.set(
            crate::dtcm::pas_stride_view(0)
                .unwrap()
                .packed_aifs()
                .get() as u32,
            0x55,
        );
        mmio.set(crate::dtcm::MAC_EDCA_SLOT_TIMING.get() as u32, 0x44);
        mmio.set(crate::dtcm::queue_to_access_category_unchecked(0).get() as u32, 1);
        mmio.set(
            crate::dtcm::pas_stride_view(0)
                .unwrap()
                .txop_limit(1)
                .unwrap()
                .get() as u32,
            64,
        );
        mmio.set(
            crate::dtcm::duration_quantum_pointer_unchecked(0).get() as u32,
            crate::platform::mac_register(0x0e70) as u32,
        );
        mmio.set(pipe_state + 4, 8);
        mmio.set(crate::dtcm::LOW_MAC_ACTIVE_TX_COUNT.get() as u32, 2);

        let bisect_stage = execute_single_probe_publication(
            &mut mmio,
            SingleProbePublicationInput {
                pipe: 0,
                slot: 0,
                pipe_state,
                slot_record: slot,
                command_storage: command,
                hardware_ring,
                frame_node: frame,
                expects_ack: false,
                batch: BatchPosition::Only,
            },
        );

        assert_eq!(bisect_stage, 0);
        assert_eq!(mmio.get(frame.raw() + 0x2c), 0x103);
        assert_eq!(mmio.get(frame.raw() + 0x18), 0x1234);
        assert_eq!(mmio.get(frame.raw() + 0x3c), 0);
        assert_eq!(mmio.get(command + 8), 0xdc00_0000);
        assert_eq!(mmio.get(crate::platform::mac_register(0x0e64) as u32), 0x55);
        assert_eq!(mmio.get(crate::platform::mac_register(0x0e70) as u32), 2);
        assert_eq!(mmio.get(PIPE_IRQ_TRIGGER), 1 << 25);
        assert_eq!(
            mmio.get(crate::dtcm::LOW_MAC_ACTIVE_TX_COUNT.get() as u32),
            3
        );
        assert_eq!(mmio.get(slot + 3), 1);
        assert_eq!(mmio.get(slot + 8), 0x0010_0000);
        assert_eq!(mmio.get(hardware_ring), 0x0010_0000);
        assert_eq!(mmio.get(pipe_state + 1), 0);
        assert_eq!(mmio.get(pipe_state + 2), 0);
        assert_eq!(mmio.get(pipe_state + 3), 1);
        assert_eq!(mmio.get(pipe_state + 4), 9);
        assert_eq!(mmio.get(pipe_state + 5), 5);
        assert_eq!(mmio.get(hardware_ring + 0x14), 1);

        let trigger_index = mmio.writes[..mmio.write_count]
            .iter()
            .position(|&(address, _)| address == PIPE_IRQ_TRIGGER)
            .unwrap_or_else(|| panic!("missing trigger write"));
        let go_index = mmio.writes[..mmio.write_count]
            .iter()
            .rposition(|&(address, value)| address == hardware_ring + 0x14 && value == 1)
            .unwrap_or_else(|| panic!("missing GO write"));
        assert!(trigger_index < go_index);
    }

    #[test]
    fn staged_two_slot_batch_triggers_once_and_publishes_both_durations() {
        let mut mmio = MockPipeMmio::new();
        let pipe = 1_u8;
        let pipe_state = pipe_state_address(pipe);
        let hardware_ring = crate::platform::tx_ring_register(usize::from(pipe), 0) as u32;
        let first_slot = crate::dtcm::MacPipeRecordAddress::from_raw_unchecked(pipe_state)
            .slot_unchecked(3);
        let last_slot = crate::dtcm::MacPipeRecordAddress::from_raw_unchecked(pipe_state)
            .slot_unchecked(0);
        mmio.set(first_slot.duration().get() as u32, 0x1111);
        mmio.set(last_slot.duration().get() as u32, 0x2222);
        mmio.set(pipe_state + 4, 8);
        mmio.set(crate::dtcm::LOW_MAC_ACTIVE_TX_COUNT.get() as u32, 4);

        assert!(finalize_staged_pipe(
            &mut mmio,
            pipe,
            pipe_state,
            hardware_ring,
            3,
            0,
        ));

        assert_eq!(mmio.get(PIPE_IRQ_TRIGGER), (1_u32 << pipe) << 25);
        assert_eq!(mmio.get(first_slot.state().get() as u32), 1);
        assert_eq!(mmio.get(last_slot.state().get() as u32), 1);
        assert_eq!(mmio.get(hardware_ring), 0x2222);
        assert_eq!(mmio.get(crate::dtcm::LOW_MAC_ACTIVE_TX_COUNT.get() as u32), 6);
        assert_eq!(mmio.get(pipe_state + 3), 1);
        assert_eq!(mmio.get(pipe_state + 4), 9);
        assert_eq!(mmio.get(pipe_state + 5), 5);
        assert_eq!(mmio.get(hardware_ring + 0x14), 1);
        let duration_writes = mmio.writes[..mmio.write_count]
            .iter()
            .filter(|&&(address, _)| address == hardware_ring)
            .map(|&(_, value)| value)
            .collect::<std::vec::Vec<_>>();
        assert_eq!(duration_writes, [0x1111, 0x2222]);
    }

    #[test]
    fn staged_four_slot_batch_wraps_once_and_triggers_once() {
        let mut mmio = MockPipeMmio::new();
        let pipe = 2_u8;
        let pipe_state = pipe_state_address(pipe);
        let hardware_ring = crate::platform::tx_ring_register(usize::from(pipe), 0) as u32;
        let record = crate::dtcm::MacPipeRecordAddress::from_raw_unchecked(pipe_state);
        for (slot, duration) in [(2, 0x1111), (3, 0x2222), (0, 0x3333), (1, 0x4444)] {
            mmio.set(record.slot_unchecked(slot).duration().get() as u32, duration);
        }
        mmio.set(pipe_state + 4, 8);
        mmio.set(crate::dtcm::LOW_MAC_ACTIVE_TX_COUNT.get() as u32, 7);

        assert!(finalize_staged_pipe(
            &mut mmio,
            pipe,
            pipe_state,
            hardware_ring,
            2,
            1,
        ));

        assert_eq!(mmio.get(PIPE_IRQ_TRIGGER), (1_u32 << pipe) << 25);
        for slot in 0..4 {
            assert_eq!(mmio.get(record.slot_unchecked(slot).state().get() as u32), 1);
        }
        assert_eq!(mmio.get(crate::dtcm::LOW_MAC_ACTIVE_TX_COUNT.get() as u32), 11);
        assert_eq!(mmio.get(hardware_ring + 0x14), 1);
        let duration_writes = mmio.writes[..mmio.write_count]
            .iter()
            .filter(|&&(address, _)| address == hardware_ring)
            .map(|&(_, value)| value)
            .collect::<std::vec::Vec<_>>();
        assert_eq!(duration_writes, [0x1111, 0x2222, 0x3333, 0x4444]);
    }

    #[test]
    fn publication_bisect_matches_only_the_selected_nonzero_boundary() {
        assert_eq!(parse_decimal_u8("0"), 0);
        assert_eq!(parse_decimal_u8("15"), 15);
        assert_eq!(parse_decimal_u8("255"), 255);
        assert!(!publication_bisect_matches(0, 0));
        assert!(!publication_bisect_matches(0, 6));
        assert!(publication_bisect_matches(6, 6));
        assert!(!publication_bisect_matches(6, 7));
    }

    #[test]
    fn ordinary_status_plan_marks_before_suppression_and_reuses_completion_helper_boundary() {
        let input = OrdinaryTxPipeStatusInput {
            pipe_active: true,
            expected_status: 0x0b,
            slot_kind: 0,
            slot_state: 3,
            global_busy: true,
            pipe_current: 1,
            pipe_last: 2,
            pipe_status: 0,
        };
        assert_eq!(
            plan_ordinary_tx_pipe_status(input, 0x0b),
            OrdinaryTxPipeStatusPlan::SuppressedAfterSlotMark
        );

        let input = OrdinaryTxPipeStatusInput {
            global_busy: false,
            slot_kind: 2,
            ..input
        };
        assert_eq!(
            plan_ordinary_tx_pipe_status(input, 0x0b),
            OrdinaryTxPipeStatusPlan::AdvanceWithoutCompletion {
                initialize_pipe_status: true,
                next_current: 2,
            }
        );

        let input = OrdinaryTxPipeStatusInput {
            slot_kind: 0,
            pipe_current: 2,
            pipe_last: 2,
            ..input
        };
        assert_eq!(
            plan_ordinary_tx_pipe_status(input, 0x0b),
            OrdinaryTxPipeStatusPlan::Complete {
                initialize_pipe_status: true,
                cursor: OrdinaryTxPipeCursorPlan::Recycle { next_head: 3 },
            }
        );
    }

    #[test]
    fn tx_status_accounting_plan_matches_vendor_status_classes() {
        assert_eq!(
            plan_mac_tx_status_accounting(0),
            MacTxStatusAccountingPlan {
                increment_range_counter: false,
                specific_counter: Some(0xfff0_2e40),
                increment_total_counter: false,
                service_status_0e_side_effects: false,
            }
        );
        assert_eq!(
            plan_mac_tx_status_accounting(4),
            MacTxStatusAccountingPlan {
                increment_range_counter: false,
                specific_counter: Some(0xfff0_1a94),
                increment_total_counter: true,
                service_status_0e_side_effects: false,
            }
        );
        assert!(plan_mac_tx_status_accounting(6).increment_range_counter);
        assert!(plan_mac_tx_status_accounting(0x18).increment_range_counter);
        assert!(!plan_mac_tx_status_accounting(0x19).increment_range_counter);
        assert!(plan_mac_tx_status_accounting(0x0e).service_status_0e_side_effects);
    }

    #[test]
    fn probe_builder_substitutes_ssid_and_channel() {
        let mut template = [0_u8; 4 + 24 + 2 + 3 + 4];
        template[0] = 0;
        template[1] = 6;
        let frame_len = (template.len() - 4) as u16;
        template[2..4].copy_from_slice(&frame_len.to_le_bytes());
        let ies = 4 + 24;
        template[ies..ies + 2].copy_from_slice(&[0, 0]);
        template[ies + 2..ies + 5].copy_from_slice(&[3, 1, 11]);
        template[ies + 5..ies + 9].copy_from_slice(&[1, 2, 0x82, 0x84]);

        let probe = match prepare_probe(Some(&template), b"test", 6) {
            Ok(probe) => probe,
            Err(error) => panic!("probe build failed: {error:?}"),
        };
        assert_eq!(probe.rate(), 6);
        assert_eq!(
            &probe.bytes()[24..],
            &[0, 4, b't', b'e', b's', b't', 3, 1, 6, 1, 2, 0x82, 0x84]
        );
    }

    #[test]
    fn probe_builder_rejects_non_probe_template() {
        assert_eq!(
            prepare_probe(Some(&[1, 0, 0, 0]), b"", 1).err(),
            Some(ProbeBuildError::WrongTemplateType)
        );
    }

    #[test]
    fn mac_event_decoder_matches_vendor_bitfields() {
        let raw = (1 << 25) | (1 << 24) | (2 << 18) | (3 << 16) | (0x37 << 8) | 0x19;
        assert_eq!(
            MacEvent::decode(raw),
            Some(MacEvent {
                raw,
                event_type: 0x37,
                pipe: 2,
                phase: 3,
                status: 0x19,
                pipe_marker: true,
                completion_marker: true,
                fatal_marker: false,
                pipe_service_marker: false,
                beacon_marker: false,
                sideband_marker: false,
            })
        );
        assert_eq!(MacEvent::decode(u32::MAX), None);
    }

    #[test]
    fn literal_status_2_event_has_no_pipe_service_or_retry_escalation() {
        let event =
            MacEvent::decode(0x0140_3902).unwrap_or_else(|| panic!("status-2 event decoded empty"));
        assert_eq!(event.event_type, 0x39);
        assert_eq!(event.completion_status(), Some(2));
        assert!(event.completion_marker);
        assert!(!event.pipe_service_marker);
        let resolution = resolve_mac_status_event(
            event,
            MacStatusSnapshot {
                pre_service_scheduler_word: SchedulerWord::new(0x100),
                latched_pipe: 0,
                pipe_active: true,
                slot_expected_status: 0x11,
                slot_state: 5,
                global_busy: false,
                mismatch_count: 2,
            },
        )
        .unwrap_or_else(|| panic!("missing status-2 resolution"));
        assert!(resolution.dispatch_ordinary);
        assert!(!resolution.ordinary_completion_eligible);
        assert!(!resolution.direct_retry);
        assert_eq!(resolution.next_mismatch_count, 2);
        assert!(!resolution.escalation_retry);
    }

    #[test]
    fn cooperative_mac_dispatch_uses_vendor_fiq_source() {
        assert!(!mac_fiq_pending(0));
        assert!(mac_fiq_pending(1 << 0x16));
        assert!(!mac_fiq_pending(1 << 0x15));
        assert!(!mac_fiq_pending(1 << 0x17));
        assert!(!mac_service_pending(0, u32::MAX));
        assert!(mac_service_pending(1 << 0x16, u32::MAX));
        assert!(mac_service_pending(0, 0));
    }

    #[test]
    fn fatal_postmortem_captures_pre_acknowledgement_ownership_state() {
        let mut mmio = MockPipeMmio::new();
        mmio.set(crate::dtcm::MAC_CURRENT_PIPE.get() as u32, 2);
        mmio.set(crate::dtcm::MAC_CURRENT_PIPE_RECORD.get() as u32, 0x0400_17f8);
        mmio.set(crate::dtcm::MAC_CURRENT_SLOT.get() as u32, 0x0400_181c);
        mmio.set(crate::dtcm::LOW_MAC_PIPE_BUSY.get() as u32, 1);
        mmio.set(MAC_EVENT_READINESS, 0xffff_ffff);
        mmio.set(PIPE_IRQ_PENDING, 0x400);
        mmio.set(PIPE_IRQ_TRIGGER, 1 << 27);
        let event = MacEvent::decode((1 << 30) | (1 << 24) | 4)
            .unwrap_or_else(|| panic!("fatal event decoded empty"));
        let mut record = MacFatalPostmortem::empty();

        capture_mac_fatal_postmortem(
            &mut mmio,
            event,
            SchedulerWord::new(0x0400),
            0x6000_00d3,
            &mut record,
        );

        assert_eq!(record.valid, 0);
        assert_eq!(record.version, 1);
        assert_eq!(record.line, 222);
        assert_eq!(record.code, 0x29);
        assert_eq!(record.event, event.raw);
        assert_eq!(record.saved_scheduler_word, 0x400);
        assert_eq!(record.cpsr, 0x6000_00d3);
        assert_eq!(record.current_pipe, 2);
        assert_eq!(record.current_pipe_record, 0x0400_17f8);
        assert_eq!(record.current_slot, 0x0400_181c);
        assert_eq!(record.pipe_busy, 1);
        assert_eq!(record.event_readiness, u32::MAX);
        assert_eq!(record.pipe_irq_pending, 0x400);
        assert_eq!(record.pipe_irq_trigger, 1 << 27);
    }

    #[test]
    fn beacon_event_programs_timer_before_scheduler_publication() {
        let mut mmio = MockPipeMmio::new();
        mmio.set(crate::dtcm::LOW_MAC_OPTIONAL_PIPE_OBJECT_WORD.get() as u32, 0x20);
        mmio.set(crate::dtcm::MAC_BEACON_SECONDARY_COMMAND.get() as u32, 0x1234);
        let mut raised = 0;

        execute_mac_beacon_event(&mut mmio, |bits| raised = bits);

        assert_eq!(mmio.get(crate::dtcm::MAC_BEACON_CONTROL_STATE.get() as u32), 5);
        assert_eq!(mmio.get(crate::dtcm::LOW_MAC_CONTROLLER_CONFIG.get() as u32), 0x2000);
        assert_eq!(mmio.get(MAC_BEACON_TIMER + 0x14), 0x8000_9234);
        assert_eq!(raised, 1 << 24);
    }

    #[test]
    fn mac_status_uses_saved_scheduler_word_and_exact_terminal_gate() {
        let event = MacEvent::decode((1 << 24) | 0x0b)
            .unwrap_or_else(|| panic!("status event decoded empty"));
        let resolution = resolve_mac_status_event(
            event,
            MacStatusSnapshot {
                pre_service_scheduler_word: SchedulerWord::new(0x100),
                latched_pipe: 0,
                pipe_active: true,
                slot_expected_status: 0x0b,
                slot_state: 3,
                global_busy: false,
                mismatch_count: 0,
            },
        )
        .unwrap_or_else(|| panic!("missing resolution"));
        assert_eq!(resolution.pending_mask, 0x100);
        assert!(resolution.dispatch_ordinary);
        assert!(resolution.ordinary_completion_eligible);
        assert!(!resolution.direct_retry);
    }

    #[test]
    fn mac_status_mismatch_escalates_only_after_third_bit23_event() {
        let event = MacEvent::decode((1 << 24) | (1 << 23) | 0x0b)
            .unwrap_or_else(|| panic!("status event decoded empty"));
        let resolution = resolve_mac_status_event(
            event,
            MacStatusSnapshot {
                pre_service_scheduler_word: SchedulerWord::new(0x200),
                latched_pipe: 1,
                pipe_active: true,
                slot_expected_status: 4,
                slot_state: 3,
                global_busy: false,
                mismatch_count: 2,
            },
        )
        .unwrap_or_else(|| panic!("missing resolution"));
        assert!(!resolution.ordinary_completion_eligible);
        assert_eq!(resolution.next_mismatch_count, 3);
        assert!(resolution.escalation_retry);
    }

    #[test]
    fn direct_retry_requires_saved_pending_bit_but_not_slot_state_three() {
        let event = MacEvent::decode((1 << 24) | 0x19)
            .unwrap_or_else(|| panic!("status event decoded empty"));
        let resolution = resolve_mac_status_event(
            event,
            MacStatusSnapshot {
                pre_service_scheduler_word: SchedulerWord::new(0x400),
                latched_pipe: 2,
                pipe_active: true,
                slot_expected_status: 0x19,
                slot_state: 4,
                global_busy: false,
                mismatch_count: 0,
            },
        )
        .unwrap_or_else(|| panic!("missing resolution"));
        assert_eq!(resolution.pending_mask, 0x400);
        assert!(!resolution.dispatch_ordinary);
        assert!(resolution.direct_retry);
    }

    #[test]
    fn bit23_mismatch_uses_post_dispatch_slot_and_saved_pending_mask() {
        let event = MacEvent::decode((1 << 24) | (1 << 23) | 0x0b)
            .unwrap_or_else(|| panic!("status event decoded empty"));
        let resolution = resolve_mac_status_event(
            event,
            MacStatusSnapshot {
                pre_service_scheduler_word: SchedulerWord::new(0x100),
                latched_pipe: 0,
                pipe_active: true,
                slot_expected_status: 4,
                slot_state: 5,
                global_busy: false,
                mismatch_count: 2,
            },
        )
        .unwrap_or_else(|| panic!("missing resolution"));
        assert_eq!(resolution.pending_mask, 0x100);
        assert_eq!(resolution.next_mismatch_count, 3);
        assert!(resolution.escalation_retry);

        let no_pending = resolve_mac_status_event(
            event,
            MacStatusSnapshot {
                pre_service_scheduler_word: SchedulerWord::new(0),
                latched_pipe: 0,
                pipe_active: true,
                slot_expected_status: 4,
                slot_state: 5,
                global_busy: false,
                mismatch_count: 2,
            },
        )
        .unwrap_or_else(|| panic!("missing resolution"));
        assert_eq!(no_pending.next_mismatch_count, 2);
        assert!(!no_pending.escalation_retry);
    }

    #[test]
    fn pipe_service_plan_preserves_vendor_priority_and_acknowledgements() {
        assert_eq!(
            plan_mac_pipe_service(SchedulerWord::new(0x1095), 3),
            MacPipeServicePlan::LowNibble {
                asserted: 5,
                backoff_pipe_mask: 5,
                trigger: 5 << 25,
                acknowledgement: !5_u32,
            }
        );
        assert_eq!(
            plan_mac_pipe_service(SchedulerWord::new(0x1080), 3),
            MacPipeServicePlan::MiddleNibble {
                selected_pipe: 3,
                quantum_pointer_address: 0x0400_10e0,
                trigger: 1 << 28,
                acknowledgement: !0x80_u32,
            }
        );
        assert_eq!(
            plan_mac_pipe_service(SchedulerWord::new(0x2000), 1),
            MacPipeServicePlan::HighNibble {
                selected_pipe: 1,
                acknowledgement: !0x2000_u32,
            }
        );
    }

    #[test]
    fn mac_event_backend_zero_budget_performs_no_mmio_equivalent_calls() {
        let mut backend = MockMacEventBackend::new(&[0], 0);
        assert_eq!(
            service_mac_event_backend(&mut backend, 0),
            MacEventLoopReport {
                processed: 0,
                stop: MacEventStopReason::BudgetExhausted,
                reschedule_required: true,
            }
        );
        assert_eq!(backend.event_index, 0);
        assert_eq!(backend.readiness_reads, 0);
        assert_eq!(backend.drain_tails, 0);
    }

    #[test]
    fn mac_event_backend_budget_stops_before_readiness_or_next_pop() {
        let raw = (1 << 25) | (2 << 16) | (0x37 << 8);
        let mut backend = MockMacEventBackend::new(&[raw], 0);
        assert_eq!(
            service_mac_event_backend(&mut backend, 1),
            MacEventLoopReport {
                processed: 1,
                stop: MacEventStopReason::BudgetExhausted,
                reschedule_required: true,
            }
        );
        assert_eq!(backend.event_index, 1);
        assert_eq!(backend.readiness_reads, 0);
        assert_eq!(backend.drain_tails, 0);
    }

    #[test]
    fn mac_event_backend_runs_drain_tail_only_after_fifo_empty() {
        let mut backend = MockMacEventBackend::new(&[0], -1);
        assert_eq!(
            service_mac_event_backend(&mut backend, 2),
            MacEventLoopReport {
                processed: 1,
                stop: MacEventStopReason::Empty,
                reschedule_required: false,
            }
        );
        assert_eq!(backend.event_index, 1);
        assert_eq!(backend.readiness_reads, 1);
        assert_eq!(backend.drain_tails, 1);
    }

    #[test]
    fn completion_callback_tables_use_exact_interface_layout() {
        assert_eq!(completion_device_address(0), 0x0400_3eb0);
        assert_eq!(completion_device_address(1), 0x0400_4260);
        assert_eq!(completion_retry_limit_offset(false), 0x114);
        assert_eq!(completion_retry_limit_offset(true), 0x115);
    }

    #[test]
    fn tala_reduction_preserves_vendor_threshold_order_and_boundaries() {
        let parameters = 0x1914_0f0a;
        assert_eq!(tala_reduction(16, 4, 100, parameters, 2), (8, 0));
        assert_eq!(tala_reduction(16, 5, 100, parameters, 2), (10, 1));
        assert_eq!(tala_reduction(16, 6, 100, parameters, 2), (12, 1));
        assert_eq!(tala_reduction(16, 8, 100, parameters, 2), (14, 2));
        assert_eq!(tala_reduction(16, 11, 100, parameters, 2), (16, 2));
    }

    #[test]
    fn tala_ampdu_length_is_read_at_the_reduction_decision() {
        let reads = core::cell::Cell::new(0);
        let result = tala_reduction_at_decision(
            || {
                reads.set(reads.get() + 1);
                16
            },
            8,
            100,
            0x1914_0f0a,
            2,
        );
        assert_eq!(reads.get(), 1);
        assert_eq!(result, (14, 2));
    }

    #[test]
    fn completion_helper_layout_matches_vendor_records() {
        assert_eq!(lmc_message_address(0), 0x0400_8bb8);
        assert_eq!(lmc_message_address(15), 0x0400_8e4c);
        assert_eq!(lmc_vif_address(0), 0x0400_3e98);
        assert_eq!(lmc_vif_address(2), 0x0400_45f8);
        assert_eq!(phy_operation_7_timer(), (crate::dtcm::MAC_PHY_OPERATION_TIMER.get() as u32, 0x0098_9680));
        assert_eq!(phy_dispatch_switch_target(2), 0x0001_6f92);
        assert_eq!(phy_dispatch_switch_target(3), 0x0001_6fa0);
        assert_eq!(phy_dispatch_switch_target(7), 0x0001_6fb6);
    }

    #[test]
    fn native_internal_context_pool_preserves_context_stride() {
        assert_eq!(crate::dtcm::INTERNAL_TX_CONTEXT_SIZE, TX_CONTEXT_SIZE);
        assert_eq!(crate::dtcm::INTERNAL_TX_CONTEXT_COUNT, TX_CONTEXT_COUNT);
        assert_eq!(crate::dtcm::INTERNAL_CONTEXT_POOL.get(), 0x0400_9080);
        assert_eq!(internal_context_address(0).raw(), 0x0400_9084);
        assert_eq!(
            internal_context_address(2).raw() - internal_context_address(0).raw(),
            (2 * TX_CONTEXT_SIZE) as u32
        );
    }

    #[test]
    fn native_completion_ring_contains_only_the_decoded_fifo() {
        assert_eq!(core::mem::size_of::<CompletionRingState>(), 0x108);
        assert_eq!(core::mem::offset_of!(CompletionRingState, consumer), 0);
        assert_eq!(core::mem::offset_of!(CompletionRingState, producer), 4);
        assert_eq!(core::mem::offset_of!(CompletionRingState, frame_nodes), 8);
    }

    #[test]
    fn only_class_zero_keeps_its_vendor_allocation_counter() {
        assert_eq!(crate::dtcm::class0_internal_context_count().get(), 0x0400_8f71);
    }

    #[test]
    fn completion_enqueue_scheduler_gate_matches_vendor_branches() {
        assert!(completion_drain_required(6, 0x0040, true));
        assert!(completion_drain_required(0, 0x0040, true));
        assert!(!completion_drain_required(0, 0x0048, true));
        assert!(completion_drain_required(0, 0x0048, false));
    }

    #[test]
    fn mac_drain_tail_requires_request_and_either_idle_predicate() {
        assert_eq!(mac_drain_tail_transition(0, true, true), None);
        assert_eq!(mac_drain_tail_transition(2, false, false), None);
        assert_eq!(mac_drain_tail_transition(2, true, false), Some(4));
        assert_eq!(mac_drain_tail_transition(2, false, true), Some(4));
        assert_eq!(mac_drain_tail_transition(0xff, true, false), Some(0xfd));
    }

    #[test]
    fn context_and_frame_node_addresses_round_trip() {
        let context = ContextAddress::from_raw(0x0400_9084).unwrap();
        let frame_node = FrameNodeAddress::from_raw(0x0400_90d8).unwrap();
        assert_eq!(context.frame_node(), frame_node);
        assert_eq!(frame_node.context(), context);
        assert_eq!(ContextAddress::from_raw(0x0400_9088), None);
        assert_eq!(FrameNodeAddress::from_raw(0x0400_90dc), None);

        let publication = PublishedSlotIdentity::new(context, 2, 3).unwrap();
        assert_eq!(publication.context, context);
        assert_eq!(publication.frame_node, frame_node);
        assert_eq!(publication.command, packet_ram::tx_command(2, 3) as u32);
        assert!(PublishedSlotIdentity::new(context, 4, 0).is_none());
        assert!(PublishedSlotIdentity::new(context, 0, 4).is_none());
    }

    #[test]
    fn context_fields_use_typed_host_and_internal_layouts() {
        let internal = ContextAddress::new(0x0400_9084);
        let host = ContextAddress::new(0x0400_5a24);

        assert_eq!(internal.requested_rate_address(), 0x0400_9090);
        assert_eq!(internal.queue_id_address(), 0x0400_9091);
        assert_eq!(internal.request_flags_address(), 0x0400_9093);
        assert_eq!(internal.completion_status_address(), 0x0400_90a4);
        assert_eq!(internal.header_length_address(), 0x0400_90c8);
        assert_eq!(internal.payload_length_address(), 0x0400_90cc);
        assert_eq!(internal.sequence_or_callback_state_address(), 0x0400_90d4);
        assert_eq!(internal.submit_state_address(), 0x0400_90d6);
        assert_eq!(internal.frame_address_address(), 0x0400_90d8);
        assert_eq!(internal.expiry_time_address(), 0x0400_90e8);
        assert_eq!(internal.ownership_bits_address(), 0x0400_9104);
        assert_eq!(internal.timing_reset_32_address(), 0x0400_910a);
        assert_eq!(internal.timing_reset_34_address(), 0x0400_910c);
        assert_eq!(internal.duration_address(), 0x0400_910e);
        assert_eq!(internal.payload_extended_address(), 0x0400_9110);
        assert_eq!(internal.payload_base_address(), 0x0400_9112);
        assert_eq!(internal.word_48_address(), 0x0400_9120);
        assert_eq!(internal.insertion_mode_address(), 0x0400_912b);
        assert_eq!(internal.byte_57_address(), 0x0400_912f);
        assert_eq!(internal.retry_random_address(), 0x0400_9132);
        assert_eq!(internal.link_id_address(), 0x0400_9144);
        assert_eq!(internal.qos_control_address(), 0x0400_914c);
        assert_eq!(internal.cipher_class_address(), 0x0400_914e);
        assert_eq!(internal.word_7c_address(), 0x0400_9154);

        assert_eq!(host.requested_rate_address(), 0x0400_5a30);
        assert_eq!(host.queue_id_address(), 0x0400_5a31);
        assert_eq!(host.request_flags_address(), 0x0400_5a33);
        assert_eq!(host.completion_status_address(), 0x0400_5a44);
        assert_eq!(host.header_length_address(), 0x0400_5a68);
        assert_eq!(host.payload_length_address(), 0x0400_5a6c);
        assert_eq!(host.sequence_or_callback_state_address(), 0x0400_5a74);
        assert_eq!(host.submit_state_address(), 0x0400_5a76);
        assert_eq!(host.frame_address_address(), 0x0400_5a78);
        assert_eq!(host.expiry_time_address(), 0x0400_5a88);
        assert_eq!(host.ownership_bits_address(), 0x0400_5aa4);
        assert_eq!(host.timing_reset_32_address(), 0x0400_5aaa);
        assert_eq!(host.timing_reset_34_address(), 0x0400_5aac);
        assert_eq!(host.duration_address(), 0x0400_5aae);
        assert_eq!(host.payload_extended_address(), 0x0400_5ab0);
        assert_eq!(host.payload_base_address(), 0x0400_5ab2);
        assert_eq!(host.word_48_address(), 0x0400_5ac0);
        assert_eq!(host.insertion_mode_address(), 0x0400_5acb);
        assert_eq!(host.byte_57_address(), 0x0400_5acf);
        assert_eq!(host.retry_random_address(), 0x0400_5ad2);
        assert_eq!(host.link_id_address(), 0x0400_5ae4);
        assert_eq!(host.qos_control_address(), 0x0400_5aec);
        assert_eq!(host.cipher_class_address(), 0x0400_5aee);
        assert_eq!(host.word_7c_address(), 0x0400_5af4);
    }

    #[test]
    fn mac_event_plan_preserves_vendor_multi_marker_order() {
        let raw = (1 << 26)
            | (1 << 25)
            | (1 << 24)
            | (1 << 23)
            | (2 << 18)
            | (3 << 16)
            | (0x37 << 8)
            | (1 << 7)
            | 0x19;
        let event = MacEvent::decode(raw).unwrap_or_else(|| panic!("valid event decoded empty"));
        assert_eq!(
            event.dispatch_plan().effects(),
            &[
                Some(MacEventEffect::Trace),
                Some(MacEventEffect::PipePhase {
                    event_type: 0x37,
                    phase: 3,
                    latch_index: Some(2),
                }),
                Some(MacEventEffect::PipeService),
                Some(MacEventEffect::TxStatus {
                    event_type: 0x37,
                    status: 0x19,
                    pipe_service_escalation: true,
                }),
                Some(MacEventEffect::Beacon),
                Some(MacEventEffect::Sideband),
                Some(MacEventEffect::Archive),
            ]
        );
    }

    #[test]
    fn popped_event_executor_captures_scheduler_once_before_service_and_status() {
        let raw =
            (1 << 26) | (1 << 25) | (1 << 24) | (1 << 23) | (3 << 16) | (0x37 << 8) | (1 << 7) | 4;
        let event = MacEvent::decode(raw).unwrap_or_else(|| panic!("valid event decoded empty"));
        let mut backend = MockPoppedEventEffects::new();

        execute_popped_single_outstanding_event(event, &mut backend);

        assert_eq!(&backend.log[..backend.len], &[1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(
            backend.service_scheduler,
            Some(SchedulerWord::new(0x1234_5678))
        );
        assert_eq!(backend.status_scheduler, backend.service_scheduler);
    }

    #[test]
    fn fatal_mac_event_stops_before_all_later_marker_effects() {
        let raw = (1 << 30) | (1 << 26) | (1 << 25) | (1 << 24) | (1 << 23) | (1 << 7);
        let event = MacEvent::decode(raw).unwrap_or_else(|| panic!("valid event decoded empty"));
        assert_eq!(
            event.dispatch_plan().effects(),
            &[Some(MacEventEffect::Trace), Some(MacEventEffect::Fatal)]
        );
    }

    #[test]
    fn probe_tracker_requires_full_completion_chain_before_return() {
        let mut tracker = ProbeTxTracker::new();
        tracker
            .prepare(0x0400_9084)
            .unwrap_or_else(|error| panic!("{error:?}"));
        tracker
            .publish(2, 1)
            .unwrap_or_else(|error| panic!("{error:?}"));
        let success = MacEvent::decode((1 << 25) | (2 << 18) | (3 << 16) | (0x37 << 8))
            .unwrap_or_else(|| panic!("valid success event decoded as empty"));
        assert_eq!(
            tracker.handle_pipe_event(success, true),
            Err(ProbeTxTransitionError::CompletionBeforeStart)
        );
        let start = MacEvent::decode((1 << 25) | (2 << 18) | (2 << 16) | (0x37 << 8))
            .unwrap_or_else(|| panic!("valid start event decoded as empty"));
        assert_eq!(tracker.handle_pipe_event(start, true), Ok(true));
        assert_eq!(tracker.handle_pipe_event(success, true), Ok(true));
        assert!(tracker.queue_terminal_completion());
        assert!(!begin_probe_completion_callback(
            &mut tracker,
            FrameNodeAddress::new(0x0400_9364 + FRAME_NODE_OFFSET),
        ));
        assert!(begin_probe_completion_callback(
            &mut tracker,
            FrameNodeAddress::new(0x0400_9084 + FRAME_NODE_OFFSET),
        ));
        let mut returned = 0;
        assert!(tracker.finish_completion_callback(|context| returned = context));
        assert_eq!(returned, 0x0400_9084);
        assert!(!tracker.finish_completion_callback(|_| panic!("double return")));
        assert_eq!(tracker.ownership(), ProbeTxOwnership::Returned);
        assert!(tracker.reset_returned());
        assert_eq!(tracker.ownership(), ProbeTxOwnership::Idle);
    }

    #[test]
    fn probe_tracker_retains_retry_status() {
        let mut tracker = ProbeTxTracker::new();
        tracker
            .prepare(0x0400_91f4)
            .unwrap_or_else(|error| panic!("{error:?}"));
        tracker
            .publish(1, 3)
            .unwrap_or_else(|error| panic!("{error:?}"));
        let start = MacEvent::decode((1 << 25) | (1 << 18) | (2 << 16) | (0x37 << 8))
            .unwrap_or_else(|| panic!("valid start event decoded as empty"));
        assert_eq!(tracker.handle_pipe_event(start, true), Ok(true));
        let retry = MacEvent::decode((1 << 24) | 0x19)
            .unwrap_or_else(|| panic!("valid retry event decoded as empty"));
        assert_eq!(tracker.handle_pipe_event(retry, true), Ok(true));
        assert_eq!(tracker.observed_status(), Some(0x19));
        assert!(tracker.resolve_retry_status(0x19));
        assert_eq!(
            tracker.ownership(),
            ProbeTxOwnership::RetryRequired {
                context: 0x0400_91f4,
                pipe: 1,
                slot: 3,
                status: 0x19,
            }
        );
    }

    #[test]
    fn status_observation_requires_explicit_terminal_resolution() {
        let mut tracker = ProbeTxTracker::new();
        tracker
            .prepare(0x0400_9364)
            .unwrap_or_else(|error| panic!("{error:?}"));
        tracker
            .publish(0, 2)
            .unwrap_or_else(|error| panic!("{error:?}"));
        let start = MacEvent::decode((1 << 25) | (2 << 16) | (0x37 << 8))
            .unwrap_or_else(|| panic!("valid start event decoded as empty"));
        assert_eq!(tracker.handle_pipe_event(start, true), Ok(true));
        let status = MacEvent::decode((1 << 24) | 0x0b)
            .unwrap_or_else(|| panic!("valid status event decoded as empty"));
        assert_eq!(tracker.handle_pipe_event(status, false), Ok(true));
        assert_eq!(tracker.observed_status(), Some(0x0b));
        assert_eq!(
            tracker.ownership(),
            ProbeTxOwnership::Started {
                context: 0x0400_9364,
                pipe: 0,
                slot: 2,
            }
        );
        assert!(!tracker.resolve_terminal_status(4));
        assert!(tracker.resolve_terminal_status(0x0b));
        assert_eq!(
            tracker.ownership(),
            ProbeTxOwnership::TerminalObserved {
                context: 0x0400_9364,
                pipe: 0,
                slot: 2,
                status: 0x0b,
            }
        );
    }

    #[test]
    fn probe_identity_generation_and_fatal_quiescence_are_explicit() {
        let mut tracker = ProbeTxTracker::new();
        tracker
            .prepare(0x0400_9084)
            .unwrap_or_else(|error| panic!("{error:?}"));
        let first = tracker
            .identity()
            .unwrap_or_else(|| panic!("missing identity"));
        assert_eq!(first.generation, 1);
        tracker.enter_fatal_quiescence();
        assert_eq!(tracker.identity(), Some(first));
        assert_eq!(
            tracker.prepare(0x0400_91f4),
            Err(ProbeTxTransitionError::Busy)
        );
        assert!(tracker.reset_fatal_quiescence());
        tracker
            .prepare(0x0400_91f4)
            .unwrap_or_else(|error| panic!("{error:?}"));
        let second = tracker
            .identity()
            .unwrap_or_else(|| panic!("missing identity"));
        assert_eq!(second.generation, 2);
        assert_ne!(first, second);
    }

    #[test]
    fn single_frame_secondary_command_matches_vendor_qos_branch() {
        assert_eq!(
            single_frame_secondary_command(0, 0x50, 0),
            0x2100_0000 | (packet_ram::duration_word(0) as u32 & 0x007f_ffff)
        );
        assert_eq!(
            single_frame_secondary_command(0, 0x50, 1),
            0x2100_0000 | (packet_ram::duration_word(1) as u32 & 0x007f_ffff)
        );
        assert_eq!(single_frame_secondary_command(1, 0x50, 0), 0x3200_0050);
    }

    #[test]
    fn single_frame_pas_timing_distinguishes_probe_and_ack_classes() {
        let probe = compute_single_frame_pas_timing(0x117, 0, 50, 0x1300, 0x2c, false)
            .unwrap_or_else(|| panic!("missing probe timing"));
        assert_eq!(probe.frame_kind, 0xff);
        assert_eq!(probe.ack, 0);
        assert_ne!(probe.payload_base, 0);
        assert_eq!(probe.total_airtime, u32::from(probe.payload_base));
        assert_eq!(
            single_frame_slot_duration(probe),
            u32::from(probe.payload_base) * 0x8000
        );

        let authentication =
            compute_single_frame_pas_timing(0x117, 0, 30, 0x0080_1000, 0x2c, false)
                .unwrap_or_else(|| panic!("missing authentication timing"));
        assert_eq!(authentication.frame_kind, 0x11);
        assert_eq!(authentication.ack, 0x2c);
        assert_eq!(
            authentication.total_airtime,
            u32::from(authentication.payload_base) + 0x2c
        );
        assert_eq!(
            single_frame_slot_duration(authentication),
            u32::from(authentication.payload_base) * 0x8000 + 0x202c
        );
    }

    #[test]
    fn single_frame_slot_accepts_both_ack_and_no_ack_frame_kinds() {
        assert!(single_frame_slot_matches(
            0x0400_90d8,
            0x0400_90d8,
            0,
            0xff,
            0xff,
        ));
        assert!(single_frame_slot_matches(
            0x0400_90d8,
            0x0400_90d8,
            0,
            0x11,
            0x11,
        ));
        assert!(!single_frame_slot_matches(
            0x0400_90d8,
            0x0400_90d8,
            0,
            0xff,
            0x11,
        ));
    }

    #[test]
    fn dot11_header_view_and_classification_make_split_boundaries_explicit() {
        assert_eq!(core::mem::size_of::<Dot11FixedHeaderLayout>(), 24);
        assert_eq!(core::mem::offset_of!(Dot11FixedHeaderLayout, frame_control), 0);
        assert_eq!(core::mem::offset_of!(Dot11FixedHeaderLayout, duration), 2);
        assert_eq!(core::mem::offset_of!(Dot11FixedHeaderLayout, address_1), 4);
        assert_eq!(core::mem::offset_of!(Dot11FixedHeaderLayout, address_2), 10);
        assert_eq!(core::mem::offset_of!(Dot11FixedHeaderLayout, address_3), 16);
        assert_eq!(core::mem::offset_of!(Dot11FixedHeaderLayout, sequence_control), 22);

        let frame = TxFrameAddress::new(0x0901_4fe8);
        assert_eq!(frame.frame_control(), 0x0901_4fe8);
        assert_eq!(frame.address_1(), 0x0901_4fec);
        assert_eq!(frame.address_2_byte_unchecked(5), 0x0901_4ff7);
        assert_eq!(frame.address_2_halfword_unchecked(2), 0x0901_4ff6);
        assert_eq!(frame.sequence_control(), 0x0901_4ffe);
        assert_eq!(frame.payload_after_fixed_header(), 0x0901_5000);

        assert_eq!(classify_dot11_header(0x0008, false), Dot11HeaderShape { length: 24, qos_data: false });
        assert_eq!(classify_dot11_header(0x0088, false), Dot11HeaderShape { length: 26, qos_data: true });
        assert_eq!(classify_dot11_header(0x8088, false), Dot11HeaderShape { length: 30, qos_data: true });
        assert_eq!(classify_dot11_header(0x0308, false), Dot11HeaderShape { length: 30, qos_data: false });
        assert_eq!(classify_dot11_header(0x0388, false), Dot11HeaderShape { length: 32, qos_data: true });
        assert_eq!(classify_dot11_header(0x8388, false), Dot11HeaderShape { length: 36, qos_data: true });
        assert_eq!(classify_dot11_header(0x0088, true), Dot11HeaderShape { length: 24, qos_data: false });
    }

    #[test]
    fn tx_frame_validation_rejects_short_frames_aliases_and_range_ends() {
        assert_eq!(
            validated_tx_frame(0x0901_4fe8, 42).map(TxFrameAddress::raw),
            Some(0x0901_4fe8),
        );
        assert_eq!(validated_tx_frame(0x0901_4fe8, 23), None);
        assert_eq!(validated_tx_frame(0x0001_4fe8, 42), None);
        assert_eq!(
            validated_tx_frame(packet_ram::RUNTIME_CPU_END - 24, 24).map(TxFrameAddress::raw),
            Some(packet_ram::RUNTIME_CPU_END - 24),
        );
        assert_eq!(validated_tx_frame(packet_ram::RUNTIME_CPU_END - 23, 24), None);
    }

    #[test]
    fn dot11_header_shapes_keep_software_parsing_separate_from_descriptor_split() {
        assert_eq!(core::mem::size_of::<Dot11FixedHeaderLayout>(), 24);
        let frame = TxFrameAddress::new(0x0901_4fe8);
        assert_eq!(frame.frame_control(), 0x0901_4fe8);
        assert_eq!(frame.address_1(), 0x0901_4fec);
        assert_eq!(frame.address_2_byte_unchecked(0), 0x0901_4ff2);
        assert_eq!(frame.sequence_control(), 0x0901_4ffe);
        assert_eq!(frame.payload_after_fixed_header(), 0x0901_5000);

        assert_eq!(classify_dot11_header(0x0008, false), Dot11HeaderShape { length: 24, qos_data: false });
        assert_eq!(classify_dot11_header(0x0308, false), Dot11HeaderShape { length: 30, qos_data: false });
        assert_eq!(classify_dot11_header(0x0088, false), Dot11HeaderShape { length: 26, qos_data: true });
        assert_eq!(classify_dot11_header(0x8088, false), Dot11HeaderShape { length: 30, qos_data: true });
        assert_eq!(classify_dot11_header(0x0388, false), Dot11HeaderShape { length: 32, qos_data: true });
        assert_eq!(classify_dot11_header(0x8388, false), Dot11HeaderShape { length: 36, qos_data: true });
        assert_eq!(classify_dot11_header(0x0088, true), Dot11HeaderShape { length: 24, qos_data: false });
    }

    #[test]
    fn single_frame_descriptor_matches_vendor_command_shape() {
        let input = SingleFramePipeInput {
            phy_rate_word: 0x123456,
            phy_control_word: 0xabcdef,
            frame_length: 42,
            hardware_rate: 3,
            frame_control: 0x0040,
            retry_flag: true,
            metadata_address: packet_ram::RuntimePacketAddress::new(0x0901_2345, 1).unwrap(),
            duration: 0x0064,
            header_address: packet_ram::RuntimePacketAddress::new(0x0901_4fe8, 42).unwrap(),
            secondary_command: 0x2100_1234,
            address_mask: u32::MAX,
            terminal_command: 0x4e14_0000,
        };
        let descriptor = build_single_frame_pipe_descriptor(input).unwrap();
        assert_eq!(
            descriptor.words(),
            &[
                0x5112_3456,
                0x50ab_cdef,
                0x5203_002e,
                0x3100_0840,
                0x4700_0008,
                0x2081_2345,
                0x3200_0064,
                0x2901_4fec,
                0x2100_1234,
                0x4001_5000,
                0x0001_2000,
                0x4e14_0000,
                0xf000_0000,
            ]
        );
        assert_eq!(
            build_single_frame_pipe_descriptor(SingleFramePipeInput {
                frame_length: DOT11_FIXED_HEADER_LENGTH - 1,
                ..input
            }),
            Err(ProbeBuildError::PacketRamMismatch),
        );
    }

    #[test]
    fn depth_two_block_ack_classifies_wrapped_window_members() {
        assert_eq!(
            classify_depth_two_block_ack(0x0fff, 0b11, [0x0fff, 0x0000]),
            [BlockAckMemberState::Acknowledged; 2],
        );
        assert_eq!(
            classify_depth_two_block_ack(0x0100, 0b01, [0x0100, 0x0101]),
            [BlockAckMemberState::Acknowledged, BlockAckMemberState::Missing],
        );
        let outside = classify_depth_two_block_ack(0x0100, u64::MAX, [0x0100, 0x0140]);
        assert_eq!(
            outside,
            [BlockAckMemberState::Acknowledged, BlockAckMemberState::OutsideWindow],
        );
        assert_eq!(
            plan_depth_two_block_ack_actions(outside, [true, true], true),
            [BlockAckMemberAction::Confirm, BlockAckMemberAction::GiveUp],
        );
        assert_eq!(
            plan_depth_two_block_ack_actions(outside, [true, true], false),
            [BlockAckMemberAction::Confirm, BlockAckMemberAction::GiveUp],
        );
        assert_eq!(
            plan_depth_two_block_ack_actions(
                [BlockAckMemberState::Missing, BlockAckMemberState::Missing],
                [true, false],
                true,
            ),
            [BlockAckMemberAction::Retry, BlockAckMemberAction::GiveUp],
        );
        assert!(depth_two_whole_retry_allowed([true; 2], true, [19; 2]));
        assert!(!depth_two_whole_retry_allowed([true, false], true, [19; 2]));
        assert!(!depth_two_whole_retry_allowed([true; 2], false, [19; 2]));
        assert!(!depth_two_whole_retry_allowed([true; 2], true, [19, 18]));
        assert!(aggregate_retry_command_owned(1, 4));
        assert!(!aggregate_retry_command_owned(1, 3));
        assert!(!aggregate_retry_command_owned(0, 4));
    }

    #[test]
    fn depth_two_block_ack_accumulates_repeated_and_growing_bitmaps() {
        let sequences = [0x0100, 0x0101];
        let first_only = classify_depth_two_block_ack(0x0100, 0b01, sequences);
        let repeated_first = merge_depth_two_block_ack_states(Some(first_only), first_only);
        assert_eq!(
            repeated_first,
            [BlockAckMemberState::Acknowledged, BlockAckMemberState::Missing],
        );

        let second_only = classify_depth_two_block_ack(0x0100, 0b10, sequences);
        assert_eq!(
            merge_depth_two_block_ack_states(Some(repeated_first), second_only),
            [BlockAckMemberState::Acknowledged; 2],
        );
        assert_eq!(
            merge_depth_two_block_ack_states(Some(second_only), first_only),
            [BlockAckMemberState::Acknowledged; 2],
        );

        let total_miss = classify_depth_two_block_ack(0x0100, 0, sequences);
        assert_eq!(total_miss, [BlockAckMemberState::Missing; 2]);
        assert_eq!(
            plan_depth_two_block_ack_actions(total_miss, [true; 2], true),
            [BlockAckMemberAction::Retry; 2],
        );
        assert_eq!(
            plan_depth_two_block_ack_actions(total_miss, [true; 2], false),
            [BlockAckMemberAction::GiveUp; 2],
        );

        // An acknowledgement is sticky for one exact aggregate even if a later
        // compressed BA shifts its window away from that member.
        let shifted = classify_depth_two_block_ack(0x0101, 0b1, sequences);
        assert_eq!(
            merge_depth_two_block_ack_states(Some(first_only), shifted),
            [BlockAckMemberState::Acknowledged; 2],
        );
    }

    #[test]
    fn planned_block_ack_handles_four_wrapped_members_and_sticky_updates() {
        let sequences = [Some(0x0ffe), Some(0x0fff), Some(0x0000), Some(0x0001)];
        let Some(first_half) = classify_planned_block_ack(0x0ffe, 0b0011, sequences) else {
            panic!("four contiguous wrapped sequences must classify");
        };
        assert_eq!(
            first_half.states,
            [
                BlockAckMemberState::Acknowledged,
                BlockAckMemberState::Acknowledged,
                BlockAckMemberState::Missing,
                BlockAckMemberState::Missing,
            ],
        );
        let Some(second_half) = classify_planned_block_ack(0x0ffe, 0b1100, sequences) else {
            panic!("shifted acknowledgement half must classify");
        };
        let Some(merged) = merge_planned_block_ack(Some(first_half), second_half) else {
            panic!("matching aggregate observations must merge");
        };
        assert_eq!(merged.states, [BlockAckMemberState::Acknowledged; 4]);
        assert_eq!(
            plan_planned_block_ack_actions(merged, [true; 4], true),
            [Some(BlockAckMemberAction::Confirm); 4],
        );
    }

    #[test]
    fn planned_block_ack_distinguishes_partial_and_whole_retry() {
        let observation = PlannedBlockAck {
            states: [
                BlockAckMemberState::Acknowledged,
                BlockAckMemberState::Missing,
                BlockAckMemberState::OutsideWindow,
                BlockAckMemberState::Missing,
            ],
            member_count: 4,
        };
        assert_eq!(
            plan_planned_block_ack_actions(observation, [true, true, true, false], true),
            [
                Some(BlockAckMemberAction::Confirm),
                Some(BlockAckMemberAction::Retry),
                Some(BlockAckMemberAction::GiveUp),
                Some(BlockAckMemberAction::GiveUp),
            ],
        );

        let total_miss = PlannedBlockAck {
            states: [BlockAckMemberState::Missing; 4],
            member_count: 4,
        };
        assert!(planned_whole_retry_allowed(
            total_miss,
            [true; 4],
            true,
            [Some(18); 4],
        ));
        assert!(!planned_whole_retry_allowed(
            total_miss,
            [true; 4],
            true,
            [Some(18), Some(18), Some(17), Some(18)],
        ));
        assert!(!planned_whole_retry_allowed(
            total_miss,
            [true; 4],
            false,
            [Some(18); 4],
        ));
        assert!(merge_planned_block_ack(
            Some(PlannedBlockAck {
                member_count: 3,
                ..total_miss
            }),
            total_miss,
        )
        .is_none());
    }

    #[test]
    fn depth_two_ampdu_descriptor_matches_vendor_opcode_stream() {
        assert_eq!(ampdu_spacing_selector(0, 19), 0);
        assert_eq!(ampdu_spacing_selector(4, 19), 4);
        assert_eq!(ampdu_spacing_selector(7, 21), 33);
        assert_eq!(ampdu_spacing_selector(8, 19), 0);
        assert_eq!(ampdu_spacing_selector(4, 13), 0);
        let descriptor = build_depth_two_ampdu_descriptor(DepthTwoAmpduInput {
            first_frame_state: 0x0901_0000,
            second_frame_state: 0x0901_0400,
            first_frame_length: 1500,
            second_frame_length: 1500,
            phy_rate_word: 0x031407,
            phy_control_word: 0x06c006,
            hardware_rate: 0x0c,
            spacing_selector: 0,
        });
        assert_eq!(descriptor.aggregate_length, 0x0bc8);
        assert_eq!(descriptor.length, 4);
        assert_eq!(
            &descriptor.words[..usize::from(descriptor.length)],
            &[0x6501_0008, 0x6600_0000, 0x6501_0408, 0xe400_0000],
        );
        assert_eq!(
            descriptor.phy_words,
            [0x5103_1407, 0x5006_c006, 0x520c_0bc8],
        );

        let spaced = build_depth_two_ampdu_descriptor(DepthTwoAmpduInput {
            spacing_selector: 2,
            ..DepthTwoAmpduInput {
                first_frame_state: 0x0901_0000,
                second_frame_state: 0x0901_0400,
                first_frame_length: 1500,
                second_frame_length: 1500,
                phy_rate_word: 0x031407,
                phy_control_word: 0x06c006,
                hardware_rate: 0x0c,
                spacing_selector: 0,
            }
        });
        assert_eq!(spaced.length, 5);
        assert_eq!(spaced.aggregate_length, 0x0bd0);
        assert_eq!(spaced.phy_words[2], 0x520c_0bd0);
        assert_eq!(
            spaced.words[2],
            ampdu_transfer_word(packet_ram::ampdu_spacing_word_address(2)),
        );
    }

    #[test]
    fn planned_depth_four_ampdu_matches_the_vendor_member_loop() {
        let members = core::array::from_fn(|index| {
            Some(PlannedAmpduMember {
                frame_state: 0x0901_0000 + index as u32 * 0x400,
                frame_length: 1500,
            })
        });
        let input = PlannedAmpduInput {
            members,
            phy_rate_word: 0x031407,
            phy_control_word: 0x06c006,
            hardware_rate: 0x0c,
            spacing_selector: 0,
        };
        let Some(descriptor) = build_planned_ampdu_descriptor(input) else {
            panic!("four contiguous members must produce an opcode plan");
        };
        assert_eq!(descriptor.member_count, 4);
        assert_eq!(descriptor.aggregate_length, 0x1790);
        assert_eq!(descriptor.length, 8);
        assert_eq!(
            &descriptor.words[..usize::from(descriptor.length)],
            &[
                0x6501_0008,
                0x6600_0000,
                0x6501_0408,
                0x6600_0000,
                0x6501_0808,
                0x6600_0000,
                0x6501_0c08,
                0xe400_0000,
            ],
        );
        assert_eq!(descriptor.phy_words[2], 0x520c_1790);

        let Some(spaced) = build_planned_ampdu_descriptor(PlannedAmpduInput {
            spacing_selector: 2,
            ..input
        }) else {
            panic!("spacing must preserve the four-member plan");
        };
        assert_eq!(spaced.length, 11);
        assert_eq!(spaced.aggregate_length, 0x17a8);
        assert_eq!(spaced.phy_words[2], 0x520c_17a8);
        assert_eq!(
            [spaced.words[2], spaced.words[5], spaced.words[8]],
            [ampdu_transfer_word(packet_ram::ampdu_spacing_word_address(2)); 3],
        );

        assert!(build_planned_ampdu_descriptor(PlannedAmpduInput {
            members: [members[0], None, members[2], None],
            ..input
        })
        .is_none());
    }

    #[test]
    fn phy_rate_words_match_legacy_and_ht_classes() {
        let legacy = build_phy_rate_words(0, 2, 0, 3, 5);
        assert_eq!(
            legacy,
            PhyRateWords {
                control: 2,
                rate: 0x0003_0405,
            }
        );
        assert_eq!(finalize_phy_control(legacy, 0, 100), 2);

        let ht_mixed = build_phy_rate_words(14, 0, 0x28, 3, 7);
        assert_eq!(
            ht_mixed,
            PhyRateWords {
                control: 6,
                rate: 0x0003_1407,
            }
        );
        assert_eq!(finalize_phy_control(ht_mixed, 14, 100), 0x0006_c006);

        let ht_greenfield = build_phy_rate_words(14, 0, 0x20, 3, 7);
        assert_eq!(finalize_phy_control(ht_greenfield, 14, 100), 6);
    }

    #[test]
    fn internal_completion_status_matches_vendor_wsm_switch_table() {
        assert_eq!(wsm_status_from_internal(0), 0);
        assert_eq!(wsm_status_from_internal(11), 6);
        assert_eq!(wsm_status_from_internal(10), 7);
        assert_eq!(wsm_status_from_internal(21), 4);
        assert_eq!(wsm_status_from_internal(24), 15);
        assert_eq!(wsm_status_from_internal(25), 1);
    }
}
