//! WSM command dispatch for the cooperative firmware reactor.
//!
//! One call services at most one detached host request. Publication and request
//! ownership remain in the same lane so synchronous commands return their token
//! exactly once while admitted class-0 TX transfers it to `HostTxDriver`.

use crate::configuration;
use crate::crypto;
use crate::hif::{ReceivedRequest, SHARED_BUFFER_SIZE, Transport};
use crate::host_tx_diagnostics;
use crate::host_tx_driver::HostTxDriver;
use crate::join;
use crate::mac_domain::MacDomain;
use crate::packet_ram;
use crate::platform::{self, program_station_address};
use crate::radio;
use crate::rate_policy;
use crate::scan;
use crate::tx;
use crate::vif;
#[cfg(any(
    feature = "dtcm-contract-diagnostics",
    feature = "experimental-dynamic-iq-trace"
))]
use crate::wsm::encode_read_mib_data_response_in_place;
use crate::wsm::{
    ADD_KEY_REQ_ID, AddKeyRequest, CONFIGURATION_REQ_ID, ConfigurationRequest, EDCA_PARAMS_REQ_ID,
    EdcaParameters, JOIN_REQ_ID, JoinRequest, READ_MIB_REQ_ID, REMOVE_KEY_REQ_ID, RESET_REQ_ID,
    RemoveKeyRequest, ResetRequest, SET_BSS_PARAMS_REQ_ID, START_SCAN_REQ_ID, STATUS_FAILURE,
    SetBssParameters, StartScanRequest, TX_QUEUE_PARAMS_REQ_ID, TX_REQ_ID, TxPowerRange,
    TxQueueParameters, TxRequest,
    WRITE_MIB_REQ_ID, WriteMibRequest, encode_configuration_response, encode_join_response,
    encode_read_mib_data_response, encode_read_mib_response, encode_status_response,
    encode_tx_confirm, encode_xr819_tx_confirm, encode_xr819_tx_confirm_details,
};

/// Scan-owned active probe TX is part of the qualified station behavior.
pub const ENABLE_SINGLE_PROBE_EXPERIMENT: bool = true;

pub fn encode_debug_event(event_id: u32, data: u32, output: &mut [u8]) -> Option<usize> {
    if output.len() < 12 {
        return None;
    }
    output[..2].copy_from_slice(&12_u16.to_le_bytes());
    output[2..4].copy_from_slice(&0x0805_u16.to_le_bytes());
    output[4..8].copy_from_slice(&event_id.to_le_bytes());
    output[8..12].copy_from_slice(&data.to_le_bytes());
    Some(12)
}

unsafe fn service_management_request(
    events: &mut tx::MacEventQueue,
    request: &TxRequest<'_>,
    if_id: u8,
    output: &mut [u8],
    publish_response: &mut bool,
) -> Result<usize, crate::wsm::Error> {
    match unsafe { tx::service_host_management_tx(events, Some((request, if_id)), 0) } {
        tx::HostManagementTxReport::Published {
            packet_id,
            bisect_stage,
        } => {
            *publish_response = true;
            if bisect_stage != 0 {
                encode_xr819_tx_confirm_details(packet_id, STATUS_FAILURE, 0, bisect_stage, output)
            } else {
                let edca =
                    unsafe { (platform::mac_register(0x0e64) as *const u32).read_volatile() };
                let quantum0 =
                    unsafe { (platform::mac_register(0x0e70) as *const u32).read_volatile() };
                let quantum1 =
                    unsafe { (platform::mac_register(0x0e74) as *const u32).read_volatile() };
                let metadata =
                    unsafe { (packet_ram::interface_metadata() as *const u8).read_volatile() };
                let secondary =
                    unsafe { (packet_ram::duration_word(0) as *const u8).read_volatile() };
                let event_id = 0x5852_0000 | (edca & 0xffff);
                let data = (quantum0 & 0xff)
                    | ((quantum1 & 0xff) << 8)
                    | (u32::from(metadata) << 16)
                    | (u32::from(secondary) << 24);
                encode_debug_event(event_id, data, output).ok_or(crate::wsm::Error::Truncated)
            }
        }
        tx::HostManagementTxReport::Failed { packet_id, .. } => {
            encode_xr819_tx_confirm(packet_id, STATUS_FAILURE, output)
        }
        _ => {
            *publish_response = false;
            encode_tx_confirm(0, STATUS_FAILURE, output)
        }
    }
}

