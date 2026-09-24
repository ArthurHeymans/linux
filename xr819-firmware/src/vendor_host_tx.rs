//! Vendor-shaped ordinary host-TX state transitions.
//!
//! This module contains pure representations of the decompiled WSM host-data
//! path. Hardware-facing code can apply these plans without borrowing the
//! internal class-6 probe/template initializer.

use crate::packet_ram;

pub(crate) use crate::dtcm::HostContextAddress;
pub const HOST_FRAME_STATE_SIZE: u32 = packet_ram::HOST_FRAME_STATE_SIZE as u32;
pub const PAS_RING_CAPACITY: usize = 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct HostConfirmationFields {
    pub(crate) tx_rate: u8,
    pub(crate) flags: u16,
    pub(crate) rate_try: [u32; 3],
}

/// Read only the fields needed to build a class-0 confirmation at the original
/// vendor read point. Retained completion/IRQ writers make volatile scalar
/// reads mandatory even though foreground code owns the HIF release token.
#[cfg(target_arch = "arm")]
pub(crate) unsafe fn confirmation_fields(context: HostContextAddress) -> HostConfirmationFields {
    unsafe {
        HostConfirmationFields {
            tx_rate: read_host_u8(context.tx_rate()),
            flags: read_host_u16(context.completion_flags()),
            rate_try: [
                read_host_u32(context.rate_try(0).unwrap()),
                read_host_u32(context.rate_try(1).unwrap()),
                read_host_u32(context.rate_try(2).unwrap()),
            ],
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostPoolError {
    Empty,
    CorruptFreeHead,
    CorruptFrameState,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostTxPhase {
    Submitted,
    Classified,
    PostCryptoQueued,
    PendingEligible,
    PasQueued,
    SchedulerReserved,
    Scheduled,
    Completing,
}

pub const fn valid_phase_transition(current: HostTxPhase, next: HostTxPhase) -> bool {
    matches!(
        (current, next),
        (HostTxPhase::Submitted, HostTxPhase::Classified)
            | (HostTxPhase::Classified, HostTxPhase::PostCryptoQueued)
            | (HostTxPhase::PostCryptoQueued, HostTxPhase::PendingEligible)
            | (HostTxPhase::PendingEligible, HostTxPhase::PasQueued)
            | (HostTxPhase::PendingEligible, HostTxPhase::Completing)
            | (HostTxPhase::PasQueued, HostTxPhase::SchedulerReserved)
            | (HostTxPhase::SchedulerReserved, HostTxPhase::PasQueued)
            | (HostTxPhase::SchedulerReserved, HostTxPhase::Scheduled)
            | (HostTxPhase::Scheduled, HostTxPhase::PasQueued)
            | (HostTxPhase::Scheduled, HostTxPhase::Completing)
    )
}

/// Fields which must remain identical across one vendor A-MPDU chain.
///
/// The decompiled builder compares the internal link and rate directly. The
/// queue-to-pipe mapping supplies the TID grouping upstream; retaining it here
/// makes that otherwise implicit contract testable before any chain pointer is
/// published.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AmpduGroupingKey {
    pub(crate) interface: u8,
    pub(crate) link: u8,
    pub(crate) tid: u8,
    pub(crate) rate: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AmpduCandidate {
    pub(crate) key: AmpduGroupingKey,
    pub(crate) frame_control: u16,
}

/// Restrict the first aggregate slice to compatible same-interface/link/TID QoS data.
pub(crate) fn can_form_ampdu_pair(first: AmpduCandidate, second: AmpduCandidate) -> bool {
    let first_qos_data = first.frame_control & 0x008c == 0x0088;
    let second_qos_data = second.frame_control & 0x008c == 0x0088;
    let (tx_tids, _) = crate::configuration::block_ack_policy();
    let operational = crate::configuration::operational_tx_ba_tids();
    let tid_enabled = first.key.tid < 8
        && tx_tids & (1 << first.key.tid) != 0
        && operational & (1 << first.key.tid) != 0;
    first_qos_data
        && second_qos_data
        && tid_enabled
        && first.key.link < 8
        && first.key.rate >= 14
        && first.key.interface == second.key.interface
        && first.key.link == second.key.link
        && first.key.tid == second.key.tid
        && first.key.rate == second.key.rate
}

/// One borrowed HIF request and its class-0 context identity. The release
/// token stays here until the inherited confirmation handoff or an explicit
/// abort. This does not claim the vendor's request-buffer lifetime is matched.
#[cfg(target_arch = "arm")]
pub struct RetainedHostTx {
    context: HostContextAddress,
    release: crate::hif::RequestReleaseToken,
    packet_id: u32,
    phase: HostTxPhase,
}

#[cfg(target_arch = "arm")]
impl RetainedHostTx {
    pub const fn context(&self) -> HostContextAddress {
        self.context
    }

    pub const fn packet_id(&self) -> u32 {
        self.packet_id
    }

    pub const fn phase(&self) -> HostTxPhase {
        self.phase
    }

    pub fn transition(&mut self, next: HostTxPhase) -> bool {
        if !valid_phase_transition(self.phase, next) {
            return false;
        }
        self.phase = next;
        true
    }

    /// Undo admission before hardware/PAS ownership has been transferred.
    ///
    /// # Safety
    /// The context must not be linked into a pending list, PAS ring, or pipe.
    pub unsafe fn abort(self) -> crate::hif::RequestReleaseToken {
        unsafe { free_host_context(self.context) };
        self.release
    }

    /// Cancel a context before PAS ownership. A queued context is first
    /// unlinked from the vendor pending list under the same IRQ exclusion used
    /// by insertion.
    ///
    /// # Safety
    /// No concurrent Rust code may mutate the pending list.
    pub unsafe fn cancel_before_pas(
        self,
        _guard: &mut crate::mac_domain::MacDomainGuard<'_>,
    ) -> Result<crate::hif::RequestReleaseToken, CancelError> {
        match self.phase {
            HostTxPhase::Submitted | HostTxPhase::Classified | HostTxPhase::PendingEligible => {}
            HostTxPhase::PostCryptoQueued => unsafe {
                remove_pending_context(_guard, self.context)?
            },
            HostTxPhase::PasQueued => unsafe {
                remove_live_pas(_guard, self.context)?;
                release_pas_accounting(self.context);
            },
            HostTxPhase::SchedulerReserved | HostTxPhase::Scheduled | HostTxPhase::Completing => {
                return Err(CancelError::HardwareOwned);
            }
        }
        unsafe { free_host_context(self.context) };
        Ok(self.release)
    }

    /// Return the class-0 context after completion accounting and hand the
    /// original HIF request token to the inherited confirmation handoff.
    ///
    /// # Safety
    /// The completion path must have removed every PAS/pipe owner first.
    pub unsafe fn finish(self) -> crate::hif::RequestReleaseToken {
        unsafe { free_host_context(self.context) };
        self.release
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HostTxMetadata {
    pub message_address: u32,
    pub packet_id: u32,
    pub max_tx_rate: u8,
    pub queue_id: u8,
    pub more: bool,
    pub flags: u8,
    pub expire_time: u32,
    pub ht_tx_parameters: u32,
    pub frame_address: u32,
    pub frame_length: u16,
    pub interface: u8,
    pub submit_timer: u32,
    pub ac: u8,
    pub frame_state_address: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HeaderClassificationError {
    Truncated,
    InvalidLength,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostPrepareError {
    WrongPhase,
    Header(HeaderClassificationError),
    Crypto(crate::crypto::CcmpError),
    Timing(crate::tx::ProbeBuildError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PostCryptoError {
    WrongPhase,
    ExistingStatus(u16),
    OptionalPipeObjectRequired,
    Descriptor(crate::tx::ProbeBuildError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CancelError {
    HardwareOwned,
    MissingFromPendingList,
    MissingFromPasRing,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HeaderClassification {
    pub frame_control: u16,
    pub header_length: u16,
    pub payload_length: u16,
    pub qos_control: u16,
    pub tid: u8,
    pub flags: u32,
    pub assign_sequence: bool,
}

/// Pure ordinary-data subset of vendor `tx_classify_hdr_len`. It preserves
/// four-address, QoS, HT-control, multicast/no-ACK, QoS ACK-policy, and
/// sequence-assignment decisions without touching VIF counters or hardware.
pub fn classify_header(
    frame: &[u8],
    initial_flags: u32,
) -> Result<HeaderClassification, HeaderClassificationError> {
    if frame.len() < 2 || frame.len() > u16::MAX as usize {
        return Err(HeaderClassificationError::Truncated);
    }
    let frame_control = u16::from_le_bytes([frame[0], frame[1]]);
    // A BlockAckReq is a 20-octet control frame, shorter than the 24-octet data
    // header, so its shape has to be recognised before any data-frame minimum is
    // applied. mac80211 sends one through this path whenever an aggregate member
    // has to be abandoned; it carries no payload, no QoS control, and no sequence
    // to assign, because its own start sequence is what moves the peer's reorder
    // window. It must reach the air unaltered.
    if frame_control & 0x00fc == 0x0084 {
        if frame.len() < 20 {
            return Err(HeaderClassificationError::Truncated);
        }
        let bar_control = u16::from_le_bytes([frame[16], frame[17]]);
        return Ok(HeaderClassification {
            frame_control,
            header_length: 20,
            // The MPDU is the frame as retained, which includes the four octets of
            // FCS space appended above for a control frame.
            payload_length: (frame.len() - 20) as u16,
            qos_control: 0,
            // mac80211 encodes the compressed BAR control as
            // `CBMTID_COMPRESSED_BA (0x0004) | (tid << 12)`, so the TID is the
            // high nibble of the little-endian control word, not the low one.
            tid: ((bar_control >> 12) & 0x0f) as u8,
            // Bit 14 selects the BlockAck response class (`frame_kind` 0x0c in
            // `compute_single_frame_pas_timing`), which is what a BlockAckReq is
            // answered with - a compressed BlockAck after SIFS, not an ACK. It is
            // also the class the host TX path retires frames on: publishing the
            // BAR with the no-response class (0x0200) left it uncompleted and the
            // host's TID queue stalled behind it (measured: 261 datagrams in 313
            // seconds against roughly 31,000 in 30).
            flags: initial_flags | 0x1000 | 0x4000,
            assign_sequence: false,
        });
    }
    if frame.len() < 24 {
        return Err(HeaderClassificationError::Truncated);
    }
    let four_address = frame_control & 0x0300 == 0x0300;
    let base_header = if four_address { 30 } else { 24 };
    if frame.len() < base_header {
        return Err(HeaderClassificationError::Truncated);
    }

    let mut flags = initial_flags | 0x1000;
    if frame[4] & 1 != 0 {
        flags |= 0x300;
    }

    let qos = frame_control & 0x008f == 0x0088;
    let mut header_length = base_header;
    let mut qos_control = 0;
    let mut tid = 0xff;
    let mut assign_sequence = false;
    if qos {
        let qos_offset = base_header;
        if frame.len() < qos_offset + 2 {
            return Err(HeaderClassificationError::Truncated);
        }
        qos_control = u16::from_le_bytes([frame[qos_offset], frame[qos_offset + 1]]);
        tid = (qos_control & 0x0f) as u8;
        flags |= 0x0040_0000;
        if frame[4] & 1 == 0 {
            flags |= 1;
            // Vendor assigns a sequence number for ordinary QoS data unless
            // subtype bit 6 selects the exceptional shape.
            if frame_control & 0x0040 == 0 {
                flags |= 0x2000_0000;
                assign_sequence = true;
            }
        }
        let ack_policy = u32::from((qos_control & 0x7f) >> 5);
        flags = (flags & 0xffcf_ffff) | (ack_policy << 20);
        if ack_policy == 1 || ack_policy == 3 {
            flags |= 0x200;
        }
        header_length += if frame_control & 0x8000 != 0 { 6 } else { 2 };
    }
    if frame.len() < header_length {
        return Err(HeaderClassificationError::InvalidLength);
    }

    Ok(HeaderClassification {
        frame_control,
        header_length: header_length as u16,
        payload_length: (frame.len() - header_length) as u16,
        qos_control,
        tid,
        flags,
        assign_sequence,
    })
}

#[cfg(target_arch = "arm")]
unsafe fn assign_sequence_number(context: HostContextAddress, frame_address: u32, tid: u8) {
    let interface = unsafe { read_host_u8(context.interface()) };
    let host_link = unsafe { read_host_u8(context.host_link()) };
    let mut internal_link = crate::vif::internal_link(interface).unwrap_or(0) as u8;
    if host_link != 0 {
        let count = unsafe {
            crate::dtcm::shared_ptr::<u16>(crate::dtcm::link_map_entry_count()).read_volatile()
        };
        let mut index = 0_u16;
        while index < count {
            let entry = crate::dtcm::link_map_entry_unchecked(usize::from(index));
            if unsafe { read_live_u8(entry.interface().get() as u32) } == interface
                && unsafe { read_live_u8(entry.host_link().get() as u32) } == host_link
            {
                internal_link = unsafe { read_live_u8(entry.internal_link().get() as u32) };
                break;
            }
            index += 1;
        }
    }
    let sequence_address = crate::dtcm::link_sequence_counter_unchecked(
        usize::from(internal_link),
        usize::from(tid),
    );
    let sequence = unsafe { crate::dtcm::shared_ptr::<u16>(sequence_address).read_volatile() };
    unsafe {
        (frame_address.wrapping_add(0x16) as *mut u16).write_volatile(sequence);
        write_host_u16(context.sequence_number(), sequence >> 4);
        crate::dtcm::shared_ptr::<u16>(sequence_address)
            .write_volatile(sequence.wrapping_add(0x10) & 0xfff0);
    }
}

/// Apply ordinary `tx_classify_hdr_len`, assign the vendor per-link/TID
/// sequence number, then run software CCMP directly in the retained HIF frame.
///
/// # Safety
/// The retained request must still be exclusively owned by this context.
#[cfg(target_arch = "arm")]
pub unsafe fn classify_and_encrypt(retained: &mut RetainedHostTx) -> Result<(), HostPrepareError> {
    if retained.phase != HostTxPhase::Submitted {
        return Err(HostPrepareError::WrongPhase);
    }
    let context = retained.context;
    let frame_address = unsafe { read_host_u32(context.frame_address()) };
    let mut frame_length = unsafe { read_host_u16(context.frame_length()) };
    // A 20-octet control MPDU is not transmitted by the MAC, while the same frame at
    // 24 octets goes out - those four octets are where the FCS sits, so the MPDU the
    // peer sees is a complete BlockAckReq (measured: 0 on air at 20 octets, 51 at 24).
    // Extend the retained frame in place and let the classification report the real
    // MPDU length rather than trimming it back to 20.
    if frame_length >= 20
        && unsafe { (frame_address as *const u8).read_volatile() } & 0xfc == 0x84
    {
        unsafe {
            core::ptr::write_bytes(
                (frame_address as *mut u8).add(usize::from(frame_length)),
                0,
                4,
            );
        }
        frame_length += 4;
        unsafe { write_host_u16(context.frame_length(), frame_length) };
    }
    let frame = unsafe {
        core::slice::from_raw_parts_mut(frame_address as *mut u8, usize::from(frame_length))
    };
    let initial_flags = unsafe { read_host_u32(context.control_bits()) };
    let classification = classify_header(frame, initial_flags).map_err(HostPrepareError::Header)?;

    if classification.assign_sequence {
        unsafe { assign_sequence_number(context, frame_address, classification.tid) };
    }
    let interface = unsafe { read_host_u8(context.interface()) };
    unsafe {
        write_host_u16(context.frame_control(), classification.frame_control);
        write_host_u32(context.header_length(), u32::from(classification.header_length));
        write_host_u32(context.payload_length(), u32::from(classification.payload_length));
        write_host_u32(context.control_bits(), classification.flags);
        write_host_u16(context.qos_control(), classification.qos_control);
        write_host_u8(context.tid(), classification.tid);
        write_host_u8(context.retry_rate(), 0xff);
        let duration_slot = read_live_u8(
            crate::dtcm::pas_stride_view_unchecked(usize::from(interface))
                .slot_bits()
                .get() as u32,
        ) & 1;
        write_host_u8(context.duration_slot(), duration_slot);
    }
    retained.phase = HostTxPhase::Classified;
    crate::crypto::encrypt_tx_frame(frame, interface).map_err(HostPrepareError::Crypto)?;
    unsafe { crate::tx::prepare_host_frame_timing(context.raw()) }.map_err(HostPrepareError::Timing)
}

/// Build `ctx+0xa0` and perform the exact mode-0 tail insertion from the
/// post-crypto callback. The pending task, PAS release, and scheduler remain
/// separate later phases.
///
/// # Safety
/// The context must be classified, crypto-complete, timing-complete, and not
/// already linked into any list.
#[cfg(target_arch = "arm")]
pub unsafe fn enqueue_post_crypto(retained: &mut RetainedHostTx) -> Result<(), PostCryptoError> {
    if retained.phase != HostTxPhase::Classified {
        return Err(PostCryptoError::WrongPhase);
    }
    let context = retained.context;
    let status = unsafe { read_host_u16(context.terminal_status()) };
    if status < 0x00fe {
        return Err(PostCryptoError::ExistingStatus(status));
    }

    // The normal unicast station path leaves this optional per-peer object
    // disabled. Do not silently skip its allocation for multicast or when the
    // vendor global enables the alternate object class.
    let flags = unsafe { read_host_u32(context.control_bits()) };
    let alternate_enabled = unsafe {
        (crate::dtcm::LOW_MAC_OPTIONAL_PIPE_OBJECT_WORD.get() as *const u16).read_volatile() != 0
    };
    if flags & 0x100 != 0 || alternate_enabled {
        return Err(PostCryptoError::OptionalPipeObjectRequired);
    }
    unsafe { write_host_u32(context.optional_pipe_object(), 0) };
    unsafe { crate::tx::build_host_frame_descriptor(context.raw()) }
        .map_err(PostCryptoError::Descriptor)?;

    let previous = unsafe { crate::tx::disable_irq_fiq_save() };
    unsafe {
        let (pending_head, pending_tail) = (crate::dtcm::pending_tx_head().get() as u32, crate::dtcm::pending_tx_tail().get() as u32);
        let tail = read_live_u32(pending_tail);
        write_host_u32(context.intrusive_next(), 0);
        if tail == 0 {
            write_live_u32(pending_head, context.raw());
        } else {
            write_context_link(tail, context.raw());
        }
        write_live_u32(pending_tail, context.raw());
        write_host_u32(context.ownership_bits(), read_host_u32(context.ownership_bits()) | 0x20);
        if read_host_u8(context.more()) == 0 {
            let scheduler_events = crate::dtcm::scheduler_pending_events().get() as u32;
            write_live_u32(
                scheduler_events,
                read_live_u32(scheduler_events) | 0x0020_0000,
            );
        }
        crate::tx::restore_irq_fiq_saved(previous);
    }
    retained.phase = HostTxPhase::PostCryptoQueued;
    Ok(())
}

#[cfg(target_arch = "arm")]
unsafe fn remove_pending_context(
    _guard: &mut crate::mac_domain::MacDomainGuard<'_>,
    context: HostContextAddress,
) -> Result<(), CancelError> {
    let (pending_head, pending_tail) = (crate::dtcm::pending_tx_head().get() as u32, crate::dtcm::pending_tx_tail().get() as u32);
    let mut prior = 0_u32;
    let mut current = unsafe { read_live_u32(pending_head) };
    while current != 0 && current != context.raw() {
        prior = current;
        current = unsafe { read_context_link(current) };
    }
    if current == 0 {
        return Err(CancelError::MissingFromPendingList);
    }
    let next = unsafe { read_context_link(current) };
    unsafe {
        if prior == 0 {
            write_live_u32(pending_head, next);
        } else {
            write_context_link(prior, next);
        }
        if read_live_u32(pending_tail) == current {
            write_live_u32(pending_tail, prior);
        }
        write_context_link(current, 0);
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PendingServiceReport {
    LeaveQueued,
    Complete(u16),
    PasQueued,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PendingServiceError {
    WrongPhase,
    PendingList(CancelError),
    Timing(crate::tx::ProbeBuildError),
    PhyState,
    PasRingFull,
}

impl PendingServiceError {
    pub const fn diagnostic_code(self) -> u8 {
        match self {
            Self::WrongPhase => 1,
            Self::PendingList(_) => 2,
            Self::Timing(_) => 3,
            Self::PhyState => 4,
            Self::PasRingFull => 5,
        }
    }
}

/// Vendor microsecond timer. Exposed so diagnostics can rate-limit without
/// duplicating the address pair.
///
/// # Safety
/// Reads live vendor timer registers.
#[cfg(target_arch = "arm")]
pub unsafe fn vendor_timer_now() -> u32 {
    unsafe { vendor_timer() }
}

#[cfg(target_arch = "arm")]
unsafe fn vendor_timer() -> u32 {
    unsafe {
        read_live_u32(0x0ac0_0004)
            .wrapping_add(crate::dtcm::initialized_timer_counter_ptr().read_volatile())
    }
}

#[cfg(any(test, target_arch = "arm"))]
const fn pas_submission_expired(submitted: u32, now: u32) -> bool {
    (submitted.wrapping_sub(now).wrapping_add(0x007a_1200) as i32) < 0
}

#[allow(unused_macros)]
macro_rules! observe_normal_power_save_release {
    ($sleeping:expr, $frame_control:expr, $link_gate:expr, $link_bit:expr, $policy:expr, $flags:expr) => {{
        let sleeping = $sleeping;
        let frame_control = $frame_control;
        let frame_kind = frame_control & 0xff;
        let normal_release = ((sleeping & $link_bit == 0)
            || frame_kind == 0x50
            || (frame_kind == 0xd0 && $policy == 0x0f))
            && (($link_bit & 1 == 0) || sleeping == 0 || $link_gate == 0)
            && $flags & (1 << 29) == 0;
        (sleeping, frame_control, normal_release)
    }};
}

/// Exact boolean/side-effect translation of `txp_program_pipe_hw(pas, 0)`.
/// Vendor return value zero means blocked; nonzero means the pending task may
/// release the frame toward PAS scheduling.
#[cfg(target_arch = "arm")]
unsafe fn program_pipe_eligible(context: HostContextAddress) -> bool {
    let pas = context.pas().raw();
    let interface = unsafe { read_live_u8(pas + 0x69) };
    if interface > 2 {
        return true;
    }
    let pas_state = crate::dtcm::pas_stride_view_unchecked(usize::from(interface));
    let Some(vif_control_bits) = crate::vif::flags(interface) else { return false; };

    let blocked = if unsafe { read_live_u8(pas_state.nonzero_block_byte().get() as u32) } != 0 {
        true
    } else if unsafe { read_live_u8(pas_state.tbtt_window_control_byte().get() as u32) } == 0 {
        vif_control_bits & (1 << 29) != 0 && vif_control_bits & 3 == 3
    } else {
        let tsf = unsafe {
            read_live_u32(crate::platform::mac_register(0x0e38) as u32)
                .wrapping_add(read_live_u32(pas_state.tsf_adjust_low().get() as u32))
        };
        let until_tbtt = unsafe { read_live_u32(pas_state.next_tbtt_low().get() as u32) }
            .wrapping_sub(tsf) as i32;
        let duration = u32::from(unsafe { read_live_u16(pas + 0x30) })
            + u32::from(unsafe { read_live_u16(pas + 0x34) })
            + u32::from(unsafe { read_live_u16(pas + 0x38) })
            + u32::from(unsafe { read_live_u16(pas + 0x36) });
        until_tbtt < 1 || until_tbtt <= duration as i32
    };
    let policy = unsafe { read_live_u8(pas + 0x0e) };
    let global = unsafe { crate::dtcm::scheduler_exclusion_state_ptr().read_volatile() };
    if (policy != 0x0f || global & 0x80 == 0) && blocked { return false; }
    if vif_control_bits & 4 == 0 { return true; }

    let link = unsafe { read_live_u8(pas + 0x6b) };
    let link_bit = 1_u16.wrapping_shl(u32::from(link));
    let vif = crate::dtcm::vif_record_unchecked(usize::from(interface));
    let (sleeping, frame_control, normal_release) = observe_normal_power_save_release!(
        crate::vif::sleeping_links(vif),
        unsafe { read_live_u16(pas + 0x0a) },
        crate::vif::link_gate(vif),
        link_bit,
        policy,
        vif_control_bits
    );
    if normal_release { return true; }

    let Some(power_save) = crate::vif::power_save_masks(interface) else { return false; };
    let awake = power_save.awake_links;
    let buffered = power_save.buffered_links;
    if awake & link_bit == 0 && buffered & link_bit == 0 {
        let blocked_links = crate::dtcm::link_release_blocked_links().get() as u32;
        if unsafe { read_live_u16(blocked_links) } & link_bit != 0 { return false; }
        if crate::vif::allowed_links(interface).unwrap_or(0) & link_bit == 0 { return false; }
        let count = unsafe { read_live_u16(crate::dtcm::link_map_entry_count().get() as u32) };
        let mut index = 0_u16;
        while index < count {
            let entry = crate::dtcm::link_map_entry_unchecked(usize::from(index));
            if unsafe { read_live_u8(pas + 0x6b) }
                == unsafe { read_live_u8(entry.host_link().get() as u32) }
            {
                let release_flags = entry.release_flags().get() as u32;
                unsafe {
                    write_live_u8(release_flags, read_live_u8(release_flags) | 2);
                    write_live_u16(blocked_links, read_live_u16(blocked_links) | link_bit);
                }
            }
            index += 1;
        }
        return false;
    }

    if awake & link_bit != 0 && buffered & link_bit != 0 && frame_control & 0x80 != 0 {
        let header = unsafe { read_live_u32(pas) };
        let qos_control = unsafe { read_live_u16(header + 0x18) };
        if qos_control & 0x10 == 0 { return true; }
        if crate::vif::clear_buffered_link(interface, buffered, link_bit).is_err() {
            crate::halt_always!();
        }
        return true;
    }

    if crate::vif::clear_awake_link_and_maybe_recompute(
        interface,
        sleeping,
        awake,
        link_bit,
    )
    .is_err()
    {
        crate::halt_always!();
    }
    true
}

#[cfg(target_arch = "arm")]
unsafe fn push_live_pas(
    _guard: &mut crate::mac_domain::MacDomainGuard<'_>,
    context: HostContextAddress,
) -> Result<(), PendingServiceError> {
    let old_tail = unsafe { read_live_u32(crate::dtcm::HOST_PAS_RING_TAIL.get() as u32) as u8 & 0x3f };
    let mut scan = unsafe { read_live_u32(crate::dtcm::HOST_PAS_RING_HEAD.get() as u32) as u8 & 0x3f };
    let mut write = old_tail;
    while scan != old_tail {
        let slot = crate::dtcm::host_pas_ring_slot_unchecked(usize::from(scan)).get() as u32;
        let value = unsafe { read_live_u32(slot) };
        if value != 0 {
            unsafe {
                write_live_u32(
                    crate::dtcm::host_pas_ring_slot_unchecked(usize::from(write)).get() as u32,
                    value,
                );
                write_live_u32(slot, 0);
            }
            write = write.wrapping_add(1) & 0x3f;
        }
        scan = scan.wrapping_add(1) & 0x3f;
    }
    unsafe {
        write_live_u32(crate::dtcm::HOST_PAS_RING_HEAD.get() as u32, u32::from(old_tail));
        write_live_u32(crate::dtcm::HOST_PAS_RING_TAIL.get() as u32, u32::from(write));
    }
    let next = write.wrapping_add(1) & 0x3f;
    if next == old_tail {
        return Err(PendingServiceError::PasRingFull);
    }
    unsafe {
        write_live_u32(
            crate::dtcm::host_pas_ring_slot_unchecked(usize::from(write)).get() as u32,
            context.pas().raw(),
        );
        write_live_u32(crate::dtcm::HOST_PAS_RING_TAIL.get() as u32, u32::from(next));
    }
    Ok(())
}

#[cfg(target_arch = "arm")]
unsafe fn push_live_pas_front(
    _guard: &mut crate::mac_domain::MacDomainGuard<'_>,
    context: HostContextAddress,
) -> Result<(), PendingServiceError> {
    let head = unsafe { read_live_u32(crate::dtcm::HOST_PAS_RING_HEAD.get() as u32) as u8 & 0x3f };
    let tail = unsafe { read_live_u32(crate::dtcm::HOST_PAS_RING_TAIL.get() as u32) as u8 & 0x3f };
    let new_head = head.wrapping_sub(1) & 0x3f;
    if new_head == tail {
        return Err(PendingServiceError::PasRingFull);
    }
    unsafe {
        write_live_u32(
            crate::dtcm::host_pas_ring_slot_unchecked(usize::from(new_head)).get() as u32,
            context.pas().raw(),
        );
        write_live_u32(
            crate::dtcm::HOST_PAS_RING_HEAD.get() as u32,
            u32::from(new_head),
        );
    }
    Ok(())
}

#[cfg(target_arch = "arm")]
pub unsafe fn retry_attempted(retained: &RetainedHostTx) -> bool {
    unsafe { read_host_u16(retained.context.try_count()) != 0 }
}

/// First non-empty PAS frame in scheduler order.
///
/// # Safety
/// The caller must serialize access to the live PAS ring.
#[cfg(target_arch = "arm")]
pub unsafe fn first_live_pas_frame() -> Option<u32> {
    let head = unsafe { read_live_u32(crate::dtcm::HOST_PAS_RING_HEAD.get() as u32) as u8 & 0x3f };
    let tail = unsafe { read_live_u32(crate::dtcm::HOST_PAS_RING_TAIL.get() as u32) as u8 & 0x3f };
    if head == tail {
        return None;
    }
    let frame = unsafe {
        read_live_u32(
            crate::dtcm::host_pas_ring_slot_unchecked(usize::from(head)).get() as u32,
        )
    };
    (frame != 0).then_some(frame)
}

pub(crate) const fn requeued_retry_control_bits(bits: u32) -> u32 {
    (bits | 0x10) & !((1 << 28) | (1 << 27) | (1 << 26) | (1 << 5))
}

/// Return a retryable hardware-owned context to the normal PAS scheduler.
///
/// # Safety
/// The aggregate slot must already be retired, and `retained` must be its
/// unique scheduled host owner. The MAC-domain guard serializes ring mutation.
#[cfg(target_arch = "arm")]
pub unsafe fn requeue_scheduled_retry(
    guard: &mut crate::mac_domain::MacDomainGuard<'_>,
    retained: &mut RetainedHostTx,
) -> Result<(), PendingServiceError> {
    if retained.phase != HostTxPhase::Scheduled {
        return Err(PendingServiceError::WrongPhase);
    }
    let context = retained.context;
    unsafe { push_live_pas_front(guard, context)? };
    unsafe {
        write_host_u32(
            context.control_bits(),
            requeued_retry_control_bits(read_host_u32(context.control_bits())),
        );
        write_host_u16(
            context.frame_control(),
            read_host_u16(context.frame_control()) | 0x0800,
        );
        write_host_u32(
            context.ownership_bits(),
            read_host_u32(context.ownership_bits()) & !0x100,
        );
        write_host_u32(context.next_in_ampdu(), 0);
        let scheduler_events = crate::dtcm::scheduler_pending_events().get() as u32;
        write_live_u32(scheduler_events, read_live_u32(scheduler_events) | 0x0020_0000);
    }
    retained.phase = HostTxPhase::PasQueued;
    Ok(())
}

#[cfg(target_arch = "arm")]
unsafe fn remove_live_pas(
    _guard: &mut crate::mac_domain::MacDomainGuard<'_>,
    context: HostContextAddress,
) -> Result<(), CancelError> {
    let target = context.pas().raw();
    let head = unsafe { read_live_u32(crate::dtcm::HOST_PAS_RING_HEAD.get() as u32) as u8 & 0x3f };
    let tail = unsafe { read_live_u32(crate::dtcm::HOST_PAS_RING_TAIL.get() as u32) as u8 & 0x3f };
    let mut scan = head;
    let mut found = false;
    while scan != tail {
        let slot = crate::dtcm::host_pas_ring_slot_unchecked(usize::from(scan)).get() as u32;
        if unsafe { read_live_u32(slot) } == target {
            unsafe { write_live_u32(slot, 0) };
            found = true;
            break;
        }
        scan = scan.wrapping_add(1) & 0x3f;
    }
    if !found {
        return Err(CancelError::MissingFromPasRing);
    }
    // Leave the hole for the vendor compaction step performed by the next PAS
    // insertion. This matches normal completion/removal ring semantics.
    Ok(())
}

#[cfg(target_arch = "arm")]
unsafe fn claim_pas_accounting(context: HostContextAddress) -> bool {
    unsafe {
        let active = crate::tx::active_pas_contexts();
        if active == 0 && !crate::phy::advance_awake_station_tx() {
            return false;
        }
        crate::tx::set_active_pas_contexts(active.wrapping_add(1));
        let interface = read_host_u8(context.interface());
        if interface < 3 {
            if crate::vif::adjust_tx_busy(interface, 1).is_err() {
                crate::halt_always!();
            }
        }
        true
    }
}

#[cfg(target_arch = "arm")]
unsafe fn release_pas_accounting(context: HostContextAddress) {
    unsafe {
        crate::tx::set_active_pas_contexts(crate::tx::active_pas_contexts().wrapping_sub(1));
        let interface = read_host_u8(context.interface());
        if interface < 3 {
            if crate::vif::adjust_tx_busy(interface, -1).is_err() {
                crate::halt_always!();
            }
        }
    }
}

#[cfg(target_arch = "arm")]
unsafe fn release_pending_to_pas(
    mac_domain: &mut crate::mac_domain::MacDomain,
    retained: &mut RetainedHostTx,
) -> Result<(), PendingServiceError> {
    let context = retained.context;
    unsafe {
        write_host_u32(context.ownership_bits(), read_host_u32(context.ownership_bits()) | 0x40);
        crate::tx::prepare_host_frame_timing(context.raw()).map_err(PendingServiceError::Timing)?;
        if !claim_pas_accounting(context) {
            return Err(PendingServiceError::PhyState);
        }
        let push_result = {
            let mut guard = mac_domain.enter();
            push_live_pas(&mut guard, context)
        };
        if let Err(error) = push_result {
            release_pas_accounting(context);
            return Err(error);
        }
    }
    retained.phase = HostTxPhase::PasQueued;
    Ok(())
}

/// Service the retained context through the vendor `task_b88e` decision and,
/// when eligible, through `tx_frame_done_release` into the global PAS ring.
///
/// # Safety
/// The context and global pending/PAS structures must be runtime-owned.
#[cfg(target_arch = "arm")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PendingLiveDiagnostic {
    pub global: u32,
    pub active_mask: u16,
    pub effective_mask: u16,
    pub vif_control_bits: u32,
    pub vif_mode_byte: u8,
    pub interface: u8,
    pub link: u8,
    pub pipe_allowed: bool,
}

/// Snapshot the exact live gates used by vendor pending task `0xb88e`.
///
/// # Safety
/// The retained context and vendor VIF/global state must remain mapped.
#[cfg(target_arch = "arm")]
pub unsafe fn pending_live_diagnostic(retained: &RetainedHostTx) -> PendingLiveDiagnostic {
    let context = retained.context;
    let interface = unsafe { read_host_u8(context.interface()) };
    let link = unsafe { read_host_u8(context.host_link()) };
    let vif = crate::vif::diagnostic_snapshot(interface);
    PendingLiveDiagnostic {
        global: unsafe { crate::dtcm::scheduler_exclusion_state_ptr().read_volatile() },
        active_mask: vif.map_or(0, |state| state.allowed_links),
        effective_mask: vif.map_or(0, |state| state.effective_links),
        vif_control_bits: vif.map_or(0, |state| state.flags),
        vif_mode_byte: vif.map_or(0, |state| state.mode),
        interface,
        link,
        pipe_allowed: unsafe { program_pipe_eligible(context) },
    }
}

#[cfg(target_arch = "arm")]
pub unsafe fn service_pending(
    mac_domain: &mut crate::mac_domain::MacDomain,
    retained: &mut RetainedHostTx,
) -> Result<PendingServiceReport, PendingServiceError> {
    if retained.phase == HostTxPhase::PendingEligible {
        unsafe { release_pending_to_pas(mac_domain, retained)? };
        return Ok(PendingServiceReport::PasQueued);
    }
    if retained.phase != HostTxPhase::PostCryptoQueued {
        return Err(PendingServiceError::WrongPhase);
    }

    let context = retained.context;
    let interface = unsafe { read_host_u8(context.interface()) };
    let link = unsafe { read_host_u8(context.host_link()) };
    let link_bit = 1_u16.wrapping_shl(u32::from(link));
    let completion_class = unsafe { read_host_u8(context.completion_class()) };
    let global_blocked = unsafe { crate::dtcm::scheduler_exclusion_state_ptr().read_volatile() } & 0xa0 != 0;
    let now = unsafe { vendor_timer() };
    let submitted = unsafe { read_host_u32(context.submit_timer()) };
    let expired = (submitted.wrapping_sub(now).wrapping_add(0x004c_4b40) as i32) < 0;

    let decision = if global_blocked {
        pending_task_decision(PendingTaskInput {
            global_blocked: true,
            active_link: true,
            link_gate_requests_removal: false,
            vif_operating: false,
            completion_class,
            pipe_allowed: false,
            expired,
        })
    } else {
        let vif = crate::dtcm::vif_record_unchecked(usize::from(interface));
        let active_mask = crate::vif::vif_read_u16(vif.allowed_links());
        let active_link = active_mask & link_bit != 0;
        let mut effective_mask = crate::vif::vif_read_u16(vif.effective_links());
        let mut effective_link = effective_mask & link_bit != 0;
        let frame_kind = unsafe { read_host_u16(context.frame_control()) } & 0xff;
        let vif_control_bits = crate::vif::vif_read_u32(vif.flags());
        if !effective_link && frame_kind != 0xd0 && effective_mask == 0 {
            if vif_control_bits & (1 << 30) != 0 {
                crate::vif::vif_write_u32(vif.flags(), vif_control_bits | (1 << 26));
            } else {
                let radio_state = crate::vif::vif_read_u8(vif.activity_state());
                if radio_state == 2 || radio_state == 3 {
                    effective_mask = active_mask;
                    if interface < 2 {
                        let sleeping = crate::vif::vif_read_u16(vif.sleeping_links());
                        let buffered = crate::vif::vif_read_u16(vif.buffered_links());
                        let awake = crate::vif::vif_read_u16(vif.awake_links());
                        effective_mask = (active_mask & (!sleeping | buffered)) | awake;
                    }
                    crate::vif::vif_write_u16(vif.effective_links(), effective_mask);
                    effective_link = effective_mask & link_bit != 0;
                }
            }
        }
        let link_gate_requests_removal =
            (effective_link || frame_kind == 0xd0) && vif_control_bits & (1 << 29) == 0;
        if (effective_link || frame_kind == 0xd0) && vif_control_bits & (1 << 29) != 0 {
            crate::vif::vif_write_u32(vif.flags(), vif_control_bits | (1 << 26));
        }
        let state = crate::vif::vif_read_u8(vif.mode());
        pending_task_decision(PendingTaskInput {
            global_blocked: false,
            active_link,
            link_gate_requests_removal,
            vif_operating: state == 4 || state == 6,
            completion_class,
            pipe_allowed: unsafe { program_pipe_eligible(context) },
            expired,
        })
    };

    match decision {
        PendingTaskDecision::LeaveQueued => Ok(PendingServiceReport::LeaveQueued),
        PendingTaskDecision::Complete(status) => {
            let remove_result = {
                let mut guard = mac_domain.enter();
                unsafe { remove_pending_context(&mut guard, context) }
            };
            remove_result.map_err(PendingServiceError::PendingList)?;
            retained.phase = HostTxPhase::PendingEligible;
            Ok(PendingServiceReport::Complete(status))
        }
        PendingTaskDecision::ReleaseToPas => {
            let remove_result = {
                let mut guard = mac_domain.enter();
                unsafe { remove_pending_context(&mut guard, context) }
            };
            remove_result.map_err(PendingServiceError::PendingList)?;
            retained.phase = HostTxPhase::PendingEligible;
            unsafe { release_pending_to_pas(mac_domain, retained)? };
            Ok(PendingServiceReport::PasQueued)
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NonAggregateSchedulerInput {
    pub expired: bool,
    pub pipe_allowed: bool,
    pub pipe: u8,
    pub idle_pipe_mask: u8,
    pub ring_contains_frame: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NonAggregateSchedulerDecision {
    LeaveQueued,
    Complete(u16),
    ReservePipe(u8),
}

pub const fn non_aggregate_scheduler_decision(
    input: NonAggregateSchedulerInput,
) -> NonAggregateSchedulerDecision {
    if input.expired {
        return NonAggregateSchedulerDecision::Complete(10);
    }
    if !input.ring_contains_frame
        || !input.pipe_allowed
        || input.pipe >= 4
        || input.idle_pipe_mask & (1 << input.pipe) == 0
    {
        return NonAggregateSchedulerDecision::LeaveQueued;
    }
    NonAggregateSchedulerDecision::ReservePipe(input.pipe)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerReserveError {
    WrongPhase,
    SchedulerBlocked,
    LeaveQueued,
    Expired,
    PipeStateUnavailable,
    Descriptor(crate::tx::ProbeBuildError),
}

impl SchedulerReserveError {
    pub const fn diagnostic_code(self) -> u8 {
        match self {
            Self::WrongPhase => 1,
            Self::SchedulerBlocked => 2,
            Self::LeaveQueued => 3,
            Self::Expired => 4,
            Self::PipeStateUnavailable => 5,
            Self::Descriptor(_) => 6,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SchedulerLiveDiagnostic {
    pub pipe: u8,
    pub idle_pipe_mask: u8,
    pub ring_head: u8,
    pub ring_tail: u8,
    pub ring_contains_frame: bool,
    pub pipe_allowed: bool,
    pub retry_gate: u8,
    pub receive_gate: u8,
}

#[cfg(target_arch = "arm")]
pub(crate) unsafe fn ampdu_candidate(retained: &RetainedHostTx) -> AmpduCandidate {
    let context = retained.context;
    AmpduCandidate {
        key: AmpduGroupingKey {
            interface: unsafe { read_host_u8(context.interface()) },
            link: unsafe { read_host_u8(context.link_id()) },
            tid: unsafe { read_host_u8(context.tid()) },
            rate: unsafe { read_host_u8(context.tx_rate()) },
        },
        frame_control: unsafe { read_host_u16(context.frame_control()) },
    }
}

/// Snapshot the non-aggregate scheduler gates without changing ownership.
///
/// # Safety
/// The retained PAS context and global ring/pipe state must remain mapped.
#[cfg(target_arch = "arm")]
pub unsafe fn scheduler_live_diagnostic(retained: &RetainedHostTx) -> SchedulerLiveDiagnostic {
    let context = retained.context;
    let pas = context.pas().raw();
    let mut idle_pipe_mask = 0_u8;
    for candidate in 0..4_u8 {
        let state = crate::dtcm::mac_pipe_state_unchecked(usize::from(candidate));
        if unsafe { read_live_u8(state.get() as u32) } == 0 {
            idle_pipe_mask |= 1 << candidate;
        }
    }
    let ac = unsafe { read_host_u8(context.access_category()) };
    let pipe = unsafe {
        read_live_u8(crate::dtcm::access_category_to_queue_unchecked(usize::from(ac)).get() as u32)
    };
    let ring_head = unsafe { read_live_u32(crate::dtcm::HOST_PAS_RING_HEAD.get() as u32) as u8 & 0x3f };
    let ring_tail = unsafe { read_live_u32(crate::dtcm::HOST_PAS_RING_TAIL.get() as u32) as u8 & 0x3f };
    let mut slot = ring_head;
    while slot != ring_tail
        && unsafe {
            read_live_u32(crate::dtcm::host_pas_ring_slot_unchecked(usize::from(slot)).get() as u32)
        } != pas
    {
        slot = slot.wrapping_add(1) & 0x3f;
    }
    SchedulerLiveDiagnostic {
        pipe,
        idle_pipe_mask,
        ring_head,
        ring_tail,
        ring_contains_frame: slot != ring_tail,
        pipe_allowed: unsafe { program_pipe_eligible(context) },
        retry_gate: unsafe {
            read_live_u8(crate::dtcm::mac_retry_hardware_state_mmio_address())
        },
        receive_gate: unsafe {
            read_live_u8(crate::dtcm::LOW_MAC_RECEIVE_GATE_BITS.get() as u32)
        },
    }
}

/// Remove an unscheduled PAS frame for class-0 rejection/expiry confirmation.
///
/// # Safety
/// The frame must still be present in the global PAS ring and own no pipe slot.
#[cfg(target_arch = "arm")]
pub unsafe fn reject_unscheduled_pas(
    guard: &mut crate::mac_domain::MacDomainGuard<'_>,
    retained: &mut RetainedHostTx,
) -> Result<(), CancelError> {
    if retained.phase != HostTxPhase::PasQueued {
        return Err(CancelError::HardwareOwned);
    }
    unsafe {
        remove_live_pas(guard, retained.context)?;
        release_pas_accounting(retained.context);
    }
    retained.phase = HostTxPhase::PendingEligible;
    Ok(())
}

/// Software-owned reservation produced by the minimum non-aggregate scheduler.
/// It has removed the PAS ring entry and built the pipe-slot descriptor, but it
/// has not advanced the producer or triggered hardware.
#[cfg(target_arch = "arm")]
pub struct HostSchedulerReservation {
    context: HostContextAddress,
    pipe: u8,
    slot: u8,
    ring_slot: u8,
    original_ring_head: u8,
    slot_record: u32,
    command: u32,
    original_control_bits: u32,
    original_slot_header: u32,
    original_slot_frame: u32,
    original_slot_auxiliary: u32,
    original_command: [u32; 16],
}

#[cfg(target_arch = "arm")]
impl HostSchedulerReservation {
    pub const fn pipe(&self) -> u8 {
        self.pipe
    }

    pub const fn slot(&self) -> u8 {
        self.slot
    }

    /// Cross the irreversible producer/MAC-trigger boundary.
    ///
    /// # Safety
    /// This reservation must exclusively own the pipe slot and retained frame.
    pub unsafe fn publish(
        self,
        guard: &mut crate::mac_domain::MacDomainGuard<'_>,
        retained: &mut RetainedHostTx,
    ) -> Result<(), (Self, crate::tx::ProbeBuildError)> {
        unsafe { self.publish_in_batch(guard, retained, crate::tx::BatchPosition::Only) }
    }

    /// Stage this reservation as part of a pipe batch. `Only` is the historic
    /// single-frame path; `First`/`Middle`/`Last` write slots without arming.
    /// After every member succeeds, the batch owner arms them together through
    /// `finalize_staged_host_class0_pipe`.
    ///
    /// # Safety
    /// Same as `publish`.
    pub unsafe fn publish_in_batch(
        self,
        guard: &mut crate::mac_domain::MacDomainGuard<'_>,
        retained: &mut RetainedHostTx,
        batch: crate::tx::BatchPosition,
    ) -> Result<(), (Self, crate::tx::ProbeBuildError)> {
        if retained.context != self.context || retained.phase != HostTxPhase::SchedulerReserved {
            return Err((self, crate::tx::ProbeBuildError::PipeSlotOwnershipMismatch));
        }
        if let Err(error) = unsafe {
            crate::tx::publish_host_class0_slot(
                guard,
                self.context.raw(),
                self.pipe,
                self.slot,
                self.slot_record,
                self.command,
                batch,
            )
        } {
            return Err((self, error));
        }
        retained.phase = HostTxPhase::Scheduled;
        Ok(())
    }

    unsafe fn restore_slot_image(&self) {
        unsafe {
            let pipe = usize::from(self.pipe);
            let slot = usize::from(self.slot);
            write_live_u32(self.slot_record, self.original_slot_header);
            write_live_u32(
                crate::dtcm::mac_pipe_slot_frame_unchecked(pipe, slot).get() as u32,
                self.original_slot_frame,
            );
            write_live_u32(
                crate::dtcm::mac_pipe_slot_auxiliary_unchecked(pipe, slot).get() as u32,
                self.original_slot_auxiliary,
            );
            for (index, word) in self.original_command.iter().copied().enumerate() {
                write_live_u32(self.command + index as u32 * 4, word);
            }
        }
    }

    /// Restore software ownership before any pipe producer or hardware trigger.
    ///
    /// # Safety
    /// The reservation must not have been published.
    pub unsafe fn cancel(
        self,
        _guard: &mut crate::mac_domain::MacDomainGuard<'_>,
        retained: &mut RetainedHostTx,
    ) -> bool {
        if retained.context != self.context || retained.phase != HostTxPhase::SchedulerReserved {
            return false;
        }
        unsafe {
            write_host_u32(self.context.control_bits(), self.original_control_bits);
            self.restore_slot_image();
            write_live_u32(
                crate::dtcm::host_pas_ring_slot_unchecked(usize::from(self.ring_slot)).get() as u32,
                self.context.pas().raw(),
            );
            write_live_u32(crate::dtcm::HOST_PAS_RING_HEAD.get() as u32, u32::from(self.original_ring_head));
        }
        retained.phase = HostTxPhase::PasQueued;
        true
    }
}

#[cfg(target_arch = "arm")]
unsafe fn queued_pas_ring_slot(context: HostContextAddress, head: u8, tail: u8) -> Option<u8> {
    let pas = context.pas().raw();
    let mut slot = head;
    while slot != tail {
        if unsafe { read_live_u32(crate::dtcm::host_pas_ring_slot_unchecked(usize::from(slot)).get() as u32) } == pas {
            return Some(slot);
        }
        slot = slot.wrapping_add(1) & 0x3f;
    }
    None
}

#[cfg(target_arch = "arm")]
/// FIFO distance from the live PAS head, independent of context arena reuse.
pub unsafe fn queued_pas_ring_distance(retained: &RetainedHostTx) -> Option<u8> {
    let position = unsafe { queued_pas_ring_position(retained) }?;
    let head = unsafe { read_live_u32(crate::dtcm::HOST_PAS_RING_HEAD.get() as u32) as u8 & 0x3f };
    Some(position.wrapping_sub(head) & 0x3f)
}

#[cfg(target_arch = "arm")]
pub unsafe fn queued_pas_ring_position(retained: &RetainedHostTx) -> Option<u8> {
    let head = unsafe {
        read_live_u32(crate::dtcm::HOST_PAS_RING_HEAD.get() as u32) as u8 & 0x3f
    };
    let tail = unsafe {
        read_live_u32(crate::dtcm::HOST_PAS_RING_TAIL.get() as u32) as u8 & 0x3f
    };
    unsafe { queued_pas_ring_slot(retained.context, head, tail) }
}

/// Select one PAS-ring frame, reserve its mapped idle pipe slot, and emit the
/// kind-0 descriptor without starting hardware.
///
/// # Safety
/// Global PAS and pipe state must be exclusively runtime-owned.
const fn scheduler_batch_control_bits(original: u32, staged: u8) -> u32 {
    original
        | if staged == 0 {
            1 << 26
        } else {
            0x0800_0000
        }
}

#[cfg(target_arch = "arm")]
pub unsafe fn reserve_non_aggregate_scheduler(
    guard: &mut crate::mac_domain::MacDomainGuard<'_>,
    retained: &mut RetainedHostTx,
) -> Result<HostSchedulerReservation, SchedulerReserveError> {
    unsafe { reserve_non_aggregate_scheduler_at(guard, retained, None) }
}

/// Reserve a later slot in the same software-owned non-aggregate batch.
///
/// # Safety
/// `pipe` must be the pipe reserved by the first batch member, and `slot`
/// must be its next unowned ring slot. No staged slot may have crossed the MAC
/// trigger boundary.
#[cfg(target_arch = "arm")]
pub unsafe fn reserve_non_aggregate_scheduler_in_batch(
    guard: &mut crate::mac_domain::MacDomainGuard<'_>,
    retained: &mut RetainedHostTx,
    pipe: u8,
    slot: u8,
    staged: u8,
) -> Result<HostSchedulerReservation, SchedulerReserveError> {
    unsafe { reserve_non_aggregate_scheduler_at(guard, retained, Some((pipe, slot, staged))) }
}

#[cfg(target_arch = "arm")]
unsafe fn reserve_non_aggregate_scheduler_at(
    _guard: &mut crate::mac_domain::MacDomainGuard<'_>,
    retained: &mut RetainedHostTx,
    batch: Option<(u8, u8, u8)>,
) -> Result<HostSchedulerReservation, SchedulerReserveError> {
    if retained.phase != HostTxPhase::PasQueued {
        return Err(SchedulerReserveError::WrongPhase);
    }
    if unsafe { read_live_u8(crate::dtcm::mac_retry_hardware_state_mmio_address()) } != 0
        || unsafe { read_live_u8(crate::dtcm::LOW_MAC_RECEIVE_GATE_BITS.get() as u32) } != 0
    {
        return Err(SchedulerReserveError::SchedulerBlocked);
    }

    let context = retained.context;
    let pas = context.pas().raw();
    let mut idle_pipe_mask = 0_u8;
    let mut candidate = 0_u8;
    while candidate < 4 {
        let state = crate::dtcm::mac_pipe_state_unchecked(usize::from(candidate));
        if unsafe { read_live_u8(state.get() as u32) } == 0 {
            idle_pipe_mask |= 1 << candidate;
        }
        candidate += 1;
    }
    let ac = unsafe { read_host_u8(context.access_category()) };
    let pipe = unsafe {
        read_live_u8(crate::dtcm::access_category_to_queue_unchecked(usize::from(ac)).get() as u32)
    };
    let head = unsafe { read_live_u32(crate::dtcm::HOST_PAS_RING_HEAD.get() as u32) as u8 & 0x3f };
    let tail = unsafe { read_live_u32(crate::dtcm::HOST_PAS_RING_TAIL.get() as u32) as u8 & 0x3f };
    let mut ring_slot = head;
    while ring_slot != tail
        && unsafe {
            read_live_u32(
                crate::dtcm::host_pas_ring_slot_unchecked(usize::from(ring_slot)).get() as u32,
            )
        } != pas
    {
        ring_slot = ring_slot.wrapping_add(1) & 0x3f;
    }
    let now = unsafe { vendor_timer() };
    let submitted = unsafe { read_host_u32(context.submit_timer()) };
    let decision = non_aggregate_scheduler_decision(NonAggregateSchedulerInput {
        expired: pas_submission_expired(submitted, now),
        pipe_allowed: unsafe { program_pipe_eligible(context) },
        pipe,
        idle_pipe_mask,
        ring_contains_frame: ring_slot != tail,
    });
    match decision {
        NonAggregateSchedulerDecision::LeaveQueued => {
            return Err(SchedulerReserveError::LeaveQueued);
        }
        NonAggregateSchedulerDecision::Complete(_) => {
            return Err(SchedulerReserveError::Expired);
        }
        NonAggregateSchedulerDecision::ReservePipe(_) => {}
    }

    let (slot, staged) = if let Some((batch_pipe, slot, staged)) = batch {
        if batch_pipe != pipe {
            return Err(SchedulerReserveError::LeaveQueued);
        }
        (slot & 3, staged)
    } else {
        let slot = unsafe {
            read_live_u8(crate::dtcm::mac_pipe_current_slot_unchecked(usize::from(pipe)).get() as u32)
        } & 3;
        (slot, 0)
    };
    let pipe_index = usize::from(pipe);
    let slot_index = usize::from(slot);
    let slot_record = crate::dtcm::mac_pipe_slot_state_word_unchecked(pipe_index, slot_index).get() as u32;
    let slot_frame = crate::dtcm::mac_pipe_slot_frame_unchecked(pipe_index, slot_index).get() as u32;
    let slot_auxiliary = crate::dtcm::mac_pipe_slot_auxiliary_unchecked(pipe_index, slot_index).get() as u32;
    let command = unsafe {
        read_live_u32(crate::dtcm::mac_pipe_slot_command_unchecked(pipe_index, slot_index).get() as u32)
    };
    let hardware_ring = unsafe {
        read_live_u32(crate::dtcm::mac_pipe_hardware_ring_unchecked(pipe_index).get() as u32)
    };
    if command == 0 || hardware_ring == 0 {
        return Err(SchedulerReserveError::PipeStateUnavailable);
    }
    let original_control_bits = unsafe { read_host_u32(context.control_bits()) };
    let original_slot_header = unsafe { read_live_u32(slot_record) };
    let original_slot_frame = unsafe { read_live_u32(slot_frame) };
    let original_slot_auxiliary = unsafe { read_live_u32(slot_auxiliary) };
    let mut original_command = [0_u32; 16];
    for (index, word) in original_command.iter_mut().enumerate() {
        *word = unsafe { read_live_u32(command + index as u32 * 4) };
    }

    unsafe {
        write_live_u32(
            crate::dtcm::host_pas_ring_slot_unchecked(usize::from(ring_slot)).get() as u32,
            0,
        );
        let mut new_head = head;
        while new_head != tail
            && read_live_u32(
                crate::dtcm::host_pas_ring_slot_unchecked(usize::from(new_head)).get() as u32,
            ) == 0
        {
            new_head = new_head.wrapping_add(1) & 0x3f;
        }
        write_live_u32(crate::dtcm::HOST_PAS_RING_HEAD.get() as u32, u32::from(new_head));
        // Vendor marks the first ordinary descriptor with bit 26 and every
        // later descriptor in the same scheduler batch with bit 27 before
        // `txp_build_pipe_descriptor(..., 0)`.
        write_host_u32(
            context.control_bits(),
            scheduler_batch_control_bits(original_control_bits, staged),
        );
        write_host_u32(
            context.ownership_bits(),
            read_host_u32(context.ownership_bits()) | 0x100,
        );
        write_host_u32(context.scheduler_timestamp(), vendor_timer());
        write_host_u32(context.next_in_ampdu(), 0);
        write_live_u8(slot_record, 0);
        write_live_u8(
            crate::dtcm::mac_pipe_slot_retry_rate_unchecked(pipe_index, slot_index).get() as u32,
            read_host_u8(context.retry_rate()),
        );
        write_live_u8(
            crate::dtcm::mac_pipe_slot_control_02_unchecked(pipe_index, slot_index).get() as u32,
            0,
        );
        write_live_u8(
            crate::dtcm::mac_pipe_slot_control_03_unchecked(pipe_index, slot_index).get() as u32,
            0,
        );
        write_live_u32(slot_frame, pas);
        // `txp_build_pipe_descriptor` clears the per-slot auxiliary descriptor
        // pointer for every kind-0 frame before emitting its command stream.
        // This storage is retained DTCM and cannot be left at its prior value.
        write_live_u32(slot_auxiliary, 0);
        write_live_u32(command, 0);
        write_live_u32(command + 4, 0);
        write_live_u32(command + 8, 0xdc00_0000);
        if let Err(error) = crate::tx::emit_host_frame_descriptor_at(context.raw(), command + 0x0c)
        {
            write_live_u32(
                crate::dtcm::host_pas_ring_slot_unchecked(usize::from(ring_slot)).get() as u32,
                pas,
            );
            write_live_u32(crate::dtcm::HOST_PAS_RING_HEAD.get() as u32, u32::from(head));
            write_host_u32(context.control_bits(), original_control_bits);
            write_live_u32(slot_record, original_slot_header);
            write_live_u32(slot_frame, original_slot_frame);
            write_live_u32(slot_auxiliary, original_slot_auxiliary);
            for (index, word) in original_command.into_iter().enumerate() {
                write_live_u32(command + index as u32 * 4, word);
            }
            return Err(SchedulerReserveError::Descriptor(error));
        }
    }
    retained.phase = HostTxPhase::SchedulerReserved;
    Ok(HostSchedulerReservation {
        context,
        pipe,
        slot,
        ring_slot,
        original_ring_head: head,
        slot_record,
        command,
        original_control_bits,
        original_slot_header,
        original_slot_frame,
        original_slot_auxiliary,
        original_command,
    })
}

pub trait HostContextWriter {
    fn write_u8(&mut self, address: crate::dtcm::DtcmAddress, value: u8);
    fn write_u16(&mut self, address: crate::dtcm::DtcmAddress, value: u16);
    fn write_u32(&mut self, address: crate::dtcm::DtcmAddress, value: u32);
}

/// Emit only fields written by `tx_wsm_buf_alloc`, `wsm_h_04_tx_req`, and the
/// non-policy portion of `tx_lmac_req_submit`. Pool-owned and unknown fields
/// remain untouched in a live context.
pub fn write_host_context_fields<W: HostContextWriter>(
    writer: &mut W,
    context: HostContextAddress,
    metadata: HostTxMetadata,
) {
    writer.write_u32(context.frame_state_address(), metadata.frame_state_address);

    writer.write_u8(context.request_flags(), 0);
    writer.write_u32(context.completion_status(), 0xfe);
    writer.write_u16(context.terminal_status(), 0xfe);
    writer.write_u32(context.ownership_bits(), 0);

    writer.write_u32(context.request_buffer(), metadata.message_address);
    writer.write_u32(context.packet_id(), metadata.packet_id);
    writer.write_u8(context.requested_rate(), metadata.max_tx_rate);
    writer.write_u8(context.queue_id(), metadata.queue_id);
    writer.write_u8(context.more(), u8::from(metadata.more));
    writer.write_u8(context.request_flags(), metadata.flags);
    writer.write_u32(context.expiry_time(), metadata.expire_time);
    writer.write_u32(context.ht_tx_parameters(), metadata.ht_tx_parameters);
    writer.write_u32(context.borrowed_frame_length(), u32::from(metadata.frame_length));
    writer.write_u32(context.borrowed_frame_address(), metadata.frame_address);
    writer.write_u8(context.rate_copy(), metadata.max_tx_rate);
    writer.write_u8(context.saved_status(), 0);
    writer.write_u16(context.completion_flags(), 0);
    for index in 0..3 {
        writer.write_u32(context.rate_try(index).unwrap(), 0);
    }

    writer.write_u32(context.ownership_bits(), 1);
    writer.write_u8(context.interface(), metadata.interface);
    writer.write_u8(context.host_link(), (metadata.queue_id & 0x3f) >> 2);
    writer.write_u8(context.queue_id(), metadata.queue_id & 3);
    writer.write_u32(context.submit_timer(), metadata.submit_timer);
    writer.write_u8(context.completion_class(), 0);
    writer.write_u16(context.sequence_or_callback_state(), 0);
    writer.write_u8(context.submit_state(), 1);
    writer.write_u32(context.optional_pipe_object(), 0);
    let flags = 0x0080_0000
        | ((metadata.ht_tx_parameters >> 11) & 0xe0)
        | if metadata.flags & 1 != 0 {
            0x0001_0000
        } else {
            0
        }
        | if metadata.ht_tx_parameters & 3 == 1 {
            8
        } else {
            0
        };
    writer.write_u32(context.control_bits(), flags);
    for index in 0..3 {
        writer.write_u32(context.pas_reset_word(index).unwrap(), 0);
    }
    writer.write_u32(context.completion_timestamp(), metadata.submit_timer.wrapping_sub(1));
    writer.write_u32(context.scheduler_timestamp(), metadata.submit_timer.wrapping_sub(1));
    for index in 0..2 {
        writer.write_u32(context.timing_scratch(index).unwrap(), 0);
    }
    writer.write_u32(context.pas_expiry_time(), metadata.expire_time);
    writer.write_u32(context.frame_address(), metadata.frame_address);
    writer.write_u8(context.access_category(), metadata.ac);
    writer.write_u8(context.request_flag_rate_bits(), (metadata.flags & 0x0f) >> 1);
    writer.write_u8(context.retry_policy(), (metadata.flags & 0x7f) >> 4);
    writer.write_u16(context.frame_length(), metadata.frame_length);
    writer.write_u16(context.terminal_status(), 0xfe);
    writer.write_u16(context.try_count(), 0);
    writer.write_u16(context.auxiliary_state(), 0);
    writer.write_u8(context.insertion_mode(), 1);
    writer.write_u32(context.next_in_ampdu(), 0);
    writer.write_u8(context.tx_rate(), metadata.max_tx_rate);
}

struct SliceContextWriter<'a> {
    context: HostContextAddress,
    image: &'a mut [u8; crate::dtcm::HOST_TX_CONTEXT_SIZE],
}

impl HostContextWriter for SliceContextWriter<'_> {
    fn write_u8(&mut self, address: crate::dtcm::DtcmAddress, value: u8) {
        self.image[address.get() - self.context.raw() as usize] = value;
    }

    fn write_u16(&mut self, address: crate::dtcm::DtcmAddress, value: u16) {
        let offset = address.get() - self.context.raw() as usize;
        self.image[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }

    fn write_u32(&mut self, address: crate::dtcm::DtcmAddress, value: u32) {
        let offset = address.get() - self.context.raw() as usize;
        self.image[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
}

/// Build a zero-based context image for tests and documentation. Runtime code
/// should call `write_host_context_fields` with a volatile writer instead.
pub fn initialize_host_context(
    image: &mut [u8; crate::dtcm::HOST_TX_CONTEXT_SIZE],
    metadata: HostTxMetadata,
) {
    image.fill(0);
    let context = HostContextAddress::from_index(0).unwrap();
    write_host_context_fields(&mut SliceContextWriter { context, image }, context, metadata);
}

#[cfg(target_arch = "arm")]
struct VolatileContextWriter;

#[cfg(target_arch = "arm")]
impl HostContextWriter for VolatileContextWriter {
    fn write_u8(&mut self, address: crate::dtcm::DtcmAddress, value: u8) {
        unsafe { write_host_u8(address, value) };
    }

    fn write_u16(&mut self, address: crate::dtcm::DtcmAddress, value: u16) {
        unsafe { write_host_u16(address, value) };
    }

    fn write_u32(&mut self, address: crate::dtcm::DtcmAddress, value: u32) {
        unsafe { write_host_u32(address, value) };
    }
}

/// Initialize vendor-written fields of a live host-pool context without
/// clearing pool-owned or currently unidentified words.
///
/// The caller must hold exclusive ownership of the aligned 0x170-byte host
/// context for the duration of this initialization.
#[cfg(target_arch = "arm")]
pub fn initialize_host_context_at(context: HostContextAddress, metadata: HostTxMetadata) {
    write_host_context_fields(&mut VolatileContextWriter, context, metadata);
}

#[cfg(target_arch = "arm")]
unsafe fn read_live_u8(address: u32) -> u8 {
    unsafe { (address as *const u8).read_volatile() }
}

#[cfg(target_arch = "arm")]
unsafe fn read_live_u16(address: u32) -> u16 {
    unsafe { (address as *const u16).read_volatile() }
}

#[cfg(target_arch = "arm")]
unsafe fn read_live_u32(address: u32) -> u32 {
    unsafe { (address as *const u32).read_volatile() }
}

#[cfg(target_arch = "arm")]
unsafe fn write_live_u8(address: u32, value: u8) {
    unsafe { (address as *mut u8).write_volatile(value) };
}

#[cfg(target_arch = "arm")]
unsafe fn write_live_u16(address: u32, value: u16) {
    unsafe { (address as *mut u16).write_volatile(value) };
}

#[cfg(target_arch = "arm")]
unsafe fn write_live_u32(address: u32, value: u32) {
    unsafe { (address as *mut u32).write_volatile(value) };
}

#[cfg(target_arch = "arm")]
#[inline(always)]
unsafe fn read_host_u8(address: crate::dtcm::DtcmAddress) -> u8 {
    unsafe { crate::dtcm::shared_ptr::<u8>(address).read_volatile() }
}

#[cfg(target_arch = "arm")]
#[inline(always)]
unsafe fn read_host_u16(address: crate::dtcm::DtcmAddress) -> u16 {
    unsafe { crate::dtcm::shared_ptr::<u16>(address).read_volatile() }
}

#[cfg(target_arch = "arm")]
#[inline(always)]
unsafe fn read_host_u32(address: crate::dtcm::DtcmAddress) -> u32 {
    unsafe { crate::dtcm::shared_ptr::<u32>(address).read_volatile() }
}

#[cfg(target_arch = "arm")]
#[inline(always)]
unsafe fn write_host_u8(address: crate::dtcm::DtcmAddress, value: u8) {
    unsafe { crate::dtcm::shared_ptr::<u8>(address).write_volatile(value) };
}

#[cfg(target_arch = "arm")]
#[inline(always)]
unsafe fn write_host_u16(address: crate::dtcm::DtcmAddress, value: u16) {
    unsafe { crate::dtcm::shared_ptr::<u16>(address).write_volatile(value) };
}

#[cfg(target_arch = "arm")]
#[inline(always)]
unsafe fn write_host_u32(address: crate::dtcm::DtcmAddress, value: u32) {
    unsafe { crate::dtcm::shared_ptr::<u32>(address).write_volatile(value) };
}

/// Read an intrusive `+0x04` link from a mixed internal/host TX list. Host
/// nodes use the semantic layout; retained internal nodes keep their existing
/// raw compatibility view until that family is decoded.
#[cfg(target_arch = "arm")]
unsafe fn read_context_link(context: u32) -> u32 {
    if let Some(context) = HostContextAddress::from_raw(context) {
        unsafe { read_host_u32(context.intrusive_next()) }
    } else {
        unsafe { read_live_u32(context.wrapping_add(4)) }
    }
}

#[cfg(target_arch = "arm")]
unsafe fn write_context_link(context: u32, next: u32) {
    if let Some(context) = HostContextAddress::from_raw(context) {
        unsafe { write_host_u32(context.intrusive_next(), next) };
    } else {
        unsafe { write_live_u32(context.wrapping_add(4), next) };
    }
}

trait HostPoolIo {
    fn read_u32(&mut self, address: crate::dtcm::DtcmAddress) -> u32;
    fn write_u8(&mut self, address: crate::dtcm::DtcmAddress, value: u8);
    fn write_u16(&mut self, address: crate::dtcm::DtcmAddress, value: u16);
    fn write_u32(&mut self, address: crate::dtcm::DtcmAddress, value: u32);
}

fn rebuild_host_free_list<I: HostPoolIo>(io: &mut I) {
    io.write_u32(crate::dtcm::HOST_CONTEXT_FREE_HEAD, 0);
    let mut head = 0;
    for index in 0..crate::dtcm::HOST_TX_CONTEXT_COUNT {
        let context = HostContextAddress::from_index(index).unwrap();
        io.write_u32(context.intrusive_next(), head);
        io.write_u16(context.terminal_status(), 0x00ff);
        io.write_u32(context.frame_state_address(), context.expected_frame_state().raw());
        head = context.raw();
    }
    io.write_u32(crate::dtcm::HOST_CONTEXT_FREE_HEAD, head);
}

fn pop_host_free_list<I: HostPoolIo>(io: &mut I) -> Result<HostContextAddress, HostPoolError> {
    let head = io.read_u32(crate::dtcm::HOST_CONTEXT_FREE_HEAD);
    let Some(context) = HostContextAddress::from_raw(head) else {
        return Err(if head == 0 { HostPoolError::Empty } else { HostPoolError::CorruptFreeHead });
    };
    if io.read_u32(context.frame_state_address()) != context.expected_frame_state().raw() {
        return Err(HostPoolError::CorruptFrameState);
    }
    let next = io.read_u32(context.intrusive_next());
    io.write_u32(crate::dtcm::HOST_CONTEXT_FREE_HEAD, next);
    io.write_u8(context.request_flags(), 0);
    io.write_u32(context.completion_status(), 0xfe);
    io.write_u16(context.terminal_status(), 0x00fe);
    io.write_u32(context.ownership_bits(), 0);
    Ok(context)
}

fn push_host_free_list<I: HostPoolIo>(io: &mut I, context: HostContextAddress) {
    io.write_u32(context.completion_status(), 0xff);
    io.write_u16(context.terminal_status(), 0x00ff);
    let ownership = io.read_u32(context.ownership_bits());
    io.write_u32(context.ownership_bits(), ownership | 0x0004_0000);
    let head = io.read_u32(crate::dtcm::HOST_CONTEXT_FREE_HEAD);
    io.write_u32(context.intrusive_next(), head);
    io.write_u32(crate::dtcm::HOST_CONTEXT_FREE_HEAD, context.raw());
}

#[cfg(target_arch = "arm")]
struct VolatileHostPoolIo;

#[cfg(target_arch = "arm")]
impl HostPoolIo for VolatileHostPoolIo {
    fn read_u32(&mut self, address: crate::dtcm::DtcmAddress) -> u32 {
        unsafe { read_host_u32(address) }
    }
    fn write_u8(&mut self, address: crate::dtcm::DtcmAddress, value: u8) {
        unsafe { write_host_u8(address, value) };
    }
    fn write_u16(&mut self, address: crate::dtcm::DtcmAddress, value: u16) {
        unsafe { write_host_u16(address, value) };
    }
    fn write_u32(&mut self, address: crate::dtcm::DtcmAddress, value: u32) {
        unsafe { write_host_u32(address, value) };
    }
}

/// Rebuild the vendor 30-entry host free list while preserving the dedicated
/// packet-SRAM descriptor address assigned to each context.
///
/// # Safety
/// No host context may be pending, PAS-owned, scheduled, or completing.
#[cfg(target_arch = "arm")]
pub unsafe fn initialize_host_pool() {
    let previous = unsafe { crate::tx::disable_irq_fiq_save() };
    unsafe {
        if crate::vif::set_host_contexts_in_flight(0).is_err() {
            crate::halt_always!();
        }
        rebuild_host_free_list(&mut VolatileHostPoolIo);
        crate::tx::restore_irq_fiq_saved(previous);
    }
}

#[cfg(not(target_arch = "arm"))]
pub unsafe fn initialize_host_pool() {
    unsafe { core::arch::asm!("") };
}

/// Allocate with the parent's deliberate recovery policy around
/// `tx_wsm_buf_alloc`: when the typed in-flight count is zero, any empty,
/// invalid-head, or frame-state mismatch rebuilds the complete pool. A zero
/// head remains an ordinary empty pool when the count is nonzero, avoiding a
/// destructive rebuild of contexts that may still be hardware-owned.
///
/// This is broader than the vendor allocator, which does not rebuild here.
/// Narrowing it to an uninitialized-head signature would change the exact
/// parent recovery semantics and requires separate failure evidence.
#[cfg(target_arch = "arm")]
pub unsafe fn allocate_host_context() -> Result<HostContextAddress, HostPoolError> {
    let previous = unsafe { crate::tx::disable_irq_fiq_save() };
    let in_flight = crate::vif::host_contexts_in_flight().unwrap_or(0);
    let result = pop_host_free_list(&mut VolatileHostPoolIo);
    match result {
        Ok(context) => {
            unsafe {
                if crate::vif::adjust_host_contexts_in_flight(1).is_err() {
                    crate::halt_always!();
                }
                crate::tx::restore_irq_fiq_saved(previous);
            }
            Ok(context)
        }
        Err(_) if in_flight == 0 => {
            unsafe { crate::tx::restore_irq_fiq_saved(previous) };
            unsafe { initialize_host_pool() };
            unsafe { allocate_host_context() }
        }
        Err(error) => {
            unsafe { crate::tx::restore_irq_fiq_saved(previous) };
            Err(error)
        }
    }
}

/// Return exactly as `tx_wsm_buf_free`.
///
/// # Safety
/// `context` must be allocated and have no remaining queue or hardware owner.
#[cfg(target_arch = "arm")]
pub unsafe fn free_host_context(context: HostContextAddress) {
    let previous = unsafe { crate::tx::disable_irq_fiq_save() };
    unsafe {
        push_host_free_list(&mut VolatileHostPoolIo, context);
        if crate::vif::adjust_host_contexts_in_flight(-1).is_err() {
            crate::halt_always!();
        }
        crate::tx::restore_irq_fiq_saved(previous);
    }
}

/// Admit an ordinary WSM TX request into one real class-0 host context while
/// retaining the original HIF packet-RAM buffer.
///
/// This intentionally stops at the `tx_lmac_req_submit` boundary. Header
/// classification, software crypto, post-crypto queueing, and scheduling must
/// advance the returned explicit phase before hardware publication.
const fn station_data_rate(requested: u8) -> u8 {
    // XR819 indices 4/5 are unsupported ERP-PBCC gaps and are not emitted by
    // cw1200. Keep a safe fallback for malformed requests while preserving the
    // host-selected CCK, OFDM, and HT rate.
    if requested <= 21 && requested != 4 && requested != 5 {
        requested
    } else {
        0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostAdmissionError {
    MalformedRequest,
    Pool(HostPoolError),
}

#[cfg(target_arch = "arm")]
pub unsafe fn admit_host_tx(
    buffer: crate::hif::RequestBuffer,
    interface: u8,
) -> Result<RetainedHostTx, (crate::hif::RequestBuffer, HostAdmissionError)> {
    let request = match crate::wsm::TxRequest::parse(buffer.payload()) {
        Ok(request) => request,
        Err(_) => return Err((buffer, HostAdmissionError::MalformedRequest)),
    };
    let queue = request.queue_id & 3;
    let ac = unsafe {
        read_live_u8(crate::dtcm::queue_to_access_category_unchecked(usize::from(queue)).get() as u32)
    };
    let submit_timer = unsafe {
        read_live_u32(0x0ac0_0004)
            .wrapping_add(crate::dtcm::initialized_timer_counter_ptr().read_volatile())
    };
    let packet_id = request.packet_id;
    let metadata = HostTxMetadata {
        message_address: buffer.buffer_address(),
        packet_id,
        max_tx_rate: station_data_rate(request.max_tx_rate),
        queue_id: request.queue_id,
        more: request.more,
        flags: request.flags,
        expire_time: request.expire_time,
        ht_tx_parameters: request.ht_tx_parameters,
        frame_address: request.frame.as_ptr() as u32,
        frame_length: request.frame.len().min(u16::MAX as usize) as u16,
        interface,
        submit_timer,
        ac,
        frame_state_address: 0,
    };
    let context = match unsafe { allocate_host_context() } {
        Ok(context) => context,
        Err(error) => return Err((buffer, HostAdmissionError::Pool(error))),
    };
    let metadata = HostTxMetadata {
        frame_state_address: context.expected_frame_state().raw(),
        ..metadata
    };
    unsafe { initialize_host_context_at(context, metadata) };
    Ok(RetainedHostTx {
        context,
        release: buffer.into_release(),
        packet_id,
        phase: HostTxPhase::Submitted,
    })
}

pub fn read_u16(image: &[u8; crate::dtcm::HOST_TX_CONTEXT_SIZE], offset: usize) -> u16 {
    u16::from_le_bytes([image[offset], image[offset + 1]])
}

pub fn read_u32(image: &[u8; crate::dtcm::HOST_TX_CONTEXT_SIZE], offset: usize) -> u32 {
    u32::from_le_bytes([
        image[offset],
        image[offset + 1],
        image[offset + 2],
        image[offset + 3],
    ])
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PendingListInsertion {
    pub context_next: u32,
    pub new_head: u32,
    pub new_tail: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PendingListRemoval {
    pub new_head: u32,
    pub new_tail: u32,
    pub prior_next: Option<u32>,
}

pub const fn remove_pending_list_node(
    head: u32,
    tail: u32,
    prior: u32,
    context: u32,
    next: u32,
) -> Option<PendingListRemoval> {
    if context == 0 || (prior == 0 && head != context) || (prior != 0 && head == context) {
        return None;
    }
    Some(PendingListRemoval {
        new_head: if prior == 0 { next } else { head },
        new_tail: if tail == context { prior } else { tail },
        prior_next: if prior == 0 { None } else { Some(next) },
    })
}

/// Exact `txq_list_insert(ctx, queue, 0)` append-at-tail mutation.
pub const fn append_pending_list(head: u32, tail: u32, context: u32) -> PendingListInsertion {
    PendingListInsertion {
        context_next: 0,
        new_head: if tail == 0 { context } else { head },
        new_tail: context,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PasRingError {
    InvalidFrameKind,
    Full,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PasRing {
    pub head: u8,
    pub tail: u8,
    pub entries: [u32; PAS_RING_CAPACITY],
}

impl PasRing {
    pub const fn empty() -> Self {
        Self {
            head: 0,
            tail: 0,
            entries: [0; PAS_RING_CAPACITY],
        }
    }

    /// Compact holes exactly as `pas_txq_push_global()` does before insertion.
    pub fn compact(&mut self) {
        let old_tail = self.tail;
        let mut scan = self.head;
        // Vendor compacts live entries into the free range beginning at the
        // old tail, then advances the logical head to that old tail.
        let mut write = old_tail;
        while scan != old_tail {
            let value = self.entries[usize::from(scan)];
            if value != 0 {
                self.entries[usize::from(write)] = value;
                write = write.wrapping_add(1) & 0x3f;
                self.entries[usize::from(scan)] = 0;
            }
            scan = scan.wrapping_add(1) & 0x3f;
        }
        self.head = old_tail;
        self.tail = write;
    }

    /// Insert a PAS pointer according to PAS `+0x53` (`ctx+0xa7`). Host WSM
    /// contexts use frame kind 1 and append at the tail.
    pub fn push(&mut self, pas: u32, frame_kind: u8) -> Result<(), PasRingError> {
        self.compact();
        match frame_kind {
            1 => {
                let next = self.tail.wrapping_add(1) & 0x3f;
                if next == self.head {
                    return Err(PasRingError::Full);
                }
                self.entries[usize::from(self.tail)] = pas;
                self.tail = next;
            }
            0 => {
                let previous = self.head.wrapping_sub(1) & 0x3f;
                if previous == self.tail {
                    return Err(PasRingError::Full);
                }
                self.entries[usize::from(previous)] = pas;
                self.head = previous;
            }
            _ => return Err(PasRingError::InvalidFrameKind),
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PendingTaskInput {
    pub global_blocked: bool,
    pub active_link: bool,
    /// Result of the VIF suspended-link/action-frame branch: true means the
    /// task has decided this context should be removed from the pending list.
    pub link_gate_requests_removal: bool,
    pub vif_operating: bool,
    pub completion_class: u8,
    pub pipe_allowed: bool,
    pub expired: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PendingTaskDecision {
    LeaveQueued,
    Complete(u16),
    ReleaseToPas,
}

/// Minimum semantic result of `task_b88e` for ordinary class-0 traffic.
pub const fn pending_task_decision(input: PendingTaskInput) -> PendingTaskDecision {
    if !input.active_link {
        return PendingTaskDecision::Complete(0x14);
    }
    if input.global_blocked {
        return if input.completion_class == 0 && input.expired {
            PendingTaskDecision::Complete(10)
        } else if input.completion_class == 6 || input.completion_class == 9 {
            PendingTaskDecision::ReleaseToPas
        } else {
            PendingTaskDecision::LeaveQueued
        };
    }
    if input.link_gate_requests_removal {
        return PendingTaskDecision::ReleaseToPas;
    }
    if input.vif_operating && (input.completion_class == 6 || input.pipe_allowed) {
        return PendingTaskDecision::ReleaseToPas;
    }
    if input.expired && input.completion_class == 0 {
        return PendingTaskDecision::Complete(10);
    }
    PendingTaskDecision::LeaveQueued
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum ObservationKind {
        SleepingLinks,
        FrameControl,
        LinkGate,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct Observation {
        kind: ObservationKind,
        address: usize,
        width: u8,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum AccessKind {
        Read,
        Write,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct PoolAccess {
        kind: AccessKind,
        address: usize,
        width: u8,
        value: u32,
    }

    struct PoolRecorder {
        bytes: std::collections::BTreeMap<usize, u8>,
        accesses: std::vec::Vec<PoolAccess>,
        entered: bool,
    }

    impl PoolRecorder {
        fn new() -> Self {
            Self {
                bytes: std::collections::BTreeMap::new(),
                accesses: std::vec::Vec::new(),
                entered: false,
            }
        }

        fn enter(&mut self) -> bool {
            if self.entered {
                false
            } else {
                self.entered = true;
                true
            }
        }

        fn leave(&mut self) {
            assert!(self.entered);
            self.entered = false;
        }

        fn seed_u32(&mut self, address: crate::dtcm::DtcmAddress, value: u32) {
            for (offset, byte) in value.to_le_bytes().into_iter().enumerate() {
                self.bytes.insert(address.get() + offset, byte);
            }
        }

        fn load(&self, address: usize, width: usize) -> u32 {
            let mut bytes = [0_u8; 4];
            for (offset, byte) in bytes.iter_mut().take(width).enumerate() {
                *byte = self.bytes.get(&(address + offset)).copied().unwrap_or(0);
            }
            u32::from_le_bytes(bytes)
        }

        fn store(&mut self, address: usize, width: usize, value: u32) {
            for (offset, byte) in value.to_le_bytes().into_iter().take(width).enumerate() {
                self.bytes.insert(address + offset, byte);
            }
        }
    }

    impl HostPoolIo for PoolRecorder {
        fn read_u32(&mut self, address: crate::dtcm::DtcmAddress) -> u32 {
            let value = self.load(address.get(), 4);
            self.accesses.push(PoolAccess { kind: AccessKind::Read, address: address.get(), width: 4, value });
            value
        }
        fn write_u8(&mut self, address: crate::dtcm::DtcmAddress, value: u8) {
            self.store(address.get(), 1, u32::from(value));
            self.accesses.push(PoolAccess { kind: AccessKind::Write, address: address.get(), width: 1, value: u32::from(value) });
        }
        fn write_u16(&mut self, address: crate::dtcm::DtcmAddress, value: u16) {
            self.store(address.get(), 2, u32::from(value));
            self.accesses.push(PoolAccess { kind: AccessKind::Write, address: address.get(), width: 2, value: u32::from(value) });
        }
        fn write_u32(&mut self, address: crate::dtcm::DtcmAddress, value: u32) {
            self.store(address.get(), 4, value);
            self.accesses.push(PoolAccess { kind: AccessKind::Write, address: address.get(), width: 4, value });
        }
    }

    struct AddressRecordingWriter {
        writes: std::vec::Vec<PoolAccess>,
    }

    impl HostContextWriter for AddressRecordingWriter {
        fn write_u8(&mut self, address: crate::dtcm::DtcmAddress, value: u8) {
            self.writes.push(PoolAccess { kind: AccessKind::Write, address: address.get(), width: 1, value: u32::from(value) });
        }
        fn write_u16(&mut self, address: crate::dtcm::DtcmAddress, value: u16) {
            self.writes.push(PoolAccess { kind: AccessKind::Write, address: address.get(), width: 2, value: u32::from(value) });
        }
        fn write_u32(&mut self, address: crate::dtcm::DtcmAddress, value: u32) {
            self.writes.push(PoolAccess { kind: AccessKind::Write, address: address.get(), width: 4, value });
        }
    }

    struct PowerSaveRecorder {
        observations: std::vec::Vec<Observation>,
        record: crate::dtcm::VifRecordAddress,
        sleeping: u16,
        frame_control: u16,
        link_gate: u8,
    }

    impl PowerSaveRecorder {
        fn sleeping_links(&mut self) -> u16 {
            self.observations.push(Observation {
                kind: ObservationKind::SleepingLinks,
                address: self.record.sleeping_links().get(),
                width: 2,
            });
            self.sleeping
        }

        fn frame_control(&mut self, address: usize) -> u16 {
            self.observations.push(Observation {
                kind: ObservationKind::FrameControl,
                address,
                width: 2,
            });
            self.frame_control
        }

        fn link_gate(&mut self) -> u8 {
            self.observations.push(Observation {
                kind: ObservationKind::LinkGate,
                address: self.record.link_gate().get(),
                width: 1,
            });
            self.link_gate
        }
    }

    #[test]
    fn power_save_observation_defers_link_gate_until_short_circuit_point() {
        let record = crate::dtcm::vif_record(0).unwrap();
        let frame_control_address = 0x0400_5a2e;
        let mut recorder = PowerSaveRecorder {
            observations: std::vec::Vec::new(),
            record,
            sleeping: 1,
            frame_control: 0,
            link_gate: 0,
        };
        let (_, _, released) = observe_normal_power_save_release!(
            recorder.sleeping_links(),
            recorder.frame_control(frame_control_address),
            recorder.link_gate(),
            1_u16,
            0_u8,
            0_u32
        );
        assert!(!released);
        assert_eq!(recorder.observations, [
            Observation { kind: ObservationKind::SleepingLinks, address: record.sleeping_links().get(), width: 2 },
            Observation { kind: ObservationKind::FrameControl, address: frame_control_address, width: 2 },
        ]);

        recorder.observations.clear();
        recorder.frame_control = 0x50;
        let (_, _, released) = observe_normal_power_save_release!(
            recorder.sleeping_links(),
            recorder.frame_control(frame_control_address),
            recorder.link_gate(),
            1_u16,
            0_u8,
            0_u32
        );
        assert!(released);
        assert_eq!(recorder.observations, [
            Observation { kind: ObservationKind::SleepingLinks, address: record.sleeping_links().get(), width: 2 },
            Observation { kind: ObservationKind::FrameControl, address: frame_control_address, width: 2 },
            Observation { kind: ObservationKind::LinkGate, address: record.link_gate().get(), width: 1 },
        ]);
    }

    #[test]
    fn host_context_addresses_preserve_pool_and_frame_state_identity() {
        let first = HostContextAddress::from_index(0).unwrap();
        let last = HostContextAddress::from_index(29).unwrap();

        assert_eq!(HostContextAddress::from_index(0), Some(first));
        assert_eq!(
            HostContextAddress::from_index(crate::dtcm::HOST_TX_CONTEXT_COUNT - 1),
            Some(last)
        );
        assert_eq!(first.raw(), crate::dtcm::HOST_TX_CONTEXTS.get() as u32);
        assert_eq!(first.expected_frame_state().raw(), packet_ram::host_frame_state(0) as u32);
        assert_eq!(last.raw(), crate::dtcm::HOST_TX_CONTEXTS.get() as u32 + 29 * 0x170);
        assert_eq!(last.expected_frame_state().raw(), packet_ram::host_frame_state(29) as u32);
        assert_eq!(HostContextAddress::from_raw(last.raw()), Some(last));
        assert_eq!(HostContextAddress::from_raw(last.raw() + 4), None);
        assert_eq!(HostContextAddress::from_index(crate::dtcm::HOST_TX_CONTEXT_COUNT), None);
    }

    #[test]
    fn host_free_list_rebuild_pop_and_push_preserve_exact_order_and_widths() {
        let mut io = PoolRecorder::new();
        assert!(io.enter());
        assert!(!io.enter());
        rebuild_host_free_list(&mut io);
        io.leave();

        let first = HostContextAddress::from_index(0).unwrap();
        let last = HostContextAddress::from_index(crate::dtcm::HOST_TX_CONTEXT_COUNT - 1).unwrap();
        assert_eq!(io.load(crate::dtcm::HOST_CONTEXT_FREE_HEAD.get(), 4), last.raw());
        assert_eq!(io.load(first.intrusive_next().get(), 4), 0);
        assert_eq!(io.load(last.intrusive_next().get(), 4), HostContextAddress::from_index(28).unwrap().raw());
        assert_eq!(io.accesses[0], PoolAccess {
            kind: AccessKind::Write,
            address: crate::dtcm::HOST_CONTEXT_FREE_HEAD.get(),
            width: 4,
            value: 0,
        });
        assert_eq!(io.accesses.last().copied(), Some(PoolAccess {
            kind: AccessKind::Write,
            address: crate::dtcm::HOST_CONTEXT_FREE_HEAD.get(),
            width: 4,
            value: last.raw(),
        }));

        io.accesses.clear();
        let popped = pop_host_free_list(&mut io).unwrap();
        assert_eq!(popped, last);
        assert_eq!(io.accesses, [
            PoolAccess { kind: AccessKind::Read, address: crate::dtcm::HOST_CONTEXT_FREE_HEAD.get(), width: 4, value: last.raw() },
            PoolAccess { kind: AccessKind::Read, address: last.frame_state_address().get(), width: 4, value: last.expected_frame_state().raw() },
            PoolAccess { kind: AccessKind::Read, address: last.intrusive_next().get(), width: 4, value: HostContextAddress::from_index(28).unwrap().raw() },
            PoolAccess { kind: AccessKind::Write, address: crate::dtcm::HOST_CONTEXT_FREE_HEAD.get(), width: 4, value: HostContextAddress::from_index(28).unwrap().raw() },
            PoolAccess { kind: AccessKind::Write, address: last.request_flags().get(), width: 1, value: 0 },
            PoolAccess { kind: AccessKind::Write, address: last.completion_status().get(), width: 4, value: 0xfe },
            PoolAccess { kind: AccessKind::Write, address: last.terminal_status().get(), width: 2, value: 0xfe },
            PoolAccess { kind: AccessKind::Write, address: last.ownership_bits().get(), width: 4, value: 0 },
        ]);

        io.accesses.clear();
        push_host_free_list(&mut io, popped);
        assert_eq!(io.accesses, [
            PoolAccess { kind: AccessKind::Write, address: last.completion_status().get(), width: 4, value: 0xff },
            PoolAccess { kind: AccessKind::Write, address: last.terminal_status().get(), width: 2, value: 0xff },
            PoolAccess { kind: AccessKind::Read, address: last.ownership_bits().get(), width: 4, value: 0 },
            PoolAccess { kind: AccessKind::Write, address: last.ownership_bits().get(), width: 4, value: 0x0004_0000 },
            PoolAccess { kind: AccessKind::Read, address: crate::dtcm::HOST_CONTEXT_FREE_HEAD.get(), width: 4, value: HostContextAddress::from_index(28).unwrap().raw() },
            PoolAccess { kind: AccessKind::Write, address: last.intrusive_next().get(), width: 4, value: HostContextAddress::from_index(28).unwrap().raw() },
            PoolAccess { kind: AccessKind::Write, address: crate::dtcm::HOST_CONTEXT_FREE_HEAD.get(), width: 4, value: last.raw() },
        ]);
    }

    #[test]
    fn host_free_list_rejects_empty_interior_and_corrupt_frame_state_pointers() {
        let first = HostContextAddress::from_index(0).unwrap();
        let mut io = PoolRecorder::new();
        assert_eq!(pop_host_free_list(&mut io), Err(HostPoolError::Empty));

        io.seed_u32(crate::dtcm::HOST_CONTEXT_FREE_HEAD, first.raw() + 4);
        assert_eq!(pop_host_free_list(&mut io), Err(HostPoolError::CorruptFreeHead));

        io.seed_u32(crate::dtcm::HOST_CONTEXT_FREE_HEAD, first.raw());
        io.seed_u32(first.frame_state_address(), 0x0900_0000);
        assert_eq!(pop_host_free_list(&mut io), Err(HostPoolError::CorruptFrameState));
    }

    #[test]
    fn context_initialization_records_field_derived_addresses_widths_and_order() {
        let context = HostContextAddress::from_index(3).unwrap();
        let metadata = HostTxMetadata {
            message_address: 0x0900_9000,
            packet_id: 0x1234_5678,
            max_tx_rate: 14,
            queue_id: 0x22,
            more: false,
            flags: 0x53,
            expire_time: 200,
            ht_tx_parameters: 1 | (0x60 << 11),
            frame_address: 0x0900_9018,
            frame_length: 147,
            interface: 1,
            submit_timer: 0x1020_3040,
            ac: 2,
            frame_state_address: context.expected_frame_state().raw(),
        };
        let mut writer = AddressRecordingWriter { writes: std::vec::Vec::new() };
        write_host_context_fields(&mut writer, context, metadata);
        assert_eq!(writer.writes[0], PoolAccess {
            kind: AccessKind::Write,
            address: context.frame_state_address().get(),
            width: 4,
            value: context.expected_frame_state().raw(),
        });
        assert_eq!(writer.writes[1..5], [
            PoolAccess { kind: AccessKind::Write, address: context.request_flags().get(), width: 1, value: 0 },
            PoolAccess { kind: AccessKind::Write, address: context.completion_status().get(), width: 4, value: 0xfe },
            PoolAccess { kind: AccessKind::Write, address: context.terminal_status().get(), width: 2, value: 0xfe },
            PoolAccess { kind: AccessKind::Write, address: context.ownership_bits().get(), width: 4, value: 0 },
        ]);
        let tail = writer.writes.last().copied().unwrap();
        assert_eq!(tail, PoolAccess {
            kind: AccessKind::Write,
            address: context.tx_rate().get(),
            width: 1,
            value: 14,
        });
        let ownership_publication = writer.writes.iter().position(|access| {
            access.address == context.ownership_bits().get() && access.value == 1
        }).unwrap();
        let request_identity = writer.writes.iter().position(|access| {
            access.address == context.request_buffer().get()
        }).unwrap();
        assert!(request_identity < ownership_publication);
    }

    #[test]
    fn host_tx_phases_cannot_skip_queue_or_scheduler_ownership() {
        assert!(valid_phase_transition(
            HostTxPhase::Submitted,
            HostTxPhase::Classified
        ));
        assert!(valid_phase_transition(
            HostTxPhase::PasQueued,
            HostTxPhase::SchedulerReserved
        ));
        assert!(valid_phase_transition(
            HostTxPhase::SchedulerReserved,
            HostTxPhase::Scheduled
        ));
        assert!(valid_phase_transition(
            HostTxPhase::Scheduled,
            HostTxPhase::PasQueued
        ));
        assert!(!valid_phase_transition(
            HostTxPhase::Submitted,
            HostTxPhase::Scheduled
        ));
        assert!(!valid_phase_transition(
            HostTxPhase::Completing,
            HostTxPhase::Submitted
        ));
    }

    #[test]
    fn header_classifier_preserves_qos_ack_policy_and_payload_split() {
        let mut frame = [0_u8; 40];
        frame[..2].copy_from_slice(&0x0188_u16.to_le_bytes());
        frame[4] = 0x02;
        frame[24..26].copy_from_slice(&0x0065_u16.to_le_bytes());

        let classification = classify_header(&frame, 0x0080_0000);
        assert_eq!(
            classification,
            Ok(HeaderClassification {
                frame_control: 0x0188,
                header_length: 26,
                payload_length: 14,
                qos_control: 0x0065,
                tid: 5,
                flags: 0x20f0_1201,
                assign_sequence: true,
            })
        );
    }

    #[test]
    fn header_classifier_accepts_a_compressed_block_ack_request() {
        // 20-octet control frame: FC, duration, RA, TA, BAR control, start
        // sequence. mac80211 encodes compressed TID 3 as 0x3004
        // (CBMTID_COMPRESSED_BA 0x0004 | tid << 12) and the ordinary-data path
        // would reject the frame as truncated.
        let mut frame = [0_u8; 20];
        frame[..2].copy_from_slice(&0x0084_u16.to_le_bytes());
        frame[4] = 0x02;
        frame[16..18].copy_from_slice(&0x3004_u16.to_le_bytes());
        frame[18..20].copy_from_slice(&0x0120_u16.to_le_bytes());

        assert_eq!(
            classify_header(&frame, 0),
            Ok(HeaderClassification {
                frame_control: 0x0084,
                header_length: 20,
                payload_length: 0,
                qos_control: 0,
                tid: 3,
                // Bit 14 marks the BlockAck response class, which is what a
                // BlockAckReq is answered with and what the host path retires on.
                flags: 0x5000,
                assign_sequence: false,
            })
        );
    }

    #[test]
    fn header_classifier_handles_four_address_ht_control() {
        let mut frame = [0_u8; 48];
        frame[..2].copy_from_slice(&0x8388_u16.to_le_bytes());
        frame[4] = 0x03;
        frame[30..32].copy_from_slice(&0x0002_u16.to_le_bytes());

        let classification = classify_header(&frame, 0);
        assert_eq!(classification.map(|value| value.header_length), Ok(36));
        assert_eq!(classification.map(|value| value.payload_length), Ok(12));
        assert_eq!(classification.map(|value| value.tid), Ok(2));
        assert_eq!(classification.map(|value| value.assign_sequence), Ok(false));
    }

    #[test]
    fn host_initializer_preserves_vendor_offsets() {
        let mut image = [0xaa; crate::dtcm::HOST_TX_CONTEXT_SIZE];
        initialize_host_context(
            &mut image,
            HostTxMetadata {
                message_address: 0x0900_9000,
                packet_id: 0x1234_5678,
                max_tx_rate: 14,
                queue_id: 0x22,
                more: false,
                flags: 0x53,
                expire_time: 200,
                ht_tx_parameters: 1 | (0x60 << 11),
                frame_address: 0x0900_9018,
                frame_length: 147,
                interface: 0,
                submit_timer: 0x1020_3040,
                ac: 2,
                frame_state_address: 0x0900_3678,
            },
        );

        assert_eq!(read_u32(&image, 0x00), 0x0900_9000);
        assert_eq!(read_u32(&image, 0x08), 0x1234_5678);
        assert_eq!(read_u32(&image, 0x10), 200);
        assert_eq!(read_u32(&image, 0x14), 1 | (0x60 << 11));
        assert_eq!(image[0x0c], 14);
        assert_eq!(image[0x0d], 2);
        assert_eq!(image[0x24], 14);
        assert_eq!(&image[0x25..0x34], &[0; 15]);
        assert_eq!(image[0xbf], 8);
        assert_eq!(image[0x53], 0);
        assert_eq!(image[0x52], 1);
        assert_eq!(image[0xa7], 1);
        assert_eq!(image[0x60], 2);
        assert_eq!(image[0x61], 1);
        assert_eq!(image[0x62], 5);
        assert_eq!(read_u16(&image, 0x5c), 147);
        assert_eq!(read_u16(&image, 0x70), 0xfe);
        assert_eq!(read_u32(&image, 0x54), 0x0900_9018);
        assert_eq!(read_u32(&image, 0xa0), 0x0900_3678);
        assert_eq!(read_u32(&image, 0x68), 0x1020_303f);
        assert_eq!(read_u32(&image, 0x58), 0x0081_0068);
    }

    #[test]
    fn station_data_rate_rejects_only_unsupported_indices() {
        assert_eq!(station_data_rate(0), 0);
        assert_eq!(station_data_rate(3), 3);
        assert_eq!(station_data_rate(4), 0);
        assert_eq!(station_data_rate(5), 0);
        assert_eq!(station_data_rate(6), 6);
        assert_eq!(station_data_rate(21), 21);
        assert_eq!(station_data_rate(u8::MAX), 0);
    }

    #[test]
    fn pending_mode_zero_appends_instead_of_prepending() {
        assert_eq!(
            append_pending_list(0x1000, 0x2000, 0x3000),
            PendingListInsertion {
                context_next: 0,
                new_head: 0x1000,
                new_tail: 0x3000,
            }
        );
        assert_eq!(append_pending_list(0, 0, 0x3000).new_head, 0x3000);
    }

    #[test]
    fn pending_removal_updates_head_tail_and_predecessor_separately() {
        assert_eq!(
            remove_pending_list_node(0x1000, 0x3000, 0, 0x1000, 0x2000),
            Some(PendingListRemoval {
                new_head: 0x2000,
                new_tail: 0x3000,
                prior_next: None,
            })
        );
        assert_eq!(
            remove_pending_list_node(0x1000, 0x3000, 0x2000, 0x3000, 0),
            Some(PendingListRemoval {
                new_head: 0x1000,
                new_tail: 0x2000,
                prior_next: Some(0),
            })
        );
    }

    #[test]
    fn host_pas_entries_append_and_holes_are_compacted() {
        let mut ring = PasRing::empty();
        assert_eq!(ring.push(0x1054, 1), Ok(()));
        assert_eq!(ring.push(0x2054, 1), Ok(()));
        ring.entries[1] = 0;
        assert_eq!(ring.push(0x3054, 1), Ok(()));

        assert_eq!(ring.entries[3], 0x2054);
        assert_eq!(ring.entries[4], 0x3054);
        assert_eq!(ring.head, 3);
        assert_eq!(ring.tail, 5);
    }

    #[test]
    fn pas_expiry_preserves_vendor_deadline_and_timer_wrap() {
        assert!(!pas_submission_expired(100, 100 + 0x007a_1200));
        assert!(pas_submission_expired(100, 101 + 0x007a_1200));
        assert!(!pas_submission_expired(u32::MAX - 10, 5));
    }

    #[test]
    fn non_aggregate_scheduler_requires_ring_pipe_and_idle_ownership() {
        let ready = NonAggregateSchedulerInput {
            expired: false,
            pipe_allowed: true,
            pipe: 2,
            idle_pipe_mask: 0b0100,
            ring_contains_frame: true,
        };
        assert_eq!(
            non_aggregate_scheduler_decision(ready),
            NonAggregateSchedulerDecision::ReservePipe(2)
        );
        assert_eq!(
            non_aggregate_scheduler_decision(NonAggregateSchedulerInput {
                idle_pipe_mask: 0,
                ..ready
            }),
            NonAggregateSchedulerDecision::LeaveQueued
        );
        assert_eq!(
            non_aggregate_scheduler_decision(NonAggregateSchedulerInput {
                expired: true,
                ..ready
            }),
            NonAggregateSchedulerDecision::Complete(10)
        );
    }

    #[test]
    fn requeued_retry_clears_scheduler_ownership_bits() {
        assert_eq!(
            requeued_retry_control_bits(0x1c08_0030),
            0x0008_0010
        );
    }

    #[test]
    fn ampdu_pair_requires_qos_data_and_one_grouping_key() {
        assert!(crate::configuration::retain_interface_mib(
            crate::wsm::MIB_ID_BLOCK_ACK_POLICY,
            &[0x3f, 0, 0x3f, 0],
        ));
        assert!(crate::configuration::retain_interface_mib(
            crate::wsm::MIB_ID_PRIVATE_TX_BA_SESSION,
            &[5, 1],
        ));
        let first = AmpduCandidate {
            key: AmpduGroupingKey { interface: 0, link: 2, tid: 5, rate: 19 },
            frame_control: 0x0188,
        };
        assert!(can_form_ampdu_pair(first, first));
        assert!(!can_form_ampdu_pair(
            first,
            AmpduCandidate { frame_control: 0x0008, ..first },
        ));
        assert!(!can_form_ampdu_pair(
            first,
            AmpduCandidate {
                key: AmpduGroupingKey { rate: 18, ..first.key },
                ..first
            },
        ));
        assert!(!can_form_ampdu_pair(
            first,
            AmpduCandidate {
                key: AmpduGroupingKey { tid: 4, ..first.key },
                ..first
            },
        ));
    }

    #[test]
    fn ordinary_batch_marks_first_and_later_descriptors_differently() {
        let original = 0x0000_1234;
        assert_eq!(scheduler_batch_control_bits(original, 0), original | 0x0400_0000);
        assert_eq!(scheduler_batch_control_bits(original, 1), original | 0x0800_0000);
        assert_eq!(scheduler_batch_control_bits(original, 3), original | 0x0800_0000);
    }

    #[test]
    fn pending_task_distinguishes_wait_reject_and_release() {
        let base = PendingTaskInput {
            global_blocked: false,
            active_link: true,
            link_gate_requests_removal: false,
            vif_operating: true,
            completion_class: 0,
            pipe_allowed: false,
            expired: false,
        };
        assert_eq!(
            pending_task_decision(base),
            PendingTaskDecision::LeaveQueued
        );
        assert_eq!(
            pending_task_decision(PendingTaskInput {
                active_link: false,
                ..base
            }),
            PendingTaskDecision::Complete(0x14)
        );
        assert_eq!(
            pending_task_decision(PendingTaskInput {
                pipe_allowed: true,
                ..base
            }),
            PendingTaskDecision::ReleaseToPas
        );
        assert_eq!(
            pending_task_decision(PendingTaskInput {
                expired: true,
                ..base
            }),
            PendingTaskDecision::Complete(10)
        );
    }
}
