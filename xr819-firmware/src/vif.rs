//! Typed, volatile access to the three physical XR819 VIF records.
//!
//! The complete ABI shape lives in [`crate::dtcm`]. Production-reachable vendor
//! and interrupt writers prevent exclusive Rust ownership, so this module never
//! creates safe references to those records. Operation-specific helpers derive
//! addresses from the typed layout and preserve the parent's volatile widths and
//! ordering. Because retained writers remain reachable, target access stays raw
//! and volatile rather than claiming an IRQ-guarded ordinary-reference owner.
//! Host backing uses an explicit re-entry-rejecting owner for test isolation.

pub const VIF_COUNT: usize = crate::dtcm::VIF_RECORD_COUNT;
pub const VIF_STRIDE: usize = crate::dtcm::VIF_RECORD_SIZE;

pub const MODE_OFFSET: usize = 0x18;
pub const ACTIVE_OFFSET: usize = 0x19;
pub const CONTROL_BITS_OFFSET: usize = 0x1c;
pub const RATE_CONFIG_OFFSET: usize = 0x20;
pub const BASIC_RATES_OFFSET: usize = 0x28;
pub const TX_BUSY_OFFSET: usize = 0x30;
pub const OWN_MAC_OFFSET: usize = 0x34;
pub const BSSID_OFFSET: usize = 0x3c;
pub const CHANNEL_OFFSET: usize = 0x42;
pub const SSID_LENGTH_OFFSET: usize = 0xec;
pub const SSID_OFFSET: usize = 0xf0;
pub const DTIM_OFFSET: usize = 0x110;
pub const ATIM_OFFSET: usize = 0x116;
pub const BEACON_INTERVAL_OFFSET: usize = 0x118;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct VifState {
    pub mode: u8,
    pub active: bool,
    pub control_bits: u32,
    pub basic_rates: u32,
    pub bssid: [u8; 6],
    pub channel: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JoinStateError {
    InvalidInterface,
    Busy,
    ChannelConflict,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VifAccessError {
    InvalidInterface,
    Reentered,
}

#[cfg(all(not(target_arch = "arm"), test))]
static HOST_VIF_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(all(not(target_arch = "arm"), test))]
std::thread_local! {
    static HOST_VIF_BORROWED: core::cell::Cell<bool> = const { core::cell::Cell::new(false) };
}

#[cfg(all(not(target_arch = "arm"), test))]
struct HostVifMutationGuard;

#[cfg(all(not(target_arch = "arm"), test))]
impl Drop for HostVifMutationGuard {
    fn drop(&mut self) {
        HOST_VIF_BORROWED.with(|borrowed| borrowed.set(false));
    }
}

#[inline(always)]
pub(crate) fn vif_read_u8(address: crate::dtcm::DtcmAddress) -> u8 {
    unsafe { crate::dtcm::shared_ptr::<u8>(address).read_volatile() }
}

#[inline(always)]
pub(crate) fn vif_read_u16(address: crate::dtcm::DtcmAddress) -> u16 {
    unsafe { crate::dtcm::shared_ptr::<u16>(address).read_volatile() }
}

#[inline(always)]
pub(crate) fn vif_read_u32(address: crate::dtcm::DtcmAddress) -> u32 {
    unsafe { crate::dtcm::shared_ptr::<u32>(address).read_volatile() }
}

#[inline(always)]
fn vif_write_u8(address: crate::dtcm::DtcmAddress, value: u8) {
    unsafe { crate::dtcm::shared_ptr::<u8>(address).write_volatile(value) };
}

#[inline(always)]
pub(crate) fn vif_write_u16(address: crate::dtcm::DtcmAddress, value: u16) {
    unsafe { crate::dtcm::shared_ptr::<u16>(address).write_volatile(value) };
}

#[inline(always)]
pub(crate) fn vif_write_u32(address: crate::dtcm::DtcmAddress, value: u32) {
    unsafe { crate::dtcm::shared_ptr::<u32>(address).write_volatile(value) };
}

#[inline(always)]
fn with_vif_mutation<R>(operation: impl FnOnce() -> R) -> Result<R, VifAccessError> {
    // Production fields remain volatile because retained vendor writers are
    // reachable. There is no ordinary-reference owner to guard on target; the
    // qualified parent ordering is preserved exactly. Host tests still use an
    // explicit process-local owner to prove nested entry fails rather than
    // silently aliasing the backing store.
    #[cfg(test)]
    {
        if HOST_VIF_BORROWED.with(core::cell::Cell::get) {
            return Err(VifAccessError::Reentered);
        }
        HOST_VIF_BORROWED.with(|borrowed| borrowed.set(true));
        let _borrow_guard = HostVifMutationGuard;
        let _mutex_guard = HOST_VIF_MUTEX.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        Ok(operation())
    }
    #[cfg(not(test))]
    {
        Ok(operation())
    }
}

pub const fn record_address(interface: u8) -> Option<usize> {
    crate::dtcm::vif_record_address(interface)
}

pub const fn is_operating_interface(interface: u8) -> bool { interface < 2 }
pub const fn is_synthetic_scan_record(interface: u8) -> bool { interface == 2 }

pub fn any_active() -> bool {
    let first = crate::dtcm::vif_record(0).unwrap();
    if vif_read_u8(first.active()) != 0 {
        return true;
    }
    let second = crate::dtcm::vif_record(1).unwrap();
    vif_read_u8(second.active()) != 0
}

pub fn active_interface() -> Option<u8> {
    let first = crate::dtcm::vif_record(0).unwrap();
    if vif_read_u8(first.active()) != 0 {
        return Some(0);
    }
    let second = crate::dtcm::vif_record(1).unwrap();
    if vif_read_u8(second.active()) != 0 {
        return Some(1);
    }
    None
}

pub fn is_active(interface: u8) -> bool {
    if !is_operating_interface(interface) {
        return false;
    }
    let record = crate::dtcm::vif_record(usize::from(interface)).unwrap();
    vif_read_u8(record.active()) != 0
}

#[inline(always)]
pub(crate) fn flags(interface: u8) -> Option<u32> {
    let record = crate::dtcm::vif_record(usize::from(interface))?;
    Some(vif_read_u32(record.flags()))
}

#[inline(always)]
pub(crate) fn sleeping_links(record: crate::dtcm::VifRecordAddress) -> u16 {
    vif_read_u16(record.sleeping_links())
}

#[inline(always)]
pub(crate) fn link_gate(record: crate::dtcm::VifRecordAddress) -> u8 {
    vif_read_u8(record.link_gate())
}

#[derive(Clone, Copy)]
pub(crate) struct PowerSaveMasks { pub awake_links: u16, pub buffered_links: u16 }

#[inline(always)]
pub(crate) fn power_save_masks(interface: u8) -> Option<PowerSaveMasks> {
    crate::dtcm::vif_record(usize::from(interface)).map(|record| PowerSaveMasks {
        awake_links: vif_read_u16(record.awake_links()),
        buffered_links: vif_read_u16(record.buffered_links()),
    })
}

#[inline(always)]
pub(crate) fn allowed_links(interface: u8) -> Option<u16> { crate::dtcm::vif_record(usize::from(interface)).map(|record| vif_read_u16(record.allowed_links())) }
#[inline(always)]
pub(crate) fn internal_link(interface: u8) -> Option<u16> { crate::dtcm::vif_record(usize::from(interface)).map(|record| vif_read_u16(record.internal_link())) }
#[inline(always)]
pub(crate) fn rts_threshold(interface: u8) -> Option<u32> { crate::dtcm::vif_record(usize::from(interface)).map(|record| vif_read_u32(record.rts_threshold())) }
#[inline(always)]
pub(crate) fn ampdu_length(interface: u8) -> Option<u16> { crate::dtcm::vif_record(usize::from(interface)).map(|record| vif_read_u16(record.ampdu_length())) }

#[inline(always)]
pub(crate) fn set_ampdu_length(interface: u8, value: u16) -> Result<(), VifAccessError> {
    let record = crate::dtcm::vif_record(usize::from(interface)).ok_or(VifAccessError::InvalidInterface)?;
    with_vif_mutation(|| vif_write_u16(record.ampdu_length(), value))
}

#[inline(always)]
pub(crate) fn adjust_tx_busy(interface: u8, delta: i16) -> Result<(), VifAccessError> {
    let record = crate::dtcm::vif_record(usize::from(interface)).ok_or(VifAccessError::InvalidInterface)?;
    with_vif_mutation(|| {
        let old = vif_read_u16(record.tx_busy());
        vif_write_u16(record.tx_busy(), if delta >= 0 { old.wrapping_add(delta as u16) } else { old.wrapping_sub(delta.unsigned_abs()) });
    })
}

#[inline(always)]
pub(crate) fn clear_buffered_link(
    interface: u8,
    buffered_observed: u16,
    link_bit: u16,
) -> Result<(), VifAccessError> {
    let record = crate::dtcm::vif_record(usize::from(interface))
        .ok_or(VifAccessError::InvalidInterface)?;
    with_vif_mutation(|| {
        vif_write_u16(record.buffered_links(), buffered_observed & !link_bit)
    })
}

#[inline(always)]
pub(crate) fn clear_awake_link_and_maybe_recompute(
    interface: u8,
    sleeping_observed: u16,
    awake_observed: u16,
    link_bit: u16,
) -> Result<(), VifAccessError> {
    let record = crate::dtcm::vif_record(usize::from(interface))
        .ok_or(VifAccessError::InvalidInterface)?;
    with_vif_mutation(|| {
        let new_awake = awake_observed & !link_bit;
        vif_write_u16(record.awake_links(), new_awake);
        if vif_read_u16(record.effective_links()) != 0 {
            let effective = (!sleeping_observed | vif_read_u16(record.buffered_links()) | new_awake) & vif_read_u16(record.allowed_links());
            vif_write_u16(record.effective_links(), effective);
        }
    })
}

#[inline(always)]
pub(crate) fn own_mac_word(interface: u8, word: usize) -> Option<u16> {
    let record = crate::dtcm::vif_record(usize::from(interface))?;
    if word >= 3 { return None; }
    Some(vif_read_u16(record.own_mac_byte(word * 2)?))
}

#[inline(always)]
pub(crate) fn rate_byte(interface: u8, index: usize) -> Option<u8> {
    crate::dtcm::vif_record(usize::from(interface))
        .and_then(|record| record.rate_byte(index))
        .map(vif_read_u8)
}

#[inline(always)]
pub(crate) fn default_rate(interface: u8, index: usize) -> Option<u8> {
    crate::dtcm::vif_record(usize::from(interface))
        .and_then(|record| record.default_rate(index))
        .map(vif_read_u8)
}

#[derive(Clone, Copy)]
pub(crate) struct DiagnosticVifSnapshot { pub allowed_links: u16, pub effective_links: u16, pub flags: u32, pub mode: u8 }

#[inline(always)]
pub(crate) fn diagnostic_snapshot(interface: u8) -> Option<DiagnosticVifSnapshot> {
    let record = crate::dtcm::vif_record(usize::from(interface))?;
    Some(DiagnosticVifSnapshot { allowed_links: vif_read_u16(record.allowed_links()), effective_links: vif_read_u16(record.effective_links()), flags: vif_read_u32(record.flags()), mode: vif_read_u8(record.mode()) })
}

#[inline(always)]
pub(crate) fn set_scan_context(channel: u16) -> Result<(), VifAccessError> {
    let record = crate::dtcm::vif_record(0).unwrap();
    with_vif_mutation(|| { vif_write_u16(record.scan_rate_config(), 0x117); vif_write_u16(record.scan_channel(), channel); vif_write_u8(record.scan_flags(), 0); })
}

#[inline(always)]
pub(crate) fn clear_scan_channel() -> Result<(), VifAccessError> {
    let record = crate::dtcm::vif_record(0).unwrap();
    with_vif_mutation(|| vif_write_u16(record.scan_channel(), 0))
}

#[inline(always)]
pub(crate) fn publish_synthetic_scan_record() -> Result<(), VifAccessError> {
    let record = crate::dtcm::vif_record(2).unwrap();
    with_vif_mutation(|| { vif_write_u8(record.active(), 2); vif_write_u32(record.flags(), 0x4000); })
}

#[inline(always)]
pub(crate) fn set_all_own_mac_byte(index: usize, value: u8) {
    let first = crate::dtcm::vif_record_unchecked(0);
    let second = crate::dtcm::vif_record_unchecked(1);
    let third = crate::dtcm::vif_record_unchecked(2);
    vif_write_u8(first.own_mac_byte(index).unwrap(), value);
    vif_write_u8(second.own_mac_byte(index).unwrap(), value);
    vif_write_u8(third.own_mac_byte(index).unwrap(), value);
}

#[inline(always)]
pub(crate) fn host_contexts_in_flight() -> Result<u8, VifAccessError> { Ok(vif_read_u8(crate::dtcm::vif_record(0).unwrap().host_contexts_in_flight())) }
#[inline(always)]
pub(crate) fn set_host_contexts_in_flight(value: u8) -> Result<(), VifAccessError> { with_vif_mutation(|| vif_write_u8(crate::dtcm::vif_record(0).unwrap().host_contexts_in_flight(), value)) }
#[inline(always)]
pub(crate) fn adjust_host_contexts_in_flight(delta: i8) -> Result<(), VifAccessError> { with_vif_mutation(|| { let address = crate::dtcm::vif_record(0).unwrap().host_contexts_in_flight(); let old = vif_read_u8(address); vif_write_u8(address, if delta >= 0 { old.wrapping_add(delta as u8) } else { old.wrapping_sub(delta.unsigned_abs()) }); }) }

#[inline(always)]
pub(crate) fn wake_reinit_candidate() -> Option<usize> {
    (0..2).find(|&interface| { let record = crate::dtcm::vif_record(interface).unwrap(); let next = crate::dtcm::vif_record(interface + 1).unwrap(); matches!(vif_read_u8(record.mode()), 5 | 6) && vif_read_u8(next.wake_reinit_flag()) != 0 })
}

#[inline(always)]
pub(crate) fn mode_address(interface: u8) -> Option<usize> { crate::dtcm::vif_record(usize::from(interface)).map(|record| record.mode().get()) }


#[cfg(target_arch = "arm")]
fn read_u8(address: usize) -> u8 { unsafe { (address as *const u8).read_volatile() } }
#[cfg(target_arch = "arm")]
fn read_u16(address: usize) -> u16 { unsafe { (address as *const u16).read_volatile() } }
#[cfg(target_arch = "arm")]
fn read_u32(address: usize) -> u32 { unsafe { (address as *const u32).read_volatile() } }
#[cfg(target_arch = "arm")]
unsafe fn write_u8(address: usize, value: u8) { unsafe { (address as *mut u8).write_volatile(value) }; }
#[cfg(target_arch = "arm")]
unsafe fn write_u16(address: usize, value: u16) { unsafe { (address as *mut u16).write_volatile(value) }; }
#[cfg(target_arch = "arm")]
unsafe fn write_u32(address: usize, value: u32) { unsafe { (address as *mut u32).write_volatile(value) }; }

#[cfg(any(target_arch = "arm", test))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PasOperationBranch {
    JoinControl,
    #[cfg(test)]
    BackoffReset,
    JoinAddresses,
    JoinFamilyPublication,
    TeardownControl,
    TeardownLastActive,
}

