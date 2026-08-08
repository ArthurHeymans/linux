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

/// Twelve gain-table indices copied by the vendor container to `0x04000e18`.
pub const IQ_CALIBRATION_GAIN_INDICES: [u32; 12] = [
    0x1a, 0x19, 0x18, 0x16, 0x15, 0x14, 0x12, 0x11, 0x10, 0x02, 0x01, 0x00,
];

/// Exact signed fixed-point cosine/sine tables used by target `0x18480`.
/// The 256-byte sequence is identical in the annotated and target images.
const DYNAMIC_IQ_COSINE: [i16; 64] = [
    32767, 32609, 32137, 31356, 30273, 28898, 27245, 25329, 23170, 20787, 18204, 15446, 12539,
    9512, 6393, 3212, 0, -3212, -6393, -9512, -12539, -15446, -18204, -20787, -23170, -25329,
    -27245, -28898, -30273, -31356, -32137, -32609, -32767, -32609, -32137, -31356, -30273, -28898,
    -27245, -25329, -23170, -20787, -18204, -15446, -12539, -9512, -6393, -3212, 0, 3212, 6393,
    9512, 12539, 15446, 18204, 20787, 23170, 25329, 27245, 28898, 30273, 31356, 32137, 32609,
];

const DYNAMIC_IQ_SINE: [i16; 64] = [
    0, 3212, 6393, 9512, 12539, 15446, 18204, 20787, 23170, 25329, 27245, 28898, 30273, 31356,
    32137, 32609, 32767, 32609, 32137, 31356, 30273, 28898, 27245, 25329, 23170, 20787, 18204,
    15446, 12539, 9512, 6393, 3212, 0, -3212, -6393, -9512, -12539, -15446, -18204, -20787, -23170,
    -25329, -27245, -28898, -30273, -31356, -32137, -32609, -32767, -32609, -32137, -31356, -30273,
    -28898, -27245, -25329, -23170, -20787, -18204, -15446, -12539, -9512, -6393, -3212,
];

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

fn delay_units(units: u32) {
    let iterations = units.saturating_mul(0x22);
    for _ in 0..iterations {
        core::hint::spin_loop();
    }
}

fn delay_timer_ticks(ticks: u32) {
    const TIMER: *const u32 = 0x0ac0_0004 as *const u32;
    let target = unsafe { TIMER.read_volatile() }.wrapping_add(ticks);
    loop {
        let now = unsafe { TIMER.read_volatile() };
        if now.wrapping_sub(target) as i32 > 0 {
            break;
        }
    }
}

/// Complete `0x18f2c -> 0x1838c` PLL publication sequence. It remains
/// intentionally detached from scan execution until the surrounding
/// calibration path is ready.
pub unsafe fn commit_channel_pll(register: u32, mode: u8, extended_settle: bool) {
    const PLL_VALUE: usize = 0x0abc_00b4;
    const PLL_CONTROL: usize = 0x0abc_00b8;
    const MODE_CONTROL: usize = 0x0abc_0084;
    const BAND_CONTROL: usize = 0x0abc_0050;

    unsafe {
        write_u32(PLL_VALUE, register);
        let control = (PLL_CONTROL as *const u32).read_volatile() & !0x2000;
        write_u32(PLL_CONTROL, control);
        delay_units(1);
        write_u32(PLL_CONTROL, control | 0x2000);
        if extended_settle {
            delay_units(0x78);
        }
        if mode == 0 {
            write_u32(
                MODE_CONTROL,
                (MODE_CONTROL as *const u32).read_volatile() & !0x40,
            );
            write_u32(
                BAND_CONTROL,
                (BAND_CONTROL as *const u32).read_volatile() & !0x200,
            );
            delay_units(10);
            write_u32(
                MODE_CONTROL,
                (MODE_CONTROL as *const u32).read_volatile() | 0x40,
            );
            write_u32(
                BAND_CONTROL,
                (BAND_CONTROL as *const u32).read_volatile() | 0x200,
            );
            delay_units(10);
        }
    }
}

/// Complete enable/disable behavior from vendor `0x168b8` for state mode 2.
pub unsafe fn set_calibration_engine_enabled(enabled: bool) {
    unsafe {
        let offset = (0x0400_9984 as *const u32).read_volatile() as usize;
        let control_address = 0x0abb_8004 + offset;
        let mut control = (control_address as *const u32).read_volatile();
        if enabled {
            control |= 0x0004_8000;
        } else {
            control &= !0x0004_8000;
        }
        write_u32(control_address, control);
    }
}

pub fn calibration_gain_value(gain: i32) -> u32 {
    if gain < 0 {
        0
    } else {
        0x40 | (gain as u32 & 0x3f)
    }
}

/// Vendor `0x17884` calibration gain selector.
pub unsafe fn set_calibration_gain(gain: i32) {
    unsafe { write_u32(0x0abb_81a4, calibration_gain_value(gain)) };
}

pub fn calibration_mode_timing(mode: u8) -> u32 {
    if matches!(mode, 2 | 3) {
        0x0020_0078
    } else {
        0x0020_0190
    }
}

/// Deterministic low-reserved-bit form of vendor `0x17bf2`.
pub fn calibration_sample_settings(i: u8, q: u8) -> u32 {
    0x0100_0100 | u32::from(i) | (u32::from(q) << 16)
}

/// Fixed `0x17b70(0x0b, 1, ...)` trigger word before hardware sets bit `0x10`.
pub const CALIBRATION_SAMPLE_COMMAND: u32 = 0x0800_000d;

/// Detached bounded form of `0x17bf2 -> 0x17b70`.
pub unsafe fn run_calibration_sample(
    i: u8,
    q: u8,
    max_polls: u32,
) -> Result<IqCalibrationCoefficient, CalibrationSampleError> {
    unsafe {
        write_u32(0x0abb_8114, calibration_sample_settings(i, q));
        write_u32(0x0abb_80f0, CALIBRATION_SAMPLE_COMMAND);
        for _ in 0..max_polls {
            if (0x0abb_80f0 as *const u32).read_volatile() & 0x10 != 0 {
                let i = decode_calibration_accumulator((0x0abb_810c as *const u32).read_volatile());
                let q = decode_calibration_accumulator((0x0abb_8110 as *const u32).read_volatile());
                return Ok(IqCalibrationCoefficient { i, q });
            }
        }
    }
    Err(CalibrationSampleError::Timeout)
}

/// Complete `0x178be -> 0x17898` calibration mode programming.
pub unsafe fn configure_calibration_mode(mode: u8) {
    let timing = calibration_mode_timing(mode);
    unsafe {
        write_u32(0x0abb_80f0, 0x0100_4008 + u32::from(mode & 3));
        write_u32(0x0abb_80f4, timing);
        write_u32(0x0abb_80f8, 0x012c_00c8);
    }
}

