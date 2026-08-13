#![no_std]
#![no_main]

use core::arch::{global_asm, naked_asm};
use core::mem::size_of;
use core::panic::PanicInfo;
use xr819_firmware::configuration;
use xr819_firmware::crypto;
use xr819_firmware::hif::{SHARED_BUFFER_SIZE, Transport};
#[cfg(feature = "join-sta-experiment")]
use xr819_firmware::join;
use xr819_firmware::mac;
use xr819_firmware::phy::{initialize_mac_core_mode0, initialize_mac_software_state};
use xr819_firmware::platform::{
    enable_packet_controller, initialize_runtime_state, prepare_dma_and_clocks,
    prepare_high_platform_support, prepare_mac_receive_hardware, prepare_main_control,
    prepare_memory_and_interrupts, prepare_packet_dma, program_station_address,
    register_packet_dma_interrupts, register_post_activation_interrupts, try_activate_hif,
    wait_for_host_download_completion,
};
use xr819_firmware::radio;
use xr819_firmware::rate_policy;
use xr819_firmware::scan;
#[cfg(feature = "tcm-size-diagnostic")]
use xr819_firmware::tcm;
use xr819_firmware::tx;
use xr819_firmware::vif;
#[cfg(feature = "join-sta-experiment")]
use xr819_firmware::wsm::JoinRequest;
use xr819_firmware::wsm::{
    ADD_KEY_REQ_ID, AddKeyRequest, CONFIGURATION_REQ_ID, ConfigurationRequest, EDCA_PARAMS_REQ_ID,
    EdcaParameters, JOIN_REQ_ID, READ_MIB_REQ_ID, REMOVE_KEY_REQ_ID, RESET_REQ_ID,
    RemoveKeyRequest, ResetRequest, START_SCAN_REQ_ID, STATUS_FAILURE, StartScanRequest,
    StartupIndication, TX_QUEUE_PARAMS_REQ_ID, TX_REQ_ID, TxPowerRange, TxQueueParameters,
    TxRequest, WRITE_MIB_REQ_ID, WriteMibRequest, encode_configuration_response,
    encode_join_complete_indication, encode_join_response, encode_read_mib_data_response,
    encode_read_mib_response, encode_scan_complete_indication, encode_status_response,
    encode_tx_confirm, encode_tx_confirm_details, encode_xr819_tx_confirm,
    encode_xr819_tx_confirm_details, encode_xr819_tx_confirm_retry_details,
};
use xr819_firmware::wsm_profile;
#[cfg(feature = "vendor-host-tx-foundation")]
use xr819_firmware::{host_tx_diagnostics, host_tx_driver::HostTxDriver};