#[cfg(any(target_arch = "arm", test))]
trait PasOperationIo {
    fn read_u8(&mut self, address: usize, branch: PasOperationBranch) -> u8;
    #[cfg(test)]
    fn read_u16(&mut self, address: usize, branch: PasOperationBranch) -> u16;
    #[cfg(test)]
    fn read_u32(&mut self, address: usize, branch: PasOperationBranch) -> u32;
    fn write_u8(&mut self, address: usize, value: u8, branch: PasOperationBranch);
    fn write_u32(&mut self, address: usize, value: u32, branch: PasOperationBranch);
    fn reset_backoff(&mut self, interface: u8);
}

#[cfg(target_arch = "arm")]
struct VolatilePasOperationIo;

#[cfg(target_arch = "arm")]
impl PasOperationIo for VolatilePasOperationIo {
    #[inline(always)]
    fn read_u8(&mut self, address: usize, _branch: PasOperationBranch) -> u8 {
        read_u8(address)
    }
    #[inline(always)]
    fn write_u8(&mut self, address: usize, value: u8, _branch: PasOperationBranch) {
        unsafe { write_u8(address, value) };
    }
    #[inline(always)]
    fn write_u32(&mut self, address: usize, value: u32, _branch: PasOperationBranch) {
        unsafe { write_u32(address, value) };
    }
    #[inline(always)]
    fn reset_backoff(&mut self, interface: u8) {
        let _ = unsafe { reset_pas_backoff(interface) };
    }
}

