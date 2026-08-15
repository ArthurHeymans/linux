//! XR819 VIF record layout used by JOIN and scan restoration.
//!
//! Vendor firmware keeps three `0x3b0`-byte records rooted at `0x04003e98`.
//! `syn_scan_finish_and_confirm` selects channel restoration instead of
//! `mac_radio_stop` when byte `+0x19` of any record is nonzero.

pub const VIF_COUNT: usize = 3;
pub const VIF_BASE: usize = 0x0400_3e98;
pub const VIF_STRIDE: usize = 0x3b0;

pub const MODE_OFFSET: usize = 0x18;
pub const ACTIVE_OFFSET: usize = 0x19;
pub const FLAGS_OFFSET: usize = 0x1c;
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

#[cfg(target_arch = "arm")]
const PAS_BASE: usize = 0x0400_3678;
#[cfg(target_arch = "arm")]
const PAS_STRIDE: usize = 0x98;
#[cfg(target_arch = "arm")]
const PAS_ACTIVE_OFFSET: usize = 0x470;

static mut ACTIVE_MASK: u8 = 0;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct VifState {
    pub mode: u8,
    pub active: bool,
    pub flags: u32,
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

pub const fn record_address(interface: u8) -> Option<usize> {
    if interface < VIF_COUNT as u8 {
        Some(VIF_BASE + interface as usize * VIF_STRIDE)
    } else {
        None
    }
}

pub fn any_active() -> bool {
    unsafe { (&raw const ACTIVE_MASK).read_volatile() != 0 }
}

pub fn active_interface() -> Option<u8> {
    let mask = unsafe { (&raw const ACTIVE_MASK).read_volatile() };
    if mask & 1 != 0 {
        Some(0)
    } else if mask & 2 != 0 {
        Some(1)
    } else {
        None
    }
}

pub fn is_active(interface: u8) -> bool {
    interface < VIF_COUNT as u8
        && unsafe { (&raw const ACTIVE_MASK).read_volatile() & (1 << interface) != 0 }
}

/// Publish the firmware-owned activity byte used by scan finish selection.
/// JOIN/RESET will become the only callers; synthetic scan context writes must
/// not accidentally activate this branch.
///
/// # Safety
/// The selected VIF record and retained state must be exclusively owned.
pub unsafe fn set_active(interface: u8, active: bool) -> bool {
    let Some(_base) = record_address(interface) else {
        return false;
    };
    let bit = 1_u8 << interface;
    unsafe {
        let mask = (&raw const ACTIVE_MASK).read_volatile();
        (&raw mut ACTIVE_MASK).write_volatile(if active { mask | bit } else { mask & !bit });
        #[cfg(target_arch = "arm")]
        ((_base + ACTIVE_OFFSET) as *mut u8).write_volatile(u8::from(active));
    }
    true
}

#[cfg(target_arch = "arm")]
fn read_u8(address: usize) -> u8 {
    unsafe { (address as *const u8).read_volatile() }
}

#[cfg(target_arch = "arm")]
fn read_u16(address: usize) -> u16 {
    unsafe { (address as *const u16).read_volatile() }
}

#[cfg(target_arch = "arm")]
fn read_u32(address: usize) -> u32 {
    unsafe { (address as *const u32).read_volatile() }
}

#[cfg(target_arch = "arm")]
unsafe fn write_u8(address: usize, value: u8) {
    unsafe { (address as *mut u8).write_volatile(value) };
}

#[cfg(target_arch = "arm")]
unsafe fn write_u16(address: usize, value: u16) {
    unsafe { (address as *mut u16).write_volatile(value) };
}

#[cfg(target_arch = "arm")]
unsafe fn write_u32(address: usize, value: u32) {
    unsafe { (address as *mut u32).write_volatile(value) };
}

#[cfg(target_arch = "arm")]
unsafe fn copy_bytes(address: usize, bytes: &[u8]) {
    for (offset, value) in bytes.iter().copied().enumerate() {
        unsafe { write_u8(address + offset, value) };
    }
}

#[cfg(target_arch = "arm")]
unsafe fn reset_pas_backoff(interface: u8) -> Result<(), JoinStateError> {
    if interface >= VIF_COUNT as u8 {
        return Err(JoinStateError::InvalidInterface);
    }
    let pas = PAS_BASE + usize::from(interface) * PAS_STRIDE;
    let override_enabled = read_u32(0x0400_2088) != 0;
    let override_window = read_u32(0x0400_208c);
    for queue in 0..4 {
        unsafe { write_u32(pas + 0x4ac + queue * 4, 0) };
        let window = if override_enabled {
            override_window
        } else {
            u32::from(read_u16(pas + 0x4cc + queue * 2))
        };
        unsafe { write_u32(pas + 0x4bc + queue * 4, window) };
    }
    Ok(())
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
    let pas = PAS_BASE + usize::from(interface) * PAS_STRIDE;
    let wire = [
        parameters.queues[3],
        parameters.queues[2],
        parameters.queues[1],
        parameters.queues[0],
    ];
    for (queue, entry) in wire.into_iter().enumerate() {
        unsafe {
            write_u16(pas + 0x4cc + queue * 2, entry.cwmin);
            write_u16(pas + 0x4d4 + queue * 2, entry.cwmax);
            write_u8(pas + 0x4dc + queue, entry.aifns);
            write_u16(pas + 0x4e0 + queue * 2, entry.txop_limit);
            write_u32(pas + 0x4e8 + queue * 4, entry.max_rx_lifetime);
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
    unsafe { write_u32(pas + 0x4fc, aifs) };

    if read_u16(0x0400_3a68) == 0 && read_u32(0x0400_1b04) != aifs {
        unsafe {
            write_u32(0x0400_1b04, aifs);
            write_u32(0x09c0_0e64, aifs);
            reset_pas_backoff(interface)?;
        }
    }
    Ok(())
}

#[cfg(target_arch = "arm")]
pub fn join_gate(interface: u8, channel: u16) -> Result<(), JoinStateError> {
    let _ = record_address(interface).ok_or(JoinStateError::InvalidInterface)?;
    for other in 0..2_u8 {
        if other == interface {
            continue;
        }
        let other_base = record_address(other).ok_or(JoinStateError::InvalidInterface)?;
        if read_u8(other_base + ACTIVE_OFFSET) != 0
            && read_u16(other_base + CHANNEL_OFFSET) != channel
        {
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
    let base = record_address(interface).ok_or(JoinStateError::InvalidInterface)?;
    let pas = PAS_BASE + usize::from(interface) * PAS_STRIDE;
    let basic_rates = if request.basic_rate_set == 0 {
        7
    } else {
        request.basic_rate_set
    };
    let lowest_rate = basic_rates.trailing_zeros().min(6) as u8;
    let flags = 1_u32 | if request.probe_for_join { 0x400 } else { 0x800 };

    unsafe {
        write_u8(base + MODE_OFFSET, 1);
        write_u8(base + 0x1a, interface);
        write_u8(base + 0x1b, 4);
        write_u32(base + FLAGS_OFFSET, flags);
        write_u32(base + RATE_CONFIG_OFFSET, 0x117);
        write_u8(base + 0x22, request.band);
        write_u8(base + 0x23, request.preamble_type);
        write_u8(base + 0x24, lowest_rate);
        write_u8(base + 0x25, lowest_rate);
        write_u8(base + 0x27, 3);
        write_u32(base + BASIC_RATES_OFFSET, basic_rates);
        // `vif_enter_operating_state()` publishes host link 0 plus the
        // firmware-internal slot 15. Class-6 uses its own context metadata;
        // excluding bit 0 here causes vendor `task_b88e` to reject class-0.
        write_u16(base + 0x2c, 0x8001);
        write_u16(base + 0x15c, 0);
        write_u16(base + 0x15e, 0);
        copy_bytes(base + OWN_MAC_OFFSET, &own_mac);
        copy_bytes(base + BSSID_OFFSET, &request.bssid);
        write_u16(base + CHANNEL_OFFSET, request.channel_number);
        write_u32(base + SSID_LENGTH_OFFSET, request.ssid.len() as u32);
        copy_bytes(base + SSID_OFFSET, request.ssid);
        for offset in request.ssid.len()..32 {
            write_u8(base + SSID_OFFSET + offset, 0);
        }
        write_u8(base + DTIM_OFFSET, request.dtim_period.max(1));
        write_u16(base + ATIM_OFFSET, request.atim_window);
        write_u32(
            base + BEACON_INTERVAL_OFFSET,
            request.beacon_interval.wrapping_shl(10),
        );
        write_u16(base + 0x12a, 9);
        write_u32(base + 0x13c, 0x100);
        copy_bytes(base + 0x140, &own_mac);
        copy_bytes(base + 0x146, &own_mac);
        copy_bytes(base + 0x14c, &request.bssid);
        write_u8(base + 0x50, 0x23);
        write_u8(base + 0x51, interface);
        write_u16(base + 0x52, request.channel_number);
        write_u32(base + 0x54, u32::MAX);
        write_u32(base + 0x5c, 0);

        // The synthetic scan record occupies PAS slot 2 only while channel
        // programming runs. Vendor JOIN recomputes the MAC mode word from the
        // actual active VIF set, so it must not remain active beside the STA.
        write_u8(PAS_BASE + 2 * PAS_STRIDE + PAS_ACTIVE_OFFSET, 0);
        write_u8(pas + PAS_ACTIVE_OFFSET, 2);
        write_u8(pas + 0x471, 4);
        write_u8(pas + 0x472, 1);
        write_u8(pas + 0x481, read_u8(PAS_BASE + 0x459));
        write_u8(pas + 0x473, 0);
        write_u32(pas + 0x478, basic_rates);
        write_u32(pas + 0x488, 0);
        write_u32(pas + 0x48c, 0);
        write_u8(pas + 0x492, 0);
        write_u8(pas + 0x493, 0);

        // `mac_apply_channel_and_vif_config` resets all four contention
        // windows after activating the selected PAS interface.
        reset_pas_backoff(interface)?;
        copy_bytes(pas + 0x47c, &own_mac);
        copy_bytes(pas + 0x482, &request.bssid);
        write_u32(0x0400_3680, request.beacon_interval.wrapping_shl(10));
        write_u32(0x0400_3684, 1 << request.band);
        let _ = set_active(interface, true);
        // Vendor JOIN activates the VIF before channel programming. The
        // channel-program tail then marks activity state 2 and derives the
        // effective link mask used by the STA branch of `task_b88e`.
        write_u8(base + ACTIVE_OFFSET, 2);
        let effective = (read_u16(base + 0x2c)
            & (!read_u16(base + 0x15c) | read_u16(base + 0x160)))
            | read_u16(base + 0x15e);
        write_u16(base + 0x2e, effective);

        // Awake-STA subset of vendor `lmc_sched_request_radio(VIF + 0x44)`.
        // JOIN owns a single channel, so no pending-owner arbitration is
        // reachable here. Channel programming already completed above; publish
        // the retained owner and its post-program state before admitting TX.
        let owner = (base + 0x44) as u32;
        let current_owner = read_u32(0x0400_8b20);
        if current_owner != 0 && current_owner != owner {
            return Err(JoinStateError::Busy);
        }
        write_u32(0x0400_8b20, owner);
        write_u32(0x0400_8b24, 0);
        write_u32(0x0400_8b2c, 0);
        write_u8(base + 0x50, 0x30);
        write_u8(base + 0x66, 3);
        // The vendor JOIN/channel-program path reaches retained PHY state 3.
        // The synthetic scan path can leave state 5 behind; carrying it into
        // ordinary STA TX produces a different packet-controller GO state.
        write_u8(0x0400_99a9, 3);
    }
    Ok(())
}

/// Vendor-shaped inactive publication used by RESET and failed JOIN rollback.
///
/// # Safety
/// The selected VIF/PAS records must be exclusively owned.
#[cfg(target_arch = "arm")]
pub unsafe fn teardown(interface: u8) -> bool {
    let Some(base) = record_address(interface) else {
        return false;
    };
    let pas = PAS_BASE + usize::from(interface) * PAS_STRIDE;
    unsafe {
        let _ = set_active(interface, false);
        write_u8(base + MODE_OFFSET, 0);
        write_u32(base + FLAGS_OFFSET, 0);
        write_u16(base + 0x2c, 0);
        write_u16(base + 0x2e, 0);
        let owner = (base + 0x44) as u32;
        if read_u32(0x0400_8b20) == owner {
            write_u32(0x0400_8b20, 0);
        }
        if read_u32(0x0400_8b2c) == owner {
            write_u32(0x0400_8b2c, 0);
        }
        write_u8(base + 0x66, 0);
        write_u8(pas + PAS_ACTIVE_OFFSET, 1);
        write_u8(pas + 0x472, 0);
        write_u8(pas + 0x473, 0);
        if !any_active() {
            write_u32(0x0400_3684, 0);
        }
    }
    true
}

/// Snapshot fields needed by JOIN-time scan restoration.
///
/// # Safety
/// The selected vendor VIF record must be initialized and stable.
#[cfg(target_arch = "arm")]
pub unsafe fn snapshot(interface: u8) -> Option<VifState> {
    let base = record_address(interface)?;
    let mut bssid = [0_u8; 6];
    for (offset, byte) in bssid.iter_mut().enumerate() {
        *byte = unsafe { ((base + BSSID_OFFSET + offset) as *const u8).read_volatile() };
    }
    Some(VifState {
        mode: unsafe { ((base + MODE_OFFSET) as *const u8).read_volatile() },
        active: unsafe { ((base + ACTIVE_OFFSET) as *const u8).read_volatile() } != 0,
        flags: unsafe { ((base + FLAGS_OFFSET) as *const u32).read_volatile() },
        basic_rates: unsafe { ((base + BASIC_RATES_OFFSET) as *const u32).read_volatile() },
        bssid,
        channel: unsafe { ((base + CHANNEL_OFFSET) as *const u16).read_volatile() },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(BASIC_RATES_OFFSET, 0x28);
        assert_eq!(BSSID_OFFSET, 0x3c);
        assert_eq!(CHANNEL_OFFSET, 0x42);
    }
}
