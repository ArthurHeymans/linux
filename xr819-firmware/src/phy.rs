//! Vendor MAC/PHY initialization data consumed by `0x16ac6`.
//!
//! These constants come from the initialized-SRAM container segments rather
//! than guessed RF values. Static MAC initialization is active; per-channel
//! programming remains bounded at the translated request ABI before `0xf802`.

use zerocopy::byteorder::little_endian::U16;
use zerocopy::{Immutable, IntoBytes};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CalibrationAnchor {
    pub selector: u8,
    pub upper: i16,
    pub lower: i16,
}

pub const MODE0_CALIBRATION_ANCHORS: [CalibrationAnchor; 22] = [
    CalibrationAnchor {
        selector: 26,
        upper: 5,
        lower: -1017,
    },
    CalibrationAnchor {
        selector: 25,
        upper: 38,
        lower: -1049,
    },
    CalibrationAnchor {
        selector: 24,
        upper: 70,
        lower: -1054,
    },
    CalibrationAnchor {
        selector: 22,
        upper: 99,
        lower: -1085,
    },
    CalibrationAnchor {
        selector: 21,
        upper: 131,
        lower: -1117,
    },
    CalibrationAnchor {
        selector: 20,
        upper: 164,
        lower: -1147,
    },
    CalibrationAnchor {
        selector: 18,
        upper: 196,
        lower: -1180,
    },
    CalibrationAnchor {
        selector: 17,
        upper: 228,
        lower: -1211,
    },
    CalibrationAnchor {
        selector: 16,
        upper: 260,
        lower: -1244,
    },
    CalibrationAnchor {
        selector: 2,
        upper: 289,
        lower: -1272,
    },
    CalibrationAnchor {
        selector: 1,
        upper: 322,
        lower: -1305,
    },
    CalibrationAnchor {
        selector: 0,
        upper: 354,
        lower: -1317,
    },
    CalibrationAnchor {
        selector: 56,
        upper: 404,
        lower: -1387,
    },
    CalibrationAnchor {
        selector: 54,
        upper: 433,
        lower: -1420,
    },
    CalibrationAnchor {
        selector: 53,
        upper: 465,
        lower: -1449,
    },
    CalibrationAnchor {
        selector: 52,
        upper: 498,
        lower: -1461,
    },
    CalibrationAnchor {
        selector: 50,
        upper: 530,
        lower: -1493,
    },
    CalibrationAnchor {
        selector: 49,
        upper: 562,
        lower: -1510,
    },
    CalibrationAnchor {
        selector: 48,
        upper: 595,
        lower: -1519,
    },
    CalibrationAnchor {
        selector: 34,
        upper: 623,
        lower: -1532,
    },
    CalibrationAnchor {
        selector: 33,
        upper: 656,
        lower: -1539,
    },
    CalibrationAnchor {
        selector: 32,
        upper: 688,
        lower: -1544,
    },
];

