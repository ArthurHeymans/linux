//! Vendor-shaped ordinary host-TX state transitions.
//!
//! This module contains pure representations of the decompiled WSM host-data
//! path. Hardware-facing code can apply these plans without borrowing the
//! internal class-6 probe/template initializer.

pub const HOST_CONTEXT_SIZE: usize = 0x170;
pub const HOST_CONTEXT_BASE: u32 = 0x0400_5a24;
pub const HOST_CONTEXT_COUNT: usize = 30;
pub const HOST_FREE_HEAD: u32 = 0x0400_87b0;
pub const HOST_IN_FLIGHT: u32 = 0x0400_3e9e;
pub const HOST_FRAME_STATE_BASE: u32 = 0x0900_3678;
pub const HOST_FRAME_STATE_SIZE: u32 = 0x54;
pub const PAS_OFFSET: usize = 0x54;
pub const PAS_RING_CAPACITY: usize = 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HostContextAddress(u32);

impl HostContextAddress {
    pub const fn from_index(index: usize) -> Option<Self> {
        if index < HOST_CONTEXT_COUNT {
            Some(Self(
                HOST_CONTEXT_BASE + index as u32 * HOST_CONTEXT_SIZE as u32,
            ))
        } else {
            None
        }
    }

    pub const fn from_raw(address: u32) -> Option<Self> {
        let offset = address.wrapping_sub(HOST_CONTEXT_BASE);
        if address >= HOST_CONTEXT_BASE
            && offset % HOST_CONTEXT_SIZE as u32 == 0
            && offset / (HOST_CONTEXT_SIZE as u32) < HOST_CONTEXT_COUNT as u32
        {
            Some(Self(address))
        } else {
            None
        }
    }

    pub const fn raw(self) -> u32 {
        self.0
    }

    pub const fn index(self) -> usize {
        ((self.0 - HOST_CONTEXT_BASE) / HOST_CONTEXT_SIZE as u32) as usize
    }

    pub const fn frame_state(self) -> u32 {
        HOST_FRAME_STATE_BASE + self.index() as u32 * HOST_FRAME_STATE_SIZE
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
            | (HostTxPhase::Scheduled, HostTxPhase::Completing)
    )
}