#[inline(always)]
fn encode_standard_read_mib(
    mib_id: u16,
    transport: &Transport,
    output: &mut [u8],
) -> Result<usize, crate::wsm::Error> {
    if mib_id != 0x100c {
        return encode_read_mib_response(STATUS_FAILURE, mib_id, output);
    }

    let diagnostics = radio::diagnostics();
    let (_scan_channels, scan_max_time) = scan::diagnostic_plan();
    let (dwell_arm, dwell_deadline, dwell_now, _dwell_waits) = scan::diagnostic_dwell();
    let (_scan_status, scan_error) = scan::diagnostic_error();
    let _iq = crate::phy::iq_hardware_diagnostics();
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
        u32::from(diagnostics.last_slot_length) | (u32::from(diagnostics.last_frame_control) << 16),
        u32::from(diagnostics.last_channel) | (u32::from(diagnostics.last_active_channel) << 16),
        diagnostics.last_trailer_word,
        scan_max_time,
        dwell_arm,
        dwell_deadline,
        dwell_now,
        unsafe { (crate::dtcm::MAC_WAKE_MODE.get() as *const u32).read_volatile() },
        unsafe {
            u32::from((crate::dtcm::phy_profile0_ready().get() as *const u8).read_volatile())
                | (u32::from(
                    (crate::dtcm::phy_auxiliary_state().get() as *const u8).read_volatile(),
                ) << 8)
                | (u32::from(
                    (crate::dtcm::phy_startup_observation().get() as *const u8).read_volatile(),
                ) << 16)
                | (u32::from(
                    (crate::dtcm::phy_silicon_variant().get() as *const u8).read_volatile(),
                ) << 24)
        },
        scan_error,
        unsafe { (platform::mac_register(0x0600) as *const u32).read_volatile() },
        unsafe { (platform::mac_register(0x0604) as *const u32).read_volatile() },
        unsafe { (platform::mac_register(0x0608) as *const u32).read_volatile() },
        if ENABLE_SINGLE_PROBE_EXPERIMENT {
            tx::probe_experiment_diagnostic_value()
        } else {
            unsafe { (packet_ram::rx_fifo_base() as *const u32).read_volatile() }
        },
    ];
    host_tx_diagnostics::populate_counters(&mut values, transport);
    #[cfg(feature = "experimental-rx-path-diagnostics")]
    {
        let diagnostics = radio::rx_path_diagnostics();
        values[11] = diagnostics.pending_passes;
        values[12] = diagnostics.blocked_by_host_request;
        values[13] = diagnostics.pending_bytes_max;
        values[14] = diagnostics.host_transfers_max;
        values[15] = diagnostics.decrypt_drops;
        values[16] = diagnostics.decrypt_authentication;
        values[17] = diagnostics.decrypt_missing_key;
        values[18] = diagnostics.auth_group;
        values[19] = diagnostics.auth_unicast;
        values[20] = diagnostics.auth_retry;
        values[21] = diagnostics.auth_last_signature;
    }
    #[cfg(feature = "experimental-service-probe")]
    crate::stage_probe::populate(&mut values);
    #[cfg(feature = "experimental-cycle-probe")]
    crate::cycle_probe::populate(&mut values);
    let mut data = [0_u8; 88];
    for (index, value) in values.into_iter().enumerate() {
        data[index * 4..index * 4 + 4].copy_from_slice(&value.to_le_bytes());
    }
    encode_read_mib_data_response(0, mib_id, &data, output)
}

#[inline(always)]
fn encode_extended_read_mib(
    mib_id: u16,
    transport: &Transport,
    output: &mut [u8],
) -> Result<usize, crate::wsm::Error> {
    #[cfg(feature = "experimental-dynamic-iq-trace")]
    if let Some(length) =
        unsafe { crate::phy::write_dynamic_iq_trace_mib(mib_id, &mut output[12..]) }
    {
        return encode_read_mib_data_response_in_place(0, mib_id, length, output);
    }
    #[cfg(feature = "dtcm-contract-diagnostics")]
    if let Some(length) =
        unsafe { crate::dtcm::write_initialized_image_snapshot_mib(mib_id, &mut output[12..]) }
    {
        return encode_read_mib_data_response_in_place(0, mib_id, length, output);
    }
    encode_standard_read_mib(mib_id, transport, output)
}

fn retain_configuration(
    request: ConfigurationRequest<'_>,
) -> Option<([u8; 6], [TxPowerRange; 2])> {
    configuration::retain(&request).ok()?;
    Some((
        configuration::snapshot()?.station_id,
        configuration::tx_power_ranges()?,
    ))
}