pub const MODE1_CALIBRATION_ANCHORS: [CalibrationAnchor; 22] = [
    CalibrationAnchor {
        selector: 26,
        upper: 189,
        lower: -1201,
    },
    CalibrationAnchor {
        selector: 25,
        upper: 222,
        lower: -1233,
    },
    CalibrationAnchor {
        selector: 24,
        upper: 254,
        lower: -1263,
    },
    CalibrationAnchor {
        selector: 22,
        upper: 283,
        lower: -1295,
    },
    CalibrationAnchor {
        selector: 21,
        upper: 316,
        lower: -1320,
    },
    CalibrationAnchor {
        selector: 20,
        upper: 348,
        lower: -1331,
    },
    CalibrationAnchor {
        selector: 18,
        upper: 380,
        lower: -1363,
    },
    CalibrationAnchor {
        selector: 17,
        upper: 412,
        lower: -1396,
    },
    CalibrationAnchor {
        selector: 16,
        upper: 445,
        lower: -1428,
    },
    CalibrationAnchor {
        selector: 2,
        upper: 473,
        lower: -1437,
    },
    CalibrationAnchor {
        selector: 1,
        upper: 506,
        lower: -1462,
    },
    CalibrationAnchor {
        selector: 0,
        upper: 538,
        lower: -1478,
    },
    CalibrationAnchor {
        selector: 56,
        upper: 405,
        lower: -1391,
    },
    CalibrationAnchor {
        selector: 54,
        upper: 434,
        lower: -1437,
    },
    CalibrationAnchor {
        selector: 53,
        upper: 467,
        lower: -1450,
    },
    CalibrationAnchor {
        selector: 52,
        upper: 499,
        lower: -1471,
    },
    CalibrationAnchor {
        selector: 50,
        upper: 531,
        lower: -1494,
    },
    CalibrationAnchor {
        selector: 49,
        upper: 564,
        lower: -1511,
    },
    CalibrationAnchor {
        selector: 48,
        upper: 596,
        lower: -1520,
    },
    CalibrationAnchor {
        selector: 34,
        upper: 624,
        lower: -1533,
    },
    CalibrationAnchor {
        selector: 33,
        upper: 657,
        lower: -1542,
    },
    CalibrationAnchor {
        selector: 32,
        upper: 689,
        lower: -1547,
    },
];

/// First normalization pass performed by vendor `0x17008`.
pub fn normalize_anchor(anchor: CalibrationAnchor, correction: i16) -> CalibrationAnchor {
    CalibrationAnchor {
        selector: anchor.selector,
        upper: (((i32::from(anchor.upper) - i32::from(correction)) + 8) >> 4) as i16,
        lower: (((i32::from(anchor.lower) + i32::from(correction)) + 8) >> 4) as i16,
    }
}

/// Expand the 22 calibration anchors into the 80 hardware words written by
/// vendor `0x17008` to `0x0ab80800`.
pub fn build_rate_table(
    anchors: &[CalibrationAnchor; 22],
    correction: i16,
    output: &mut [u32; 80],
) {
    let mut normalized = [CalibrationAnchor {
        selector: 0,
        upper: 0,
        lower: 0,
    }; 22];
    for (destination, source) in normalized.iter_mut().zip(anchors) {
        *destination = normalize_anchor(*source, correction);
    }

    for (index, word) in output.iter_mut().enumerate() {
        let target = index as i16 - 90;
        let mut selected = 0;
        let mut best_lower = -24;
        for candidate in (0..normalized.len()).rev() {
            let anchor = normalized[candidate];
            if target < -(anchor.upper + 24) && anchor.lower < best_lower {
                selected = candidate;
                best_lower = anchor.lower;
            }
        }

        let anchor = normalized[selected];
        let halfword = u16::from(anchor.selector) | (((anchor.upper as u16) & 0x7f) << 8);
        *word = u32::from(halfword) | (u32::from(halfword) << 16);
    }
}

/// One-based index stored in the low seven bits of `0x0ab80410`.
pub fn rate_table_boundary(table: &[u32; 80]) -> u8 {
    table
        .iter()
        .position(|word| ((word & 0x7fff) >> 8) < 11)
        .map(|index| index as u8 + 1)
        .unwrap_or(80)
}

pub const COMMON_INITIALIZATION_LISTS: [&[(u32, u32)]; 4] = [
    &[(0x0ab8_0108, 0x0020_0300)],
    &[
        (0x0ab8_807c, 0x0000_0001),
        (0x0ab8_8058, 0x0000_63d9),
        (0x0ab8_808c, 0x0000_103f),
        (0x0ab8_8090, 0x1010_103f),
    ],
    &[(0x0ab9_0000, 0), (0x0ab9_0014, 0x0fff_ffff)],
    &[
        (0x0aba_0004, 0x0000_0033),
        (0x0aba_0008, 0x0000_013e),
        (0x0aba_000c, 0x0000_00a6),
        (0x0aba_0010, 0x0000_0001),
        (0x0aba_0014, 0x0000_0040),
        (0x0ab8_0c2c, 0x0000_004f),
        (0x0aba_8540, 0x0000_017f),
    ],
];

