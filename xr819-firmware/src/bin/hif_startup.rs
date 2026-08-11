#![no_std]
#![no_main]

use core::arch::naked_asm;
use core::mem::size_of;
use core::panic::PanicInfo;
use xr819_firmware::configuration;
use xr819_firmware::crypto;
use xr819_firmware::hif::Transport;
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
use xr819_firmware::scan;
#[cfg(feature = "tcm-size-diagnostic")]
use xr819_firmware::tcm;
use xr819_firmware::tx;
#[cfg(feature = "vendor-host-tx-foundation")]
use xr819_firmware::vendor_host_tx;
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
    encode_xr819_tx_confirm_details,
};

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
        unsafe { (0x0900_ff98 as *mut u32).write_volatile(marker) };
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
        if let Err(error) = mac::initialize_vendor_startup_state(1_000_000) {
            (0x0900_ff98 as *mut u32).write_volatile(0x4d41_433f);
            (0x0900_ff9c as *mut u32).write_volatile(error as u32 + 1);
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
        label: b"XR819 open Rust WSM",
        config: [0; 4],
    }
    .encode(buffer)
    .unwrap_or(0);

    if length != 0 {
        unsafe { transport.publish(length as u16) };
    }

    let mut pending_scan_completion: Option<scan::ScanCompletion> = None;
    let mut pending_join_complete: Option<u32> = None;
    let mut pending_tx_confirmation: Option<(u32, u32, u8, u8)> = None;
    let mut pending_tx_debug_event: Option<(u32, u32)> = None;
    #[cfg(feature = "vendor-host-tx-foundation")]
    let mut pending_vendor_host_tx: Option<vendor_host_tx::RetainedHostTx> = None;
    #[cfg(feature = "vendor-host-tx-foundation")]
    let mut pending_vendor_confirmation: Option<(vendor_host_tx::RetainedHostTx, u32, u8)> = None;
    #[cfg(feature = "vendor-host-tx-foundation")]
    let mut pending_vendor_reservation: Option<vendor_host_tx::HostSchedulerReservation> = None;
    loop {
        let _ = transport.service_interrupt();

        #[cfg(feature = "vendor-host-tx-foundation")]
        if pending_vendor_confirmation.is_none() && pending_vendor_reservation.is_none() {
            let completed = pending_vendor_host_tx.as_mut().and_then(|retained| {
                match unsafe { vendor_host_tx::service_pending(retained) } {
                    Ok(vendor_host_tx::PendingServiceReport::Complete(status)) => Some(status),
                    Ok(vendor_host_tx::PendingServiceReport::PasQueued) => None,
                    Ok(vendor_host_tx::PendingServiceReport::LeaveQueued) | Err(_) => None,
                }
            });
            if let (Some(status), Some(mut retained)) = (completed, pending_vendor_host_tx.take()) {
                let _ = retained.transition(vendor_host_tx::HostTxPhase::Completing);
                pending_vendor_confirmation =
                    Some((retained, tx::wsm_status_from_internal(status), 0));
            }
        }

        #[cfg(feature = "vendor-host-tx-foundation")]
        if pending_vendor_reservation.is_none() {
            let mut expired = false;
            if let Some(retained) = pending_vendor_host_tx.as_mut()
                && retained.phase() == vendor_host_tx::HostTxPhase::PasQueued
            {
                match unsafe { vendor_host_tx::reserve_non_aggregate_scheduler(retained) } {
                    Ok(reservation) => pending_vendor_reservation = Some(reservation),
                    Err(vendor_host_tx::SchedulerReserveError::Expired) => expired = true,
                    Err(_) => {}
                }
            }
            if expired
                && let Some(mut retained) = pending_vendor_host_tx.take()
                && unsafe { vendor_host_tx::reject_unscheduled_pas(&mut retained) }.is_ok()
            {
                let _ = retained.transition(vendor_host_tx::HostTxPhase::Completing);
                pending_vendor_confirmation = Some((retained, tx::wsm_status_from_internal(10), 0));
            }
        }

        #[cfg(feature = "vendor-host-tx-foundation")]
        if let Some(reservation) = pending_vendor_reservation.take()
            && let Some(retained) = pending_vendor_host_tx.as_mut()
            && let Err((reservation, _)) = unsafe { reservation.publish(retained) }
        {
            pending_vendor_reservation = Some(reservation);
        }

        #[cfg(feature = "vendor-host-tx-foundation")]
        if pending_vendor_confirmation.is_none()
            && let Some(retained) = pending_vendor_host_tx.as_mut()
            && retained.phase() == vendor_host_tx::HostTxPhase::Scheduled
            && let Some((context, status, ack_failures)) =
                unsafe { tx::service_host_class0_runtime(32) }
            && context == retained.context().raw()
            && let Some(mut retained) = pending_vendor_host_tx.take()
        {
            let _ = retained.transition(vendor_host_tx::HostTxPhase::Completing);
            pending_vendor_confirmation =
                Some((retained, tx::wsm_status_from_internal(status), ack_failures));
        }

        #[cfg(feature = "join-sta-experiment")]
        if pending_tx_confirmation.is_none()
            && let tx::HostManagementTxReport::Completed {
                packet_id,
                status,
                tx_rate,
                ack_failures,
            } = unsafe { tx::service_host_management_tx(None, 32) }
        {
            pending_tx_confirmation = Some((packet_id, status, tx_rate, ack_failures));
        }

        if pending_tx_debug_event.is_none() {
            pending_tx_debug_event = tx::take_tx_debug_event();
        }

        if let Some((event_id, data)) = pending_tx_debug_event
            && transport.output_available()
        {
            let output = unsafe { transport.output_buffer() };
            if let Some(length) = encode_debug_event(event_id, data, output) {
                pending_tx_debug_event = None;
                unsafe { transport.publish(length as u16) };
            }
        }

        if let Some(status) = pending_join_complete
            && transport.output_available()
        {
            let output = unsafe { transport.output_buffer() };
            if let Ok(length) = encode_join_complete_indication(status, output) {
                pending_join_complete = None;
                unsafe { transport.publish(length as u16) };
            }
        }

        #[cfg(feature = "join-sta-experiment")]
        if let Some((packet_id, status, tx_rate, ack_failures)) = pending_tx_confirmation
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
                unsafe { transport.publish(length as u16) };
            }
        }

        #[cfg(feature = "vendor-host-tx-foundation")]
        if let Some((retained, status, ack_failures)) = pending_vendor_confirmation.as_ref()
            && transport.output_available()
        {
            let packet_id = retained.packet_id();
            let context = retained.context().raw();
            let tx_rate = unsafe { (context.wrapping_add(0x63) as *const u8).read_volatile() };
            let output = unsafe { transport.output_buffer() };
            let encoded = if join::uses_cw1200_wsm() {
                encode_tx_confirm_details(packet_id, *status, tx_rate, *ack_failures, output)
            } else {
                encode_xr819_tx_confirm_details(packet_id, *status, tx_rate, *ack_failures, output)
            };
            if let Ok(length) = encoded
                && let Some((retained, _, _)) = pending_vendor_confirmation.take()
            {
                unsafe { transport.publish(length as u16) };
                let release = unsafe { retained.finish() };
                transport.release_request(release);
            }
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
                    configuration::template_frame(),
                    opportunity,
                    &probe_ssid[..ssid_length.unwrap_or(0)],
                    32,
                )
            };
            unsafe {
                match report {
                    tx::ProbeExperimentReport::Published {
                        opportunity,
                        publication,
                    } => {
                        (0x0900_ffa0 as *mut u32).write_volatile(0x5055_4231);
                        (0x0900_ffa4 as *mut u32).write_volatile(publication.context.raw());
                        (0x0900_ffa8 as *mut u32).write_volatile(
                            u32::from(publication.pipe)
                                | (u32::from(publication.slot) << 8)
                                | (u32::from(opportunity.ssid_index) << 16),
                        );
                    }
                    tx::ProbeExperimentReport::Completed {
                        opportunity,
                        context,
                        status,
                    } => {
                        if !scan::complete_probe(opportunity, status) {
                            (0x0900_ffa0 as *mut u32).write_volatile(0x5458_3f3f);
                            loop {
                                core::hint::spin_loop();
                            }
                        }
                        (0x0900_ffa0 as *mut u32).write_volatile(0x5458_444e);
                        (0x0900_ffa4 as *mut u32).write_volatile(context.raw());
                        (0x0900_ffa8 as *mut u32).write_volatile(u32::from(status));
                    }
                    tx::ProbeExperimentReport::Failed { opportunity, error } => {
                        let _ = scan::fail_probe(opportunity);
                        (0x0900_ffa0 as *mut u32).write_volatile(0x5458_463f);
                        (0x0900_ffa4 as *mut u32).write_volatile(error as u32);
                    }
                    tx::ProbeExperimentReport::Idle | tx::ProbeExperimentReport::Servicing => {}
                }
            }
        }

        if let Some(completion) = pending_scan_completion
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
                unsafe { transport.publish(length as u16) };
            }
        }

        // Control indications take priority over scan RX. Otherwise a steady
        // stream of probe responses can consume the last HIF descriptor every
        // pass and leave a completed scan pending until the host times out.
        if pending_scan_completion.is_none() {
            if let (Some(if_id), Some(channel)) = (scan::active_interface(), scan::active_channel())
            {
                if transport.output_available() {
                    if let Some(indication) = unsafe { radio::poll_scan_indication(if_id, channel) }
                    {
                        unsafe { transport.publish_radio(indication) };
                    }
                }
            } else if let Some(if_id) = vif::active_interface() {
                let channel = unsafe { vif::snapshot(if_id) }
                    .map(|state| state.channel)
                    .unwrap_or(0);
                if transport.output_available()
                    && let Some(indication) =
                        unsafe { radio::poll_joined_indication(if_id, channel) }
                {
                    unsafe { transport.publish_radio(indication) };
                }
            } else {
                // Vendor RX processing never stops between scans. Recycle one
                // slot per cooperative pass so a later dwell cannot consume
                // idle-era beacons as if they had just arrived.
                let channel = unsafe { (0x0400_3a68 as *const u16).read_volatile() };
                unsafe { radio::discard_one_idle(channel) };
            }
        }

        if transport.output_available() {
            if let Some(request) = transport.poll_request() {
                let output = unsafe { transport.output_buffer() };
                let mut publish_response = true;
                #[allow(unused_mut)]
                let mut release_current_request = true;
                let response_length = if request.if_id > 2 {
                    encode_status_response(request.id | 0x0400, STATUS_FAILURE, output)
                } else if request.id == CONFIGURATION_REQ_ID {
                    let configured = ConfigurationRequest::parse(request.payload).ok().and_then(
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
                } else if request.id == START_SCAN_REQ_ID {
                    let status = match StartScanRequest::parse(request.payload) {
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
                                            request.if_id,
                                        )
                                        .map(|_| ())
                                        .map_err(|_| ())
                                    },
                                    _ => Err(()),
                                }
                            };
                            match preparation {
                                Ok(()) => match scan::begin(&scan_request, request.if_id) {
                                    Ok(()) => 0,
                                    Err(scan::ScanError::Busy) => 4,
                                    Err(_) => 2,
                                },
                                Err(()) => 2,
                            }
                        }
                        Err(_) => 2,
                    };
                    encode_status_response(request.id | 0x0400, status, output)
                } else if request.id == TX_QUEUE_PARAMS_REQ_ID {
                    let status = match TxQueueParameters::parse(request.payload) {
                        Ok(parameters) => {
                            configuration::retain_tx_queue(parameters);
                            0
                        }
                        Err(_) => 2,
                    };
                    encode_status_response(request.id | 0x0400, status, output)
                } else if request.id == EDCA_PARAMS_REQ_ID {
                    let status = match EdcaParameters::parse(request.payload) {
                        Ok(parameters) => {
                            configuration::retain_edca(parameters);
                            unsafe {
                                vif::apply_edca(request.if_id, parameters)
                                    .map(|_| 0)
                                    .unwrap_or(2)
                            }
                        }
                        Err(_) => 2,
                    };
                    encode_status_response(request.id | 0x0400, status, output)
                } else if request.id == WRITE_MIB_REQ_ID {
                    let status = match WriteMibRequest::parse(request.payload) {
                        Ok(request)
                            if configuration::retain_interface_mib(
                                request.mib_id,
                                request.data,
                            ) =>
                        {
                            0
                        }
                        _ => STATUS_FAILURE,
                    };
                    encode_status_response(request.id | 0x0400, status, output)
                } else if request.id == READ_MIB_REQ_ID {
                    let mib_id = request
                        .payload
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
                        let mut data = [0_u8; 88];
                        for (index, value) in values.into_iter().enumerate() {
                            data[index * 4..index * 4 + 4].copy_from_slice(&value.to_le_bytes());
                        }
                        encode_read_mib_data_response(0, mib_id, &data, output)
                    } else {
                        encode_read_mib_response(STATUS_FAILURE, mib_id, output)
                    }
                } else if request.id == ADD_KEY_REQ_ID {
                    let status = AddKeyRequest::parse(request.payload)
                        .ok()
                        .and_then(|key| crypto::add_key(request.if_id, &key).ok())
                        .map(|()| 0)
                        .unwrap_or(STATUS_FAILURE);
                    encode_status_response(request.id | 0x0400, status, output)
                } else if request.id == REMOVE_KEY_REQ_ID {
                    let status = RemoveKeyRequest::parse(request.payload)
                        .ok()
                        .and_then(|key| crypto::remove_key(key.index).ok())
                        .map(|()| 0)
                        .unwrap_or(STATUS_FAILURE);
                    encode_status_response(request.id | 0x0400, status, output)
                } else if request.id == RESET_REQ_ID {
                    let status = match ResetRequest::parse(request.payload) {
                        Ok(_) => {
                            #[cfg(feature = "vendor-host-tx-foundation")]
                            {
                                if let Some(reservation) = pending_vendor_reservation.take()
                                    && let Some(retained) = pending_vendor_host_tx.as_mut()
                                {
                                    let _ = unsafe { reservation.cancel(retained) };
                                }
                                if pending_vendor_host_tx.as_ref().is_some_and(|retained| {
                                    retained.phase() != vendor_host_tx::HostTxPhase::Scheduled
                                }) && let Some(retained) = pending_vendor_host_tx.take()
                                {
                                    // Pending, PAS-ring, and unpublished slot
                                    // reservations are reversible. Scheduled
                                    // hardware ownership must complete normally.
                                    if let Ok(release) = unsafe { retained.cancel_before_pas() } {
                                        transport.release_request(release);
                                    }
                                }
                                if let Some((retained, _, _)) = pending_vendor_confirmation.take() {
                                    let release = unsafe { retained.finish() };
                                    transport.release_request(release);
                                }
                            }
                            #[cfg(feature = "join-sta-experiment")]
                            {
                                if unsafe { join::reset(request.if_id) } {
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
                    encode_status_response(request.id | 0x0400, status, output)
                } else if request.id == JOIN_REQ_ID {
                    #[cfg(feature = "join-sta-experiment")]
                    let status = match JoinRequest::parse(request.payload) {
                        Ok(join_request) => unsafe {
                            join::activate_sta(request.if_id, &join_request)
                                .map(|_| 0)
                                .unwrap_or(STATUS_FAILURE)
                        },
                        Err(_) => STATUS_FAILURE,
                    };
                    #[cfg(not(feature = "join-sta-experiment"))]
                    let status = STATUS_FAILURE;
                    if status == 0
                        && request
                            .payload
                            .get(0x0f)
                            .is_some_and(|flags| flags & 0x20 != 0)
                    {
                        pending_join_complete = Some(0);
                    }
                    encode_join_response(status, -160, 200, output)
                } else if request.id == TX_REQ_ID {
                    #[cfg(feature = "join-sta-experiment")]
                    {
                        match TxRequest::parse(request.payload) {
                            Ok(tx_request) => {
                                #[cfg(feature = "vendor-host-tx-foundation")]
                                let vendor_response = if tx_request.is_unicast_data()
                                    && !tx_request.is_unicast_eapol()
                                {
                                    Some(if pending_vendor_host_tx.is_some() {
                                        if join::uses_cw1200_wsm() {
                                            encode_tx_confirm(
                                                tx_request.packet_id,
                                                STATUS_FAILURE,
                                                output,
                                            )
                                        } else {
                                            encode_xr819_tx_confirm(
                                                tx_request.packet_id,
                                                STATUS_FAILURE,
                                                output,
                                            )
                                        }
                                    } else {
                                        match unsafe {
                                            vendor_host_tx::admit_host_tx(
                                                &tx_request,
                                                request.if_id,
                                                request.release,
                                            )
                                        } {
                                            Ok(mut retained) => match unsafe {
                                                vendor_host_tx::classify_and_encrypt(&mut retained)
                                            } {
                                                Ok(()) => match unsafe {
                                                    vendor_host_tx::enqueue_post_crypto(
                                                        &mut retained,
                                                    )
                                                } {
                                                    Ok(()) => {
                                                        pending_vendor_host_tx = Some(retained);
                                                        release_current_request = false;
                                                        publish_response = false;
                                                        Ok(0)
                                                    }
                                                    Err(_) => {
                                                        let release = unsafe { retained.abort() };
                                                        transport.release_request(release);
                                                        release_current_request = false;
                                                        if join::uses_cw1200_wsm() {
                                                            encode_tx_confirm(
                                                                tx_request.packet_id,
                                                                STATUS_FAILURE,
                                                                output,
                                                            )
                                                        } else {
                                                            encode_xr819_tx_confirm(
                                                                tx_request.packet_id,
                                                                STATUS_FAILURE,
                                                                output,
                                                            )
                                                        }
                                                    }
                                                },
                                                Err(_) => {
                                                    let release = unsafe { retained.abort() };
                                                    transport.release_request(release);
                                                    release_current_request = false;
                                                    if join::uses_cw1200_wsm() {
                                                        encode_tx_confirm(
                                                            tx_request.packet_id,
                                                            STATUS_FAILURE,
                                                            output,
                                                        )
                                                    } else {
                                                        encode_xr819_tx_confirm(
                                                            tx_request.packet_id,
                                                            STATUS_FAILURE,
                                                            output,
                                                        )
                                                    }
                                                }
                                            },
                                            Err(_) => {
                                                if join::uses_cw1200_wsm() {
                                                    encode_tx_confirm(
                                                        tx_request.packet_id,
                                                        STATUS_FAILURE,
                                                        output,
                                                    )
                                                } else {
                                                    encode_xr819_tx_confirm(
                                                        tx_request.packet_id,
                                                        STATUS_FAILURE,
                                                        output,
                                                    )
                                                }
                                            }
                                        }
                                    })
                                } else {
                                    None
                                };
                                #[cfg(not(feature = "vendor-host-tx-foundation"))]
                                let vendor_response: Option<
                                    Result<usize, xr819_firmware::wsm::Error>,
                                > = None;

                                if let Some(response) = vendor_response {
                                    response
                                } else {
                                    match unsafe {
                                        tx::service_host_management_tx(
                                            Some((&tx_request, request.if_id)),
                                            0,
                                        )
                                    } {
                                        tx::HostManagementTxReport::Published {
                                            packet_id,
                                            bisect_stage,
                                        } => {
                                            publish_response = true;
                                            if bisect_stage != 0 {
                                                if join::uses_cw1200_wsm() {
                                                    encode_tx_confirm_details(
                                                        packet_id,
                                                        STATUS_FAILURE,
                                                        0,
                                                        bisect_stage,
                                                        output,
                                                    )
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
                                                let edca = unsafe {
                                                    (0x09c0_0e64 as *const u32).read_volatile()
                                                };
                                                let quantum0 = unsafe {
                                                    (0x09c0_0e70 as *const u32).read_volatile()
                                                };
                                                let quantum1 = unsafe {
                                                    (0x09c0_0e74 as *const u32).read_volatile()
                                                };
                                                let metadata = unsafe {
                                                    (0x0900_8008 as *const u8).read_volatile()
                                                };
                                                let secondary = unsafe {
                                                    (0x0900_7bc0 as *const u8).read_volatile()
                                                };
                                                let event_id = 0x5852_0000 | (edca & 0xffff);
                                                let data = (quantum0 & 0xff)
                                                    | ((quantum1 & 0xff) << 8)
                                                    | (u32::from(metadata) << 16)
                                                    | (u32::from(secondary) << 24);
                                                encode_debug_event(event_id, data, output)
                                                    .ok_or(xr819_firmware::wsm::Error::Truncated)
                                            }
                                        }
                                        tx::HostManagementTxReport::Failed {
                                            packet_id, ..
                                        } => {
                                            if join::uses_cw1200_wsm() {
                                                encode_tx_confirm(packet_id, STATUS_FAILURE, output)
                                            } else {
                                                encode_xr819_tx_confirm(
                                                    packet_id,
                                                    STATUS_FAILURE,
                                                    output,
                                                )
                                            }
                                        }
                                        _ => {
                                            publish_response = false;
                                            encode_tx_confirm(0, STATUS_FAILURE, output)
                                        }
                                    }
                                }
                            }
                            Err(_) => {
                                let packet_id = request
                                    .payload
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
                        let packet_id = request
                            .payload
                            .get(..4)
                            .map(|value| {
                                u32::from_le_bytes([value[0], value[1], value[2], value[3]])
                            })
                            .unwrap_or(0);
                        encode_xr819_tx_confirm(packet_id, STATUS_FAILURE, output)
                    }
                } else {
                    // Do not report success for commands whose state effects are
                    // not implemented. A complete status word lets cw1200 fail the
                    // command cleanly instead of proceeding on false assumptions.
                    encode_status_response(request.id | 0x0400, STATUS_FAILURE, output)
                };
                if publish_response && let Ok(length) = response_length {
                    unsafe { transport.publish(length as u16) };
                }
                // Most commands finish synchronously and release their borrowed
                // packet-RAM request here. The future vendor host-TX path will
                // transfer this token into class-0 state and release it only
                // after confirmation.
                if release_current_request {
                    transport.release_request(request.release);
                }
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