#[cfg(test)]
fn reset_pas_backoff_with_io<I: PasOperationIo>(interface: u8, io: &mut I) {
    let pas = crate::dtcm::pas_stride_view_unchecked(usize::from(interface));
    let override_enabled = io.read_u32(crate::dtcm::pas_backoff_override_enabled().get(), PasOperationBranch::BackoffReset) != 0;
    let override_window = io.read_u32(crate::dtcm::pas_backoff_override_window().get(), PasOperationBranch::BackoffReset);
    for queue in 0..4 {
        io.write_u32(
            pas.retry_count_unchecked(queue).get(),
            0,
            PasOperationBranch::BackoffReset,
        );
        let window = if override_enabled {
            override_window
        } else {
            u32::from(io.read_u16(
                pas.cw_min_unchecked(queue).get(),
                PasOperationBranch::BackoffReset,
            ))
        };
        io.write_u32(
            pas.contention_window_unchecked(queue).get(),
            window,
            PasOperationBranch::BackoffReset,
        );
    }
}

#[cfg(target_arch = "arm")]
#[inline(never)]
unsafe fn reset_pas_backoff(interface: u8) -> Result<(), JoinStateError> {
    if interface >= VIF_COUNT as u8 {
        return Err(JoinStateError::InvalidInterface);
    }
    let pas = crate::dtcm::pas_stride_view_unchecked(usize::from(interface));
    let override_enabled = read_u32(crate::dtcm::pas_backoff_override_enabled().get()) != 0;
    let override_window = read_u32(crate::dtcm::pas_backoff_override_window().get());
    for queue in 0..4 {
        unsafe { write_u32(pas.retry_count_unchecked(queue).get(), 0) };
        let window = if override_enabled {
            override_window
        } else {
            u32::from(read_u16(pas.cw_min_unchecked(queue).get()))
        };
        unsafe { write_u32(pas.contention_window_unchecked(queue).get(), window) };
    }
    Ok(())
}

