#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::ptr::{read_volatile, write_volatile};

const TRACE_SOURCE: *const u32 = 0x0900_fd20 as *const u32;
const DOWNLOAD_DEBUG: *mut u32 = 0x0900_ff80 as *mut u32;
const CHECKPOINT_DEBUG: *mut u32 = 0x0900_fd00 as *mut u32;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    unsafe {
        // The failed probe path already dumps ten words at 0x0900ff80 and
        // four words at 0x0900fd00. Mirror the retained Rust TX execution
        // trace into both existing diagnostic windows, then deliberately
        // remain silent so the host reaches that bounded timeout path.
        for index in 0..10 {
            let value = read_volatile(TRACE_SOURCE.add(index));
            write_volatile(DOWNLOAD_DEBUG.add(index), value);
            if index < 4 {
                write_volatile(CHECKPOINT_DEBUG.add(index), value);
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