/// Service at most one host request from the command lane.
///
/// # Safety
/// The caller must hold the unique cooperative owners for `events`,
/// `mac_domain`, `transport`, and `host_tx_driver` and must not re-enter this
/// lane before the call returns.
/// Outcome of one dispatched request: whether the multi-dispatch loop may
/// consume another TX request in the same pass.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SingleOutcome {
    AdmittedTx,
    Settled,
}

/// Maximum TX admissions per pass under multi-dispatch: one full depth-4
/// batch. The first sync command, failed admission, or empty queue ends the
/// loop, preserving single-shot semantics for everything but TX refill.
#[cfg(feature = "experimental-multi-dispatch")]
const MAX_TX_ADMIT_PER_PASS: u8 = 4;

pub unsafe fn service_one(
    transport: &mut Transport,
    events: &mut tx::MacEventQueue,
    mac_domain: &mut MacDomain,
    host_tx_driver: &mut HostTxDriver,
    response_scratch: &mut [u8; SHARED_BUFFER_SIZE],
    pending_join_complete: &mut Option<u32>,
) {
    if !transport.publication_available() {
        return;
    }
    #[cfg(feature = "experimental-multi-dispatch")]
    {
        // Vendor `hif_rx_process()` reschedules itself while descriptors are
        // ready instead of waiting for the next pass. Drain up to a full
        // batch of TX admissions here; anything else settles the pass.
        let mut admitted = 0_u8;
        loop {
            let Some(request) = transport.poll_request() else {
                break;
            };
            let outcome = unsafe {
                dispatch_single_request(
                    transport,
                    events,
                    mac_domain,
                    host_tx_driver,
                    &mut *response_scratch,
                    pending_join_complete,
                    request,
                )
            };
            if outcome == SingleOutcome::Settled {
                break;
            }
            admitted += 1;
            if admitted >= MAX_TX_ADMIT_PER_PASS {
                break;
            }
        }
        return;
    }
    let Some(request) = transport.poll_request() else {
        return;
    };
    unsafe {
        dispatch_single_request(
            transport,
            events,
            mac_domain,
            host_tx_driver,
            &mut *response_scratch,
            pending_join_complete,
            request,
        );
    }
}