#[cfg(any(target_arch = "arm", test))]
#[inline(always)]
fn publish_join_pas_with_io<I: PasOperationIo>(
    interface: u8,
    basic_rate_bits: u32,
    band: u8,
    beacon_interval: u32,
    own_mac: &[u8; 6],
    bssid: &[u8; 6],
    io: &mut I,
) {
    let pas = crate::dtcm::pas_stride_view_unchecked(usize::from(interface));
    io.write_u8(
        crate::dtcm::pas_stride_view_unchecked(2).activity_state().get(),
        0,
        PasOperationBranch::JoinControl,
    );
    io.write_u8(pas.activity_state().get(), 2, PasOperationBranch::JoinControl);
    io.write_u8(pas.slot_bits().get(), 4, PasOperationBranch::JoinControl);
    io.write_u8(pas.mode_byte().get(), 1, PasOperationBranch::JoinControl);
    let copied_path_byte = io.read_u8(
        crate::dtcm::low_mac_own_mac_byte_unchecked(0, 5).get(),
        PasOperationBranch::JoinControl,
    );
    io.write_u8(
        pas.own_mac_byte_unchecked(5).get(),
        copied_path_byte,
        PasOperationBranch::JoinControl,
    );
    io.write_u8(
        pas.path_selector_byte().get(),
        0,
        PasOperationBranch::JoinControl,
    );
    io.write_u32(
        pas.basic_rate_bits().get(),
        basic_rate_bits,
        PasOperationBranch::JoinControl,
    );
    io.write_u32(pas.tsf_adjust_low().get(), 0, PasOperationBranch::JoinControl);
    io.write_u32(pas.tsf_adjust_high().get(), 0, PasOperationBranch::JoinControl);
    io.write_u8(
        pas.nonzero_block_byte().get(),
        0,
        PasOperationBranch::JoinControl,
    );
    io.write_u8(
        pas.tbtt_window_control_byte().get(),
        0,
        PasOperationBranch::JoinControl,
    );

    io.reset_backoff(interface);

    for (index, value) in own_mac.iter().copied().enumerate() {
        io.write_u8(
            pas.own_mac_byte_unchecked(index).get(),
            value,
            PasOperationBranch::JoinAddresses,
        );
    }
    for (index, value) in bssid.iter().copied().enumerate() {
        io.write_u8(
            pas.bssid_byte_unchecked(index).get(),
            value,
            PasOperationBranch::JoinAddresses,
        );
    }
    io.write_u32(
        crate::dtcm::LOW_MAC_BEACON_INTERVAL.get(),
        beacon_interval.wrapping_shl(10),
        PasOperationBranch::JoinFamilyPublication,
    );
    io.write_u32(
        crate::dtcm::LOW_MAC_BAND_BITS.get(),
        1 << band,
        PasOperationBranch::JoinFamilyPublication,
    );
}

#[cfg(any(target_arch = "arm", test))]
#[inline(always)]
fn publish_teardown_pas_with_io<I: PasOperationIo>(interface: u8, last_active: bool, io: &mut I) {
    let pas = crate::dtcm::pas_stride_view_unchecked(usize::from(interface));
    io.write_u8(pas.activity_state().get(), 1, PasOperationBranch::TeardownControl);
    io.write_u8(pas.mode_byte().get(), 0, PasOperationBranch::TeardownControl);
    io.write_u8(
        pas.path_selector_byte().get(),
        0,
        PasOperationBranch::TeardownControl,
    );
    if last_active {
        io.write_u32(
            crate::dtcm::LOW_MAC_BAND_BITS.get(),
            0,
            PasOperationBranch::TeardownLastActive,
        );
    }
}

/// Apply WSM EDCA through vendor `edca_apply_params` (`0x136a6`).
///
/// # Safety
/// The selected PAS record and MAC EDCA registers must be exclusively owned.
#[cfg(target_arch = "arm")]
pub unsafe fn apply_edca(
    interface: u8,
    parameters: crate::wsm::EdcaParameters,
) -> Result<(), JoinStateError> {
    if interface >= VIF_COUNT as u8 {
        return Err(JoinStateError::InvalidInterface);
    }
    let pas = crate::dtcm::pas_stride_view_unchecked(usize::from(interface));
    let wire = [
        parameters.queues[3],
        parameters.queues[2],
        parameters.queues[1],
        parameters.queues[0],
    ];
    for (queue, entry) in wire.into_iter().enumerate() {
        unsafe {
            write_u16(pas.cw_min_unchecked(queue).get(), entry.cwmin);
            write_u16(pas.cw_max_unchecked(queue).get(), entry.cwmax);
            write_u8(pas.aifs_unchecked(queue).get(), entry.aifns);
            write_u16(pas.txop_limit_unchecked(queue).get(), entry.txop_limit);
            write_u32(
                pas.max_rx_lifetime_unchecked(queue).get(),
                entry.max_rx_lifetime,
            );
        }
    }
    // Vendor copies the WSM payload verbatim into PAS +0x4cc. The host wire
    // order is params[3], [2], [1], [0], so +0x4dc..+0x4df contain q3..q0.
    // `edca_apply_params` (annotated-main.c:24040-24043) then packs q2, q0,
    // q1, q3 in that order. Keep this expressed in wire indices so it remains
    // visibly identical to the vendor formula.
    let aifs = u32::from(wire[1].aifns)
        .wrapping_add(u32::from(wire[3].aifns.wrapping_sub(1)) << 12)
        .wrapping_add(u32::from(wire[2].aifns) << 8)
        .wrapping_add(u32::from(wire[0].aifns) << 4)
        .wrapping_sub(0x111);
    unsafe { write_u32(pas.packed_aifs().get(), aifs) };

    if read_u16(crate::dtcm::LOW_MAC_CURRENT_CHANNEL.get()) == 0 && read_u32(crate::dtcm::MAC_EDCA_SLOT_TIMING.get()) != aifs {
        unsafe {
            write_u32(crate::dtcm::MAC_EDCA_SLOT_TIMING.get(), aifs);
            write_u32(crate::platform::mac_register(0x0e64), aifs);
            reset_pas_backoff(interface)?;
        }
    }
    Ok(())
}

