#![no_std]
#![no_main]

use core::arch::naked_asm;
use core::mem::size_of;
use core::panic::PanicInfo;
use xr819_firmware::configuration;
use xr819_firmware::hif::Transport;
use xr819_firmware::phy::{initialize_mac_core_mode0, initialize_mac_software_state};
use xr819_firmware::platform::{
    initialize_runtime_state, prepare_dma_and_clocks, prepare_high_platform_support,
    prepare_main_control, prepare_memory_and_interrupts, prepare_packet_dma,
    register_post_activation_interrupts, try_activate_hif, wait_for_host_download_completion,
};
use xr819_firmware::scan;
use xr819_firmware::wsm::{
    CONFIGURATION_REQ_ID, ConfigurationRequest, START_SCAN_REQ_ID, StartScanRequest,
    StartupIndication, encode_configuration_response, encode_scan_complete_indication,
    encode_status_response,
};

unsafe extern "C" {
    static mut __bss_start: u32;
    static mut __bss_end: u32;
}

#[unsafe(no_mangle)]
#[used]
static STARTUP_DEBUG_STAGE: u32 = u32::MAX;

unsafe fn clear_rust_bss() {
    let mut address = (&raw mut __bss_start) as usize;
    let end = (&raw mut __bss_end) as usize;
    while address < end {
        unsafe { (address as *mut u32).write_volatile(0) };
        address += size_of::<u32>();
    }
}

fn debug_stop(stage: u32, marker: u32) {
    let selected = unsafe { (&raw const STARTUP_DEBUG_STAGE).read_volatile() };
    if selected == stage {
        unsafe { (0x0900_ff98 as *mut u32).write_volatile(marker) };
        loop {
            core::hint::spin_loop();
        }
    }
}

#[unsafe(naked)]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
pub extern "C" fn _start() -> ! {
    naked_asm!(
        // Vendor reset startup keeps all mode stacks below 0x0400c000. A stack
        // in the 0xfffxxxxx window becomes unavailable at the exact
        // 0x0aa80004 = 0x200 transition performed by 0x00000bb0.
        "ldr r0, =0x0400c000",
        "mov sp, r0",
        "b {main}",
        main = sym rust_main,
    );
}

#[unsafe(no_mangle)]
extern "C" fn rust_main() -> ! {
    unsafe { clear_rust_bss() };
    initialize_runtime_state();
    if !wait_for_host_download_completion(10_000_000) {
        unsafe {
            (0x0900_ff98 as *mut u32).write_volatile(0x444c_3f3f);
            (0x0900_ff9c as *mut u32).write_volatile((0x0400_1428 as *const u32).read_volatile());
        }
        loop {
            core::hint::spin_loop();
        }
    }

    debug_stop(0, 0x5354_4700);
    unsafe { (0x0900_ff98 as *mut u32).write_volatile(0x504c_5431) };
    prepare_main_control();
    debug_stop(1, 0x5354_4701);
    prepare_memory_and_interrupts();
    debug_stop(2, 0x5354_4702);
    unsafe { (0x0900_ff98 as *mut u32).write_volatile(0x504c_5432) };
    prepare_high_platform_support();
    debug_stop(3, 0x5354_4703);
    unsafe { (0x0900_ff98 as *mut u32).write_volatile(0x4849_4731) };
    prepare_dma_and_clocks();
    debug_stop(4, 0x5354_4704);
    unsafe { (0x0900_ff98 as *mut u32).write_volatile(0x504c_5433) };

    if !try_activate_hif(1_000_000) {
        unsafe {
            (0x0900_ff98 as *mut u32).write_volatile(0x4849_463f);
            (0x0900_ff9c as *mut u32).write_volatile((0x0ab0_0134 as *const u32).read_volatile());
        }
        loop {
            core::hint::spin_loop();
        }
    }

    debug_stop(5, 0x5354_4705);
    register_post_activation_interrupts();
    unsafe { (0x0900_ff98 as *mut u32).write_volatile(0x4849_4630) };
    let mut transport = unsafe { Transport::initialize() };
    debug_stop(6, 0x5354_4706);

    // Vendor 0x9ac calls packet-DMA initialization immediately after 0x94c.
    prepare_packet_dma();
    debug_stop(8, 0x5354_4708);

    // Vendor 0x16d24 performs the complete 0x16ac6 software and hardware
    // setup before setting MAC control bit 11. Policy remains inactive until a
    // real channel request is serviced.
    unsafe {
        initialize_mac_software_state();
        initialize_mac_core_mode0();
    }
    debug_stop(9, 0x5354_4709);

    let buffer = unsafe { transport.output_buffer() };
    let length = StartupIndication {
        input_buffers: 30,
        input_buffer_size: 1632,
        status: 0,
        hardware_id: 7,
        hardware_sub_id: 9,
        firmware_capabilities: 3,
        firmware_type: 2,
        firmware_api: 1,
        firmware_build: 1,
        firmware_version: 1,
        label: b"XR819 open Rust WSM",
        config: [0; 4],
    }
    .encode(buffer)
    .unwrap_or(0);

    if length != 0 {
        unsafe { transport.publish(length as u16) };
    }

    loop {
        let _ = transport.service_interrupt();

        // Service firmware-owned work before accepting another host command.
        // This preserves response-before-indication ordering without allowing a
        // queued follow-up command to overtake scan completion.
        if let Some(completion) = scan::service() {
            let output = unsafe { transport.output_buffer() };
            if let Ok(length) = encode_scan_complete_indication(
                completion.status,
                completion.psm,
                completion.num_channels,
                completion.vendor_field,
                output,
            ) {
                unsafe { transport.publish(length as u16) };
            }
        }

        if let Some(request) = transport.poll_request() {
            let output = unsafe { transport.output_buffer() };
            let response_length = if request.id == CONFIGURATION_REQ_ID {
                let station_id = ConfigurationRequest::parse(request.payload)
                    .ok()
                    .and_then(|configuration_request| {
                        configuration::retain(&configuration_request).ok()?;
                        configuration::snapshot().map(|snapshot| snapshot.station_id)
                    })
                    .unwrap_or([0; 6]);
                encode_configuration_response(station_id, output)
            } else if request.id == START_SCAN_REQ_ID {
                let status = match StartScanRequest::parse(request.payload) {
                    Ok(request) => match scan::begin(&request) {
                        Ok(()) => 0,
                        Err(scan::ScanError::Busy) => 4,
                        Err(_) => 2,
                    },
                    Err(_) => 2,
                };
                encode_status_response(request.id | 0x0400, status, output)
            } else {
                encode_status_response(request.id | 0x0400, 0, output)
            };
            if let Ok(length) = response_length {
                unsafe { transport.publish(length as u16) };
            }
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
