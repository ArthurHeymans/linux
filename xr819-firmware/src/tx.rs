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
const WSM_TX_CONTEXT_BASE: usize = crate::dtcm::HOST_TX_CONTEXTS.get();
const WSM_TX_CONTEXT_COUNT: usize = crate::dtcm::HOST_TX_CONTEXT_COUNT;
const WSM_TX_CONTEXT_FREE_HEAD: usize = crate::dtcm::HOST_TX_CONTEXT_FREE_HEAD.get();
const TX_BUFFER_SIZE: usize = packet_ram::INTERNAL_TX_BUFFER_SIZE;
const FRAME_NODE_OFFSET: u32 = 0x54;
// The class-0 allocation counter remains in the untranslated vendor
// accounting record. Its independently decoded non-class-0 counter, probe
// sequence, PAS accounting, and completed-frame FIFO are native Rust state.
const CLASS0_INTERNAL_CONTEXTS: usize = crate::dtcm::CONTEXT_COMPLETION_PREFIX.get() + 5;
const COMPLETION_RING_CAPACITY: usize = 64;

#[inline(always)]
fn internal_context_free_head() -> *mut u32 {
    crate::dtcm::internal_context_free_head_ptr()
}

#[inline(always)]
fn internal_context_base() -> usize {
    crate::dtcm::internal_context_ptr(0).map_or_else(|| unreachable!(), |context| context as usize)
}

#[inline(always)]
fn internal_context_address(index: usize) -> usize {
    crate::dtcm::internal_context_ptr(index)
        .map_or_else(|| unreachable!(), |context| context as usize)
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

#[cfg(not(all(target_arch = "arm", target_feature = "thumb-mode")))]
const SCHEDULER_PENDING: usize = 0x0400_1fd4;
#[cfg(not(target_arch = "arm"))]
const PIPE_RETRY_RANDOM_STATE: u32 = 0x0400_142c;
const PIPE_RECORDS: u32 = 0x0400_1680;
const CURRENT_PIPE: u32 = 0x0400_1f78;
const CURRENT_PIPE_RECORD: u32 = CURRENT_PIPE + 0x0c;
const CURRENT_SLOT: u32 = CURRENT_PIPE + 0x10;
const PIPE_IRQ_PENDING: u32 = crate::platform::mac_register(0x0e84) as u32;
const PIPE_IRQ_TRIGGER: u32 = crate::platform::mac_register(0x0e98) as u32;
const PIPE_QUANTUM_POINTERS: u32 = 0x0400_10d4;
const PIPE_QUANTUM: u32 = 0x0000_0fff;
const PIPE_BUSY: u32 = PIPE_RECORDS + 7;
const QUEUE_BACKOFF_TABLE: u32 = 0x0400_02dc;
const PIPE_STATUS_COUNTER: u32 = 0xfff0_1aa4;
const PIPE_STATUS_ACCOUNTING: u32 = 0x0400_1f7c;
const PIPE_RETRY_INACTIVE_SENTINEL: u32 = 0xff00_ffff;
// `txp_pipe_advance_slot` acknowledges with `-((0x1110 << pipe) + 0x10)`, which
// is a different lane from the `0x100 << pipe` publication ownership mask.
const PIPE_ADVANCE_ACK_BASE: u32 = 0x0000_1110;
const PIPE_RETRY_HARDWARE_STATE: u32 = 0x0400_1e6c;
const PIPE_RETRY_SPECIAL_ACK: u32 = 0x0000_f010;
const PIPE_RETRY_RANDOM_STATS: u32 = 0xfff0_2e7c;
const PIPE_RETRY_RATE_MAP: u32 = 0x0400_1aec;
const PIPE_RETRY_TIMING_TABLE: u32 = 0x0400_0138;
const PAS_ACK_TIMING_TABLE: usize = 0x0400_16c8;
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
const MAC_BEACON_STATE: u32 = 0x0400_1a80;
const MAC_BEACON_CONFIG: u32 = crate::dtcm::LOW_MAC_RUNTIME_ROOT.get() as u32;
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

    pub const fn raw(self) -> u32 {
        self.0
    }

    pub const fn frame_node(self) -> FrameNodeAddress {
        FrameNodeAddress(self.0.wrapping_add(FRAME_NODE_OFFSET))
    }
}

impl FrameNodeAddress {
    pub const fn new(address: u32) -> Self {
        Self(address)
    }

    pub const fn raw(self) -> u32 {
        self.0
    }