// The ARM9 exception vectors execute in ARM state even though the firmware
// body is Thumb. Each terminal veneer saves the unmodified shared registers on
// its preinitialized mode stack before committing the fixed noinit record.
#[cfg(target_arch = "arm")]
global_asm!(
    r#"
    .syntax unified
    .arm
    .section .vectors,"ax",%progbits
    .align 2
    .global __xr819_vectors
__xr819_vectors:
    ldr pc, [pc, #24]
    ldr pc, [pc, #24]
    ldr pc, [pc, #24]
    ldr pc, [pc, #24]
    ldr pc, [pc, #24]
    ldr pc, [pc, #24]
    ldr pc, [pc, #24]
    ldr pc, [pc, #24]
    .word xr819_reset_entry
    .word xr819_exception_undef
    .word xr819_exception_svc
    .word xr819_exception_prefetch_abort
    .word xr819_exception_data_abort
    .word xr819_exception_reserved
    .word xr819_exception_irq
    .word xr819_exception_fiq

    .global xr819_reset_entry
xr819_reset_entry:
    mrs r0, cpsr
    bic r0, r0, #31
    orr r0, r0, #192

    orr r1, r0, #27
    msr cpsr_c, r1
    ldr sp, =0x0400b100
    orr r1, r0, #23
    msr cpsr_c, r1
    ldr sp, =0x0400b200
    orr r1, r0, #19
    msr cpsr_c, r1
    ldr sp, =0x0400b300
    orr r1, r0, #18
    msr cpsr_c, r1
    ldr sp, =0x0400b400
    orr r1, r0, #17
    msr cpsr_c, r1
    ldr sp, =0x0400b500
    orr r1, r0, #31
    msr cpsr_c, r1
    ldr sp, =0x0400c000
    ldr r0, =_start
    bx r0

    .macro XR819_EXCEPTION name, kind
    .global \name
\name:
    stmdb sp!, {{r0-r12, lr}}
    mov r0, #\kind
    b xr819_exception_common
    .endm

    XR819_EXCEPTION xr819_exception_undef, 1
    XR819_EXCEPTION xr819_exception_svc, 2
    XR819_EXCEPTION xr819_exception_prefetch_abort, 3
    XR819_EXCEPTION xr819_exception_data_abort, 4
    XR819_EXCEPTION xr819_exception_reserved, 5
    XR819_EXCEPTION xr819_exception_irq, 6
    XR819_EXCEPTION xr819_exception_fiq, 7

xr819_exception_common:
    mrs r10, cpsr
    orr r11, r10, #192
    msr cpsr_c, r11

    ldr r1, =XR819_EXCEPTION_RECORD
    mov r2, #0
    str r2, [r1, #0]
    str r0, [r1, #4]

    add r2, r1, #8
    mov r3, sp
    mov r4, #13
1:
    ldr r5, [r3], #4
    str r5, [r2], #4
    subs r4, r4, #1
    bne 1b

    add r5, sp, #56
    str r5, [r1, #60]
    ldr r6, [sp, #52]
    str r6, [r1, #64]
    mrs r7, spsr
    str r7, [r1, #68]
    str r10, [r1, #72]

    cmp r0, #4
    subeq r8, r6, #8
    beq 2f
    tst r7, #32
    subne r8, r6, #2
    subeq r8, r6, #4
2:
    str r8, [r1, #76]
    mrc p15, 0, r8, c5, c0, 0
    str r8, [r1, #80]
    mrc p15, 0, r8, c6, c0, 0
    str r8, [r1, #84]

    ldr r8, =0x58434651
    str r8, [r1, #0]
    ldr r3, =xr819_exception_terminal
    blx r3
3:
    b 3b
    "#
);

// Explicit rollback boundary for scan-owned active probe TX. This feature is
// enabled by default after repeated cross-scan hardware validation; building
// with `--no-default-features` retains the passive fallback.
const ENABLE_SINGLE_PROBE_EXPERIMENT: bool = cfg!(feature = "probe-tx-experiment");

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
        let _ = marker;
        loop {
            core::hint::spin_loop();
        }
    }
}

fn encode_debug_event(event_id: u32, data: u32, output: &mut [u8]) -> Option<usize> {
    if output.len() < 12 {
        return None;
    }
    output[..2].copy_from_slice(&12_u16.to_le_bytes());
    output[2..4].copy_from_slice(&0x0805_u16.to_le_bytes());
    output[4..8].copy_from_slice(&event_id.to_le_bytes());
    output[8..12].copy_from_slice(&data.to_le_bytes());
    Some(12)
}

#[cfg(feature = "join-sta-experiment")]
unsafe fn service_management_request(
    events: &mut tx::MacEventQueue,
    request: &TxRequest<'_>,
    if_id: u8,
    output: &mut [u8],
    publish_response: &mut bool,
) -> Result<usize, xr819_firmware::wsm::Error> {
    match unsafe { tx::service_host_management_tx(events, Some((request, if_id)), 0) } {
        tx::HostManagementTxReport::Published {
            packet_id,
            bisect_stage,
        } => {
            *publish_response = true;
            if bisect_stage != 0 {
                if join::uses_cw1200_wsm() {
                    encode_tx_confirm_details(packet_id, STATUS_FAILURE, 0, bisect_stage, output)
                } else {
                    encode_xr819_tx_confirm_details(
                        packet_id,
                        STATUS_FAILURE,
                        0,
                        bisect_stage,
                        output,
                    )
                }
            } else {
                let edca = unsafe { (0x09c0_0e64 as *const u32).read_volatile() };
                let quantum0 = unsafe { (0x09c0_0e70 as *const u32).read_volatile() };
                let quantum1 = unsafe { (0x09c0_0e74 as *const u32).read_volatile() };
                let metadata = unsafe { (0x0900_8008 as *const u8).read_volatile() };
                let secondary = unsafe { (0x0900_7bc0 as *const u8).read_volatile() };
                let event_id = 0x5852_0000 | (edca & 0xffff);
                let data = (quantum0 & 0xff)
                    | ((quantum1 & 0xff) << 8)
                    | (u32::from(metadata) << 16)
                    | (u32::from(secondary) << 24);
                encode_debug_event(event_id, data, output)
                    .ok_or(xr819_firmware::wsm::Error::Truncated)
            }
        }
        tx::HostManagementTxReport::Failed { packet_id, .. } => {
            if join::uses_cw1200_wsm() {
                encode_tx_confirm(packet_id, STATUS_FAILURE, output)
            } else {
                encode_xr819_tx_confirm(packet_id, STATUS_FAILURE, output)
            }
        }
        _ => {
            *publish_response = false;
            encode_tx_confirm(0, STATUS_FAILURE, output)
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
        loop {
            core::hint::spin_loop();
        }
    }

    debug_stop(0, 0x5354_4700);
    prepare_main_control();
    debug_stop(1, 0x5354_4701);
    prepare_memory_and_interrupts();
    debug_stop(2, 0x5354_4702);
    prepare_high_platform_support();
    debug_stop(3, 0x5354_4703);
    prepare_dma_and_clocks();
    debug_stop(4, 0x5354_4704);

    if !try_activate_hif(1_000_000) {
        loop {
            core::hint::spin_loop();
        }
    }

    debug_stop(5, 0x5354_4705);
    register_post_activation_interrupts();
    let mut transport = unsafe { Transport::initialize() };
    let mut mac_events = unsafe { tx::MacEventQueue::claim() };
    debug_stop(6, 0x5354_4706);

    // Vendor 0x9ac calls packet-DMA initialization immediately after 0x94c.
    prepare_packet_dma();
    prepare_mac_receive_hardware();
    unsafe { radio::initialize() };
    register_packet_dma_interrupts();
    debug_stop(8, 0x5354_4708);

    // Vendor 0x16d24 performs the complete 0x16ac6 software and hardware
    // setup before setting MAC control bit 11. Policy remains inactive until a
    // real channel request is serviced.
    unsafe {
        initialize_mac_software_state();
        initialize_mac_core_mode0();
        if let Err(_error) = mac::initialize_vendor_startup_state(1_000_000) {
            loop {
                core::hint::spin_loop();
            }
        }
        mac::initialize_tx_pipe_state();
        // Vendor `0x5a8` initializes this pool only after `0x14c` returns.
        xr819_firmware::tx::initialize_internal_pool();
    }
    // Vendor `0xc80` is the final hardware-visible step before `0x158fc`.
    enable_packet_controller();
    debug_stop(9, 0x5354_4709);

    let buffer = unsafe { transport.output_buffer() };
    let length = StartupIndication {
        input_buffers: 30,
        input_buffer_size: 1632,
        // Mainline rejects nonzero startup status and firmware types above 4.
        status: 0,
        hardware_id: 7,
        hardware_sub_id: 9,
        // Bit 0 advertises 2.4 GHz. XR819 has no 5 GHz radio, so bit 1 must
        // remain clear or cw1200 exposes a phantom 5 GHz band.
        firmware_capabilities: 1,
        firmware_type: 2,
        firmware_api: 1,
        firmware_build: 1,
        firmware_version: 1,
        label: wsm_profile::STARTUP_LABEL,
        config: [0; 4],
    }
    .encode(buffer)
    .unwrap_or(0);

    if length != 0 {
        transport.publish(length as u16);
    }

    let mut pending_scan_completion: Option<scan::ScanCompletion> = None;
    let mut pending_join_complete: Option<u32> = None;
    let mut pending_tx_confirmation: Option<(u32, u32, u8, u8)> = None;
    let mut pending_tx_debug_event: Option<(u32, u32)> = None;
    // Vendor timestamp at which the output path was first seen blocked.
    #[cfg(all(feature = "hif-stall-dump", target_arch = "arm"))]
    let mut output_blocked_since: Option<u32> = None;
    // Vendor timestamp of the first class-0 publication.
    #[cfg(all(feature = "class0-first-frame-dump", target_arch = "arm"))]
    let mut first_publication_at: Option<u32> = None;
    // Vendor timestamp of the last 200ms TX pipe watchdog tick.
    #[cfg(all(feature = "pipe-watchdog", target_arch = "arm"))]
    let mut last_watchdog_tick: u32 = 0;
    // Command responses and retained class-0 confirmations are copied into
    // their original 1632-byte request buffers before publication. This buffer
    // is scratch only and is never exposed through a HIF descriptor.
    let mut response_scratch = [0_u8; SHARED_BUFFER_SIZE];
    #[cfg(feature = "vendor-host-tx-foundation")]
    let mut host_tx_driver = HostTxDriver::new();
    loop {
        #[cfg(all(feature = "vendor-host-tx-diagnostics", target_arch = "arm"))]
        unsafe {
            xr819_firmware::hif::validate_tx_boundary(0x20, 0xff, 0xff, 0, 0);
        }
        let _ = transport.service_interrupt();
        #[cfg(all(feature = "vendor-host-tx-diagnostics", target_arch = "arm"))]
        unsafe {
            xr819_firmware::hif::validate_tx_boundary(0x21, 0xff, 0xff, 0, 0);
        }
        // Vendor runs a 200 ms timer (`FUN_00003bac`) that decrements each
        // programmed pipe's watchdog byte and recovers a pipe that has stayed
        // armed without completing. Without it an armed pipe is unrecoverable:
        // retirement only ever runs from a delivered status, and the MAC stops
        // delivering statuses for a wedged pipe.
        #[cfg(all(feature = "pipe-watchdog", target_arch = "arm"))]
        {
            let now = unsafe { xr819_firmware::vendor_host_tx::vendor_timer_now() };
            if now.wrapping_sub(last_watchdog_tick) >= 200_000 {
                last_watchdog_tick = now;
                unsafe {
                    tx::service_pipe_watchdog_tick_runtime();
                }
            }
        }

        // A ready host request may be a synchronous command. Preserve one
        // output descriptor for it instead of allowing asynchronous events or
        // TX confirmations to starve the command lane.
        let host_request_waiting = transport.request_available();

        // Every host-visible channel dies at the same moment under load: the
        // counters MIB returns zeros and indications stop, while the driver
        // still reports the BH alive. Both need a free shared slot, so catch
        // the moment the output path has been blocked continuously and report
        // the accounting through the emergency descriptor, which does not need
        // a shared slot and is therefore still deliverable.
        #[cfg(all(feature = "hif-stall-dump", target_arch = "arm"))]
        {
            let now = unsafe { xr819_firmware::vendor_host_tx::vendor_timer_now() };
            if transport.output_available() {
                output_blocked_since = None;
            } else {
                let since = *output_blocked_since.get_or_insert(now);
                if now.wrapping_sub(since) > 3_000_000 {
                    let snapshot = transport.stall_snapshot();
                    unsafe {
                        xr819_firmware::hif::publish_terminal_exception(
                            [
                                0x4849_5354, // "HIST"
                                now.wrapping_sub(since),
                                snapshot[0],
                                snapshot[1],
                                snapshot[2],
                                snapshot[3],
                                snapshot[4],
                                snapshot[5],
                                snapshot[6],
                                snapshot[7],
                                snapshot[8],
                                snapshot[9],
                                snapshot[10],
                                snapshot[11],
                                snapshot[12],
                                snapshot[13],
                                0,
                                0,
                            ],
                            b"xr819-hif-output-stalled",
                        );
                    }
                    loop {
                        core::hint::spin_loop();
                    }
                }
            }
        }

        // Report the fate of the first class-0 frame over the emergency
        // descriptor. The counters MIB cannot answer this: it is readable only
        // before any data frame exists, and the first published frame that the
        // MAC refuses disables the host command lane entirely.
        #[cfg(all(feature = "class0-first-frame-dump", target_arch = "arm"))]
        {
            let counters = unsafe { xr819_firmware::host_tx_diagnostics::counters_snapshot() };
            let published = counters[xr819_firmware::host_tx_diagnostics::counter::PUBLISHED];
            let now = unsafe { xr819_firmware::vendor_host_tx::vendor_timer_now() };
            if published > 0 {
                let since = *first_publication_at.get_or_insert(now);
                // The first frames succeed, so a short window only shows a
                // healthy funnel. The late window instead lands inside the
                // flood, where the path has wedged.
                let window = if cfg!(feature = "class0-late-frame-dump") {
                    20_000_000
                } else {
                    500_000
                };
                if now.wrapping_sub(since) > window {
                    unsafe {
                        xr819_firmware::hif::publish_terminal_exception(
                            [
                                0x4330_4646, // "C0FF"
                                now.wrapping_sub(since),
                                counters[0],
                                counters[1],
                                counters[2],
                                counters[3],
                                counters[4],
                                counters[5],
                                counters[6],
                                counters[7],
                                counters[8],
                                counters[9],
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                            ],
                            b"xr819-class0-first-frame",
                        );
                    }
                    loop {
                        core::hint::spin_loop();
                    }
                }
            }
        }

        #[cfg(feature = "vendor-host-tx-foundation")]
        if let Some(event) = unsafe {
            host_tx_driver.service(
                &mut mac_events,
                !tx::host_management_runtime_active(),
                pending_tx_debug_event.is_none(),
            )
        } {
            pending_tx_debug_event = Some(event);
        }
        #[cfg(all(feature = "vendor-host-tx-diagnostics", target_arch = "arm"))]
        unsafe {
            xr819_firmware::hif::validate_tx_boundary(0x22, 0xff, 0xff, 0, 0);
        }

        #[cfg(feature = "vendor-host-tx-foundation")]
        let management_runtime_available = host_tx_driver.management_runtime_available();
        #[cfg(not(feature = "vendor-host-tx-foundation"))]
        let management_runtime_available = true;

        #[cfg(feature = "join-sta-experiment")]
        if management_runtime_available
            && pending_tx_confirmation.is_none()
            && let tx::HostManagementTxReport::Completed {
                packet_id,
                status,
                tx_rate,
                ack_failures,
            } = unsafe { tx::service_host_management_tx(&mut mac_events, None, 32) }
        {
            pending_tx_confirmation = Some((packet_id, status, tx_rate, ack_failures));
        }

        #[cfg(not(feature = "vendor-host-tx-foundation"))]
        if pending_tx_debug_event.is_none() {
            pending_tx_debug_event = tx::take_tx_debug_event();
        }

        // Push the class-0 lifecycle counters into the host-side trace. The
        // counters MIB stops answering under load, which is exactly when these
        // matter; indications keep flowing past that point.
        #[cfg(all(feature = "class0-lifecycle-counters", target_arch = "arm"))]
        if pending_tx_debug_event.is_none() {
            pending_tx_debug_event = unsafe {
                xr819_firmware::host_tx_diagnostics::take_counter_event(
                    xr819_firmware::vendor_host_tx::vendor_timer_now(),
                )
            };
        }

        if let Some((event_id, data)) = pending_tx_debug_event
            && !host_request_waiting
            && transport.output_available()
        {
            let output = unsafe { transport.output_buffer() };
            if let Some(length) = encode_debug_event(event_id, data, output) {
                pending_tx_debug_event = None;
                transport.publish(length as u16);
            }
        }

        if let Some(status) = pending_join_complete
            && !host_request_waiting
            && transport.output_available()
        {
            let output = unsafe { transport.output_buffer() };
            if let Ok(length) = encode_join_complete_indication(status, output) {
                pending_join_complete = None;
                transport.publish(length as u16);
            }
        }

        #[cfg(feature = "join-sta-experiment")]
        if let Some((packet_id, status, tx_rate, ack_failures)) = pending_tx_confirmation
            && !host_request_waiting
            && transport.output_available()
        {
            let output = unsafe { transport.output_buffer() };
            let encoded = if join::uses_cw1200_wsm() {
                encode_tx_confirm_details(packet_id, status, tx_rate, ack_failures, output)
            } else {
                encode_xr819_tx_confirm_details(packet_id, status, tx_rate, ack_failures, output)
            };
            if let Ok(length) = encoded {
                pending_tx_confirmation = None;
                transport.publish(length as u16);
            }
        }

        #[cfg(feature = "vendor-host-tx-foundation")]
        if let Some(confirmation) = host_tx_driver.confirmation()
            && !host_request_waiting
            && transport.response_available()
        {
            let encoded = if join::uses_cw1200_wsm() {
                encode_tx_confirm_details(
                    confirmation.packet_id,
                    confirmation.status,
                    confirmation.tx_rate,
                    confirmation.ack_failures,
                    &mut response_scratch,
                )
            } else {
                encode_xr819_tx_confirm_retry_details(
                    confirmation.packet_id,
                    confirmation.status,
                    confirmation.tx_rate,
                    confirmation.ack_failures,
                    confirmation.rate_try,
                    &mut response_scratch,
                )
            };
            if encoded.is_ok() {
                unsafe {
                    host_tx_diagnostics::capture_confirmation_identity(
                        confirmation.packet_id,
                        confirmation.context,
                        confirmation.status,
                        confirmation.ack_failures,
                    );
                }
            }
            if let Ok(length) = encoded
                && let Some(release) = unsafe { host_tx_driver.finish_confirmation() }
            {
                unsafe {
                    transport.publish_request_in_place(
                        release,
                        &response_scratch[..length],
                        length as u16,
                    );
                }
            }
        } else if host_tx_driver.confirmation().is_some() {
            unsafe { host_tx_diagnostics::trace(0x4854_4000, 0, 0) };
        }

        // Retain completion until a HIF descriptor is available. This prevents
        // a full ring from overwriting an unreclaimed zero-copy RX token.
        if pending_scan_completion.is_none() {
            pending_scan_completion = scan::service();
        }

        #[cfg(feature = "probe-tx-experiment")]
        {
            let mut probe_ssid = [0_u8; scan::MAX_SSID_LEN];
            let mut opportunity = scan::claim_probe_opportunity();
            let ssid_length =
                opportunity.and_then(|value| scan::copy_probe_ssid(value, &mut probe_ssid));
            if let (Some(value), None) = (opportunity, ssid_length) {
                opportunity = None;
                let _ = scan::fail_probe(value);
            }
            let report = unsafe {
                tx::service_guarded_probe_experiment(
                    &mut mac_events,
                    configuration::template_frame(),
                    opportunity,
                    &probe_ssid[..ssid_length.unwrap_or(0)],
                    32,
                )
            };
            match report {
                tx::ProbeExperimentReport::Published {
                    opportunity,
                    publication,
                } => {
                    let _ = (opportunity, publication);
                }
                tx::ProbeExperimentReport::Completed {
                    opportunity,
                    context,
                    status,
                } => {
                    if !scan::complete_probe(opportunity, status) {
                        loop {
                            core::hint::spin_loop();
                        }
                    }
                    let _ = context;
                }
                tx::ProbeExperimentReport::Failed { opportunity, error } => {
                    let _ = scan::fail_probe(opportunity);
                    let _ = error;
                }
                tx::ProbeExperimentReport::Idle | tx::ProbeExperimentReport::Servicing => {}
            }
        }

        if let Some(completion) = pending_scan_completion
            && !host_request_waiting
            && transport.output_available()
        {
            let output = unsafe { transport.output_buffer() };
            if let Ok(length) = encode_scan_complete_indication(
                completion.status,
                completion.psm,
                completion.num_channels,
                completion.vendor_field
                    | radio::diagnostic_word()
                    | if ENABLE_SINGLE_PROBE_EXPERIMENT {
                        tx::probe_experiment_diagnostic_word()
                    } else {
                        0
                    },
                output,
            ) {
                pending_scan_completion = None;
                transport.publish(length as u16);
            }
        }

        // Control indications and inbound host requests take priority over RX.
        // Otherwise a steady joined RX stream can consume the last firmware-to-
        // host descriptor every pass, preventing `poll_request()` from ever
        // detaching an ordinary TX request even though the driver accounts its
        // input buffer as used.
        if pending_scan_completion.is_none() && !host_request_waiting {
            if let (Some(if_id), Some(channel)) = (scan::active_interface(), scan::active_channel())
            {
                if transport.publication_available() {
                    if let Some(indication) = unsafe { radio::poll_scan_indication(if_id, channel) }
                    {
                        transport.publish_radio(indication);
                    }
                }
            } else if let Some(if_id) = vif::active_interface() {
                let channel = unsafe { vif::snapshot(if_id) }
                    .map(|state| state.channel)
                    .unwrap_or(0);
                if transport.publication_available()
                    && let Some(indication) =
                        unsafe { radio::poll_joined_indication(if_id, channel) }
                {
                    transport.publish_radio(indication);
                }
            } else {
                // Vendor RX processing never stops between scans. Recycle one
                // slot per cooperative pass so a later dwell cannot consume
                // idle-era beacons as if they had just arrived.
                let channel = unsafe { (0x0400_3a68 as *const u16).read_volatile() };
                unsafe { radio::discard_one_idle(channel) };
            }
        }

        // Vendor `hif_rx_process()` dispatches exactly one completed request
        // per invocation, then reschedules itself if another descriptor is
        // already ready. Keep that cooperative boundary instead of batching
        // request execution in one main-loop pass. Class 0 and class 6 also
        // converge on one vendor scheduler, so leave the host descriptor owned
        // by the transport while class-0 hardware publication is active rather
        // than entering the independent management publisher.
        if transport.publication_available()
            && management_runtime_available
            && let Some(request) = transport.poll_request()
        {
            let output = &mut response_scratch;
            let mut publish_response = true;
            let request_id = request.id;
            let request_if_id = request.if_id;
            unsafe {
                xr819_firmware::host_tx_diagnostics::record_hif_event(1, request_id, request_if_id);
            }
            let mut request_buffer = Some(request.buffer);
            let request_payload = request_buffer
                .as_ref()
                .expect("request buffer is present")
                .payload();
            let response_length = if request_if_id > 2 {
                encode_status_response(request_id | 0x0400, STATUS_FAILURE, output)
            } else if request_id == CONFIGURATION_REQ_ID {
                let configured = ConfigurationRequest::parse(request_payload).ok().and_then(
                    |configuration_request| {
                        configuration::retain(&configuration_request).ok()?;
                        Some((
                            configuration::snapshot()?.station_id,
                            configuration::tx_power_ranges()?,
                        ))
                    },
                );
                let (station_id, tx_power_ranges) = configured.unwrap_or((
                    [0; 6],
                    [
                        TxPowerRange {
                            min_power_level: -160,
                            max_power_level: 200,
                            stepping: 0,
                        },
                        TxPowerRange {
                            min_power_level: -160,
                            max_power_level: 200,
                            stepping: 0,
                        },
                    ],
                ));
                program_station_address(station_id);
                encode_configuration_response(station_id, tx_power_ranges, output)
            } else if request_id == START_SCAN_REQ_ID {
                let status = match StartScanRequest::parse(request_payload) {
                    Ok(scan_request) => {
                        #[cfg(feature = "probe-tx-experiment")]
                        let preparation: Result<(), ()> = Ok(());
                        #[cfg(not(feature = "probe-tx-experiment"))]
                        let preparation = if scan_request.num_probes == 0 {
                            Ok(())
                        } else {
                            let channel = scan_request
                                .channel(0)
                                .ok()
                                .and_then(|value| u8::try_from(value.number).ok());
                            let ssid = if scan_request.num_ssids == 0 {
                                Some(&[][..])
                            } else {
                                scan_request.ssid(0).ok()
                            };
                            match (channel, ssid) {
                                (Some(channel), Some(ssid)) => unsafe {
                                    tx::validate_probe_preparation(
                                        configuration::template_frame(),
                                        ssid,
                                        channel,
                                        request_if_id,
                                    )
                                    .map(|_| ())
                                    .map_err(|_| ())
                                },
                                _ => Err(()),
                            }
                        };
                        match preparation {
                            Ok(()) => match scan::begin(&scan_request, request_if_id) {
                                Ok(()) => 0,
                                Err(scan::ScanError::Busy) => 4,
                                Err(_) => 2,
                            },
                            Err(()) => 2,
                        }
                    }
                    Err(_) => 2,
                };
                encode_status_response(request_id | 0x0400, status, output)
            } else if request_id == TX_QUEUE_PARAMS_REQ_ID {
                let status = match TxQueueParameters::parse(request_payload) {
                    Ok(parameters) => {
                        configuration::retain_tx_queue(parameters);
                        0
                    }
                    Err(_) => 2,
                };
                encode_status_response(request_id | 0x0400, status, output)
            } else if request_id == EDCA_PARAMS_REQ_ID {
                let status = match EdcaParameters::parse(request_payload) {
                    Ok(parameters) => {
                        configuration::retain_edca(parameters);
                        unsafe {
                            vif::apply_edca(request_if_id, parameters)
                                .map(|_| 0)
                                .unwrap_or(2)
                        }
                    }
                    Err(_) => 2,
                };
                encode_status_response(request_id | 0x0400, status, output)
            } else if request_id == WRITE_MIB_REQ_ID {
                let status = match WriteMibRequest::parse(request_payload) {
                    Ok(request)
                        if request.mib_id == 0x1006
                            && request.data.len()
                                == if wsm_profile::CW1200_COMPATIBLE { 1 } else { 4 } =>
                    {
                        0
                    }
                    Ok(request)
                        if request.mib_id == rate_policy::MIB_ID_SET_TX_RATE_RETRY_POLICY =>
                    {
                        rate_policy::install(request.data)
                            .map(|()| 0)
                            .unwrap_or(STATUS_FAILURE)
                    }
                    Ok(request)
                        if configuration::retain_interface_mib(request.mib_id, request.data) =>
                    {
                        0
                    }
                    _ => STATUS_FAILURE,
                };
                encode_status_response(request_id | 0x0400, status, output)
            } else if request_id == READ_MIB_REQ_ID {
                let mib_id = request_payload
                    .get(..2)
                    .map(|value| u16::from_le_bytes([value[0], value[1]]))
                    .unwrap_or(0);
                if mib_id == 0x100c {
                    let diagnostics = radio::diagnostics();
                    let (_scan_channels, scan_max_time) = scan::diagnostic_plan();
                    let (dwell_arm, dwell_deadline, dwell_now, _dwell_waits) =
                        scan::diagnostic_dwell();
                    let (_scan_status, scan_error) = scan::diagnostic_error();
                    let _iq = xr819_firmware::phy::iq_hardware_diagnostics();
                    #[allow(unused_mut)]
                    let mut values = [
                        diagnostics.producer_changes,
                        diagnostics.bad_magic,
                        diagnostics.valid_slots,
                        diagnostics.indications,
                        diagnostics.malformed_slots,
                        diagnostics.filtered_frames,
                        diagnostics.oversized_frames,
                        diagnostics.released_slots,
                        u32::from(diagnostics.last_slot_length)
                            | (u32::from(diagnostics.last_frame_control) << 16),
                        u32::from(diagnostics.last_channel)
                            | (u32::from(diagnostics.last_active_channel) << 16),
                        diagnostics.last_trailer_word,
                        scan_max_time,
                        dwell_arm,
                        dwell_deadline,
                        dwell_now,
                        unsafe { (0x0400_1ae4 as *const u32).read_volatile() },
                        unsafe {
                            u32::from((0x0400_9959 as *const u8).read_volatile())
                                | (u32::from((0x0400_995c as *const u8).read_volatile()) << 8)
                                | (u32::from((0x0400_99c4 as *const u8).read_volatile()) << 16)
                                | (u32::from((0x0400_998c as *const u8).read_volatile()) << 24)
                        },
                        scan_error,
                        unsafe { (0x09c0_0600 as *const u32).read_volatile() },
                        unsafe { (0x09c0_0604 as *const u32).read_volatile() },
                        unsafe { (0x09c0_0608 as *const u32).read_volatile() },
                        if ENABLE_SINGLE_PROBE_EXPERIMENT {
                            tx::probe_experiment_diagnostic_value()
                        } else {
                            unsafe { (0x0940_0000 as *const u32).read_volatile() }
                        },
                    ];
                    #[cfg(feature = "tcm-size-diagnostic")]
                    {
                        let info = tcm::read_region_info();
                        values[17] = info.tcm_type_register;
                        values[18] = info.dtcm_register;
                        values[19] = info.itcm_register;
                        values[20] = tcm::size_kib(info.dtcm_register).unwrap_or(0xffff)
                            | (tcm::size_kib(info.itcm_register).unwrap_or(0xffff) << 16);
                    }
                    #[cfg(feature = "vendor-host-tx-foundation")]
                    host_tx_diagnostics::populate_counters(&mut values, &transport);
                    let mut data = [0_u8; 88];
                    for (index, value) in values.into_iter().enumerate() {
                        data[index * 4..index * 4 + 4].copy_from_slice(&value.to_le_bytes());
                    }
                    encode_read_mib_data_response(0, mib_id, &data, output)
                } else {
                    encode_read_mib_response(STATUS_FAILURE, mib_id, output)
                }
            } else if request_id == ADD_KEY_REQ_ID {
                let status = AddKeyRequest::parse(request_payload)
                    .ok()
                    .and_then(|key| crypto::add_key(request_if_id, &key).ok())
                    .map(|()| 0)
                    .unwrap_or(STATUS_FAILURE);
                encode_status_response(request_id | 0x0400, status, output)
            } else if request_id == REMOVE_KEY_REQ_ID {
                let status = RemoveKeyRequest::parse(request_payload)
                    .ok()
                    .and_then(|key| crypto::remove_key(key.index).ok())
                    .map(|()| 0)
                    .unwrap_or(STATUS_FAILURE);
                encode_status_response(request_id | 0x0400, status, output)
            } else if request_id == RESET_REQ_ID {
                let status = match ResetRequest::parse(request_payload) {
                    Ok(_) => {
                        #[cfg(feature = "vendor-host-tx-foundation")]
                        unsafe {
                            host_tx_driver.reset();
                        }
                        #[cfg(feature = "join-sta-experiment")]
                        {
                            if unsafe { join::reset(request_if_id) } {
                                0
                            } else {
                                STATUS_FAILURE
                            }
                        }
                        #[cfg(not(feature = "join-sta-experiment"))]
                        0
                    }
                    Err(_) => STATUS_FAILURE,
                };
                encode_status_response(request_id | 0x0400, status, output)
            } else if request_id == JOIN_REQ_ID {
                #[cfg(feature = "join-sta-experiment")]
                let status = match JoinRequest::parse(request_payload) {
                    Ok(join_request) => unsafe {
                        join::activate_sta(request_if_id, &join_request)
                            .map(|_| 0)
                            .unwrap_or(STATUS_FAILURE)
                    },
                    Err(_) => STATUS_FAILURE,
                };
                #[cfg(not(feature = "join-sta-experiment"))]
                let status = STATUS_FAILURE;
                if status == 0
                    && request_payload
                        .get(0x0f)
                        .is_some_and(|flags| flags & 0x20 != 0)
                {
                    pending_join_complete = Some(0);
                }
                encode_join_response(status, -160, 200, output)
            } else if request_id == TX_REQ_ID {
                #[cfg(feature = "vendor-host-tx-foundation")]
                unsafe {
                    host_tx_diagnostics::trace(
                        0x4854_0004,
                        u32::from(request_if_id)
                            | (u32::try_from(request_payload.len()).unwrap_or(u32::MAX) << 8),
                        0,
                    );
                }
                #[cfg(feature = "join-sta-experiment")]
                {
                    match TxRequest::parse(request_payload) {
                        Ok(tx_request) => {
                            #[cfg(feature = "vendor-host-tx-foundation")]
                            unsafe {
                                let frame_control =
                                    u16::from_le_bytes([tx_request.frame[0], tx_request.frame[1]]);
                                host_tx_diagnostics::trace(
                                    0x4854_0005,
                                    u32::from(frame_control)
                                        | (u32::try_from(tx_request.frame.len())
                                            .unwrap_or(u32::MAX)
                                            << 16),
                                    u32::from(tx_request.is_unicast_data())
                                        | (u32::from(tx_request.is_unicast_eapol()) << 1),
                                );
                            }
                            #[cfg(feature = "vendor-host-tx-foundation")]
                            if tx_request.is_unicast_data() && !tx_request.is_unicast_eapol() {
                                let packet_id = tx_request.packet_id;
                                let admitted = unsafe {
                                    host_tx_driver.admit(
                                        request_buffer.take().expect("request buffer is present"),
                                        request_if_id,
                                        &mut transport,
                                    )
                                };
                                if admitted {
                                    publish_response = false;
                                    Ok(0)
                                } else {
                                    unsafe {
                                        host_tx_diagnostics::trace(0x4854_00e1, packet_id, 0);
                                    }
                                    if join::uses_cw1200_wsm() {
                                        encode_tx_confirm(packet_id, STATUS_FAILURE, output)
                                    } else {
                                        encode_xr819_tx_confirm(packet_id, STATUS_FAILURE, output)
                                    }
                                }
                            } else {
                                unsafe {
                                    service_management_request(
                                        &mut mac_events,
                                        &tx_request,
                                        request_if_id,
                                        output,
                                        &mut publish_response,
                                    )
                                }
                            }
                            #[cfg(not(feature = "vendor-host-tx-foundation"))]
                            unsafe {
                                service_management_request(
                                    &mut mac_events,
                                    &tx_request,
                                    request_if_id,
                                    output,
                                    &mut publish_response,
                                )
                            }
                        }
                        Err(_) => {
                            let packet_id = request_payload
                                .get(..4)
                                .map(|value| {
                                    u32::from_le_bytes([value[0], value[1], value[2], value[3]])
                                })
                                .unwrap_or(0);
                            if join::uses_cw1200_wsm() {
                                encode_tx_confirm(packet_id, STATUS_FAILURE, output)
                            } else {
                                encode_xr819_tx_confirm(packet_id, STATUS_FAILURE, output)
                            }
                        }
                    }
                }
                #[cfg(not(feature = "join-sta-experiment"))]
                {
                    let packet_id = request_payload
                        .get(..4)
                        .map(|value| u32::from_le_bytes([value[0], value[1], value[2], value[3]]))
                        .unwrap_or(0);
                    encode_xr819_tx_confirm(packet_id, STATUS_FAILURE, output)
                }
            } else {
                // Do not report success for commands whose state effects are
                // not implemented. A complete status word lets cw1200 fail the
                // command cleanly instead of proceeding on false assumptions.
                encode_status_response(request_id | 0x0400, STATUS_FAILURE, output)
            };
            if publish_response && let Ok(length) = response_length {
                let response_id = u16::from_le_bytes([output[2], output[3]]) & 0x1fff;
                unsafe {
                    xr819_firmware::host_tx_diagnostics::record_hif_event(
                        2,
                        response_id,
                        length as u8,
                    );
                }
                if response_id & 0x0400 != 0
                    && let Some(buffer) = request_buffer.take()
                {
                    unsafe {
                        transport.publish_request_in_place(
                            buffer.into_release(),
                            &output[..length],
                            length as u16,
                        );
                    }
                } else if transport.output_available() {
                    let indication = unsafe { transport.output_buffer() };
                    indication[..length].copy_from_slice(&output[..length]);
                    transport.publish(length as u16);
                }
            }
            // Synchronous commands return their owning request buffer here.
            // Ordinary class-0 TX moves it into HostTxDriver until confirmation.
            if let Some(buffer) = request_buffer {
                transport.release_request(buffer.into_release());
            }
        }
        core::hint::spin_loop();
    }
}

#[cfg(target_arch = "arm")]
fn panic_file_hash(file: &str) -> u32 {
    file.bytes().fold(0x811c_9dc5_u32, |hash, byte| {
        (hash ^ u32::from(byte)).wrapping_mul(0x0100_0193)
    })
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    #[cfg(target_arch = "arm")]
    {
        let (line, column, file_hash) = info
            .location()
            .map(|location| {
                (
                    location.line(),
                    location.column(),
                    panic_file_hash(location.file()),
                )
            })
            .unwrap_or((0, 0, 0));
        xr819_firmware::exception::panic_terminal(line, column, file_hash)
    }
    #[cfg(not(target_arch = "arm"))]
    {
        let _ = info;
        loop {
            core::hint::spin_loop();
        }
    }
}
