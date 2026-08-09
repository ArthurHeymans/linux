//! Internal management-frame TX preparation.
//!
//! This module intentionally stops before hardware publication. The vendor
//! queue/pipe ownership and IRQ completion path must be translated before a
//! prepared frame can become DMA-owned.

use core::cell::UnsafeCell;

use crate::configuration::MAX_TEMPLATE_FRAME_LEN;

const TX_CONTEXT_BASE: usize = 0x0400_9084;
const TX_CONTEXT_SIZE: usize = 0x170;
const TX_CONTEXT_COUNT: usize = 3;
const TX_BUFFER_BASE: usize = 0x0901_4fa8;
const TX_BUFFER_SIZE: usize = 0x400;

struct ProbeChecksum(UnsafeCell<u32>);

unsafe impl Sync for ProbeChecksum {}

static PROBE_CHECKSUM: ProbeChecksum = ProbeChecksum(UnsafeCell::new(0));

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
        let address = context as usize;
        ((address + 0x70) as *mut u16).write_volatile(0x00ff);
        let flags = (address + 0x80) as *mut u32;
        flags.write_volatile(flags.read_volatile() | 0x0002_0000);
        let free_head = 0x0400_9080 as *mut u32;
        let old_head = free_head.read_volatile();
        ((address + 4) as *mut u32).write_volatile(old_head);
        free_head.write_volatile(context);
        let allocated = 0x0400_8f70 as *mut u8;
        allocated.write_volatile(allocated.read_volatile().wrapping_sub(1));
    }
}

