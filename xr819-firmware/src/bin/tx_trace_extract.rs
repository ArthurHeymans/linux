#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::ptr::write_volatile;

// Retained packet-RAM TX tracing was removed because this address is exactly
// host HIF request buffer 18. This extractor is intentionally inert.
const DOWNLOAD_DEBUG: *mut u32 = 0x0900_ff80 as *mut u32;
const CHECKPOINT_DEBUG: *mut u32 = 0x0900_fd00 as *mut u32;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    unsafe {
        for index in 0..10 {
            write_volatile(DOWNLOAD_DEBUG.add(index), 0);
            if index < 4 {
                write_volatile(CHECKPOINT_DEBUG.add(index), 0);
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
