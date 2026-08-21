//! Vendor MAC/PHY initialization data consumed by `0x16ac6`.
//!
//! These constants come from the initialized-SRAM container segments rather
//! than guessed RF values. Static MAC initialization is active; per-channel
//! programming remains bounded at the translated request ABI before `0xf802`.

use core::cell::UnsafeCell;
use core::mem::MaybeUninit;

use zerocopy::byteorder::little_endian::U16;
use zerocopy::{Immutable, IntoBytes};

// Vendor references to 0x040099f8, 0x040099fc, and 0x04009a06 are confined
// to `phy_apply_cfg_if_channel_match`, whose translated owner is
// `program_channel_pll`. The adjacent force flag at 0x04009a08 has additional
// calibration and RF consumers and deliberately remains fixed.
#[repr(C)]
struct ChannelPllCache {
    integer: u32,
    fractional: u32,
    channel: u16,
}

struct SharedChannelPllCache(UnsafeCell<ChannelPllCache>);

unsafe impl Sync for SharedChannelPllCache {}

static CHANNEL_PLL_CACHE: SharedChannelPllCache = SharedChannelPllCache(UnsafeCell::new(
    ChannelPllCache {
        integer: 0,
        fractional: 0,
        channel: 0,
    },
));

unsafe fn cached_pll_channel() -> u16 {
    let cache = CHANNEL_PLL_CACHE.0.get();
    unsafe { (&raw const (*cache).channel).read_volatile() }
}

unsafe fn cached_pll_divider() -> (u32, u32) {
    let cache = CHANNEL_PLL_CACHE.0.get();
    unsafe {
        (
            (&raw const (*cache).integer).read_volatile(),
            (&raw const (*cache).fractional).read_volatile(),
        )
    }
}

unsafe fn set_cached_pll_divider(integer: u32, fractional: u32, channel: u16) {
    let cache = CHANNEL_PLL_CACHE.0.get();
    unsafe {
        (&raw mut (*cache).integer).write_volatile(integer);
        (&raw mut (*cache).fractional).write_volatile(fractional);
        (&raw mut (*cache).channel).write_volatile(channel);
    }
}

// Vendor references to 0x040099d4 and 0x040099d6 are confined to the
// translated channel-power publisher and TX-gain programmer. The adjacent
// threshold at 0x04009a04 has additional consumers and remains fixed.
#[repr(C)]
struct ChannelPowerLimits {
    low_rate: i16,
    high_rate: i16,
}

struct SharedChannelPowerLimits(UnsafeCell<ChannelPowerLimits>);

unsafe impl Sync for SharedChannelPowerLimits {}

static CHANNEL_POWER_LIMITS: SharedChannelPowerLimits =
    SharedChannelPowerLimits(UnsafeCell::new(ChannelPowerLimits {
        low_rate: 0,
        high_rate: 0,
    }));

unsafe fn set_channel_power_limits(low_rate: i16, high_rate: i16) {
    let limits = CHANNEL_POWER_LIMITS.0.get();
    unsafe {
        (&raw mut (*limits).low_rate).write_volatile(low_rate);
        (&raw mut (*limits).high_rate).write_volatile(high_rate);
    }
}