#[inline(always)]
fn prepare_sta_record(
    record: crate::dtcm::VifRecordAddress,
    interface: u8,
    request: &crate::wsm::JoinRequest<'_>,
    own_mac: [u8; 6],
    basic_rates: u32,
    lowest_rate: u8,
    join_control_bits: u32,
) -> Result<(), VifAccessError> {
    with_vif_mutation(|| {
        vif_write_u8(record.mode(), 1);
        vif_write_u8(record.interface(), interface);
        vif_write_u8(record.role(), 4);
        vif_write_u32(record.flags(), join_control_bits);
        vif_write_u32(record.rate_configuration(), 0x117);
        vif_write_u8(record.rate_byte(2).unwrap(), request.band);
        vif_write_u8(record.rate_byte(3).unwrap(), request.preamble_type);
        vif_write_u8(record.rate_byte(4).unwrap(), lowest_rate);
        vif_write_u8(record.rate_byte(5).unwrap(), lowest_rate);
        vif_write_u8(record.rate_byte(7).unwrap(), 3);
        vif_write_u32(record.basic_rates(), basic_rates);
        vif_write_u16(record.allowed_links(), 0x8001);
        vif_write_u16(record.sleeping_links(), 0);
        vif_write_u16(record.awake_links(), 0);
        for (index, value) in own_mac.into_iter().enumerate() { vif_write_u8(record.own_mac_byte(index).unwrap(), value); }
        for (index, value) in request.bssid.into_iter().enumerate() { vif_write_u8(record.bssid_byte(index).unwrap(), value); }
        vif_write_u16(record.channel(), request.channel_number);
        vif_write_u32(record.ssid_length(), request.ssid.len() as u32);
        for index in 0..32 { vif_write_u8(record.ssid_byte(index).unwrap(), request.ssid.get(index).copied().unwrap_or(0)); }
        vif_write_u8(record.dtim_period(), request.dtim_period.max(1));
        vif_write_u16(record.atim_window(), request.atim_window);
        vif_write_u32(record.beacon_interval(), request.beacon_interval.wrapping_shl(10));
        vif_write_u16(record.internal_link(), 9);
        vif_write_u32(record.link_object_flags(), 0x100);
        for (index, value) in own_mac.into_iter().enumerate() {
            vif_write_u8(record.link_own_mac_0_byte(index).unwrap(), value);
            vif_write_u8(record.link_own_mac_1_byte(index).unwrap(), value);
        }
        for (index, value) in request.bssid.into_iter().enumerate() { vif_write_u8(record.link_bssid_byte(index).unwrap(), value); }
        vif_write_u8(record.operating_state(), 0x23);
        vif_write_u8(record.owner_interface(), interface);
        vif_write_u16(record.owner_channel(), request.channel_number);
        vif_write_u32(record.owner_deadline(), u32::MAX);
        vif_write_u32(record.owner_flags(), 0);
    })
}

#[cfg(any(target_arch = "arm", test))]
trait RadioOwnerIo {
    fn read_owner(&mut self) -> u32;
    fn write_owner(&mut self, address: usize, value: u32);
}

#[cfg(target_arch = "arm")]
struct VolatileRadioOwnerIo;

#[cfg(target_arch = "arm")]
impl RadioOwnerIo for VolatileRadioOwnerIo {
    #[inline(always)]
    fn read_owner(&mut self) -> u32 { read_u32(crate::dtcm::radio_owner().get()) }

    #[inline(always)]
    fn write_owner(&mut self, address: usize, value: u32) {
        unsafe { write_u32(address, value) };
    }
}

#[cfg(any(target_arch = "arm", test))]
#[inline(always)]
fn publish_radio_owner_with_io<I: RadioOwnerIo>(owner: u32, io: &mut I) -> bool {
    let current_owner = io.read_owner();
    if current_owner != 0 && current_owner != owner {
        return false;
    }
    io.write_owner(crate::dtcm::radio_owner().get(), owner);
    io.write_owner(crate::dtcm::radio_wait_head().get(), 0);
    io.write_owner(crate::dtcm::deferred_radio_owner().get(), 0);
    true
}

#[inline(always)]
fn publish_sta_after_owner(
    record: crate::dtcm::VifRecordAddress,
    owner_acquired: bool,
) -> Result<(), JoinStateError> {
    if !owner_acquired {
        return Err(JoinStateError::Busy);
    }
    with_vif_mutation(|| {
        vif_write_u8(record.active(), 2);
        let effective = (vif_read_u16(record.allowed_links())
            & (!vif_read_u16(record.sleeping_links()) | vif_read_u16(record.buffered_links())))
            | vif_read_u16(record.awake_links());
        vif_write_u16(record.effective_links(), effective);
        vif_write_u8(record.operating_state(), 0x30);
        vif_write_u8(record.activity_state(), 3);
    })
    .map_err(|_| JoinStateError::Busy)
}

#[inline(always)]
fn deactivate_record(record: crate::dtcm::VifRecordAddress) {
    vif_write_u8(record.active(), 0);
    vif_write_u8(record.mode(), 0);
    vif_write_u32(record.flags(), 0);
    vif_write_u16(record.allowed_links(), 0);
    vif_write_u16(record.effective_links(), 0);
    vif_write_u8(record.activity_state(), 0);
}

#[cfg(target_arch = "arm")]
pub fn join_gate(interface: u8, channel: u16) -> Result<(), JoinStateError> {
    if !is_operating_interface(interface) {
        return Err(JoinStateError::InvalidInterface);
    }
    for other in 0..2_usize {
        if other == usize::from(interface) {
            continue;
        }
        let record = crate::dtcm::vif_record(other).unwrap();
        if vif_read_u8(record.active()) != 0 && vif_read_u16(record.channel()) != channel {
            return Err(JoinStateError::ChannelConflict);
        }
    }
    Ok(())
}

/// Retain the minimal vendor STA-BSS state after channel programming. Activity
/// is published last so scan finish cannot observe a partially built VIF.
///
/// # Safety
/// JOIN must exclusively own the selected VIF/PAS records.
#[cfg(target_arch = "arm")]
pub unsafe fn activate_sta(
    interface: u8,
    request: &crate::wsm::JoinRequest<'_>,
    own_mac: [u8; 6],
) -> Result<(), JoinStateError> {
    if !is_operating_interface(interface) {
        return Err(JoinStateError::InvalidInterface);
    }
    let record = crate::dtcm::vif_record(usize::from(interface)).unwrap();
    let basic_rates = if request.basic_rate_set == 0 {
        7
    } else {
        request.basic_rate_set
    };
    let lowest_rate = basic_rates.trailing_zeros().min(6) as u8;
    // Proven JOIN publications: bit 0 plus exactly one of bit 10/bit 11.
    let join_control_bits = 1_u32 | if request.probe_for_join { 0x400 } else { 0x800 };

    // Preliminary preparation is deliberately non-final. A Busy radio-owner
    // result may leave these vendor-compatible defaults, but cannot publish an
    // operating VIF, effective links, or activity state.
    prepare_sta_record(
        record,
        interface,
        request,
        own_mac,
        basic_rates,
        lowest_rate,
        join_control_bits,
    ).map_err(|_| JoinStateError::Busy)?;

    // Use the qualified low-MAC/PAS address owner and exact parent publication
    // sequence. VIF typing does not duplicate PAS arithmetic or PS mutations.
    publish_join_pas_with_io(
        interface,
        basic_rates,
        request.band,
        request.beacon_interval,
        &own_mac,
        &request.bssid,
        &mut VolatilePasOperationIo,
    );

    let owner = record.radio_owner().get() as u32;
    if !publish_radio_owner_with_io(owner, &mut VolatileRadioOwnerIo) {
        return Err(JoinStateError::Busy);
    }
    publish_sta_after_owner(record, true)?;
    unsafe { write_u8(crate::dtcm::phy_retained_state().get(), 3) };
    Ok(())
}