/// One dispatched request: the former `service_one` body. Reports whether the
/// multi-dispatch loop may consume another TX request in the same pass.
#[allow(clippy::too_many_arguments)]
unsafe fn dispatch_single_request(
    transport: &mut Transport,
    events: &mut tx::MacEventQueue,
    mac_domain: &mut MacDomain,
    host_tx_driver: &mut HostTxDriver,
    response_scratch: &mut [u8; SHARED_BUFFER_SIZE],
    pending_join_complete: &mut Option<u32>,
    request: ReceivedRequest,
) -> SingleOutcome {

    let output: &mut [u8] = &mut response_scratch[..];
    let mut publish_response = true;
    let request_id = request.id;
    let request_if_id = request.if_id;
    unsafe {
        host_tx_diagnostics::record_hif_event(1, request_id, request_if_id);
    }
    let mut request_buffer = Some(request.buffer);
    let mut outcome = SingleOutcome::Settled;
    let request_payload = request_buffer
        .as_ref()
        .expect("request buffer is present")
        .payload();
    let response_length = if request_if_id > 2 {
        encode_status_response(request_id | 0x0400, STATUS_FAILURE, output)
    } else if request_id == CONFIGURATION_REQ_ID {
        let configured = match ConfigurationRequest::parse(request_payload) {
            Ok(request) => retain_configuration(request),
            Err(_) => None,
        };
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
            Ok(scan_request) => match scan::begin(&scan_request, request_if_id) {
                Ok(()) => 0,
                Err(scan::ScanError::Busy) => 4,
                Err(_) => 2,
            },
            Err(_) => 2,
        };
        encode_status_response(request_id | 0x0400, status, output)
    } else if request_id == SET_BSS_PARAMS_REQ_ID {
        // JOIN already publishes the active VIF, channel, and rate state used by
        // this firmware. Accept the driver's post-association beacon-loss/AID
        // policy so it can complete SAE/PMF setup; beacon-loss offload remains
        // intentionally host-managed until that event path is implemented.
        let status = SetBssParameters::parse(request_payload)
            .map(|_| 0)
            .unwrap_or(STATUS_FAILURE);
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
            Ok(request) if request.mib_id == 0x1006 && request.data.len() == 4 => 0,
            Ok(request) if request.mib_id == rate_policy::MIB_ID_SET_TX_RATE_RETRY_POLICY => {
                rate_policy::install(request.data)
                    .map(|()| 0)
                    .unwrap_or(STATUS_FAILURE)
            }
            Ok(request) if configuration::retain_interface_mib(request.mib_id, request.data) => 0,
            _ => STATUS_FAILURE,
        };
        encode_status_response(request_id | 0x0400, status, output)
    } else if request_id == READ_MIB_REQ_ID {
        let mib_id = request_payload
            .get(..2)
            .map(|value| u16::from_le_bytes([value[0], value[1]]))
            .unwrap_or(0);
        encode_extended_read_mib(mib_id, &*transport, output)
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
                unsafe {
                    host_tx_driver.reset(mac_domain);
                }
                if unsafe { join::reset(request_if_id) } {
                    0
                } else {
                    STATUS_FAILURE
                }
            }
            Err(_) => STATUS_FAILURE,
        };
        encode_status_response(request_id | 0x0400, status, output)
    } else if request_id == JOIN_REQ_ID {
        let status = match JoinRequest::parse(request_payload) {
            Ok(join_request) => unsafe {
                join::activate_sta(request_if_id, &join_request)
                    .map(|_| 0)
                    .unwrap_or(STATUS_FAILURE)
            },
            Err(_) => STATUS_FAILURE,
        };
        if status == 0
            && request_payload
                .get(0x0f)
                .is_some_and(|flags| flags & 0x20 != 0)
        {
            *pending_join_complete = Some(0);
        }
        encode_join_response(status, -160, 200, output)
    } else if request_id == TX_REQ_ID {
        unsafe {
            host_tx_diagnostics::trace(
                0x4854_0004,
                u32::from(request_if_id)
                    | (u32::try_from(request_payload.len()).unwrap_or(u32::MAX) << 8),
                0,
            );
        }
        match TxRequest::parse(request_payload) {
            Ok(tx_request) => {
                unsafe {
                    let frame_control =
                        u16::from_le_bytes([tx_request.frame[0], tx_request.frame[1]]);
                    host_tx_diagnostics::trace(
                        0x4854_0005,
                        u32::from(frame_control)
                            | (u32::try_from(tx_request.frame.len()).unwrap_or(u32::MAX) << 16),
                        u32::from(tx_request.is_unicast_data())
                            | (u32::from(tx_request.is_unicast_eapol()) << 1),
                    );
                }
                // A BlockAckReq is a control frame and goes through the host TX path so
                // that its completion produces the WSM confirmation mac80211 waits on;
                // the internal class-6 context behind the management publisher cannot.
                if (tx_request.is_unicast_data() && !tx_request.is_unicast_eapol())
                    || tx_request.is_unicast_control()
                {
                    let packet_id = tx_request.packet_id;
                    let admitted = unsafe {
                        host_tx_driver.admit(
                            request_buffer.take().expect("request buffer is present"),
                            request_if_id,
                            transport,
                        )
                    };
                    if admitted {
                        publish_response = false;
                        outcome = SingleOutcome::AdmittedTx;
                        #[cfg(feature = "experimental-cycle-probe")]
                        crate::cycle_probe::note_admit();
                        Ok(0)
                    } else {
                        unsafe {
                            host_tx_diagnostics::trace(0x4854_00e1, packet_id, 0);
                        }
                        encode_xr819_tx_confirm(packet_id, STATUS_FAILURE, output)
                    }
                } else if !host_tx_driver.management_runtime_available() {
                    encode_xr819_tx_confirm(tx_request.packet_id, STATUS_FAILURE, output)
                } else {
                    unsafe {
                        service_management_request(
                            events,
                            &tx_request,
                            request_if_id,
                            output,
                            &mut publish_response,
                        )
                    }
                }
            }
            Err(_) => {
                let packet_id = request_payload
                    .get(..4)
                    .map(|value| u32::from_le_bytes([value[0], value[1], value[2], value[3]]))
                    .unwrap_or(0);
                encode_xr819_tx_confirm(packet_id, STATUS_FAILURE, output)
            }
        }
    } else {
        // Do not report success for commands whose state effects are not
        // implemented. A complete status word lets cw1200 fail the command
        // cleanly instead of proceeding on false assumptions.
        encode_status_response(request_id | 0x0400, STATUS_FAILURE, output)
    };

    if publish_response && let Ok(length) = response_length {
        let response_id = u16::from_le_bytes([output[2], output[3]]) & 0x1fff;
        unsafe {
            host_tx_diagnostics::record_hif_event(2, response_id, length as u8);
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

    // Synchronous commands return their owning request buffer here. Ordinary
    // class-0 TX moves it into HostTxDriver until confirmation.
    if let Some(buffer) = request_buffer {
        transport.release_request(buffer.into_release());
    }
    outcome
}