unsafe fn channel_power_limit(high_rate: bool) -> i16 {
    let limits = CHANNEL_POWER_LIMITS.0.get();
    unsafe {
        if high_rate {
            (&raw const (*limits).high_rate).read_volatile()
        } else {
            (&raw const (*limits).low_rate).read_volatile()
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IqHardwareDiagnostics {
    pub baseline_i: i32,
    pub target_i: i32,
    pub baseline_polls: u32,
    pub target_polls: u32,
}

struct SharedIqDiagnostics(UnsafeCell<IqHardwareDiagnostics>);
unsafe impl Sync for SharedIqDiagnostics {}

static IQ_DIAGNOSTICS: SharedIqDiagnostics =
    SharedIqDiagnostics(UnsafeCell::new(IqHardwareDiagnostics {
        baseline_i: 0,
        target_i: 0,
        baseline_polls: 0,
        target_polls: 0,
    }));
static LAST_SAMPLE_POLLS: SharedIqDiagnostics =
    SharedIqDiagnostics(UnsafeCell::new(IqHardwareDiagnostics {
        baseline_i: 0,
        target_i: 0,
        baseline_polls: 0,
        target_polls: 0,
    }));

pub fn iq_hardware_diagnostics() -> IqHardwareDiagnostics {
    unsafe { *IQ_DIAGNOSTICS.0.get() }
}

fn last_sample_polls() -> u32 {
    unsafe { (*LAST_SAMPLE_POLLS.0.get()).baseline_polls }
}

unsafe fn set_last_sample_polls(value: u32) {
    unsafe { (*LAST_SAMPLE_POLLS.0.get()).baseline_polls = value };
}

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
        let offset = (crate::dtcm::phy_offset_word().get() as *const u32).read_volatile() as usize;
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
    calibration_sample_settings_from(0, i, q)
}

fn calibration_sample_settings_from(current: u32, i: u8, q: u8) -> u32 {
    let with_i = (current & !0x0000_02ff) | u32::from(i);
    let with_q = (with_i & !0x02ff_0000) | (u32::from(q) << 16);
    with_q | 0x0100_0100
}

/// Fixed `0x17b70(0x0b, 1, ...)` trigger word before hardware sets bit `0x10`.
pub const CALIBRATION_SAMPLE_COMMAND: u32 = 0x0800_000d;

/// Detached bounded form of `0x17bf2 -> 0x17b70`.
pub unsafe fn run_calibration_sample_mode(
    i: u8,
    q: u8,
    measurement_mode: u8,
    max_polls: u32,
) -> Result<IqCalibrationCoefficient, CalibrationSampleError> {
    unsafe {
        // The optimized vendor call chain deliberately carries r3 from
        // `rf_select_band_regs()` through `rf_cal_path_setup()` and
        // `rf_set_iq_dac()`. At both `rf_set_iq_dac` (0x17c46) and
        // `rf_measure_iq` (0x17bc4), that fourth argument is the register-base
        // value below, not either destination register's previous contents.
        const VENDOR_CALL_CONTEXT: u32 = 0x0abb_80c0;
        write_u32(
            0x0abb_8114,
            calibration_sample_settings_from(VENDOR_CALL_CONTEXT, i, q),
        );
        let command = (((VENDOR_CALL_CONTEXT & 0xf000_ffff & !0xf0)
            | 0x0800_0008
            | (u32::from(measurement_mode & 1) << 2))
            & !3)
            .wrapping_add(1);
        write_u32(0x0abb_80f0, command);
        for polls in 0..max_polls {
            if (0x0abb_80f0 as *const u32).read_volatile() & 0x10 != 0 {
                set_last_sample_polls(polls + 1);
                let i = decode_calibration_accumulator((0x0abb_810c as *const u32).read_volatile());
                let q = decode_calibration_accumulator((0x0abb_8110 as *const u32).read_volatile());
                return Ok(IqCalibrationCoefficient { i, q });
            }
        }
        set_last_sample_polls(max_polls);
    }
    Err(CalibrationSampleError::Timeout)
}

pub unsafe fn run_calibration_sample(
    i: u8,
    q: u8,
    max_polls: u32,
) -> Result<IqCalibrationCoefficient, CalibrationSampleError> {
    unsafe { run_calibration_sample_mode(i, q, 1, max_polls) }
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

/// Live `phy_program_clock_divisor` publication.
///
/// # Safety
///
/// The vendor channel state at `0x0400994c` must be initialized.
pub unsafe fn program_channel_measurement_timing() -> Option<i32> {
    unsafe {
        let mode = (crate::dtcm::phy_profile().get() as *const u8).read_volatile();
        let frequency_khz = (crate::dtcm::phy_frequency_khz().get() as *const u32).read_volatile();
        let timing = channel_measurement_timing(mode, frequency_khz)?;
        write_u32(0x0ab8_8020, timing as u32);
        Some(timing)
    }
}

/// Live `phy_set_freq_offset` publication.
///
/// # Safety
///
/// The vendor channel and PHY software states must be initialized.
pub unsafe fn publish_channel_frequency_offset() -> i16 {
    unsafe {
        let mode = (crate::dtcm::phy_profile().get() as *const u8).read_volatile();
        let frequency_khz = (crate::dtcm::phy_frequency_khz().get() as *const u32).read_volatile();
        let offset = channel_frequency_offset_mhz(mode, frequency_khz);
        let current = (crate::dtcm::phy_state_scale().get() as *const i32).read_volatile();
        if current != i32::from(offset) {
            (crate::dtcm::phy_state_scale().get() as *mut i32).write_volatile(i32::from(offset));
        }
        offset
    }
}

/// Exact `phy_copy_cfg_slot` state copy.
///
/// # Safety
///
/// The vendor channel state must be initialized and `slot` must identify its
/// fixed two-entry cache.
pub unsafe fn copy_channel_configuration_slot(slot: u8) {
    if slot > 1 {
        return;
    }
    unsafe {
        let value = (crate::dtcm::phy_measured_b().get() as *const u32).read_volatile();
        write_u32(crate::dtcm::phy_channel_cache_unchecked(usize::from(slot)).get(), value);
    }
}

/// Exact enable/disable writes from annotated `phy_agc_enable`.
///
/// # Safety
///
/// The AGC and MAC/PHY register banks must be enabled.
pub unsafe fn set_phy_agc_enabled(enabled: bool) {
    unsafe {
        let current = (0x0ab8_0c38 as *const u32).read_volatile();
        let updated = if enabled {
            write_u32(0x0abb_81a0, 0);
            let profile = (crate::dtcm::phy_profile().get() as *const u8).read_volatile();
            write_u32(0x0ab8_006c, if profile == 1 { 0 } else { 0x0ebf });
            write_u32(0x0ab8_0068, 0);
            current | 0x1800
        } else {
            current & !0x1800
        };
        write_u32(0x0ab8_0c38, updated);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChannelCalibrationRequirement {
    AlreadyValid,
    RunCalibration,
    UnsupportedProfile,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChannelPllError {
    UnsupportedProfile,
    InvalidReference,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TemperatureMeasurement {
    pub value: i32,
    pub accepted: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TemperatureMeasurementError {
    HardwareFaultNoRestore,
    InvalidCalibrationNoRestore,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChannelPowerError {
    InvalidThresholdTable,
    InvalidRateTable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GainProgrammingError {
    InvalidProfile,
    InvalidDivisor,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct GainComputationInput {
    rate: u8,
    first_limit: i32,
    second_limit: i32,
    rate_base: i32,
    gain_coefficient_a: i32,
    gain_coefficient_b: i32,
    rssi_rate_scale: i32,
    rssi_temperature_coefficient: i32,
    rssi_divisor_coefficient: i32,
    rssi_multiplier_coefficient: i32,
    rssi_denominator: i32,
    rssi_offset: i32,
    measured_a: i32,
    measured_b: i32,
    analog_enabled: i32,
    analog_word_2c: u32,
    analog_word_30: u32,
    state_scale: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct GainComputationResult {
    selected_power: i32,
    gain_code: u16,
    rssi_value: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChannelTransitionError {
    Pll(ChannelPllError),
    InvalidTiming,
    Temperature(TemperatureMeasurementError),
    Calibration(IqCalibrationHardwareError),
    Power(ChannelPowerError),
    Gain(GainProgrammingError),
    MacWake(crate::mac::MacWakeError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChannelTransitionResult {
    pub divider: PllDivider,
    pub timing: i32,
    pub temperature: TemperatureMeasurement,
    pub calibration_ran: bool,
    pub threshold: i16,
    pub first_tx_power: i16,
    pub second_tx_power: i16,
    pub frequency_offset: i16,
}

/// Channel form of `phy_apply_cfg_pair`, including its cached
/// integer/fractional pair and vendor PLL restart. Profile zero uses the
/// 2407/2484 MHz mapping and multiplier 1250; profile one uses dispatcher case
/// zero's 5000 MHz base and multiplier 1000.
///
/// # Safety
///
/// The vendor channel state and PLL register bank must be initialized.
pub unsafe fn program_channel_pll(channel: u16) -> Result<PllDivider, ChannelPllError> {
    unsafe {
        let profile = (crate::dtcm::phy_profile().get() as *const u8).read_volatile();
        let (frequency_khz, multiplier) = match profile {
            0 => (channel_frequency_khz_2ghz(channel), 1250),
            1 => ((5000 + u32::from(channel & 0xff) * 5) * 1000, 1000),
            _ => return Err(ChannelPllError::UnsupportedProfile),
        };
        write_u32(crate::dtcm::phy_frequency_khz().get(), frequency_khz);
        let reference = (crate::dtcm::phy_reference_word().get() as *const u32).read_volatile();
        let correction = i32::from((crate::dtcm::phy_correction().get() as *const i16).read_volatile());
        let correction = i64::from(reference)
            .wrapping_mul(i64::from(correction))
            .wrapping_div(1000);
        let divisor = i64::from(reference)
            .wrapping_mul(1000)
            .wrapping_add(correction);
        if divisor <= 0 || divisor > i64::from(u32::MAX) {
            return Err(ChannelPllError::InvalidReference);
        }
        let cached_channel = cached_pll_channel();
        let cache_forced = (crate::dtcm::phy_extended_settle().get() as *const u8).read_volatile() != 0;
        let divider = if cached_channel == channel && !cache_forced {
            let (integer, fractional) = cached_pll_divider();
            PllDivider {
                integer,
                fractional,
                register: integer.wrapping_shl(21) | fractional,
            }
        } else {
            let divider = pll_divider(frequency_khz, multiplier, divisor as u32)
                .ok_or(ChannelPllError::InvalidReference)?;
            set_cached_pll_divider(divider.integer, divider.fractional, channel);
            divider
        };
        commit_channel_pll(divider.register, profile, cache_forced);
        Ok(divider)
    }
}

/// Parameter-zero path of annotated `rf_measure_temp_and_vbat`. The two fatal
/// exits intentionally skip restoration, matching the vendor routine.
///
/// # Safety
///
/// The vendor DTCM state and RF/PHY register banks must be initialized.
pub unsafe fn measure_temperature_primary()
-> Result<TemperatureMeasurement, TemperatureMeasurementError> {
    unsafe {
        write_u32(
            0x0abb_8004,
            (0x0abb_8004 as *const u32).read_volatile() | 0x800,
        );
        let saved_abc0084 = (0x0abc_0084 as *const u32).read_volatile();
        write_u32(
            0x0abc_0084,
            (0x0abc_0098 as *const u32).read_volatile() | 0x0700_0200,
        );
        let saved_abc006c = (0x0abc_006c as *const u32).read_volatile();
        write_u32(0x0abc_006c, (0x0abc_0080 as *const u32).read_volatile());
        let saved_abc0050 = (0x0abc_0050 as *const u32).read_volatile();
        write_u32(0x0abc_0050, (0x0abc_0064 as *const u32).read_volatile());
        let saved_abc0034 = (0x0abc_0034 as *const u32).read_volatile();
        write_u32(
            0x0abc_0034,
            (0x0abc_0048 as *const u32).read_volatile() | 0x101,
        );
        let saved_abc0004 = (0x0abc_0004 as *const u32).read_volatile();
        write_u32(0x0abc_0004, saved_abc0004 | 2);
        let saved_abb800c = (0x0abb_800c as *const u32).read_volatile();
        write_u32(0x0abb_800c, 0x0000_e080);
        delay_units(10);
        let status = (0x0abb_800c as *const u32).read_volatile();
        if status & 0x20 != 0 {
            return Err(TemperatureMeasurementError::HardwareFaultNoRestore);
        }
        delay_units(10);
        let denominator = i32::from((crate::dtcm::phy_denominator().get() as *const i16).read_volatile());
        if denominator == 0 {
            return Err(TemperatureMeasurementError::InvalidCalibrationNoRestore);
        }
        let raw = (0x0abb_82d8 as *const i32)
            .read_volatile()
            .wrapping_mul(0x47);
        let offset = i32::from((crate::dtcm::phy_correction_offset().get() as *const i16).read_volatile());
        let converted = raw
            .wrapping_sub(offset)
            .wrapping_mul(1000)
            .wrapping_div(denominator);
        let accepted = (converted.wrapping_add(-41_000) as u32) <= 17_900;
        let value = if accepted { converted } else { 38_000 };
        if accepted {
            write_u32(crate::dtcm::phy_measured_b().get(), converted as u32);
        }
        delay_units(1);

        write_u32(0x0abc_0004, saved_abc0004);
        write_u32(0x0abc_0084, saved_abc0084);
        write_u32(0x0abc_006c, saved_abc006c);
        write_u32(0x0abc_0050, saved_abc0050);
        write_u32(0x0abc_0034, saved_abc0034);
        write_u32(0x0abb_800c, saved_abb800c);
        Ok(TemperatureMeasurement { value, accepted })
    }
}

/// Live threshold lookup from the profile descriptor table at `0x0400145c`.
///
/// # Safety
///
/// The vendor SDD-derived descriptor and record pointers must remain valid.
pub unsafe fn lookup_channel_threshold(channel: u16) -> Result<i16, ChannelPowerError> {
    unsafe {
        let profile = usize::from((crate::dtcm::phy_profile().get() as *const u8).read_volatile());
        if profile > 1 {
            return Err(ChannelPowerError::InvalidThresholdTable);
        }
        let descriptor = 0x0400_145c + profile * 8;
        let count = usize::from(((descriptor + 1) as *const u8).read_volatile());
        if count > 64 {
            return Err(ChannelPowerError::InvalidThresholdTable);
        }
        let default = ((descriptor + 2) as *const i16).read_volatile();
        let records = ((descriptor + 4) as *const u32).read_volatile() as usize;
        let mut selected = default;
        for index in 0..count {
            let record = records + index * 4;
            let threshold = (record as *const u16).read_volatile();
            if threshold > channel {
                break;
            }
            selected = ((record + 2) as *const i16).read_volatile();
        }
        Ok(selected)
    }
}

/// Live translation of `phy_txpower_from_rate_table`.
///
/// # Safety
///
/// The SDD-derived rate-table pointer in PHY software state must be valid.
pub unsafe fn channel_tx_power_from_rate_table(
    channel: u8,
    second_column: bool,
) -> Result<i16, ChannelPowerError> {
    unsafe {
        let table = (crate::dtcm::phy_table_pointer().get() as *const u32).read_volatile() as usize;
        if table == 0 {
            return Err(ChannelPowerError::InvalidRateTable);
        }
        let count = usize::from(((table + 0x46) as *const u8).read_volatile());
        if count > 64 {
            return Err(ChannelPowerError::InvalidRateTable);
        }
        let records = table + 0x16;
        let mut selected = 0_usize;
        for index in 0..count {
            let threshold = ((records + index * 3) as *const u8).read_volatile();
            if channel <= threshold {
                selected = if index != 0 && channel < threshold {
                    index - 1
                } else {
                    index
                };
                break;
            }
            selected = index;
        }
        let column = if second_column { 1 } else { 2 };
        let encoded = i32::from(((records + selected * 3 + column) as *const u8).read_volatile());
        let base = i32::from((crate::dtcm::phy_threshold().get() as *const i16).read_volatile());
        Ok(base.wrapping_add(encoded.wrapping_mul(4)) as i16)
    }
}

/// Publish threshold and both channel TX-power values to the PHY software
/// state used by the gain-table path.
///
/// # Safety
///
/// Both vendor SDD-derived tables and PHY software state must be initialized.
pub unsafe fn publish_channel_power(channel: u8) -> Result<(i16, i16, i16), ChannelPowerError> {
    unsafe {
        let profile = (crate::dtcm::phy_profile().get() as *const u8).read_volatile();
        let threshold_id = match profile {
            0 => 0x30,
            1 => 0x31,
            _ => return Err(ChannelPowerError::InvalidThresholdTable),
        };
        let threshold =
            crate::configuration::channel_threshold_correction(threshold_id, u16::from(channel))
                .ok_or(ChannelPowerError::InvalidThresholdTable)?;
        write_u16(crate::dtcm::phy_threshold().get(), threshold as u16);
        let first = channel_tx_power_from_rate_table(channel, false)?;
        let second = channel_tx_power_from_rate_table(channel, true)?;
        set_channel_power_limits(first, second);
        Ok((threshold, first, second))
    }
}

fn gain_div(numerator: i32, denominator: i32) -> Result<i32, GainProgrammingError> {
    numerator
        .checked_div(denominator)
        .ok_or(GainProgrammingError::InvalidDivisor)
}

/// Vendor logarithmic approximation at 0x1961c. Arithmetic deliberately
/// wraps at 32 bits, matching the Thumb `adds`/`muls` implementation.
fn gain_index_from_value(mut value: i32) -> i32 {
    let mut result = 0x1_0000i32;
    for (threshold, action) in [
        (-363_408, 0u8),
        (-181_704, 1),
        (-90_852, 2),
        (-45_426, 3),
        (-26_573, 4),
        (-14_623, 5),
        (-7_719, 6),
        (-3_973, 7),
        (-2_017, 8),
    ] {
        let reduced = value.wrapping_add(threshold);
        if reduced >= 0 {
            result = match action {
                0 => 0x100_0000,
                1 => result.wrapping_shl(4),
                2 => result.wrapping_shl(2),
                3 => result.wrapping_shl(1),
                4 => result.wrapping_add(result >> 1),
                5 => result.wrapping_add(result >> 2),
                6 => result.wrapping_add(result >> 3),
                7 => result.wrapping_add(result >> 4),
                _ => result.wrapping_add(result >> 5),
            };
            value = reduced;
        }
    }
    for (threshold, shift) in [(0x3f8, 6), (0x1fe, 7)] {
        let reduced = value.wrapping_sub(threshold);
        if reduced >= 0 {
            result = result.wrapping_add(result >> shift);
            value = reduced;
        }
    }
    for (bit, shift) in [
        (8, 8),
        (7, 9),
        (6, 10),
        (5, 11),
        (4, 12),
        (3, 13),
        (2, 14),
        (1, 15),
        (0, 16),
    ] {
        if value & (1 << bit) != 0 {
            result = result.wrapping_add(result >> shift);
        }
    }
    result
}

fn gain_code(index: i32) -> u16 {
    let table_index = (index.clamp(4, 32) as u32) >> 2;
    (1u16 << (table_index + 2)).wrapping_sub(4)
}

fn compute_gain_entry(
    input: GainComputationInput,
) -> Result<GainComputationResult, GainProgrammingError> {
    let measured_term = input.measured_a.wrapping_mul(0x580) >> 16;
    let first = gain_div(
        input
            .gain_coefficient_a
            .wrapping_mul(measured_term.wrapping_sub(845))
            >> 4,
        10_000,
    )?;
    let analog_value = if input.analog_enabled == 0 {
        -80
    } else {
        let magnitude = (input.analog_word_2c & 0x7ff) as i32;
        if input.analog_word_2c & 0x800 != 0 {
            -magnitude
        } else {
            magnitude
        }
    };
    let second_input = 480i32.wrapping_sub(
        (9005i32.wrapping_mul(56_445i32.wrapping_sub(input.measured_b)) >> 16)
            .wrapping_sub(analog_value),
    );
    let second = gain_div(input.gain_coefficient_b.wrapping_mul(second_input), 10_000)?;
    let lookup = first
        .wrapping_add(second)
        .wrapping_add(input.rate_base)
        .wrapping_sub(48);
    let selected_power = input.first_limit.min(input.second_limit.min(lookup));

    let raw_gain_input = ((input.analog_word_2c & 0x0fff_ffff) >> 12) as i32;
    let combined = ((input.analog_word_30 & 0xfff) << 4) | (input.analog_word_2c >> 28);
    let signed_magnitude = ((input.analog_word_2c & 0x7ff) << 12) as i32;
    let mut analog_offset = if input.analog_word_2c & 0x800 != 0 {
        -signed_magnitude
    } else {
        signed_magnitude
    };
    let mut analog_normalization = combined as i32;
    if input.analog_enabled == 0 || combined.wrapping_sub(0xf00) >= 0x2101 {
        // The second branch intentionally retains `raw_gain_input`.
        analog_offset = -327_680;
        analog_normalization = 0x2000;
    }

    let base_gain = gain_index_from_value(7545i32.wrapping_mul(raw_gain_input) >> 8);
    let temperature_input = 9005i32
        .wrapping_mul(56_445i32.wrapping_sub(input.measured_b))
        .wrapping_sub(analog_offset.wrapping_mul(16))
        .wrapping_sub(analog_normalization.wrapping_mul(4096));
    let temperature = gain_div(
        input
            .rssi_temperature_coefficient
            .wrapping_mul(temperature_input)
            >> 4,
        1000,
    )?;
    let combined_gain = base_gain.wrapping_add(temperature);
    let scaled_a = gain_div(
        combined_gain.wrapping_mul(input.rssi_multiplier_coefficient),
        100,
    )?;
    let scaled_b = gain_div(
        combined_gain
            .wrapping_mul(input.rssi_divisor_coefficient)
            .wrapping_mul(input.state_scale),
        10_000,
    )?;
    let target = selected_power.wrapping_add(if input.rate < 2 { 64 } else { 48 });
    let target_gain = gain_index_from_value(7545i32.wrapping_mul(target) >> 4);
    let ratio = gain_div(target_gain.wrapping_shl(8), scaled_a.wrapping_add(scaled_b))?;
    let ratio = gain_div(
        ratio
            .wrapping_mul(1000)
            .wrapping_add(input.rssi_offset.wrapping_shl(8)),
        input.rssi_denominator,
    )?;
    let scaled = gain_div(ratio.wrapping_mul(100), input.rssi_rate_scale)?.max(0);
    let mut index = (scaled >> 8) & 0xfffc;
    if scaled & 0x3ff != 0 {
        index = index.wrapping_add(4);
    }
    index = index.clamp(4, 32);
    let rssi_value = ((((gain_div(scaled.wrapping_mul(input.rssi_rate_scale), index)? as u32)
        & 0x00ff_ffff)
        >> 8)
        .max(100)) as u16;

    Ok(GainComputationResult {
        selected_power,
        gain_code: gain_code(index),
        rssi_value,
    })
}

unsafe fn gain_computation_input(
    profile: u8,
    rate: u8,
    first_limit: i32,
    second_limit: i32,
) -> Result<GainComputationInput, GainProgrammingError> {
    unsafe {
        if profile > 1 {
            return Err(GainProgrammingError::InvalidProfile);
        }
        let bank = crate::dtcm::sdd_profile_unchecked(usize::from(profile)).get();
        let rate = rate.min(10);
        let coefficient_base = crate::dtcm::sdd_gain_coefficient_unchecked(usize::from(profile) * 2).get();
        Ok(GainComputationInput {
            rate,
            first_limit,
            second_limit,
            rate_base: i32::from(((bank + usize::from(rate) * 2) as *const i16).read_volatile()),
            gain_coefficient_a: i32::from((coefficient_base as *const u16).read_volatile()),
            gain_coefficient_b: i32::from(((coefficient_base + 2) as *const u16).read_volatile()),
            rssi_rate_scale: i32::from(
                (crate::dtcm::sdd_rssi_rate_scale_unchecked(usize::from(profile), usize::from(rate)).get() as *const i16).read_volatile(),
            ),
            rssi_temperature_coefficient: i32::from((crate::dtcm::sdd_calibration_coefficient_unchecked(usize::from(profile)).get() as *const i16).read_volatile()),
            rssi_divisor_coefficient: i32::from((crate::dtcm::sdd_conversion_value_unchecked(usize::from(profile), 0).get() as *const i16).read_volatile()),
            rssi_multiplier_coefficient: i32::from((crate::dtcm::sdd_conversion_value_unchecked(usize::from(profile), 1).get() as *const i16).read_volatile()),
            rssi_denominator: i32::from((crate::dtcm::sdd_rssi_coefficient_unchecked(usize::from(profile), 0).get() as *const i16).read_volatile()),
            rssi_offset: i32::from((crate::dtcm::sdd_rssi_coefficient_unchecked(usize::from(profile), 1).get() as *const i16).read_volatile()),
            measured_a: (crate::dtcm::phy_measured_a().get() as *const i32).read_volatile(),
            measured_b: (crate::dtcm::phy_measured_b().get() as *const i32).read_volatile(),
            analog_enabled: i32::from((crate::dtcm::scheduler_analog_enabled().get() as *const i16).read_volatile()),
            analog_word_2c: (crate::dtcm::scheduler_analog_word(0).unwrap().get() as *const u32).read_volatile(),
            analog_word_30: (crate::dtcm::scheduler_analog_word(1).unwrap().get() as *const u32).read_volatile(),
            state_scale: (crate::dtcm::phy_state_scale().get() as *const i32).read_volatile(),
        })
    }
}

fn encoded_gain_words(gain_code: u16, rssi_value: u16) -> (u32, u32) {
    let rssi = u32::from(rssi_value) & 0x3ff;
    let gain = u32::from(gain_code) & 0x3ff;
    let quotient = if rssi == 0 { 0x800 } else { 0x0003_2f53 / rssi };
    let companion = if quotient < 0x801 {
        0x1000 - quotient
    } else {
        0x800
    };
    ((gain << 10) | rssi, companion)
}

/// Vendor `phy_temp_compensate_all_slots`/`phy_program_gain_for_channel`
/// sequence. One entry is produced for every hardware rate slot before PHY
/// operation 1 and RX re-enable.
///
/// # Safety
///
/// SDD state, analog state, and the gain MMIO banks must be initialized.
pub unsafe fn program_all_tx_gain_slots(power_tenths_dbm: i32) -> Result<(), GainProgrammingError> {
    unsafe {
        if (crate::dtcm::phy_calibration_stage().get() as *const u8).read_volatile() < 2 {
            return Ok(());
        }
        let profile = (crate::dtcm::phy_profile().get() as *const u8).read_volatile();
        if profile > 1 {
            return Err(GainProgrammingError::InvalidProfile);
        }
        let requested_offset = gain_div(power_tenths_dbm.wrapping_shl(4), 10)?;
        let first_table = crate::dtcm::sdd_profile_unchecked(usize::from(profile)).get();
        for slot in 0..16usize {
            let rate = (slot as u8).min(10);
            let rate_limit =
                i32::from(((first_table + usize::from(rate) * 2) as *const i16).read_volatile())
                    .wrapping_add(requested_offset);
            let second_limit = rate_limit.min(i32::from(channel_power_limit(rate > 1)));
            let result = compute_gain_entry(gain_computation_input(
                profile,
                rate,
                requested_offset,
                second_limit,
            )?)?;

            let record = 0x0400_146c + slot * 0x10;
            write_u8(record, rate);
            write_u16(record + 2, requested_offset as u16);
            write_u16(record + 4, result.selected_power as u16);
            write_u16(record + 6, 0);
            write_u32(record + 8, 0);
            write_u16(record + 0x0c, result.gain_code);
            write_u16(record + 0x0e, result.rssi_value);

            let (entry, companion) = encoded_gain_words(result.gain_code, result.rssi_value);
            write_u32(0x0abb_801c + slot * 4, entry);
            write_u32(0x0abb_8400 + slot * 4, companion);
        }
        Ok(())
    }
}

/// Prepare the profile-specific correction cache used by
/// `phy_set_channel_full` before its mode-2 calibration call.
///
/// # Safety
///
/// The vendor channel state and correction bank must be initialized.
pub unsafe fn prepare_channel_calibration_cache() -> ChannelCalibrationRequirement {
    unsafe {
        let profile = (crate::dtcm::phy_profile().get() as *const u8).read_volatile();
        let validity_address = match profile {
            0 => crate::dtcm::phy_profile0_ready().get(),
            1 => crate::dtcm::phy_profile1_ready().get(),
            _ => return ChannelCalibrationRequirement::UnsupportedProfile,
        };
        if (validity_address as *const u8).read_volatile() != 0 {
            return ChannelCalibrationRequirement::AlreadyValid;
        }
        copy_channel_configuration_slot(1);
        let default_correction = if profile == 0 { 0x0ece_0000 } else { 0 };
        for index in 0..16_usize {
            write_u32(0x0abb_8068 + index * 4, default_correction);
        }
        ChannelCalibrationRequirement::RunCalibration
    }
}

/// Record the channel associated with a newly valid profile cache.
///
/// # Safety
///
/// The vendor channel state must be initialized.
pub unsafe fn record_calibrated_channel(channel: u16) {
    unsafe {
        match (crate::dtcm::phy_profile().get() as *const u8).read_volatile() {
            0 if (crate::dtcm::phy_profile0_ready().get() as *const u8).read_volatile() == 1 => write_u16(crate::dtcm::phy_profile1_channel().get(), channel),
            1 if (crate::dtcm::phy_profile1_ready().get() as *const u8).read_volatile() == 1 => write_u16(crate::dtcm::phy_retained_channel().get(), channel),
            _ => {}
        }
    }
}

unsafe fn rf_init_stage_d_mode0() {
    const BASE: usize = 0x0abc_0040;
    unsafe {
        let state = crate::dtcm::PHY_PROFILE_STATE.get();
        let alternate = ((state + 0x40) as *const u8).read_volatile() == 1;
        write_u32(BASE - 0x3c, 0x304);
        write_u32(BASE - 0x38, if alternate { 0x9200 } else { 0x9000 });
        let remap = (crate::dtcm::scheduler_remap_primary().get() as *const u32).read_volatile();
        write_u32(
            BASE - 0x20,
            ((remap & 0x07ff_ffff) >> 26)
                .wrapping_mul(0x2000_0000)
                .wrapping_add(0x0f4a_c008),
        );
        write_u32(
            BASE - 0x1c,
            if alternate { 0x29ff_1800 } else { 0x2987_1800 },
        );
        write_u32(BASE - 0x28, 0);
        write_u32(BASE - 0x34, 0x0050_0100);
        write_u32(BASE + 0x10, 0x0095_0000);
        write_u32(BASE + 0x14, 0x00b5_3830);
        write_u32(BASE + 0x18, 0x00b5_7ff1);
        for offset in [0x1c, 0x20, 0x24, 0x28] {
            write_u32(BASE + offset, 0x00b4_7ff1);
        }
        write_u32(BASE - 0x0c, 0x0000_1c04);
        write_u32(BASE - 8, 0x0000_1e55);
        write_u32(BASE - 4, 0x0000_1e55);
        for offset in [0, 4, 8, 0x0c] {
            write_u32(BASE + offset, 0x0000_1fff);
        }
    }
}

unsafe fn rf_init_stage_a_mode0() {
    const BASE: usize = 0x0abc_00c0;
    unsafe {
        write_u32(BASE + 0x30, 0xa0);
        write_u32(BASE - 0x98, 0x0800_1f01);
        write_u32(BASE - 0x90, 0x001c_0000);
        write_u32(
            BASE - 0xa4,
            u32::from((crate::dtcm::phy_zero_select().get() as *const u8).read_volatile() == 0) * 9,
        );
        write_u32(BASE + 0x14, 0x0703_0100);
        write_u32(BASE + 0x18, 0x7f3f_1f0f);
        write_u32(BASE + 0x24, 0x0000_ffff);
        write_u32(BASE + 0x1c, 0x0703_0100);
        write_u32(BASE + 0x20, 0x7f3f_1f0f);
        write_u32(BASE - 0x94, crate::dtcm::RF_INITIALIZATION_ROOT.get() as u32);
        if (crate::dtcm::scheduler_analog_enabled().get() as *const u16).read_volatile() == 0 {
            write_u32(0x0ac8_005c, 0x6a25_5800);
            write_u32(0x0ac8_00e8, 0x10c);
        }
        let override_value = (crate::dtcm::phy_override_value().get() as *const u8).read_volatile();
        if override_value != 0 {
            write_u32(
                0x0ac8_005c,
                (0x0ac8_005c as *const u32).read_volatile() & !0x7f
                    | u32::from(override_value & 0x7f),
            );
        }
        write_u32(BASE - 0xac, 0x0027_0047);
        write_u32(BASE - 0x54, 0);
        write_u32(BASE - 0x40, 0x0002_0006);
        write_u32(BASE - 0x50, 0x0002_0516);
        write_u32(BASE - 0x4c, 0x0002_0516);
        write_u32(BASE - 0x48, 0x0002_4f36);
        write_u32(BASE - 0x44, 0x0002_4f36);
        write_u32(BASE + 0x38, 0x0002_4f36);
        write_u32(BASE - 0x3c, 0x1000);
        write_u32(BASE - 0x28, 0x0400_1000);
        write_u32(BASE - 0x38, 0x0e2c_aa32);
        write_u32(BASE - 0x34, 0x0e7c_aef2);
        write_u32(BASE - 0x30, 0x0f7c_aef2);
        write_u32(BASE - 0x2c, 0x0ffc_aef2);
        write_u32(BASE + 0x34, 0x0f6c_a8f2);
    }
}

unsafe fn rf_init_stage_b_mode0() {
    const BASE: usize = 0x0abc_00c0;
    unsafe {
        write_u32(BASE - 0x18, 0);
        write_u32(BASE - 0x1c, 0);
        write_u32(BASE - 0x20, 0x0000_140a);
        write_u32(BASE - 0x24, 0x0030_0000);
        write_u32(BASE, 0x1350_381e);
        let alternate = (crate::dtcm::phy_silicon_variant().get() as *const u8).read_volatile() == 1;
        write_u32(BASE - 0xb8, if alternate { 0x8202 } else { 0x8002 });
        write_u32(BASE - 0xbc, 0x0008_0206);
        write_u32(BASE - 0x8c, 0x0000_0201);
        delay_timer_ticks(10);
        write_u32(BASE - 0x24, 0x0030_0001);
        delay_timer_ticks(0x87);
        write_u32(BASE - 0x24, 0x0030_0030);
        write_u32(BASE - 0x24, 0x0030_0000);
        write_u32(BASE - 0xb8, if alternate { 0x9200 } else { 0x9000 });
        write_u32(BASE - 0x8c, 0x0000_1e05);
        write_u32(BASE - 0xbc, 0x304);
        write_u32(BASE - 0xbc, 0x0008_0306);
        write_u32(BASE - 0x70, 0x0823_5801);
        write_u32(BASE - 0x8c, 0x4c0);
        delay_timer_ticks(10);
        write_u32(BASE - 0x70, 0x0822_5801);
        write_u32(BASE, 0x1350_381c);
        write_u32(0x0abb_8004, 0x0008_8200);
        write_u32(BASE, 0x1350_381d);
        delay_timer_ticks(0x32);
        write_u32(BASE - 0xbc, 0x304);
        write_u32(BASE, 0x1350_381c);
        write_u32(BASE - 0x70, 0x0095_0000);
        write_u32(BASE - 0x8c, 0x0000_1c04);
        write_u32(0x0abb_8004, 0);
        delay_timer_ticks(10);
    }
}

unsafe fn rf_init_stage_c_mode0() {
    const BASE: usize = 0x0abc_00c0;
    unsafe {
        let reference = (crate::dtcm::phy_reference_word().get() as *const u32).read_volatile();
        let range = u32::from(reference >= 0x5dc0) + u32::from(reference >= 0xbb80);
        let gain = if reference >= 0xbb80 {
            2
        } else if reference >= 0x8340 {
            3
        } else if reference >= 0x4e20 {
            4
        } else {
            5
        } * 0x4000;
        write_u32(BASE - 0x14, 0x001a_3080 | gain);
        write_u32(BASE - 0x10, range + 0x200);
        write_u32(BASE - 4, 0);
        write_u32(BASE - 8, 0);
        write_u32(BASE - 0x14, 0x001a_30b8 | gain);
        write_u32(BASE - 0x10, range + 0x200);
        write_u32(BASE - 4, 0);
        write_u32(BASE - 8, 0x0020_0000);
        delay_timer_ticks(10);
        write_u32(BASE - 0x14, 0x001a_30fb | gain);
        write_u32(BASE - 0x10, range + 0x7200);
        write_u32(BASE - 4, 0);
        write_u32(BASE - 8, 0x0020_0412);
        delay_timer_ticks(5);
        write_u32(BASE - 0x14, (gain + 0x0100_0000) | 0x03fa_30fb);
        write_u32(BASE - 0x10, 0x0000_723c | range);
        write_u32(BASE - 4, 0);
        write_u32(BASE - 8, 0x0020_0412);
        delay_timer_ticks(1);
        write_u32(BASE - 0x10, 0x0035_723c | range);
        delay_timer_ticks(1);
        write_u32(BASE - 4, 0x0004_0000);
        write_u32(BASE - 8, 0x0020_2412);
        delay_timer_ticks(0x78);
    }
}

unsafe fn apply_first_channel_detector_state() {
    const SUBSTATE: &[(u32, u32)] = &[
        (0x0abb_8004, 0x0000_0034),
        (0x0abb_8008, 0x001f_0077),
        (0x0abb_80e8, 0x000f_0190),
        (0x0abb_80ec, 0x0000_001f),
        (0x0abb_80f4, 0x0020_0190),
        (0x0abb_80f8, 0x012c_00c8),
        (0x0abb_80fc, 0x0036_8ccc),
        (0x0abb_81a0, 0),
    ];
    const EXPANDED: [u8; 32] = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 1, 0, 0, 0, 4, 8, 1, 2, 0, 0, 0, 0,
        2, 8,
    ];
    const TAIL: &[(u32, u32)] = &[
        (0x0abb_82c4, 0x0005_0082),
        (0x0abb_82c8, 0x0005_0070),
        (0x0abb_82cc, 0x000d_0010),
        (0x0abb_82d0, 0),
        (0x0abb_82d4, 0),
        (0x0abb_8388, 1),
    ];
    unsafe {
        prepare_rf_mode0_stage();
        if (crate::dtcm::PHY_PROFILE_STATE.get() as *const u8).read_volatile() == 2 {
            apply_register_list(SUBSTATE);
        }
        for (index, value) in EXPANDED.into_iter().enumerate() {
            write_u32(0x0abb_8300 + index * 4, u32::from(value));
        }
        apply_register_list(TAIL);
    }
}

unsafe fn prepare_rf_mode0_stage() {
    unsafe {
        let control = (0x0ac8_0064 as *const u32).read_volatile();
        if control & 1 == 0 {
            write_u32(0x0ac8_0064, 0x11);
            delay_timer_ticks(0x28);
            write_u32(0x0ac8_0064, 1);
        }
        rf_init_stage_d_mode0();
        rf_init_stage_a_mode0();
        rf_init_stage_b_mode0();
        rf_init_stage_c_mode0();
        if (crate::dtcm::phy_profile0_state().get() as *const u8).read_volatile() == 0 {
            write_u8(crate::dtcm::phy_profile0_state().get(), 2);
            initialize_mac_core_mode0();
        }
    }
}

unsafe fn program_mode2_band_hardware() {
    unsafe {
        let bandwidth = (0x0aba_8040 as *const u32).read_volatile();
        write_u32(0x0aba_8040, bandwidth | 0x0002_0000);
        let timing = (0x0ab8_0c00 as *const u32).read_volatile();
        write_u32(
            0x0ab8_0c00,
            (timing & 0xfc00_ffff).wrapping_add(0x00b4_0000),
        );
        write_u32(0x0abd_0004, 4);

        let mut control = (0x0abb_8004 as *const u32).read_volatile();
        control = ((control & !1) | 0x2000_0000) & 0xe000_7fff | 0x800;
        // Dispatcher case 2 at 0x1745c clears r0 (bit 1), sets r2 (bit 2),
        // and selects bandwidth mode 2. The shared tail clears bit 4 for PHY
        // profile 2.
        control = (control & !2) | 4;
        if (crate::dtcm::PHY_PROFILE_STATE.get() as *const u8).read_volatile() == 2 {
            control &= !0x10;
        }
        write_u32(0x0abb_8004, control);
    }
}

unsafe fn program_scan_receive_band() {
    unsafe {
        // `phy_do_channel_switch` derives dispatcher mode 2 from scan rate
        // configuration 0x0117: bits 0 and 4 are both set and bit 5 is clear.
        // Profile byte +2 remains zero for the 2.4 GHz synth/calibration path.
        write_u8(crate::dtcm::phy_profile().get(), 0);
        write_u8(crate::dtcm::phy_phase().get(), 2);
        write_u8(crate::dtcm::phy_calibration_stage().get(), 3);
        write_u8(crate::dtcm::phy_calibration_state().get(), 1);
        program_mode2_band_hardware();
    }
}

unsafe fn publish_completed_receive_state() {
    unsafe {
        // The active channel path leaves the PHY controller running at
        // 0x0ac80064 == 1. Writing 0x10 here is vendor radio-stop behavior.
        write_u8(crate::dtcm::phy_calibration_stage().get(), 3);
        write_u8(crate::dtcm::MAC_WAKE_PHY_STATE.get(), 2);

        let mut state = (crate::dtcm::phy_retained_state().get() as *const u8).read_volatile();
        if state != 5 {
            state = 3;
            write_u8(crate::dtcm::phy_retained_state().get(), state);
        }
        write_u8(crate::dtcm::MAC_PHY_OPERATION_COMMAND.get(), 1);
        write_u32(crate::dtcm::MAC_PHY_OPERATION_STATE.get(), u32::from(state));
        write_u8(crate::dtcm::MAC_PHY_OPERATION_OUTPUT.get(), state);
        write_u32(crate::dtcm::MAC_PHY_OPERATION_TIMEOUT.get(), 0x0098_9680);
        write_u8(crate::dtcm::MAC_PHY_DISPATCH_COMMAND.get() + 1, 0);

        let runtime_flags = crate::dtcm::scheduler_runtime_flags().get();
        let keep_awake = (runtime_flags as *const u32).read_volatile();
        write_u32(runtime_flags, keep_awake | 0x0004_0000);
    }
}

unsafe fn set_packet_receive_enabled(enabled: bool, max_polls: u32) -> bool {
    let control = crate::platform::mac_register(0x0600) as *mut u32;
    unsafe {
        let value = control.read_volatile();
        if enabled {
            control.write_volatile(value | 1);
            let state = crate::dtcm::LOW_MAC_RECEIVE_GATE_BITS.get() as *mut u8;
            state.write_volatile(state.read_volatile() & !1);
            // Tail of `phy_resume_state4` (`0x2864`). The TX scheduler call is
            // intentionally omitted until its queues are translated.
            (crate::dtcm::LOW_MAC_RECEIVE_STATE_BYTE.get() as *mut u8).write_volatile(4);
            true
        } else {
            control.write_volatile(value & !1);
            let mut polls = 0;
            while control.read_volatile() & (1 << 23) != 0 {
                if polls >= max_polls {
                    return false;
                }
                polls += 1;
                core::hint::spin_loop();
            }
            let state = crate::dtcm::LOW_MAC_RECEIVE_GATE_BITS.get() as *mut u8;
            state.write_volatile(state.read_volatile() | 1);
            true
        }
    }
}

/// Awake-station subset of vendor `phy_state_advance(0)` (`0x820a`).
///
/// JOIN has already completed channel wake and cannot reach the state-1 wake
/// branch here. The retained reprogram latch and state-3 resume tail must still
/// run before the first ordinary class-0 active-TX claim.
///
/// # Safety
/// PHY state and MAC channel registers must be exclusively owned.
pub unsafe fn advance_awake_station_tx() -> bool {
    unsafe {
        write_u8(crate::dtcm::phy_phase().get(), 0);
        let state = (crate::dtcm::LOW_MAC_RECEIVE_STATE_BYTE.get() as *const u8).read_volatile();
        if state == 1 {
            return false;
        }
        let retained_state = (crate::dtcm::phy_retained_state().get() as *const u8).read_volatile();
        // Vendor `phy_state_advance` never writes `0x040099a9`; it only reads
        // it to decide whether to run the reprogram tail. A hardware capture of
        // the accepted class-6 publication versus the refused class-0 one shows
        // the working path publishes with retained state 5, so forcing it to 3
        // here put the PHY into a configuration the MAC does not accept.
        if retained_state == 5 {
            write_u8(crate::dtcm::phy_retained_state().get(), 3);
        }
        if retained_state != 0 {
            crate::mac::reprogram_after_channel();
        }
        write_u8(crate::dtcm::phy_scale_i_byte_unchecked(1).get(), 0);
        if state == 3 && !set_packet_receive_enabled(true, 100_000) {
            return false;
        }
        true
    }
}

/// Exact `phy_cal_step_start` used by `mac_radio_stop` after command 7 has
/// been stopped.
#[cfg(target_arch = "arm")]
pub unsafe fn start_scan_stop_calibration_state() {
    unsafe {
        (0x0ac8_0064 as *mut u32).write_volatile(0x10);
        (crate::dtcm::phy_calibration_stage().get() as *mut u8).write_volatile(1);
        if (crate::dtcm::phy_phase().get() as *const u8).read_volatile() == 3 {
            (crate::dtcm::phy_profile1_ready().get() as *mut u8).write_volatile(0);
            (crate::dtcm::phy_retained_channel().get() as *mut u16).write_volatile(100);
            (crate::dtcm::phy_calibration_state().get() as *mut u8).write_volatile(1);
        }
    }
}

#[cfg(target_arch = "arm")]
pub unsafe fn begin_scan_stop_rx_disable() {
    unsafe {
        let control = (crate::platform::mac_register(0x0600) as *mut u32).read_volatile();
        (crate::platform::mac_register(0x0600) as *mut u32).write_volatile(control & !1);
    }
}

#[cfg(target_arch = "arm")]
pub unsafe fn scan_stop_rx_hardware_drained() -> bool {
    let drained = unsafe {
        (crate::platform::mac_register(0x0600) as *const u32).read_volatile() & (1 << 23) == 0
    };
    if drained {
        unsafe {
            let state =
                (crate::dtcm::LOW_MAC_RECEIVE_GATE_BITS.get() as *mut u8).read_volatile();
            (crate::dtcm::LOW_MAC_RECEIVE_GATE_BITS.get() as *mut u8)
                .write_volatile(state | 1);
        }
    }
    drained
}

unsafe fn run_vendor_mode_calibration(
    calibration_max_polls: u32,
) -> Result<(), IqCalibrationHardwareError> {
    let _ = unsafe { run_iq_calibration_core(true, calibration_max_polls) }?;
    unsafe { run_vendor_dynamic_mode_calibration() };
    Ok(())
}

/// Keep the dynamic-IQ phase out of the preceding IQ calibration frame. The
/// recovered vendor sequence is unchanged, but sequential temporary sets no
/// longer occupy the system stack simultaneously.
#[inline(never)]
unsafe fn run_vendor_dynamic_mode_calibration() {
    // Fixed mode-zero arguments assembled by `rf_apply_channel_settings(0,
    // 0x07ff0110)` at 0x1899c. At 0x18a54 the vendor stores 12 in snapshot
    // offset 0x290; `rf_save_band_regs` uses that field as the mode-zero index
    // into the halfword table at 0x04000dd0. The DFT phase seeds and all search
    // steps likewise come directly from the stack image built there.
    let control_configuration = 0x07ff_0110_u32;
    let table_value = unsafe { (0x0400_0de8 as *const u16).read_volatile() as u32 };
    let sample_width_shift = unsafe { (crate::dtcm::phy_sample_width().get() as *const u16).read_volatile() as u8 };
    let configuration = DynamicIqHardwareCalibrationConfiguration {
        alternate_profile: false,
        table_value,
        synth_frequency: 12,
        calibration_command: 3,
        seed: [-2, 5, -1, -1],
        common_step: 0x100,
        first_step: 0x100,
        second_step: 0x100,
        requested_passes: 0x10,
        configuration_flags: unsafe { (0x0abb_8004 as *const u32).read_volatile() },
        control_configuration,
        dft: DynamicIqDftConfiguration {
            mode: 0,
            first_phase_seed: 0x10,
            second_phase_seed: 0x0c,
            third_phase_seed: 8,
            sample_width_shift,
            capture_polls: 0x2710,
        },
    };
    let mut samples = [0_u32; 64];
    let _ = unsafe { run_vendor_dynamic_iq_hardware_calibration(configuration, &mut samples) };
}

unsafe fn begin_channel_transition(
    channel: u16,
    calibration_max_polls: u32,
) -> Result<ChannelTransitionResult, ChannelTransitionError> {
    unsafe { crate::mac::prepare_scan_context(channel) };
    unsafe { crate::mac::reinitialize_after_wake(calibration_max_polls) }
        .map_err(ChannelTransitionError::MacWake)?;
    unsafe { crate::mac::program_before_scan_channel(channel) };
    if !unsafe { set_packet_receive_enabled(false, calibration_max_polls) } {
        return Err(ChannelTransitionError::InvalidTiming);
    }
    let same_mode_channel = unsafe {
        (crate::dtcm::phy_phase().get() as *const u8).read_volatile() == 2
            && (crate::dtcm::phy_profile().get() as *const u8).read_volatile() == 0
            && (crate::dtcm::phy_channel().get() as *const u16).read_volatile() == channel
            && (crate::dtcm::phy_calibration_state().get() as *const u8).read_volatile() == 0
    };
    if !same_mode_channel {
        unsafe {
            // First channel-switch path `thunk_16c92`: initialize the detector and
            // expanded AGC table before the requested channel operation.
            if (crate::dtcm::LOW_MAC_RECEIVE_STATE_BYTE.get() as *const u8).read_volatile() != 4 {
                apply_first_channel_detector_state();
            }
            // `phy_cal_advance_stage` (0x16bfa) reapplies these four initialized
            // register lists before `phy_cal_set_flag`, whose mode-zero path runs
            // the four RF initialization stages before dispatching the band mode.
            for list in COMMON_INITIALIZATION_LISTS {
                apply_register_list(list);
            }
            // Scan rate configuration 0x0117 selects dispatcher mode 2 in
            // `phy_do_channel_switch` (0xf7fc). The first channel performs a
            // pre-calibration dispatch, RF initialization, then the requested
            // mode-2 dispatch.
            program_mode2_band_hardware();
            prepare_rf_mode0_stage();
            // Vendor `phy_init_once` keeps this outside its one-shot guard and
            // rebuilds the SDD-corrected AGC table after RF initialization.
            build_mode0_gain_tables();
            program_scan_receive_band();
            write_u16(crate::dtcm::phy_channel().get(), channel);
        }
    }
    let divider = unsafe { program_channel_pll(channel) }.map_err(ChannelTransitionError::Pll)?;

    let mut calibration_ran = false;
    if !same_mode_channel && unsafe { (crate::dtcm::phy_transition_gate().get() as *const u8).read_volatile() } != 0 {
        unsafe { run_vendor_mode_calibration(calibration_max_polls) }
            .map_err(ChannelTransitionError::Calibration)?;
        calibration_ran = true;
    }

    let timing = unsafe { program_channel_measurement_timing() }
        .ok_or(ChannelTransitionError::InvalidTiming)?;
    let temperature =
        unsafe { measure_temperature_primary() }.map_err(ChannelTransitionError::Temperature)?;
    unsafe { copy_channel_configuration_slot(0) };

    if !same_mode_channel
        && unsafe { prepare_channel_calibration_cache() }
            == ChannelCalibrationRequirement::RunCalibration
    {
        unsafe { run_vendor_mode_calibration(calibration_max_polls) }
            .map_err(ChannelTransitionError::Calibration)?;
        unsafe { record_calibrated_channel(channel) };
        calibration_ran = true;
    }

    unsafe { set_phy_agc_enabled(true) };
    let (threshold, first_tx_power, second_tx_power) =
        unsafe { publish_channel_power(channel as u8) }.map_err(ChannelTransitionError::Power)?;
    let frequency_offset = unsafe { publish_channel_frequency_offset() };
    // Linux normally supplies MIB 0x0006 before channel activation. Preserve
    // its standard 20 dBm default for early scan bring-up if that write has
    // not arrived yet; the vendor path consumes the same value in deci-dBm.
    let tx_power_tenths_dbm = crate::configuration::current_tx_power_tenths_dbm().unwrap_or(200);
    unsafe { program_all_tx_gain_slots(tx_power_tenths_dbm) }
        .map_err(ChannelTransitionError::Gain)?;
    // First return from vendor `phy_cal_run_step_timed`: state 1, followed by
    // a 120-tick cooperative settle interval.
    unsafe {
        write_u8(crate::dtcm::phy_calibration_state().get(), 0);
        write_u8(crate::dtcm::MAC_WAKE_PHY_STATE.get(), 1);
    };
    Ok(ChannelTransitionResult {
        divider,
        timing,
        temperature,
        calibration_ran,
        threshold,
        first_tx_power,
        second_tx_power,
        frequency_offset,
    })
}

unsafe fn finish_channel_transition(
    calibration_max_polls: u32,
) -> Result<(), ChannelTransitionError> {
    unsafe {
        // Preserve vendor wake ordering: full MAC reinitialization can set the
        // channel-reprogram flag consumed immediately afterward.
        if (crate::dtcm::MAC_WAKE_RESTORE_PENDING.get() as *const u8).read_volatile() != 0 {
            crate::mac::reinitialize_after_wake(calibration_max_polls)
                .map_err(ChannelTransitionError::MacWake)?;
        }
        let status = crate::dtcm::LOW_MAC_FIFO_STATUS.get() as *mut u8;
        let value = status.read_volatile();
        if value & 1 != 0 {
            status.write_volatile(value & !1);
        } else if value & 4 == 0 && (crate::dtcm::MAC_WAKE_TRANSITION_PENDING.get() as *const u8).read_volatile() != 0 {
            crate::mac::reprogram_after_channel();
        }
        publish_completed_receive_state();
        set_packet_receive_enabled(true, calibration_max_polls);
        crate::mac::program_scan_station_mode();
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChannelTransitionScheduler {
    state: u8,
    deadline: u32,
    result: ChannelTransitionResult,
    calibration_max_polls: u32,
}

impl ChannelTransitionScheduler {
    pub const fn new() -> Self {
        Self {
            state: 0,
            deadline: 0,
            result: ChannelTransitionResult {
                divider: PllDivider {
                    integer: 0,
                    fractional: 0,
                    register: 0,
                },
                timing: 0,
                temperature: TemperatureMeasurement {
                    value: 0,
                    accepted: false,
                },
                calibration_ran: false,
                threshold: 0,
                first_tx_power: 0,
                second_tx_power: 0,
                frequency_offset: 0,
            },
            calibration_max_polls: 0,
        }
    }

    pub fn is_idle(&self) -> bool {
        self.state == 0
    }

    /// Starts the hardware-producing phase and arms the vendor 120-tick settle
    /// phase. This is cooperative rather than a Rust `Future`: no executor,
    /// allocation, or wake infrastructure is required.
    pub unsafe fn start(
        &mut self,
        channel: u16,
        calibration_max_polls: u32,
    ) -> Result<(), ChannelTransitionError> {
        if !self.is_idle() {
            return Err(ChannelTransitionError::InvalidTiming);
        }
        // Vendor `mac_set_channel` returns immediately when the requested MAC
        // channel is already active. Preserve RX/FIFO state instead of cycling
        // the analogue front end for every repeated single-channel scan.
        if unsafe {
            (crate::dtcm::LOW_MAC_CURRENT_CHANNEL.get() as *const u16).read_volatile() == channel
                && (crate::dtcm::LOW_MAC_RECEIVE_STATE_BYTE.get() as *const u8).read_volatile() == 4
                && (crate::dtcm::phy_phase().get() as *const u8).read_volatile() == 2
        } {
            let (integer, fractional) = unsafe { cached_pll_divider() };
            self.result.divider = PllDivider {
                integer,
                fractional,
                register: integer.wrapping_shl(21) | fractional,
            };
            self.result.calibration_ran = false;
            self.calibration_max_polls = calibration_max_polls;
            self.state = 3;
            return Ok(());
        }
        let result = unsafe { begin_channel_transition(channel, calibration_max_polls) }?;
        self.calibration_max_polls = calibration_max_polls;
        self.result = result;
        self.state = 1;
        Ok(())
    }

    /// Starts the 120-tick interval after the hardware-producing phase has
    /// returned, matching the placement of `fw_read_timer()` in vendor
    /// `phy_cal_run_step_timed`.
    pub fn arm_settle(&mut self, now: u32) {
        if self.state == 1 {
            self.deadline = now.wrapping_add(0x78);
            self.state = 2;
        }
    }

    /// Advances the timer-driven phase and returns the completed transition.
    pub unsafe fn service(
        &mut self,
        now: u32,
    ) -> Result<Option<ChannelTransitionResult>, ChannelTransitionError> {
        if self.state == 3 {
            self.state = 0;
            return Ok(Some(self.result));
        }
        if self.state != 2 {
            return Ok(None);
        }
        if (now.wrapping_sub(self.deadline) as i32) < 0 {
            return Ok(None);
        }
        unsafe { finish_channel_transition(self.calibration_max_polls) }?;
        self.state = 0;
        Ok(Some(self.result))
    }
}

impl Default for ChannelTransitionScheduler {
    fn default() -> Self {
        Self::new()
    }
}

pub unsafe fn run_channel_transition(
    channel: u16,
    calibration_max_polls: u32,
) -> Result<ChannelTransitionResult, ChannelTransitionError> {
    let result = unsafe { begin_channel_transition(channel, calibration_max_polls) }?;
    delay_timer_ticks(0x78);
    unsafe { finish_channel_transition(calibration_max_polls) }?;
    Ok(result)
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

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct IqCalibrationReferences {
    pub coefficient_i: i32,
    pub coefficient_q: i32,
    pub scale_i: i32,
    pub scale_q: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IqCalibrationHardwareResult {
    pub series: IqCalibrationSeries,
    pub references: IqCalibrationReferences,
    pub secondary: Option<IqCalibrationCoefficient>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IqCalibrationHardwareError {
    BaselineTimeout { gain_index: u32 },
    TargetTimeout { gain_index: u32 },
    SecondaryBaselineTimeout,
    SecondaryTargetTimeout,
    InvalidSecondaryScale,
}

struct SharedIqCalibrationScratch {
    samples: UnsafeCell<[IqCalibrationSample; 12]>,
    series: UnsafeCell<MaybeUninit<IqCalibrationSeries>>,
}
unsafe impl Sync for SharedIqCalibrationScratch {}

static IQ_CALIBRATION_SCRATCH: SharedIqCalibrationScratch = SharedIqCalibrationScratch {
    samples: UnsafeCell::new(
        [IqCalibrationSample {
            baseline_i: 0,
            baseline_q: 0,
            target_i: 0,
            target_q: 0,
        }; 12],
    ),
    series: UnsafeCell::new(MaybeUninit::uninit()),
};

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

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DynamicIqStageExecution {
    pub capture_samples: bool,
    pub program_candidate: bool,
    pub correlate_samples: bool,
    pub initial_control_before_program: bool,
    pub initial_control_before_correlate: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DynamicIqStageControl {
    pub hardware: u32,
    pub correlation: u32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DynamicIqDftConfiguration {
    pub mode: u8,
    pub first_phase_seed: u32,
    pub second_phase_seed: u32,
    pub third_phase_seed: u32,
    pub sample_width_shift: u8,
    pub capture_polls: u32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DynamicIqHardwareStageResult {
    pub measurement: DynamicIqDftResult,
    pub capture_ready: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DynamicIqHardwareSearchResult {
    pub search: Option<DynamicIqAveragedSearch>,
    pub all_captures_ready: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DynamicIqHardwareVerificationResult {
    pub metrics: DynamicIqVerificationMetrics,
    pub capture_ready: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DynamicIqHardwareCalibrationConfiguration {
    pub alternate_profile: bool,
    pub table_value: u32,
    pub synth_frequency: i32,
    pub calibration_command: i32,
    pub seed: [i32; 4],
    pub common_step: i32,
    pub first_step: i32,
    pub second_step: i32,
    pub requested_passes: u8,
    pub configuration_flags: u32,
    pub control_configuration: u32,
    pub dft: DynamicIqDftConfiguration,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DynamicIqHardwareCalibrationResult {
    pub profile_shortcut: bool,
    pub synth_prepared: bool,
    pub finalization: Option<DynamicIqFinalization>,
    pub all_search_captures_ready: bool,
    pub verification_capture_ready: bool,
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

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DynamicIqAverage {
    pub accumulated: [i32; 4],
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DynamicIqVerificationMetrics {
    pub reference: i32,
    pub second: i32,
    pub third: i32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DynamicIqAveragedSearch {
    pub baseline: DynamicIqVerificationMetrics,
    pub final_candidate: [i32; 4],
    pub completed_passes: u32,
    pub accepted_passes: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DynamicIqFinalization {
    pub verification: DynamicIqVerification,
    pub publication: Option<DynamicIqCorrectionPlan>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DynamicIqVerification {
    pub retained_values: [i32; 4],
    pub first_pair_rejected: bool,
    pub second_pair_rejected: bool,
    pub first_quality_failed: bool,
    pub second_quality_failed: bool,
    /// The vendor only checks rejection of the second packed pair before
    /// publishing both words. The first-pair rejection flag is calculated but
    /// not consulted; this odd behavior is preserved explicitly.
    pub vendor_publication_allowed: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DynamicIqBandDerivedValues {
    pub abc0020: u32,
    pub abc0030: u32,
    pub abb801c: u32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DynamicIqCalibrationCommandWords {
    pub abb805c: u32,
    pub abb8060: u32,
    pub abb8064: u32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DynamicIqBandRegisterSnapshot {
    pub abd0000: u32,
    pub abc0004: u32,
    pub abc0008: u32,
    pub abc0020: u32,
    pub abc0024: u32,
    pub abc0028: u32,
    pub abc0030: u32,
    pub abc0034: u32,
    pub abc0050: u32,
    pub abc006c: u32,
    pub abc0084: u32,
    pub abc00b4: u32,
    pub abc00fc: u32,
    pub abb8004: u32,
    pub abb800c: u32,
    pub abb801c: u32,
    pub abb8068: u32,
    pub abb80a8: u32,
    pub abb81a4: u32,
    pub abb81a8: u32,
    pub restore_abb8068: bool,
    pub restore_abb80a8: bool,
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

pub fn unpack_dynamic_iq_pair(value: u32) -> (i32, i32) {
    let decode = |raw: u32| ((raw << 20) as i32) >> 20;
    (decode(value & 0x0fff), decode((value >> 16) & 0x0fff))
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

/// Publish both packed correction words in vendor order.
///
/// # Safety
///
/// The MAC/PHY register bank must be enabled and writable.
pub unsafe fn apply_dynamic_iq_correction_plan(plan: DynamicIqCorrectionPlan) {
    unsafe {
        write_u32(plan.first_address as usize, plan.first_value);
        write_u32(plan.second_address as usize, plan.second_value);
    }
}

/// Replicate the final packed pair across the two sixteen-word correction banks
/// before verification, matching `0x18dae..0x18dd0`.
///
/// # Safety
///
/// The MAC/PHY register bank must be enabled and writable.
pub unsafe fn replicate_dynamic_iq_correction_banks(plan: DynamicIqCorrectionPlan) {
    unsafe {
        for index in 1..16_u32 {
            write_u32(
                plan.first_address.wrapping_add(index * 4) as usize,
                plan.first_value,
            );
            write_u32(
                plan.second_address.wrapping_add(index * 4) as usize,
                plan.second_value,
            );
        }
    }
}

/// Publish verification flags and accepted packed words into the profile's
/// persistent software state. Existing flags are only set, never cleared.
///
/// # Safety
///
/// DTCM at `0x0400994c` must contain the initialized vendor calibration state.
pub unsafe fn publish_dynamic_iq_final_state(
    alternate_profile: bool,
    finalization: DynamicIqFinalization,
) {
    let base = crate::dtcm::PHY_PROFILE_STATE.get();
    unsafe {
        if finalization.verification.second_quality_failed {
            write_u8(base + if alternate_profile { 0x7a } else { 0x78 }, 1);
        }
        if finalization.verification.first_quality_failed {
            write_u8(base + if alternate_profile { 0x7b } else { 0x79 }, 1);
        }
        if let Some(publication) = finalization.publication {
            let value_offset = if alternate_profile { 0x70 } else { 0x68 };
            write_u32(base + value_offset, publication.first_value);
            write_u32(base + value_offset + 4, publication.second_value);
            write_u8(base + if alternate_profile { 0x15 } else { 0x0d }, 1);
        }
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

pub fn dynamic_iq_verification_metrics(
    measurement: DynamicIqDftResult,
) -> DynamicIqVerificationMetrics {
    let [reference, second, third] =
        dynamic_iq_metrics([measurement.first, measurement.second, measurement.third]);
    DynamicIqVerificationMetrics {
        reference,
        second,
        third,
    }
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

/// Hardware-operation schedule around one dispatcher stage in annotated
/// `rf_apply_channel_settings`. Operations occur in field order: capture,
/// candidate publication, then DFT correlation. Stages 0 and 7 intentionally
/// dispatch without a fresh measurement.
pub fn dynamic_iq_stage_execution(stage: u8, first_pass: bool) -> Option<DynamicIqStageExecution> {
    if stage >= 13 {
        return None;
    }
    let measure = stage != 0 && stage != 7;
    Some(DynamicIqStageExecution {
        capture_samples: measure,
        program_candidate: stage != 6 && stage != 12,
        correlate_samples: measure,
        initial_control_before_program: first_pass && stage == 0,
        initial_control_before_correlate: first_pass && stage == 1,
    })
}

/// Stage-local control-word construction from annotated
/// `rf_apply_channel_settings`. Stage 1's first-pass override affects the DFT
/// input but is deliberately not republished to the hardware register.
pub fn dynamic_iq_stage_control(
    configuration: u32,
    stage: u8,
    first_pass: bool,
) -> Option<DynamicIqStageControl> {
    if stage >= 13 {
        return None;
    }
    let phase = configuration >> 22;
    let stage_phase = if stage < 7 { phase >> 1 } else { phase };
    let base = (configuration & 0xffff_9fff) | 0x003f_0000;
    let control =
        ((base & 0x003f_ffff) | stage_phase.wrapping_shl(16)) & !0xff | (stage_phase & 0xff);
    let initial = |value: u32| (value & !0xff) | 0xff | 0xffc0_0000;
    let hardware = if first_pass && stage == 0 {
        initial(control)
    } else {
        control
    };
    let correlation = if first_pass && stage == 1 {
        initial(control)
    } else {
        hardware
    };
    Some(DynamicIqStageControl {
        hardware,
        correlation,
    })
}

/// Control word used by the post-average verification capture. The vendor
/// retains the full-phase stage-12 fields, then forces the low byte and upper
/// control mask before both hardware publication and DFT.
pub fn dynamic_iq_verification_control(configuration: u32) -> u32 {
    let control = dynamic_iq_stage_control(configuration, 12, false)
        .map(|value| value.hardware)
        .unwrap_or(0);
    (control & !0xff) | 0xff | 0xffc0_0000
}

/// Apply the persistent search-state effects of one switch case from annotated
/// `rf_op_dispatch2`.
///
/// Cases 1 and 8 consume all three DFT outputs for their local magnitude
/// calculations. The retained correlations below mirror the vendor workspace;
/// transient normalized magnitudes remain internal to the refinement helpers.
/// Cases 6 and 12 perform the two analytical refinements.
pub fn run_dynamic_iq_search_pass<F>(
    state: &mut DynamicIqSearchState,
    first_pass: bool,
    mut execute: F,
) -> bool
where
    F: FnMut(u8, [i32; 4], DynamicIqStageExecution) -> Option<DynamicIqDftResult>,
{
    for stage in 0..13 {
        let Some(execution) = dynamic_iq_stage_execution(stage, first_pass) else {
            return false;
        };
        let Some(measurement) = execute(stage, state.candidate, execution) else {
            return false;
        };
        if !apply_dynamic_iq_search_stage(state, stage, measurement) {
            return false;
        }
        state.candidate = normalize_dynamic_iq_candidate(state.candidate);
        if stage == 6 || stage == 12 {
            state.current = state.candidate;
        }
    }
    true
}

pub fn run_dynamic_iq_averaged_search<F>(
    initial: [i32; 4],
    seed: [i32; 4],
    common_step: i32,
    first_step: i32,
    second_step: i32,
    requested_passes: u8,
    configuration_flags: u32,
    mut execute: F,
) -> Option<DynamicIqAveragedSearch>
where
    F: FnMut(u32, u8, [i32; 4], DynamicIqStageExecution) -> Option<DynamicIqDftResult>,
{
    let completed_passes = dynamic_iq_search_pass_count(requested_passes, configuration_flags);
    if completed_passes < 2 {
        return None;
    }
    let mut average = DynamicIqAverage::default();
    let mut baseline = None;
    for pass_index in 0..completed_passes {
        let candidate = average.candidate_for_pass(pass_index, initial);
        let mut state = DynamicIqSearchState::new(candidate, common_step, first_step, second_step);
        if !run_dynamic_iq_search_pass(&mut state, pass_index == 0, |stage, values, plan| {
            let measurement = execute(pass_index, stage, values, plan)?;
            if pass_index == 0 && stage == 1 {
                baseline = Some(dynamic_iq_verification_metrics(measurement));
            }
            Some(measurement)
        }) {
            return None;
        }
        average.retain_pass(pass_index, state.current);
    }
    let accepted_passes = completed_passes - 1;
    Some(DynamicIqAveragedSearch {
        baseline: baseline?,
        final_candidate: average.final_candidate(accepted_passes, seed)?,
        completed_passes,
        accepted_passes,
    })
}

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

/// Outer search-pass count from annotated `rf_apply_channel_settings`.
/// Configuration bit 1 doubles the low-byte pass count.
pub const fn dynamic_iq_search_pass_count(requested: u8, configuration_flags: u32) -> u32 {
    requested as u32 * if configuration_flags & 2 == 0 { 1 } else { 2 }
}

impl DynamicIqAverage {
    /// Candidate used to begin a vendor averaging pass. Passes zero and one use
    /// the caller's current value; later passes start from the average of all
    /// accepted passes 1..pass_index-1.
    pub fn candidate_for_pass(&self, pass_index: u32, current: [i32; 4]) -> [i32; 4] {
        if pass_index <= 1 {
            return current;
        }
        self.accumulated
            .map(|value| value.wrapping_div(pass_index as i32 - 1))
    }

    /// Pass zero is the vendor baseline pass and is deliberately excluded from
    /// the accepted-candidate accumulator.
    pub fn retain_pass(&mut self, pass_index: u32, values: [i32; 4]) {
        if pass_index == 0 {
            return;
        }
        for (sum, value) in self.accumulated.iter_mut().zip(values) {
            *sum = sum.wrapping_add(value);
        }
    }

    pub fn final_candidate(&self, accepted_count: u32, seed: [i32; 4]) -> Option<[i32; 4]> {
        if accepted_count == 0 {
            return None;
        }
        let divisor = accepted_count as i32;
        Some(core::array::from_fn(|index| {
            self.accumulated[index]
                .wrapping_div(divisor)
                .wrapping_add(seed[index])
        }))
    }
}

/// Final additive seed at annotated `0x18d4e..0x18db0`.
pub fn dynamic_iq_final_seed(
    alternate_profile: bool,
    initial: DynamicIqInitialCandidate,
) -> [i32; 4] {
    if alternate_profile {
        [
            i32::from(initial.first),
            i32::from(initial.second),
            i32::from(initial.third),
            i32::from(initial.fourth),
        ]
    } else {
        [-2, 5, -1, -1]
    }
}

/// Pure verification and restoration decisions from annotated
/// `0x18dfe..0x18e9e`, cross-checked against Radare2 `pd:g` and Thumb
/// disassembly. `baseline` is the pass-zero stage-one metric triplet.
pub fn verify_dynamic_iq_candidate(
    values: [i32; 4],
    saved_values: [i32; 4],
    baseline: DynamicIqVerificationMetrics,
    measured: DynamicIqVerificationMetrics,
) -> DynamicIqVerification {
    let second_pair_rejected =
        baseline.second <= measured.second >> 2 || measured.reference <= measured.second;
    let first_pair_rejected =
        baseline.third <= measured.third >> 2 || measured.reference <= measured.third;
    let mut retained_values = values;
    if second_pair_rejected {
        retained_values[2] = saved_values[2];
        retained_values[3] = saved_values[3];
    }
    if first_pair_rejected {
        retained_values[0] = saved_values[0];
        retained_values[1] = saved_values[1];
    }
    DynamicIqVerification {
        retained_values,
        first_pair_rejected,
        second_pair_rejected,
        first_quality_failed: measured.reference <= measured.third.wrapping_shl(10),
        second_quality_failed: measured.reference <= measured.second.wrapping_shl(10),
        // This intentionally preserves the vendor's asymmetric final test at
        // 0x18e82: only the second-pair rejection flag suppresses publication.
        vendor_publication_allowed: !second_pair_rejected,
    }
}

pub fn finalize_dynamic_iq_search(
    averaged: DynamicIqAveragedSearch,
    saved_values: [i32; 4],
    measured: DynamicIqVerificationMetrics,
) -> DynamicIqFinalization {
    let verification = verify_dynamic_iq_candidate(
        averaged.final_candidate,
        saved_values,
        averaged.baseline,
        measured,
    );
    // The vendor snapshots the packed candidate before verification and
    // conditionally publishes those original words to software state. A
    // rejected first pair is restored in hardware later but is not substituted
    // in this asymmetric software publication.
    let publication = verification
        .vendor_publication_allowed
        .then(|| dynamic_iq_correction_plan(averaged.final_candidate));
    DynamicIqFinalization {
        verification,
        publication,
    }
}

/// Forced switch case 1 of `phy_cal_cmd_dispatch` (`0x16990`). Radare2 `pd:g`
/// confirms that the wrapper's fixed argument word `8` contributes bits 27 and
/// 31, producing `0x88000000 | (command & 0xff)`.
pub fn dynamic_iq_calibration_start_words(command: i32) -> DynamicIqCalibrationCommandWords {
    DynamicIqCalibrationCommandWords {
        abb805c: 0x12,
        abb8060: 0x8800_0000 | (command as u32 & 0xff),
        abb8064: 0,
    }
}

/// Forced switch case 0 of `phy_cal_cmd_dispatch`: clear all three command
/// words at `0x0abb805c..0x0abb8064`.
pub const fn dynamic_iq_calibration_stop_words() -> DynamicIqCalibrationCommandWords {
    DynamicIqCalibrationCommandWords {
        abb805c: 0,
        abb8060: 0,
        abb8064: 0,
    }
}

unsafe fn write_dynamic_iq_calibration_command(words: DynamicIqCalibrationCommandWords) {
    unsafe {
        write_u32(0x0abb_805c, words.abb805c);
        write_u32(0x0abb_8060, words.abb8060);
        write_u32(0x0abb_8064, words.abb8064);
    }
}

/// Simple profile branch of annotated `rf_program_synth_freq` (`0x1869e`).
/// It performs the vendor's 21-step restoring division rather than replacing
/// it with wider host arithmetic, preserving all 32-bit wraparound behavior.
pub fn dynamic_iq_synth_register_mode1(frequency: i32, integer: i32, reference: u32) -> u32 {
    let mut remainder = 0x0004_c4b4_u32.wrapping_mul(frequency as u32);
    let mut divisor = reference.wrapping_mul(1000);
    let mut fractional = 0_u32;
    for bit in (0..=20).rev() {
        divisor >>= 1;
        if divisor <= remainder {
            remainder = remainder.wrapping_sub(divisor);
            fractional |= 1 << bit;
        }
    }
    fractional | (integer as u32).wrapping_shl(21)
}

/// Larger fixed-point branch of annotated `rf_program_synth_freq`, expressed
/// as the same wrapping 64-bit operations performed by the firmware helpers.
pub fn dynamic_iq_synth_register_mode0(
    frequency: u32,
    current_register: u32,
    reference: u32,
) -> Option<u32> {
    let first = u64::from(current_register.wrapping_add(10)).wrapping_mul(u64::from(reference))
        / 0x9c40_0000;
    let scaled_reference = u64::from(reference).wrapping_shl(25) / 1000;
    if scaled_reference == 0 {
        return None;
    }
    let ratio = first.wrapping_shl(25) / scaled_reference;
    let combined = scaled_reference
        .wrapping_mul(ratio)
        .wrapping_add(0x00a0_0000_u64.wrapping_mul(u64::from(frequency)));
    let denominator_base = scaled_reference.wrapping_shl(3) / 10;
    let denominator = denominator_base.wrapping_shl(3);
    if denominator == 0 {
        return None;
    }
    let combined = combined.wrapping_shl(3);
    let quotient = combined / denominator;
    let remainder = combined.wrapping_sub(quotient.wrapping_mul(denominator));
    let fraction = remainder.wrapping_shl(28) / denominator;
    let scaled_fraction = fraction.wrapping_mul(0x20_0000);
    let packed =
        scaled_fraction.wrapping_add((quotient & 0x7ff).wrapping_mul(0x2_0000).wrapping_shl(32));
    Some((packed / 0x1000_0000) as u32)
}

pub fn dynamic_iq_synth_register(
    mode: u8,
    frequency: i32,
    current_register: u32,
    reference: u32,
) -> Option<u32> {
    if mode == 1 {
        Some(dynamic_iq_synth_register_mode1(
            frequency,
            (current_register >> 21) as i32,
            reference,
        ))
    } else {
        dynamic_iq_synth_register_mode0(frequency as u32, current_register, reference)
    }
}

/// Read the live mode, reference and current PLL word and calculate the synth
/// result before any band register is modified.
///
/// # Safety
///
/// The DTCM calibration state and PLL register must be initialized and readable.
pub unsafe fn prepare_dynamic_iq_synth_register(frequency: i32) -> Option<u32> {
    unsafe {
        let mode = (crate::dtcm::phy_profile().get() as *const u8).read_volatile();
        let reference = (crate::dtcm::phy_reference_word().get() as *const u32).read_volatile();
        let current_register = (0x0abc_00b4 as *const u32).read_volatile();
        dynamic_iq_synth_register(mode, frequency, current_register, reference)
    }
}

pub fn dynamic_iq_band_derived_values(
    alternate_profile: bool,
    table_value: u32,
    source_04001ff4: u32,
) -> DynamicIqBandDerivedValues {
    let abc0020 = if alternate_profile {
        0x1f00_d000
    } else {
        ((source_04001ff4 & 0x07ff_ffff) >> 26)
            .wrapping_mul(0x2000_0000)
            .wrapping_add(0x0f00_c000)
    };
    DynamicIqBandDerivedValues {
        abc0020,
        abc0030: 0x3ff_u32.wrapping_sub(table_value),
        abb801c: table_value
            .wrapping_mul(0x400)
            .wrapping_add(if alternate_profile { 0x100 } else { 0xc0 }),
    }
}

/// Detached acquisition envelope from annotated `rf_save_band_regs`
/// (`0x187f4`), cross-checked against uninterrupted Thumb disassembly.
///
/// # Safety
///
/// The MAC/PHY banks must be enabled. `synth_register` must have been prepared
/// from the still-unmodified live state. Pair this with
/// [`restore_dynamic_iq_band_registers`] on every non-fatal path.
pub unsafe fn begin_dynamic_iq_band_registers(
    alternate_profile: bool,
    table_value: u32,
    synth_register: u32,
    calibration_command: i32,
) -> DynamicIqBandRegisterSnapshot {
    let mut snapshot = DynamicIqBandRegisterSnapshot::default();
    unsafe {
        delay_units(1);
        snapshot.abd0000 = (0x0abd_0000 as *const u32).read_volatile();
        write_u32(0x0abd_0000, 0x0000_4000);
        snapshot.abc0004 = (0x0abc_0004 as *const u32).read_volatile();
        write_u32(0x0abc_0004, 0x0000_0306);
        snapshot.abc0008 = (0x0abc_0008 as *const u32).read_volatile();
        write_u32(0x0abc_0008, 0x0000_8000);

        let derived;
        if alternate_profile {
            derived = dynamic_iq_band_derived_values(true, table_value, 0);
            delay_units(1);
            snapshot.abc0028 = (0x0abc_0028 as *const u32).read_volatile();
            write_u32(0x0abc_0028, 0x0be0_1f01);
            snapshot.abc00fc = (0x0abc_00fc as *const u32).read_volatile();
            write_u32(0x0abc_00fc, 0);
            snapshot.abc006c = (0x0abc_006c as *const u32).read_volatile();
            write_u32(0x0abc_006c, 0x0001_70fe);
            snapshot.abc0084 = (0x0abc_0084 as *const u32).read_volatile();
            write_u32(0x0abc_0084, 0x0840_7872);
            snapshot.abc0034 = (0x0abc_0034 as *const u32).read_volatile();
            write_u32(0x0abc_0034, 0x0000_0ecf);
            snapshot.abc0020 = (0x0abc_0020 as *const u32).read_volatile();
            write_u32(0x0abc_0020, derived.abc0020);
            snapshot.abc0050 = (0x0abc_0050 as *const u32).read_volatile();
            write_u32(0x0abc_0050, 0x0021_7c63);
            snapshot.abc0024 = (0x0abc_0024 as *const u32).read_volatile();
            write_u32(0x0abc_0024, 0x09ff_d803);
            delay_units(10);
            write_u32(0x0abc_0050, 0x0020_7c63);
            write_u32(0x0abc_0084, 0x0840_6872);
            // The vendor leaves the zeroed workspace slot as the restore value
            // in this branch instead of capturing the live register.
            write_u32(0x0abb_81a4, 0x5a);
        } else {
            snapshot.abc0028 = (0x0abc_0028 as *const u32).read_volatile();
            write_u32(0x0abc_0028, 0x0900_1f01);
            snapshot.abc00fc = (0x0abc_00fc as *const u32).read_volatile();
            write_u32(0x0abc_00fc, 0);
            snapshot.abc006c = (0x0abc_006c as *const u32).read_volatile();
            write_u32(0x0abc_006c, 0x0002_4f36);
            snapshot.abc0084 = (0x0abc_0084 as *const u32).read_volatile();
            write_u32(0x0abc_0084, 0x0840_b872);
            snapshot.abc0034 = (0x0abc_0034 as *const u32).read_volatile();
            write_u32(0x0abc_0034, 0x0000_06c3);
            snapshot.abc0020 = (0x0abc_0020 as *const u32).read_volatile();
            let source_04001ff4 = (crate::dtcm::scheduler_remap_primary().get() as *const u32).read_volatile();
            derived = dynamic_iq_band_derived_values(false, table_value, source_04001ff4);
            write_u32(0x0abc_0020, derived.abc0020);
            snapshot.abc0050 = (0x0abc_0050 as *const u32).read_volatile();
            write_u32(0x0abc_0050, 0x0021_7c63);
            snapshot.abc0024 = (0x0abc_0024 as *const u32).read_volatile();
            write_u32(0x0abc_0024, 0x0987_1800);
            delay_units(10);
            write_u32(0x0abc_0050, 0x0020_7c63);
            write_u32(0x0abc_0084, 0x0840_a872);
            snapshot.abb81a4 = (0x0abb_81a4 as *const u32).read_volatile();
            write_u32(0x0abb_81a4, 0x56);
        }

        snapshot.abb801c = (0x0abb_801c as *const u32).read_volatile();
        write_u32(0x0abb_801c, derived.abb801c);
        snapshot.abc0030 = (0x0abc_0030 as *const u32).read_volatile();
        write_u32(0x0abc_0030, derived.abc0030);
        snapshot.abc00b4 = (0x0abc_00b4 as *const u32).read_volatile();
        let mode = (crate::dtcm::phy_profile().get() as *const u8).read_volatile();
        let extended_settle = (crate::dtcm::phy_extended_settle().get() as *const u8).read_volatile() != 0;
        commit_channel_pll(synth_register, mode, extended_settle);
        snapshot.abb800c = (0x0abb_800c as *const u32).read_volatile();
        write_u32(0x0abb_800c, 1);
        snapshot.abb8068 = (0x0abb_8068 as *const u32).read_volatile();
        snapshot.abb80a8 = (0x0abb_80a8 as *const u32).read_volatile();
        snapshot.abb81a8 = (0x0abb_81a8 as *const u32).read_volatile();
        write_u32(0x0abb_81a8, 2);
        write_dynamic_iq_calibration_command(dynamic_iq_calibration_start_words(
            calibration_command,
        ));
        snapshot.abb8004 = (0x0abb_8004 as *const u32).read_volatile();
        write_u32(0x0abb_8004, 0x0019_8600);
        delay_units(6);
    }
    snapshot
}

/// Detached restoration envelope from annotated `rf_load_band_regs`
/// (`0x1843e`). The correction-register restore flags intentionally remain
/// mutable because the vendor workspace can change them during acquisition.
///
/// # Safety
///
/// The snapshot must come from the matching acquisition.
pub unsafe fn restore_dynamic_iq_band_registers(snapshot: &DynamicIqBandRegisterSnapshot) {
    unsafe {
        write_u32(0x0abc_0004, snapshot.abc0004);
        write_u32(0x0abc_0008, snapshot.abc0008);
        write_u32(0x0abc_0028, snapshot.abc0028);
        write_u32(0x0abc_0030, snapshot.abc0030);
        write_u32(0x0abc_00fc, snapshot.abc00fc);
        write_u32(0x0abc_006c, snapshot.abc006c);
        write_u32(0x0abc_0084, snapshot.abc0084);
        write_u32(0x0abc_0034, snapshot.abc0034);
        write_u32(0x0abc_0020, snapshot.abc0020);
        write_u32(0x0abc_0050, snapshot.abc0050);
        write_u32(0x0abc_0024, snapshot.abc0024);
        delay_units(10);
        write_u32(0x0abb_81a4, snapshot.abb81a4);
        write_dynamic_iq_calibration_command(dynamic_iq_calibration_stop_words());
        write_u32(0x0abb_800c, snapshot.abb800c);
        write_u32(0x0abb_801c, snapshot.abb801c);
        write_u32(0x0abb_81a8, snapshot.abb81a8);
        if snapshot.restore_abb80a8 {
            write_u32(0x0abb_80a8, snapshot.abb80a8);
        }
        if snapshot.restore_abb8068 {
            write_u32(0x0abb_8068, snapshot.abb8068);
        }
        write_u32(0x0abb_8004, snapshot.abb8004);
        let mode = (crate::dtcm::phy_profile().get() as *const u8).read_volatile();
        let extended_settle = (crate::dtcm::phy_extended_settle().get() as *const u8).read_volatile() != 0;
        commit_channel_pll(snapshot.abc00b4, mode, extended_settle);
        write_u32(0x0abd_0000, snapshot.abd0000);
        delay_units(6);
    }
}

/// Control update from vendor `0x18600`, named `phy_set_reg2c_bit8` in the
/// annotated firmware archive.
pub fn dynamic_iq_capture_control(control: u32) -> u32 {
    control | 0x100
}

/// Publish the stage control to `0x0abb81ac`, matching
/// `phy_set_reg2c_bit8` (`0x18654`).
///
/// # Safety
///
/// The MAC/PHY register bank must be enabled and writable.
pub unsafe fn apply_dynamic_iq_capture_control(control: u32) -> u32 {
    let published = dynamic_iq_capture_control(control);
    unsafe { write_u32(0x0abb_81ac, published) };
    published
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

/// Execute one detached hardware stage in exact vendor order. Capture timeout
/// is reported but does not suppress the 64-word copy or DFT, matching
/// `rf_capture_adc_samples`.
///
/// # Safety
///
/// The calibration engine and MAC/PHY register banks must already be enabled.
pub unsafe fn execute_dynamic_iq_hardware_stage(
    candidate: [i32; 4],
    execution: DynamicIqStageExecution,
    control: DynamicIqStageControl,
    configuration: DynamicIqDftConfiguration,
    samples: &mut [u32; 64],
) -> DynamicIqHardwareStageResult {
    let mut capture_ready = true;
    if execution.capture_samples {
        capture_ready = unsafe { capture_dynamic_iq_samples(samples, configuration.capture_polls) };
    }
    if execution.program_candidate {
        unsafe { apply_dynamic_iq_correction_plan(dynamic_iq_correction_plan(candidate)) };
        unsafe { apply_dynamic_iq_capture_control(control.hardware) };
    }
    let measurement = if execution.correlate_samples {
        dynamic_iq_dft(
            samples,
            control.correlation,
            configuration.mode,
            configuration.first_phase_seed,
            configuration.second_phase_seed,
            configuration.third_phase_seed,
            configuration.sample_width_shift,
        )
    } else {
        DynamicIqDftResult::default()
    };
    DynamicIqHardwareStageResult {
        measurement,
        capture_ready,
    }
}

/// Connect the pass/dispatcher state machine to the detached MMIO acquisition
/// path. Readiness timeouts are accumulated for diagnostics but retain vendor
/// behavior by allowing every pass to continue with the copied sample window.
///
/// # Safety
///
/// The dynamic-IQ band snapshot must already be active and all referenced
/// MAC/PHY registers must be accessible.
pub unsafe fn run_dynamic_iq_hardware_search(
    initial: [i32; 4],
    seed: [i32; 4],
    common_step: i32,
    first_step: i32,
    second_step: i32,
    requested_passes: u8,
    configuration_flags: u32,
    control_configuration: u32,
    dft_configuration: DynamicIqDftConfiguration,
    samples: &mut [u32; 64],
) -> DynamicIqHardwareSearchResult {
    let mut all_captures_ready = true;
    let search = run_dynamic_iq_averaged_search(
        initial,
        seed,
        common_step,
        first_step,
        second_step,
        requested_passes,
        configuration_flags,
        |pass_index, stage, candidate, execution| {
            let control = dynamic_iq_stage_control(control_configuration, stage, pass_index == 0)?;
            // Vendor `rf_dft_correlate_samples` reads the current dispatcher
            // stage from calibration-state offset 0x94. It is not a fixed
            // caller-supplied DFT mode.
            let mut stage_dft = dft_configuration;
            stage_dft.mode = stage;
            let result = unsafe {
                execute_dynamic_iq_hardware_stage(candidate, execution, control, stage_dft, samples)
            };
            all_captures_ready &= result.capture_ready;
            Some(result.measurement)
        },
    );
    DynamicIqHardwareSearchResult {
        search,
        all_captures_ready,
    }
}

/// Publish the averaged candidate and perform the vendor's final stage-one
/// verification capture.
///
/// # Safety
///
/// The dynamic-IQ band snapshot must remain active and the calibration engine
/// must still own the ADC path.
pub unsafe fn execute_dynamic_iq_hardware_verification(
    candidate: [i32; 4],
    control_configuration: u32,
    dft_configuration: DynamicIqDftConfiguration,
    samples: &mut [u32; 64],
) -> DynamicIqHardwareVerificationResult {
    let publication = dynamic_iq_correction_plan(candidate);
    unsafe { apply_dynamic_iq_correction_plan(publication) };
    unsafe { replicate_dynamic_iq_correction_banks(publication) };
    let control = dynamic_iq_verification_control(control_configuration);
    unsafe { apply_dynamic_iq_capture_control(control) };
    let capture_ready =
        unsafe { capture_dynamic_iq_samples(samples, dft_configuration.capture_polls) };
    // The vendor verification path stores stage 1 before its final capture.
    let measurement = dynamic_iq_dft(
        samples,
        control,
        1,
        dft_configuration.first_phase_seed,
        dft_configuration.second_phase_seed,
        dft_configuration.third_phase_seed,
        dft_configuration.sample_width_shift,
    );
    DynamicIqHardwareVerificationResult {
        metrics: dynamic_iq_verification_metrics(measurement),
        capture_ready,
    }
}

/// Complete detached hardware envelope for annotated
/// `rf_apply_channel_settings` (`0x1899c`). Every normal return restores the
/// saved band registers. Failed searches force restoration of both original IQ
/// words; successful verification maps each rejection flag to its matching
/// conditional restore before applying the optional vendor publication.
///
/// # Safety
///
/// The caller must provide exclusive ownership of the calibration engine and
/// initialized live synth inputs for the active hardware profile.
pub unsafe fn run_dynamic_iq_hardware_calibration(
    configuration: DynamicIqHardwareCalibrationConfiguration,
    samples: &mut [u32; 64],
) -> DynamicIqHardwareCalibrationResult {
    let Some(synth_register) =
        (unsafe { prepare_dynamic_iq_synth_register(configuration.synth_frequency) })
    else {
        return DynamicIqHardwareCalibrationResult::default();
    };
    let mut snapshot = unsafe {
        begin_dynamic_iq_band_registers(
            configuration.alternate_profile,
            configuration.table_value,
            synth_register,
            configuration.calibration_command,
        )
    };
    let first_saved = unpack_dynamic_iq_pair(snapshot.abb8068);
    let second_saved = unpack_dynamic_iq_pair(snapshot.abb80a8);
    let saved_values = [first_saved.0, first_saved.1, second_saved.0, second_saved.1];
    let hardware_search = unsafe {
        run_dynamic_iq_hardware_search(
            saved_values,
            configuration.seed,
            configuration.common_step,
            configuration.first_step,
            configuration.second_step,
            configuration.requested_passes,
            configuration.configuration_flags,
            configuration.control_configuration,
            configuration.dft,
            samples,
        )
    };
    let mut result = DynamicIqHardwareCalibrationResult {
        profile_shortcut: false,
        synth_prepared: true,
        finalization: None,
        all_search_captures_ready: hardware_search.all_captures_ready,
        verification_capture_ready: false,
    };
    if let Some(search) = hardware_search.search {
        let verification = unsafe {
            execute_dynamic_iq_hardware_verification(
                search.final_candidate,
                configuration.control_configuration,
                configuration.dft,
                samples,
            )
        };
        let finalization = finalize_dynamic_iq_search(search, saved_values, verification.metrics);
        snapshot.restore_abb8068 = finalization.verification.first_pair_rejected;
        snapshot.restore_abb80a8 = finalization.verification.second_pair_rejected;
        unsafe { publish_dynamic_iq_final_state(configuration.alternate_profile, finalization) };
        result.finalization = Some(finalization);
        result.verification_capture_ready = verification.capture_ready;
    } else {
        snapshot.restore_abb8068 = true;
        snapshot.restore_abb80a8 = true;
    }
    unsafe { restore_dynamic_iq_band_registers(&snapshot) };
    result
}

/// Exact outer profile gate from annotated `rf_apply_channel_settings`. A
/// nonzero profile byte skips acquisition and marks the alternate-profile state
/// valid; profile zero executes the primary path with its fixed final seed.
///
/// # Safety
///
/// The vendor DTCM state and calibration hardware must be initialized.
pub unsafe fn run_vendor_dynamic_iq_hardware_calibration(
    mut configuration: DynamicIqHardwareCalibrationConfiguration,
    samples: &mut [u32; 64],
) -> DynamicIqHardwareCalibrationResult {
    let profile = unsafe { (crate::dtcm::phy_profile().get() as *const u8).read_volatile() };
    if profile != 0 {
        unsafe { write_u8(crate::dtcm::phy_profile1_ready().get(), 1) };
        return DynamicIqHardwareCalibrationResult {
            profile_shortcut: true,
            ..DynamicIqHardwareCalibrationResult::default()
        };
    }
    configuration.alternate_profile = false;
    configuration.seed = [-2, 5, -1, -1];
    unsafe { run_dynamic_iq_hardware_calibration(configuration, samples) }
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
fn build_iq_calibration_series_in<'a>(
    samples: &[IqCalibrationSample; 12],
    initial_shift_state: u32,
    output: &'a mut MaybeUninit<IqCalibrationSeries>,
) -> &'a mut IqCalibrationSeries {
    let series = output.as_mut_ptr();
    let iterations = unsafe {
        core::ptr::addr_of_mut!((*series).iterations).cast::<IqCalibrationIteration>()
    };
    let mut final_shift_state = initial_shift_state;
    for index in 0..12 {
        let sample = samples[index];
        let coefficient = primary_iq_calibration(sample);
        // Annotated `rf_compute_iq_gain_corr` reloads the same profile shift
        // word for every gain; it does not feed one gain's result into the next.
        let publication = iq_calibration_publication(
            IQ_CALIBRATION_GAIN_INDICES[index],
            coefficient,
            initial_shift_state,
        );
        if let Some(publication) = publication {
            final_shift_state = publication.next_shift_state;
        }
        unsafe {
            iterations.add(index).write(IqCalibrationIteration {
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
            });
        }
    }
    unsafe {
        core::ptr::addr_of_mut!((*series).final_shift_state).write(final_shift_state);
        output.assume_init_mut()
    }
}

pub fn build_iq_calibration_series(
    samples: &[IqCalibrationSample; 12],
    initial_shift_state: u32,
) -> IqCalibrationSeries {
    let mut output = MaybeUninit::uninit();
    *build_iq_calibration_series_in(samples, initial_shift_state, &mut output)
}

pub unsafe fn apply_iq_calibration_primary(series: &IqCalibrationSeries) {
    unsafe {
        for iteration in &series.iterations {
            let address = 0x0abb_8118_u32.wrapping_add(iteration.gain_index.wrapping_mul(4));
            write_u16(
                address as usize,
                pack_iq_signed8_pair(iteration.coefficient),
            );
        }
    }
}

pub unsafe fn apply_iq_calibration_normalized(series: &IqCalibrationSeries) {
    unsafe {
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

/// Detached publication half of `0x17ac8 -> 0x179ea`.
pub unsafe fn apply_iq_calibration_series(series: &IqCalibrationSeries) {
    unsafe {
        apply_iq_calibration_primary(series);
        apply_iq_calibration_normalized(series);
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

unsafe fn finish_iq_calibration_hardware(
    snapshot: IqCalibrationRegisterSnapshot,
    selected_mode: u8,
) {
    unsafe {
        clear_calibration_sample_settings();
        end_iq_calibration_path(snapshot);
        configure_calibration_mode(selected_mode);
        set_calibration_gain(-1);
        set_calibration_engine_enabled(false);
    }
}

/// Complete acquisition/publication envelope of annotated `rf_calibrate_iq_dc`
/// (`0x17c74`, target `0x17c20`). All timeout exits restore the path, band
/// selector, test tone and calibration-engine gate.
///
/// # Safety
///
/// The caller must exclusively own the RF calibration engine and MAC/PHY
/// register banks.
#[inline(never)]
unsafe fn run_iq_calibration_core(
    include_secondary: bool,
    max_polls: u32,
) -> Result<
    (IqCalibrationReferences, Option<IqCalibrationCoefficient>),
    IqCalibrationHardwareError,
> {
    const EMPTY_SAMPLE: IqCalibrationSample = IqCalibrationSample {
        baseline_i: 0,
        baseline_q: 0,
        target_i: 0,
        target_q: 0,
    };

    unsafe { set_calibration_engine_enabled(true) };
    let selected_mode = unsafe { (0x0abb_80f0 as *const u32).read_volatile() as u8 & 3 };
    let profile_base = crate::dtcm::PHY_PROFILE_STATE.get();
    if unsafe { (profile_base as *const u8).add(0x10).read_volatile() } == 0 {
        let shift_state = unsafe { (0x0abb_8680 as *const u32).read_volatile() };
        unsafe { write_u32(profile_base + 0x2c, shift_state) };
    }
    let initial_shift_state = unsafe { ((profile_base + 0x2c) as *const u32).read_volatile() };
    unsafe { configure_calibration_mode(0) };
    let snapshot = unsafe { begin_iq_calibration_path(0, 0) };
    let samples = unsafe { &mut *IQ_CALIBRATION_SCRATCH.samples.get() };
    samples.fill(EMPTY_SAMPLE);
    for (index, gain_index) in IQ_CALIBRATION_GAIN_INDICES.iter().copied().enumerate() {
        unsafe { set_calibration_gain(gain_index as i32) };
        let baseline = match unsafe { run_calibration_sample(0x11, 0x11, max_polls) } {
            Ok(value) => value,
            Err(_) => {
                unsafe { finish_iq_calibration_hardware(snapshot, selected_mode) };
                return Err(IqCalibrationHardwareError::BaselineTimeout { gain_index });
            }
        };
        let baseline_polls = last_sample_polls();
        let target = match unsafe { run_calibration_sample(1, 1, max_polls) } {
            Ok(value) => value,
            Err(_) => {
                unsafe { finish_iq_calibration_hardware(snapshot, selected_mode) };
                return Err(IqCalibrationHardwareError::TargetTimeout { gain_index });
            }
        };
        let target_polls = last_sample_polls();
        samples[index] = IqCalibrationSample {
            baseline_i: baseline.i,
            baseline_q: baseline.q,
            target_i: target.i,
            target_q: target.q,
        };
        if index == 11 {
            unsafe {
                *IQ_DIAGNOSTICS.0.get() = IqHardwareDiagnostics {
                    baseline_i: baseline.i,
                    target_i: target.i,
                    baseline_polls,
                    target_polls,
                };
            }
        }
    }
    let series_storage = unsafe { &mut *IQ_CALIBRATION_SCRATCH.series.get() };
    let series = build_iq_calibration_series_in(samples, initial_shift_state, series_storage);
    let final_iteration = &series.iterations[11];
    let references = IqCalibrationReferences {
        coefficient_i: final_iteration.coefficient.i,
        coefficient_q: final_iteration.coefficient.q,
        scale_i: final_iteration.scale_i,
        scale_q: final_iteration.scale_q,
    };
    unsafe {
        write_u32(crate::dtcm::phy_coefficient_i().get(), references.coefficient_i as u32);
        write_u32(crate::dtcm::phy_coefficient_q().get(), references.coefficient_q as u32);
        write_u32(crate::dtcm::phy_scale_i().get(), references.scale_i as u32);
        write_u32(crate::dtcm::phy_scale_q().get(), references.scale_q as u32);
        write_u8(profile_base + 0x10, 1);
    }

    let secondary = if include_secondary {
        let dac_i = rescale_signed(references.coefficient_i, 6, 8) as u8;
        let dac_q = rescale_signed(references.coefficient_q, 6, 8) as u8;
        unsafe { set_calibration_gain(0x20) };
        let baseline = match unsafe { run_calibration_sample_mode(dac_i, dac_q, 1, max_polls) } {
            Ok(value) => value,
            Err(_) => {
                unsafe { finish_iq_calibration_hardware(snapshot, selected_mode) };
                return Err(IqCalibrationHardwareError::SecondaryBaselineTimeout);
            }
        };
        let profile = unsafe { (crate::dtcm::phy_profile().get() as *const u8).read_volatile() };
        let secondary_snapshot = unsafe { begin_iq_calibration_path(1, profile) };
        let target = match unsafe { run_calibration_sample_mode(dac_i, dac_q, 0, max_polls) } {
            Ok(value) => value,
            Err(_) => {
                unsafe { end_iq_calibration_path(secondary_snapshot) };
                unsafe { finish_iq_calibration_hardware(snapshot, selected_mode) };
                return Err(IqCalibrationHardwareError::SecondaryTargetTimeout);
            }
        };
        unsafe { end_iq_calibration_path(secondary_snapshot) };
        let Some(correction) = secondary_iq_calibration(
            baseline.i,
            baseline.q,
            target.i,
            target.q,
            references.scale_i,
            references.scale_q,
        ) else {
            unsafe { finish_iq_calibration_hardware(snapshot, selected_mode) };
            return Err(IqCalibrationHardwareError::InvalidSecondaryScale);
        };
        Some(correction)
    } else {
        None
    };

    unsafe {
        apply_iq_calibration_primary(series);
        if let Some(correction) = secondary {
            write_u16(0x0abb_8198, pack_iq_signed8_pair(correction));
            write_u16(0x0abb_819c, 0);
        }
        apply_iq_calibration_normalized(series);
        finish_iq_calibration_hardware(snapshot, selected_mode);
    }
    Ok((references, secondary))
}

pub unsafe fn run_iq_calibration(
    include_secondary: bool,
    max_polls: u32,
) -> Result<IqCalibrationHardwareResult, IqCalibrationHardwareError> {
    let (references, secondary) =
        unsafe { run_iq_calibration_core(include_secondary, max_polls) }?;
    let series = unsafe { *(&*IQ_CALIBRATION_SCRATCH.series.get()).assume_init_ref() };
    Ok(IqCalibrationHardwareResult {
        series,
        references,
        secondary,
    })
}

pub unsafe fn run_primary_iq_calibration(
    max_polls: u32,
) -> Result<IqCalibrationHardwareResult, IqCalibrationHardwareError> {
    unsafe { run_iq_calibration(false, max_polls) }
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
unsafe fn detect_rf_silicon_variant() -> u8 {
    unsafe {
        let first = (0xfff1_7f90 as *const u32).read_volatile();
        let second = (0xfff1_7f94 as *const u32).read_volatile();
        let third = (0xfff1_7f98 as *const u32).read_volatile();
        if first == 0x302e_3530 && second == 0x3130_2e36 && third == 0x0000_3631 {
            1
        } else if first == 0x302e_3830 && second == 0x3130_2e32 && third == 0x0000_3036 {
            2
        } else {
            6
        }
    }
}

pub unsafe fn initialize_mac_software_state() {
    const STATE: usize = crate::dtcm::PHY_PROFILE_STATE.get();

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
        let startup_mode = (crate::dtcm::scheduler_startup_mode().get() as *const u16).read_volatile();
        if startup_mode & 2 == 0 {
            write_u8(STATE + 0x15, 1);
        }

        // Wake-context bytes read by `mac_reprogram_after_channel` and the
        // internally gated body of `mac_reinit_after_wake`. They sit below the
        // vendor BSS range cleared by `initialize_runtime_state`, so initialize
        // them explicitly before any cooperative channel transition.
        write_u8(crate::dtcm::MAC_WAKE_TRANSITION_PENDING.get(), 0);
        write_u8(crate::dtcm::MAC_WAKE_RESTORE_PENDING.get(), 0);

        write_u8(crate::dtcm::phy_measurement_control().get(), 0);
        write_u8(crate::dtcm::phy_silicon_variant().get(), detect_rf_silicon_variant());
        write_u8(crate::dtcm::phy_retained_state().get(), 0);
        write_u16(crate::dtcm::phy_retained_channel().get(), 100);
        write_u8(crate::dtcm::phy_calibration_state().get(), 1);
        write_u8(crate::dtcm::phy_calibration_aux().get(), 0);
        write_u8(crate::dtcm::phy_extended_settle().get(), 1);

        // Vendor 0x198f2 mode-zero state pointers.
        write_u32(crate::dtcm::phy_table_pointer().get(), crate::dtcm::SDD_CONFIGURATION_TABLES.get() as u32);
        write_u32(crate::dtcm::phy_calibration_table_a().get(), 0x0400_1088);
        write_u32(crate::dtcm::phy_calibration_table_b().get(), 0x0400_1098);
        write_u32(crate::dtcm::phy_state_scale().get(), u32::MAX);
        write_u8(crate::dtcm::phy_table_control().get(), 0);

        // Vendor 0x16ca4 derives these from remap window two at 0x04001ffc.
        let remap = (crate::dtcm::scheduler_remap_secondary().get() as *const u32).read_volatile();
        let (first, second) = derive_remap_timing(remap);
        write_u16(crate::dtcm::phy_denominator().get(), first);
        write_u16(crate::dtcm::phy_correction_offset().get(), second);
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
unsafe fn build_mode0_gain_tables() {
    let correction = unsafe { (crate::dtcm::sdd_agc_correction_unchecked(0).get() as *const i16).read_volatile() };
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
    }
}

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

    unsafe { build_mode0_gain_tables() };

    unsafe {
        let mac_control = (0x0ab8_0c00 as *mut u32).read_volatile();
        (0x0ab8_0c00 as *mut u32).write_volatile(mac_control | (1 << 11));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vendor_channel6_gain_input(rate: u8, power_tenths_dbm: i32) -> GainComputationInput {
        let rate_index = rate.min(10);
        let rate_base = if rate_index < 2 {
            304
        } else if rate_index < 10 {
            288
        } else {
            272
        };
        let rate_scale = match rate_index {
            0 | 1 => 380,
            2..=6 => 348,
            7 => 310,
            8 => 276,
            9 => 246,
            _ => 220,
        };
        GainComputationInput {
            rate: rate_index,
            first_limit: power_tenths_dbm * 16 / 10,
            second_limit: 480.min(rate_base + power_tenths_dbm * 16 / 10),
            rate_base,
            gain_coefficient_a: 23_000,
            gain_coefficient_b: 45,
            rssi_rate_scale: rate_scale,
            rssi_temperature_coefficient: -14,
            rssi_divisor_coefficient: -30,
            rssi_multiplier_coefficient: 100,
            rssi_denominator: 125,
            rssi_offset: 0,
            measured_a: 38_000,
            measured_b: 48_000,
            analog_enabled: 0x9600,
            analog_word_2c: 0,
            analog_word_30: 0,
            state_scale: -1,
        }
    }

    #[test]
    fn cooperative_transition_waits_across_timer_wrap() {
        let mut scheduler = ChannelTransitionScheduler::new();
        assert!(scheduler.is_idle());
        scheduler.state = 1;
        scheduler.arm_settle(0xffff_fff0);
        assert_eq!(scheduler.deadline, 0x68);
        assert_eq!(unsafe { scheduler.service(0x20) }, Ok(None));
        assert!(!scheduler.is_idle());
    }

    #[test]
    fn gain_entry_encoding_matches_vendor_oracle() {
        for (rate, power, gain, rssi, entry, companion) in [
            (0, 249, 1020, 653, 0x000f_f28d, 0x0ec1),
            (2, 233, 1020, 519, 0x000f_f207, 0x0e6e),
            (10, 217, 1020, 462, 0x000f_f1ce, 0x0e3d),
        ] {
            let result = compute_gain_entry(vendor_channel6_gain_input(rate, 200)).unwrap();
            assert_eq!(result.selected_power, power);
            assert_eq!(result.gain_code, gain);
            assert_eq!(result.rssi_value, rssi);
            assert_eq!(encoded_gain_words(gain, rssi), (entry, companion));
        }
    }

    #[test]
    fn gain_entries_match_vendor_across_power_levels() {
        for (power, rate, entry, companion) in [
            (-100, 0, 0x3113, 0x0d0a),
            (-100, 2, 0x3114, 0x0d0c),
            (-100, 10, 0x70b8, 0x0b92),
            (0, 0, 0x7122, 0x0d31),
            (0, 2, 0x7102, 0x0cd7),
            (0, 10, 0xf0c2, 0x0bcd),
            (100, 0, 0x000f_f158, 0x0da2),
            (100, 2, 0x000f_f132, 0x0d56),
            (100, 10, 0x000f_f132, 0x0d56),
        ] {
            let result = compute_gain_entry(vendor_channel6_gain_input(rate, power)).unwrap();
            assert_eq!(
                encoded_gain_words(result.gain_code, result.rssi_value),
                (entry, companion),
                "power={power}, rate={rate}",
            );
        }
    }

    #[test]
    fn gain_index_uses_vendor_mapping_boundaries() {
        assert_eq!(gain_index_from_value(0), 0x1_0000);
        assert_eq!(gain_code(4), 4);
        assert_eq!(gain_code(32), 1020);
    }

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
    fn native_channel_pll_cache_has_only_the_translated_tuple() {
        assert_eq!(core::mem::size_of::<ChannelPllCache>(), 12);
        assert_eq!(core::mem::offset_of!(ChannelPllCache, integer), 0);
        assert_eq!(core::mem::offset_of!(ChannelPllCache, fractional), 4);
        assert_eq!(core::mem::offset_of!(ChannelPllCache, channel), 8);
    }

    #[test]
    fn native_channel_power_limits_match_the_vendor_pair() {
        assert_eq!(core::mem::size_of::<ChannelPowerLimits>(), 4);
        assert_eq!(core::mem::offset_of!(ChannelPowerLimits, low_rate), 0);
        assert_eq!(core::mem::offset_of!(ChannelPowerLimits, high_rate), 2);
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
        assert_eq!(
            dynamic_iq_stage_execution(0, true),
            Some(DynamicIqStageExecution {
                capture_samples: false,
                program_candidate: true,
                correlate_samples: false,
                initial_control_before_program: true,
                initial_control_before_correlate: false,
            })
        );
        assert_eq!(
            dynamic_iq_stage_execution(6, false),
            Some(DynamicIqStageExecution {
                capture_samples: true,
                program_candidate: false,
                correlate_samples: true,
                initial_control_before_program: false,
                initial_control_before_correlate: false,
            })
        );
        assert_eq!(
            dynamic_iq_stage_execution(7, false),
            Some(DynamicIqStageExecution {
                capture_samples: false,
                program_candidate: true,
                correlate_samples: false,
                initial_control_before_program: false,
                initial_control_before_correlate: false,
            })
        );
        assert_eq!(dynamic_iq_stage_execution(13, false), None);
        assert_eq!(
            dynamic_iq_stage_control(0x1234_5678, 0, true),
            Some(DynamicIqStageControl {
                hardware: 0xffff_16ff,
                correlation: 0xffff_16ff,
            })
        );
        assert_eq!(
            dynamic_iq_stage_control(0x1234_5678, 1, true),
            Some(DynamicIqStageControl {
                hardware: 0x003f_1624,
                correlation: 0xffff_16ff,
            })
        );
        assert_eq!(
            dynamic_iq_stage_control(0x1234_5678, 7, false),
            Some(DynamicIqStageControl {
                hardware: 0x007f_1648,
                correlation: 0x007f_1648,
            })
        );
        assert_eq!(dynamic_iq_stage_control(0, 13, false), None);
        assert_eq!(dynamic_iq_verification_control(0x1234_5678), 0xffff_16ff);

        let mut pass_search = DynamicIqSearchState::new([10, 20, 30, 40], 2, 3, 4);
        let mut executed_stages = 0_u8;
        assert!(run_dynamic_iq_search_pass(
            &mut pass_search,
            true,
            |stage, _candidate, execution| {
                assert_eq!(stage, executed_stages);
                assert_eq!(execution, dynamic_iq_stage_execution(stage, true).unwrap());
                executed_stages += 1;
                Some(measurement(
                    i16::from(stage) + 1,
                    i16::from(stage) + 2,
                    i16::from(stage) + 3,
                ))
            }
        ));
        assert_eq!(executed_stages, 13);

        let mut averaged_executions = 0;
        let averaged = run_dynamic_iq_averaged_search(
            [1, 2, 3, 4],
            [10, 20, 30, 40],
            0,
            0,
            0,
            3,
            0,
            |_pass, _stage, _candidate, _execution| {
                averaged_executions += 1;
                Some(DynamicIqDftResult::default())
            },
        )
        .unwrap_or_default();
        assert_eq!(averaged_executions, 39);
        assert_eq!(averaged.completed_passes, 3);
        assert_eq!(averaged.accepted_passes, 2);
        assert_eq!(averaged.final_candidate, [11, 22, 33, 44]);
        assert_eq!(averaged.baseline, DynamicIqVerificationMetrics::default());

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

        assert_eq!(dynamic_iq_search_pass_count(3, 0), 3);
        assert_eq!(dynamic_iq_search_pass_count(3, 2), 6);
        assert_eq!(
            unpack_dynamic_iq_pair(dynamic_iq_correction_plan([-4, -11, -5, 1]).first_value),
            (-4, -11)
        );
        assert_eq!(
            dynamic_iq_calibration_start_words(7),
            DynamicIqCalibrationCommandWords {
                abb805c: 0x12,
                abb8060: 0x8800_0007,
                abb8064: 0,
            }
        );
        assert_eq!(
            dynamic_iq_calibration_stop_words(),
            DynamicIqCalibrationCommandWords::default()
        );
        assert_eq!(dynamic_iq_synth_register_mode1(1, 3, 24_000), 0x0060_6aab);
        assert_eq!(dynamic_iq_synth_register_mode1(10, 7, 24_000), 0x00e4_2aab);
        assert_eq!(
            dynamic_iq_synth_register(1, 1, 3 << 21, 24_000),
            Some(0x0060_6aab)
        );
        assert_eq!(
            dynamic_iq_synth_register_mode0(1, 0x0012_3456, 24_000),
            Some(0x0000_8555)
        );
        assert_eq!(
            dynamic_iq_synth_register_mode0(64, 0x0065_4321, 24_000),
            Some(0x0071_5555)
        );
        assert_eq!(
            dynamic_iq_band_derived_values(false, 0x10, 0x0400_0000),
            DynamicIqBandDerivedValues {
                abc0020: 0x2f00_c000,
                abc0030: 0x3ef,
                abb801c: 0x40c0,
            }
        );
        assert_eq!(
            dynamic_iq_band_derived_values(true, 0x10, 0),
            DynamicIqBandDerivedValues {
                abc0020: 0x1f00_d000,
                abc0030: 0x3ef,
                abb801c: 0x4100,
            }
        );
        let mut average = DynamicIqAverage::default();
        average.retain_pass(0, [100, 100, 100, 100]);
        assert_eq!(average.accumulated, [0; 4]);
        average.retain_pass(1, [10, 20, 30, 40]);
        assert_eq!(
            average.candidate_for_pass(2, [1, 2, 3, 4]),
            [10, 20, 30, 40]
        );
        average.retain_pass(2, [20, 30, 40, 50]);
        assert_eq!(
            average.candidate_for_pass(3, [1, 2, 3, 4]),
            [15, 25, 35, 45]
        );
        assert_eq!(
            average.final_candidate(2, [-2, 5, -1, -1]),
            Some([13, 30, 34, 44])
        );
        assert_eq!(
            dynamic_iq_final_seed(false, dynamic_iq_initial_candidate(2)),
            [-2, 5, -1, -1]
        );
        assert_eq!(
            dynamic_iq_final_seed(true, dynamic_iq_initial_candidate(2)),
            [7, -7, -5, 1]
        );
        assert_eq!(
            verify_dynamic_iq_candidate(
                [1, 2, 3, 4],
                [10, 20, 30, 40],
                DynamicIqVerificationMetrics {
                    reference: 0,
                    second: 100,
                    third: 100,
                },
                DynamicIqVerificationMetrics {
                    reference: 50,
                    second: 60,
                    third: 10,
                },
            ),
            DynamicIqVerification {
                retained_values: [1, 2, 30, 40],
                first_pair_rejected: false,
                second_pair_rejected: true,
                first_quality_failed: true,
                second_quality_failed: true,
                vendor_publication_allowed: false,
            }
        );
        let asymmetric_verification = verify_dynamic_iq_candidate(
            [1, 2, 3, 4],
            [10, 20, 30, 40],
            DynamicIqVerificationMetrics {
                reference: 0,
                second: 100,
                third: 100,
            },
            DynamicIqVerificationMetrics {
                reference: 100,
                second: 10,
                third: 120,
            },
        );
        assert_eq!(asymmetric_verification.retained_values, [10, 20, 3, 4]);
        assert!(asymmetric_verification.first_pair_rejected);
        assert!(!asymmetric_verification.second_pair_rejected);
        assert!(asymmetric_verification.vendor_publication_allowed);
        let finalized = finalize_dynamic_iq_search(
            DynamicIqAveragedSearch {
                baseline: DynamicIqVerificationMetrics {
                    reference: 0,
                    second: 100,
                    third: 100,
                },
                final_candidate: [1, 2, 3, 4],
                completed_passes: 3,
                accepted_passes: 2,
            },
            [10, 20, 30, 40],
            DynamicIqVerificationMetrics {
                reference: 100,
                second: 10,
                third: 120,
            },
        );
        assert_eq!(finalized.verification, asymmetric_verification);
        assert_eq!(
            finalized.publication,
            Some(dynamic_iq_correction_plan([1, 2, 3, 4]))
        );

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
        let series = build_iq_calibration_series(&[repeated_sample; 12], 0x0025_4310);
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
            series.iterations[0]
                .publication
                .map(|publication| publication.next_shift_state),
            series.iterations[11]
                .publication
                .map(|publication| publication.next_shift_state)
        );
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