    pub const fn context(self) -> ContextAddress {
        ContextAddress(self.0.wrapping_sub(FRAME_NODE_OFFSET))
    }
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

#[derive(Clone, Copy)]
struct PublishedSlotIdentity {
    context: ContextAddress,
    frame_node: FrameNodeAddress,
    pipe: u8,
    slot: u8,
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

fn is_wsm_tx_context(context: u32) -> bool {
    let address = context as usize;
    (WSM_TX_CONTEXT_BASE..WSM_TX_CONTEXT_BASE + WSM_TX_CONTEXT_COUNT * TX_CONTEXT_SIZE)
        .contains(&address)
        && (address - WSM_TX_CONTEXT_BASE).is_multiple_of(TX_CONTEXT_SIZE)
}

unsafe fn release_context_address(context: u32) {
    unsafe {
        let address = context as usize;
        ((address + 0x70) as *mut u16).write_volatile(0x00ff);
        let flags = (address + 0x80) as *mut u32;
        flags.write_volatile(flags.read_volatile() | 0x0002_0000);
        let free_head = internal_context_free_head();
        let old_head = free_head.read_volatile();
        ((address + 4) as *mut u32).write_volatile(old_head);
        free_head.write_volatile(context);
        set_active_internal_contexts(active_internal_contexts().wrapping_sub(1));
    }
}

unsafe fn release_wsm_context_address(context: u32) {
    unsafe {
        let address = context as usize;
        let header = ((address + 0x1c) as *const u32).read_volatile();
        let backing = (0..TX_CONTEXT_COUNT)
            .map(|index| internal_context_address(index) as u32)
            .find(|candidate| expected_header_address(*candidate) == Some(header));
        ((address + 0x20) as *mut u32).write_volatile(0xff);
        ((address + 0x70) as *mut u16).write_volatile(0x00ff);
        let flags = (address + 0x80) as *mut u32;
        flags.write_volatile(flags.read_volatile() | 0x0004_0000);
        let free_head = WSM_TX_CONTEXT_FREE_HEAD as *mut u32;
        ((address + 4) as *mut u32).write_volatile(free_head.read_volatile());
        free_head.write_volatile(context);
        if crate::vif::adjust_host_contexts_in_flight(-1).is_err() {
            crate::halt_always!();
        }
        if let Some(backing) = backing {
            release_context_address(backing);
        }
    }
}

fn expected_header_address(context: u32) -> Option<u32> {
    let offset = usize::try_from(context)
        .ok()?
        .checked_sub(internal_context_base())?;
    if offset % TX_CONTEXT_SIZE != 0 {
        return None;
    }
    let index = offset / TX_CONTEXT_SIZE;
    (index < TX_CONTEXT_COUNT).then_some((packet_ram::internal_tx_buffer(index) + 0x40) as u32)
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
    hardware_rate_code: u8,
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
        rate: class | u32::from(rate_attribute & 0x0f) | (u32::from(hardware_rate_code & 7) << 16),
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
        0x2100_0000 | (packet_ram::duration_word(usize::from(duration_slot)) as u32 & 0x007f_ffff)
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
    pub metadata_address: u32,
    pub duration: u16,
    pub header_address: u32,
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
) -> SingleFramePipeDescriptor {
    let mut words = [0_u32; 13];
    let frame_control = u32::from(input.frame_control) | if input.retry_flag { 0x0800 } else { 0 };
    words[0] = 0x5100_0000 | (input.phy_rate_word & 0x00ff_ffff);
    words[1] = 0x5000_0000 | (input.phy_control_word & 0x00ff_ffff);
    words[2] = 0x5200_0000
        | (u32::from(input.hardware_rate) << 16)
        | u32::from(input.frame_length.wrapping_add(4));
    words[3] = 0x3100_0000 + frame_control;
    words[4] = 0x4700_0000 + (frame_control >> 8);
    words[5] = 0x2080_0000 | (input.metadata_address & 0x007f_ffff);
    words[6] = 0x3200_0000 | u32::from(input.duration);
    words[7] = 0x2900_0000 | (input.header_address.wrapping_add(4) & 0x007f_ffff);
    words[8] = input.secondary_command;
    let mut length = 9;
    if input.frame_length > 24 {
        let payload = input.header_address.wrapping_add(24);
        words[9] = 0x4000_0000 | (input.address_mask & payload & 0xf6ff_ffff);
        words[10] = (u32::from(input.frame_length - 24) & 0x0fff) << 12 | (payload & 3);
        length = 11;
    }
    words[length] = input.terminal_command;
    words[length + 1] = 0xf000_0000;
    SingleFramePipeDescriptor {
        words,
        length: (length + 2) as u8,
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
    output.current_pipe = u32::from(mmio.read_u8(CURRENT_PIPE));
    output.current_pipe_record = mmio.read_u32(CURRENT_PIPE_RECORD);
    output.current_slot = mmio.read_u32(CURRENT_SLOT);
    output.pipe_busy = u32::from(mmio.read_u8(PIPE_BUSY));
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
            quantum_pointer_address: PIPE_QUANTUM_POINTERS + u32::from(selected_pipe) * 4,
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

fn pipe_state_address(pipe: u8) -> u32 {
    PIPE_RECORDS + u32::from(pipe & 3) * 0x6c + 0xa0
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
pub fn advance_pipe_slot<M: MacPipeMmio>(mmio: &mut M, pipe: u8) -> bool {
    let pipe_state = pipe_state_address(pipe);
    let armed = mmio.read_u8(pipe_state + 3) != 0;
    let cursor = if armed {
        mmio.write_u8(pipe_state + 3, 0);
        // Vendor saturates this abort counter at 0xff rather than wrapping.
        let aborts = mmio.read_u8(pipe_state + 6);
        if aborts != 0xff {
            mmio.write_u8(pipe_state + 6, aborts.wrapping_add(1));
        }
        mmio.read_u8(pipe_state + 1).wrapping_add(1) & 3
    } else {
        mmio.read_u8(pipe_state) & 3
    };
    let ring = mmio.read_u32(pipe_state + 8);
    mmio.write_u32(ring + 0x18, PIPE_RETRY_INACTIVE_SENTINEL);
    mmio.write_u32(
        PIPE_IRQ_PENDING,
        0_u32.wrapping_sub((PIPE_ADVANCE_ACK_BASE << (pipe & 3)).wrapping_add(0x10)),
    );
    resync_pipe_ring_cursor(mmio, ring, cursor);
    armed
}

/// Writes both `ring + 0x20` cursor fields, preserving the pending-slot mask in
/// bits 23:0 and the two hardware-owned high bits. This is the only part of
/// `txp_pipe_advance_slot` that the packet controller reads back, and it is the
/// half the open firmware has never performed.
fn resync_pipe_ring_cursor<M: MacPipeMmio>(mmio: &mut M, ring: u32, cursor: u8) {
    let cursor = u32::from(cursor & 3);
    let word = mmio.read_u32(ring + 0x20);
    mmio.write_u32(
        ring + 0x20,
        (word & 0xc0ff_ffff) | (cursor << 24) | (cursor << 27),
    );
}

/// Restores the vendor invariant asserted at the end of `txp_fn_4425`
/// (`0x38c`): the software producer `pipe_state + 0` equals the hardware ring
/// cursor `(ring[0x20] & 0x3fffffff) >> 27`. Unlike `advance_pipe_slot` this
/// touches neither the pipe acknowledgement nor the command sentinel, so it is
/// safe to call from the completion handler that already retired the burst.
pub fn resync_pipe_cursor<M: MacPipeMmio>(mmio: &mut M, pipe: u8) {
    let pipe_state = pipe_state_address(pipe);
    let ring = mmio.read_u32(pipe_state + 8);
    if ring == 0 {
        return;
    }
    let producer = mmio.read_u8(pipe_state);
    resync_pipe_ring_cursor(mmio, ring, producer);
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
    let pipe_state = pipe_state_address(pipe);
    let ring = mmio.read_u32(pipe_state + 8);
    let ring_word = if ring == 0 {
        0
    } else {
        mmio.read_u32(ring + 0x20)
    };
    let packed = u32::from(pipe & 3)
        | (u32::from(mmio.read_u8(pipe_state) & 0x0f) << 4)
        | (u32::from(mmio.read_u8(pipe_state + 1) & 0x0f) << 8)
        | (u32::from(mmio.read_u8(pipe_state + 2) & 0x0f) << 12)
        | (u32::from(mmio.read_u8(pipe_state + 3) & 0x0f) << 16)
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
    let pipe = unsafe { read_u8(CURRENT_PIPE as usize) } & 3;
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
        if packet_ram::contains_owned_storage(address) {
            unsafe { read_u32(address + offset) }
        } else {
            0
        }
    };
    let (completion_consumer, completion_producer) = unsafe { COMPLETION_RING.cursors() };
    let words = [
        event.raw,
        saved_scheduler_word.raw(),
        phase << 24 | u32::from(mismatch_count) << 8 | u32::from(pipe),
        unsafe { read_u32(CURRENT_SLOT as usize) },
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
        u32::from(unsafe { read_u8(PIPE_BUSY as usize) }),
        u32::from(unsafe { read_u8(crate::dtcm::LOW_MAC_ACTIVE_TX_COUNT.get()) }),
        u32::from(unsafe { active_pas_contexts() }),
        completion_consumer,
        completion_producer,
        unsafe { read_u32(PIPE_IRQ_PENDING as usize) },
        unsafe { read_u32(MAC_EVENT_READINESS as usize) },
    ];
    unsafe { crate::host_tx_diagnostics::record_status2_snapshot(phase, &words) };
}

fn current_slot_address<M: MacPipeMmio>(mmio: &mut M, pipe: u8) -> u32 {
    let pipe_state = pipe_state_address(pipe);
    pipe_state
        .wrapping_add(0x0c)
        .wrapping_add(u32::from(mmio.read_u8(pipe_state + 2)) * 0x18)
}

fn publish_current_pipe_slot<M: MacPipeMmio>(mmio: &mut M, pipe: u8) -> (u32, u32) {
    let pipe_state = pipe_state_address(pipe);
    let slot = current_slot_address(mmio, pipe);
    mmio.write_u32(CURRENT_PIPE_RECORD, pipe_state);
    mmio.write_u32(CURRENT_SLOT, slot);
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
    let latched_pipe = mmio.read_u8(CURRENT_PIPE) & 3;
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
                    let slot = current_slot_address(mmio, pipe);
                    let pas = mmio.read_u32(slot + 0x0c);
                    let backoff_word = mmio.read_u32(slot + 0x14);
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
        let pipe_state = postmortem.current_pipe_record as usize;
        let slot = postmortem.current_slot as usize;
        let command = read_u32(slot.wrapping_add(0x14)) as usize;
        let hardware_ring = read_u32(pipe_state.wrapping_add(8)) as usize;
        // The slot the MAC itself is pointing at, which is not necessarily the
        // one we think is current.
        let cursor_slot = ((read_u32(hardware_ring.wrapping_add(0x20)) >> 27) & 3) as usize;
        let cursor_command = packet_ram::tx_command(postmortem.current_pipe as usize, cursor_slot);
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
            // `current_pipe_record` is already `PIPE_RECORDS + pipe*0x6c + 0xa0`
            // (a capture showed 0x04001720 = 0x04001680 + 0xa0 for pipe 0), and
            // `hardware_ring` is read from `+8`, matching vendor's `iVar4+0xa8`.
            // So the slot bytes are at +0..+3, not +0xa0..+0xa3.
            u32::from(read_u8(pipe_state))
                | (u32::from(read_u8(pipe_state.wrapping_add(1))) << 8)
                | (u32::from(read_u8(pipe_state.wrapping_add(2))) << 16)
                | (u32::from(read_u8(pipe_state.wrapping_add(3))) << 24),
            cursor_command as u32,
            read_u32(cursor_command),
            read_u32(cursor_command.wrapping_add(0x14)),
            rx[0],
            rx[1],
            postmortem.current_pipe,
            pipe_state as u32,
            hardware_ring as u32,
            read_u32(hardware_ring.wrapping_add(0x20)),
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

/// Completes one slot the hardware has already left behind, using the vendor
/// give-up status `0x0b` so the host receives a truthful TX failure instead of
/// silently losing the frame.
///
/// # Safety
/// The pipe must be armed, the slot started, and the caller must exclusively
/// own the pipe records and completion state.
#[cfg(target_arch = "arm")]
unsafe fn retire_unmatched_tx_slot<B: TxStatusPolicy>(
    pipe_state: usize,
    slot: usize,
    cursor: OrdinaryTxPipeCursorPlan,
    backend: &mut B,
) {
    unsafe {
        crate::host_tx_diagnostics::bump(crate::host_tx_diagnostics::counter::RETIRED);
        let frame_node = read_u32(slot + 0x0c);
        if frame_node != 0 {
            complete_tx_pipe_slot(
                FrameNodeAddress::new(frame_node),
                slot as u32,
                0x0b,
                backend,
            );
        } else {
            write_u32(slot + 0x0c, 0);
        }
        match cursor {
            OrdinaryTxPipeCursorPlan::AdvanceCurrent { next_current } => {
                write_u8(pipe_state + 2, next_current);
                write_u8(pipe_state, next_current);
            }
            OrdinaryTxPipeCursorPlan::Recycle { next_head } => {
                write_u8(pipe_state, next_head);
                write_u8(pipe_state + 3, 0);
                write_u8(pipe_state + 4, 0);
                write_u8(pipe_state + 5, 5);
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
            let pipe_state = pipe_state_address(pipe) as usize;
            let programmed = read_u8(pipe_state + 4) & 1 != 0;
            let armed = read_u8(pipe_state + 3) == 1;
            let counter = read_u8(pipe_state + 5) as i8;
            match plan_pipe_watchdog(programmed, armed, counter) {
                PipeWatchdogAction::Idle => (),
                PipeWatchdogAction::Tick | PipeWatchdogAction::Nudge => {
                    write_u8(pipe_state + 5, counter.wrapping_sub(1) as u8);
                }
                PipeWatchdogAction::Expired => {
                    let current = read_u8(pipe_state + 2) & 3;
                    let slot = pipe_state + usize::from(current) * 0x18 + 0x0c;
                    let cursor = hardware_pipe_cursor(&mut VolatileMacPipeMmio, pipe)
                        .map(|cursor| OrdinaryTxPipeCursorPlan::Recycle {
                            next_head: cursor & 3,
                        })
                        .unwrap_or(OrdinaryTxPipeCursorPlan::Recycle {
                            next_head: read_u8(pipe_state + 1).wrapping_add(1) & 3,
                        });
                    crate::host_tx_diagnostics::bump(
                        crate::host_tx_diagnostics::counter::WATCHDOG_RECOVERED,
                    );
                    retire_unmatched_tx_slot(pipe_state, slot, cursor, backend);
                    // Vendor `txp_fn_4155` clears the programmed/abort bits and
                    // reloads the counter after a recovery pass.
                    write_u8(pipe_state + 4, read_u8(pipe_state + 4) & 0xf6);
                    write_u8(pipe_state + 5, 5);
                }
            }
        }
    }
}

/// Reads the hardware ring cursor (`ring + 0x20` bits 29:27) for one pipe.
/// `None` when the pipe has no programmed ring.
fn hardware_pipe_cursor<M: MacPipeMmio>(mmio: &mut M, pipe: u8) -> Option<u8> {
    let ring = mmio.read_u32(pipe_state_address(pipe) + 8);
    if ring == 0 {
        return None;
    }
    Some(((mmio.read_u32(ring + 0x20) >> 27) & 3) as u8)
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
    let pipe = unsafe { read_u8(CURRENT_PIPE as usize) } & 3;
    let pipe_state = pipe_state_address(pipe) as usize;
    let slot = current_slot_address(&mut VolatileMacPipeMmio, pipe) as usize;
    unsafe {
        write_u32(CURRENT_PIPE_RECORD as usize, pipe_state as u32);
        write_u32(CURRENT_SLOT as usize, slot as u32);

        let input = OrdinaryTxPipeStatusInput {
            pipe_active: read_u8(pipe_state + 3) == 1,
            expected_status: read_u8(slot + 1),
            slot_kind: read_u8(slot),
            slot_state: read_u8(slot + 3),
            global_busy: read_u8(PIPE_BUSY as usize) != 0,
            pipe_current: read_u8(pipe_state + 2),
            pipe_last: read_u8(pipe_state + 1),
            pipe_status: read_u8(pipe_state + 5),
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
        write_u8(slot + 3, 5);
        match plan {
            OrdinaryTxPipeStatusPlan::Ineligible => unreachable!(),
            OrdinaryTxPipeStatusPlan::SuppressedAfterSlotMark => (),
            OrdinaryTxPipeStatusPlan::AdvanceWithoutCompletion {
                initialize_pipe_status,
                next_current,
            } => {
                if initialize_pipe_status {
                    write_u8(pipe_state + 5, 1);
                }
                write_u32(
                    PIPE_STATUS_COUNTER as usize,
                    read_u32(PIPE_STATUS_COUNTER as usize).wrapping_add(1),
                );
                write_u32(
                    PIPE_STATUS_ACCOUNTING as usize,
                    read_u32(PIPE_STATUS_ACCOUNTING as usize).wrapping_add(1),
                );
                write_u8(pipe_state + 2, next_current);
            }
            OrdinaryTxPipeStatusPlan::Complete {
                initialize_pipe_status,
                cursor,
            } => {
                crate::host_tx_diagnostics::bump(crate::host_tx_diagnostics::counter::COMPLETED);
                if initialize_pipe_status {
                    write_u8(pipe_state + 5, 1);
                }
                let pas = read_u32(slot + 0x0c);
                backend.reset_status_backoff(
                    read_u8(pas as usize + 0x69),
                    read_u8((QUEUE_BACKOFF_TABLE + u32::from(pipe)) as usize),
                    pas.wrapping_add(0x60),
                    QUEUE_BACKOFF_TABLE,
                );
                let pas_address = pas as usize;
                write_u32(pas_address + 0x2c, read_u32(pas_address + 0x2c) | 0x200);
                complete_tx_pipe_slot(FrameNodeAddress::new(pas), slot as u32, 0, backend);
                match cursor {
                    OrdinaryTxPipeCursorPlan::AdvanceCurrent { next_current } => {
                        write_u8(pipe_state + 2, next_current);
                        write_u8(pipe_state, next_current);
                    }
                    OrdinaryTxPipeCursorPlan::Recycle { next_head } => {
                        write_u8(pipe_state, next_head);
                        write_u8(pipe_state + 3, 0);
                        write_u8(pipe_state + 4, 0);
                        write_u8(pipe_state + 5, 5);
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
            write_u32(0x0400_1d50, read_u32(PIPE_RECORDS as usize + 0x14));
            let _ = backend.find_rx_frame_by_subtype(0x80);
            if read_u32(0x0400_1ae8) != 0 {
                let control = read_u32(0x0400_1ab0) & !1;
                write_u32(0x0400_1ab0, control);
                write_u32(crate::platform::mac_register(0x0a00), control);
                write_u32(0x0400_1aa8, 1);
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
    /// Complete the one current non-aggregate frame with vendor status `0x0b`.
    GiveUp,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SingleTxRetryOutcome {
    MaskNotOwned,
    SlotNotStarted,
    InactivePipeAcknowledged,
    Rearmed,
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

    fn complete_give_up(&mut self, frame_node: FrameNodeAddress, slot: u32, status: u16);

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
    let state = _mmio.read_u32(PIPE_RETRY_RANDOM_STATE);

    let mixed = (state >> 4) ^ state;
    let next = (state << 27) | (mixed & 0x07ff_ffff);

    #[cfg(target_arch = "arm")]
    unsafe {
        RETRY_RANDOM_STATE.0.get().write_volatile(next);
    }
    #[cfg(not(target_arch = "arm"))]
    _mmio.write_u32(PIPE_RETRY_RANDOM_STATE, next);

    next & 0x00ff_ffff
}

fn build_single_frame_duration<M: MacPipeMmio>(
    mmio: &mut M,
    descriptor: u32,
    frame_node: FrameNodeAddress,
    expects_ack: bool,
) {
    let frame = frame_node.raw();
    mmio.write_u32(descriptor, 0);

    let interface = u32::from(mmio.read_u8(frame + 0x69));
    let selector = u32::from(mmio.read_u8(frame + 0x0c));
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

    mmio.write_u16(frame + 0x5a, random);
    mmio.write_u32(descriptor + 4, (u32::from(random) & 0x0fff) << 10);

    let duration_word = if expects_ack {
        let rate = u32::from(mmio.read_u8(frame + 0x0f));
        let timing_index = u32::from(mmio.read_u8(PIPE_RETRY_RATE_MAP + rate));
        let timing = u32::from(mmio.read_u16(PIPE_RETRY_TIMING_TABLE + timing_index * 2));
        let duration = mmio
            .read_u32(PIPE_RECORDS + 0x20)
            .wrapping_add(mmio.read_u32(PIPE_RECORDS + 0x1c).wrapping_mul(2))
            .wrapping_add(timing)
            & 0xffff;
        0xd800_0000 | (((duration & 0x03ff) * 8).wrapping_add(0x2000))
    } else {
        0xdc00_0000
    };
    mmio.write_u32(descriptor + 8, duration_word);
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
    let pipe = pipe & 3;
    let pipe_state = pipe_state_address(pipe);

    let flags = mmio.read_u32(frame_node.raw() + 4);
    if mmio.read_u8(slot) == 1 {
        backend.fatal_unsupported_rearm_shape(pipe, slot, frame_node);
    }
    if flags & 0x0008_0000 != 0 {
        backend.rebuild_rate_descriptor(pipe, slot, frame_node);
    }

    let descriptor = mmio.read_u32(slot + 0x14);
    let status = mmio.read_u8(slot + 1);
    build_fixed_rate_retry_duration(mmio, descriptor, frame_node);
    let descriptor_flags = mmio.read_u32(descriptor + 4) | u32::from(status).wrapping_add(0x80);
    mmio.write_u32(descriptor + 4, descriptor_flags);

    mmio.write_u32(PIPE_IRQ_TRIGGER, (1_u32 << pipe) << 25);

    let command = mmio.read_u32(pipe_state + 8);
    if mmio.read_u8(PIPE_RETRY_HARDWARE_STATE) & 2 != 0 {
        mmio.write_u32(command + 0x18, PIPE_RETRY_INACTIVE_SENTINEL);
        mmio.write_u32(
            PIPE_IRQ_PENDING,
            0_u32.wrapping_sub(pending_mask.wrapping_add(PIPE_RETRY_SPECIAL_ACK)),
        );
        return SingleFrameRearmOutcome::HardwareSentinelAcknowledged;
    }

    let producer = mmio.read_u8(pipe_state) & 3;
    let current = mmio.read_u8(pipe_state + 2) & 3;
    if producer != current {
        // Vendor's multi-slot branch first checks a mapped companion slot. For
        // an ordinary legacy batch that slot has kind 0 and a different PAS;
        // with bit 15 clear it takes the simple fallback: clear only the
        // current command-mask bit and acknowledge, without forcing producer
        // to follow the hardware cursor. The remaining remapped/A-MPDU shape
        // is still deliberately unsupported.
        if mmio.read_u8(slot) == 0 && flags & (1 << 15) == 0 {
            let command = mmio.read_u32(pipe_state + 8);
            let command_mask = mmio.read_u32(command + 0x20);
            mmio.write_u32(command + 0x20, command_mask & !(1_u32 << current));
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
        let descriptor_flags = mmio.read_u32(descriptor + 4) | u32::from(status).wrapping_add(0x80);
        mmio.write_u32(descriptor + 4, descriptor_flags);
    }
    let command_mask = mmio.read_u32(command + 0x20);
    mmio.write_u32(command + 0x20, command_mask & !(1_u32 << current));
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
    let pipe = mmio.read_u8(CURRENT_PIPE) & 3;
    let pipe_state = pipe_state_address(pipe);
    mmio.write_u32(CURRENT_PIPE_RECORD, pipe_state);
    let slot = current_slot_address(mmio, pipe);
    mmio.write_u32(CURRENT_SLOT, slot);
    let owned_mask = 0x100_u32 << pipe;
    let pending = scheduler_word.raw();

    if pending & owned_mask == 0 {
        return SingleTxRetryOutcome::MaskNotOwned;
    }
    if mmio.read_u8(slot + 3) != 3 {
        return SingleTxRetryOutcome::SlotNotStarted;
    }

    let frame_node = FrameNodeAddress::new(mmio.read_u32(slot + 0x0c));
    // Vendor ownership transition: started -> retry-owned, then globally busy.
    mmio.write_u8(slot + 3, 4);
    mmio.write_u8(PIPE_BUSY, 1);

    if mmio.read_u8(pipe_state + 3) != 1 {
        for selected_pipe in 0..4_u8 {
            if pending & (0x100_u32 << selected_pipe) != 0 {
                let command = mmio.read_u32(pipe_state_address(selected_pipe) + 8);
                mmio.write_u32(command + 0x18, PIPE_RETRY_INACTIVE_SENTINEL);
            }
        }
        mmio.write_u32(
            PIPE_IRQ_PENDING,
            0_u32.wrapping_sub(owned_mask.wrapping_add(1)),
        );
        return SingleTxRetryOutcome::InactivePipeAcknowledged;
    }

    if mmio.read_u8(slot + 1) == 6 {
        backend.fatal_unsupported_multi_slot_retry(pipe, slot, frame_node);
    }
    if (mmio.read_u8(pipe_state + 5) as i8) < 1 {
        mmio.write_u8(pipe_state + 5, 1);
    }

    match backend.decide_retry(pipe, slot, frame_node) {
        SingleTxRetryDecision::Rearm => {
            backend.rearm_and_ack(pipe, slot, frame_node, owned_mask);
            SingleTxRetryOutcome::Rearmed
        }
        SingleTxRetryDecision::GiveUp => {
            // The trigger precedes completion and the command completion word
            // follows `txp_fn_2441`, exactly as in the non-status-6 branch.
            mmio.write_u32(PIPE_IRQ_TRIGGER, (1_u32 << pipe) << 25);
            let current = mmio.read_u8(pipe_state + 2);
            let last = mmio.read_u8(pipe_state + 1);
            backend.complete_give_up(frame_node, slot, 0x0b);
            let command = mmio.read_u32(pipe_state + 8);
            mmio.write_u32(command + 0x1c, 1);

            let next = current.wrapping_add(1) & 3;
            mmio.write_u8(pipe_state + 2, next);
            mmio.write_u8(pipe_state, next);
            if current == last {
                mmio.write_u8(pipe_state + 3, 0);
                mmio.write_u8(pipe_state + 4, 0);
                mmio.write_u8(pipe_state + 5, 5);
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
            unsafe { write_u8(CURRENT_PIPE as usize, pipe) };
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
                        write_u8(PIPE_RECORDS as usize + 6, 0);
                        service_pipe_tx_start(pipe, self.backend);
                    }
                } else {
                    unsafe { service_mac_irq_count_status(event_type) };
                }
            }
            3 | 1 if phase == 3 || event_type == 0x19 => unsafe {
                if read_u8(PIPE_RECORDS as usize + 0x0a) != 0 {
                    write_u8(PIPE_RECORDS as usize + 0x0a, 0);
                    let pending = 0x0400_1fd4_usize;
                    write_u32(pending, read_u32(pending) | 0x10);
                }
                if read_u8(PIPE_RECORDS as usize + 6) != 0 {
                    let index = usize::from(read_u8(PIPE_RECORDS as usize + 0x0c));
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
        let pipe = unsafe { read_u8(CURRENT_PIPE as usize) } & 3;
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

        let pipe_state = pipe_state_address(pipe) as usize;
        if unsafe { read_u8(pipe_state + 3) } != 1 {
            return;
        }
        let slot = pipe_state + 0x0c + usize::from(unsafe { read_u8(pipe_state + 2) }) * 0x18;
        if unsafe { read_u8(slot + 1) } == status {
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
    let mut previous_link = 0x0400_2014_u32;
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
    mmio.read_u32(0x0400_2014) == timer
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
    let pending = mmio.read_u32(0x0400_1fd4);
    let claimed = pending & mask;
    if claimed != 0 {
        mmio.write_u32(0x0400_1fd4, pending & !claimed);
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
        let pending = read_u32(0x0400_1fd8) & !0x10;
        write_u32(0x0400_1fd8, pending);
        if pending == 0 {
            write_u32(0x0400_1fd4, read_u32(0x0400_1fd4) | 4);
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
        if read_u32(0x0400_1fd8) & 0x10 != 0 && read_u32(0x0400_2014) == timer {
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
            .wrapping_add(read_u32(0x0400_143c))
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

        if became_head && read_u32(0x0400_2028) == 0 {
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

#[cfg(target_arch = "arm")]
pub struct SingleProbeMacBackend {
    retry: BoundedSingleTxRetry,
    mismatch: [u8; 4],
    publications: [Option<PublishedSlotIdentity>; 16],
    completed: [Option<HostClass0Completion>; 4],
}

#[cfg(target_arch = "arm")]
impl SingleProbeMacBackend {
    pub const fn new(max_retries: u8) -> Self {
        Self {
            retry: BoundedSingleTxRetry::new(max_retries),
            mismatch: [0; 4],
            publications: [None; 16],
            completed: [None; 4],
        }
    }

    fn register_publication(
        &mut self,
        batch: BatchPosition,
        context: ContextAddress,
        pipe: u8,
        slot: u8,
    ) {
        if batch == BatchPosition::Only {
            self.retry.reset();
            self.mismatch = [0; 4];
            self.publications = [None; 16];
        } else if batch == BatchPosition::First {
            self.retry.reset();
            self.mismatch[usize::from(pipe & 3)] = 0;
        }
        let publication_index = usize::from(pipe & 3) * 4 + usize::from(slot & 3);
        self.publications[publication_index] = Some(PublishedSlotIdentity {
            context,
            frame_node: context.frame_node(),
            pipe: pipe & 3,
            slot: slot & 3,
        });
    }

    fn push_completion(&mut self, completion: HostClass0Completion) {
        let Some(entry) = self.completed.iter_mut().find(|entry| entry.is_none()) else {
            terminal_probe_backend_fault(completion.pipe)
        };
        *entry = Some(completion);
    }

    pub fn take_completion(&mut self) -> Option<HostClass0Completion> {
        self.completed.iter_mut().find_map(|entry| entry.take())
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
        let interface = u32::from(mmio.read_u8(frame_node + 0x69));
        let selector = u32::from(mmio.read_u8(frame_node + 0x0c));
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
    fn find_pipe_by_mac_upper(&mut self, _mac_upper: u32) -> Option<u32> {
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
                        // The vendor-host runtime retains class-0 context and
                        // HIF ownership until its WSM confirmation is actually
                        // published by the main dispatcher.
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
            let ack_failures = if is_wsm_tx_context(context.raw()) {
                unsafe { read_u16(frame_node.raw() as usize + 0x1e) }.min(u16::from(u8::MAX)) as u8
            } else {
                self.retry.attempts()
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
        let context = frame_node.raw().wrapping_sub(FRAME_NODE_OFFSET);
        if unsafe { prepare_host_frame_timing(context) }.is_err() {
            terminal_probe_backend_fault(pipe);
        }
        let command = unsafe { read_u32(slot as usize + 0x14) };
        if command == 0
            || unsafe { emit_host_frame_descriptor_at(context, command.wrapping_add(0x0c)) }
                .is_err()
        {
            terminal_probe_backend_fault(pipe);
        }
        unsafe {
            write_u32(
                frame_node.raw() as usize + 4,
                read_u32(frame_node.raw() as usize + 4) & !0x000c_0000,
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

#[cfg(target_arch = "arm")]
impl SingleTxRetryBackend for SingleProbeMacBackend {
    fn decide_retry(
        &mut self,
        _pipe: u8,
        _slot: u32,
        frame_node: FrameNodeAddress,
    ) -> SingleTxRetryDecision {
        if !is_wsm_tx_context(frame_node.raw().wrapping_sub(FRAME_NODE_OFFSET)) {
            return self.retry.decide();
        }

        let context = frame_node.raw().wrapping_sub(FRAME_NODE_OFFSET);
        let rate = unsafe { read_u8(frame_node.raw() as usize + 0x0f) };
        let status_address = context + 0x28 + u32::from(rate >> 3) * 4;
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

        let policy_index = unsafe { read_u8(frame_node.raw() as usize + 0x0e) };
        let Some(policy) = crate::rate_policy::get(policy_index) else {
            return self.retry.decide();
        };
        let try_count = unsafe { read_u16(frame_node.raw() as usize + 0x1e) };
        let flags = unsafe { read_u32(frame_node.raw() as usize + 4) };
        let long_frame = (flags & 0x7ff) >> 9 != 0;
        match crate::rate_policy::retry_step(policy, rate, try_count, long_frame) {
            crate::rate_policy::RetryStep::GiveUp => SingleTxRetryDecision::GiveUp,
            crate::rate_policy::RetryStep::Rearm { rate: next_rate } => {
                let rate_changed = next_rate != rate && (flags & 0x20 == 0 || next_rate > 13);
                let first_retry = flags & 0x10 == 0;
                if first_retry || rate_changed {
                    unsafe {
                        if rate_changed {
                            write_u8(frame_node.raw() as usize + 0x0f, next_rate);
                        }
                        write_u32(
                            frame_node.raw() as usize + 4,
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
                    write_u16(frame_node.raw() as usize + 0x1e, try_count.wrapping_add(1));
                }
                self.retry.record_rearm();
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
        execute_fixed_rate_single_frame_rearm(
            &mut VolatileMacPipeMmio,
            pipe,
            slot,
            frame_node,
            pending_mask,
            self,
        );
    }

    fn complete_give_up(&mut self, frame_node: FrameNodeAddress, slot: u32, status: u16) {
        unsafe { crate::host_tx_diagnostics::bump(crate::host_tx_diagnostics::counter::GIVE_UP) };
        unsafe { complete_tx_pipe_slot(frame_node, slot, status, self) };
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
    if !is_wsm_tx_context(context) || pipe >= 4 || slot >= 4 {
        return Err(ProbeBuildError::UnsupportedPublicationShape);
    }
    let frame_node = FrameNodeAddress::new(context + FRAME_NODE_OFFSET);
    if !single_frame_slot_matches(
        unsafe { read_u32(slot_record as usize + 0x0c) },
        frame_node.raw(),
        unsafe { read_u8(slot_record as usize) },
        unsafe { read_u8(slot_record as usize + 1) },
        unsafe { read_u8(frame_node.raw() as usize + 0x56) },
    ) {
        return Err(ProbeBuildError::PipeSlotOwnershipMismatch);
    }
    let pipe_state = pipe_state_address(pipe);
    let hardware_ring = unsafe { read_u32(pipe_state as usize + 8) };
    let live_command = unsafe { read_u32(slot_record as usize + 0x14) };
    if command != live_command || !packet_ram::tx_commands().contains(&(command as usize)) {
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
    if hardware_ring == 0 {
        return Err(ProbeBuildError::PipeStateUnavailable);
    }
    #[cfg(all(feature = "vendor-host-tx-diagnostics", target_arch = "arm"))]
    unsafe {
        crate::hif::validate_tx_boundary(0x12, pipe, slot, command, hardware_ring);
    }

    let runtime = unsafe { &mut *PROBE_EXPERIMENT.0.get() };
    runtime
        .backend
        .register_publication(batch, ContextAddress::new(context), pipe, slot);
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
                expects_ack: read_u8(frame_node.raw() as usize + 0x56) != 0xff,
                batch,
            },
        );
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
/// Bits 0..7 contain retry attempts, bit 8 reports a queued completion, and
/// bits 16..31 contain its internal status when present.
#[cfg(target_arch = "arm")]
pub unsafe fn host_class0_runtime_diagnostic() -> u32 {
    let runtime = unsafe { &*PROBE_EXPERIMENT.0.get() };
    u32::from(runtime.backend.retry.attempts())
        | runtime
            .backend
            .completed
            .iter()
            .find_map(|completion| *completion)
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
            in("r0") SCHEDULER_PENDING,
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
        let pending = SCHEDULER_PENDING as *mut u32;
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
        let node = frame_node.raw() as usize;
        let context = frame_node.context().raw() as usize;
        COMPLETION_RING.enqueue(frame_node);

        let flags = (node + 0x2c) as *mut u32;
        flags.write_volatile(flags.read_volatile() | (1 << 14));
        if ((node + 0x1c) as *const u16).read_volatile() == 0x16 {
            let completion_flags = (node + 0x50) as *mut u16;
            completion_flags.write_volatile(completion_flags.read_volatile() | 2);
        } else {
            raise_scheduler_bits(1 << 21);
        }

        let completion_class = ((context + 0x53) as *const u8).read_volatile();
        let frame_control = ((node + 0x0a) as *const u16).read_volatile();
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
    fn find_pipe_by_mac_upper(&mut self, mac_upper: u32) -> Option<u32>;
}

fn lmc_message_address(index: u8) -> u32 {
    (0x0400_8bb8_usize + usize::from(index) * 0x2c) as u32
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
pub unsafe fn allocate_lmc_message<F>(allocation_failed: F) -> Option<u32>
where
    F: FnOnce(),
{
    unsafe {
        let state = 0x0400_8ad8_usize;
        let producer = read_u8(state + 0xd3).wrapping_add(1) & 0x0f;
        if read_u8(state + 0xd4) == producer {
            allocation_failed();
            return None;
        }
        write_u8(state + 0xd3, producer);
        Some(lmc_message_address(producer))
    }
}

/// Exact status-`0x0b` BA transition at matching-payload `0x6dae`.
///
/// # Safety
/// `context` and the pointer returned by `find_pipe` must reference valid
/// vendor records.
pub unsafe fn mark_ba_session_state_5<F>(context: ContextAddress, find_pipe: F)
where
    F: FnOnce(u32) -> Option<u32>,
{
    unsafe {
        if read_u16(crate::dtcm::LOW_MAC_OPTIONAL_PIPE_OBJECT_WORD.get()) == 0 {
            return;
        }
        let header = read_u32(context.raw() as usize + 0x54);
        let Some(pipe) = find_pipe(header.wrapping_add(4)) else {
            return;
        };
        let state = read_u8(pipe as usize + 6);
        if state & 4 == 0 {
            write_u8(pipe as usize + 6, 5);
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
        let command = 0x0400_1d40_usize;
        let output = 0x0400_1d48_usize;
        let global_state = 0x0400_99a9_usize;
        write_u8(command, 3);
        if read_u8(global_state) == 4 {
            write_u8(output + 1, read_u8(output + 1) | 2);
            write_u8(global_state, 5);
        }
        write_u8(output, read_u8(global_state));
        write_u32(output + 4, 0x0098_9680);
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
        let command = 0x0400_1d40_usize;
        let output = 0x0400_1d48_usize;
        let global_state = 0x0400_99a9_usize;
        write_u8(command, 2);
        write_u8(command + 1, secondary);
        if read_u8(global_state) != 5 {
            write_u8(0x0400_997c, secondary);
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
        write_u32(output + 4, 0x0098_9680);
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
        let state = 0x0400_1d20_usize;
        let output = state + 0x18;
        let global_state = 0x0400_99a9_usize;
        write_u8(state + 0x10, 1);
        write_u8(state + 0x21, 0);
        if read_u8(global_state) != 5 {
            write_u8(global_state, 3);
        }
        write_u8(output, read_u8(global_state));
        write_u32(output + 4, 0x0098_9680);
        write_u32(state + 0x0c, u32::from(read_u8(output)));
        if publication_bisect_reached(4) {
            return 4;
        }
        start_scheduler_timer((state - 8) as u32, read_u32(state + 0x1c))
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
        let slot = slot as usize;
        let timestamp = read_u32(0x0ac0_0004);
        let mut final_link_state = 10_u8;
        let link = read_u8(first_frame_node.raw() as usize + 0x6c);
        let slot_kind = read_u8(slot);

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
            let descriptor = read_u32(slot + 0x10);
            write_u32(descriptor as usize, read_u32(0x0400_1f90));
            write_u32(0x0400_1f90, descriptor);
        }

        backend.rate_recovery_on_success(first_frame_node, status);
        write_u32(slot + 0x0c, 0);

        let mut frame_node = first_frame_node;
        loop {
            let node = frame_node.raw() as usize;
            if slot_kind == 1 {
                write_u32(node + 0x2c, read_u32(node + 0x2c) | 0x400);
                write_u16(node + 0x50, read_u16(node + 0x50) | 1);
            }
            let class_bits = ((read_u32(node + 4) >> 18) & 0x0c) as u16;
            write_u16(node + 0x50, read_u16(node + 0x50) | class_bits);
            write_u16(node + 0x1c, status);
            write_u32(node + 0x14, timestamp);

            if slot_kind == 1 {
                if status == 0 {
                    raise_scheduler_bits(1 << 21);
                } else if status == 0x0b {
                    final_link_state = 0x0b;
                }
            } else {
                write_u32(node + 0x2c, read_u32(node + 0x2c) | 0x800);
                enqueue_completion_frame_node(frame_node, || {
                    backend.special_completion_gate(frame_node)
                });
            }

            let next = read_u32(node + 0x3c);
            if next == 0 {
                break;
            }
            frame_node = FrameNodeAddress::new(next);
        }

        if slot_kind == 1 {
            if link < 8 {
                backend.link_set_state(link, final_link_state);
            }
            if final_link_state != 0x0b {
                let ba_table = crate::dtcm::ba_pipe_record_address_unchecked(0).get();
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
                        let entry = ba_table + usize::from(selected_pipe) * 0x38;
                        if read_u8(entry + 0x10) > 4 {
                            write_u32(entry + 0x18, read_u32(frame + 0x14));
                            write_u32(entry + 0x1c, read_u32(frame + 0x18));
                            write_u16(entry + 0x16, read_u16(frame + 0x12) >> 4);
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
                    let fifo = read_u32(0x0400_1f84) as usize;
                    if read_u8(fifo + 2) == read_u8(fifo + 1) && link < 8 {
                        let entry = ba_table + usize::from(link) * 0x38;
                        write_u32(entry + 0x18, 0);
                        write_u32(entry + 0x1c, 0);
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
        let global = 0x0400_1680_usize;
        let pipe_state = global + usize::from(pipe) * 0x6c + 0xa0;
        let current = read_u8(pipe_state + 2);
        let current_slot = pipe_state + usize::from(current) * 0x18 + 0x0c;
        let current_frame = FrameNodeAddress::new(read_u32(current_slot + 0x0c));

        write_u32(0xfff0_1a98, read_u32(0xfff0_1a98).wrapping_add(1));
        write_u8(global + 7, 0);
        write_u8(current_slot + 3, 3);

        if read_u8(pipe_state + 3) == 1 && read_u8(current_slot + 1) == 0xff {
            let frame = current_frame.raw() as usize;
            let flags = read_u32(frame + 4);
            if flags & (1 << 4) == 0 {
                backend.set_frame_lifetime(current_frame);
            }
            if flags & (1 << 15) == 0 {
                backend.reset_backoff(
                    read_u8(frame + 0x69),
                    read_u8(0x0400_02dc + usize::from(pipe)),
                );
            }

            let last = read_u8(pipe_state + 1);
            if current == last {
                let mut index = read_u8(pipe_state);
                loop {
                    let slot = pipe_state + usize::from(index) * 0x18 + 0x0c;
                    complete_tx_pipe_slot(
                        FrameNodeAddress::new(read_u32(slot + 0x0c)),
                        slot as u32,
                        0,
                        backend,
                    );
                    if index == last {
                        break;
                    }
                    index = index.wrapping_add(1) & 3;
                }
                write_u8(pipe_state, last.wrapping_add(1) & 3);
                write_u8(pipe_state + 3, 0);
                write_u8(pipe_state + 4, 0);
                write_u8(pipe_state + 5, 5);
            } else {
                write_u8(pipe_state + 2, current.wrapping_add(1) & 3);
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
        write_u8(0x0400_1f8c + usize::from(pipe), 0);
        if read_u32(0x0400_1d2c) == 4 {
            dispatch_phy_command_3();
            write_u32(0x0400_1d2c, 2);
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
        let global = 0x0400_1680_usize;
        write_u32(
            global + 0x40,
            read_u32(crate::platform::mac_register(0x0604)),
        );
        let pipe_state = global + usize::from(pipe) * 0x6c + 0xa0;
        let current = read_u8(pipe_state + 2);
        let slot = pipe_state + usize::from(current) * 0x18 + 0x0c;
        if read_u8(pipe_state + 3) == 0 {
            backend.start_without_pending_diagnostic(pipe);
            return;
        }

        crate::host_tx_diagnostics::bump(crate::host_tx_diagnostics::counter::TX_START);
        write_u8(slot + 3, 2);
        let frame_node = read_u32(slot + 0x0c) as usize;
        if read_u32(0x0400_1d2c) == 3 {
            let secondary = read_u8(0x0400_0194 + usize::from(read_u8(frame_node + 0x0f)));
            dispatch_phy_command_2(secondary);
            if read_u8(0x0400_1d48) == 4 {
                write_u32(0x0400_1d2c, 4);
            } else {
                raise_scheduler_bits(1 << 18);
            }
        }

        if read_u8(pipe_state + 3) == 1
            && read_u8(slot) == 0
            && read_u16(frame_node + 0x0a) & 0x0f != 4
            && read_u32(frame_node + 4) & 1 == 0
        {
            let rate = read_u8(frame_node + 0x6a);
            let duration = read_u16(packet_ram::duration_word(usize::from(rate)));
            let header = read_u32(frame_node) as usize;
            write_u16(header + 0x16, duration);
            write_u16(frame_node + 0x54, duration);
            write_u32(frame_node + 4, read_u32(frame_node + 4) | 1);
            write_u8(global + 6, 1);
            write_u8(global + 0x0c, rate);
        }

        let slot_state = read_u32(slot + 0x14);
        if read_u32(slot_state as usize + 8) & (1 << 27) == 0 && current != read_u8(pipe_state + 1)
        {
            write_u8(pipe_state + 2, current.wrapping_add(1) & 3);
        }
    }
}

#[allow(unreachable_code)]
pub unsafe fn release_lmc_radio_scheduler<B: RadioCompletionEffects>(owner: u32, backend: &mut B) {
    unsafe {
        let owner_address = owner as usize;
        let interface = usize::from(read_u8(owner_address + 0x0d));
        let released_vif = lmc_vif_address(interface);
        if read_u16(released_vif + 0x30) != 0 {
            write_u32(0x0400_8b2c, owner);
            return;
        }
        write_u32(0x0400_8b2c, 0);
        if read_u32(0x0400_8b20) != owner {
            infallible_to_never(backend.radio_owner_mismatch());
        }

        write_u32(0x0400_8b20, 0);
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
                write_u32(0x0400_8b20, (vif + 0x44) as u32);
                write_u8(vif + 0x66, 3);
                return;
            }
        }

        let pending = read_u32(0x0400_8b24);
        if pending == 0 {
            return;
        }
        write_u32(0x0400_8b20, pending);
        write_u32(0x0400_8b24, read_u32(pending as usize + 4));
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
    (0x0400_1d18, 0x0098_9680)
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
        let state = 0x0400_1d20_usize;
        write_u8(state + 0x10, 7);
        write_u8(state + 0x21, 0);
        write_u8(state + 0x18, 1);
        let (timer, duration) = phy_operation_7_timer();
        write_u32(state + 0x1c, duration);
        write_u32(state + 0x0c, u32::from(read_u8(state + 0x18)));
        let timeout = read_u32(state + 0x1c);
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
        let context = context.raw() as usize;
        if read_u16(context + 0x70) == 0x16 {
            return;
        }
        let interface = read_u8(context + 0xbd);
        let state = crate::dtcm::power_save_observed_view(usize::from(interface))
            .map_or_else(|| unreachable!(), crate::dtcm::DtcmAddress::get);
        if read_u8(state + 0xfc) != 0
            && (read_u32(context + 0x58) & 0x03ff_ffff) >> 24 == 0
            && read_u16(context + 0x5e) & 0x4f != 0x48
        {
            write_u16(state + 0x44, read_u16(state + 0x44) | 8);
            backend.timer_start((state + 0xe8) as u32, read_u32(state + 0x118));
        }

        if read_u32(0x0400_1ae8) != 0 {
            if read_u8(0x0400_94fe) < 3 {
                return;
            }
            backend.timer_start((state + 0xe8) as u32, 0x0000_1f40);
            if read_u8(0x0400_94fe) == 4 {
                backend.try_enter_sleep_all();
            }
            return;
        }

        if read_u16(context + 0x70) == 0 {
            write_u16(lmc_vif_address(usize::from(interface)) + 0x1e2, 0);
        }
        let mode = read_u8(state + 0x40);
        if mode != 1 {
            if mode == 0 && read_u8(0x0400_94fe) >= 3 {
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
        let context_flags = read_u32(context + 0x58);
        if context_flags & (1 << 25) != 0 {
            backend.timer_start((state + 0xc0) as u32, read_u32(0x0400_9504));
            return;
        }

        let queue = read_u8(context + 0x60);
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
        let node = frame_node.raw() as usize;
        let interface = usize::from(read_u8(node + 0x69));
        let override_value = read_u16(0xfff0_1a7c + interface * 2);
        if override_value != 0 && crate::vif::rts_threshold(interface as u8).unwrap_or(0) >= 0x660 {
            return;
        }

        let status = read_u16(node + 0x1c);
        let success = 0x0400_8f4c + interface * 4;
        let failure = 0x0400_8f54 + interface * 4;
        let tries_total = 0x0400_8f5c + interface * 4;
        let penalty = 0x0400_8f64 + interface * 4;
        if status == 0 {
            write_u32(success, read_u32(success).wrapping_add(1));
        } else if status == 0x0b {
            write_u32(failure, read_u32(failure).wrapping_add(1));
        }

        let completed = read_u32(success).wrapping_add(read_u32(failure));
        let policy = usize::from(read_u8(node + 0x0e));
        let short_retries = u32::from(read_u8(
            crate::dtcm::rate_policy_short_retry_limit_compat(policy).get(),
        ));
        let expected = short_retries.wrapping_mul(15).wrapping_add(99) / 100;
        let tries = u32::from(read_u16(node + 0x1e));
        write_u32(tries_total, read_u32(tries_total).wrapping_add(tries));
        let addition = if tries > (expected & 0xffff) {
            1_u32 << (tries.wrapping_sub(expected & 0xffff).wrapping_add(1) & 0x0f)
        } else {
            tries
        };
        write_u32(penalty, read_u32(penalty).wrapping_add(addition));

        let parameter0 = read_u32(0x0400_1fc0);
        let evaluation_window = ((parameter0 >> 16) & 0xff).wrapping_mul(200);
        if completed < evaluation_window && read_u32(penalty) <= 0x400 {
            return;
        }

        let parameter1 = read_u32(0x0400_1fc4);
        let weighted_penalty = read_u32(penalty).wrapping_mul(100);
        let weighted_total = short_retries.wrapping_mul(completed);
        let (mut next, minimum) = tala_reduction_at_decision(
            || crate::vif::ampdu_length(interface as u8).unwrap_or(0),
            weighted_total,
            weighted_penalty,
            parameter1,
            (parameter0 >> 8) & 0xff,
        );

        let sample_count = read_u32(0x0400_1f80);
        let sample_divisor = read_u32(0x0400_1f7c);
        if sample_count.wrapping_add(sample_divisor) == 0 {
            if read_u8(0x0400_1f79) != 0
                && read_u16(0x0400_1fb4) < 50
                && read_u32(tries_total)
                    .wrapping_add(read_u32(success))
                    .wrapping_mul(80)
                    < read_u32(tries_total).wrapping_mul(100)
            {
                write_u8(0x0400_1f79, 0);
            }
        } else {
            let sample = sample_count.wrapping_mul(600) / sample_count.wrapping_add(sample_divisor);
            let average =
                sample.wrapping_add(u32::from(read_u16(0x0400_1fb4)).wrapping_mul(4)) / 10;
            write_u16(0x0400_1fb4, average as u16);
            if average > 75 {
                write_u8(0x0400_1f79, 1);
            }
            write_u32(0x0400_1f80, 0);
            write_u32(0x0400_1f7c, 0);
        }

        let control = read_u8(0x0400_1fbc);
        if weighted_total.wrapping_mul(parameter1 & 0xff) >> 1 < weighted_penalty {
            write_u8(crate::dtcm::TALA_ACCOUNTING.get() + interface, 0);
            if control & 2 == 0 {
                write_u8(0x0400_1fbc, control | 1);
            }
        } else {
            let streak_address = crate::dtcm::TALA_ACCOUNTING.get() + interface;
            let streak = read_u8(streak_address).wrapping_add(1);
            write_u8(streak_address, streak);
            if ((parameter0 >> 24) & 0x0f) <= u32::from(streak) {
                write_u8(0x0400_1f79, 1);
                let average = read_u16(0x0400_1fb4);
                write_u16(0x0400_1fb4, average.wrapping_sub(average >> 5));
                next = next.wrapping_add(1) & 0xff;
                if control & 2 != 0 {
                    write_u8(0x0400_1fbc, control | 1);
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
        if read_u32(0x0400_140c) != 0 {
            let mut index = cursor.next;
            while index != cursor.target {
                let node = COMPLETION_RING.frame_node(index);
                if read_u8(node.wrapping_sub(1) as usize) == 0 {
                    zero_class_pending = zero_class_pending.wrapping_add(1);
                }
                index = index.wrapping_add(1) & 0x3f;
            }
        }

        while let Some(frame_node) = cursor.pop() {
            let node = frame_node.raw() as usize;
            let context = frame_node.context();
            let context_address = context.raw() as usize;
            let status = read_u16(node + 0x1c);
            let interface = usize::from(read_u8(node + 0x69));

            update_tala_for_completion(frame_node);
            write_u16(node.wrapping_sub(0x2e), read_u32(node + 0x50) as u16);

            let flags = read_u32(node + 4);
            if flags & (1 << 5) != 0 {
                let stats = 0x0400_12a0_usize;
                let accumulated =
                    u64::from(read_u32(stats + 8)) | (u64::from(read_u32(stats + 0x0c)) << 32);
                let accumulated = accumulated.wrapping_add(u64::from(read_u16(node + 8)));
                write_u32(stats + 8, accumulated as u32);
                write_u32(stats + 0x0c, (accumulated >> 32) as u32);
                write_u32(stats + 4, read_u32(stats + 4).wrapping_add(1));

                if flags & (1 << 6) != 0 {
                    write_u32(stats, read_u32(stats).wrapping_add(1));
                    if backend.completion_messages_enabled()
                        && (flags >> 20) & 3 != 0
                        && read_u8(0x0400_8ba8) & 1 != 0
                    {
                        if let Some(message) =
                            allocate_lmc_message(|| backend.message_allocation_failed())
                        {
                            let message = message as usize;
                            write_u8(message, 7);
                            write_u8(message + 0x28, interface as u8);
                            write_u8(message + 4, read_u8(node + 0x52));
                            write_u8(message + 0x29, read_u8(node + 0x6c));
                            let queue = usize::from(read_u8(node + 0x0c));
                            write_u8(message + 5, read_u8(0x0400_02e0 + queue));
                            write_u16(message + 6, read_u16(node + 0x54) << 4);
                            let header = read_u32(node) as usize;
                            write_u16(message + 8, read_u16(header + 4));
                            write_u16(message + 0x0a, read_u16(header + 6));
                            write_u16(message + 0x0c, read_u16(header + 8));
                            raise_scheduler_bits(1 << 22);
                        }
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
            if interface < 3 {
                if crate::vif::adjust_tx_busy(interface as u8, -1).is_err() {
                    crate::halt_always!();
                }
            }
            if interface < 2 {
                service_power_save_completion(context, backend);
            }
            if zero_class_pending > 1 && read_u8(node.wrapping_sub(1)) == 0 {
                write_u16(
                    context_address + 0x26,
                    read_u16(context_address + 0x26) | 0x20,
                );
                zero_class_pending = zero_class_pending.wrapping_sub(1);
            }
            write_u32(node + 0x2c, read_u32(node + 0x2c) | 0x8000);
            backend.complete_context(context, status);
        }

        if active_pas_contexts() == 0 && backend.completion_idle_policy_enabled() {
            start_phy_operation_7(backend);
            let owner = read_u32(0x0400_8b2c);
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
        let address = context.raw() as usize;
        if ((address + 0x70) as *const u16).read_volatile() == 0x00ff {
            return CompletedContextDispatch::NoCallback;
        }

        let saved_status = ((address + 0x72) as *const u16).read_volatile();
        ((address + 0x25) as *mut u8).write_volatile(saved_status as u8);
        ((address + 0x70) as *mut u16).write_volatile(completion_status);
        ((address + 0x20) as *mut u32).write_volatile(u32::from(completion_status));

        let aggregate = ((address + 0x4c) as *const u32).read_volatile();
        if aggregate != 0 {
            let budget = (aggregate as usize + 7) as *mut u8;
            let budget_value = budget.read_volatile() as i8;
            if budget_value > 0 && ((address + 0x53) as *const u8).read_volatile() == 0 {
                budget.write_volatile((budget_value - 1) as u8);
            }
        }

        let interface = ((address + 0xbd) as *const u8).read_volatile();
        if interface < 2 {
            let device = completion_device_address(interface);
            let retry_class = if completion_status == 0
                && ((address + 0x5e) as *const u16).read_volatile() & 0x0f == 8
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
                let peer = ((address + 0x54) as *const u32).read_volatile() as usize;
                if countdown_value != 0 && ((peer + 4) as *const u8).read_volatile() & 1 != 0 {
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

        let completion_class = ((address + 0x53) as *const u8).read_volatile();
        let callback_address =
            (0x0400_0260_usize + usize::from(completion_class) * 4) as *const u32;
        if callback_address.read_volatile() == 0 {
            return CompletedContextDispatch::NoCallback;
        }
        let ownership_flags = (address + 0x80) as *mut u32;
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
            if retry_limit != 0xff && ((address + 0x63) as *const u8).read_volatile() <= retry_limit
            {
                let retry_count = (device + 0x0f) as *mut u8;
                let mut retries = retry_count.read_volatile();
                if completion_status == 0 && retries < 5 {
                    retries = retries.wrapping_add(1);
                    retry_count.write_volatile(retries);
                }
                let consumed = ((address + 0x25) as *const u8).read_volatile();
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
        let address = context.raw() as usize;
        let free_head = internal_context_free_head();
        ((address + 4) as *mut u32).write_volatile(free_head.read_volatile());
        ((address + 0x70) as *mut u16).write_volatile(0x00ff);
        let flags = (address + 0x80) as *mut u32;
        flags.write_volatile(flags.read_volatile() | (1 << 17));
        free_head.write_volatile(context.raw());

        let class = ((address + 0x53) as *const u8).read_volatile();
        if class == 0 {
            let counter = CLASS0_INTERNAL_CONTEXTS as *mut u8;
            counter.write_volatile(counter.read_volatile().wrapping_sub(1));
        } else {
            set_active_internal_contexts(active_internal_contexts().wrapping_sub(1));
        }

        let pending_queue = 0x0400_8ad8 as *mut u8;
        if pending_queue.add(0x0b).read_volatile() != 0 {
            service_pending_queue();
        }
        let control = pending_queue.add(0xd0);
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
    mmio.write_u32(MAC_BEACON_STATE + 0x28, 5);
    if mmio.read_u16(MAC_BEACON_CONFIG + 0x12) != 0 {
        mmio.write_u16(PIPE_RECORDS + 8, 0x2000);
        let timer = mmio
            .read_u32(MAC_BEACON_STATE + 0x2c)
            .wrapping_add(u32::from(mmio.read_u16(MAC_BEACON_CONFIG + 0x12)) * 0x400)
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
        let pending = 0x0400_1fd4 as *mut u32;
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
        if read_u32(0x0400_1ae8) != 0 {
            write_u32(crate::dtcm::LOW_MAC_BAND_BITS.get(), 1);
        }
        write_u8(PIPE_RECORDS as usize + 6, 1);
        let interface = usize::from(read_u8(0x0400_1d58));
        write_u8(
            PIPE_RECORDS as usize + 0x0c,
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
                let state = read_u32(0x0400_1aa8);
                if state == 4 {
                    let pending = 0x0400_1fd4_usize;
                    write_u32(pending, read_u32(pending) | (1 << 24));
                } else if state != 5 {
                    service_mac_beacon_event();
                }
                write_u32(0x0400_1aa8, 1);
            }
            0x35 if read_u8(0x0400_8b95) == 2 => {
                write_u8(0x0400_8b95, 4);
                let pending = 0x0400_1fd4_usize;
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
        (0x0400_1d14 as *mut u32).write_volatile(captured);
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
    unsafe { (0x0400_1abc as *mut u32).write_volatile(raw) }
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
        let record = 0x0400_1720_usize + pipe * 0x6c;
        if unsafe { ((record + 3) as *const u8).read_volatile() } != 0 {
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
        let control = (0x0400_1e6c as *mut u8).read_volatile();
        let next = mac_drain_tail_transition(control, mac_hardware_idle(), mac_pipe_records_idle());
        if let Some(next) = next {
            (0x0400_1e6c as *mut u8).write_volatile(next);
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
        if frame_type != 0x50 && (0x0400_860c as *const u8).read_volatile() != 0 {
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
    duration: u32,
) {
    let frame = input.frame_node.raw();
    // Vendor txp_scheduler_run performs no descriptor readback between its
    // trigger and GO writes. Gather the expensive image directly into BSS
    // while the pipe is inactive, avoiding a 152-byte firmware stack copy.
    unsafe {
        crate::host_tx_diagnostics::begin_pre_go_snapshot();
        crate::host_tx_diagnostics::write_pre_go_snapshot_word(
            0,
            mmio.read_u32(input.slot_record) & 0x00ff_ffff | 0x0100_0000,
        );
        crate::host_tx_diagnostics::write_pre_go_snapshot_word(1, duration);
        crate::host_tx_diagnostics::write_pre_go_snapshot_word(
            2,
            mmio.read_u32(input.slot_record + 0x0c),
        );
        crate::host_tx_diagnostics::write_pre_go_snapshot_word(
            3,
            mmio.read_u32(input.slot_record + 0x10),
        );
        crate::host_tx_diagnostics::write_pre_go_snapshot_word(
            4,
            mmio.read_u32(input.slot_record + 0x14),
        );
        crate::host_tx_diagnostics::write_pre_go_snapshot_word(5, mmio.read_u32(frame + 4));
        crate::host_tx_diagnostics::write_pre_go_snapshot_word(
            6,
            u32::from(mmio.read_u16(frame + 8)),
        );
        crate::host_tx_diagnostics::write_pre_go_snapshot_word(
            7,
            u32::from(mmio.read_u16(frame + 0x0a)),
        );
        crate::host_tx_diagnostics::write_pre_go_snapshot_word(
            8,
            u32::from(mmio.read_u8(frame + 0x0d)),
        );
        crate::host_tx_diagnostics::write_pre_go_snapshot_word(
            9,
            u32::from(mmio.read_u8(frame + 0x0f)),
        );
        crate::host_tx_diagnostics::write_pre_go_snapshot_word(
            10,
            u32::from(mmio.read_u16(frame + 0x36)),
        );
        crate::host_tx_diagnostics::write_pre_go_snapshot_word(
            11,
            u32::from(mmio.read_u16(frame + 0x38)),
        );
        crate::host_tx_diagnostics::write_pre_go_snapshot_word(
            12,
            u32::from(mmio.read_u16(frame + 0x3a)),
        );
        crate::host_tx_diagnostics::write_pre_go_snapshot_word(13, mmio.read_u32(frame + 0x48));
        crate::host_tx_diagnostics::write_pre_go_snapshot_word(
            14,
            u32::from(mmio.read_u8(frame + 0x56)),
        );
        crate::host_tx_diagnostics::write_pre_go_snapshot_word(
            15,
            u32::from(mmio.read_u8(frame + 0x69)),
        );
        crate::host_tx_diagnostics::write_pre_go_snapshot_word(
            16,
            u32::from(mmio.read_u8(frame + 0x6a)),
        );
        for index in 0..16_u32 {
            crate::host_tx_diagnostics::write_pre_go_snapshot_word(
                17 + index as usize,
                mmio.read_u32(input.command_storage + index * 4),
            );
        }
        crate::host_tx_diagnostics::write_pre_go_snapshot_word(33, duration);
        crate::host_tx_diagnostics::write_pre_go_snapshot_word(
            34,
            mmio.read_u32(input.hardware_ring + 0x0c),
        );
        crate::host_tx_diagnostics::write_pre_go_snapshot_word(
            35,
            mmio.read_u32(input.hardware_ring + 0x10),
        );
        crate::host_tx_diagnostics::write_pre_go_snapshot_word(36, 0);
        crate::host_tx_diagnostics::write_pre_go_snapshot_word(
            37,
            mmio.read_u32(input.pipe_state) & 0x00ff_ffff | 0x0100_0000,
        );
    }
}

pub fn execute_single_probe_publication<M: MacPipeMmio>(
    mmio: &mut M,
    input: SingleProbePublicationInput,
) -> u8 {
    let pipe = input.pipe & 3;
    let slot = input.slot & 3;
    let frame = input.frame_node.raw();

    // `current` tracks hardware progress through the batch, so only the first
    // staged frame sets it (vendor: `*(byte *)(iVar4 + 0xa2) = *pbVar8`, once,
    // before the publish loop).
    if input.batch.sets_current() {
        mmio.write_u8(input.pipe_state + 2, slot);
    }
    // The inactive first-submission branch of `txp_scheduler_run` does not
    // call `txp_pipe_advance_slot`; startup already synchronized the ring and
    // software cursors. Slot-advance publication belongs only to cleanup/rearm
    // paths where pipe state +3 was already active.
    let ownership_flags = mmio.read_u32(frame + 0x2c) | 0x100;
    mmio.write_u32(frame + 0x2c, ownership_flags);
    let timestamp = mmio.read_u32(0x0ac0_0004);
    mmio.write_u32(frame + 0x18, timestamp);
    mmio.write_u32(frame + 0x3c, 0);
    if publication_bisect_reached(4) {
        return 4;
    }

    let timing = SingleFramePasTiming {
        payload_extended: mmio.read_u16(frame + 0x38),
        payload_base: mmio.read_u16(frame + 0x3a),
        ack: mmio.read_u16(frame + 0x36),
        total_airtime: mmio.read_u32(frame + 0x48),
        frame_kind: mmio.read_u8(frame + 0x56),
    };
    let expects_ack = timing.frame_kind != 0xff;
    debug_assert_eq!(input.expects_ack, expects_ack);
    mmio.write_u32(input.slot_record + 8, single_frame_slot_duration(timing));
    build_single_frame_duration(mmio, input.command_storage, input.frame_node, expects_ack);
    if expects_ack {
        let flags = mmio.read_u32(input.command_storage + 4)
            | u32::from(timing.frame_kind).wrapping_add(0x80);
        mmio.write_u32(input.command_storage + 4, flags);
    }
    mmio.write_u8(input.pipe_state + 1, slot);
    mmio.write_u32(input.hardware_ring + 0x14, 0);
    #[cfg(feature = "vendor-host-tx-diagnostics")]
    {
        let diagnostic_duration = mmio.read_u32(input.slot_record + 8);
        prepare_pre_go_publication(mmio, input, diagnostic_duration);
    }
    if publication_bisect_reached(5) {
        return 5;
    }

    let interface = u32::from(mmio.read_u8(frame + 0x69));
    let pas = crate::dtcm::pas_stride_view_unchecked(interface as usize);
    let edca_slot_timing = mmio.read_u32(pas.packed_aifs().get() as u32);
    if mmio.read_u32(0x0400_1b04) != edca_slot_timing {
        mmio.write_u32(
            crate::platform::mac_register(0x0e64) as u32,
            edca_slot_timing,
        );
        mmio.write_u32(0x0400_1b04, edca_slot_timing);
    }
    if publication_bisect_reached(6) {
        return 6;
    }

    let queue = u32::from(mmio.read_u8(0x0400_02dc + u32::from(pipe)));
    let mut quantum =
        u32::from(mmio.read_u16(pas.txop_limit_unchecked(queue as usize).get() as u32));
    let airtime = mmio.read_u32(frame + 0x48) & 0xffff;
    if quantum == 0 {
        if (mmio.read_u32(frame + 4) & 0x0fff) >> 10 != 0 {
            quantum = airtime;
        }
    } else if quantum <= airtime {
        let frame_policy = mmio.read_u16(frame + 0x50) | 8;
        mmio.write_u16(frame + 0x50, frame_policy);
        quantum = airtime;
    }
    let quantum_destination = mmio.read_u32(PIPE_QUANTUM_POINTERS + u32::from(pipe) * 4);
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
            slot,
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
    mmio.write_u8(input.slot_record + 3, 1);
    let duration = mmio.read_u32(input.slot_record + 8);
    mmio.write_u32(input.hardware_ring, duration);
    mmio.write_u8(input.pipe_state + 3, 1);
    let pipe_flags = mmio.read_u8(input.pipe_state + 4) | 1;
    mmio.write_u8(input.pipe_state + 4, pipe_flags);
    mmio.write_u8(input.pipe_state + 5, 5);
    #[cfg(feature = "vendor-host-tx-diagnostics")]
    unsafe {
        let actual_ring_duration = mmio.read_u32(input.hardware_ring);
        crate::host_tx_diagnostics::write_pre_go_snapshot_word(33, actual_ring_duration);
        crate::host_tx_diagnostics::commit_pre_go_snapshot();
    }
    #[cfg(all(feature = "vendor-host-tx-diagnostics", target_arch = "arm"))]
    unsafe {
        crate::hif::validate_tx_boundary(
            0x14,
            pipe,
            slot,
            input.command_storage,
            input.hardware_ring,
        );
    }
    mmio.write_u32(input.hardware_ring + 0x14, 1);
    #[cfg(all(feature = "vendor-host-tx-diagnostics", target_arch = "arm"))]
    unsafe {
        crate::hif::validate_tx_boundary(
            0x15,
            pipe,
            slot,
            input.command_storage,
            input.hardware_ring,
        );
    }
    0
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
    slot_record: u32,
    command: u32,
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
            let slot_record = self.slot_record as usize;
            let valid = single_frame_slot_matches(
                read_u32(slot_record + 0x0c),
                frame_node.raw(),
                read_u8(slot_record),
                read_u8(slot_record + 1),
                read_u8(frame_node.raw() as usize + 0x56),
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

            backend.register_publication(
                BatchPosition::Only,
                ContextAddress::new(self.context.context),
                self.pipe,
                self.slot,
            );
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
            let live_command = read_u32(slot_record + 0x14);
            if self.command != live_command
                || !packet_ram::tx_commands().contains(&(self.command as usize))
            {
                crate::hif::publish_halting_exception(
                    [
                        0x5458_4341,
                        u32::from(self.pipe),
                        u32::from(self.slot),
                        self.slot_record,
                        self.command,
                        live_command,
                        pipe_state,
                        hardware_ring,
                        read_u32(slot_record),
                        read_u32(slot_record + 8),
                        read_u32(slot_record + 0x0c),
                        read_u32(slot_record + 0x10),
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
            if hardware_ring == 0 {
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
            crate::radio::record_tx_command_signature(6, self.command);
            execute_single_probe_publication(
                &mut VolatileMacPipeMmio,
                SingleProbePublicationInput {
                    pipe: self.pipe,
                    slot: self.slot,
                    pipe_state,
                    slot_record: self.slot_record,
                    command_storage: self.command,
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
            let slot_record = self.slot_record as usize;
            let expected = self.context.context + 0x54;
            if ((slot_record + 0x0c) as *const u32).read_volatile() != expected {
                return Err(ProbeBuildError::PipeSlotOwnershipMismatch);
            }
            ((slot_record + 0x0c) as *mut u32).write_volatile(self.original_slot_frame);
            (slot_record as *mut u32).write_volatile(self.original_slot_header);
            for (index, word) in self.original_command.into_iter().enumerate() {
                ((self.command as usize + index * 4) as *mut u32).write_volatile(word);
            }
            if is_wsm_tx_context(self.context.context) {
                release_wsm_context_address(self.context.context);
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
        let frame = context.context as usize + FRAME_NODE_OFFSET as usize;
        let interface = usize::from(read_u8(frame + 0x69));
        if interface > 2 {
            return Err(ProbeBuildError::InvalidInterface);
        }
        let flags = read_u32(frame + 4);
        let rate = read_u8(frame + 0x0f);
        let pas = crate::dtcm::pas_stride_view_unchecked(interface);
        let rate_map = pas.rate_map_unchecked(usize::from(rate));
        let timing_index = usize::from(read_u8(rate_map.get()));
        // Preserve frame+0x0d exactly as initialized by the vendor HIF path
        // from `(wsm_tx_flags & 0x0f) >> 1`. `txp_submit_to_pipe()` passes this
        // rate attribute directly to `pas_build_phy_rate_words()`. The rate
        // map index below selects ACK timing only; writing it into frame+0x0d
        // changed healthy vendor `0x5104....` words into `0x5107....`.
        let ack_table = if flags & 0x4000 != 0 {
            PAS_ACK_TIMING_TABLE + (0x74 - 0x48)
        } else {
            PAS_ACK_TIMING_TABLE
        };
        let ack_duration = read_u16(ack_table + timing_index * 2);
        let mode = read_u8(pas.mode_byte().get());
        let header = read_u32(frame) as usize;
        let special_peer = (mode == 5 || mode == 6)
            && (0..6).all(|offset| {
                read_u8(header + 10 + offset)
                    == read_u8(
                        crate::dtcm::low_mac_peer_address_byte_unchecked(interface, offset).get(),
                    )
            });
        let timing = compute_single_frame_pas_timing(
            read_u16(PIPE_RECORDS as usize + 2),
            rate,
            read_u16(frame + 8),
            flags,
            ack_duration,
            special_peer,
        )
        .ok_or(ProbeBuildError::UnsupportedPublicationShape)?;

        write_u16(frame + 0x32, 0);
        write_u16(frame + 0x34, 0);
        write_u16(frame + 0x36, timing.ack);
        write_u16(frame + 0x38, timing.payload_extended);
        write_u16(frame + 0x3a, timing.payload_base);
        write_u32(frame + 0x48, timing.total_airtime);
        write_u8(frame + 0x56, timing.frame_kind);
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
        let context_address = context as usize;
        free_head.write_volatile(((context_address + 4) as *const u32).read_volatile());

        set_active_internal_contexts(active_internal_contexts().wrapping_add(1));
        ((context_address + 0x0d) as *mut u8).write_volatile(0);
        ((context_address + 0x4c) as *mut u32).write_volatile(0);
        ((context_address + 0x53) as *mut u8).write_volatile(6);
        ((context_address + 0x52) as *mut u8).write_volatile(1);
        let sequence = probe_context_sequence();
        ((context_address + 0x50) as *mut u16).write_volatile(sequence);
        set_probe_context_sequence(sequence.wrapping_add(1));
        ((context_address + 0x0f) as *mut u8).write_volatile(0);
        ((context_address + 0xa7) as *mut u8).write_volatile(1);
        ((context_address + 0x58) as *mut u32).write_volatile(0);
        ((context_address + 0x70) as *mut u16).write_volatile(0x00fe);
        ((context_address + 0x72) as *mut u16).write_volatile(0);
        ((context_address + 0xa4) as *mut u16).write_volatile(0);
        ((context_address + 0x80) as *mut u32).write_volatile(1);
        ((context_address + 0x90) as *mut u32).write_volatile(0);
        // Vendor sources `ctx+0x98` from a ROM-owned pointer. Preserve the
        // pool value until that ROM/global state is translated; zero is not a
        // reference-faithful substitute once the context becomes live.
        ((context_address + 0x60) as *mut u8)
            .write_volatile((0x0400_02dc as *const u8).read_volatile());
        ((context_address + 0x61) as *mut u8).write_volatile(0);

        let header = ((context_address + 0x1c) as *const u32).read_volatile();
        if expected_header_address(context) != Some(header) {
            release_context_address(context);
            return Err(ProbeBuildError::InvalidContextPointer);
        }
        copy_to_packet_ram(header, probe.bytes());
        if !packet_ram_matches(header, probe.bytes()) {
            release_context_address(context);
            return Err(ProbeBuildError::PacketRamMismatch);
        }
        let if_id = if_id.min(2);
        for word in 0..3 {
            let Some(value) = crate::vif::own_mac_word(if_id, word) else {
                release_context_address(context);
                return Err(ProbeBuildError::InvalidContextPointer);
            };
            ((header + 0x0a + word as u32 * 2) as *mut u16).write_volatile(value);
        }
        ((context_address + 0x5c) as *mut u16).write_volatile(probe.length as u16);
        ((context_address + 0xbf) as *mut u8).write_volatile(0x0f);
        ((context_address + 0xbd) as *mut u8).write_volatile(if_id);

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
        ((context_address + 0x0c) as *mut u8).write_volatile(rate);
        ((context_address + 0x63) as *mut u8).write_volatile(rate);
        ((context_address + 0x62) as *mut u8).write_volatile(0x0f);
        ((context_address + 0xab) as *mut u8).write_volatile(0xff);
        ((context_address + 0x64) as *mut u32).write_volatile(0);
        ((context_address + 0x54) as *mut u32).write_volatile(header);
        flags |= 0x1000;
        if ((header + 4) as *const u32).read_volatile() & 1 != 0 {
            flags |= 0x300;
        }
        ((context_address + 0x58) as *mut u32).write_volatile(flags);
        ((context_address + 0x80) as *mut u32).write_volatile(3);

        let frame_control = (header as *const u32).read_volatile() as u16;
        ((context_address + 0x5e) as *mut u16).write_volatile(frame_control);
        ((context_address + 0x44) as *mut u32).write_volatile(24);
        ((context_address + 0x48) as *mut u32)
            .write_volatile((probe.length as u32).saturating_sub(24));
        let duration_slot = (crate::dtcm::pas_stride_view_unchecked(usize::from(if_id))
            .slot_bits()
            .get() as *const u8)
            .read_volatile()
            & 1;
        ((context_address + 0xbe) as *mut u8).write_volatile(duration_slot);
        ((context_address + 0xaa) as *mut u8).write_volatile(0xff);
        ((context_address + 0xc8) as *mut u16).write_volatile(0);
        ((context_address + 0xca) as *mut u8).write_volatile(9);
        ((context_address + 0xd0) as *mut u16).write_volatile(0x10);
        let mut prepared = PreparedProbeContext {
            context,
            header,
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
        let free_head = WSM_TX_CONTEXT_FREE_HEAD as *mut u32;
        let mut destination = free_head.read_volatile();
        if destination == 0 || !is_wsm_tx_context(destination) {
            // Keep management/EAPOL startup byte-for-byte free of host-pool
            // retained-memory reconstruction. Initialize the pool only when
            // the first ordinary host frame actually needs class-0 ownership.
            crate::mac::initialize_wsm_tx_context_pool();
            destination = free_head.read_volatile();
        }
        if destination == 0 || !is_wsm_tx_context(destination) {
            release_context_address(source.context);
            return Err(ProbeBuildError::ContextPoolEmpty);
        }
        let destination_address = destination as usize;
        free_head.write_volatile(((destination_address + 4) as *const u32).read_volatile());
        if crate::vif::adjust_host_contexts_in_flight(1).is_err() {
            crate::halt_always!();
        }

        let destination_request = (destination_address as *const u32).read_volatile();
        let destination_frame_state = ((destination_address + 0xa0) as *const u32).read_volatile();
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
        (destination_address as *mut u32).write_volatile(destination_request);
        ((destination_address + 0x0f) as *mut u8).write_volatile(0);
        ((destination_address + 0x1c) as *mut u32).write_volatile(source.header);
        ((destination_address + 0x20) as *mut u32).write_volatile(0xfe);
        ((destination_address + 0x54) as *mut u32).write_volatile(source.header);
        ((destination_address + 0x70) as *mut u16).write_volatile(0xfe);
        ((destination_address + 0xa0) as *mut u32).write_volatile(destination_frame_state);
        ((destination_address + 0x53) as *mut u8).write_volatile(0);
        ((destination_address + 0xbf) as *mut u8).write_volatile(0);
        ((destination_address + 0x80) as *mut u32).write_volatile(3);

        Ok(PreparedProbeContext {
            context: destination,
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
        if is_wsm_tx_context(context.context) {
            release_wsm_context_address(context.context);
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
) -> SingleFramePipeDescriptor {
    unsafe {
        let address = context.context as usize;
        let rate = ((address + 0x63) as *const u8).read_volatile();
        let tx_flags = ((address + 0x58) as *const u32).read_volatile();
        let hardware_rate_code = ((address + 0x61) as *const u8).read_volatile();
        let legacy_mode = (0x0400_1685 as *const u8).read_volatile();
        let rate_attribute =
            (0x0400_0194_usize.wrapping_add(usize::from(rate)) as *const u8).read_volatile();
        let hardware_rate =
            (0x0400_01aa_usize.wrapping_add(usize::from(rate)) as *const u8).read_volatile();
        let phy = build_phy_rate_words(
            rate,
            legacy_mode,
            tx_flags,
            hardware_rate_code,
            rate_attribute,
        );
        let if_id = ((address + 0xbd) as *const u8).read_volatile();
        let metadata_address = packet_ram::interface_metadata_byte(usize::from(if_id)) as u32;
        let duration_slot = ((address + 0xbe) as *const u8).read_volatile();
        // `txp_submit_to_pipe` uses PAS `bVifSlot` (`ctx+0xbe`) here when
        // flags bit 0 is clear. Both internal and host contexts use this
        // selector; the TX rate indexes different PHY tables.
        let secondary_address = packet_ram::duration_word(usize::from(duration_slot)) as u32;
        build_single_frame_pipe_descriptor(SingleFramePipeInput {
            phy_rate_word: phy.rate,
            phy_control_word: finalize_phy_control(phy, rate, context.length),
            frame_length: context.length,
            hardware_rate,
            frame_control: ((address + 0x5e) as *const u16).read_volatile(),
            retry_flag: ((address + 0x58) as *const u32).read_volatile() & 0x10 != 0,
            metadata_address,
            duration: ((address + 0x8a) as *const u16).read_volatile(),
            header_address: context.header,
            secondary_command: 0x2100_0000 | (secondary_address & 0x007f_ffff),
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
        let address = context.context as usize;
        let rate = ((address + 0x63) as *const u8).read_volatile();
        let tx_flags = ((address + 0x58) as *const u32).read_volatile();
        let hardware_rate_code = ((address + 0x61) as *const u8).read_volatile();
        let legacy_mode = (0x0400_1685 as *const u8).read_volatile();
        let rate_attribute =
            (0x0400_0194_usize.wrapping_add(usize::from(rate)) as *const u8).read_volatile();
        let hardware_rate =
            (0x0400_01aa_usize.wrapping_add(usize::from(rate)) as *const u8).read_volatile();
        let phy = build_phy_rate_words(
            rate,
            legacy_mode,
            tx_flags,
            hardware_rate_code,
            rate_attribute,
        );
        let if_id = ((address + 0xbd) as *const u8).read_volatile();
        let duration_slot = ((address + 0xbe) as *const u8).read_volatile();
        let frame_control = u32::from(((address + 0x5e) as *const u16).read_volatile())
            | if ((address + 0x58) as *const u32).read_volatile() & 0x10 != 0 {
                0x0800
            } else {
                0
            };
        let mut checksum = 0_u32;
        let mut word_index = 0_u32;
        let mut mismatch = false;
        let mut add = |word: u32| {
            let pointer = (destination + word_index * 4) as *mut u32;
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
        add(0x2080_0000
            | (packet_ram::interface_metadata_byte(usize::from(if_id)) as u32 & 0x007f_ffff));
        add(0x3200_0000 | u32::from(((address + 0x8a) as *const u16).read_volatile()));
        add(0x2900_0000 | (context.header.wrapping_add(4) & 0x007f_ffff));
        add(single_frame_secondary_command(
            tx_flags,
            (context.header.wrapping_add(0x16) as *const u16).read_volatile(),
            duration_slot,
        ));
        if context.length > 24 {
            let payload = context.header.wrapping_add(24);
            add(0x4000_0000 | (0x007f_fffc & payload & 0xf6ff_ffff));
            add((u32::from(context.length - 24) & 0x0fff) << 12 | (payload & 3));
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
fn host_prepared_context(context: u32) -> Result<PreparedProbeContext, ProbeBuildError> {
    if !is_wsm_tx_context(context) {
        return Err(ProbeBuildError::InvalidContextPointer);
    }
    let address = context as usize;
    Ok(PreparedProbeContext {
        context,
        header: unsafe { ((address + 0x54) as *const u32).read_volatile() },
        length: unsafe { ((address + 0x5c) as *const u16).read_volatile() },
        rate: unsafe { ((address + 0x63) as *const u8).read_volatile() },
        expects_ack: unsafe { ((address + 0x58) as *const u32).read_volatile() & 0x300 == 0 },
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
    let address = context as usize;
    let destination = unsafe { ((address + 0xa0) as *const u32).read_volatile() };
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
        let previous = mask_irq_fiq_terminal();
        let pending = 0x0400_8ad8_usize;
        let old_head = read_u32(pending);
        let old_tail = read_u32(pending + 4);

        // `txq_list_insert(context, queue, 2)` prepends to the pending list.
        write_u32(context as usize + 4, old_head);
        if old_tail == 0 {
            write_u32(pending + 4, context);
        }
        write_u32(pending, context);
        write_u32(
            context as usize + 0x80,
            read_u32(context as usize + 0x80) | 0x20,
        );

        // The joined/active task accepts this frame, removes the same head,
        // and passes it through `tx_frame_done_release`.
        let next = read_u32(context as usize + 4);
        write_u32(pending, next);
        if read_u32(pending + 4) == context {
            write_u32(pending + 4, if next == 0 { 0 } else { old_tail });
        }
        write_u32(context as usize + 4, 0);
        write_u32(
            context as usize + 0x80,
            read_u32(context as usize + 0x80) | 0x40,
        );

        // `pas_txq_push_global(context + 0x54)` appends class-0 host frames.
        let ring = 0x0400_1578_usize;
        let head = read_u32(ring) as u8 & 0x3f;
        let tail = read_u32(ring + 4) as u8 & 0x3f;
        let following = tail.wrapping_add(1) & 0x3f;
        if head != tail || following == head {
            restore_irq_fiq(previous);
            return Err(ProbeBuildError::PipeSlotBusy);
        }
        let frame_node = context + FRAME_NODE_OFFSET;
        write_u32(ring + 8 + usize::from(tail) * 4, frame_node);
        write_u32(ring + 4, u32::from(following));

        // The minimum scheduler diagnostic consumes exactly the frame it just
        // enqueued, preserving an otherwise-empty ring for the direct backend.
        let selected = read_u32(ring + 8 + usize::from(head) * 4);
        if selected != frame_node {
            restore_irq_fiq(previous);
            return Err(ProbeBuildError::UnsupportedPublicationShape);
        }
        write_u32(ring + 8 + usize::from(head) * 4, 0);
        write_u32(ring, u32::from(head.wrapping_add(1) & 0x3f));
        // The non-aggregate scheduler branch marks the selected frame before
        // `txp_build_pipe_descriptor(..., 0)`.
        write_u32(
            context as usize + 0x58,
            read_u32(context as usize + 0x58) | 0x0400_0000,
        );
        restore_irq_fiq(previous);
    }
    Ok(())
}

unsafe fn prepare_context_publication(
    context: PreparedProbeContext,
) -> Result<PreparedProbePublication, ProbeBuildError> {
    unsafe {
        let address = context.context as usize;
        let queue = ((address + 0x60) as *const u8).read_volatile();
        let pipe =
            (0x0400_02e0_usize.wrapping_add(usize::from(queue)) as *const u8).read_volatile();
        if pipe >= 4 {
            release_unpublished_probe_context(context);
            return Err(ProbeBuildError::PipeStateUnavailable);
        }
        let pipe_record = 0x0400_1720_usize + usize::from(pipe) * 0x6c;
        let hardware = ((pipe_record + 8) as *const u32).read_volatile();
        let slot = (pipe_record as *const u8).read_volatile() & 3;
        let slot_record = pipe_record + 0x0c + usize::from(slot) * 0x18;
        let command = ((slot_record + 0x14) as *const u32).read_volatile();
        if hardware == 0 || command == 0 {
            release_unpublished_probe_context(context);
            return Err(ProbeBuildError::PipeStateUnavailable);
        }
        // Startup imports persistent slot tails from vendor state. They are not
        // zero-valued idle sentinels, so detached reservation must preserve and
        // restore them rather than infer ownership from their contents.
        let original_slot_header = (slot_record as *const u32).read_volatile();
        let original_slot_frame = ((slot_record + 0x0c) as *const u32).read_volatile();
        let mut original_command = [0_u32; 16];
        for (index, word) in original_command.iter_mut().enumerate() {
            *word = ((command as usize + index * 4) as *const u32).read_volatile();
        }
        ((address + 0x80) as *mut u32)
            .write_volatile(((address + 0x80) as *const u32).read_volatile() | 0x100);
        // `0xa712` receives the frame node at `context+0x54`, not the context
        // base. Its kind-0 branch stores frame-node `+0x56` (context `+0xaa`)
        // in `slot+1`; success handler `0x9cdc` requires this marker to be
        // `0xff` before entering the release loop.
        (slot_record as *mut u8).write_volatile(0);
        ((slot_record + 1) as *mut u8)
            .write_volatile(((address + 0xaa) as *const u8).read_volatile());
        ((slot_record + 2) as *mut u8).write_volatile(0);
        ((slot_record + 3) as *mut u8).write_volatile(0);
        ((slot_record + 0x0c) as *mut u32).write_volatile(context.context + 0x54);
        (command as *mut u32).write_volatile(0);
        ((command + 4) as *mut u32).write_volatile(0);
        ((command + 8) as *mut u32).write_volatile(0xdc00_0000);
        let checksum = match emit_prepared_probe_descriptor(&context, command + 0x0c) {
            Ok(checksum) => checksum,
            Err(error) => {
                ((slot_record + 0x0c) as *mut u32).write_volatile(original_slot_frame);
                (slot_record as *mut u32).write_volatile(original_slot_header);
                for (index, word) in original_command.into_iter().enumerate() {
                    ((command as usize + index * 4) as *mut u32).write_volatile(word);
                }
                release_unpublished_probe_context(context);
                return Err(error);
            }
        };
        Ok(PreparedProbePublication {
            context,
            pipe,
            slot,
            slot_record: slot_record as u32,
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
    let address = context.context as usize;
    unsafe { copy_to_packet_ram(context.header, request.frame) };
    if !unsafe { packet_ram_matches(context.header, request.frame) } {
        unsafe { release_context_address(context.context) };
        return Err(ProbeBuildError::PacketRamMismatch);
    }
    let queue = request.queue_id.min(3);
    unsafe {
        let ac = (0x0400_02dc_usize.wrapping_add(usize::from(queue)) as *const u8).read_volatile();
        ((address + 0x60) as *mut u8).write_volatile(ac);
        ((address + 0x61) as *mut u8).write_volatile((request.flags & 0x0f) >> 1);
        ((address + 0x62) as *mut u8).write_volatile((request.flags & 0x7f) >> 4);
        ((address + 0xbf) as *mut u8).write_volatile(1);
    }
    if request.max_tx_rate < 22 {
        context.rate = request.max_tx_rate;
        unsafe {
            ((address + 0x0c) as *mut u8).write_volatile(context.rate);
            ((address + 0x63) as *mut u8).write_volatile(context.rate);
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
        ((address + 0x58) as *mut u32).write_volatile(tx_flags);
        ((address + 0x64) as *mut u32).write_volatile(request.expire_time);
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
    let address = context.context as usize;
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
    let address_mode = frame_control & 0x0300;
    let mut header_length = if address_mode == 0x0300 { 30 } else { 24 };
    let qos_data = !legacy_eapol && frame_control & 0x008f == 0x0088;
    if qos_data {
        header_length += if frame_control & 0x8000 != 0 { 6 } else { 2 };
    }
    let queue = host_queue;
    unsafe {
        if !legacy_eapol {
            ((address + 0x44) as *mut u32).write_volatile(header_length);
            ((address + 0x48) as *mut u32)
                .write_volatile((scratch.length as u32).saturating_sub(header_length));
            // RustCrypto has already filled the host-reserved CCMP header and
            // MIC space. Mark ordinary data as having no pending hardware
            // crypto work. The validated EAPOL compatibility path preserves
            // the original class-6 context status unchanged.
            if frame_control & 0x400c == 0x4008 {
                ((address + 0x70) as *mut u16).write_volatile(0x0010);
            }
        }
        // `ctx+0x60` is the vendor AC selected through the four-entry WSM
        // queue map, not the raw WSM queue ID. The adjacent bytes retain the
        // PTA priority and retry-policy selector packed in WSM TX flags.
        let ac = (0x0400_02dc_usize.wrapping_add(usize::from(queue)) as *const u8).read_volatile();
        ((address + 0x60) as *mut u8).write_volatile(ac);
        ((address + 0x61) as *mut u8).write_volatile((request.flags & 0x0f) >> 1);
        ((address + 0x62) as *mut u8).write_volatile((request.flags & 0x7f) >> 4);
        // This direct cooperative publisher still uses an internal class-6
        // context rather than the vendor WSM class-0 pool. Link slot 1 is the
        // validated internal slot for that temporary path.
        ((address + 0xbf) as *mut u8).write_volatile(1);
    }
    if request.max_tx_rate < 22 {
        context.rate = request.max_tx_rate;
        unsafe {
            ((address + 0x0c) as *mut u8).write_volatile(context.rate);
            ((address + 0x63) as *mut u8).write_volatile(context.rate);
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
        ((address + 0x58) as *mut u32).write_volatile(tx_flags);
        ((address + 0x64) as *mut u32).write_volatile(request.expire_time);
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
        let state = 0x0400_1d20_usize;
        write_u8(state + 0x10, 7);
        write_u8(state + 0x21, 0);
        write_u8(state + 0x18, 1);
        let (timer, duration) = phy_operation_7_timer();
        write_u32(state + 0x1c, duration);
        write_u32(state + 0x0c, u32::from(read_u8(state + 0x18)));
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
            ((context + 0x1c) as *mut u32).write_volatile((buffer + 0x40) as u32);
            ((context + 0xc4) as *mut u32).write_volatile((buffer + 0x20) as u32);
            ((context + 0x70) as *mut u16).write_volatile(0x00ff);
            ((context + 0x04) as *mut u32).write_volatile(previous);
            previous = context as u32;
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

#[cfg(test)]
mod tests {
    use super::*;

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
        completed: Option<(FrameNodeAddress, u32, u16)>,
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

        fn complete_give_up(&mut self, frame_node: FrameNodeAddress, slot: u32, status: u16) {
            self.completed = Some((frame_node, slot, status));
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
        mmio.set(CURRENT_PIPE, 2);
        mmio.set(pipe_state_address(2) + 2, 1);
        let current_slot = current_slot_address(&mut mmio, 2);
        mmio.set(current_slot + 0x0c, 0x1234);
        mmio.set(current_slot + 0x14, 0x5678);
        let mut policy = MockTxPolicy::new();

        execute_mac_pipe_service(&mut mmio, SchedulerWord::new(0x0f), &mut policy);

        assert_eq!(mmio.writes[0], (CURRENT_PIPE_RECORD, pipe_state_address(2)));
        assert_eq!(
            mmio.writes[1],
            (CURRENT_SLOT, pipe_state_address(2) + 0x0c + 0x18)
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
        mmio.set(CURRENT_PIPE, 0);
        let selected = 3;
        let selected_state = pipe_state_address(selected);
        mmio.set(selected_state + 2, 2);
        mmio.set(PIPE_QUANTUM_POINTERS + 12, 0x1234);
        let mut policy = MockTxPolicy::new();

        execute_mac_pipe_service(&mut mmio, SchedulerWord::new(0x0080), &mut policy);

        assert_eq!(mmio.writes[2], (0x1234, PIPE_QUANTUM));
        assert_eq!(mmio.writes[3], (PIPE_IRQ_TRIGGER, 1 << 28));
        assert_eq!(mmio.writes[4], (CURRENT_PIPE_RECORD, selected_state));
        assert_eq!(
            mmio.writes[5],
            (CURRENT_SLOT, selected_state + 0x0c + 2 * 0x18)
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
        mmio.set(pipe_state + 8, 0x9000);
        // Pending-slot mask and hardware-owned high bits must survive.
        mmio.set(0x9020, 0x8000_000f | (1 << 24) | (1 << 27));

        assert!(advance_pipe_slot(&mut mmio, 2));

        assert_eq!(mmio.get(pipe_state + 3), 0);
        assert_eq!(mmio.get(pipe_state + 6), 8);
        assert_eq!(mmio.get(0x9018), PIPE_RETRY_INACTIVE_SENTINEL);
        assert_eq!(mmio.get(PIPE_IRQ_PENDING), 0_u32.wrapping_sub(0x4450));
        // cursor = (last + 1) & 3 = 3, written to bits 26:24 and 29:27.
        assert_eq!(mmio.get(0x9020), 0x8000_000f | (3 << 24) | (3 << 27));
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
        mmio.set(pipe_state + 8, 0x9000);

        assert!(!advance_pipe_slot(&mut mmio, 0));

        // A saturated abort counter is left alone, and the idle branch mirrors
        // the producer rather than `last + 1`.
        assert_eq!(mmio.get(pipe_state + 6), 0xff);
        assert_eq!(mmio.get(0x9020), (2 << 24) | (2 << 27));
    }

    #[test]
    fn resync_pipe_cursor_matches_vendor_invariant_and_keeps_mask() {
        let mut mmio = MockPipeMmio::new();
        let pipe_state = pipe_state_address(1);
        mmio.set(pipe_state, 3);
        mmio.set(pipe_state + 8, 0x9080);
        mmio.set(0x90a0, 0xc000_00aa | (1 << 24) | (1 << 27));

        resync_pipe_cursor(&mut mmio, 1);

        let word = mmio.get(0x90a0);
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
        mmio.set(pipe_state + 8, 0x9000);
        mmio.set(0x9020, (1 << 24) | (1 << 27));

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
        mmio.set(CURRENT_PIPE, 1);
        mmio.set(pipe_state_address(1) + 2, 0);
        let slot = current_slot_address(&mut mmio, 1);
        mmio.set(slot + 3, 3);
        mmio.set(pipe_state_address(1) + 8, 0x9000);
        mmio.set(pipe_state_address(3) + 8, 0xa000);
        let mut backend = MockRetryBackend::new(SingleTxRetryDecision::GiveUp);

        let outcome = execute_single_outstanding_tx_retry(
            &mut mmio,
            SchedulerWord::new(0x0a00),
            &mut backend,
        );

        assert_eq!(outcome, SingleTxRetryOutcome::InactivePipeAcknowledged);
        assert_eq!(mmio.writes[0], (CURRENT_PIPE_RECORD, pipe_state_address(1)));
        assert_eq!(mmio.writes[1], (CURRENT_SLOT, slot));
        assert_eq!(mmio.get(slot + 3), 4);
        assert_eq!(mmio.get(PIPE_BUSY), 1);
        assert_eq!(mmio.writes[2], (0x9018, PIPE_RETRY_INACTIVE_SENTINEL));
        assert_eq!(mmio.writes[3], (0xa018, PIPE_RETRY_INACTIVE_SENTINEL));
        assert_eq!(mmio.writes[4], (PIPE_IRQ_PENDING, 0xffff_fdff));
        assert_eq!(backend.decided, None);
    }

    #[test]
    fn single_retry_give_up_completes_before_command_and_ack() {
        let mut mmio = MockPipeMmio::new();
        mmio.set(CURRENT_PIPE, 0);
        let pipe_state = pipe_state_address(0);
        mmio.set(pipe_state + 1, 0);
        mmio.set(pipe_state + 2, 0);
        mmio.set(pipe_state + 3, 1);
        mmio.set(pipe_state + 5, 0);
        mmio.set(pipe_state + 8, 0x9000);
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
            Some((FrameNodeAddress::new(0x0400_90d8), slot, 0x0b))
        );
        assert_eq!(mmio.get(PIPE_IRQ_TRIGGER), 1 << 25);
        assert_eq!(mmio.get(0x901c), 1);
        assert_eq!(mmio.get(pipe_state), 1);
        assert_eq!(mmio.get(pipe_state + 2), 1);
        assert_eq!(mmio.get(pipe_state + 3), 0);
        assert_eq!(mmio.get(pipe_state + 4), 0);
        assert_eq!(mmio.get(pipe_state + 5), 5);
        assert_eq!(mmio.get(PIPE_IRQ_PENDING), 0xffff_fef0);
    }

    #[test]
    fn single_retry_rearm_retains_slot_ownership_for_backend() {
        let mut mmio = MockPipeMmio::new();
        mmio.set(CURRENT_PIPE, 2);
        let pipe_state = pipe_state_address(2);
        mmio.set(pipe_state + 2, 1);
        mmio.set(pipe_state + 3, 1);
        mmio.set(pipe_state + 5, 1);
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
        assert_eq!(mmio.get(PIPE_BUSY), 1);
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
        mmio.set(pipe_state + 8, 0x9000);
        mmio.set(0x9020, 0x0f);
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
        mmio.set(PIPE_RETRY_RANDOM_STATE, 0x0012_3456);
        mmio.set(
            crate::dtcm::pas_stride_view(0)
                .unwrap()
                .contention_window(0)
                .unwrap()
                .get() as u32,
            0x001f,
        );
        mmio.set(PIPE_RETRY_RATE_MAP + 2, 3);
        mmio.set(PIPE_RETRY_TIMING_TABLE + 6, 0x20);
        mmio.set(PIPE_RECORDS + 0x1c, 2);
        mmio.set(PIPE_RECORDS + 0x20, 3);
        mmio.set(PIPE_RETRY_HARDWARE_STATE, 0);
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
            .position(|(address, _)| *address == 0x9020)
            .unwrap_or_else(|| panic!("missing retry command-mask write"));
        assert!(descriptor_index < trigger_index);
        assert!(trigger_index < command_mask_index);
        assert_eq!(mmio.get(0xa000), 0);
        assert_eq!(mmio.get(PIPE_RETRY_RANDOM_STATE), 0xb013_1713);
        assert_eq!(mmio.get(PIPE_RETRY_RANDOM_STATS), 0x13);
        assert_eq!(mmio.get(PIPE_RETRY_RANDOM_STATS + 12), 1);
        assert_eq!(mmio.get(frame.raw() + 0x5a), 0x13);
        assert_eq!(mmio.get(0xa004), 0x4d7f);
        assert_eq!(mmio.get(0xa008), 0xd800_2138);
        assert_eq!(mmio.get(0x9020), 0x0e);
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
        mmio.set(pipe_state + 8, 0x9000);
        mmio.set(0x9020, 0x0f);
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
        mmio.set(PIPE_RETRY_RANDOM_STATE, 0x0012_3456);
        mmio.set(
            crate::dtcm::pas_stride_view(0)
                .unwrap()
                .contention_window(0)
                .unwrap()
                .get() as u32,
            0x001f,
        );
        mmio.set(PIPE_RETRY_RATE_MAP + 2, 3);
        mmio.set(PIPE_RETRY_TIMING_TABLE + 6, 0x20);
        mmio.set(PIPE_RECORDS + 0x1c, 2);
        mmio.set(PIPE_RECORDS + 0x20, 3);
        mmio.set(PIPE_RETRY_HARDWARE_STATE, 0);
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
        assert_eq!(mmio.get(0x9020), 0x0d);
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
        mmio.set(pipe_state + 8, 0x9000);
        mmio.set(0x9020, 1);
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
        mmio.set(PIPE_RETRY_RANDOM_STATE, 1);
        mmio.set(
            crate::dtcm::pas_stride_view(0)
                .unwrap()
                .contention_window(0)
                .unwrap()
                .get() as u32,
            0,
        );
        mmio.set(PIPE_RETRY_RATE_MAP + 2, 0);
        mmio.set(PIPE_RETRY_TIMING_TABLE, 0);
        mmio.set(PIPE_RETRY_HARDWARE_STATE, 0);
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
        mmio.set(pipe_state + 8, 0x9000);
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
        mmio.set(PIPE_RETRY_RANDOM_STATE, 1);
        mmio.set(
            crate::dtcm::pas_stride_view(0)
                .unwrap()
                .contention_window(0)
                .unwrap()
                .get() as u32,
            0,
        );
        mmio.set(PIPE_RETRY_RATE_MAP + 1, 0);
        mmio.set(PIPE_RETRY_TIMING_TABLE, 0);
        mmio.set(PIPE_RETRY_HARDWARE_STATE, 2);
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
        assert_eq!(mmio.get(0x9018), 0xff00_ffff);
        assert_eq!(mmio.get(PIPE_IRQ_PENDING), 0xffff_0df0);
    }

    #[test]
    fn single_probe_publication_preserves_vendor_trigger_and_go_order() {
        let mut mmio = MockPipeMmio::new();
        let pipe_state = pipe_state_address(0);
        let slot = pipe_state + 0x0c;
        let hardware_ring = 0x8000;
        let command = 0x9000;
        let frame = FrameNodeAddress::new(0x0400_90d8);
        mmio.set(frame.raw() + 0x2c, 3);
        mmio.set(frame.raw() + 0x3a, 0x20);
        mmio.set(frame.raw() + 0x56, 0xff);
        mmio.set(frame.raw() + 0x69, 0);
        mmio.set(frame.raw() + 0x0c, 0);
        mmio.set(0x0ac0_0004, 0x1234);
        mmio.set(PIPE_RETRY_RANDOM_STATE, 1);
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
        mmio.set(0x0400_1b04, 0x44);
        mmio.set(0x0400_02dc, 1);
        mmio.set(
            crate::dtcm::pas_stride_view(0)
                .unwrap()
                .txop_limit(1)
                .unwrap()
                .get() as u32,
            64,
        );
        mmio.set(
            PIPE_QUANTUM_POINTERS,
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
        mmio.set(CURRENT_PIPE, 2);
        mmio.set(CURRENT_PIPE_RECORD, 0x0400_17f8);
        mmio.set(CURRENT_SLOT, 0x0400_181c);
        mmio.set(PIPE_BUSY, 1);
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
        mmio.set(MAC_BEACON_CONFIG + 0x12, 0x20);
        mmio.set(MAC_BEACON_STATE + 0x2c, 0x1234);
        let mut raised = 0;

        execute_mac_beacon_event(&mut mmio, |bits| raised = bits);

        assert_eq!(mmio.get(MAC_BEACON_STATE + 0x28), 5);
        assert_eq!(mmio.get(PIPE_RECORDS + 8), 0x2000);
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
        assert_eq!(phy_operation_7_timer(), (0x0400_1d18, 0x0098_9680));
        assert_eq!(phy_dispatch_switch_target(2), 0x0001_6f92);
        assert_eq!(phy_dispatch_switch_target(3), 0x0001_6fa0);
        assert_eq!(phy_dispatch_switch_target(7), 0x0001_6fb6);
    }

    #[test]
    fn native_internal_context_pool_preserves_context_stride() {
        assert_eq!(crate::dtcm::INTERNAL_TX_CONTEXT_SIZE, TX_CONTEXT_SIZE);
        assert_eq!(crate::dtcm::INTERNAL_TX_CONTEXT_COUNT, TX_CONTEXT_COUNT);
        assert_eq!(crate::dtcm::INTERNAL_CONTEXT_POOL.get(), 0x0400_9080);
        assert_eq!(internal_context_address(0), internal_context_base());
        assert_eq!(
            internal_context_address(2) - internal_context_base(),
            2 * TX_CONTEXT_SIZE
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
        assert_eq!(CLASS0_INTERNAL_CONTEXTS, 0x0400_8f71);
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
        let context = ContextAddress::new(0x0400_9084);
        assert_eq!(context.frame_node().raw(), 0x0400_90d8);
        assert_eq!(context.frame_node().context(), context);
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
    fn single_frame_descriptor_matches_vendor_command_shape() {
        let descriptor = build_single_frame_pipe_descriptor(SingleFramePipeInput {
            phy_rate_word: 0x123456,
            phy_control_word: 0xabcdef,
            frame_length: 42,
            hardware_rate: 3,
            frame_control: 0x0040,
            retry_flag: true,
            metadata_address: 0x0901_2345,
            duration: 0x0064,
            header_address: 0x0901_4fe8,
            secondary_command: 0x2100_1234,
            address_mask: u32::MAX,
            terminal_command: 0x4e14_0000,
        });
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
