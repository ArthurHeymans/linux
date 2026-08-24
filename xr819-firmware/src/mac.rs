//! Vendor MAC wake restoration and post-channel reprogramming.

use crate::{packet_ram, platform, radio};

const SHARED: usize = crate::dtcm::LOW_MAC_GLOBAL.get();
const WAKE: usize = crate::dtcm::MAC_WAKE_RUNTIME_STATE.get();

#[inline(always)]
fn packet_offset(address: usize) -> u32 {
    address as u32 & 0x007f_ffff
}

const RATE_ENCODING: [u8; 22] = [
    0, 0, 1, 1, 0, 0, 2, 3, 4, 5, 6, 7, 8, 9, 2, 4, 5, 6, 7, 8, 9, 10,
];
// Matching-container DTCM `0x04000138..0x0400014c`, consumed by vendor
// `tx_build_duration_desc()` through the rate map at `0x04001aec`.
const TX_DURATION_TIMING: [u16; 10] = [
    0x00c2, 0x00c2, 0x001c, 0x0000, 0x001e, 0x0021, 0x0002, 0x0004, 0x000b, 0x0016,
];

const RATE_ATTRIBUTE: [u8; 22] = [
    0, 1, 2, 3, 3, 3, 11, 15, 10, 14, 9, 13, 8, 12, 0, 1, 2, 3, 4, 5, 6, 7,
];
const PRIMARY_MCS_FALLBACK: [u8; 8] = [6, 8, 9, 10, 11, 12, 13, 13];
const SECONDARY_MCS_FALLBACK: [u8; 8] = [14, 15, 16, 17, 18, 19, 20, 21];
const CCK_DIVISORS: [u16; 4] = [2, 4, 11, 22];
const OFDM_DIVISORS: [u16; 16] = [
    24, 36, 48, 72, 96, 144, 192, 216, 26, 52, 78, 104, 156, 208, 234, 260,
];
const OFFSET_TABLE: [[u8; 4]; 6] = [
    [0x00, 0x60, 0x30, 0x70],
    [0x04, 0x64, 0x34, 0x74],
    [0x08, 0x68, 0x38, 0x78],
    [0, 0, 0, 0],
    [0x10, 0x20, 0x40, 0x50],
    [0x18, 0x28, 0x48, 0x58],
];
const RATE_PAIRS: [(u8, u8); 31] = [
    (1, 0),
    (1, 1),
    (1, 2),
    (1, 3),
    (0, 1),
    (0, 2),
    (0, 3),
    (2, 6),
    (2, 7),
    (2, 8),
    (2, 9),
    (2, 10),
    (2, 11),
    (2, 12),
    (2, 13),
    (5, 14),
    (5, 15),
    (5, 16),
    (5, 17),
    (5, 18),
    (5, 19),
    (5, 20),
    (5, 21),
    (4, 14),
    (4, 15),
    (4, 16),
    (4, 17),
    (4, 18),
    (4, 19),
    (4, 20),
    (4, 21),
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MacWakeError {
    PipeControllerTimeout,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MacStartupError {
    PacketDmaStopTimeout,
    PipeControllerTimeout,
}

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

const fn ceil_div(numerator: u32, denominator: u32) -> u32 {
    numerator.wrapping_add(denominator - 1) / denominator
}

pub const fn rate_phy_class(cfg: u16, rate: u8, stream: bool) -> u8 {
    if rate < 4 {
        if cfg & 0x100 != 0 || rate == 0 || (cfg & 0x400 != 0 && rate == 1) {
            1
        } else {
            0
        }
    } else if rate < 14 {
        2
    } else if stream {
        5
    } else {
        4
    }
}

pub fn base_airtime(cfg: u16, rate: u8, length: u16, stream: bool) -> u16 {
    let class = rate_phy_class(cfg, rate, stream);
    let value = if class < 2 {
        ceil_div(
            u32::from(length) * 16,
            u32::from(CCK_DIVISORS[(rate & 3) as usize]),
        ) + if class == 1 { 0xc0 } else { 0x60 }
    } else {
        let numerator = (u32::from(length) * 8 + 0x16) & 0xffff;
        let mut value = 4 * ceil_div(
            numerator,
            u32::from(OFDM_DIVISORS[((rate - 6) & 15) as usize]),
        );
        if cfg & 0x10 != 0 {
            value += 6;
        }
        value += match class {
            2 => 0x14,
            4 => 0x18,
            _ => 0x24,
        };
        if class == 2 && cfg & 0x40 != 0 {
            value *= 2;
        }
        value
    };
    value as u16
}

pub fn extended_airtime(cfg: u16, rate: u8, length: u16, stream: bool) -> u16 {
    let class = rate_phy_class(cfg, rate, stream);
    let mut value = u32::from(base_airtime(cfg, rate, length, stream));
    value += if cfg & 0x40 != 0 {
        0x20
    } else if cfg & 0x10 != 0 && class < 3 {
        0x0a
    } else {
        0x10
    };
    value as u16
}

pub fn ofdm_duration(rate: u8, length: u16) -> u16 {
    if rate < 6 {
        return 0;
    }
    let value = ceil_div(
        u32::from(length) * 8 + 0x16,
        u32::from(OFDM_DIVISORS[((rate - 6) & 15) as usize]),
    );
    ((value * 3 + 9) & 0xfff) as u16
}

fn fill_fallbacks(mut basic: u32) -> [u8; 22] {
    for rate in 0..4 {
        let bit = 1_u32 << rate;
        if basic & bit != 0 {
            break;
        }
        basic |= bit & 0x0f;
    }
    for rate in 6..11 {
        let bit = 1_u32 << rate;
        if basic & bit != 0 {
            break;
        }
        basic |= bit & 0x540;
    }
    let mut result = [0_u8; 22];
    let mut fallback = 0_u8;
    for rate in 0..14 {
        if 0x003f_ffcf & (1_u32 << rate) != 0 {
            if basic & (1_u32 << rate) != 0 {
                fallback = rate as u8;
            }
            result[rate] = fallback;
        }
    }
    fallback = 6;
    for rate in 14..22 {
        let candidate = PRIMARY_MCS_FALLBACK[rate - 14];
        if basic & (1_u32 << candidate) != 0 {
            fallback = candidate;
        }
        result[rate] = fallback;
    }
    result
}

fn secondary_fallbacks(basic: u32, initial: u8) -> [u8; 8] {
    let mut result = [initial; 8];
    let mut fallback = initial;
    for (index, candidate) in SECONDARY_MCS_FALLBACK.into_iter().enumerate() {
        if basic & (1_u32 << candidate) != 0 {
            fallback = candidate;
        }
        result[index] = fallback;
    }
    result
}

fn build_rate_entry(cfg: u16, hardware_class: u8, fallback: u8) -> [u32; 4] {
    let d14 = extended_airtime(cfg, fallback, 14, true);
    let d32 = extended_airtime(cfg, fallback, 32, true);
    let mut class = hardware_class;
    if hardware_class < 3 && (fallback == 0 || (fallback == 1 && cfg & 0x400 != 0)) {
        class = 1;
    } else if hardware_class >= 3 {
        class = rate_phy_class(cfg, fallback, true);
    }
    let duration14 = u32::from(ofdm_duration(fallback, 14));
    let duration32 = u32::from(ofdm_duration(fallback, 32));
    [
        u32::from(d14) | (u32::from(RATE_ATTRIBUTE[fallback as usize]) << 24),
        u32::from(d32)
            | (u32::from(RATE_ENCODING[fallback as usize]) << 24)
            | (u32::from(class) << 28),
        duration14 | (duration14 << 12),
        duration32 | (duration32 << 12),
    ]
}

unsafe fn program_rate_tables(vif: usize) {
    let cfg = unsafe { read_u16(SHARED + 2) };
    let basic = unsafe { read_u32(vif + 8) };
    let fallbacks = fill_fallbacks(basic);
    for (rate, fallback) in fallbacks.iter().copied().enumerate() {
        if 0x003f_ffcf & (1_u32 << rate) == 0 {
            continue;
        }
        unsafe {
            write_u8(vif + 0x24 + rate, fallback);
            write_u16(
                SHARED + 0x48 + rate * 2,
                extended_airtime(cfg, fallback, 14, true),
            );
            write_u16(
                SHARED + 0x74 + rate * 2,
                extended_airtime(cfg, fallback, 32, true),
            );
            write_u8(SHARED + 0x46c + rate, rate_phy_class(cfg, fallback, true));
        }
    }
    let secondary = secondary_fallbacks(basic, fallbacks[21]);
    for rate in 14..22 {
        unsafe { write_u8(SHARED + 0x47a + rate, secondary[rate - 14]) };
    }

    let column = if unsafe { read_u8(vif + 0x21) } == 0 {
        0
    } else {
        2
    };
    for (hardware_class, rate) in RATE_PAIRS {
        let fallback = fallbacks[rate as usize];
        let mut index = OFFSET_TABLE[hardware_class as usize][column]
            .wrapping_add(RATE_ATTRIBUTE[rate as usize]);
        if hardware_class == 2 {
            index = index.wrapping_sub(8);
        }
        let entry = build_rate_entry(cfg, hardware_class, fallback);
        for (word, value) in entry.into_iter().enumerate() {
            unsafe { write_u32(packet_ram::rate_entry(usize::from(index)) + word * 4, value) };
        }
        if hardware_class >= 4 {
            let secondary = unsafe { read_u8(SHARED + 0x47a + usize::from(rate)) };
            let secondary_index = OFFSET_TABLE[hardware_class as usize][1]
                .wrapping_add(RATE_ATTRIBUTE[rate as usize]);
            let entry = build_rate_entry(cfg, hardware_class, secondary);
            for (word, value) in entry.into_iter().enumerate() {
                unsafe {
                    write_u32(
                        packet_ram::rate_entry(usize::from(secondary_index)) + word * 4,
                        value,
                    )
                };
            }
        }
    }
}

unsafe fn program_ifs_timing() {
    let cfg = unsafe { read_u16(SHARED + 2) };
    let index = if cfg & 0x11 == 0x11 { 4 } else { 11 };
    let entry = unsafe { read_u32(packet_ram::rate_entry(index)) } & 0x00ff_ffff;
    let base = unsafe { read_u32(SHARED + 0x1c) };
    unsafe {
        write_u32(
            SHARED + 0x44,
            base.wrapping_mul(3).wrapping_add(entry).wrapping_mul(8),
        )
    };
}

/// Rebuild the selected active PAS rate tables after JOIN publishes
/// `PAS_BASE+0x470`. Vendor `mac_apply_channel_and_vif_config` activates the
/// PAS record before `pas_reprogram_all_vif_rate_tables`; the rebuilt JOIN
/// path performs channel transition first, so it must make this call after
/// VIF activation instead of silently skipping the new record.
///
/// # Safety
/// The selected PAS record and shared rate RAM must be exclusively owned.
pub unsafe fn program_active_vif_rate_tables(interface: u8) -> bool {
    if interface >= 3 {
        return false;
    }
    unsafe {
        program_rate_tables(
            crate::dtcm::pas_stride_view_unchecked(usize::from(interface))
                .activity_state()
                .get(),
        )
    };
    true
}

unsafe fn program_pipe_slot(pointer: usize, slot: u8) {
    unsafe {
        write_u32(
            packet_ram::response_pointer(usize::from(slot)),
            (pointer as u32) & 0xf6ff_ffff,
        )
    };
    let logical = if slot == 0x1b {
        30
    } else if slot == 0x1c {
        31
    } else {
        slot - 2
    };
    let register_bit = if logical == 30 { 31 } else { logical };
    for address in [
        crate::platform::mac_register(0x0a08),
        crate::platform::mac_register(0x0a0c),
    ] {
        let value = unsafe { read_u32(address) } | (1_u32 << register_bit);
        unsafe { write_u32(address, value) };
    }
    let value = unsafe { read_u32(crate::platform::mac_register(0x0a10)) }
        & !(1_u32 << logical)
        & 0x00ff_ffff;
    unsafe { write_u32(crate::platform::mac_register(0x0a10), value) };
}

unsafe fn build_control_frame(if_id: u8, pointer: usize, ack: bool) {
    let control = unsafe { read_u8(crate::dtcm::LOW_MAC_RESPONSE_CONTROL_BYTE.get()) };
    unsafe {
        write_u32(
            pointer,
            read_u32(SHARED + 0x30).wrapping_shl(16) | 0x2000_0000,
        );
        write_u16(pointer + 4, 0);
        write_u16(pointer + 6, 0);
        write_u32(pointer + 8, 0x5800_0000);
    }
    let words = [
        0x5900_0000 | (u32::from(control) << 13),
        0x5800_0003,
        0x5a00_000e,
        if ack { 0x3100_00d4 } else { 0x3100_00c4 },
        0x4700_0000,
        0x2080_800c + u32::from(if_id),
        if ack { 0x0700_2008 } else { 0x0700_2004 },
        0x0700_6828,
        0x0700_4600,
        0xf000_0000,
    ];
    for (index, value) in words.into_iter().enumerate() {
        unsafe { write_u32(pointer + 0x0c + index * 4, value) };
    }
    let slot = if ack { 0x15 + if_id } else { 9 + if_id };
    unsafe { program_pipe_slot(pointer, slot) };
    if ack
        && unsafe {
            read_u8(crate::dtcm::low_mac_response_enabled_unchecked(usize::from(if_id)).get())
        } != 0
    {
        unsafe { program_pipe_slot(pointer, 0x14) };
    }
}

/// Build and install the vendor immediate ACK/CTS and response descriptors.
/// Channel transition can skip `mac_reprogram_after_channel` when its retained
/// wake flags select the already-restored branch, so JOIN must explicitly
/// republish these slots after activating the STA record.
///
/// # Safety
/// Packet RAM and the MAC pipe-controller registers must be exclusively owned.
pub unsafe fn program_immediate_response_descriptors() {
    unsafe {
        build_control_frame(0, packet_ram::response_command(4), true);
        build_control_frame(1, packet_ram::response_command(5), true);
        build_control_frame(0, packet_ram::response_command(6), false);
        build_control_frame(1, packet_ram::response_command(7), false);
        install_response_descriptors();
    }
}

unsafe fn save_register_context() {
    unsafe {
        write_u32(crate::platform::mac_register(0x1404), read_u32(crate::dtcm::runtime_register_context_unchecked(0).get()));
        write_u32(crate::platform::mac_register(0x1408), read_u32(crate::dtcm::runtime_register_context_unchecked(1).get()));
        write_u32(
            crate::platform::mac_register(0x140c),
            u32::from(read_u16(crate::dtcm::saved_register_context().get())),
        );
        write_u32(crate::platform::mac_register(0x1410), read_u32(crate::dtcm::runtime_register_context_unchecked(2).get()));
        write_u32(crate::platform::mac_register(0x1400), read_u32(crate::dtcm::runtime_register_context_unchecked(3).get()));
    }
}

/// Full vendor `mac_reprogram_after_channel` for the currently represented VIF
/// and packet-RAM state.
/// Build the temporary mode-zero channel/VIF image used by
/// `syn_scan_program_channel()` before `mac_apply_channel_and_vif_config()`.
pub unsafe fn prepare_scan_context(channel: u16) {
    unsafe {
        let rate_config = 0x0117_u16;
        if crate::vif::set_scan_context(channel).is_err() {
            crate::halt_always!();
        }

        write_u32(crate::platform::mac_register(0x0200), 0);
        write_u8(SHARED, 0);
        write_u16(SHARED + 2, rate_config);
        write_u8(SHARED + 5, 0);

        // `syn_scan_program_channel` publishes the temporary VIF index and
        // active-record mask before entering `mac_apply_channel_and_vif_config`.
        if crate::vif::publish_synthetic_scan_record().is_err() {
            crate::halt_always!();
        }

        // Synthetic scan record 2 (`0x04003678 + 2 * 0x98`) is marked as
        // scan-active by `phy_set_band_reg`/`mac_apply_channel_and_vif_config`.
        let scan_pas = crate::dtcm::pas_stride_view_unchecked(2);
        write_u8(scan_pas.activity_state().get(), 2);
        write_u8(scan_pas.mode_byte().get(), 0x0f);
        write_u32(scan_pas.basic_rate_bits().get(), 1);
        // Base 0x07e3b85c plus the non-matching temporary-record mask
        // 0x00100502 from the vendor scan path.
        write_u32(crate::dtcm::MAC_WAKE_MODE.get(), 0x07f3_bd5e);
    }
}

/// Temporary station-mode register image installed by the vendor synthetic
/// scan path after `phy_do_channel_switch()` returns.
pub unsafe fn program_before_scan_channel(channel: u16) {
    unsafe {
        write_u32(
            crate::platform::mac_register(0x0800),
            packet_offset(packet_ram::rate_ram().start),
        );
        let scan_vif = crate::dtcm::pas_stride_view_unchecked(2);
        program_slot_timings(
            read_u16(SHARED + 2),
            read_u32(scan_vif.slot_timing_word().get()),
        );

        let mut first_active = false;
        let mut second_active = false;
        for index in 0..3 {
            let vif = crate::dtcm::pas_stride_view_unchecked(index);
            if read_u8(vif.activity_state().get()) != 2 {
                continue;
            }
            write_u8(vif.rate_table_column().get(), 0);
            if first_active && !second_active {
                second_active = true;
                write_u8(vif.rate_table_column().get(), 1);
                write_u32(
                    crate::platform::mac_register(0x0270),
                    if read_u8(vif.path_selector_byte().get()) == 0 {
                        0x0100_0000
                    } else {
                        1 << 26
                    },
                );
            } else {
                first_active = true;
            }
            program_rate_tables(vif.activity_state().get());
        }
        write_u32(
            crate::platform::mac_register(0x0314),
            if second_active { 0x0100_0000 } else { 0 },
        );
        program_ifs_timing();
        write_u16(crate::dtcm::LOW_MAC_CURRENT_CHANNEL.get(), channel);
    }
}

pub unsafe fn program_scan_station_mode() {
    unsafe {
        // Final publication at 0xfa34 in `mac_apply_channel_and_vif_config`.
        write_u16(SHARED + 8, 0x1000);
        write_u32(crate::dtcm::MAC_BEACON_SELECTOR.get(), 0x0018_0180);
        write_u32(crate::platform::mac_register(0x0a04), 0x0018_0180);
        write_u32(crate::platform::mac_register(0x0a1c), 0x827b_ffdf);
        write_u32(crate::platform::mac_register(0x0204), 0x0019_8000);
        write_u32(crate::platform::mac_register(0x0200), read_u32(crate::dtcm::MAC_WAKE_MODE.get()));
        write_u32(crate::platform::mac_register(0x0310), 0x7800_0000);
    }
}

/// Exact STA-mode register tail from vendor `mac_program_mode_sta` (`0x10b80`).
///
/// # Safety
/// MAC mode registers must be exclusively owned during JOIN activation.
#[cfg(target_arch = "arm")]
unsafe fn active_station_mode_word() -> u32 {
    unsafe {
        let reference_path = read_u8(crate::dtcm::low_mac_own_mac_byte_unchecked(0, 5).get());
        (0..3).fold(0x07e3_b85c_u32, |mode, index| {
            let record = crate::dtcm::pas_stride_view_unchecked(index);
            if read_u8(record.activity_state().get()) != 2 {
                return mode;
            }
            let path_mask = if read_u8(record.own_mac_byte_unchecked(5).get()) == reference_path {
                0x0008_0281
            } else {
                0x0010_0502
            };
            mode | path_mask
                | if read_u8(record.mode_byte().get()) == 2 {
                    0x4000
                } else {
                    0
                }
        })
    }
}

#[cfg(target_arch = "arm")]
pub unsafe fn program_joined_station_mode() {
    unsafe {
        let mode = active_station_mode_word();
        write_u32(crate::dtcm::MAC_WAKE_MODE.get(), mode);
        write_u32(crate::dtcm::MAC_BEACON_SELECTOR.get(), 0x0018_0180);
        write_u32(crate::platform::mac_register(0x0a04), 0x0018_0180);
        write_u32(crate::platform::mac_register(0x0a1c), 0x827b_ffdf);
        write_u32(crate::platform::mac_register(0x0204), 0x0019_8000);
        write_u32(crate::platform::mac_register(0x0200), mode);
        write_u32(crate::platform::mac_register(0x0310), 0x7800_0000);
    }
}

/// Exact mode-1 BSSID publication from vendor `mac_program_bssid` (`0x10b1a`).
///
/// # Safety
/// Address-match registers must be exclusively owned.
#[cfg(target_arch = "arm")]
pub unsafe fn program_joined_bssid(bssid: [u8; 6]) {
    unsafe {
        write_u32(
            crate::platform::mac_register(0x003c),
            u32::from_le_bytes([bssid[0], bssid[1], bssid[2], bssid[3]]),
        );
        write_u32(
            crate::platform::mac_register(0x0040),
            u32::from(u16::from_le_bytes([bssid[4], bssid[5]])),
        );
        write_u32(crate::platform::mac_register(0x0044), 0x101);
        write_u16(crate::dtcm::mac_bssid_mode().get(), 3);
    }
}

pub unsafe fn reprogram_after_channel() {
    unsafe {
        write_u8(WAKE + 0x1d, 0);
        write_u32(
            crate::platform::mac_register(0x0800),
            packet_offset(packet_ram::rate_ram().start),
        );
        for index in 0..3 {
            let vif = crate::dtcm::pas_stride_view_unchecked(index);
            if read_u8(vif.activity_state().get()) == 2 {
                program_rate_tables(vif.activity_state().get());
            }
        }
        program_ifs_timing();
        program_immediate_response_descriptors();
        write_u32(crate::platform::mac_register(0x0200), read_u32(WAKE + 0x24));
        save_register_context();
    }
}

unsafe fn program_mac_address(source: usize, base: usize, mode: u32) {
    unsafe {
        write_u32(base, read_u32(source));
        write_u32(base + 4, u32::from(read_u16(source + 4)));
        write_u32(base + 8, mode);
    }
}

unsafe fn rebuild_pipe_state() {
    // `prepare_packet_dma` already programmed these once, before MAC core
    // enable, which is where vendor's `mac_hw_init_pipes` does it and the only
    // time vendor does it (`annotated-main.c:18541-18587`). Repeating it here
    // re-writes ring+0x14 (GO) while the MAC is live, and cycles 0x09c00e8c
    // 0xbf -> 0 -> 0xbf underneath a running command-fetch engine. Final
    // register values are identical, so this only matters if any of it
    // disturbs fetch state.
    unsafe {
        write_u32(crate::platform::mac_register(0x0e8c), 0);
        write_u32(crate::platform::mac_register(0x0e60), 2);
        write_u32(crate::platform::mac_register(0x0e80), 0x100);
        write_u32(crate::platform::mac_register(0x0e88), 0xff);
    }
    let descriptors = [
        crate::platform::tx_ring_register_offset(0x0000),
        crate::platform::tx_ring_register_offset(0x0080),
        crate::platform::tx_ring_register_offset(0x0100),
        crate::platform::tx_ring_register_offset(0x0180),
    ];
    for (pipe, base) in descriptors.into_iter().enumerate() {
        unsafe {
            write_u32(base + 0x0c, packet_offset(packet_ram::tx_command(pipe, 0)));
            write_u32(base + 0x10, packet_ram::TX_COMMAND_SIZE as u32);
            write_u32(base + 0x14, 1);
        }
    }
    for index in 0..4 {
        let record = SHARED + index * 0x6c;
        let descriptor = descriptors[index];
        let slot = ((unsafe { read_u32(descriptor + 0x20) } & 0x07ff_ffff) >> 24) as u8;
        unsafe {
            write_u8(record + 0xa0, slot);
            write_u8(record + 0xa1, slot);
            write_u8(record + 0xa2, slot);
            write_u8(record + 0xa3, 0);
            write_u32(record + 0xa8, descriptor as u32);
        }
        for entry in 0..4 {
            unsafe {
                write_u32(
                    record + 0xc0 + entry * 0x18,
                    packet_ram::tx_command(index, entry) as u32,
                )
            };
        }
    }
    unsafe { write_u32(crate::platform::mac_register(0x0e8c), 0xbf) };
    unsafe {
        let list = packet_ram::automatic_response_list().start;
        write_u32(list, 0x4e14_0000);
        for index in 0..33 {
            write_u32(list + 4 + index * 4, 0x2200_0000 | packet_offset(list));
        }
        write_u32(list + 4 + 33 * 4, 0xf000_0000);
    }
}

/// Builds the vendor pipe-record pointers needed before any TX descriptor can
/// be prepared for hardware ownership.
///
/// # Safety
/// Packet DMA and the four hardware pipe blocks must already be initialized.
pub unsafe fn initialize_tx_pipe_state() {
    unsafe {
        rebuild_pipe_state();
        // Vendor packet-controller startup publishes both global retry timing
        // terms. The second survives retained startup on this target, while
        // the first otherwise remains zero and shortens every retry by 18 us.
        write_u32(SHARED + 0x1c, 9);
        write_u32(SHARED + 0x20, 10);
        // Hardware ring cursor -> software slot translation used by the
        // vendor's nontrivial retry-retirement branch.
        write_u32(crate::dtcm::QUEUE_PIPE_MAPPINGS.get(), 0x0201_0003);
    }
    // `txp_submit_to_pipe` emits a 0x20800000 descriptor command sourcing
    // one byte from this per-interface packet-SRAM metadata vector. Hardware
    // ORs that byte into the high frame-control octet. Vendor startup clears
    // the vector; leaving retained packet data here turned 0x00b0
    // authentication frames into protected/ToDS/more-data 0x61b0 frames.
    unsafe { write_u32(packet_ram::interface_metadata(), 0) };
    for (index, value) in TX_DURATION_TIMING.into_iter().enumerate() {
        unsafe { write_u16(crate::dtcm::TX_DURATION_TIMING_TABLE.get() + index * 2, value) };
    }
    for pipe in 0..4 {
        unsafe {
            write_u32(
                crate::dtcm::DURATION_QUANTUM_POINTERS.get() + pipe * 4,
                crate::platform::mac_register(0x0e70) as u32 + pipe as u32 * 4,
            )
        };
    }
    for (base, values) in [
        (crate::dtcm::RATE_ENCODING_TABLE.get(), &RATE_ENCODING[..]),
        (crate::dtcm::RATE_ATTRIBUTE_TABLE.get(), &RATE_ATTRIBUTE[..]),
        (crate::dtcm::QUEUE_TO_ACCESS_CATEGORY.get(), &[1_u8, 0, 2, 3][..]),
        (crate::dtcm::ACCESS_CATEGORY_TO_QUEUE.get(), &[1_u8, 0, 2, 3][..]),
    ] {
        for (offset, value) in values.iter().copied().enumerate() {
            unsafe { write_u8(base + offset, value) };
        }
    }
}

/// Rebuilds the vendor 30-entry host WSM TX context free list.
///
/// This pool is distinct from the three internal management/template contexts
/// at `0x04009084`. The current cooperative publisher does not allocate from it
/// yet, but startup must not leave its retained ownership state stale while the
/// class-0 host scheduler is translated.
pub unsafe fn initialize_wsm_tx_context_pool() {
    unsafe { crate::vendor_host_tx::initialize_host_pool() };
}

/// Hardware-owning prefix of vendor `mac_radio_stop` for the current
/// no-active-VIF scan branch. TX ownership must already be proven empty.
#[cfg(target_arch = "arm")]
pub unsafe fn begin_unjoined_scan_radio_stop() {
    unsafe {
        write_u32(SHARED + 0x18, 0);
        let previous = crate::tx::disable_irq_fiq_save();
        write_u16(SHARED + 8, 0);
        write_u16(crate::dtcm::RADIO_STOP_WORD_02.get(), 0);
        write_u32(crate::dtcm::MAC_BEACON_CONTROL.get(), 0);
        write_u8(crate::dtcm::MAC_BEACON_MODE.get(), 4);
        write_u32(crate::platform::mac_register(0x0a28), 0);
        write_u32(crate::platform::mac_register(0x0a00), 0x1030_0000);
        write_u32(crate::platform::mac_register(0x0a04), read_u32(crate::dtcm::MAC_BEACON_SELECTOR.get()));
        for index in 0..32 {
            write_u32(
                packet_ram::response_pointer(index),
                packet_offset(packet_ram::response_command(8)),
            );
        }
        crate::tx::restore_irq_fiq_saved(previous);
    }
}

/// Final state publication from vendor `mac_radio_stop`, after packet RX,
/// PHY command 7, and software RX draining have completed.
#[cfg(target_arch = "arm")]
pub unsafe fn finish_unjoined_scan_radio_stop() {
    unsafe {
        write_u16(crate::dtcm::LOW_MAC_CURRENT_CHANNEL.get(), 0);
        write_u8(crate::dtcm::LOW_MAC_RECEIVE_STATE_BYTE.get(), 0);
        write_u8(crate::dtcm::MAC_WAKE_PHY_STATE.get(), 2);
        write_u8(SHARED + 0x0a, 0);
        write_u8(SHARED + 0x0b, 0);
        write_u8(crate::dtcm::MAC_RADIO_STOP_STATE.get(), 0);
        for vif in 0..3 {
            write_u8(
                crate::dtcm::pas_stride_view_unchecked(vif)
                    .activity_state()
                    .get(),
                1,
            );
        }
    }
    unsafe { crate::tx::clear_scheduler_bits(1 << 18) };
}

unsafe fn build_tbtt(pointer: usize) {
    let n = unsafe { read_u32(SHARED + 0x30) }
        .wrapping_add(unsafe { read_u32(SHARED + 0x1c) }.wrapping_mul(8))
        & 0x1fff;
    unsafe {
        write_u32(pointer, n | (n << 16) | 0x2000_0000);
        write_u16(pointer + 6, 0xdc01);
        write_u32(pointer + 8, 0x8000_0000);
    }
}

extern "C" fn inactive_startup_task() {}

/// Uninterrupted hardware and shared-state translation of vendor `0x0000014c`.
///
/// Vendor callback addresses cannot be retained because those Thumb routines
/// are not present in the Rust image. Their scheduler slots are populated with
/// an inert Rust callback while preserving the original allocation/order.
///
/// # Safety
/// Packet DMA and the MAC clock domain must already be initialized.
pub unsafe fn initialize_vendor_startup_state(max_polls: u32) -> Result<(), MacStartupError> {
    unsafe {
        // 0x14e -> 0x7e50: stop packet DMA and wait for bit 23 to clear.
        let control = read_u32(crate::platform::mac_register(0x0600));
        if control & 1 != 0 {
            write_u32(crate::platform::mac_register(0x0600), control & !1);
            let mut polls = 0;
            while read_u32(crate::platform::mac_register(0x0600)) & (1 << 23) != 0 {
                if polls >= max_polls {
                    return Err(MacStartupError::PacketDmaStopTimeout);
                }
                polls += 1;
                core::hint::spin_loop();
            }
        }
        write_u8(
            crate::dtcm::LOW_MAC_RECEIVE_GATE_BITS.get(),
            read_u8(crate::dtcm::LOW_MAC_RECEIVE_GATE_BITS.get()) | 1,
        );
        // `mac_hw_reset_regs` (`0x000000bc`) clears the complete scheduler
        // retry/drain control word before any PAS work can become runnable.
        // DTCM is retained across firmware downloads, so relying on BSS-style
        // zero initialization leaves stale control bits that block TX forever.
        write_u32(crate::dtcm::MAC_RETRY_HARDWARE_STATE.get(), 0);

        // 0x152 -> 0x10024: collapse producer, consumer, scan, and release.
        let producer = read_u32(crate::platform::mac_register(0x0604));
        write_u32(SHARED + 0x10, producer);
        write_u32(SHARED + 0x14, producer);
        write_u32(crate::platform::mac_register(0x0608), producer);
        let dma_control = read_u32(crate::platform::mac_register(0x0600));
        write_u32(crate::platform::mac_register(0x0600), dma_control);

        // Normal startup clears these two descriptor-source halfwords here.
        // Unlike the wake path below, `fw_subsystem_init` never copies them
        // from PAS_BASE-8. Their retained DTCM counterparts at 0x04003670/72
        // are BSS and have no producer in the decompiled normal-start path.
        write_u16(packet_ram::duration_word(0), 0);
        write_u16(packet_ram::duration_word(1), 0);
        // Vendor initialized DTCM supplies zero for the EDCA hardware cache
        // and the optional contention-window override controls. Rebuilt code
        // consumes all three, so reconstruct them explicitly.
        write_u32(crate::dtcm::MAC_EDCA_SLOT_TIMING.get(), 0);
        write_u32(crate::dtcm::pas_backoff_override_enabled().get(), 0);
        write_u32(crate::dtcm::pas_backoff_override_window().get(), 0);
        for word in 0..5 {
            write_u32(
                crate::dtcm::rate_policy_word_unchecked(0, word).get(),
                read_u32(crate::dtcm::INITIALIZED_RATE_POLICIES.get() + word * 4),
            );
        }
        for word in 0..5 {
            write_u32(
                crate::dtcm::rate_policy_word_unchecked(1, word).get(),
                read_u32(crate::dtcm::INITIALIZED_RATE_POLICIES.get() + 0x14 + word * 4),
            );
        }

        write_u8(crate::dtcm::LOW_MAC_RESPONSE_CONTROL_BYTE.get(), 0);
        write_u8(SHARED + 4, 0);
        write_u16(SHARED + 2, 0x13);
        write_u8(SHARED + 5, 0);
        write_u32(crate::dtcm::MAC_TX_QUEUE_HEAD.get(), 0);
        write_u32(crate::dtcm::MAC_TX_QUEUE_TAIL.get(), 0);
        write_u8(crate::dtcm::MAC_BEACON_MODE.get(), 2);
        write_u32(SHARED + 0x10, 0);
        write_u32(SHARED + 0x14, 0);
        for address in [crate::dtcm::MAC_BEACON_CONTROL_STATE.get(), crate::dtcm::MAC_BEACON_SECONDARY_COMMAND.get(), crate::dtcm::MAC_BEACON_CONTROL.get(), crate::dtcm::MAC_BEACON_SELECTOR.get()] {
            write_u32(address, 0);
        }

        for pointer in [
            packet_ram::response_command(8),
            packet_ram::response_command(0),
            packet_ram::response_command(1),
            packet_ram::response_command(2),
            packet_ram::response_command(3),
            packet_ram::response_command(4),
            packet_ram::response_command(5),
            packet_ram::response_command(6),
            packet_ram::response_command(7),
            packet_ram::response_command(11),
            packet_ram::response_command(12),
        ] {
            build_tbtt(pointer);
        }

        write_u32(
            crate::platform::mac_register(0x0c00),
            packet_offset(packet_ram::response_pointers().start),
        );
        for index in 0..32 {
            write_u32(
                packet_ram::response_pointer(index),
                packet_offset(packet_ram::response_command(8)),
            );
        }
        for index in 0..23 {
            write_u32(crate::platform::mac_register(0x0210) + index * 4, 0);
        }

        let mut polls = 0;
        while read_u32(crate::platform::mac_register(0x0a20)) & 0x8000_0000 == 0 {
            if polls >= max_polls {
                return Err(MacStartupError::PipeControllerTimeout);
            }
            polls += 1;
            core::hint::spin_loop();
        }

        // 0x26a -> 0x52c.
        platform::prepare_mac_receive_hardware();

        write_u32(crate::dtcm::HOST_PAS_RING_HEAD.get(), 0);
        write_u32(crate::dtcm::HOST_PAS_RING_TAIL.get(), 0);

        let callback = inactive_startup_task as *const () as usize as u32 | 1;
        for index in [7_usize, 12, 11, 27, 13] {
            write_u32(crate::dtcm::scheduler_handler_unchecked(index).get(), callback);
        }
        // The timer objects are initialized below, but their intrusive-list
        // root is separate retained DTCM state. Leaving it untouched makes the
        // first timer insertion follow stale firmware pointers and corrupt the
        // cooperative scheduler before a TX confirmation can reach the host.
        write_u32(crate::dtcm::scheduler_timer_list_head().get(), 0);
        for object in [crate::dtcm::MAC_PHY_OPERATION_TIMER.get(), crate::dtcm::MAC_WAKE_TIMER.get()] {
            write_u32(object + 0x0c, callback);
            write_u32(object + 0x10, 0);
            write_u32(object + 4, 0);
        }
        write_u8(crate::dtcm::LOW_MAC_RECEIVE_STATE_BYTE.get(), 0);
        write_u8(crate::dtcm::MAC_WAKE_PHY_STATE.get(), 2);

        for pipe in 0..4 {
            let state = SHARED + 0xa0 + pipe * 0x6c;
            write_u8(state + 4, 0);
            write_u8(state + 5, 5);
            write_u16(state + 6, 0);
        }
    }
    Ok(())
}

unsafe fn reset_lmc_pool() {
    unsafe {
        write_u32(crate::dtcm::encryption_generation().get(), 0);
        write_u32(crate::dtcm::encryption_free_head().get(), 0);
        let mut head = 0_u32;
        for block in [packet_ram::lmc_anchor(0), packet_ram::lmc_anchor(1)] {
            write_u32(block, 0);
            write_u32(block + 0x18, head);
            write_u32(block + 0xf4, block as u32 + 0x1c);
            head = block as u32;
            write_u32(crate::dtcm::encryption_free_head().get(), head);
        }
    }
}

unsafe fn program_slot_timings(cfg: u16, base: u32) {
    let initial = if cfg & 0x20 != 0 { 0x10 } else { 10 };
    unsafe {
        write_u32(SHARED + 0x24, base.wrapping_add(initial));
        write_u32(SHARED + 0x28, base.wrapping_mul(2).wrapping_add(initial));
        write_u32(SHARED + 0x20, initial);
        write_u32(SHARED + 0x2c, base.wrapping_mul(3).wrapping_add(initial));
        write_u32(SHARED + 0x30, 2);
        write_u32(SHARED + 0x34, base.wrapping_mul(8).wrapping_add(2));
        write_u32(SHARED + 0x1c, base);
        let x16 = base.wrapping_mul(0x10).wrapping_add(2);
        let x24 = base.wrapping_mul(0x18).wrapping_add(2);
        write_u32(SHARED + 0x38, x16);
        write_u32(SHARED + 0x3c, x24);
        write_u32(
            crate::platform::mac_register(0x0e30),
            base.wrapping_mul(8).wrapping_sub(1),
        );
        write_u32(crate::platform::mac_register(0x0e58), x16);
        write_u32(crate::platform::mac_register(0x0e5c), x24);
        for (address, value) in [
            (crate::platform::mac_register(0x0618), 0x0000_004d),
            (crate::platform::mac_register(0x0614), 0x0000_01cd),
            (crate::platform::mac_register(0x0428), 0x0000_049d),
            (crate::platform::mac_register(0x042c), 0x0000_049d),
            (crate::platform::mac_register(0x0444), 0x0000_0258),
            (crate::platform::mac_register(0x0434), 0x0000_003d),
            (crate::platform::mac_register(0x0438), 0x0000_003d),
            (crate::platform::mac_register(0x0448), 0x0000_0258),
            (crate::platform::mac_register(0x043c), 0x0000_0298),
            (crate::platform::mac_register(0x0440), 0x0000_0298),
            (crate::platform::mac_register(0x044c), 0x0000_0134),
        ] {
            write_u32(address, value);
        }
        write_u32(crate::platform::mac_register(0x0810), 0);
    }
}

unsafe fn build_ba_descriptor(if_id: u8, pointer: usize) {
    let control = unsafe { read_u8(crate::dtcm::LOW_MAC_RESPONSE_CONTROL_BYTE.get()) };
    let words = [
        0x5900_0000 + u32::from(control) * 0x2000,
        0x5800_0003,
        0x5a00_0020,
        0x3100_0094,
        0x4700_0000,
        0x2080_800c + u32::from(if_id),
        0x3200_0000,
        0x6900_0000,
        0x0700_6820,
        0x6800_0000,
        0x6000_0000,
        0x0700_4600,
        0xf000_0000,
    ];
    for (index, value) in words.into_iter().enumerate() {
        unsafe { write_u32(pointer + index * 4, value) };
    }
}

unsafe fn clear_pipe_slot_ex(slot: u8) {
    let bit = slot - 2;
    unsafe {
        write_u32(
            crate::platform::mac_register(0x0a08),
            read_u32(crate::platform::mac_register(0x0a08)) & !(1_u32 << bit),
        );
        write_u32(
            crate::platform::mac_register(0x0a0c),
            read_u32(crate::platform::mac_register(0x0a0c)) & !(1_u32 << bit),
        );
        write_u32(
            crate::platform::mac_register(0x0a10),
            read_u32(crate::platform::mac_register(0x0a10)) & !(1_u32 << bit) & 0x00ff_ffff,
        );
        let field_register = if slot > 0x11 {
            crate::platform::mac_register(0x0a18)
        } else {
            crate::platform::mac_register(0x0a14)
        };
        write_u32(
            field_register,
            read_u32(field_register) & !(3_u32 << (u32::from(bit) * 2)),
        );
    }
}

unsafe fn set_pipe_enabled(slot: u8) {
    let bit = slot - 2;
    unsafe {
        write_u32(
            crate::platform::mac_register(0x0a04),
            read_u32(crate::platform::mac_register(0x0a04)) | (1_u32 << bit),
        )
    };
}

unsafe fn install_response_descriptors() {
    unsafe {
        clear_pipe_slot_ex(2);
        clear_pipe_slot_ex(3);
    }
    for (if_id, pointer) in [
        (0_u8, packet_ram::response_command(11)),
        (1, packet_ram::response_command(12)),
    ] {
        unsafe {
            write_u32(
                pointer,
                read_u32(SHARED + 0x30).wrapping_shl(16) | 0x2000_0000,
            );
            write_u16(pointer + 4, 0);
            write_u16(pointer + 6, 0);
            write_u32(pointer + 8, 0x5800_0000);
            build_ba_descriptor(if_id, pointer + 0x0c);
        }
        for slot in if if_id == 0 {
            [2_u8, 0x0b]
        } else {
            [3_u8, 0x0c]
        } {
            unsafe { program_pipe_slot(pointer, slot) };
        }
    }
    unsafe {
        set_pipe_enabled(2);
        set_pipe_enabled(3);
        clear_pipe_slot_ex(0x0b);
        clear_pipe_slot_ex(0x0c);
        set_pipe_enabled(0x0b);
        set_pipe_enabled(0x0c);
        let first = packet_offset(packet_ram::response_command(11));
        let second = packet_offset(packet_ram::response_command(12));
        write_u32(crate::dtcm::MAC_BEACON_RESPONSE_COMMANDS.get(), first);
        write_u32(crate::dtcm::MAC_BEACON_RESPONSE_COMMANDS.get() + 4, second);
        write_u32(crate::dtcm::MAC_BEACON_SECONDARY_COMMAND.get(), first);
        write_u32(crate::dtcm::MAC_BEACON_CONTROL.get(), second);
    }
}

unsafe fn export_pipe_counters() {
    for index in 0..4 {
        let record = SHARED + index * 0x44;
        let hardware = crate::platform::mac_register(0x0040) + index * 0x0c;
        if unsafe { read_u32(record + 0x718) } == 0 {
            unsafe {
                write_u32(hardware + 0x20, u32::MAX);
                write_u32(hardware + 0x24, 0xff);
                write_u32(hardware + 0x28, 0xff);
            }
        } else {
            let source = record + 0x6dc;
            let stats = crate::platform::mac_register(0x1200) + index * 0x20;
            unsafe {
                write_u32(hardware + 0x20, read_u32(source));
                write_u32(hardware + 0x24, read_u32(source + 4) & 0xffff);
                write_u32(hardware + 0x28, u32::from(read_u8(record + 0x700)));
                write_u32(stats + 0x10, read_u32(record + 0x708));
                write_u32(stats + 0x14, read_u32(record + 0x70c));
                write_u32(stats + 0x18, read_u32(record + 0x710));
                write_u32(stats + 0x1c, read_u32(record + 0x714));
                write_u32(stats + 4, read_u32(record + 0x71c));
                write_u32(stats, read_u32(record + 0x718));
            }
        }
    }
    unsafe {
        write_u32(
            crate::platform::mac_register(0x011c),
            read_u32(crate::platform::mac_register(0x0114)),
        )
    };
}

unsafe fn program_mode_registers() {
    let mode = unsafe { read_u32(WAKE + 0x24) };
    let wide = mode & (1 << 18) != 0;
    let selector = if wide { 0x001c_0783 } else { 0x0018_0783 };
    unsafe {
        write_u32(crate::dtcm::MAC_BEACON_SELECTOR.get(), selector);
        write_u32(crate::platform::mac_register(0x0a04), selector);
        write_u32(crate::platform::mac_register(0x0a1c), 0x827b_ffdf);
        write_u32(
            crate::platform::mac_register(0x0204),
            0x0279_fe00 | if wide { 0x0004_0000 } else { 0 },
        );
        write_u32(crate::platform::mac_register(0x0200), mode);
        write_u32(crate::platform::mac_register(0x0310), 0x7800_0000);
    }
}

/// Vendor `mac_reinit_after_wake`, including static RX/MAC restoration, pipe
/// state, FIFO synchronization, descriptor images, bounded controller wait,
/// and configured-mode publication.
pub unsafe fn reinitialize_after_wake(max_polls: u32) -> Result<(), MacWakeError> {
    unsafe {
        if read_u8(WAKE + 0x1e) == 0 {
            return Ok(());
        }
        write_u8(WAKE + 0x1e, 0);
    }
    // Vendor `mac_reinit_after_wake` restores the RX subsystem and pipe state,
    // but does not repeat the global packet-DMA/controller reset performed at
    // cold startup. Reprogramming that block while the controller is enabled
    // can expose its reset/default routing state.
    platform::prepare_mac_receive_hardware();
    unsafe {
        program_mac_address(
            crate::dtcm::low_mac_own_mac_byte_unchecked(0, 0).get(),
            crate::platform::mac_register(0x0030),
            0x101,
        );
        program_mac_address(
            crate::dtcm::low_mac_own_mac_byte_unchecked(1, 0).get(),
            crate::platform::mac_register(0x0048),
            0x101,
        );
        rebuild_pipe_state();
        let producer = read_u32(crate::platform::mac_register(0x0604));
        radio::synchronize_after_wake(producer);
        // `mac_reinit_after_wake` restores these only after RX, pipe, and
        // register synchronization. This is wake-context restoration, not
        // VIF/JOIN programming.
        write_u16(packet_ram::duration_word(0), read_u16(crate::dtcm::duration_source(0).unwrap().get()));
        write_u16(packet_ram::duration_word(1), read_u16(crate::dtcm::duration_source(1).unwrap().get()));

        if let Some(vif) = crate::vif::wake_reinit_candidate() {
            program_mac_address(
                crate::dtcm::low_mac_peer_address_byte_unchecked(vif, 0).get(),
                crate::platform::mac_register(0x003c),
                0x101,
            );
            write_u32(crate::platform::mac_register(0x0258), 0x0200_0000);
            write_u32(crate::platform::mac_register(0x0248), 0x0200_0001);
        }

        for pointer in [
            packet_ram::response_command(8),
            packet_ram::response_command(0),
            packet_ram::response_command(1),
            packet_ram::response_command(2),
            packet_ram::response_command(3),
            packet_ram::response_command(4),
            packet_ram::response_command(5),
            packet_ram::response_command(6),
            packet_ram::response_command(7),
            packet_ram::response_command(11),
            packet_ram::response_command(12),
        ] {
            build_tbtt(pointer);
        }
        for index in 0..32 {
            write_u32(
                packet_ram::response_pointer(index),
                read_u32(crate::dtcm::wake_response_pointer_unchecked(index).get()),
            );
        }
        write_u32(
            crate::platform::mac_register(0x0c00),
            packet_offset(packet_ram::response_pointers().start),
        );
        let mut polls = 0;
        while read_u32(crate::platform::mac_register(0x0a20)) & 0x8000_0000 == 0 {
            if polls >= max_polls {
                // Retry the complete wake restoration on the next cooperative
                // service call rather than continuing from a partial reset.
                write_u8(WAKE + 0x1e, 1);
                return Err(MacWakeError::PipeControllerTimeout);
            }
            polls += 1;
            core::hint::spin_loop();
        }
        write_u32(crate::platform::mac_register(0x1300), 0x0100_0000);
        write_u32(crate::platform::mac_register(0x0090), 5);
        write_u32(crate::platform::mac_register(0x0094), 0x23);
        write_u32(crate::platform::mac_register(0x0098), 0x30);
        reset_lmc_pool();

        if read_u16(crate::dtcm::LOW_MAC_CURRENT_CHANNEL.get()) != 0 {
            write_u8(WAKE + 0x1d, 1);
            program_slot_timings(read_u16(SHARED + 2), read_u32(SHARED + 0x1c));
            install_response_descriptors();
            export_pipe_counters();
            if read_u32(WAKE + 0x28) != 0 {
                program_mac_address(
                    read_u32(WAKE + 0x28) as usize,
                    crate::platform::mac_register(0x003c),
                    0x301,
                );
            }
            program_mode_registers();
            write_u32(WAKE + 0x24, read_u32(crate::platform::mac_register(0x0200)));
            write_u32(crate::platform::mac_register(0x0200), 0x00c0_8018);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_classes_match_vendor_groups() {
        assert_eq!(rate_phy_class(0, 0, true), 1);
        assert_eq!(rate_phy_class(0, 1, true), 0);
        assert_eq!(rate_phy_class(0, 6, true), 2);
        assert_eq!(rate_phy_class(0, 14, false), 4);
        assert_eq!(rate_phy_class(0, 14, true), 5);
    }

    #[test]
    fn duration_and_fallback_tables_are_bounded() {
        assert_eq!(TX_DURATION_TIMING[0], 0x00c2);
        assert_eq!(TX_DURATION_TIMING[9], 0x0016);
        assert_eq!(ofdm_duration(0, 14), 0);
        assert!(extended_airtime(0x17, 0, 14, true) != 0);
        let fallback = fill_fallbacks(0);
        assert_eq!(fallback[0], 0);
        assert_eq!(fallback[13], 10);
        assert_eq!(fallback[21], 10);
        assert_eq!(secondary_fallbacks(0, fallback[21]), [10; 8]);
        assert_eq!(
            secondary_fallbacks(1 << 16, 10),
            [10, 10, 16, 16, 16, 16, 16, 16]
        );
    }
}
