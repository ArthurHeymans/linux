//! Loader-applied XR819 hardware initialization sections.
//!
//! Vendor firmware container section type 2 is a sequence of MMIO
//! address/value pairs. The vendor download bootloader applies this section
//! before jumping to the low firmware image. A flat custom image must reproduce
//! that loader side effect explicitly.

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
    }
}