pub fn measurement_path_value(band_mode: u8) -> u32 {
    if band_mode == 1 { 0 } else { 0x0ebf }
}

/// Result scaling from the mode-zero round in vendor `0x19f8e`.
pub fn scale_channel_measurement(raw_counter: u32, first: i16, second: i16) -> Option<i32> {
    if first == 0 {
        return None;
    }
    let numerator = (i64::from(raw_counter) * 0x47 - i64::from(second)) * 1000;
    Some((numerator / i64::from(first)) as i32)
}

/// Vendor `0x19f8e` accepts mode-zero results from 41000 through 58988 and
/// otherwise uses fallback `0x9470` (38000).
pub fn select_mode0_measurement(scaled: i32) -> (u32, bool) {
    if (41_000..=58_988).contains(&scaled) {
        (scaled as u32, true)
    } else {
        (0x9470, false)
    }
}

/// Mode-zero trigger word and completion predicate from vendor `0x19f8e`.
pub const MODE0_MEASUREMENT_CONTROL: u32 = 0x0000_e080;

pub fn channel_measurement_ready(control: u32) -> bool {
    control & 0x20 == 0
}

/// Complete MMIO sequence from vendor `0x17e92`.
pub unsafe fn set_channel_measurement_path(enabled: bool, band_mode: u8) {
    const MAC_CONTROL: usize = 0x0ab8_0c38;
    unsafe {
        let mut control = (MAC_CONTROL as *const u32).read_volatile();
        if enabled {
            control |= 0x1800;
            write_u32(0x0abb_81a0, 0);
            write_u32(0x0ab8_006c, measurement_path_value(band_mode));
            write_u32(0x0ab8_0068, 0);
        } else {
            control &= !0x1800;
        }
        write_u32(MAC_CONTROL, control);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChannelMeasurementSnapshot {
    abb800c: u32,
    abc0004: u32,
    abc0034: u32,
    abc0050: u32,
    abc006c: u32,
    abc0084: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChannelMeasurementError {
    InvalidCalibration,
    NotReady,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CalibrationSampleError {
    Timeout,
}

/// Register save-and-override envelope from vendor `0x1a1fc`, followed by the
/// `0x19f8e` save of `0x0abb800c`.
pub unsafe fn begin_channel_measurement() -> ChannelMeasurementSnapshot {
    unsafe {
        let snapshot = ChannelMeasurementSnapshot {
            abb800c: (0x0abb_800c as *const u32).read_volatile(),
            abc0004: (0x0abc_0004 as *const u32).read_volatile(),
            abc0034: (0x0abc_0034 as *const u32).read_volatile(),
            abc0050: (0x0abc_0050 as *const u32).read_volatile(),
            abc006c: (0x0abc_006c as *const u32).read_volatile(),
            abc0084: (0x0abc_0084 as *const u32).read_volatile(),
        };

        write_u32(
            0x0abb_8004,
            (0x0abb_8004 as *const u32).read_volatile() | 0x800,
        );
        write_u32(
            0x0abc_0084,
            (0x0abc_0098 as *const u32).read_volatile() | 0x0700_0200,
        );
        write_u32(0x0abc_006c, (0x0abc_0080 as *const u32).read_volatile());
        write_u32(0x0abc_0050, (0x0abc_0064 as *const u32).read_volatile());
        write_u32(
            0x0abc_0034,
            (0x0abc_0048 as *const u32).read_volatile() | 0x101,
        );
        write_u32(0x0abc_0004, snapshot.abc0004 | 2);
        snapshot
    }
}

/// Exact register restoration tail from vendor `0x19f8e`.
pub unsafe fn end_channel_measurement(snapshot: ChannelMeasurementSnapshot) {
    unsafe {
        write_u32(0x0abc_0004, snapshot.abc0004);
        write_u32(0x0abc_0084, snapshot.abc0084);
        write_u32(0x0abc_006c, snapshot.abc006c);
        write_u32(0x0abc_0050, snapshot.abc0050);
        write_u32(0x0abc_0034, snapshot.abc0034);
        write_u32(0x0abb_800c, snapshot.abb800c);
    }
}

/// Complete mode-zero measurement round from vendor `0x19f8e`. Like the
/// vendor function, a not-ready result returns before restoring the temporary
/// register envelope. Callers must treat that result as a fatal channel setup
/// failure rather than continuing with partially restored hardware.
pub unsafe fn run_mode0_channel_measurement(
    first: i16,
    second: i16,
) -> Result<u32, ChannelMeasurementError> {
    if first == 0 {
        return Err(ChannelMeasurementError::InvalidCalibration);
    }

    unsafe {
        let snapshot = begin_channel_measurement();
        write_u32(0x0abb_800c, 0);
        write_u32(0x0abb_800c, MODE0_MEASUREMENT_CONTROL);
        delay_timer_ticks(10);
        if !channel_measurement_ready((0x0abb_800c as *const u32).read_volatile()) {
            return Err(ChannelMeasurementError::NotReady);
        }
        delay_timer_ticks(10);

        let raw = (0x0abb_82f0 as *const u32).read_volatile();
        let scaled = scale_channel_measurement(raw, first, second)
            .ok_or(ChannelMeasurementError::InvalidCalibration)?;
        let (selected, _) = select_mode0_measurement(scaled);
        delay_timer_ticks(1);
        end_channel_measurement(snapshot);
        Ok(selected)
    }
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChannelThresholdCorrection {
    pub upper_channel: u16,
    pub value: i16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChannelCalibrationStep {
    pub upper_channel: u8,
    pub first: u8,
    pub second: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IqCalibrationSample {
    pub baseline_i: i32,
    pub baseline_q: i32,
    pub target_i: i32,
    pub target_q: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IqCalibrationCoefficient {
    pub i: i32,
    pub q: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IqCalibrationControls {
    pub during_settle: u32,
    pub active: u32,
    pub abc0034: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IqCalibrationRegisterSnapshot {
    abc0004: u32,
    abc0034: u32,
    abc0050: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PublishedIqCoefficient {
    pub shift: u8,
    pub normalized_i: i32,
    pub normalized_q: i32,
    pub packed: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IqCalibrationPublication {
    pub primary_address: u32,
    pub normalized_first_address: u32,
    pub normalized_second_address: u32,
    pub primary_value: u16,
    pub normalized_value: u32,
    pub next_shift_state: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IqCalibrationIteration {
    pub gain_index: u32,
    pub sample: IqCalibrationSample,
    pub scale_i: i32,
    pub scale_q: i32,
    pub coefficient: IqCalibrationCoefficient,
    pub publication: Option<IqCalibrationPublication>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IqCalibrationSeries {
    pub iterations: [IqCalibrationIteration; 12],
    pub final_shift_state: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DynamicIqInitialCandidate {
    pub first: i8,
    pub second: i8,
    pub third: i8,
    pub fourth: i8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DynamicIqCorrectionPlan {
    pub first_address: u32,
    pub second_address: u32,
    pub first_value: u32,
    pub second_value: u32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DynamicIqCorrelation {
    pub real: i16,
    pub imaginary: i16,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DynamicIqDftResult {
    pub first: DynamicIqCorrelation,
    pub second: DynamicIqCorrelation,
    pub third: DynamicIqCorrelation,
}

/// Typed state for annotated `rf_op_dispatch2` (`0x17f34`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DynamicIqSearchState {
    pub candidate: [i32; 4],
    pub current: [i32; 4],
    pub common_step: i32,
    pub first_step: i32,
    pub second_step: i32,
    pub reference_correlation: DynamicIqCorrelation,
    pub common_correlations: [DynamicIqCorrelation; 6],
    pub axis_correlations: [DynamicIqCorrelation; 5],
}

impl DynamicIqSearchState {
    pub const fn new(
        current: [i32; 4],
        common_step: i32,
        first_step: i32,
        second_step: i32,
    ) -> Self {
        Self {
            candidate: current,
            current,
            common_step,
            first_step,
            second_step,
            reference_correlation: DynamicIqCorrelation {
                real: 0,
                imaginary: 0,
            },
            common_correlations: [DynamicIqCorrelation {
                real: 0,
                imaginary: 0,
            }; 6],
            axis_correlations: [DynamicIqCorrelation {
                real: 0,
                imaginary: 0,
            }; 5],
        }
    }
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

/// Pure table walk from vendor `0x19dd0`. It returns the value belonging to
/// the last threshold not greater than the channel, or the table default.
pub fn select_threshold_correction(
    channel: u16,
    default: i16,
    entries: &[ChannelThresholdCorrection],
) -> i16 {
    entries
        .iter()
        .take_while(|entry| entry.upper_channel <= channel)
        .last()
        .map_or(default, |entry| entry.value)
}

/// Pure three-byte channel-step lookup from vendor `0x1a112`.
pub fn select_channel_calibration(
    channel: u8,
    first: bool,
    base: i16,
    entries: &[ChannelCalibrationStep],
) -> Option<i16> {
    let mut selected = entries.first()?;
    for (index, entry) in entries.iter().enumerate() {
        if channel <= entry.upper_channel {
            selected = if channel < entry.upper_channel && index != 0 {
                &entries[index - 1]
            } else {
                entry
            };
            break;
        }
    }
    let value = if first {
        selected.first
    } else {
        selected.second
    };
    Some(base.wrapping_add(i16::from(value) * 4))
}

fn primary_iq_axis(baseline: i32, target: i32) -> i32 {
    let mut delta = target.wrapping_sub(baseline).wrapping_mul(-0x100);
    if delta == 0 {
        delta = 1;
    }
    baseline
        .wrapping_neg()
        .wrapping_shl(14)
        .wrapping_div(delta)
        .wrapping_add(0x44)
}

/// Core coefficient calculation repeated twelve times by vendor `0x17c20`.
pub fn primary_iq_calibration(sample: IqCalibrationSample) -> IqCalibrationCoefficient {
    IqCalibrationCoefficient {
        i: primary_iq_axis(sample.baseline_i, sample.target_i),
        q: primary_iq_axis(sample.baseline_q, sample.target_q),
    }
}

/// Refinement coefficient calculation at the second measurement stage of
/// vendor `0x17c20`.
pub fn secondary_iq_calibration(
    baseline_i: i32,
    baseline_q: i32,
    target_i: i32,
    target_q: i32,
    scale_i: i32,
    scale_q: i32,
) -> Option<IqCalibrationCoefficient> {
    if scale_i == 0 || scale_q == 0 {
        return None;
    }
    Some(IqCalibrationCoefficient {
        i: baseline_i
            .wrapping_sub(target_i)
            .wrapping_mul(0x4000)
            .wrapping_div(scale_i),
        q: baseline_q
            .wrapping_sub(target_q)
            .wrapping_mul(0x4000)
            .wrapping_div(scale_q),
    })
}

/// Surviving four-byte candidate seed at `0x189a2..0x189da`. Earlier bytes in
/// the branch are overwritten before the search begins.
pub fn dynamic_iq_initial_candidate(mode: u8) -> DynamicIqInitialCandidate {
    if matches!(mode, 1 | 2) {
        DynamicIqInitialCandidate {
            first: 7,
            second: -7,
            third: -5,
            fourth: 1,
        }
    } else {
        DynamicIqInitialCandidate {
            first: -4,
            second: -11,
            third: -4,
            fourth: 0,
        }
    }
}

fn pack_dynamic_iq_pair(first: i32, second: i32) -> u32 {
    (first as u32 & 0x0fff) | ((second as u32 & 0x0fff) << 16)
}

/// Pure register plan from vendor `0x18612` for four signed correction values.
pub fn dynamic_iq_correction_plan(values: [i32; 4]) -> DynamicIqCorrectionPlan {
    DynamicIqCorrectionPlan {
        first_address: 0x0abb_8068,
        second_address: 0x0abb_80a8,
        first_value: pack_dynamic_iq_pair(values[0], values[1]),
        second_value: pack_dynamic_iq_pair(values[2], values[3]),
    }
}

/// Candidate normalization and rejection at target `0x18c3e..0x18cb8`.
pub fn normalize_dynamic_iq_candidate(mut values: [i32; 4]) -> [i32; 4] {
    for value in &mut values[..2] {
        if *value > 0x1ff {
            *value = value.wrapping_sub(0x400);
        }
        if !(-0x200..=0x200).contains(value) {
            *value = 0;
        }
    }
    for value in &mut values[2..] {
        if *value > 0x7ff {
            *value = value.wrapping_sub(0x1000);
        }
        if !(-0x800..=0x800).contains(value) {
            *value = 0;
        }
    }
    values
}

fn dynamic_iq_metrics<const N: usize>(correlations: [DynamicIqCorrelation; N]) -> [i32; N] {
    let mut maximum = 0_i32;
    for correlation in correlations {
        maximum = maximum
            .max(i32::from(correlation.real).unsigned_abs() as i32)
            .max(i32::from(correlation.imaginary).unsigned_abs() as i32);
    }
    let component_shift = if maximum >> 15 == 0 {
        0
    } else {
        32 - (maximum as u32 >> 15).leading_zeros()
    };
    let mut metrics = [0_i32; N];
    let mut metric_maximum = 0_i32;
    for (index, correlation) in correlations.iter().enumerate() {
        let real = i32::from(correlation.real) >> component_shift;
        let imaginary = i32::from(correlation.imaginary) >> component_shift;
        metrics[index] = real
            .wrapping_mul(real)
            .wrapping_add(imaginary.wrapping_mul(imaginary));
        metric_maximum = metric_maximum.max(metrics[index]);
    }
    let metric_shift = if metric_maximum >> 13 == 0 {
        0
    } else {
        32 - (metric_maximum as u32 >> 13).leading_zeros()
    };
    for metric in &mut metrics {
        *metric >>= metric_shift;
    }
    metrics
}

/// Analytical stage 6 of the thirteen-stage dynamic IQ search.
pub fn refine_dynamic_iq_common_pair(
    current_third: i32,
    current_fourth: i32,
    step: i32,
    correlations: [DynamicIqCorrelation; 6],
) -> (i32, i32) {
    let [m0, m1, m2, m3, m4, m5] = dynamic_iq_metrics(correlations);
    let denominator = m2
        .wrapping_mul(m2)
        .wrapping_add(m3.wrapping_mul(m0.wrapping_mul(2).wrapping_sub(m1).wrapping_sub(m2)))
        .wrapping_add(m4.wrapping_mul(m4.wrapping_add(m2).wrapping_sub(m1)))
        .wrapping_add(
            m5.wrapping_mul(
                m5.wrapping_add(m0.wrapping_mul(2))
                    .wrapping_sub(m2.wrapping_mul(2))
                    .wrapping_sub(m4.wrapping_mul(2)),
            ),
        )
        .wrapping_mul(2)
        .wrapping_add(m0.wrapping_mul(m1.wrapping_mul(4).wrapping_sub(m0.wrapping_mul(6))));

    let mut third = current_third;
    if m1.wrapping_sub(m0.wrapping_mul(2)).wrapping_add(m2) > 0 && denominator != 0 {
        let numerator = m0
            .wrapping_mul(2)
            .wrapping_mul(m2.wrapping_sub(m1))
            .wrapping_add(m3.wrapping_mul(m1.wrapping_sub(m0).wrapping_sub(m5)))
            .wrapping_add(
                m4.wrapping_mul(
                    m0.wrapping_add(m1)
                        .wrapping_sub(m2.wrapping_mul(2))
                        .wrapping_add(m5)
                        .wrapping_add(m3)
                        .wrapping_sub(m4),
                ),
            );
        third = third.wrapping_sub(step.wrapping_mul(numerator).wrapping_div(denominator));
    }

    let mut fourth = current_fourth;
    if m3.wrapping_sub(m0.wrapping_mul(2)).wrapping_add(m4) > 0 && denominator != 0 {
        let numerator = m0
            .wrapping_mul(
                m2.wrapping_sub(m1)
                    .wrapping_add(m4.wrapping_sub(m3).wrapping_mul(2)),
            )
            .wrapping_add(
                m2.wrapping_mul(
                    m1.wrapping_sub(m2)
                        .wrapping_add(m3)
                        .wrapping_sub(m4.wrapping_mul(2))
                        .wrapping_add(m5),
                ),
            )
            .wrapping_add(m1.wrapping_mul(m3.wrapping_sub(m5)));
        fourth = fourth.wrapping_sub(step.wrapping_mul(numerator).wrapping_div(denominator));
    }
    (third, fourth)
}

fn clamp_dynamic_iq_delta(delta: i32, step: i32) -> i32 {
    let limit = step.wrapping_mul(4);
    (delta / 2).clamp(limit.wrapping_neg(), limit)
}

/// Analytical stage 12 of the thirteen-stage dynamic IQ search.
pub fn refine_dynamic_iq_axis_pair(
    current_first: i32,
    current_second: i32,
    first_step: i32,
    second_step: i32,
    correlations: [DynamicIqCorrelation; 5],
) -> (i32, i32) {
    let [m0, m1, m2, m3, m4] = dynamic_iq_metrics(correlations);
    let mut first = current_first;
    let first_denominator = m1.wrapping_sub(m0.wrapping_mul(2)).wrapping_add(m2);
    if first_denominator > 0 {
        let delta = first_step
            .wrapping_mul(m1.wrapping_sub(m2))
            .wrapping_div(first_denominator);
        first = first.wrapping_add(clamp_dynamic_iq_delta(delta, first_step));
    }
    let mut second = current_second;
    let second_denominator = m3.wrapping_sub(m0.wrapping_mul(2)).wrapping_add(m4);
    if second_denominator > 0 {
        let delta = second_step
            .wrapping_mul(m3.wrapping_sub(m4))
            .wrapping_div(second_denominator);
        second = second.wrapping_add(clamp_dynamic_iq_delta(delta, second_step));
    }
    (first, second)
}

/// Apply the persistent search-state effects of one switch case from annotated
/// `rf_op_dispatch2`.
///
/// Cases 1 and 8 consume all three DFT outputs for their local magnitude
/// calculations. The retained correlations below mirror the vendor workspace;
/// transient normalized magnitudes remain internal to the refinement helpers.
/// Cases 6 and 12 perform the two analytical refinements.
pub fn apply_dynamic_iq_search_stage(
    state: &mut DynamicIqSearchState,
    stage: u8,
    measurement: DynamicIqDftResult,
) -> bool {
    match stage {
        0 => {
            state.candidate = state.current;
            state.candidate[2] = state.current[2].wrapping_sub(state.common_step);
        }
        1 => {
            state.reference_correlation = measurement.first;
            state.common_correlations[0] = measurement.second;
            state.axis_correlations[0] = measurement.third;
            state.candidate = state.current;
            state.candidate[2] = state.current[2].wrapping_add(state.common_step);
        }
        2 => {
            state.common_correlations[1] = measurement.second;
            state.candidate = state.current;
            state.candidate[2] = state.current[2].wrapping_add(state.common_step);
            state.candidate[3] = state.current[3].wrapping_add(state.common_step);
        }
        3 => {
            state.common_correlations[2] = measurement.second;
            state.candidate = state.current;
            state.candidate[3] = state.current[3].wrapping_sub(state.common_step);
        }
        4 => {
            state.common_correlations[5] = measurement.second;
            state.candidate = state.current;
            state.candidate[3] = state.current[3].wrapping_add(state.common_step);
        }
        5 => state.common_correlations[3] = measurement.second,
        6 => {
            state.common_correlations[4] = measurement.second;
            let (third, fourth) = refine_dynamic_iq_common_pair(
                state.current[2],
                state.current[3],
                state.common_step,
                state.common_correlations,
            );
            state.current[2] = third;
            state.current[3] = fourth;
            state.candidate = state.current;
        }
        7 => {
            state.candidate = state.current;
            state.candidate[1] = state.current[1].wrapping_sub(state.second_step);
        }
        8 => {
            state.common_correlations[0] = measurement.second;
            state.axis_correlations[0] = measurement.third;
            state.candidate = state.current;
            state.candidate[1] = state.current[1].wrapping_add(state.second_step);
        }
        9 => {
            state.axis_correlations[3] = measurement.third;
            state.candidate = state.current;
            state.candidate[0] = state.current[0].wrapping_sub(state.first_step);
        }
        10 => {
            state.axis_correlations[4] = measurement.third;
            state.candidate[0] = state.current[0].wrapping_add(state.first_step);
        }
        11 => state.axis_correlations[1] = measurement.third,
        12 => {
            state.axis_correlations[2] = measurement.third;
            let (first, second) = refine_dynamic_iq_axis_pair(
                state.current[0],
                state.current[1],
                state.first_step,
                state.second_step,
                state.axis_correlations,
            );
            state.current[0] = first;
            state.current[1] = second;
            state.candidate = state.current;
        }
        13 => {}
        _ => return false,
    }
    true
}

/// Control update from vendor `0x18600`, named `phy_set_reg2c_bit8` in the
/// annotated firmware archive.
pub fn dynamic_iq_capture_control(control: u32) -> u32 {
    control | 0x100
}

/// Detached vendor `0x185bc`/annotated `rf_capture_adc_samples`: wait up to the
/// requested poll count for status bit 15 to clear, then copy all 64 ADC words.
/// The vendor proceeds with the copy after timeout, so readiness is returned
/// separately rather than suppressing the captured data.
pub unsafe fn capture_dynamic_iq_samples(output: &mut [u32; 64], max_polls: u32) -> bool {
    let mut ready = false;
    unsafe {
        for _ in 0..max_polls {
            let status = (0x0abb_81ac as *const u32).read_volatile();
            delay_units(1);
            if status & 0x8000 == 0 {
                ready = true;
                break;
            }
        }
        for (index, sample) in output.iter_mut().enumerate() {
            *sample = (0x0abb_81c4 as *const u32).add(index).read_volatile();
        }
    }
    ready
}

fn dynamic_iq_table_scale(table: &[i16; 64], phase: u32, sample: i32) -> i32 {
    sample.wrapping_mul(i32::from(table[(phase & 0x3f) as usize])) >> 5
}

fn dynamic_iq_accumulate(correlation: &mut (i32, i32), phase: &mut u32, step: u32, sample: i32) {
    correlation.0 =
        correlation
            .0
            .wrapping_add(dynamic_iq_table_scale(&DYNAMIC_IQ_COSINE, *phase, sample));
    correlation.1 =
        correlation
            .1
            .wrapping_sub(dynamic_iq_table_scale(&DYNAMIC_IQ_SINE, *phase, sample));
    *phase = phase.wrapping_add(step) & 0x3f;
}

/// Pure translation of target `0x18480` (`rf_dft_correlate_samples` in the
/// annotated archive). It computes up to three signed complex correlations
/// from one 64-word ADC capture according to the vendor measurement mode.
pub fn dynamic_iq_dft(
    samples: &[u32; 64],
    control: u32,
    mode: u8,
    first_phase_seed: u32,
    second_phase_seed: u32,
    third_phase_seed: u32,
    sample_width_shift: u8,
) -> DynamicIqDftResult {
    let multiplier = ((control >> 13) & 3).wrapping_add(1);
    let first_step = (multiplier.wrapping_mul(first_phase_seed) & 0xff) >> 1;
    let second_step = (multiplier.wrapping_mul(second_phase_seed) & 0xff) >> 1;
    let third_step = (multiplier.wrapping_mul(third_phase_seed) & 0xff) >> 1;
    let encoded_width = control >> 22;
    let sample_shift = if encoded_width == 0 {
        0
    } else {
        32 - encoded_width.leading_zeros()
    };
    let sign_threshold = 0x7ff_u32.wrapping_shl(u32::from(sample_width_shift));
    let sample_modulus = 0x1000_u32.wrapping_shl(u32::from(sample_width_shift));

    let mut first = (0_i32, 0_i32);
    let mut second = (0_i32, 0_i32);
    let mut third = (0_i32, 0_i32);
    let mut first_phase = 0_u32;
    let mut second_phase = 0_u32;
    let mut third_phase = 0_u32;

    for &raw in samples {
        let raw = if raw > sign_threshold {
            raw.wrapping_sub(sample_modulus)
        } else {
            raw
        };
        let sample = ((raw.wrapping_shl(4) as i32) >> sample_shift) as i16 as i32;
        if matches!(mode, 1 | 8) {
            dynamic_iq_accumulate(&mut first, &mut first_phase, first_step, sample);
        }
        if matches!(mode, 1 | 8) || mode > 8 {
            dynamic_iq_accumulate(&mut third, &mut third_phase, third_step, sample);
        }
        if mode <= 8 {
            dynamic_iq_accumulate(&mut second, &mut second_phase, second_step, sample);
        }
    }

    DynamicIqDftResult {
        first: DynamicIqCorrelation {
            real: (first.0 >> 16) as i16,
            imaginary: (first.1 >> 16) as i16,
        },
        second: DynamicIqCorrelation {
            real: (second.0 >> 16) as i16,
            imaginary: (second.1 >> 16) as i16,
        },
        third: DynamicIqCorrelation {
            real: (third.0 >> 16) as i16,
            imaginary: (third.1 >> 16) as i16,
        },
    }
}

/// Vendor `0x19534 -> 0x19518`: rounded signed down-scaling followed by a
/// clamp to the requested output width.
pub fn rescale_signed(mut value: i32, output_bits: u8, input_bits: u8) -> i32 {
    let shift = input_bits.saturating_sub(output_bits);
    if shift != 0 {
        value = (value >> (shift - 1)).wrapping_add(1) >> 1;
    }
    let limit = 1_i32 << (output_bits - 1);
    value.clamp(-limit, limit - 1)
}

/// Vendor `0x17b70` sign-extends a 23-bit accumulator and converts it to a
/// rounded, saturated signed 12-bit sample through `0x19534`.
pub fn decode_calibration_accumulator(raw: u32) -> i32 {
    let signed = ((raw << 9) as i32) >> 9;
    rescale_signed(signed, 12, 23)
}

/// Low 16-bit I/Q coefficient packing performed by vendor `0x17ac8` after
/// signed 8-bit saturation.
pub fn pack_iq_signed8_pair(coefficient: IqCalibrationCoefficient) -> u16 {
    let i = rescale_signed(coefficient.i, 8, 8) as u8;
    let q = rescale_signed(coefficient.q, 8, 8) as u8;
    u16::from(i) | (u16::from(q) << 8)
}

/// Pure control-word construction from the enable branch of vendor `0x178ce`.
/// Pure coefficient normalization and packing from vendor `0x179ea`.
pub fn publish_iq_coefficient(
    mut coefficient: IqCalibrationCoefficient,
) -> Option<PublishedIqCoefficient> {
    if coefficient.i == 0 || coefficient.q == 0 {
        return None;
    }
    let mut magnitude = coefficient
        .i
        .unsigned_abs()
        .max(coefficient.q.unsigned_abs());
    let mut shift = 0_u8;
    while magnitude < 0x0004_0000 {
        magnitude = magnitude.wrapping_shl(1);
        coefficient.i = coefficient.i.wrapping_shl(1);
        coefficient.q = coefficient.q.wrapping_shl(1);
        shift = shift.wrapping_add(1);
    }
    let i = rescale_signed((-0x2000_0000_i32).wrapping_div(coefficient.i), 10, 12);
    let q = rescale_signed((-0x2000_0000_i32).wrapping_div(coefficient.q), 10, 12);
    Some(PublishedIqCoefficient {
        shift,
        normalized_i: coefficient.i,
        normalized_q: coefficient.q,
        packed: (i as u32 & 0x1ff) | ((q as u32 & 0x1ff) << 16),
    })
}

/// Exact rotate/subtract shift-state update at the tail of each `0x179ea`
/// publication iteration.
pub fn update_iq_shift_state(mut state: u32, shift: u8) -> u32 {
    let encoded_shift = u32::from(shift).wrapping_shl(28);
    state = state.rotate_right(8);
    state = state.wrapping_sub(encoded_shift);
    state = state.rotate_right(24);
    state = state.rotate_right(4);
    state = state.wrapping_sub(encoded_shift);
    state.rotate_right(28)
}

pub fn iq_calibration_publication(
    gain_index: u32,
    coefficient: IqCalibrationCoefficient,
    shift_state: u32,
) -> Option<IqCalibrationPublication> {
    let normalized = publish_iq_coefficient(coefficient)?;
    let gain_offset = gain_index.wrapping_mul(4);
    Some(IqCalibrationPublication {
        primary_address: 0x0abb_8118_u32.wrapping_add(gain_offset),
        normalized_first_address: 0x0abb_8600_u32.wrapping_add(gain_offset),
        normalized_second_address: 0x0abb_8680_u32.wrapping_add(gain_offset),
        primary_value: pack_iq_signed8_pair(coefficient),
        normalized_value: normalized.packed,
        next_shift_state: update_iq_shift_state(shift_state, normalized.shift),
    })
}

/// Allocation-free twelve-gain arithmetic and publication plan from the main
/// loop of vendor `0x17c20`. Hardware sample acquisition remains separate.
pub fn build_iq_calibration_series(
    samples: [IqCalibrationSample; 12],
    initial_shift_state: u32,
) -> IqCalibrationSeries {
    const EMPTY: IqCalibrationIteration = IqCalibrationIteration {
        gain_index: 0,
        sample: IqCalibrationSample {
            baseline_i: 0,
            baseline_q: 0,
            target_i: 0,
            target_q: 0,
        },
        scale_i: 0,
        scale_q: 0,
        coefficient: IqCalibrationCoefficient { i: 0, q: 0 },
        publication: None,
    };

    let mut iterations = [EMPTY; 12];
    let mut shift_state = initial_shift_state;
    for index in 0..12 {
        let sample = samples[index];
        let coefficient = primary_iq_calibration(sample);
        let publication = iq_calibration_publication(
            IQ_CALIBRATION_GAIN_INDICES[index],
            coefficient,
            shift_state,
        );
        if let Some(publication) = publication {
            shift_state = publication.next_shift_state;
        }
        iterations[index] = IqCalibrationIteration {
            gain_index: IQ_CALIBRATION_GAIN_INDICES[index],
            sample,
            scale_i: sample
                .target_i
                .wrapping_sub(sample.baseline_i)
                .wrapping_mul(-0x100),
            scale_q: sample
                .target_q
                .wrapping_sub(sample.baseline_q)
                .wrapping_mul(-0x100),
            coefficient,
            publication,
        };
    }

    IqCalibrationSeries {
        iterations,
        final_shift_state: shift_state,
    }
}

/// Detached publication half of `0x17ac8 -> 0x179ea`. Vendor ordering is
/// preserved: all primary coefficients are written before normalized values
/// and the evolving shift state.
pub unsafe fn apply_iq_calibration_series(series: &IqCalibrationSeries) {
    unsafe {
        for iteration in &series.iterations {
            let address = 0x0abb_8118_u32.wrapping_add(iteration.gain_index.wrapping_mul(4));
            write_u16(
                address as usize,
                pack_iq_signed8_pair(iteration.coefficient),
            );
        }
        for iteration in &series.iterations {
            if let Some(publication) = iteration.publication {
                write_u32(
                    publication.normalized_first_address as usize,
                    publication.normalized_value,
                );
                write_u32(
                    publication.normalized_second_address as usize,
                    publication.next_shift_state,
                );
            }
        }
    }
}

pub fn iq_calibration_controls(
    original_abc0050: u32,
    path: u8,
    band_mode: u8,
) -> IqCalibrationControls {
    let mut control = (original_abc0050 | 1) & !0x0e;
    control |= 0x1df0 | 0x2000 | 0x4000;
    control &= !0x0006_8000;
    control = control.wrapping_add(0x0004_0000);
    control &= !0x0018_0000;
    control = control.wrapping_add(0x0010_0000);
    control |= 0x0020_0000;
    control &= !0x7f00_0000;
    control &= 0x7fff_ffff;

    if path == 0 {
        control |= 0x0001_0000;
        control &= !0x00c0_0200;
    } else if path == 1 {
        control &= !0x0001_0000;
        if band_mode == 1 {
            control |= 0x0040_0000;
            control &= !0x0080_0000;
        } else {
            control |= 0x0080_0000;
            control &= !0x0040_0000;
        }
        control |= 0x200;
    }

    IqCalibrationControls {
        during_settle: control,
        active: if path == 0 {
            control & !0x0001_0000
        } else {
            control
        },
        abc0034: 0x1fff,
    }
}

/// Complete enable branch of vendor `0x178ce` with a returned restoration
/// snapshot replacing its vendor workspace slots.
pub unsafe fn begin_iq_calibration_path(path: u8, band_mode: u8) -> IqCalibrationRegisterSnapshot {
    unsafe {
        let snapshot = IqCalibrationRegisterSnapshot {
            abc0004: (0x0abc_0004 as *const u32).read_volatile(),
            abc0034: (0x0abc_0034 as *const u32).read_volatile(),
            abc0050: (0x0abc_0050 as *const u32).read_volatile(),
        };
        let controls = iq_calibration_controls(snapshot.abc0050, path, band_mode);
        write_u32(0x0abc_0004, snapshot.abc0004 | 2);
        write_u32(0x0abc_0050, controls.during_settle);
        write_u32(0x0abc_0034, controls.abc0034);
        delay_timer_ticks(10);
        if controls.active != controls.during_settle {
            write_u32(0x0abc_0050, controls.active);
        }
        snapshot
    }
}

/// Exact disable/restore branch of vendor `0x178ce`.
pub unsafe fn end_iq_calibration_path(snapshot: IqCalibrationRegisterSnapshot) {
    unsafe {
        write_u32(0x0abc_0004, snapshot.abc0004);
        write_u32(0x0abc_0050, snapshot.abc0050);
        write_u32(0x0abc_0034, snapshot.abc0034);
    }
}

/// Vendor `0x179c0` clears the sample fields while preserving unrelated bits.
pub unsafe fn clear_calibration_sample_settings() {
    unsafe {
        let value = (0x0abb_8114 as *const u32).read_volatile();
        write_u32(0x0abb_8114, value & !0x03ff_03ff);
    }
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
        assert_eq!(IQ_CALIBRATION_GAIN_INDICES.len(), 12);
        assert_eq!(IQ_CALIBRATION_GAIN_INDICES[0], 0x1a);
        assert_eq!(IQ_CALIBRATION_GAIN_INDICES[11], 0);
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
        assert_eq!(measurement_path_value(0), 0x0ebf);
        assert_eq!(measurement_path_value(1), 0);
        assert_eq!(scale_channel_measurement(715, 1032, -852), Some(50_016));
        assert_eq!(select_mode0_measurement(50_016), (50_016, true));
        assert_eq!(select_mode0_measurement(40_999), (38_000, false));
        assert_eq!(select_mode0_measurement(58_989), (38_000, false));
        assert!(channel_measurement_ready(MODE0_MEASUREMENT_CONTROL));
        assert!(!channel_measurement_ready(MODE0_MEASUREMENT_CONTROL | 0x20));
        assert_eq!(
            pll_divider(channel_frequency_khz_2ghz(6), 1250, 26_000),
            Some(PllDivider {
                integer: 117_163,
                fractional: 967_916,
                register: 0x356e_c4ec,
            })
        );
        let corrections = [
            ChannelThresholdCorrection {
                upper_channel: 3,
                value: 10,
            },
            ChannelThresholdCorrection {
                upper_channel: 8,
                value: 20,
            },
        ];
        assert_eq!(select_threshold_correction(2, -1, &corrections), -1);
        assert_eq!(select_threshold_correction(6, -1, &corrections), 10);
        assert_eq!(select_threshold_correction(11, -1, &corrections), 20);
        let steps = [
            ChannelCalibrationStep {
                upper_channel: 3,
                first: 2,
                second: 4,
            },
            ChannelCalibrationStep {
                upper_channel: 8,
                first: 5,
                second: 7,
            },
        ];
        assert_eq!(select_channel_calibration(2, true, 100, &steps), Some(108));
        assert_eq!(select_channel_calibration(6, false, 100, &steps), Some(116));
        assert_eq!(
            primary_iq_calibration(IqCalibrationSample {
                baseline_i: 10,
                baseline_q: -10,
                target_i: 20,
                target_q: 0,
            }),
            IqCalibrationCoefficient { i: 132, q: 4 }
        );
        assert_eq!(
            secondary_iq_calibration(20, -10, 10, -20, 5, 5),
            Some(IqCalibrationCoefficient {
                i: 32_768,
                q: 32_768,
            })
        );
        assert_eq!(
            dynamic_iq_initial_candidate(0),
            DynamicIqInitialCandidate {
                first: -4,
                second: -11,
                third: -4,
                fourth: 0,
            }
        );
        assert_eq!(
            dynamic_iq_initial_candidate(2),
            DynamicIqInitialCandidate {
                first: 7,
                second: -7,
                third: -5,
                fourth: 1,
            }
        );
        assert_eq!(
            dynamic_iq_correction_plan([-4, -11, -5, 1]),
            DynamicIqCorrectionPlan {
                first_address: 0x0abb_8068,
                second_address: 0x0abb_80a8,
                first_value: 0x0ff5_0ffc,
                second_value: 0x0001_0ffb,
            }
        );
        assert_eq!(dynamic_iq_capture_control(0x1234), 0x1334);
        assert_eq!(dynamic_iq_capture_control(0x1334), 0x1334);
        assert_eq!(
            normalize_dynamic_iq_candidate([0x3ff, 0x801, 0xfff, 0x1801]),
            [-1, 0, -1, 0]
        );
        let mut impulse = [0_u32; 64];
        impulse[0] = 0x7ff;
        let all_correlations = dynamic_iq_dft(&impulse, 0, 1, 1, 1, 1, 0);
        assert_eq!(
            all_correlations,
            DynamicIqDftResult {
                first: DynamicIqCorrelation {
                    real: 511,
                    imaginary: 0,
                },
                second: DynamicIqCorrelation {
                    real: 511,
                    imaginary: 0,
                },
                third: DynamicIqCorrelation {
                    real: 511,
                    imaginary: 0,
                },
            }
        );
        assert_eq!(
            dynamic_iq_dft(&impulse, 0, 2, 1, 1, 1, 0),
            DynamicIqDftResult {
                second: DynamicIqCorrelation {
                    real: 511,
                    imaginary: 0,
                },
                ..DynamicIqDftResult::default()
            }
        );
        assert_eq!(
            dynamic_iq_dft(&impulse, 0, 9, 1, 1, 1, 0),
            DynamicIqDftResult {
                third: DynamicIqCorrelation {
                    real: 511,
                    imaginary: 0,
                },
                ..DynamicIqDftResult::default()
            }
        );
        assert_eq!(
            refine_dynamic_iq_common_pair(
                100,
                200,
                8,
                [
                    DynamicIqCorrelation {
                        real: 1,
                        imaginary: 0
                    },
                    DynamicIqCorrelation {
                        real: 2,
                        imaginary: 0
                    },
                    DynamicIqCorrelation {
                        real: 3,
                        imaginary: 0
                    },
                    DynamicIqCorrelation {
                        real: 4,
                        imaginary: 0
                    },
                    DynamicIqCorrelation {
                        real: 5,
                        imaginary: 0
                    },
                    DynamicIqCorrelation {
                        real: 6,
                        imaginary: 0
                    },
                ],
            ),
            (99, 200)
        );
        assert_eq!(
            refine_dynamic_iq_axis_pair(
                100,
                200,
                8,
                8,
                [
                    DynamicIqCorrelation {
                        real: 1,
                        imaginary: 0
                    },
                    DynamicIqCorrelation {
                        real: 3,
                        imaginary: 0
                    },
                    DynamicIqCorrelation {
                        real: 1,
                        imaginary: 0
                    },
                    DynamicIqCorrelation {
                        real: 3,
                        imaginary: 0
                    },
                    DynamicIqCorrelation {
                        real: 1,
                        imaginary: 0
                    },
                ],
            ),
            (104, 204)
        );

        let measurement = |first, second, third| DynamicIqDftResult {
            first: DynamicIqCorrelation {
                real: first,
                imaginary: 0,
            },
            second: DynamicIqCorrelation {
                real: second,
                imaginary: 0,
            },
            third: DynamicIqCorrelation {
                real: third,
                imaginary: 0,
            },
        };
        let mut search = DynamicIqSearchState::new([10, 20, 30, 40], 2, 3, 4);
        assert!(apply_dynamic_iq_search_stage(
            &mut search,
            0,
            DynamicIqDftResult::default()
        ));
        assert_eq!(search.candidate, [10, 20, 28, 40]);
        apply_dynamic_iq_search_stage(&mut search, 1, measurement(11, 12, 13));
        assert_eq!(search.reference_correlation.real, 11);
        assert_eq!(search.common_correlations[0].real, 12);
        assert_eq!(search.axis_correlations[0].real, 13);
        assert_eq!(search.candidate, [10, 20, 32, 40]);
        apply_dynamic_iq_search_stage(&mut search, 2, measurement(0, 22, 0));
        assert_eq!(search.candidate, [10, 20, 32, 42]);
        apply_dynamic_iq_search_stage(&mut search, 3, measurement(0, 32, 0));
        assert_eq!(search.candidate, [10, 20, 30, 38]);
        apply_dynamic_iq_search_stage(&mut search, 4, measurement(0, 42, 0));
        assert_eq!(search.candidate, [10, 20, 30, 42]);
        apply_dynamic_iq_search_stage(&mut search, 5, measurement(0, 52, 0));
        assert_eq!(
            search.common_correlations.map(|value| value.real),
            [12, 22, 32, 52, 0, 42]
        );
        apply_dynamic_iq_search_stage(&mut search, 7, DynamicIqDftResult::default());
        assert_eq!(search.candidate, [10, 16, 30, 40]);
        apply_dynamic_iq_search_stage(&mut search, 8, measurement(81, 82, 83));
        assert_eq!(search.reference_correlation.real, 11);
        assert_eq!(search.candidate, [10, 24, 30, 40]);
        apply_dynamic_iq_search_stage(&mut search, 9, measurement(0, 0, 93));
        assert_eq!(search.candidate, [7, 20, 30, 40]);
        apply_dynamic_iq_search_stage(&mut search, 10, measurement(0, 0, 103));
        assert_eq!(search.candidate, [13, 20, 30, 40]);
        apply_dynamic_iq_search_stage(&mut search, 11, measurement(0, 0, 113));
        assert_eq!(
            search.axis_correlations.map(|value| value.real),
            [83, 113, 0, 93, 103]
        );
        assert!(!apply_dynamic_iq_search_stage(
            &mut search,
            14,
            DynamicIqDftResult::default()
        ));

        assert_eq!(rescale_signed(100, 6, 8), 25);
        assert_eq!(rescale_signed(200, 6, 8), 31);
        assert_eq!(rescale_signed(-200, 6, 8), -32);
        assert_eq!(calibration_gain_value(-1), 0);
        assert_eq!(calibration_gain_value(0x20), 0x60);
        assert_eq!(calibration_gain_value(0x7f), 0x7f);
        assert_eq!(calibration_mode_timing(0), 0x0020_0190);
        assert_eq!(calibration_mode_timing(2), 0x0020_0078);
        assert_eq!(calibration_sample_settings(0x11, 0x11), 0x0111_0111);
        assert_eq!(calibration_sample_settings(1, 1), 0x0101_0101);
        assert_eq!(CALIBRATION_SAMPLE_COMMAND, 0x0800_000d);
        assert_eq!(decode_calibration_accumulator(0), 0);
        assert_eq!(decode_calibration_accumulator(0x1000), 2);
        assert_eq!(decode_calibration_accumulator(0x3f_ffff), 2047);
        assert_eq!(decode_calibration_accumulator(0x40_0000), -2048);
        assert_eq!(
            pack_iq_signed8_pair(IqCalibrationCoefficient { i: 132, q: 4 }),
            0x047f
        );
        assert_eq!(
            pack_iq_signed8_pair(IqCalibrationCoefficient { i: -1, q: -128 }),
            0x80ff
        );
        assert_eq!(
            publish_iq_coefficient(IqCalibrationCoefficient { i: 132, q: 4 }),
            Some(PublishedIqCoefficient {
                shift: 11,
                normalized_i: 270_336,
                normalized_q: 8_192,
                packed: 0x10,
            })
        );
        assert_eq!(update_iq_shift_state(0x0025_4310, 11), 0x0025_4365);
        assert_eq!(
            iq_calibration_publication(3, IqCalibrationCoefficient { i: 132, q: 4 }, 0x0025_4310,),
            Some(IqCalibrationPublication {
                primary_address: 0x0abb_8124,
                normalized_first_address: 0x0abb_860c,
                normalized_second_address: 0x0abb_868c,
                primary_value: 0x047f,
                normalized_value: 0x10,
                next_shift_state: 0x0025_4365,
            })
        );
        let repeated_sample = IqCalibrationSample {
            baseline_i: 10,
            baseline_q: -10,
            target_i: 20,
            target_q: 0,
        };
        let series = build_iq_calibration_series([repeated_sample; 12], 0x0025_4310);
        assert_eq!(series.iterations[0].gain_index, 0x1a);
        assert_eq!(series.iterations[11].gain_index, 0);
        assert_eq!(series.iterations[0].scale_i, -2560);
        assert_eq!(series.iterations[0].scale_q, -2560);
        assert_eq!(
            series.iterations[0].coefficient,
            IqCalibrationCoefficient { i: 132, q: 4 }
        );
        assert_eq!(
            series.iterations[0].publication.map(|plan| (
                plan.primary_address,
                plan.normalized_first_address,
                plan.normalized_second_address,
            )),
            Some((0x0abb_8180, 0x0abb_8668, 0x0abb_86e8))
        );
        assert_eq!(
            series.iterations[11].publication.map(|plan| (
                plan.primary_address,
                plan.normalized_first_address,
                plan.normalized_second_address,
            )),
            Some((0x0abb_8118, 0x0abb_8600, 0x0abb_8680))
        );
        assert_ne!(series.final_shift_state, 0x0025_4310);
        assert_eq!(
            iq_calibration_controls(0, 0, 0),
            IqCalibrationControls {
                during_settle: 0x0035_7df1,
                active: 0x0034_7df1,
                abc0034: 0x1fff,
            }
        );
        assert_eq!(iq_calibration_controls(0, 1, 0).active, 0x00b4_7ff1);
        assert_eq!(iq_calibration_controls(0, 1, 1).active, 0x0074_7ff1);

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
