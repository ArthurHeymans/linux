#![no_std]
#![no_main]

use core::arch::naked_asm;
use core::panic::PanicInfo;
use xr819_firmware::download::Control;
use xr819_firmware::loader::{
    apply_vendor_memory_initialization, apply_vendor_register_initialization,
};

const LOW_MAIN_IMAGE_BASE: usize = 0;
const THUMB_ENTRY: usize = 1;
const LOW_EXTENSION_VENEER: usize = 0x0000_9720;

unsafe fn install_high_extension_veneer() {
    unsafe {
        // Thumb `bx pc; b .-2` switches to the aligned ARM literal veneer.
        (LOW_EXTENSION_VENEER as *mut u16).write_volatile(0x4778);
        ((LOW_EXTENSION_VENEER + 2) as *mut u16).write_volatile(0xe7fd);
        // ARM `ldr pc, [pc, #-4]`, followed by its high-SRAM target literal.
        ((LOW_EXTENSION_VENEER + 4) as *mut u32).write_volatile(0xe51f_f004);
        ((LOW_EXTENSION_VENEER + 8) as *mut u32).write_volatile(0xfff0_0000);
    }
}

#[unsafe(naked)]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
pub extern "C" fn _start() -> ! {
    naked_asm!(
        "mrs r0, cpsr",
        "orr r0, r0, #0xc0",
        "msr cpsr_c, r0",
        // Match the vendor CP15 TCM-region setup: enable the fixed ITCM
        // mapping at instruction address 0, then enable DTCM at 0x04000000.
        // The streamed Thumb image is copied to ITCM; its stack uses DTCM.
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
    // Keep the validated low startup image byte-for-byte while placing any
    // extension payload in the vendor high-SRAM executable window.
    if control
        .copy_image_split_to(size, 0x7500, LOW_MAIN_IMAGE_BASE, 0xfff0_0000)
        .is_ok()
    {
        apply_vendor_register_initialization();
        apply_vendor_memory_initialization();
        if size > 0x7500 {
            unsafe { install_high_extension_veneer() };
        }
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