/// Vendor-shaped inactive publication used by RESET and failed JOIN rollback.
///
/// # Safety
/// The selected VIF/PAS records must be exclusively owned.
#[cfg(target_arch = "arm")]
pub unsafe fn teardown(interface: u8) -> bool {
    if !is_operating_interface(interface) {
        return false;
    }
    let record = crate::dtcm::vif_record(usize::from(interface)).unwrap();
    deactivate_record(record);
    let owner = record.radio_owner().get() as u32;
    unsafe {
        let radio_owner = crate::dtcm::radio_owner().get(); if read_u32(radio_owner) == owner { write_u32(radio_owner, 0); }
        let deferred_owner = crate::dtcm::deferred_radio_owner().get(); if read_u32(deferred_owner) == owner { write_u32(deferred_owner, 0); }
    }
    publish_teardown_pas_with_io(interface, !any_active(), &mut VolatilePasOperationIo);
    true
}

/// Snapshot fields needed by JOIN-time scan restoration.
///
/// # Safety
/// The selected vendor VIF record must be initialized and stable.
#[cfg(target_arch = "arm")]
pub fn snapshot(interface: u8) -> Option<VifState> {
    let record = crate::dtcm::vif_record(usize::from(interface))?;
    let mut bssid = [0_u8; 6];
    for (index, byte) in bssid.iter_mut().enumerate() { *byte = vif_read_u8(record.bssid_byte(index)?); }
    Some(VifState {
        mode: vif_read_u8(record.mode()),
        active: vif_read_u8(record.active()) != 0,
        control_bits: vif_read_u32(record.flags()),
        basic_rates: vif_read_u32(record.basic_rates()),
        bssid,
        channel: vif_read_u16(record.channel()),
    })
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;

    static TEST_VIF_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn test_vif_guard() -> std::sync::MutexGuard<'static, ()> {
        TEST_VIF_MUTEX.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    #[test]
    fn vendor_record_addresses_use_three_bounded_strides() {
        assert_eq!(record_address(0), Some(0x0400_3e98));
        assert_eq!(record_address(1), Some(0x0400_4248));
        assert_eq!(record_address(2), Some(0x0400_45f8));
        assert_eq!(record_address(3), None);
    }

    #[test]
    fn scan_restore_fields_match_vendor_offsets() {
        assert_eq!(ACTIVE_OFFSET, 0x19);
        assert_eq!(CONTROL_BITS_OFFSET, 0x1c);
        assert_eq!(BASIC_RATES_OFFSET, 0x28);
        assert_eq!(BSSID_OFFSET, 0x3c);
        assert_eq!(CHANNEL_OFFSET, 0x42);
    }

    fn zero_test_record(interface: usize) -> crate::dtcm::VifRecordAddress {
        let record = crate::dtcm::vif_record(interface).unwrap();
        for offset in 0..VIF_STRIDE {
            unsafe {
                crate::dtcm::shared_ptr::<u8>(crate::dtcm::DtcmAddress::new(
                    record_address(interface as u8).unwrap() + offset,
                ).unwrap()).write_volatile(0);
            }
        }
        record
    }

    #[test]
    fn nested_vif_mutation_is_explicitly_rejected() {
        let _guard = test_vif_guard();
        assert_eq!(
            with_vif_mutation(|| with_vif_mutation(|| ())),
            Ok(Err(VifAccessError::Reentered))
        );
    }

    #[test]
    fn activity_semantics_exclude_synthetic_record_from_reset() {
        let _guard = test_vif_guard();
        let first = zero_test_record(0);
        let synthetic = zero_test_record(2);
        vif_write_u8(first.active(), 2);
        vif_write_u8(synthetic.active(), 2);
        assert!(is_active(0));
        assert!(!is_active(2));
        assert_eq!(active_interface(), Some(0));
        assert!(!is_operating_interface(2));
        assert_eq!(vif_read_u8(synthetic.active()), 2);
    }

    #[test]
    fn busy_join_does_not_publish_final_vif_state() {
        let _guard = test_vif_guard();
        let record = zero_test_record(0);
        vif_write_u8(record.mode(), 1);
        vif_write_u8(record.operating_state(), 0x23);
        assert_eq!(publish_sta_after_owner(record, false), Err(JoinStateError::Busy));
        assert_eq!(vif_read_u8(record.active()), 0);
        assert_eq!(vif_read_u8(record.operating_state()), 0x23);
        assert_eq!(vif_read_u8(record.activity_state()), 0);
    }

    #[test]
    fn teardown_clears_operating_fields_but_never_record_two() {
        let _guard = test_vif_guard();
        let record = zero_test_record(1);
        vif_write_u8(record.active(), 2);
        vif_write_u8(record.mode(), 6);
        vif_write_u32(record.flags(), u32::MAX);
        vif_write_u16(record.allowed_links(), 0x8001);
        vif_write_u16(record.effective_links(), 1);
        vif_write_u8(record.activity_state(), 3);
        deactivate_record(record);
        assert_eq!(vif_read_u8(record.active()), 0);
        assert_eq!(vif_read_u8(record.mode()), 0);
        assert_eq!(vif_read_u32(record.flags()), 0);
        assert_eq!(vif_read_u16(record.allowed_links()), 0);
        assert_eq!(vif_read_u16(record.effective_links()), 0);
        assert_eq!(vif_read_u8(record.activity_state()), 0);
    }

    #[test]
    fn rate_word_overlay_preserves_independent_byte_reads() {
        let _guard = test_vif_guard();
        let record = zero_test_record(0);
        vif_write_u32(record.rate_configuration(), 0x4433_2211);
        assert_eq!((0..4).map(|index| vif_read_u8(record.rate_byte(index).unwrap())).collect::<std::vec::Vec<_>>(), [0x11, 0x22, 0x33, 0x44]);
        vif_write_u8(record.rate_byte(7).unwrap(), 3);
        assert_eq!(vif_read_u32(record.rate_configuration()), 0x4433_2211);
        assert_eq!(vif_read_u8(record.rate_byte(7).unwrap()), 3);
    }

    #[test]
    fn power_save_mutations_preserve_vendor_branch_behavior() {
        let _guard = test_vif_guard();
        let record = zero_test_record(0);
        vif_write_u16(record.allowed_links(), 0x000f);
        vif_write_u16(record.buffered_links(), 0x0004);
        vif_write_u16(record.awake_links(), 0x0003);
        clear_awake_link_and_maybe_recompute(0, 0xfffa, 3, 0x0002).unwrap();
        assert_eq!(vif_read_u16(record.awake_links()), 1);
        assert_eq!(vif_read_u16(record.effective_links()), 0);
        vif_write_u16(record.effective_links(), 0x8000);
        vif_write_u16(record.awake_links(), 7);
        clear_awake_link_and_maybe_recompute(0, 0xfffa, 3, 0x0002).unwrap();
        assert_eq!(vif_read_u16(record.awake_links()), 1);
        assert_eq!(vif_read_u16(record.effective_links()), 5);

        vif_write_u16(record.buffered_links(), 0x0008);
        clear_buffered_link(0, 0x0004, 0x0002).unwrap();
        assert_eq!(vif_read_u16(record.buffered_links()), 0x0004);
    }

    #[test]
    fn wake_selection_reads_the_following_physical_record() {
        let _guard = test_vif_guard();
        let first = zero_test_record(0);
        let second = zero_test_record(1);
        let third = zero_test_record(2);
        vif_write_u8(first.mode(), 5);
        vif_write_u8(third.wake_reinit_flag(), 1);
        assert_eq!(wake_reinit_candidate(), None);
        vif_write_u8(second.wake_reinit_flag(), 1);
        assert_eq!(wake_reinit_candidate(), Some(0));
        vif_write_u8(first.mode(), 0);
        vif_write_u8(second.mode(), 6);
        assert_eq!(wake_reinit_candidate(), Some(1));
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum AccessKind {
        Read,
        Write,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct Access {
        kind: AccessKind,
        width: u8,
        address: usize,
        value: u32,
        branch: PasOperationBranch,
    }

    #[derive(Default)]
    struct RecordingPasIo {
        accesses: std::vec::Vec<Access>,
    }

    impl RecordingPasIo {
        fn record(
            &mut self,
            kind: AccessKind,
            width: u8,
            address: usize,
            value: u32,
            branch: PasOperationBranch,
        ) {
            self.accesses.push(Access { kind, width, address, value, branch });
        }
    }

    impl PasOperationIo for RecordingPasIo {
        fn read_u8(&mut self, address: usize, branch: PasOperationBranch) -> u8 {
            let value = if address == crate::dtcm::low_mac_own_mac_byte_unchecked(0, 5).get() {
                0xa5
            } else {
                0
            };
            self.record(AccessKind::Read, 1, address, u32::from(value), branch);
            value
        }

        fn read_u16(&mut self, address: usize, branch: PasOperationBranch) -> u16 {
            let first = crate::dtcm::pas_stride_view(1).unwrap();
            let value = (0..4)
                .find(|&queue| first.cw_min(queue).unwrap().get() == address)
                .map_or(0, |queue| 0x10 + queue as u16);
            self.record(AccessKind::Read, 2, address, u32::from(value), branch);
            value
        }

        fn read_u32(&mut self, address: usize, branch: PasOperationBranch) -> u32 {
            let value = if address == 0x0400_208c { 0x55aa } else { 0 };
            self.record(AccessKind::Read, 4, address, value, branch);
            value
        }

        fn write_u8(&mut self, address: usize, value: u8, branch: PasOperationBranch) {
            self.record(AccessKind::Write, 1, address, u32::from(value), branch);
        }

        fn write_u32(&mut self, address: usize, value: u32, branch: PasOperationBranch) {
            self.record(AccessKind::Write, 4, address, value, branch);
        }

        fn reset_backoff(&mut self, interface: u8) {
            reset_pas_backoff_with_io(interface, self);
        }
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct OwnerAccess {
        kind: AccessKind,
        address: usize,
        width: u8,
        value: u32,
    }

    struct RecordingOwnerIo {
        accesses: std::vec::Vec<OwnerAccess>,
        current_owner: u32,
    }

    impl RadioOwnerIo for RecordingOwnerIo {
        fn read_owner(&mut self) -> u32 {
            self.accesses.push(OwnerAccess {
                kind: AccessKind::Read,
                address: 0x0400_8b20,
                width: 4,
                value: self.current_owner,
            });
            self.current_owner
        }

        fn write_owner(&mut self, address: usize, value: u32) {
            self.accesses.push(OwnerAccess {
                kind: AccessKind::Write,
                address,
                width: 4,
                value,
            });
        }
    }

    #[test]
    fn busy_join_model_runs_pas_then_owner_check_without_final_vif_publication() {
        let _guard = test_vif_guard();
        let record = zero_test_record(0);
        let own_mac = [1, 2, 3, 4, 5, 6];
        let request = crate::wsm::JoinRequest {
            mode: 0,
            band: 1,
            channel_number: 11,
            bssid: [7, 8, 9, 10, 11, 12],
            atim_window: 0,
            preamble_type: 1,
            probe_for_join: false,
            dtim_period: 2,
            flags: 0,
            ssid: b"busy-model",
            beacon_interval: 100,
            basic_rate_set: 7,
        };
        prepare_sta_record(record, 0, &request, own_mac, 7, 0, 0x801).unwrap();
        assert_eq!(vif_read_u8(record.mode()), 1);
        assert_eq!(vif_read_u8(record.operating_state()), 0x23);
        assert_eq!(vif_read_u8(record.active()), 0);
        assert_eq!(vif_read_u16(record.effective_links()), 0);
        assert_eq!(vif_read_u8(record.activity_state()), 0);

        let mut pas = RecordingPasIo::default();
        publish_join_pas_with_io(
            0,
            7,
            request.band,
            request.beacon_interval,
            &own_mac,
            &request.bssid,
            &mut pas,
        );
        assert_eq!(pas.accesses.len(), 40);
        assert_eq!(pas.accesses.first().unwrap().branch, PasOperationBranch::JoinControl);
        assert_eq!(pas.accesses.last().unwrap().branch, PasOperationBranch::JoinFamilyPublication);

        let owner = record.radio_owner().get() as u32;
        let mut owner_io = RecordingOwnerIo {
            accesses: std::vec::Vec::new(),
            current_owner: owner.wrapping_add(4),
        };
        let acquired = publish_radio_owner_with_io(owner, &mut owner_io);
        assert!(!acquired);
        assert_eq!(owner_io.accesses, [OwnerAccess {
            kind: AccessKind::Read,
            address: 0x0400_8b20,
            width: 4,
            value: owner.wrapping_add(4),
        }]);
        assert_eq!(publish_sta_after_owner(record, acquired), Err(JoinStateError::Busy));
        assert_eq!(vif_read_u8(record.active()), 0);
        assert_eq!(vif_read_u16(record.effective_links()), 0);
        assert_eq!(vif_read_u8(record.operating_state()), 0x23);
        assert_eq!(vif_read_u8(record.activity_state()), 0);
    }

    #[test]
    fn join_pas_helper_records_actual_access_width_value_branch_and_order() {
        let mut io = RecordingPasIo::default();
        publish_join_pas_with_io(
            1,
            0x1234,
            1,
            100,
            &[1, 2, 3, 4, 5, 6],
            &[7, 8, 9, 10, 11, 12],
            &mut io,
        );

        let pas = crate::dtcm::pas_stride_view(1).unwrap();
        let expected_prefix = [
            Access { kind: AccessKind::Write, width: 1, address: crate::dtcm::pas_stride_view(2).unwrap().activity_state().get(), value: 0, branch: PasOperationBranch::JoinControl },
            Access { kind: AccessKind::Write, width: 1, address: pas.activity_state().get(), value: 2, branch: PasOperationBranch::JoinControl },
            Access { kind: AccessKind::Write, width: 1, address: pas.slot_bits().get(), value: 4, branch: PasOperationBranch::JoinControl },
            Access { kind: AccessKind::Write, width: 1, address: pas.mode_byte().get(), value: 1, branch: PasOperationBranch::JoinControl },
            Access { kind: AccessKind::Read, width: 1, address: crate::dtcm::low_mac_own_mac_byte(0, 5).unwrap().get(), value: 0xa5, branch: PasOperationBranch::JoinControl },
            Access { kind: AccessKind::Write, width: 1, address: pas.own_mac_byte(5).unwrap().get(), value: 0xa5, branch: PasOperationBranch::JoinControl },
            Access { kind: AccessKind::Write, width: 1, address: pas.path_selector_byte().get(), value: 0, branch: PasOperationBranch::JoinControl },
            Access { kind: AccessKind::Write, width: 4, address: pas.basic_rate_bits().get(), value: 0x1234, branch: PasOperationBranch::JoinControl },
            Access { kind: AccessKind::Write, width: 4, address: pas.tsf_adjust_low().get(), value: 0, branch: PasOperationBranch::JoinControl },
            Access { kind: AccessKind::Write, width: 4, address: pas.tsf_adjust_high().get(), value: 0, branch: PasOperationBranch::JoinControl },
            Access { kind: AccessKind::Write, width: 1, address: pas.nonzero_block_byte().get(), value: 0, branch: PasOperationBranch::JoinControl },
            Access { kind: AccessKind::Write, width: 1, address: pas.tbtt_window_control_byte().get(), value: 0, branch: PasOperationBranch::JoinControl },
        ];
        assert_eq!(&io.accesses[..expected_prefix.len()], &expected_prefix);
        assert_eq!(io.accesses[12], Access { kind: AccessKind::Read, width: 4, address: 0x0400_2088, value: 0, branch: PasOperationBranch::BackoffReset });
        assert_eq!(io.accesses[13], Access { kind: AccessKind::Read, width: 4, address: 0x0400_208c, value: 0x55aa, branch: PasOperationBranch::BackoffReset });
        assert_eq!(io.accesses[14], Access { kind: AccessKind::Write, width: 4, address: pas.retry_count(0).unwrap().get(), value: 0, branch: PasOperationBranch::BackoffReset });
        assert_eq!(io.accesses[15], Access { kind: AccessKind::Read, width: 2, address: pas.cw_min(0).unwrap().get(), value: 0x10, branch: PasOperationBranch::BackoffReset });
        assert_eq!(io.accesses[16], Access { kind: AccessKind::Write, width: 4, address: pas.contention_window(0).unwrap().get(), value: 0x10, branch: PasOperationBranch::BackoffReset });
        assert_eq!(io.accesses.len(), 40);
        assert_eq!(io.accesses[38], Access { kind: AccessKind::Write, width: 4, address: crate::dtcm::LOW_MAC_BEACON_INTERVAL.get(), value: 100 << 10, branch: PasOperationBranch::JoinFamilyPublication });
        assert_eq!(io.accesses[39], Access { kind: AccessKind::Write, width: 4, address: crate::dtcm::LOW_MAC_BAND_BITS.get(), value: 2, branch: PasOperationBranch::JoinFamilyPublication });
        assert!(io.accesses[26..38].iter().all(|access| access.branch == PasOperationBranch::JoinAddresses));
    }

    #[test]
    fn teardown_pas_helper_records_last_active_branch_after_control_writes() {
        let mut io = RecordingPasIo::default();
        publish_teardown_pas_with_io(0, true, &mut io);
        let pas = crate::dtcm::pas_stride_view(0).unwrap();
        assert_eq!(io.accesses, [
            Access { kind: AccessKind::Write, width: 1, address: pas.activity_state().get(), value: 1, branch: PasOperationBranch::TeardownControl },
            Access { kind: AccessKind::Write, width: 1, address: pas.mode_byte().get(), value: 0, branch: PasOperationBranch::TeardownControl },
            Access { kind: AccessKind::Write, width: 1, address: pas.path_selector_byte().get(), value: 0, branch: PasOperationBranch::TeardownControl },
            Access { kind: AccessKind::Write, width: 4, address: crate::dtcm::LOW_MAC_BAND_BITS.get(), value: 0, branch: PasOperationBranch::TeardownLastActive },
        ]);

        let mut still_active = RecordingPasIo::default();
        publish_teardown_pas_with_io(0, false, &mut still_active);
        assert_eq!(still_active.accesses, io.accesses[..3]);
    }
}