fn expected_header_address(context: u32) -> Option<u32> {
    let offset = usize::try_from(context)
        .ok()?
        .checked_sub(TX_CONTEXT_BASE)?;
    if offset % TX_CONTEXT_SIZE != 0 {
        return None;
    }
    let index = offset / TX_CONTEXT_SIZE;
    (index < TX_CONTEXT_COUNT).then_some((TX_BUFFER_BASE + index * TX_BUFFER_SIZE + 0x40) as u32)
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
    DescriptorReadbackMismatch,
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
    queue_bits: u8,
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
        rate: class | u32::from(rate_attribute & 0x0f) | (u32::from(queue_bits & 7) << 16),
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

    pub fn is_pipe_start(self) -> bool {
        self.pipe_marker && self.phase == 2 && self.event_type == 0x37
    }

    pub fn is_pipe_success(self) -> bool {
        self.pipe_marker && self.phase == 3 && self.event_type == 0x37
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
    Completed {
        context: u32,
        pipe: u8,
        slot: u8,
        status: u8,
    },
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
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MacEventServiceReport {
    pub drained: u32,
    pub handled: u32,
    pub unhandled: u32,
    pub blocked: Option<u32>,
}

impl ProbeTxTracker {
    pub const fn new() -> Self {
        Self {
            ownership: ProbeTxOwnership::Idle,
            latched_pipe: None,
        }
    }

    pub fn ownership(&self) -> ProbeTxOwnership {
        self.ownership
    }

    pub fn owned_pipe(&self) -> Option<u8> {
        match self.ownership {
            ProbeTxOwnership::PipeOwned { pipe, .. }
            | ProbeTxOwnership::Started { pipe, .. }
            | ProbeTxOwnership::RetryRequired { pipe, .. }
            | ProbeTxOwnership::Completed { pipe, .. } => Some(pipe),
            ProbeTxOwnership::Idle | ProbeTxOwnership::Prepared { .. } => None,
        }
    }

    pub fn prepare(&mut self, context: u32) -> Result<(), ProbeTxTransitionError> {
        if self.ownership != ProbeTxOwnership::Idle {
            return Err(ProbeTxTransitionError::Busy);
        }
        self.latched_pipe = None;
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
        pipe_pending: bool,
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
            self.latched_pipe = Some(event.pipe);
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
            self.ownership = ProbeTxOwnership::Completed {
                context,
                pipe,
                slot,
                status: 0,
            };
            return Ok(true);
        }
        if event.completion_marker {
            if self.latched_pipe != Some(pipe) {
                return Err(ProbeTxTransitionError::WrongPipe);
            }
            self.ownership = if pipe_pending && matches!(event.status, 4 | 0x19) {
                ProbeTxOwnership::RetryRequired {
                    context,
                    pipe,
                    slot,
                    status: event.status,
                }
            } else {
                // Vendor dispatches non-pending status events through
                // `txp_pipe_tx_status`, which completes and releases the slot.
                ProbeTxOwnership::Completed {
                    context,
                    pipe,
                    slot,
                    status: event.status,
                }
            };
            return Ok(true);
        }
        Ok(false)
    }

    pub fn reclaim_completed(&mut self) -> Option<u32> {
        let ProbeTxOwnership::Completed { context, .. } = self.ownership else {
            return None;
        };
        self.latched_pipe = None;
        self.ownership = ProbeTxOwnership::Idle;
        Some(context)
    }
}

impl Default for ProbeTxTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Cooperatively drains the MAC event FIFO using the same primary/status read
/// sequence as vendor FIQ handler `0x9eb4`. It is intentionally not called by
/// the runtime until all events needed by a published probe can be acknowledged.
///
/// # Safety
/// Reading the FIFO consumes hardware events. The caller must own MAC event
/// servicing while FIQ remains masked.
pub unsafe fn service_probe_mac_events(
    tracker: &mut ProbeTxTracker,
    max_events: u32,
) -> Result<MacEventServiceReport, ProbeTxTransitionError> {
    let mut report = MacEventServiceReport::default();
    while report.drained < max_events {
        // `+0x24` exposes the next event without consuming it. Refuse to pop a
        // word whose non-pipe side effects are not translated yet.
        let peek = unsafe { (0x09c0_0a24 as *const u32).read_volatile() };
        let Some(peeked) = MacEvent::decode(peek) else {
            break;
        };
        if peeked.fatal_marker
            || peeked.pipe_service_marker
            || peeked.beacon_marker
            || peeked.sideband_marker
        {
            report.blocked = Some(peek);
            break;
        }

        let raw = unsafe { (0x09c0_0a20 as *const u32).read_volatile() };
        let Some(event) = MacEvent::decode(raw) else {
            break;
        };
        report.drained = report.drained.wrapping_add(1);
        let owned_pipe = tracker.owned_pipe();
        let belongs_to_probe = (event.is_pipe_start() || event.is_pipe_success())
            && owned_pipe == Some(event.pipe)
            || event.completion_marker && owned_pipe.is_some();
        if belongs_to_probe {
            let pipe = owned_pipe.unwrap_or(event.pipe);
            let pending =
                unsafe { (0x09c0_0e84 as *const u32).read_volatile() } & (0x100_u32 << pipe) != 0;
            if tracker.handle_pipe_event(event, pending)? {
                report.handled = report.handled.wrapping_add(1);
            } else {
                report.unhandled = report.unhandled.wrapping_add(1);
            }
        } else {
            report.unhandled = report.unhandled.wrapping_add(1);
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
        let free_head = 0x0400_9080 as *mut u32;
        let context = free_head.read_volatile();
        if context == 0 {
            return Err(ProbeBuildError::ContextPoolEmpty);
        }
        let context_address = context as usize;
        free_head.write_volatile(((context_address + 4) as *const u32).read_volatile());

        let global = 0x0400_8f6c_usize;
        let allocated = (global + 4) as *mut u8;
        allocated.write_volatile(allocated.read_volatile().wrapping_add(1));
        ((context_address + 0x0d) as *mut u8).write_volatile(0);
        ((context_address + 0x4c) as *mut u32).write_volatile(0);
        ((context_address + 0x53) as *mut u8).write_volatile(6);
        ((context_address + 0x52) as *mut u8).write_volatile(1);
        let sequence = ((global + 8) as *const u16).read_volatile();
        ((context_address + 0x50) as *mut u16).write_volatile(sequence);
        ((global + 8) as *mut u16).write_volatile(sequence.wrapping_add(1));
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
        ((header + 0x0a) as *mut u16).write_volatile(
            ((0x0400_3ecc + usize::from(if_id) * 0x3b0) as *const u16).read_volatile(),
        );
        ((header + 0x0c) as *mut u16).write_volatile(
            ((0x0400_3ece + usize::from(if_id) * 0x3b0) as *const u16).read_volatile(),
        );
        ((header + 0x0e) as *mut u16).write_volatile(
            ((0x0400_3ed0 + usize::from(if_id) * 0x3b0) as *const u16).read_volatile(),
        );
        ((context_address + 0x5c) as *mut u16).write_volatile(probe.length as u16);
        ((context_address + 0xbf) as *mut u8).write_volatile(0x0f);
        ((context_address + 0xbd) as *mut u8).write_volatile(if_id);

        // Ordinary foreground scans store explicit rate 0xff, after which
        // `lmc_tx_assign_default_rate` resolves the VIF default.
        let vif = 0x0400_3e98_usize + usize::from(if_id) * 0x3b0;
        let mut rate = if ((vif + 0x26) as *const u8).read_volatile() == 0 {
            ((vif + 0x12c) as *const u8).read_volatile()
        } else {
            ((vif + 0x12d) as *const u8).read_volatile()
        };
        if rate == 0xff || ((vif + 0x27) as *const u8).read_volatile() < 3 {
            rate = ((vif + 0x24) as *const u8).read_volatile();
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
        let vif_slot = ((0x0400_3ae9 + usize::from(if_id) * 0x98) as *const u8).read_volatile() & 1;
        ((context_address + 0xbe) as *mut u8).write_volatile(vif_slot);
        ((context_address + 0xaa) as *mut u8).write_volatile(0xff);
        ((context_address + 0xc8) as *mut u16).write_volatile(0);
        ((context_address + 0xca) as *mut u8).write_volatile(9);
        ((context_address + 0xd0) as *mut u16).write_volatile(0x10);
        // Broadcast probe requests do not expect an ACK. The general PAS
        // timing path may replace this once queue scheduling is translated.
        ((context_address + 0x8a) as *mut u16).write_volatile(0);

        Ok(PreparedProbeContext {
            context,
            header,
            length: probe.length as u16,
            rate,
        })
    }
}

/// Returns a context that has never been queued or published.
///
/// # Safety
/// `context` must still be software-owned and must not have entered a queue.
pub unsafe fn release_unpublished_probe_context(context: PreparedProbeContext) {
    unsafe { release_context_address(context.context) };
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
        let queue_bits = ((address + 0x61) as *const u8).read_volatile();
        let legacy_mode = (0x0400_1685 as *const u8).read_volatile();
        let rate_attribute =
            (0x0400_0194_usize.wrapping_add(usize::from(rate)) as *const u8).read_volatile();
        let hardware_rate =
            (0x0400_01aa_usize.wrapping_add(usize::from(rate)) as *const u8).read_volatile();
        let phy = build_phy_rate_words(rate, legacy_mode, tx_flags, queue_bits, rate_attribute);
        let if_id = ((address + 0xbd) as *const u8).read_volatile();
        let vif_slot = ((address + 0xbe) as *const u8).read_volatile();
        let metadata_address = 0x0900_8008_u32.wrapping_add(u32::from(if_id));
        let secondary_address = 0x0900_7bc0_u32.wrapping_add(u32::from(vif_slot) * 2);
        build_single_frame_pipe_descriptor(SingleFramePipeInput {
            phy_rate_word: phy.rate,
            phy_control_word: phy.control,
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
        let queue_bits = ((address + 0x61) as *const u8).read_volatile();
        let legacy_mode = (0x0400_1685 as *const u8).read_volatile();
        let rate_attribute =
            (0x0400_0194_usize.wrapping_add(usize::from(rate)) as *const u8).read_volatile();
        let hardware_rate =
            (0x0400_01aa_usize.wrapping_add(usize::from(rate)) as *const u8).read_volatile();
        let phy = build_phy_rate_words(rate, legacy_mode, tx_flags, queue_bits, rate_attribute);
        let if_id = ((address + 0xbd) as *const u8).read_volatile();
        let vif_slot = ((address + 0xbe) as *const u8).read_volatile();
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
        add(0x5000_0000 | (phy.control & 0x00ff_ffff));
        add(0x5200_0000 | (u32::from(hardware_rate) << 16) | u32::from(context.length + 4));
        add(0x3100_0000 + frame_control);
        add(0x4700_0000 + (frame_control >> 8));
        add(0x2080_0000 | (0x0900_8008_u32.wrapping_add(u32::from(if_id)) & 0x007f_ffff));
        add(0x3200_0000 | u32::from(((address + 0x8a) as *const u16).read_volatile()));
        add(0x2900_0000 | (context.header.wrapping_add(4) & 0x007f_ffff));
        add(0x2100_0000 | (0x0900_7bc0_u32.wrapping_add(u32::from(vif_slot) * 2) & 0x007f_ffff));
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

/// Exercises complete detached preparation, stores a command-list checksum,
/// and releases the context without queueing or publishing it.
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
    let probe = unsafe {
        prepare_probe_into(
            &mut *PREPARED_PROBE_SCRATCH.0.get(),
            template,
            ssid,
            channel,
        )?
    };
    let context = unsafe { prepare_probe_context(probe, if_id) }?;
    let checksum = unsafe {
        let address = context.context as usize;
        let queue = ((address + 0x60) as *const u8).read_volatile();
        let pipe =
            (0x0400_02e0_usize.wrapping_add(usize::from(queue)) as *const u8).read_volatile();
        if pipe >= 4 {
            release_unpublished_probe_context(context);
            return Err(ProbeBuildError::PipeStateUnavailable);
        }
        let pipe_record = 0x0400_1720_usize + usize::from(pipe) * 0x6c;
        let hardware = (pipe_record + 8) as *const u32;
        let slot = (pipe_record as *const u8).read_volatile() & 3;
        let slot_record = pipe_record + 0x0c + usize::from(slot) * 0x18;
        let command = ((slot_record + 0x14) as *const u32).read_volatile();
        if hardware.read_volatile() == 0 || command == 0 {
            release_unpublished_probe_context(context);
            return Err(ProbeBuildError::PipeStateUnavailable);
        }

        ((address + 0x80) as *mut u32)
            .write_volatile(((address + 0x80) as *const u32).read_volatile() | 0x100);
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
                ((slot_record + 0x0c) as *mut u32).write_volatile(0);
                release_unpublished_probe_context(context);
                return Err(error);
            }
        };
        ((slot_record + 0x0c) as *mut u32).write_volatile(0);
        checksum
    };
    unsafe {
        (&raw mut *PROBE_CHECKSUM.0.get()).write_volatile(checksum);
        release_unpublished_probe_context(context);
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
        (0x0400_3688 as *mut u32).write_volatile(0x0901_5ba8);
        (0x0400_36f8 as *mut u32).write_volatile(0x0901_5ba8);

        let mut previous = 0_u32;
        for index in 0..TX_CONTEXT_COUNT {
            let context = TX_CONTEXT_BASE + index * TX_CONTEXT_SIZE;
            let buffer = TX_BUFFER_BASE + index * TX_BUFFER_SIZE;
            ((context + 0x1c) as *mut u32).write_volatile((buffer + 0x40) as u32);
            ((context + 0xc4) as *mut u32).write_volatile((buffer + 0x20) as u32);
            ((context + 0x70) as *mut u16).write_volatile(0x00ff);
            ((context + 0x04) as *mut u32).write_volatile(previous);
            previous = context as u32;
        }
        (0x0400_9080 as *mut u32).write_volatile(previous);
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
    fn probe_tracker_requires_start_before_success_and_reclaims_once() {
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
        assert_eq!(tracker.reclaim_completed(), Some(0x0400_9084));
        assert_eq!(tracker.reclaim_completed(), None);
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
    fn nonpending_status_event_completes_the_probe() {
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
        assert_eq!(
            tracker.ownership(),
            ProbeTxOwnership::Completed {
                context: 0x0400_9364,
                pipe: 0,
                slot: 2,
                status: 0x0b,
            }
        );
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
        assert_eq!(
            build_phy_rate_words(0, 2, 0, 3, 5),
            PhyRateWords {
                control: 2,
                rate: 0x0003_0405,
            }
        );
        assert_eq!(
            build_phy_rate_words(14, 0, 0x28, 3, 7),
            PhyRateWords {
                control: 6,
                rate: 0x0003_1407,
            }
        );
    }
}
