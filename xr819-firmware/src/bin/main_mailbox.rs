#![no_std]
#![no_main]

use core::arch::naked_asm;
use core::panic::PanicInfo;
use core::ptr::{read_volatile, write_volatile};

const MAILBOX: *mut u32 = 0x0900_ff98 as *mut u32;
const HEARTBEAT: *mut u32 = 0x0900_ff9c as *mut u32;
const MAIN_MAGIC: u32 = 0x5753_4d31; // "WSM1"

#[unsafe(naked)]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
pub unsafe extern "C" fn _start() -> ! {
    naked_asm!(
        "ldr sp, =0xfff1ff00",
        "b {main}",
        main = sym rust_main,
    );
}

#[unsafe(no_mangle)]
extern "C" fn rust_main() -> ! {
    unsafe {
        write_volatile(MAILBOX, MAIN_MAGIC);
        write_volatile(HEARTBEAT, _start as *const () as usize as u32);
    }
    loop {
        unsafe {
            let value = read_volatile(HEARTBEAT);
            write_volatile(HEARTBEAT, value.wrapping_add(1));
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
