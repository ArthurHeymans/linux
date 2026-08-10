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
pub const BASIC_RATES_OFFSET: usize = 0x28;
pub const BSSID_OFFSET: usize = 0x3c;
pub const CHANNEL_OFFSET: usize = 0x42;

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