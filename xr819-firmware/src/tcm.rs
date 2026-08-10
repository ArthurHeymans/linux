//! Feature-gated CP15 TCM region diagnostics.

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TcmRegionInfo {
    pub tcm_type_register: u32,
    pub dtcm_register: u32,
    pub itcm_register: u32,
}

pub const fn size_kib(region_register: u32) -> Option<u32> {
    const SIZES: [i16; 16] = [0, -1, -1, 4, 8, 16, 32, 64, 128, 256, 512, 1024, -1, -1, -1, -1];
    let value = SIZES[((region_register >> 2) & 0x0f) as usize];
    if value < 0 { None } else { Some(value as u32) }
}

#[cfg(all(target_arch = "arm", target_feature = "thumb-mode", feature = "tcm-size-diagnostic"))]
core::arch::global_asm!(
    ".syntax unified",
    ".pushsection .text.xr819_read_tcm_regions, \"ax\", %progbits",
    ".arm",
    ".align 2",
    ".global xr819_read_tcm_regions",
    ".type xr819_read_tcm_regions, %function",
    "xr819_read_tcm_regions:",
    "push {{lr}}",
    "mrc p15, 0, r3, c0, c0, 2",
    "mrc p15, 0, r12, c9, c1, 0",
    "mrc p15, 0, lr, c9, c1, 1",
    "str r3, [r0]",
    "str r12, [r1]",
    "str lr, [r2]",
    "pop {{pc}}",
    ".size xr819_read_tcm_regions, .-xr819_read_tcm_regions",
    ".popsection",
);

#[cfg(all(target_arch = "arm", not(target_feature = "thumb-mode"), feature = "tcm-size-diagnostic"))]
unsafe fn read_registers(tcm_type: &mut u32, dtcm: &mut u32, itcm: &mut u32) {
    unsafe {
        core::arch::asm!(
            "mrc p15, 0, {tcm_type}, c0, c0, 2",
            "mrc p15, 0, {dtcm}, c9, c1, 0",
            "mrc p15, 0, {itcm}, c9, c1, 1",
            tcm_type = out(reg) *tcm_type,
            dtcm = out(reg) *dtcm,
            itcm = out(reg) *itcm,
            options(nostack),
        );
    }
}

#[cfg(all(target_arch = "arm", target_feature = "thumb-mode", feature = "tcm-size-diagnostic"))]
unsafe fn read_registers(tcm_type: &mut u32, dtcm: &mut u32, itcm: &mut u32) {
    unsafe extern "C" {
        fn xr819_read_tcm_regions(tcm_type: *mut u32, dtcm: *mut u32, itcm: *mut u32);
    }
    unsafe { xr819_read_tcm_regions(tcm_type, dtcm, itcm) };
}

#[cfg(all(target_arch = "arm", feature = "tcm-size-diagnostic"))]
pub fn read_region_info() -> TcmRegionInfo {
    let mut info = TcmRegionInfo::default();
    unsafe {
        read_registers(
            &mut info.tcm_type_register,
            &mut info.dtcm_register,
            &mut info.itcm_register,
        )
    };
    info
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