#![no_std]
#![no_main]

use core::arch::naked_asm;
use core::panic::PanicInfo;
use core::ptr::{read_volatile, write_volatile};

const DOWNLOAD_IMAGE_SIZE: *mut u32 = 0x0900_ff80 as *mut u32;
const DOWNLOAD_TRACE_PC: *mut u32 = 0x0900_ff8c as *mut u32;
const MAILBOX_MAGIC: u32 = 0x5852_3831; // "XR81"

#[unsafe(naked)]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
pub unsafe extern "C" fn _start() -> ! {
    naked_asm!(
        "ldr sp, =0x0800ff00",
        "b {main}",
        main = sym rust_main,
    );
}

#[unsafe(no_mangle)]
extern "C" fn rust_main() -> ! {
    unsafe {
        write_volatile(DOWNLOAD_TRACE_PC, _start as *const () as usize as u32);
        write_volatile(DOWNLOAD_IMAGE_SIZE, MAILBOX_MAGIC);
    }

    loop {
        // Keep an observable heartbeat without requiring a timer or interrupt
        // controller. The host can verify that the CPU remains alive by
        // reading a changing value from the trace-PC mailbox word.
        unsafe {
            let value = read_volatile(DOWNLOAD_TRACE_PC);
            write_volatile(DOWNLOAD_TRACE_PC, value.wrapping_add(1));
        }
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
