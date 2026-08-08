//! Loader-applied XR819 hardware initialization sections.
//!
//! Vendor firmware container loader side effects needed by flat custom images.
//!
//! Besides the 41 type-2 MMIO address/value pairs, the vendor container carries
//! four later type-0 copies directly into MAC/PHY memory at `0x0ab81000` and
//! `0x0ab88400..0x0ab88f2f`. The copies and all 41 ordered pairs were verified
//! byte-for-byte against the annotated `fw_xr819.bin` supplied in
//! `xr819-fw.tar.gz`. Both Rust downloaders reproduce all five groups before
//! entering the main image.

const VENDOR_MEMORY_INITIALIZATION: &[(usize, &[u8])] = &[
    (0x0ab8_1000, include_bytes!("../data/vendor-ab81000.bin")),
    (0x0ab8_8400, include_bytes!("../data/vendor-ab88400.bin")),
    (0x0ab8_8800, include_bytes!("../data/vendor-ab88800.bin")),
    (0x0ab8_8c00, include_bytes!("../data/vendor-ab88c00.bin")),
];

const VENDOR_REGISTER_INITIALIZATION: &[(u32, u32)] = &[
    (0x0ab8_0004, 0x0000_0023),
    (0x0ab8_0008, 0x0000_003f),
    (0x0ab8_0014, 0x0000_001c),
    (0x0ab8_0018, 0x0000_002c),
    (0x0ab8_001c, 0x0000_0044),
    (0x0ab8_0020, 0x0000_0041),
    (0x0ab8_0024, 0x0000_003e),
    (0x0ab8_0064, 0x0021_0140),
    (0x0ab8_0070, 0x0000_0006),
    (0x0ab8_0074, 0x0000_4040),
    (0x0ab8_0080, 0x0190_007e),
    (0x0ab8_0108, 0x0020_0300),
    (0x0ab8_010c, 0x0000_0009),
    (0x0ab8_0110, 0x0050_01c2),
    (0x0ab8_0408, 0x0000_0081),
    (0x0ab8_040c, 0x0000_007f),
    (0x0ab8_0c48, 0x0050_0850),
    (0x0ab8_0c04, 0x0000_000d),
    (0x0ab8_0c08, 0x0010_00a0),
    (0x0ab8_0c24, 0x0007_fcef),
    (0x0ab8_0c1c, 0x0000_0004),
    (0x0ab8_0c10, 0x0000_0afb),
    (0x0ab8_0c3c, 0x0803_f7a0),
    (0x0ab8_0c28, 0x0000_0085),
    (0x0ab8_0c30, 0x0000_00fc),
    (0x0ab8_0000, 0x001e_27ff),
    (0x0ab8_0044, 0x0150_0040),
    (0x0ab8_8014, 0x0033_0003),
    (0x0ab8_808c, 0x0000_103f),
    (0x0ab8_8090, 0x1010_103f),
    (0x0ab8_0c54, 0x0000_001f),
    (0x0ab9_0008, 0xa290_9330),
    (0x0aba_8040, 0x0007_7bbd),
    (0x0aba_805c, 0x430c_261a),
    (0x0aba_8574, 0x090a_0f0f),
    (0x0aba_8070, 0x0000_0004),
    (0x0aba_8074, 0x3638_3939),
    (0x0aba_8550, 0x0003_6389),
    (0x0aba_8558, 0x1aa0_ffa2),
    (0x0abb_0008, 0x0000_026d),
    (0x0abb_0078, 0x0048_0035),
];

pub fn apply_vendor_memory_initialization() {
    for &(destination, data) in VENDOR_MEMORY_INITIALIZATION {
        for (index, word) in data.chunks_exact(4).enumerate() {
            let value = u32::from_le_bytes([word[0], word[1], word[2], word[3]]);
            unsafe { ((destination + index * 4) as *mut u32).write_volatile(value) };
        }
    }
}

pub fn apply_vendor_register_initialization() {
    for &(address, value) in VENDOR_REGISTER_INITIALIZATION {
        unsafe { (address as *mut u32).write_volatile(value) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vendor_section_has_all_41_pairs() {
        assert_eq!(VENDOR_REGISTER_INITIALIZATION.len(), 41);
        assert_eq!(VENDOR_MEMORY_INITIALIZATION.len(), 4);
        assert_eq!(VENDOR_MEMORY_INITIALIZATION[0].1.len(), 0x400);
        assert!(
            VENDOR_MEMORY_INITIALIZATION[1..]
                .iter()
                .all(|(_, data)| data.len() == 0x330)
        );
    }
}
