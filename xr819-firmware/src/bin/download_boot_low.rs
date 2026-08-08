#![no_std]
#![no_main]

use core::arch::naked_asm;
use core::panic::PanicInfo;
use xr819_firmware::download::Control;
use xr819_firmware::loader::apply_vendor_register_initialization;

const LOW_MAIN_IMAGE_BASE: usize = 0;
const THUMB_ENTRY: usize = 1;

#[unsafe(naked)]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
pub extern "C" fn _start() -> ! {
    naked_asm!(
        "mrs r0, cpsr",
        "orr r0, r0, #0xc0",
        "msr cpsr_c, r0",
        "mov r2, #1",
        "mcr p15, 0, r2, c9, c1, 1",
        "mov r1, #0x00400000",
        "orr r2, r1, r2",
        "mcr p15, 0, r2, c9, c1, 0",
        // Match the vendor high bootstrap before low Thumb startup. The
        // translated 0x0aa80004 = 0x200 clock/reset write otherwise stops
        // instruction progress immediately after postcode DMC3 on cold boot.
        "ldr r2, =0x00001f74",
        "mcr p15, 0, r2, c1, c0, 0",
        "ldr r0, =__relocated_load",
        "ldr r1, =__relocated_start",
        "ldr r2, =__relocated_end",
        "ldr r3, ={main}",
        "2:",
        "cmp r1, r2",
        "ldrlo r4, [r0], #4",
        "strlo r4, [r1], #4",
        "blo 2b",
        "ldr sp, =0x0400c000",
        "bx r3",
        main = sym rust_main,
    );
}

#[unsafe(no_mangle)]
extern "C" fn rust_main() -> ! {
    let control = unsafe { Control::get() };
    control.advertise();
    let size = control.wait_for_image_size();
    if control.copy_image_to(size, LOW_MAIN_IMAGE_BASE).is_ok() {
        apply_vendor_register_initialization();
        for _ in 0..1_000_000 {
            core::hint::spin_loop();
        }
        let entry: extern "C" fn() -> ! = unsafe { core::mem::transmute(THUMB_ENTRY) };
        entry();
    }

    loop {
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
