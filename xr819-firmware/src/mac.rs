//! Vendor MAC wake restoration and post-channel reprogramming.

use crate::{platform, radio};

const SHARED: usize = 0x0400_1680;
const WAKE: usize = 0x0400_1ac0;
const PAS_BASE: usize = 0x0400_3ae8;
const RATE_RAM: usize = 0x0900_75c0;

const RATE_ENCODING: [u8; 22] = [
    0, 0, 1, 1, 0, 0, 2, 3, 4, 5, 6, 7, 8, 9, 2, 4, 5, 6, 7, 8, 9, 10,
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

pub fn extended_airtime(cfg: u16, rate: u8, length: u16, stream: bool) -> u16 {
    let class = rate_phy_class(cfg, rate, stream);
    let mut value = if class < 2 {
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
            unsafe { write_u32(RATE_RAM + usize::from(index) * 0x10 + word * 4, value) };
        }
        if hardware_class >= 4 {
            let secondary = unsafe { read_u8(SHARED + 0x47a + usize::from(rate)) };
            let secondary_index = OFFSET_TABLE[hardware_class as usize][1]
                .wrapping_add(RATE_ATTRIBUTE[rate as usize]);
            let entry = build_rate_entry(cfg, hardware_class, secondary);
            for (word, value) in entry.into_iter().enumerate() {
                unsafe {
                    write_u32(
                        RATE_RAM + usize::from(secondary_index) * 0x10 + word * 4,
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
    let entry = unsafe { read_u32(RATE_RAM + index * 0x10) } & 0x00ff_ffff;
    let base = unsafe { read_u32(SHARED + 0x1c) };
    unsafe {
        write_u32(
            SHARED + 0x44,
            base.wrapping_mul(3).wrapping_add(entry).wrapping_mul(8),
        )
    };
}

unsafe fn program_pipe_slot(pointer: usize, slot: u8) {
    unsafe {
        write_u32(
            0x0900_7000 + usize::from(slot) * 4,
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
    for address in [0x09c0_0a08, 0x09c0_0a0c] {
        let value = unsafe { read_u32(address) } | (1_u32 << register_bit);
        unsafe { write_u32(address, value) };
    }
    let value = unsafe { read_u32(0x09c0_0a10) } & !(1_u32 << logical) & 0x00ff_ffff;
    unsafe { write_u32(0x09c0_0a10, value) };
}

unsafe fn build_control_frame(if_id: u8, pointer: usize, ack: bool) {
    let control = unsafe { read_u8(0x0400_3a70) };
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
    if ack && unsafe { read_u8(0x0400_3ae4 + usize::from(if_id)) } != 0 {
        unsafe { program_pipe_slot(pointer, 0x14) };
    }
}

unsafe fn save_register_context() {
    unsafe {
        write_u32(0x09c0_1404, read_u32(0x0400_2078));
        write_u32(0x09c0_1408, read_u32(0x0400_207c));
        write_u32(0x09c0_140c, u32::from(read_u16(0x0400_8606)));
        write_u32(0x09c0_1410, read_u32(0x0400_2080));
        write_u32(0x09c0_1400, read_u32(0x0400_2084));
    }
}

/// Full vendor `mac_reprogram_after_channel` for the currently represented VIF
/// and packet-RAM state.
pub unsafe fn reprogram_after_channel() {
    unsafe {
        write_u8(WAKE + 0x1d, 0);
        write_u32(0x09c0_0800, 0x0000_75c0);
        for index in 0..3 {
            let vif = PAS_BASE + index * 0x98;
            if read_u8(vif) == 2 {
                program_rate_tables(vif);
            }
        }
        program_ifs_timing();
        build_control_frame(0, 0x0900_7d14, true);
        build_control_frame(1, 0x0900_7d68, true);
        build_control_frame(0, 0x0900_7dbc, false);
        build_control_frame(1, 0x0900_7e10, false);
        write_u32(0x09c0_0200, read_u32(WAKE + 0x24));
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
    unsafe {
        write_u32(0x09c0_0e8c, 0);
        write_u32(0x09c0_0e60, 2);
        write_u32(0x09c0_0e80, 0x100);
        write_u32(0x09c0_0e88, 0xff);
    }
    let descriptors = [0x09c6_0000, 0x09c6_0080, 0x09c6_0100, 0x09c6_0180];
    let addresses = [0x7080_u32, 0x71d0, 0x7320, 0x7470];
    for (base, address) in descriptors.into_iter().zip(addresses) {
        unsafe {
            write_u32(base + 0x0c, address);
            write_u32(base + 0x10, 0x54);
            write_u32(base + 0x14, 1);
        }
    }
    for index in 0..4 {
        let record = SHARED + index * 0x6c - 0x80;
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
                    0x0900_7080 + index as u32 * 0x150 + entry as u32 * 0x54,
                )
            };
        }
    }
    unsafe { write_u32(0x09c0_0e8c, 0xbf) };
    unsafe {
        write_u32(0x0901_6a28, 0x4e14_0000);
        for index in 0..33 {
            write_u32(0x0901_6a2c + index * 4, 0x2201_6a28);
        }
        write_u32(0x0901_6ab0, 0xf000_0000);
    }
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

unsafe fn reset_lmc_pool() {
    unsafe {
        write_u32(0x0400_8620, 0);
        write_u32(0x0400_861c, 0);
        let mut head = 0_u32;
        for block in [0x0901_6ab4_usize, 0x0901_7500] {
            write_u32(block, 0);
            write_u32(block + 0x18, head);
            write_u32(block + 0xf4, block as u32 + 0x1c);
            head = block as u32;
            write_u32(0x0400_861c, head);
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
        write_u32(0x09c0_0e30, base.wrapping_mul(8).wrapping_sub(1));
        write_u32(0x09c0_0e58, x16);
        write_u32(0x09c0_0e5c, x24);
        for (address, value) in [
            (0x09c0_0618, 0x0000_004d),
            (0x09c0_0614, 0x0000_01cd),
            (0x09c0_0428, 0x0000_049d),
            (0x09c0_042c, 0x0000_049d),
            (0x09c0_0444, 0x0000_0258),
            (0x09c0_0434, 0x0000_003d),
            (0x09c0_0438, 0x0000_003d),
            (0x09c0_0448, 0x0000_0258),
            (0x09c0_043c, 0x0000_0298),
            (0x09c0_0440, 0x0000_0298),
            (0x09c0_044c, 0x0000_0134),
        ] {
            write_u32(address, value);
        }
        write_u32(0x09c0_0810, 0);
    }
}

unsafe fn build_ba_descriptor(if_id: u8, pointer: usize) {
    let control = unsafe { read_u8(0x0400_3a70) };
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
        write_u32(0x09c0_0a08, read_u32(0x09c0_0a08) & !(1_u32 << bit));
        write_u32(0x09c0_0a0c, read_u32(0x09c0_0a0c) & !(1_u32 << bit));
        write_u32(
            0x09c0_0a10,
            read_u32(0x09c0_0a10) & !(1_u32 << bit) & 0x00ff_ffff,
        );
        let field_register = if slot > 0x11 {
            0x09c0_0a18
        } else {
            0x09c0_0a14
        };
        write_u32(
            field_register,
            read_u32(field_register) & !(3_u32 << (u32::from(bit) * 2)),
        );
    }
}

unsafe fn set_pipe_enabled(slot: u8) {
    let bit = slot - 2;
    unsafe { write_u32(0x09c0_0a04, read_u32(0x09c0_0a04) | (1_u32 << bit)) };
}

unsafe fn install_response_descriptors() {
    unsafe {
        clear_pipe_slot_ex(2);
        clear_pipe_slot_ex(3);
    }
    for (if_id, pointer) in [(0_u8, 0x0900_7f60_usize), (1, 0x0900_7fb4)] {
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
        write_u32(0x0400_1a88, 0x0000_7f60);
        write_u32(0x0400_1a8c, 0x0000_7fb4);
        write_u32(0x0400_1aac, 0x0000_7f60);
        write_u32(0x0400_1ab0, 0x0000_7fb4);
    }
}

unsafe fn export_pipe_counters() {
    for index in 0..4 {
        let record = SHARED + index * 0x44;
        let hardware = 0x09c0_0040 + index * 0x0c;
        if unsafe { read_u32(record + 0x718) } == 0 {
            unsafe {
                write_u32(hardware + 0x20, u32::MAX);
                write_u32(hardware + 0x24, 0xff);
                write_u32(hardware + 0x28, 0xff);
            }
        } else {
            let source = record + 0x6dc;
            let stats = 0x09c0_1200 + index * 0x20;
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
    unsafe { write_u32(0x09c0_011c, read_u32(0x09c0_0114)) };
}

unsafe fn program_mode_registers() {
    let mode = unsafe { read_u32(WAKE + 0x24) };
    let wide = mode & (1 << 18) != 0;
    let selector = if wide { 0x001c_0783 } else { 0x0018_0783 };
    unsafe {
        write_u32(0x0400_1ab4, selector);
        write_u32(0x09c0_0a04, selector);
        write_u32(0x09c0_0a1c, 0x827b_ffdf);
        write_u32(
            0x09c0_0204,
            0x0279_fe00 | if wide { 0x0004_0000 } else { 0 },
        );
        write_u32(0x09c0_0200, mode);
        write_u32(0x09c0_0310, 0x7800_0000);
    }
}

/// Vendor `mac_reinit_after_wake`, including static RX/MAC restoration, pipe
/// state, FIFO synchronization, descriptor images, bounded controller wait,
/// and configured-mode publication.
pub unsafe fn reinitialize_after_wake(max_polls: u32) -> Result<(), MacWakeError> {
    unsafe { write_u8(WAKE + 0x1e, 0) };
    platform::prepare_mac_receive_hardware();
    unsafe {
        program_mac_address(0x0400_3acc, 0x09c0_0030, 0x101);
        program_mac_address(0x0400_3ad2, 0x09c0_0048, 0x101);
        rebuild_pipe_state();
        let producer = read_u32(0x09c0_0604);
        radio::synchronize_after_wake(producer);
        write_u16(0x0900_7bc0, read_u16(0x0400_3670));
        write_u16(0x0900_7bc2, read_u16(0x0400_3672));

        for vif in 0..2 {
            let state = 0x0400_3e98 + vif * 0x3b0;
            let mode = read_u8(state + 0x18);
            if (mode == 5 || mode == 6) && read_u8(state + 0x3c6) != 0 {
                program_mac_address(0x0400_3ad8 + vif * 6, 0x09c0_003c, 0x101);
                write_u32(0x09c0_0258, 0x0200_0000);
                write_u32(0x09c0_0248, 0x0200_0001);
                break;
            }
        }

        for pointer in [
            0x0900_7e64,
            0x0900_7bc4,
            0x0900_7c18,
            0x0900_7c6c,
            0x0900_7cc0,
            0x0900_7d14,
            0x0900_7d68,
            0x0900_7dbc,
            0x0900_7e10,
            0x0900_7f60,
            0x0900_7fb4,
        ] {
            build_tbtt(pointer);
        }
        for index in 0..32 {
            write_u32(0x0900_7000 + index * 4, read_u32(0x0400_35f0 + index * 4));
        }
        write_u32(0x09c0_0c00, 0x0000_7000);
        let mut polls = 0;
        while read_u32(0x09c0_0a20) & 0x8000_0000 == 0 {
            if polls >= max_polls {
                // Retry the complete wake restoration on the next cooperative
                // service call rather than continuing from a partial reset.
                write_u8(WAKE + 0x1e, 1);
                return Err(MacWakeError::PipeControllerTimeout);
            }
            polls += 1;
            core::hint::spin_loop();
        }
        write_u32(0x09c0_1300, 0x0100_0000);
        write_u32(0x09c0_0090, 5);
        write_u32(0x09c0_0094, 0x23);
        write_u32(0x09c0_0098, 0x30);
        reset_lmc_pool();

        if read_u16(0x0400_3a68) != 0 {
            write_u8(WAKE + 0x1d, 1);
            program_slot_timings(read_u16(SHARED + 2), read_u32(SHARED + 0x1c));
            install_response_descriptors();
            export_pipe_counters();
            if read_u32(WAKE + 0x28) != 0 {
                program_mac_address(read_u32(WAKE + 0x28) as usize, 0x09c0_003c, 0x301);
            }
            program_mode_registers();
            write_u32(WAKE + 0x24, read_u32(0x09c0_0200));
            write_u32(0x09c0_0200, 0x00c0_8018);
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