pub const MODE0_INITIALIZATION_LIST: &[(u32, u32)] = &[
    (0x0ab8_0400, 0x55f4_282b),
    (0x0ab8_0410, 0x0000_003a),
    (0x0ab8_0c20, 0),
    (0x0aba_803c, 0x7787_1f1b),
    (0x0ab8_0404, 0x0010_137a),
    (0x0ab8_0414, 0x0000_00ab),
    (0x0ab8_0418, 0),
    (0x0ab8_041c, 0x0008_0ea0),
    (0x0abc_8004, 0x0000_0109),
    (0x0aba_8048, 0x031f_d6f0),
];

unsafe fn write_u8(address: usize, value: u8) {
    unsafe { (address as *mut u8).write_volatile(value) };
}

unsafe fn write_u16(address: usize, value: u16) {
    unsafe { (address as *mut u16).write_volatile(value) };
}

unsafe fn write_u32(address: usize, value: u32) {
    unsafe { (address as *mut u32).write_volatile(value) };
}

unsafe fn apply_register_list(list: &[(u32, u32)]) {
    for &(address, value) in list {
        unsafe { (address as *mut u32).write_volatile(value) };
    }
}

#[derive(Clone, Copy, Debug, Eq, Immutable, IntoBytes, PartialEq)]
#[repr(C)]
pub struct ChannelProgramRequestWire {
    pub operation: u8,
    pub option: u8,
    pub control: U16,
    pub channel: U16,
    pub link_count: u8,
    pub link_id: u8,
}

/// Exact eight-byte request assembled by vendor scan callback `0x14304` before
/// entering the large channel programmer at `0xf802`.
pub fn build_scan_channel_program_request(
    band: u8,
    flags: u8,
    channel: u16,
) -> ChannelProgramRequestWire {
    ChannelProgramRequestWire {
        operation: 0,
        option: (flags & 4) >> 2,
        control: channel_control_word(band, 0, channel).into(),
        channel: channel.into(),
        link_count: 1,
        link_id: 2,
    }
}

/// Vendor `0xfdfa` channel-control word used by scan callback `0x14304`.
pub fn channel_control_word(band: u8, bandwidth_mode: u8, channel: u16) -> u16 {
    let mut control = if band == 1 { 0x26 } else { 0x17 };
    control |= match bandwidth_mode {
        0 => 0x100,
        1 => 0x200,
        2 => 0x400,
        _ => 0,
    };
    if channel & 0x100 != 0 {
        control |= 0x40;
    }
    control
}

