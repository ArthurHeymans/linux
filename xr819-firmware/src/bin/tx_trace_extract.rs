#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::ptr::write_volatile;

// Retained packet-RAM TX tracing was removed because this diagnostic overlay
// aliases active HIF storage. This extractor is intentionally lifecycle-only.
unsafe extern "C" {
    static __diagnostic_download_control: u8;
    static __diagnostic_checkpoint_base: u8;
}

#[inline(always)]
fn download_debug() -> *mut u32 {
    (&raw const __diagnostic_download_control).cast_mut().cast()
}

#[inline(always)]
fn checkpoint_debug() -> *mut u32 {
    (&raw const __diagnostic_checkpoint_base).cast_mut().cast()
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    unsafe {
        for index in 0..10 {
            write_volatile(download_debug().add(index), 0);
            if index < 4 {
                write_volatile(checkpoint_debug().add(index), 0);
            }
        }
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
