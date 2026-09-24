#![no_std]
#![no_main]

use core::arch::{global_asm, naked_asm};
use core::cell::UnsafeCell;
use core::mem::{MaybeUninit, size_of};
use core::panic::PanicInfo;
use xr819_firmware::command::{
    ENABLE_SINGLE_PROBE_EXPERIMENT, service_one as service_one_command,
};
use xr819_firmware::configuration;
use xr819_firmware::hif::{HifQueues, HifRingState, SHARED_BUFFER_SIZE, Transport};
use xr819_firmware::mac;
use xr819_firmware::mac_domain::MacDomain;
use xr819_firmware::phy::{initialize_mac_core_mode0, initialize_mac_software_state};
use xr819_firmware::platform::{
    enable_packet_controller, initialize_runtime_state, prepare_dma_and_clocks,
    prepare_high_platform_support, prepare_mac_receive_hardware, prepare_main_control,
    prepare_memory_and_interrupts, prepare_packet_dma, register_packet_dma_interrupts,
    register_post_activation_interrupts, service_masked_packet_dma_interrupt, try_activate_hif,
    wait_for_host_download_completion,
};
use xr819_firmware::radio;
use xr819_firmware::scan;
use xr819_firmware::tx;
use xr819_firmware::vif;
use xr819_firmware::wsm::{
    StartupIndication, encode_join_complete_indication, encode_scan_complete_indication,
    encode_xr819_multi_tx_confirm_header, encode_xr819_tx_confirm_details,
    encode_xr819_tx_confirm_retry_entry,
};
use xr819_firmware::host_tx_driver::HostTxDriver;

/// Const-initialized storage taken once by the single reset-time owner.
///
/// Unlike a general static cell this needs no atomics: the references never
/// escape `rust_main`, and reset initializes each cell exactly once before
/// entering the cooperative loop.
struct SingleBootCell<T>(UnsafeCell<MaybeUninit<T>>);

unsafe impl<T> Sync for SingleBootCell<T> {}

impl<T> SingleBootCell<T> {
    const fn new() -> Self {
        Self(UnsafeCell::new(MaybeUninit::uninit()))
    }