/// Vendor `0x124c0` classifier stored at `0x04003c20` before `0xf802`.
pub fn channel_control_gate(control: u16) -> u32 {
    if (!control & 0x22) == 0 || (control & 0x7f) == 0x12 {
        0x40
    } else {
        1
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChannelTunePlan {
    pub phy_mode: u8,
    pub recalibrate: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModeTransitionAction {
    ReturnUnchanged,
    ClearChannelPending,
    Reinitialize,
}

/// Pure leading decisions from vendor `0xf78c`, called by `0xf802` with the
/// request channel. The resulting mode and recalibration flag are passed to
/// `0x16dd6`, the intact radio-channel transition entry.
pub fn channel_tune_plan(operation: u8, control: u16) -> ChannelTunePlan {
    let phy_mode = if control & 0x20 != 0 {
        3
    } else if control & 0x11 == 0x11 {
        2
    } else {
        0
    };
    let recalibrate = phy_mode != 3 && !matches!(operation, 1 | 2 | 4);
    ChannelTunePlan {
        phy_mode,
        recalibrate,
    }
}

/// Leading branch structure of vendor `0x16b2e`. In the normal mode-zero
/// startup state, a scan request remains in PHY mode 2 and skips the expensive
/// mode/table reinitialization path.
pub fn mode_transition_action(
    current_mode: u8,
    auxiliary_mode_active: bool,
    requested_mode: u8,
    force: bool,
) -> ModeTransitionAction {
    if current_mode == requested_mode && !force {
        if auxiliary_mode_active {
            ModeTransitionAction::ReturnUnchanged
        } else {
            ModeTransitionAction::ClearChannelPending
        }
    } else {
        ModeTransitionAction::Reinitialize
    }
}

/// Vendor `0x1682a` channel-to-frequency mapping for PHY band/mode byte zero.
/// The returned unit is kHz; channel 14 uses its dedicated 2484 MHz value.
pub fn channel_frequency_khz_2ghz(channel: u16) -> u32 {
    if channel < 14 {
        (2407 + u32::from(channel) * 5) * 1000
    } else {
        2484 * 1000
    }
}

/// Arithmetic from vendor `0x17224`, which programs `0x0ab88020` during an RF
/// channel transition. `reference_clock_khz` is the output of `0x1682a`.
pub fn channel_measurement_timing(mode: u8, reference_clock_khz: u32) -> Option<i32> {
    let clock_mhz = reference_clock_khz / 1000;
    if clock_mhz == 0 {
        return None;
    }
    let periods = if mode == 0 { 20 } else { 10 };
    Some(-(((periods << 14) / clock_mhz) as i32))
}

/// Signed MHz offset cached by vendor `0x19928` after channel calibration.
/// Mode zero is relative to channel 7's 2442 MHz center frequency.
pub fn channel_frequency_offset_mhz(mode: u8, frequency_khz: u32) -> i16 {
    let nominal_mhz = if mode == 0 { 0x098a } else { 0x157c };
    ((frequency_khz / 1000) as i32 - nominal_mhz) as i16
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PllDivider {
    pub integer: u32,
    pub fractional: u32,
    pub register: u32,
}

/// Pure translation of vendor `0x18f2c -> 0x18ef0`. It computes the integer
/// and 21-bit fractional PLL divider later packed into `0x0abc00b4`.
pub fn pll_divider(frequency_khz: u32, multiplier: u32, crystal_khz: u32) -> Option<PllDivider> {
    if crystal_khz == 0 {
        return None;
    }
    let product = u64::from(frequency_khz) * u64::from(multiplier);
    let integer = product / u64::from(crystal_khz);
    let remainder = product % u64::from(crystal_khz);
    let fractional = (remainder << 21) / u64::from(crystal_khz);
    let integer = integer as u32;
    let fractional = fractional as u32;
    Some(PllDivider {
        integer,
        fractional,
        register: integer.wrapping_shl(21) | fractional,
    })
}

pub fn derive_remap_timing(remap: u32) -> (u16, u16) {
    let mut first = ((remap << 2) >> 23) as i16;
    let mut second = ((remap << 11) >> 24) as i16;
    if first & 0x100 != 0 {
        first = -(first & !0x100);
    }
    if second & 0x80 != 0 {
        second = -(second & !0x80);
    }
    (
        first.wrapping_add(1000) as u16,
        second.wrapping_mul(71) as u16,
    )
}

/// Software-state translation of vendor `0x16a38`, `0x198f2`, and `0x16ca4`
/// for the XR819 mode selected by the vendor version string.
///
/// # Safety
///
/// The vendor BSS range and boot/remap state must already be initialized.
pub unsafe fn initialize_mac_software_state() {
    const STATE: usize = 0x0400_994c;

    unsafe {
        write_u8(STATE, 2);
        write_u8(STATE + 1, 3);
        write_u8(STATE + 2, 0);
        write_u8(STATE + 3, 2);
        write_u16(STATE + 6, 7);
        write_u32(STATE + 0x28, 0x0025_4310);
        write_u8(STATE + 0x11, 0);
        write_u8(STATE + 0x12, 0);
        write_u8(STATE + 0x13, 0);
        write_u8(STATE + 0x14, 0);
        write_u8(STATE + 0x15, 0);
        write_u16(STATE + 0x16, 7);
        write_u32(STATE + 0x20, 0x0000_6590);
        write_u32(STATE + 0x24, 0);
        write_u32(STATE + 0x38, 0);
        write_u16(STATE + 0x3c, 0);
        write_u32(STATE + 0x48, 0x0000_9470);
        write_u32(STATE + 0x4c, 0x0000_bb80);
        write_u8(STATE + 0x0b, 40);
        write_u8(STATE + 0x0c, 2);
        write_u8(STATE + 0x0d, 0);
        write_u16(STATE + 0x0e, 10);
        write_u16(STATE + 0x36, 10);

        // Vendor startup mode bit 1 controls this compatibility flag.
        let startup_mode = (0x0400_1fe6 as *const u16).read_volatile();
        if startup_mode & 2 == 0 {
            write_u8(STATE + 0x15, 1);
        }

        write_u8(0x0400_997c, 0);
        write_u8(0x0400_998c, 14);
        write_u8(0x0400_99a9, 0);
        write_u16(0x0400_99ce, 100);
        write_u8(0x0400_99d0, 1);
        write_u8(0x0400_99d1, 0);
        write_u8(0x0400_99f4 + 0x14, 1);

        // Vendor 0x198f2 mode-zero state pointers.
        write_u32(0x0400_99d8, 0x0400_34b0);
        write_u32(0x0400_99ec, 0x0400_1088);
        write_u32(0x0400_99f0, 0x0400_1098);
        write_u32(0x0400_99f4, u32::MAX);
        write_u8(0x0400_99f4 + 0x15, 0);

        // Vendor 0x16ca4 derives these from remap window two at 0x04001ffc.
        let remap = (0x0400_1ffc as *const u32).read_volatile();
        let (first, second) = derive_remap_timing(remap);
        write_u16(0x0400_9990, first);
        write_u16(0x0400_9992, second);
    }
}

/// Hardware-producing portion of vendor `0x171ce -> 0x17008` for mode zero.
///
/// # Safety
///
/// Packet DMA, interrupt routing, and callback-table entries must already be
/// installed. The current startup calls this only for static mode-zero hardware
/// preparation; channel policy remains inactive while callbacks are diagnostic
/// stubs.
pub unsafe fn initialize_mac_core_mode0() {
    for index in 0..=0x40 {
        unsafe { ((0x0aba_2000 + index * 4) as *mut u32).write_volatile(0x00ed_00ed) };
    }

    unsafe {
        (0x0aba_805c as *mut u32).write_volatile(0x2708_2026);
        let silicon_mode = (0x0400_1fbc as *const u8).read_volatile();
        let timing = if silicon_mode == 2 {
            0x03a0_00a0
        } else {
            0x03a0_0710
        };
        (0x0ab8_0c08 as *mut u32).write_volatile(timing);
        (0x0ab8_0c3c as *mut u32).write_volatile(0xff03_f7a0);
        (0x0ab8_0064 as *mut u32).write_volatile(0x0021_0280);
    }

    for list in COMMON_INITIALIZATION_LISTS {
        unsafe { apply_register_list(list) };
    }

    let correction = unsafe { (0x0400_34f8 as *const i16).read_volatile() };
    let mut table = [0; 80];
    build_rate_table(&MODE0_CALIBRATION_ANCHORS, correction, &mut table);
    for (index, value) in table.iter().copied().enumerate() {
        unsafe { ((0x0ab8_0800 + index * 4) as *mut u32).write_volatile(value) };
    }

    unsafe { apply_register_list(MODE0_INITIALIZATION_LIST) };

    unsafe {
        let control = (0x0ab8_0400 as *mut u32).read_volatile();
        (0x0ab8_0400 as *mut u32)
            .write_volatile((control & 0xffff_80ff) | (table[0] & 0x0000_7f00));
        let boundary = (0x0ab8_0410 as *mut u32).read_volatile();
        (0x0ab8_0410 as *mut u32)
            .write_volatile((boundary & 0xffff_ff80) | u32::from(rate_table_boundary(&table)));

        let mac_control = (0x0ab8_0c00 as *mut u32).read_volatile();
        (0x0ab8_0c00 as *mut u32).write_volatile(mac_control | (1 << 11));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vendor_calibration_tables_have_all_anchors() {
        assert_eq!(MODE0_CALIBRATION_ANCHORS.len(), 22);
        assert_eq!(MODE1_CALIBRATION_ANCHORS.len(), 22);
        assert_eq!(MODE0_CALIBRATION_ANCHORS[0].selector, 26);
        assert_eq!(MODE1_CALIBRATION_ANCHORS[21].lower, -1547);
    }

    #[test]
    fn normalization_matches_vendor_signed_rounding() {
        assert_eq!(
            normalize_anchor(MODE0_CALIBRATION_ANCHORS[0], 0),
            CalibrationAnchor {
                selector: 26,
                upper: 0,
                lower: -64,
            }
        );
    }

    #[test]
    fn channel_control_matches_vendor_scan_mode() {
        assert_eq!(channel_control_word(0, 0, 1), 0x0117);
        assert_eq!(channel_control_word(1, 2, 0x101), 0x0466);
        assert_eq!(channel_control_gate(0x0117), 1);
        assert_eq!(channel_control_gate(0x0122), 0x40);
        assert_eq!(channel_control_gate(0x0112), 0x40);
        assert_eq!(
            channel_tune_plan(0, 0x0117),
            ChannelTunePlan {
                phy_mode: 2,
                recalibrate: true,
            }
        );
        assert_eq!(
            channel_tune_plan(0, 0x0137),
            ChannelTunePlan {
                phy_mode: 3,
                recalibrate: false,
            }
        );
        assert!(!channel_tune_plan(2, 0x0117).recalibrate);
        assert_eq!(
            mode_transition_action(2, false, 2, false),
            ModeTransitionAction::ClearChannelPending
        );
        assert_eq!(
            mode_transition_action(2, true, 2, false),
            ModeTransitionAction::ReturnUnchanged
        );
        assert_eq!(
            mode_transition_action(2, false, 3, false),
            ModeTransitionAction::Reinitialize
        );
        assert_eq!(channel_frequency_khz_2ghz(1), 2_412_000);
        assert_eq!(channel_frequency_khz_2ghz(6), 2_437_000);
        assert_eq!(channel_frequency_khz_2ghz(13), 2_472_000);
        assert_eq!(channel_frequency_khz_2ghz(14), 2_484_000);
        assert_eq!(
            channel_measurement_timing(0, channel_frequency_khz_2ghz(7)),
            Some(-134)
        );
        assert_eq!(channel_frequency_offset_mhz(0, 2_442_000), 0);
        assert_eq!(channel_frequency_offset_mhz(0, 2_437_000), -5);
        assert_eq!(channel_frequency_offset_mhz(1, 2_442_000), -3058);
        assert_eq!(
            pll_divider(channel_frequency_khz_2ghz(6), 1250, 26_000),
            Some(PllDivider {
                integer: 117_163,
                fractional: 967_916,
                register: 0x356e_c4ec,
            })
        );

        let request = build_scan_channel_program_request(0, 4, 6);
        assert_eq!(request.as_bytes(), &[0, 1, 0x17, 0x01, 6, 0, 1, 2]);
    }

    #[test]
    fn remap_timing_matches_vendor_window_two() {
        assert_eq!(derive_remap_timing(0x0411_8000), (1032, (-852_i16) as u16));
    }

    #[test]
    fn mode0_rate_table_matches_vendor_generation() {
        let mut table = [0; 80];
        build_rate_table(&MODE0_CALIBRATION_ANCHORS, 0, &mut table);

        assert_eq!(table[0], 0x2b20_2b20);
        assert_eq!(table[54], 0x0a14_0a14);
        assert_eq!(table[62], 0x0219_0219);
        assert_eq!(table[79], 0x001a_001a);
        assert_eq!(rate_table_boundary(&table), 55);
    }
}