/// One borrowed HIF request and its class-0 context identity. The release
/// token deliberately stays here until confirmation or an explicit abort.
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
    pub unsafe fn cancel_before_pas(self) -> Result<crate::hif::RequestReleaseToken, CancelError> {
        match self.phase {
            HostTxPhase::Submitted | HostTxPhase::Classified | HostTxPhase::PendingEligible => {}
            HostTxPhase::PostCryptoQueued => unsafe { remove_pending_context(self.context)? },
            HostTxPhase::PasQueued => unsafe {
                remove_live_pas(self.context)?;
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
    /// original HIF request token back to the confirmation dispatcher.
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
    if frame.len() < 24 || frame.len() > u16::MAX as usize {
        return Err(HeaderClassificationError::Truncated);
    }
    let frame_control = u16::from_le_bytes([frame[0], frame[1]]);
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
    const VIF_BASE: u32 = 0x0400_3e98;
    const VIF_STRIDE: u32 = 0x3b0;
    const LINK_MAP_OFFSET: u32 = 0x4920;
    const LINK_MAP_COUNT: u32 = 0x0400_87cc;
    const SEQUENCE_BASE: u32 = 0x0400_8890;

    let interface = unsafe { read_live_u8(context.raw() + 0xbd) };
    let host_link = unsafe { read_live_u8(context.raw() + 0xbf) };
    let vif = VIF_BASE + u32::from(interface) * VIF_STRIDE;
    let mut internal_link = unsafe { read_live_u8(vif + 0x12a) };
    if host_link != 0 {
        let count = unsafe { (LINK_MAP_COUNT as *const u16).read_volatile() };
        let mut index = 0_u16;
        while index < count {
            let entry = VIF_BASE + LINK_MAP_OFFSET + u32::from(index) * 0x0c;
            if unsafe { read_live_u8(entry + 0x19) } == interface
                && unsafe { read_live_u8(entry + 0x18) } == host_link
            {
                internal_link = unsafe { read_live_u8(entry + 0x1a) };
                break;
            }
            index += 1;
        }
    }
    let sequence_address = SEQUENCE_BASE + u32::from(internal_link) * 0x20 + u32::from(tid) * 2;
    let sequence = unsafe { (sequence_address as *const u16).read_volatile() };
    unsafe {
        (frame_address.wrapping_add(0x16) as *mut u16).write_volatile(sequence);
        write_live_u16(context.raw() + 0xa8, sequence >> 4);
        (sequence_address as *mut u16).write_volatile(sequence.wrapping_add(0x10) & 0xfff0);
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
    let frame_address = unsafe { read_live_u32(context.raw() + 0x54) };
    let frame_length = unsafe { (context.raw().wrapping_add(0x5c) as *const u16).read_volatile() };
    let frame = unsafe {
        core::slice::from_raw_parts_mut(frame_address as *mut u8, usize::from(frame_length))
    };
    let initial_flags = unsafe { read_live_u32(context.raw() + 0x58) };
    let classification = classify_header(frame, initial_flags).map_err(HostPrepareError::Header)?;

    if classification.assign_sequence {
        unsafe { assign_sequence_number(context, frame_address, classification.tid) };
    }
    let interface = unsafe { read_live_u8(context.raw() + 0xbd) };
    unsafe {
        write_live_u16(context.raw() + 0x5e, classification.frame_control);
        write_live_u32(
            context.raw() + 0x44,
            u32::from(classification.header_length),
        );
        write_live_u32(
            context.raw() + 0x48,
            u32::from(classification.payload_length),
        );
        write_live_u32(context.raw() + 0x58, classification.flags);
        write_live_u16(context.raw() + 0xc8, classification.qos_control);
        write_live_u8(context.raw() + 0xa6, classification.tid);
        write_live_u8(context.raw() + 0xaa, 0xff);
        let vif_slot = read_live_u8(0x0400_3ae9 + u32::from(interface) * 0x98) & 1;
        write_live_u8(context.raw() + 0xbe, vif_slot);
        crate::host_tx_diagnostics::capture_submission_identity(
            retained.packet_id,
            context.raw(),
            read_live_u8(context.raw() + 0x0f),
            read_live_u8(context.raw() + 0x61),
            read_live_u8(context.raw() + 0x60),
            classification.flags,
        );
    }
    retained.phase = HostTxPhase::Classified;
    unsafe { crate::host_tx_diagnostics::bump(crate::host_tx_diagnostics::counter::ADMITTED) };
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
    let context = retained.context.raw();
    let status = unsafe { (context.wrapping_add(0x70) as *const u16).read_volatile() };
    if status < 0x00fe {
        return Err(PostCryptoError::ExistingStatus(status));
    }

    // The normal unicast station path leaves this optional per-peer object
    // disabled. Do not silently skip its allocation for multicast or when the
    // vendor global enables the alternate object class.
    let flags = unsafe { read_live_u32(context + 0x58) };
    let alternate_enabled = unsafe { (0x0400_3a6a as *const u16).read_volatile() != 0 };
    if flags & 0x100 != 0 || alternate_enabled {
        return Err(PostCryptoError::OptionalPipeObjectRequired);
    }
    unsafe { write_live_u32(context + 0x4c, 0) };
    unsafe { crate::tx::build_host_frame_descriptor(context) }
        .map_err(PostCryptoError::Descriptor)?;

    let previous = unsafe { crate::tx::disable_irq_fiq_save() };
    unsafe {
        const PENDING: u32 = 0x0400_8ad8;
        let tail = read_live_u32(PENDING + 4);
        write_live_u32(context + 4, 0);
        if tail == 0 {
            write_live_u32(PENDING, context);
        } else {
            write_live_u32(tail + 4, context);
        }
        write_live_u32(PENDING + 4, context);
        write_live_u32(context + 0x80, read_live_u32(context + 0x80) | 0x20);
        if read_live_u8(context + 0x0e) == 0 {
            const SCHEDULER_EVENTS: u32 = 0x0400_1fd4;
            write_live_u32(
                SCHEDULER_EVENTS,
                read_live_u32(SCHEDULER_EVENTS) | 0x0020_0000,
            );
        }
        crate::tx::restore_irq_fiq_saved(previous);
    }
    retained.phase = HostTxPhase::PostCryptoQueued;
    Ok(())
}

#[cfg(target_arch = "arm")]
unsafe fn remove_pending_context(context: HostContextAddress) -> Result<(), CancelError> {
    const PENDING: u32 = 0x0400_8ad8;
    let previous = unsafe { crate::tx::disable_irq_fiq_save() };
    let mut prior = 0_u32;
    let mut current = unsafe { read_live_u32(PENDING) };
    while current != 0 && current != context.raw() {
        prior = current;
        current = unsafe { read_live_u32(current + 4) };
    }
    if current == 0 {
        unsafe { crate::tx::restore_irq_fiq_saved(previous) };
        return Err(CancelError::MissingFromPendingList);
    }
    let next = unsafe { read_live_u32(current + 4) };
    unsafe {
        if prior == 0 {
            write_live_u32(PENDING, next);
        } else {
            write_live_u32(prior + 4, next);
        }
        if read_live_u32(PENDING + 4) == current {
            write_live_u32(PENDING + 4, prior);
        }
        write_live_u32(current + 4, 0);
        crate::tx::restore_irq_fiq_saved(previous);
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
    unsafe { read_live_u32(0x0ac0_0004).wrapping_add(read_live_u32(0x0400_143c)) }
}

/// Exact boolean/side-effect translation of `txp_program_pipe_hw(pas, 0)`.
/// Vendor return value zero means blocked; nonzero means the pending task may
/// release the frame toward PAS scheduling.
#[cfg(target_arch = "arm")]
unsafe fn program_pipe_eligible(context: HostContextAddress) -> bool {
    const PAS_STATE_BASE: u32 = 0x0400_3678;
    const VIF_BASE: u32 = 0x0400_3e98;
    const LINK_STATE: u32 = 0x0400_87b8;
    let pas = context.raw() + PAS_OFFSET as u32;
    let interface = unsafe { read_live_u8(pas + 0x69) };
    if interface > 2 {
        return true;
    }
    let pas_state = PAS_STATE_BASE + u32::from(interface) * 0x98;
    let vif = VIF_BASE + u32::from(interface) * 0x3b0;
    let vif_flags = unsafe { read_live_u32(vif + 0x1c) };

    let blocked = if unsafe { read_live_u8(pas_state + 0x492) } != 0 {
        true
    } else if unsafe { read_live_u8(pas_state + 0x493) } == 0 {
        vif_flags & (1 << 29) != 0 && vif_flags & 3 == 3
    } else {
        let tsf =
            unsafe { read_live_u32(0x09c0_0e38).wrapping_add(read_live_u32(pas_state + 0x488)) };
        let until_tbtt = unsafe { read_live_u32(pas_state + 0x474) }.wrapping_sub(tsf) as i32;
        let duration = u32::from(unsafe { read_live_u16(pas + 0x30) })
            + u32::from(unsafe { read_live_u16(pas + 0x34) })
            + u32::from(unsafe { read_live_u16(pas + 0x38) })
            + u32::from(unsafe { read_live_u16(pas + 0x36) });
        until_tbtt < 1 || until_tbtt <= duration as i32
    };
    let policy = unsafe { read_live_u8(pas + 0x0e) };
    let global = unsafe { read_live_u32(0x0400_1fcc) };
    if (policy != 0x0f || global & 0x80 == 0) && blocked {
        return false;
    }
    if vif_flags & 4 == 0 {
        return true;
    }

    let link = unsafe { read_live_u8(pas + 0x6b) };
    let link_bit = 1_u16.wrapping_shl(u32::from(link));
    let sleeping = unsafe { read_live_u16(vif + 0x15c) };
    let frame_control = unsafe { read_live_u16(pas + 0x0a) };
    let frame_kind = frame_control & 0xff;
    let normal_release = ((sleeping & link_bit == 0)
        || frame_kind == 0x50
        || (frame_kind == 0xd0 && policy == 0x0f))
        && ((link_bit & 1 == 0) || sleeping == 0 || unsafe { read_live_u8(vif + 0x164) } == 0)
        && vif_flags & (1 << 29) == 0;
    if normal_release {
        return true;
    }

    let awake = unsafe { read_live_u16(vif + 0x15e) };
    let buffered = unsafe { read_live_u16(vif + 0x160) };
    if awake & link_bit == 0 && buffered & link_bit == 0 {
        if unsafe { read_live_u16(LINK_STATE + 0x16) } & link_bit != 0 {
            return false;
        }
        if unsafe { read_live_u16(vif + 0x2c) } & link_bit == 0 {
            return false;
        }
        let count = unsafe { read_live_u16(LINK_STATE + 0x14) };
        let mut index = 0_u16;
        while index < count {
            let entry = LINK_STATE + u32::from(index) * 0x0c;
            if unsafe { read_live_u8(pas + 0x6b) } == unsafe { read_live_u8(entry + 0x18) } {
                unsafe {
                    write_live_u8(entry + 0x1c, read_live_u8(entry + 0x1c) | 2);
                    write_live_u16(
                        LINK_STATE + 0x16,
                        read_live_u16(LINK_STATE + 0x16) | link_bit,
                    );
                }
            }
            index += 1;
        }
        return false;
    }

    if awake & link_bit != 0 && buffered & link_bit != 0 && frame_control & 0x80 != 0 {
        let header = unsafe { read_live_u32(pas) };
        let qos_control = unsafe { read_live_u16(header + 0x18) };
        if qos_control & 0x10 == 0 {
            return true;
        }
        unsafe { write_live_u16(vif + 0x160, buffered & !link_bit) };
        return true;
    }

    let new_awake = awake & !link_bit;
    unsafe {
        write_live_u16(vif + 0x15e, new_awake);
        if read_live_u16(vif + 0x2e) != 0 {
            write_live_u16(
                vif + 0x2e,
                (!sleeping | read_live_u16(vif + 0x160) | new_awake) & read_live_u16(vif + 0x2c),
            );
        }
    }
    true
}

#[cfg(target_arch = "arm")]
unsafe fn push_live_pas(context: HostContextAddress) -> Result<(), PendingServiceError> {
    const RING: u32 = 0x0400_1578;
    let previous = unsafe { crate::tx::disable_irq_fiq_save() };
    let old_tail = unsafe { read_live_u32(RING + 4) as u8 & 0x3f };
    let mut scan = unsafe { read_live_u32(RING) as u8 & 0x3f };
    let mut write = old_tail;
    while scan != old_tail {
        let slot = RING + 8 + u32::from(scan) * 4;
        let value = unsafe { read_live_u32(slot) };
        if value != 0 {
            unsafe {
                write_live_u32(RING + 8 + u32::from(write) * 4, value);
                write_live_u32(slot, 0);
            }
            write = write.wrapping_add(1) & 0x3f;
        }
        scan = scan.wrapping_add(1) & 0x3f;
    }
    unsafe {
        write_live_u32(RING, u32::from(old_tail));
        write_live_u32(RING + 4, u32::from(write));
    }
    let next = write.wrapping_add(1) & 0x3f;
    if next == old_tail {
        unsafe { crate::tx::restore_irq_fiq_saved(previous) };
        return Err(PendingServiceError::PasRingFull);
    }
    unsafe {
        write_live_u32(
            RING + 8 + u32::from(write) * 4,
            context.raw() + PAS_OFFSET as u32,
        );
        write_live_u32(RING + 4, u32::from(next));
        crate::tx::restore_irq_fiq_saved(previous);
    }
    Ok(())
}

#[cfg(target_arch = "arm")]
unsafe fn remove_live_pas(context: HostContextAddress) -> Result<(), CancelError> {
    const RING: u32 = 0x0400_1578;
    let target = context.raw() + PAS_OFFSET as u32;
    let previous = unsafe { crate::tx::disable_irq_fiq_save() };
    let head = unsafe { read_live_u32(RING) as u8 & 0x3f };
    let tail = unsafe { read_live_u32(RING + 4) as u8 & 0x3f };
    let mut scan = head;
    let mut found = false;
    while scan != tail {
        let slot = RING + 8 + u32::from(scan) * 4;
        if unsafe { read_live_u32(slot) } == target {
            unsafe { write_live_u32(slot, 0) };
            found = true;
            break;
        }
        scan = scan.wrapping_add(1) & 0x3f;
    }
    if !found {
        unsafe { crate::tx::restore_irq_fiq_saved(previous) };
        return Err(CancelError::MissingFromPasRing);
    }
    // Leave the hole for the vendor compaction step performed by the next PAS
    // insertion. This matches normal completion/removal ring semantics.
    unsafe { crate::tx::restore_irq_fiq_saved(previous) };
    Ok(())
}

#[cfg(target_arch = "arm")]
unsafe fn claim_pas_accounting(context: HostContextAddress) -> bool {
    unsafe {
        let active = read_live_u16(0x0400_8f76);
        if active == 0 && !crate::phy::advance_awake_station_tx() {
            return false;
        }
        write_live_u16(0x0400_8f76, active.wrapping_add(1));
        let interface = read_live_u8(context.raw() + 0xbd);
        if interface < 3 {
            let vif_active = 0x0400_3e98 + u32::from(interface) * 0x3b0 + 0x30;
            write_live_u16(vif_active, read_live_u16(vif_active).wrapping_add(1));
        }
        true
    }
}

#[cfg(target_arch = "arm")]
unsafe fn release_pas_accounting(context: HostContextAddress) {
    unsafe {
        write_live_u16(0x0400_8f76, read_live_u16(0x0400_8f76).wrapping_sub(1));
        let interface = read_live_u8(context.raw() + 0xbd);
        if interface < 3 {
            let vif_active = 0x0400_3e98 + u32::from(interface) * 0x3b0 + 0x30;
            write_live_u16(vif_active, read_live_u16(vif_active).wrapping_sub(1));
        }
    }
}

#[cfg(target_arch = "arm")]
unsafe fn release_pending_to_pas(retained: &mut RetainedHostTx) -> Result<(), PendingServiceError> {
    let context = retained.context;
    unsafe {
        write_live_u32(
            context.raw() + 0x80,
            read_live_u32(context.raw() + 0x80) | 0x40,
        );
        crate::tx::prepare_host_frame_timing(context.raw()).map_err(PendingServiceError::Timing)?;
        if !claim_pas_accounting(context) {
            return Err(PendingServiceError::PhyState);
        }
        if let Err(error) = push_live_pas(context) {
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
    pub vif_flags: u32,
    pub vif_state: u8,
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
    let interface = unsafe { read_live_u8(context.raw() + 0xbd) };
    let link = unsafe { read_live_u8(context.raw() + 0xbf) };
    let vif = 0x0400_3e98 + u32::from(interface) * 0x3b0;
    PendingLiveDiagnostic {
        global: unsafe { read_live_u32(0x0400_1fcc) },
        active_mask: unsafe { read_live_u16(vif + 0x2c) },
        effective_mask: unsafe { read_live_u16(vif + 0x2e) },
        vif_flags: unsafe { read_live_u32(vif + 0x1c) },
        vif_state: unsafe { read_live_u8(vif + 0x18) },
        interface,
        link,
        pipe_allowed: unsafe { program_pipe_eligible(context) },
    }
}

#[cfg(target_arch = "arm")]
pub unsafe fn service_pending(
    retained: &mut RetainedHostTx,
) -> Result<PendingServiceReport, PendingServiceError> {
    if retained.phase == HostTxPhase::PendingEligible {
        unsafe { release_pending_to_pas(retained)? };
        return Ok(PendingServiceReport::PasQueued);
    }
    if retained.phase != HostTxPhase::PostCryptoQueued {
        return Err(PendingServiceError::WrongPhase);
    }

    let context = retained.context;
    let interface = unsafe { read_live_u8(context.raw() + 0xbd) };
    let link = unsafe { read_live_u8(context.raw() + 0xbf) };
    let vif = 0x0400_3e98 + u32::from(interface) * 0x3b0;
    let link_bit = 1_u16.wrapping_shl(u32::from(link));
    let completion_class = unsafe { read_live_u8(context.raw() + 0x53) };
    let global_blocked = unsafe { read_live_u32(0x0400_1fcc) } & 0xa0 != 0;
    let now = unsafe { vendor_timer() };
    let submitted = unsafe { read_live_u32(context.raw() + 0x40) };
    let expired = (submitted.wrapping_sub(now).wrapping_add(0x004c_4b40) as i32) < 0;

    let mut decision_input = PendingTaskInput {
        global_blocked,
        active_link: true,
        link_gate_requests_removal: false,
        vif_operating: false,
        completion_class,
        pipe_allowed: false,
        expired,
    };
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
        let active_mask = unsafe { read_live_u16(vif + 0x2c) };
        let active_link = active_mask & link_bit != 0;
        let mut effective_mask = unsafe { read_live_u16(vif + 0x2e) };
        let mut effective_link = effective_mask & link_bit != 0;
        let frame_kind = unsafe { read_live_u16(context.raw() + 0x5e) } & 0xff;
        let vif_flags = unsafe { read_live_u32(vif + 0x1c) };
        if !effective_link && frame_kind != 0xd0 && effective_mask == 0 {
            if vif_flags & (1 << 30) != 0 {
                unsafe { write_live_u32(vif + 0x1c, vif_flags | 0x0400_0000) };
            } else {
                // `vif_resume_tx_after_radio`: the joined runtime already owns
                // the radio, so reproduce its effective-link publication.
                let radio_state = unsafe { read_live_u8(vif + 0x66) };
                if radio_state == 2 || radio_state == 3 {
                    effective_mask = active_mask;
                    if interface < 2 {
                        let sleeping = unsafe { read_live_u16(vif + 0x15c) };
                        let buffered = unsafe { read_live_u16(vif + 0x160) };
                        let awake = unsafe { read_live_u16(vif + 0x15e) };
                        effective_mask = (active_mask & (!sleeping | buffered)) | awake;
                    }
                    unsafe { write_live_u16(vif + 0x2e, effective_mask) };
                    effective_link = effective_mask & link_bit != 0;
                }
            }
        }
        let link_gate_requests_removal =
            (effective_link || frame_kind == 0xd0) && vif_flags & (1 << 29) == 0;
        if (effective_link || frame_kind == 0xd0) && vif_flags & (1 << 29) != 0 {
            unsafe { write_live_u32(vif + 0x1c, vif_flags | 0x0400_0000) };
        }
        let state = unsafe { read_live_u8(vif + 0x18) };
        decision_input = PendingTaskInput {
            global_blocked: false,
            active_link,
            link_gate_requests_removal,
            vif_operating: state == 4 || state == 6,
            completion_class,
            pipe_allowed: unsafe { program_pipe_eligible(context) },
            expired,
        };
        pending_task_decision(decision_input)
    };

    match decision {
        PendingTaskDecision::LeaveQueued => Ok(PendingServiceReport::LeaveQueued),
        PendingTaskDecision::Complete(status) => {
            unsafe { remove_pending_context(context).map_err(PendingServiceError::PendingList)? };
            retained.phase = HostTxPhase::PendingEligible;
            Ok(PendingServiceReport::Complete(status))
        }
        PendingTaskDecision::ReleaseToPas => {
            unsafe { remove_pending_context(context).map_err(PendingServiceError::PendingList)? };
            retained.phase = HostTxPhase::PendingEligible;
            unsafe { release_pending_to_pas(retained)? };
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

/// Snapshot the non-aggregate scheduler gates without changing ownership.
///
/// # Safety
/// The retained PAS context and global ring/pipe state must remain mapped.
#[cfg(target_arch = "arm")]
pub unsafe fn scheduler_live_diagnostic(retained: &RetainedHostTx) -> SchedulerLiveDiagnostic {
    let context = retained.context;
    let pas = context.raw() + PAS_OFFSET as u32;
    let mut idle_pipe_mask = 0_u8;
    for candidate in 0..4_u8 {
        let pipe_state = 0x0400_1720 + u32::from(candidate) * 0x6c;
        if unsafe { read_live_u8(pipe_state + 3) } == 0 {
            idle_pipe_mask |= 1 << candidate;
        }
    }
    let ac = unsafe { read_live_u8(pas + 0x0c) };
    let pipe = unsafe { read_live_u8(0x0400_02e0 + u32::from(ac)) };
    let ring_head = unsafe { read_live_u32(0x0400_1578) as u8 & 0x3f };
    let ring_tail = unsafe { read_live_u32(0x0400_157c) as u8 & 0x3f };
    let mut slot = ring_head;
    while slot != ring_tail && unsafe { read_live_u32(0x0400_1580 + u32::from(slot) * 4) } != pas {
        slot = slot.wrapping_add(1) & 0x3f;
    }
    SchedulerLiveDiagnostic {
        pipe,
        idle_pipe_mask,
        ring_head,
        ring_tail,
        ring_contains_frame: slot != ring_tail,
        pipe_allowed: unsafe { program_pipe_eligible(context) },
        retry_gate: unsafe { read_live_u8(0x0400_1e6c) },
        receive_gate: unsafe { read_live_u8(0x0400_3a6d) },
    }
}

/// Remove an unscheduled PAS frame for class-0 rejection/expiry confirmation.
///
/// # Safety
/// The frame must still be present in the global PAS ring and own no pipe slot.
#[cfg(target_arch = "arm")]
pub unsafe fn reject_unscheduled_pas(retained: &mut RetainedHostTx) -> Result<(), CancelError> {
    if retained.phase != HostTxPhase::PasQueued {
        return Err(CancelError::HardwareOwned);
    }
    unsafe {
        remove_live_pas(retained.context)?;
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
    original_flags: u32,
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
    /// single-frame path; `First`/`Middle` write the slot without arming, and
    /// `Last` arms the pipe for every slot staged since `First`.
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
        unsafe {
            crate::host_tx_diagnostics::bump(crate::host_tx_diagnostics::counter::PUBLISHED);
        }
        retained.phase = HostTxPhase::Scheduled;
        Ok(())
    }

    /// Restore software ownership before any pipe producer or hardware trigger.
    ///
    /// # Safety
    /// The reservation must not have been published.
    pub unsafe fn cancel(self, retained: &mut RetainedHostTx) -> bool {
        if retained.context != self.context || retained.phase != HostTxPhase::SchedulerReserved {
            return false;
        }
        unsafe {
            write_live_u32(self.context.raw() + 0x58, self.original_flags);
            write_live_u32(self.slot_record, self.original_slot_header);
            write_live_u32(self.slot_record + 0x0c, self.original_slot_frame);
            write_live_u32(self.slot_record + 0x10, self.original_slot_auxiliary);
            for (index, word) in self.original_command.into_iter().enumerate() {
                write_live_u32(self.command + index as u32 * 4, word);
            }
            write_live_u32(
                0x0400_1580 + u32::from(self.ring_slot) * 4,
                self.context.raw() + PAS_OFFSET as u32,
            );
            write_live_u32(0x0400_1578, u32::from(self.original_ring_head));
        }
        retained.phase = HostTxPhase::PasQueued;
        true
    }
}

/// Select one PAS-ring frame, reserve its mapped idle pipe slot, and emit the
/// kind-0 descriptor without starting hardware.
///
/// # Safety
/// Global PAS and pipe state must be exclusively runtime-owned.
const fn scheduler_batch_flags(original: u32, staged: u8) -> u32 {
    original
        | if staged == 0 {
            0x0400_0000
        } else {
            0x0800_0000
        }
}

#[cfg(target_arch = "arm")]
pub unsafe fn reserve_non_aggregate_scheduler(
    _guard: &mut crate::mac_domain::MacDomainGuard<'_>,
    retained: &mut RetainedHostTx,
) -> Result<HostSchedulerReservation, SchedulerReserveError> {
    if retained.phase != HostTxPhase::PasQueued {
        return Err(SchedulerReserveError::WrongPhase);
    }
    if unsafe { read_live_u8(0x0400_1e6c) } != 0 || unsafe { read_live_u8(0x0400_3a6d) } != 0 {
        return Err(SchedulerReserveError::SchedulerBlocked);
    }

    let context = retained.context;
    let pas = context.raw() + PAS_OFFSET as u32;
    let mut idle_pipe_mask = 0_u8;
    let mut candidate = 0_u8;
    while candidate < 4 {
        let pipe_state = 0x0400_1720 + u32::from(candidate) * 0x6c;
        if unsafe { read_live_u8(pipe_state + 3) } == 0 {
            idle_pipe_mask |= 1 << candidate;
        }
        candidate += 1;
    }
    let ac = unsafe { read_live_u8(pas + 0x0c) };
    let pipe = unsafe { read_live_u8(0x0400_02e0 + u32::from(ac)) };
    let head = unsafe { read_live_u32(0x0400_1578) as u8 & 0x3f };
    let tail = unsafe { read_live_u32(0x0400_157c) as u8 & 0x3f };
    let mut ring_slot = head;
    while ring_slot != tail
        && unsafe { read_live_u32(0x0400_1580 + u32::from(ring_slot) * 4) } != pas
    {
        ring_slot = ring_slot.wrapping_add(1) & 0x3f;
    }
    let now = unsafe { vendor_timer() };
    let submitted = unsafe { read_live_u32(context.raw() + 0x40) };
    let decision = non_aggregate_scheduler_decision(NonAggregateSchedulerInput {
        expired: (submitted.wrapping_sub(now).wrapping_add(0x007a_1200) as i32) < 0,
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

    let pipe_state = 0x0400_1720 + u32::from(pipe) * 0x6c;
    let slot = unsafe { read_live_u8(pipe_state) } & 3;
    let slot_record = pipe_state + 0x0c + u32::from(slot) * 0x18;
    let command = unsafe { read_live_u32(slot_record + 0x14) };
    let hardware_ring = unsafe { read_live_u32(pipe_state + 8) };
    if command == 0 || hardware_ring == 0 {
        return Err(SchedulerReserveError::PipeStateUnavailable);
    }
    #[cfg(all(feature = "vendor-host-tx-diagnostics", target_arch = "arm"))]
    unsafe {
        crate::hif::validate_tx_boundary(0x10, pipe, slot, command, hardware_ring);
    }
    let original_flags = unsafe { read_live_u32(context.raw() + 0x58) };
    let original_slot_header = unsafe { read_live_u32(slot_record) };
    let original_slot_frame = unsafe { read_live_u32(slot_record + 0x0c) };
    let original_slot_auxiliary = unsafe { read_live_u32(slot_record + 0x10) };
    let mut original_command = [0_u32; 16];
    for (index, word) in original_command.iter_mut().enumerate() {
        *word = unsafe { read_live_u32(command + index as u32 * 4) };
    }

    unsafe {
        write_live_u32(0x0400_1580 + u32::from(ring_slot) * 4, 0);
        let mut new_head = head;
        while new_head != tail && read_live_u32(0x0400_1580 + u32::from(new_head) * 4) == 0 {
            new_head = new_head.wrapping_add(1) & 0x3f;
        }
        write_live_u32(0x0400_1578, u32::from(new_head));
        // Vendor marks the first ordinary descriptor with bit 26 and every
        // later descriptor in the same scheduler batch with bit 27 before
        // `txp_build_pipe_descriptor(..., 0)`.
        write_live_u32(
            context.raw() + 0x58,
            scheduler_batch_flags(original_flags, 0),
        );
        write_live_u32(
            context.raw() + 0x80,
            read_live_u32(context.raw() + 0x80) | 0x100,
        );
        write_live_u32(context.raw() + 0x6c, vendor_timer());
        write_live_u32(context.raw() + 0x90, 0);
        write_live_u8(slot_record, 0);
        write_live_u8(slot_record + 1, read_live_u8(context.raw() + 0xaa));
        write_live_u8(slot_record + 2, 0);
        write_live_u8(slot_record + 3, 0);
        write_live_u32(slot_record + 0x0c, pas);
        // `txp_build_pipe_descriptor` clears the per-slot auxiliary descriptor
        // pointer for every kind-0 frame before emitting its command stream.
        // This storage is retained DTCM and cannot be left at its prior value.
        write_live_u32(slot_record + 0x10, 0);
        write_live_u32(command, 0);
        write_live_u32(command + 4, 0);
        write_live_u32(command + 8, 0xdc00_0000);
        if let Err(error) = crate::tx::emit_host_frame_descriptor_at(context.raw(), command + 0x0c)
        {
            write_live_u32(0x0400_1580 + u32::from(ring_slot) * 4, pas);
            write_live_u32(0x0400_1578, u32::from(head));
            write_live_u32(context.raw() + 0x58, original_flags);
            write_live_u32(slot_record, original_slot_header);
            write_live_u32(slot_record + 0x0c, original_slot_frame);
            write_live_u32(slot_record + 0x10, original_slot_auxiliary);
            for (index, word) in original_command.into_iter().enumerate() {
                write_live_u32(command + index as u32 * 4, word);
            }
            return Err(SchedulerReserveError::Descriptor(error));
        }
        #[cfg(all(feature = "vendor-host-tx-diagnostics", target_arch = "arm"))]
        crate::hif::validate_tx_boundary(0x11, pipe, slot, command, hardware_ring);
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
        original_flags,
        original_slot_header,
        original_slot_frame,
        original_slot_auxiliary,
        original_command,
    })
}

pub trait HostContextWriter {
    fn write_u8(&mut self, offset: usize, value: u8);
    fn write_u16(&mut self, offset: usize, value: u16);
    fn write_u32(&mut self, offset: usize, value: u32);
}

/// Emit only fields written by `tx_wsm_buf_alloc`, `wsm_h_04_tx_req`, and the
/// non-policy portion of `tx_lmac_req_submit`. Pool-owned and unknown fields
/// remain untouched in a live context.
pub fn write_host_context_fields<W: HostContextWriter>(writer: &mut W, metadata: HostTxMetadata) {
    writer.write_u32(0xa0, metadata.frame_state_address);

    writer.write_u8(0x0f, 0);
    writer.write_u32(0x20, 0xfe);
    writer.write_u16(0x70, 0xfe);
    writer.write_u32(0x80, 0);

    writer.write_u32(0x00, metadata.message_address);
    writer.write_u32(0x08, metadata.packet_id);
    writer.write_u8(0x0c, metadata.max_tx_rate);
    writer.write_u8(0x0d, metadata.queue_id);
    writer.write_u8(0x0e, u8::from(metadata.more));
    writer.write_u8(0x0f, metadata.flags);
    writer.write_u32(0x10, metadata.expire_time);
    writer.write_u32(0x14, metadata.ht_tx_parameters);
    writer.write_u32(0x18, u32::from(metadata.frame_length));
    writer.write_u32(0x1c, metadata.frame_address);
    writer.write_u8(0x24, metadata.max_tx_rate);
    writer.write_u8(0x25, 0);
    writer.write_u16(0x26, 0);
    writer.write_u32(0x28, 0);
    writer.write_u32(0x2c, 0);
    writer.write_u32(0x30, 0);

    writer.write_u32(0x80, 1);
    writer.write_u8(0xbd, metadata.interface);
    writer.write_u8(0xbf, (metadata.queue_id & 0x3f) >> 2);
    writer.write_u8(0x0d, metadata.queue_id & 3);
    writer.write_u32(0x40, metadata.submit_timer);
    writer.write_u8(0x53, 0);
    writer.write_u16(0x50, 0);
    writer.write_u8(0x52, 1);
    writer.write_u32(0x4c, 0);
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
    writer.write_u32(0x58, flags);
    writer.write_u32(0x74, 0);
    writer.write_u32(0x78, 0);
    writer.write_u32(0x7c, 0);
    writer.write_u32(0x68, metadata.submit_timer.wrapping_sub(1));
    writer.write_u32(0x6c, metadata.submit_timer.wrapping_sub(1));
    writer.write_u32(0x38, 0);
    writer.write_u32(0x3c, 0);
    writer.write_u32(0x64, metadata.expire_time);
    writer.write_u32(0x54, metadata.frame_address);
    writer.write_u8(0x60, metadata.ac);
    writer.write_u8(0x61, (metadata.flags & 0x0f) >> 1);
    writer.write_u8(0x62, (metadata.flags & 0x7f) >> 4);
    writer.write_u16(0x5c, metadata.frame_length);
    writer.write_u16(0x70, 0xfe);
    writer.write_u16(0x72, 0);
    writer.write_u16(0xa4, 0);
    writer.write_u8(0xa7, 1);
    writer.write_u32(0x90, 0);
    writer.write_u8(0x63, metadata.max_tx_rate);
}

struct SliceContextWriter<'a>(&'a mut [u8; HOST_CONTEXT_SIZE]);

impl HostContextWriter for SliceContextWriter<'_> {
    fn write_u8(&mut self, offset: usize, value: u8) {
        self.0[offset] = value;
    }

    fn write_u16(&mut self, offset: usize, value: u16) {
        self.0[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }

    fn write_u32(&mut self, offset: usize, value: u32) {
        self.0[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
}

/// Build a zero-based context image for tests and documentation. Runtime code
/// should call `write_host_context_fields` with a volatile writer instead.
pub fn initialize_host_context(image: &mut [u8; HOST_CONTEXT_SIZE], metadata: HostTxMetadata) {
    image.fill(0);
    write_host_context_fields(&mut SliceContextWriter(image), metadata);
}

#[cfg(target_arch = "arm")]
struct VolatileContextWriter {
    base: core::ptr::NonNull<u8>,
}

#[cfg(target_arch = "arm")]
impl HostContextWriter for VolatileContextWriter {
    fn write_u8(&mut self, offset: usize, value: u8) {
        unsafe { self.base.as_ptr().add(offset).write_volatile(value) };
    }

    fn write_u16(&mut self, offset: usize, value: u16) {
        unsafe {
            self.base
                .as_ptr()
                .add(offset)
                .cast::<u16>()
                .write_volatile(value)
        };
    }

    fn write_u32(&mut self, offset: usize, value: u32) {
        unsafe {
            self.base
                .as_ptr()
                .add(offset)
                .cast::<u32>()
                .write_volatile(value)
        };
    }
}

/// Initialize vendor-written fields of a live host-pool context without
/// clearing pool-owned or currently unidentified words.
///
/// # Safety
/// `address` must be an exclusively owned, aligned 0x170-byte host context.
#[cfg(target_arch = "arm")]
pub unsafe fn initialize_host_context_at(address: u32, metadata: HostTxMetadata) {
    unsafe {
        let mut writer = VolatileContextWriter {
            base: core::ptr::NonNull::new_unchecked(address as *mut u8),
        };
        write_host_context_fields(&mut writer, metadata);
    }
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

/// Rebuild the vendor 30-entry host free list while preserving the dedicated
/// packet-SRAM descriptor address assigned to each context.
///
/// # Safety
/// No host context may be pending, PAS-owned, scheduled, or completing.
#[cfg(target_arch = "arm")]
pub unsafe fn initialize_host_pool() {
    let previous = unsafe { crate::tx::disable_irq_fiq_save() };
    unsafe {
        write_live_u8(HOST_IN_FLIGHT, 0);
        write_live_u32(HOST_FREE_HEAD, 0);
        let mut head = 0;
        let mut index = 0;
        while index < HOST_CONTEXT_COUNT {
            let context =
                HostContextAddress(HOST_CONTEXT_BASE + index as u32 * HOST_CONTEXT_SIZE as u32);
            write_live_u32(context.raw() + 4, head);
            write_live_u16(context.raw() + 0x70, 0x00ff);
            write_live_u32(context.raw() + 0xa0, context.frame_state());
            head = context.raw();
            index += 1;
        }
        write_live_u32(HOST_FREE_HEAD, head);
        crate::tx::restore_irq_fiq_saved(previous);
    }
}

#[cfg(not(target_arch = "arm"))]
pub unsafe fn initialize_host_pool() {
    unsafe { core::arch::asm!("") };
}

/// Allocate exactly as `tx_wsm_buf_alloc`. A zero free head is treated as an
/// empty pool when the in-flight count is nonzero, rather than destructively
/// rebuilding contexts that may still be hardware-owned.
#[cfg(target_arch = "arm")]
pub unsafe fn allocate_host_context() -> Result<HostContextAddress, HostPoolError> {
    let previous = unsafe { crate::tx::disable_irq_fiq_save() };
    let mut head = unsafe { read_live_u32(HOST_FREE_HEAD) };
    let in_flight = unsafe { read_live_u8(HOST_IN_FLIGHT) };
    if in_flight == 0 && HostContextAddress::from_raw(head).is_none() {
        unsafe { crate::tx::restore_irq_fiq_saved(previous) };
        unsafe { initialize_host_pool() };
        return unsafe { allocate_host_context() };
    }
    let Some(context) = HostContextAddress::from_raw(head) else {
        unsafe { crate::tx::restore_irq_fiq_saved(previous) };
        return Err(if head == 0 {
            HostPoolError::Empty
        } else {
            HostPoolError::CorruptFreeHead
        });
    };
    if unsafe { read_live_u32(context.raw() + 0xa0) } != context.frame_state() {
        unsafe { crate::tx::restore_irq_fiq_saved(previous) };
        if in_flight == 0 {
            unsafe { initialize_host_pool() };
            return unsafe { allocate_host_context() };
        }
        return Err(HostPoolError::CorruptFrameState);
    }
    head = unsafe { read_live_u32(context.raw() + 4) };
    unsafe {
        write_live_u32(HOST_FREE_HEAD, head);
        write_live_u8(context.raw() + 0x0f, 0);
        write_live_u32(context.raw() + 0x20, 0xfe);
        write_live_u16(context.raw() + 0x70, 0x00fe);
        write_live_u32(context.raw() + 0x80, 0);
        write_live_u8(HOST_IN_FLIGHT, read_live_u8(HOST_IN_FLIGHT).wrapping_add(1));
        crate::tx::restore_irq_fiq_saved(previous);
    }
    Ok(context)
}

/// Return exactly as `tx_wsm_buf_free`.
///
/// # Safety
/// `context` must be allocated and have no remaining queue or hardware owner.
#[cfg(target_arch = "arm")]
pub unsafe fn free_host_context(context: HostContextAddress) {
    let previous = unsafe { crate::tx::disable_irq_fiq_save() };
    unsafe {
        write_live_u32(context.raw() + 0x20, 0xff);
        write_live_u16(context.raw() + 0x70, 0x00ff);
        let ownership = read_live_u32(context.raw() + 0x80);
        write_live_u32(context.raw() + 0x80, ownership | 0x0004_0000);
        let head = read_live_u32(HOST_FREE_HEAD);
        write_live_u32(context.raw() + 4, head);
        write_live_u32(HOST_FREE_HEAD, context.raw());
        write_live_u8(HOST_IN_FLIGHT, read_live_u8(HOST_IN_FLIGHT).wrapping_sub(1));
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
    let ac = unsafe { read_live_u8(0x0400_02dc + u32::from(queue)) };
    let submit_timer =
        unsafe { read_live_u32(0x0ac0_0004).wrapping_add(read_live_u32(0x0400_143c)) };
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
        frame_state_address: context.frame_state(),
        ..metadata
    };
    unsafe { initialize_host_context_at(context.raw(), metadata) };
    Ok(RetainedHostTx {
        context,
        release: buffer.into_release(),
        packet_id,
        phase: HostTxPhase::Submitted,
    })
}

pub fn read_u16(image: &[u8; HOST_CONTEXT_SIZE], offset: usize) -> u16 {
    u16::from_le_bytes([image[offset], image[offset + 1]])
}

pub fn read_u32(image: &[u8; HOST_CONTEXT_SIZE], offset: usize) -> u32 {
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

    #[test]
    fn host_context_addresses_preserve_pool_and_frame_state_identity() {
        let first = HostContextAddress(HOST_CONTEXT_BASE);
        let last = HostContextAddress(HOST_CONTEXT_BASE + 29 * 0x170);

        assert_eq!(HostContextAddress::from_index(0), Some(first));
        assert_eq!(
            HostContextAddress::from_index(HOST_CONTEXT_COUNT - 1),
            Some(last)
        );
        assert_eq!(first.raw(), HOST_CONTEXT_BASE);
        assert_eq!(first.frame_state(), HOST_FRAME_STATE_BASE);
        assert_eq!(last.raw(), HOST_CONTEXT_BASE + 29 * 0x170);
        assert_eq!(last.frame_state(), HOST_FRAME_STATE_BASE + 29 * 0x54);
        assert_eq!(HostContextAddress::from_raw(last.raw()), Some(last));
        assert_eq!(HostContextAddress::from_raw(last.raw() + 4), None);
        assert_eq!(HostContextAddress::from_index(HOST_CONTEXT_COUNT), None);
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
        let mut image = [0xaa; HOST_CONTEXT_SIZE];
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
    fn ordinary_batch_marks_first_and_later_descriptors_differently() {
        let original = 0x0000_1234;
        assert_eq!(scheduler_batch_flags(original, 0), original | 0x0400_0000);
        assert_eq!(scheduler_batch_flags(original, 1), original | 0x0800_0000);
        assert_eq!(scheduler_batch_flags(original, 3), original | 0x0800_0000);
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
