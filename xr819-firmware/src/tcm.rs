//! Feature-gated CP15 TCM region diagnostics.

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TcmRegionInfo {
    pub tcm_type_register: u32,
    pub dtcm_register: u32,
    pub itcm_register: u32,
}

pub const fn size_kib(region_register: u32) -> Option<u32> {
    const SIZES: [i16; 16] = [
        0, -1, -1, 4, 8, 16, 32, 64, 128, 256, 512, 1024, -1, -1, -1, -1,
    ];
    let value = SIZES[((region_register >> 2) & 0x0f) as usize];
    if value < 0 { None } else { Some(value as u32) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_architected_tcm_region_size_field() {
        assert_eq!(size_kib(3 << 2), Some(4));
        assert_eq!(size_kib(7 << 2), Some(64));
        assert_eq!(size_kib(8 << 2), Some(128));
        assert_eq!(size_kib(1 << 2), None);
    }
}
