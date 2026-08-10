#![no_std]
#![no_main]

use core::panic::PanicInfo;

type StablePublish = unsafe extern "C" fn(*mut u8, u16);

/// High-SRAM proof that split-loaded extension code can execute while the
/// byte-identical low startup retains HIF ownership.
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
pub unsafe extern "C" fn xr819_hif_extension_probe(transport: *mut u8, length: u16) {
    unsafe { (0x0900_fd00 as *mut u32).write_volatile(0x4558_5431) };
    let stable_publish: StablePublish = unsafe { core::mem::transmute(0x0000_162d_usize) };
    unsafe { stable_publish(transport, length) };
    unsafe { (0x0900_fd04 as *mut u32).write_volatile(0x4558_5432) };
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
