#![no_std]
#![no_main]

use core::panic::PanicInfo;

type StablePublish = unsafe extern "C" fn(*mut u8, u16);
type StableDivide = unsafe extern "C" fn(u32, u32) -> u32;

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

/// Preserve the stable channel-frequency division call while inserting the
/// mandatory vendor-compatible 16-slot gain build after channel-power state is
/// published by the low image.
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.gain_entry")]
pub unsafe extern "C" fn xr819_gain_extension(dividend: u32, divisor: u32) -> u32 {
    let stable_divide: StableDivide = unsafe { core::mem::transmute(0x0000_7081_usize) };
    let quotient = unsafe { stable_divide(dividend, divisor) };
    let status = match unsafe { xr819_firmware::phy::program_all_tx_gain_slots(200) } {
        Ok(()) => 0x4741_494e,
        Err(_) => 0x4741_4946,
    };
    unsafe { (0x0900_fd08 as *mut u32).write_volatile(status) };
    quotient
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
