#![no_std]
#![no_main]

use core::arch::naked_asm;
use core::panic::PanicInfo;
use core::ptr::{read_volatile, write_volatile};

unsafe extern "C" {
    static __diagnostic_main_mailbox: u8;
    static __diagnostic_main_heartbeat: u8;
}

#[inline(always)]
fn mailbox() -> *mut u32 {
    (&raw const __diagnostic_main_mailbox).cast_mut().cast()
}

#[inline(always)]
fn heartbeat() -> *mut u32 {
    (&raw const __diagnostic_main_heartbeat).cast_mut().cast()
}

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
        write_volatile(mailbox(), MAIN_MAGIC);
        write_volatile(heartbeat(), _start as *const () as usize as u32);
    }
    loop {
        unsafe {
            let value = read_volatile(heartbeat());
            write_volatile(heartbeat(), value.wrapping_add(1));
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
