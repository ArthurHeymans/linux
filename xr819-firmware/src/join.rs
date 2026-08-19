//! Minimal STA JOIN activation boundary.
//!
//! Ordinary WSM TX carries the authentication and association frames Linux
//! sends after JOIN success.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JoinError {
    Unsupported,
    NotConfigured,
    ScanActive,
    VifState,
    Channel,
}

/// Activate the minimal vendor STA-BSS state and program its operating channel.
///
/// # Safety
/// JOIN must exclusively own channel, VIF/PAS, and MAC address-match state.
#[cfg(target_arch = "arm")]
pub unsafe fn activate_sta(
    interface: u8,
    request: &crate::wsm::JoinRequest<'_>,
) -> Result<(), JoinError> {
    if interface > 1 || !request.supported_sta_shape() {
        return Err(JoinError::Unsupported);
    }
    if crate::scan::active_interface().is_some() {
        return Err(JoinError::ScanActive);
    }
    if !crate::tx::probe_runtime_quiescent() {
        return Err(JoinError::VifState);
    }
    let configuration = crate::configuration::snapshot().ok_or(JoinError::NotConfigured)?;
    crate::vif::join_gate(interface, request.channel_number).map_err(|_| JoinError::VifState)?;

    unsafe {
        let _ = crate::vif::teardown(interface);
        if crate::phy::run_channel_transition(request.channel_number, 100_000).is_err() {
            let _ = crate::vif::teardown(interface);
            return Err(JoinError::Channel);
        }
        crate::vif::activate_sta(interface, request, configuration.station_id)
            .map_err(|_| JoinError::VifState)?;
        if !crate::mac::program_active_vif_rate_tables(interface) {
            let _ = crate::vif::teardown(interface);
            return Err(JoinError::VifState);
        }
        crate::mac::program_immediate_response_descriptors();
        crate::mac::program_joined_bssid(request.bssid);
        // Vendor STA mode installs the receive/address-match state required for
        // authentication and association responses after BSSID publication.
        crate::mac::program_joined_station_mode();
    }
    Ok(())
}

#[cfg(target_arch = "arm")]
pub unsafe fn reset(interface: u8) -> bool {
    if !crate::vif::is_active(interface) {
        return interface < 2;
    }
    if !crate::tx::probe_runtime_quiescent() {
        return false;
    }
    if !unsafe { crate::vif::teardown(interface) } {
        return false;
    }
    if crate::vif::any_active() {
        return true;
    }
    // Preserve the running RX/channel owner until the next scan or JOIN. The
    // full vendor stop depends on ordinary-TX scheduler drain work that is not
    // represented yet; forcing it here made the post-auth scan lose completion.
    unsafe {
        (crate::platform::mac_register(0x0044) as *mut u32).write_volatile(0);
        crate::mac::program_scan_station_mode();
    }
    true
}