    /// # Safety
    /// This may be called only once for each cell during one firmware boot.
    unsafe fn init_with(&'static self, value: impl FnOnce() -> T) -> &'static mut T {
        unsafe { (&mut *self.0.get()).write(value()) }
    }
}

static HIF_RING_STATE: SingleBootCell<HifRingState> = SingleBootCell::new();
static HIF_QUEUES: SingleBootCell<HifQueues> = SingleBootCell::new();
static TRANSPORT: SingleBootCell<Transport> = SingleBootCell::new();
static RESPONSE_SCRATCH: SingleBootCell<[u8; SHARED_BUFFER_SIZE]> = SingleBootCell::new();
static HOST_TX_DRIVER: SingleBootCell<HostTxDriver> = SingleBootCell::new();

/// Single owner of cooperative reactor resources and retained lane state.
///
/// Hardware and DTCM ownership remain governed by their existing domains; this
/// value makes the foreground scheduler's Rust ownership explicit without
/// changing lane priority or per-pass budgets.
struct Firmware {
    transport: &'static mut Transport,
    mac_events: tx::MacEventQueue,
    mac_domain: MacDomain,
    response_scratch: &'static mut [u8; SHARED_BUFFER_SIZE],
    host_tx_driver: &'static mut HostTxDriver,
    pending_scan_completion: Option<scan::ScanCompletion>,
    pending_join_complete: Option<u32>,
    pending_tx_confirmation: Option<(u32, u32, u8, u8)>,
    #[cfg(target_arch = "arm")]
    last_watchdog_tick: u32,
    #[cfg(target_arch = "arm")]
    watchdog_timer_countdown: u8,
}

impl Firmware {
    fn new(
        transport: &'static mut Transport,
        mac_events: tx::MacEventQueue,
        mac_domain: MacDomain,
        response_scratch: &'static mut [u8; SHARED_BUFFER_SIZE],
        host_tx_driver: &'static mut HostTxDriver,
    ) -> Self {
        Self {
            transport,
            mac_events,
            mac_domain,
            response_scratch,
            host_tx_driver,
            pending_scan_completion: None,
            pending_join_complete: None,
            pending_tx_confirmation: None,
            #[cfg(target_arch = "arm")]
            last_watchdog_tick: 0,
            #[cfg(target_arch = "arm")]
            watchdog_timer_countdown: 0,
        }
    }
}

fn publish_coalesced_host_tx_confirmations(firmware: &mut Firmware, host_request_waiting: bool) {
    const MAX_CONFIRMATIONS: usize = 4;
    if host_request_waiting || !firmware.transport.response_available() {
        return;
    }
    let count = firmware
        .host_tx_driver
        .confirmation_count(MAX_CONFIRMATIONS);
    if count == 0 {
        return;
    }
    let Ok(length) = encode_xr819_multi_tx_confirm_header(count, firmware.response_scratch) else {
        return;
    };
    let mut first_release = None;
    for index in 0..count {
        let Some(confirmation) = firmware.host_tx_driver.confirmation() else {
            return;
        };
        if encode_xr819_tx_confirm_retry_entry(
            confirmation.packet_id,
            confirmation.status,
            confirmation.tx_rate,
            confirmation.ack_failures,
            confirmation.flags,
            confirmation.rate_try,
            8 + index * 32,
            firmware.response_scratch,
        )
        .is_err()
        {
            return;
        }
        let Some(release) = (unsafe { firmware.host_tx_driver.finish_confirmation() }) else {
            return;
        };
        if first_release.is_none() {
            first_release = Some(release);
        } else {
            firmware.transport.release_request(release);
        }
    }
    if let Some(release) = first_release {
        unsafe {
            firmware.transport.publish_request_in_place(
                release,
                &firmware.response_scratch[..length],
                length as u16,
            );
        }
    }
}

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
    ldr sp, =0x0400a100
    orr r1, r0, #23
    msr cpsr_c, r1
    ldr sp, =0x0400a200
    orr r1, r0, #19
    msr cpsr_c, r1
    ldr sp, =0x0400a300
    orr r1, r0, #18
    msr cpsr_c, r1
    ldr sp, =0x0400a400
    orr r1, r0, #17
    msr cpsr_c, r1
    ldr sp, =0x0400a500
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

unsafe extern "C" {
    static mut __bss_start: u32;
    static mut __bss_end: u32;
}

#[unsafe(no_mangle)]
#[used]
static STARTUP_DEBUG_STAGE: u32 = u32::MAX;

unsafe fn clear_words(mut address: usize, end: usize) {
    while address < end {
        unsafe { (address as *mut u32).write_volatile(0) };
        address += size_of::<u32>();
    }
}

unsafe fn clear_rust_bss() {
    unsafe {
        clear_words(
            (&raw mut __bss_start) as usize,
            (&raw mut __bss_end) as usize,
        );
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
    unsafe {
        clear_rust_bss();
        xr819_firmware::dtcm::zero_initialized_data();
    }
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
    #[cfg(target_arch = "arm")]
    xr819_firmware::crypto::run_hardware_ccmp_selftest();
    let hif_ring_state = unsafe { HIF_RING_STATE.init_with(HifRingState::new) };
    let hif_queues = unsafe { HIF_QUEUES.init_with(HifQueues::new) };
    let transport =
        unsafe { TRANSPORT.init_with(|| Transport::initialize(hif_ring_state, hif_queues)) };
    let mac_events = unsafe { tx::MacEventQueue::claim() };
    let mac_domain = MacDomain::new();
    debug_stop(6, 0x5354_4706);

    // Vendor 0x9ac calls packet-DMA initialization immediately after 0x94c.
    prepare_packet_dma();
    // Vendor programs the RX/MAC tables once, after MAC core enable, via
    // `rx_subsystem_init`; we also do it there (`mac.rs:831`). This earlier
    // copy has no vendor counterpart.
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

    let startup_label: &[u8] = b"XR819 open Rust native";

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
        label: startup_label,
        config: [0; 4],
    }
    .encode(buffer)
    .unwrap_or(0);

    if length != 0 {
        transport.publish(length as u16);
    }

    // Command responses and retained class-0 confirmations are copied into
    // their original 1632-byte request buffers before publication. This buffer
    // is scratch only and is never exposed through a HIF descriptor.
    let response_scratch = unsafe { RESPONSE_SCRATCH.init_with(|| [0; SHARED_BUFFER_SIZE]) };
    let host_tx_driver = unsafe { HOST_TX_DRIVER.init_with(HostTxDriver::new) };
    let mut firmware = Firmware::new(
        transport,
        mac_events,
        mac_domain,
        response_scratch,
        host_tx_driver,
    );
    // Main-loop rate. Admission -> publication is 37 ms under load with a
    // budget of 4 over 30 contexts, implying ~5 ms per pass, and raising the
    // budget made things worse, so the cost is per pass. An earlier attempt
    // timestamped each iteration from 0x0ac00004; reading that register every
    // pass broke association outright, so count iterations instead and derive
    // the rate from the counter delta between harness samples. An increment is
    // nearly free where an MMIO read is not.
    loop {
        let _ = service_masked_packet_dma_interrupt();
        let _ = firmware.transport.service_interrupt();
        // Vendor runs a 200 ms timer (`FUN_00003bac`) that decrements each
        // programmed pipe's watchdog byte and recovers a pipe that has stayed
        // armed without completing. Without it an armed pipe is unrecoverable:
        // retirement only ever runs from a delivered status, and the MAC stops
        // delivering statuses for a wedged pipe.
        #[cfg(target_arch = "arm")]
        {
            let (next_countdown, sample_timer) =
                tx::advance_watchdog_timer_divider(firmware.watchdog_timer_countdown);
            firmware.watchdog_timer_countdown = next_countdown;
            if sample_timer {
                let now = unsafe { xr819_firmware::vendor_host_tx::vendor_timer_now() };
                // The vendor-shaped five-tick expiry takes about one second. A
                // stuck pipe remains armed during that window and blocks every
                // new reservation for the pipe. Retirement cannot help because
                // it only runs from a delivered status.
                if now.wrapping_sub(firmware.last_watchdog_tick) >= 200_000 {
                    firmware.last_watchdog_tick = now;
                    unsafe {
                        tx::service_pipe_watchdog_tick_runtime();
                    }
                }
            }
        }

        // A ready host request may be a synchronous command. Preserve one
        // output descriptor for it instead of allowing asynchronous events or
        // TX confirmations to starve the command lane.
        let host_request_waiting = firmware.transport.request_available();

        unsafe {
            firmware.host_tx_driver.service(
                &mut firmware.mac_events,
                &mut firmware.mac_domain,
                !tx::host_management_runtime_active(),
            );
        }

        let management_runtime_available = firmware.host_tx_driver.management_runtime_available();

        if (management_runtime_available || unsafe { tx::host_management_runtime_active() })
            && firmware.pending_tx_confirmation.is_none()
            && let tx::HostManagementTxReport::Completed {
                packet_id,
                status,
                tx_rate,
                ack_failures,
            } = unsafe { tx::service_host_management_tx(&mut firmware.mac_events, None, 32) }
        {
            firmware.pending_tx_confirmation = Some((packet_id, status, tx_rate, ack_failures));
        }

        if let Some(status) = firmware.pending_join_complete
            && !host_request_waiting
            && firmware.transport.output_available()
        {
            let output = unsafe { firmware.transport.output_buffer() };
            if let Ok(length) = encode_join_complete_indication(status, output) {
                firmware.pending_join_complete = None;
                firmware.transport.publish(length as u16);
            }
        }

        if let Some((packet_id, status, tx_rate, ack_failures)) = firmware.pending_tx_confirmation
            && !host_request_waiting
            && firmware.transport.output_available()
        {
            let output = unsafe { firmware.transport.output_buffer() };
            let encoded =
                encode_xr819_tx_confirm_details(packet_id, status, tx_rate, ack_failures, output);
            if let Ok(length) = encoded {
                firmware.pending_tx_confirmation = None;
                firmware.transport.publish(length as u16);
            }
        }

        publish_coalesced_host_tx_confirmations(&mut firmware, host_request_waiting);

        // Retain completion until a HIF descriptor is available. This prevents
        // a full ring from overwriting an unreclaimed zero-copy RX token.
        if firmware.pending_scan_completion.is_none() {
            firmware.pending_scan_completion = scan::service();
        }

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
                    &mut firmware.mac_events,
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

        if let Some(completion) = firmware.pending_scan_completion
            && !host_request_waiting
            && firmware.transport.output_available()
        {
            let output = unsafe { firmware.transport.output_buffer() };
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
                firmware.pending_scan_completion = None;
                firmware.transport.publish(length as u16);
            }
        }

        // Control indications and inbound host requests take priority over RX.
        // Otherwise a steady joined RX stream can consume the last firmware-to-
        // host descriptor every pass, preventing `poll_request()` from ever
        // detaching an ordinary TX request even though the driver accounts its
        // input buffer as used.
        if firmware.pending_scan_completion.is_none() && !host_request_waiting {
            if let (Some(if_id), Some(channel)) = (scan::active_interface(), scan::active_channel())
            {
                if firmware.transport.publication_available() {
                    if let Some(indication) = unsafe { radio::poll_scan_indication(if_id, channel) }
                    {
                        firmware.transport.publish_radio(indication);
                    }
                }
            } else if let Some(if_id) = vif::active_interface() {
                let channel = vif::snapshot(if_id).map(|state| state.channel).unwrap_or(0);
                let publication_available = firmware.transport.publication_available();
                if publication_available
                    && let Some(indication) =
                        unsafe { radio::poll_joined_indication(if_id, channel) }
                {
                    firmware.transport.publish_radio(indication);
                }
            } else {
                // Vendor RX processing never stops between scans. Recycle one
                // slot per cooperative pass so a later dwell cannot consume
                // idle-era beacons as if they had just arrived.
                let channel = unsafe {
                    (xr819_firmware::dtcm::LOW_MAC_CURRENT_CHANNEL.get() as *const u16)
                        .read_volatile()
                };
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
        // Answering host requests must not depend on a data frame completing.
        // `management_runtime_available` is false while any class-0 frame is in
        // flight, so gating the poll on it starves the command lane under
        // continuous TX and forever if a frame sticks: the host stops getting
        // replies and declares `[BH] Fatal error` long before the 5 x 200 ms
        // pipe watchdog can recover the slot. Vendor has no such coupling,
        // which is why it can discard unmatched statuses and lean on the
        // watchdog. The management publisher this gate was protecting is
        // already gated separately where it is serviced.
        unsafe {
            service_one_command(
                &mut *firmware.transport,
                &mut firmware.mac_events,
                &mut firmware.mac_domain,
                &mut *firmware.host_tx_driver,
                &mut *firmware.response_scratch,
                &mut firmware.pending_join_complete,
            );
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
