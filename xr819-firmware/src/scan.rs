//! Retained WSM scan plan and cooperative scan state.
//!
//! This module owns the host request after the borrowed HIF RX buffer is
//! recycled, iterates every retained channel, and holds each successful tuning
//! result for the host-provided maximum dwell time.

use core::cell::UnsafeCell;

use crate::configuration::{self, Mode0ChannelCalibration};
use crate::phy::{
    ChannelProgramRequestWire, ChannelTransitionScheduler, ChannelTunePlan, PllDivider,
    build_scan_channel_program_request, channel_control_word, channel_frequency_khz_2ghz,
    channel_tune_plan, pll_divider,
};
use crate::wsm::{ScanChannel, StartScanRequest};

pub const MAX_SCAN_CHANNELS: usize = 48;
pub const VENDOR_MAX_SCAN_CHANNELS: usize = 34;
pub const MAX_SCAN_SSIDS: usize = 2;
pub const MAX_SSID_LEN: usize = 32;

fn deadline_reached(now: u32, deadline: u32) -> bool {
    now.wrapping_sub(deadline) as i32 >= 0
}

fn transition_error_code(error: crate::phy::ChannelTransitionError) -> u32 {
    match error {
        crate::phy::ChannelTransitionError::Pll(_) => 1,
        crate::phy::ChannelTransitionError::InvalidTiming => 2,
        crate::phy::ChannelTransitionError::Temperature(_) => 3,
        crate::phy::ChannelTransitionError::Calibration(error) => match error {
            crate::phy::IqCalibrationHardwareError::BaselineTimeout { gain_index } => {
                0x400 + gain_index
            }
            crate::phy::IqCalibrationHardwareError::TargetTimeout { gain_index } => {
                0x500 + gain_index
            }
            crate::phy::IqCalibrationHardwareError::SecondaryBaselineTimeout => 0x600,
            crate::phy::IqCalibrationHardwareError::SecondaryTargetTimeout => 0x601,
            crate::phy::IqCalibrationHardwareError::InvalidSecondaryScale => 0x602,
        },
        crate::phy::ChannelTransitionError::Power(_) => 5,
        crate::phy::ChannelTransitionError::MacWake(_) => 6,
        crate::phy::ChannelTransitionError::Gain(_) => 7,
    }
}

#[cfg(target_arch = "arm")]
fn vendor_timer() -> u32 {
    unsafe {
        (0x0ac0_0004 as *const u32)
            .read_volatile()
            .wrapping_add((0x0400_143c as *const u32).read_volatile())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScanError {
    TooManyChannels,
    TooManySsids,
    InvalidRecord,
    InvalidChannelTiming,

    Busy,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RetainedScanChannel {
    pub number: u16,
    pub min_channel_time: u32,
    pub max_channel_time: u32,
}

impl From<ScanChannel> for RetainedScanChannel {
    fn from(channel: ScanChannel) -> Self {
        Self {
            number: channel.number,
            min_channel_time: channel.min_channel_time,
            max_channel_time: channel.max_channel_time,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScanCompletion {
    pub status: u32,
    pub psm: u8,
    pub num_channels: u8,
    pub vendor_field: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProbeOpportunity {
    pub generation: u32,
    pub if_id: u8,
    pub channel: u8,
    pub ssid_index: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ActiveProbePhase {
    Disabled,
    WaitingForTune,
    WaitingForDelay,
    InFlight,
    Done,
    Failed,
}

struct ScanStorage {
    active: bool,
    completion_delay: u8,
    hardware_tune_pending: bool,
    hardware_status: u32,
    current_channel_index: u8,
    dwell_deadline: u32,
    transition: ChannelTransitionScheduler,
    started_at: u32,
    elapsed_ticks: u32,
    dwell_arm_now: u32,
    dwell_armed_deadline: u32,
    dwell_last_now: u32,
    dwell_waits: u32,
    hardware_error_code: u32,
    finish_state: u8,
    finish_deadline: u32,
    if_id: u8,
    band: u8,
    scan_type: u8,
    flags: u8,
    num_probes: u8,
    probe_delay: u8,
    num_channels: u8,
    channels: [RetainedScanChannel; MAX_SCAN_CHANNELS],
    num_ssids: u8,
    ssid_lengths: [u8; MAX_SCAN_SSIDS],
    ssids: [[u8; MAX_SSID_LEN]; MAX_SCAN_SSIDS],
    probe_deadline: u32,
    probe_generation: u32,
    probe_phase: ActiveProbePhase,
    probe_round: u8,
    probe_ssid_index: u8,
    probe_last_status: u16,
    /// Destructive boot-wide guard. Increase only after the previous bounded
    /// publication/reclamation count has completed repeatedly on hardware.
    probe_publication_budget: u8,
}

impl ScanStorage {
    const fn new() -> Self {
        Self {
            active: false,
            completion_delay: 0,
            hardware_tune_pending: false,
            hardware_status: 0,
            current_channel_index: 0,
            dwell_deadline: 0,
            transition: ChannelTransitionScheduler::new(),
            started_at: 0,
            elapsed_ticks: 0,
            dwell_arm_now: 0,
            dwell_armed_deadline: 0,
            dwell_last_now: 0,
            dwell_waits: 0,
            hardware_error_code: 0,
            finish_state: 0,
            finish_deadline: 0,
            if_id: 0,
            band: 0,
            scan_type: 0,
            flags: 0,
            num_probes: 0,
            probe_delay: 0,
            num_channels: 0,
            channels: [RetainedScanChannel {
                number: 0,
                min_channel_time: 0,
                max_channel_time: 0,
            }; MAX_SCAN_CHANNELS],
            num_ssids: 0,
            ssid_lengths: [0; MAX_SCAN_SSIDS],
            ssids: [[0; MAX_SSID_LEN]; MAX_SCAN_SSIDS],
            probe_deadline: 0,
            probe_generation: 0,
            probe_phase: ActiveProbePhase::Disabled,
            probe_round: 0,
            probe_ssid_index: 0,
            probe_last_status: 0,
            probe_publication_budget: 4,
        }
    }
}

struct SharedScan(UnsafeCell<ScanStorage>);

unsafe impl Sync for SharedScan {}

static SCAN: SharedScan = SharedScan(UnsafeCell::new(ScanStorage::new()));

#[cfg(target_arch = "arm")]
fn set_vendor_scan_active(active: bool) {
    unsafe {
        (0x0400_860c as *mut u8).write_volatile(u8::from(active));
        if active {
            crate::tx::set_scheduler_bits(1 << 10);
        } else {
            crate::tx::clear_scheduler_bits(1 << 10);
        }
    }
}

#[cfg(not(target_arch = "arm"))]
fn set_vendor_scan_active(_active: bool) {}

pub fn begin(request: &StartScanRequest<'_>, if_id: u8) -> Result<(), ScanError> {
    let num_channels = usize::from(request.num_channels);
    let num_ssids = usize::from(request.num_ssids);
    if if_id > 2 {
        return Err(ScanError::InvalidRecord);
    }
    if num_channels > VENDOR_MAX_SCAN_CHANNELS {
        return Err(ScanError::TooManyChannels);
    }
    if num_ssids > MAX_SCAN_SSIDS {
        return Err(ScanError::TooManySsids);
    }

    let storage = unsafe { &mut *SCAN.0.get() };
    if storage.active {
        return Err(ScanError::Busy);
    }

    // Vendor 0x13d44 validates every channel before copying the request into
    // retained scan state.
    for index in 0..num_channels {
        let channel = request
            .channel(index)
            .map_err(|_| ScanError::InvalidRecord)?;
        if channel.max_channel_time == 0
            || channel.max_channel_time < channel.min_channel_time
            || u32::from(request.probe_delay) > channel.min_channel_time.wrapping_mul(0x400)
        {
            return Err(ScanError::InvalidChannelTiming);
        }
    }

    storage.if_id = if_id;
    storage.band = request.band;
    storage.scan_type = request.scan_type;
    storage.flags = request.flags;
    // Active builds retain the host's requested dwell. The no-default-features
    // image remains the conservative passive rollback.
    let passive_fallback = false;
    {
        storage.num_probes = request.num_probes;
        storage.probe_delay = request.probe_delay;
    }
    storage.num_channels = request.num_channels;
    for index in 0..num_channels {
        let mut channel: RetainedScanChannel = request
            .channel(index)
            .map(Into::into)
            .map_err(|_| ScanError::InvalidRecord)?;
        if passive_fallback {
            if request.num_channels == 1 {
                // Two beacon intervals compensate for absent probe responses.
                channel.min_channel_time = channel.min_channel_time.max(220);
                channel.max_channel_time = channel.max_channel_time.max(250);
            } else {
                // Keep multi-channel batches below the driver's scan-command
                // timeout until active probe TX is available.
                channel.min_channel_time = channel.min_channel_time.max(110);
                channel.max_channel_time = channel.max_channel_time.max(120);
            }
        }
        storage.channels[index] = channel;
    }

    storage.num_ssids = request.num_ssids;
    for index in 0..num_ssids {
        let ssid = request.ssid(index).map_err(|_| ScanError::InvalidRecord)?;
        storage.ssid_lengths[index] = ssid.len() as u8;
        storage.ssids[index][..ssid.len()].copy_from_slice(ssid);
        storage.ssids[index][ssid.len()..].fill(0);
    }

    storage.completion_delay = 0;
    storage.hardware_tune_pending = request.num_channels != 0;
    storage.hardware_status = 0;
    storage.current_channel_index = 0;
    storage.dwell_deadline = 0;
    storage.transition = ChannelTransitionScheduler::new();
    #[cfg(target_arch = "arm")]
    {
        storage.started_at = vendor_timer();
    }
    storage.elapsed_ticks = 0;
    storage.dwell_arm_now = 0;
    storage.dwell_armed_deadline = 0;
    storage.dwell_last_now = 0;
    storage.dwell_waits = 0;
    storage.hardware_error_code = 0;
    {
        storage.finish_state = 0;
        storage.finish_deadline = 0;
    }
    {
        storage.probe_publication_budget = 4;
        storage.probe_deadline = 0;
        storage.probe_generation = storage.probe_generation.wrapping_add(1);
        storage.probe_phase = if storage.num_probes != 0
            && storage.num_channels != 0
            && storage.probe_publication_budget != 0
        {
            ActiveProbePhase::WaitingForTune
        } else {
            ActiveProbePhase::Disabled
        };
        storage.probe_round = 0;
        storage.probe_ssid_index = 0;
        storage.probe_last_status = 0;
    }
    storage.active = true;
    set_vendor_scan_active(true);
    Ok(())
}

#[cfg(target_arch = "arm")]
unsafe fn service_unjoined_scan_finish(storage: &mut ScanStorage) -> bool {
    match storage.finish_state {
        0 => false,
        1 => {
            if !crate::tx::probe_runtime_quiescent() || crate::radio::host_transfer_outstanding() {
                return false;
            }
            unsafe {
                // The class-6 callback raises vendor scheduler bit 21 after
                // returning the context. For the single-probe domain there is
                // no remaining TX confirmation work, so consume that narrow
                // scheduler task before resetting MAC state.
                crate::tx::clear_scheduler_bits(1 << 21);
                crate::mac::begin_unjoined_scan_radio_stop();
                crate::phy::begin_scan_stop_rx_disable();
            }
            storage.finish_deadline = vendor_timer().wrapping_add(0x0040_0000);
            storage.finish_state = 2;
            false
        }
        2 => {
            if !unsafe { crate::phy::scan_stop_rx_hardware_drained() } {
                if (vendor_timer().wrapping_sub(storage.finish_deadline) as i32) >= 0 {
                    unsafe { crate::tx::fatal_scan_stop_timeout() };
                }
                return false;
            }
            unsafe {
                crate::tx::stop_phy_operation_7();
                crate::phy::start_scan_stop_calibration_state();
            }
            storage.finish_state = 3;
            false
        }
        3 => {
            if !crate::radio::fifo_quiescent() {
                let index = usize::from(storage.num_channels.saturating_sub(1));
                unsafe { crate::radio::discard_one_idle(storage.channels[index].number) };
                return false;
            }
            unsafe {
                if crate::vif::clear_scan_channel().is_err() {
                    crate::halt_always!();
                }
                crate::mac::finish_unjoined_scan_radio_stop();
            }
            storage.finish_state = 4;
            true
        }
        _ => true,
    }
}

fn publish_scan_completion(storage: &mut ScanStorage) -> Option<ScanCompletion> {
    storage.active = false;
    #[cfg(target_arch = "arm")]
    {
        storage.elapsed_ticks = vendor_timer().wrapping_sub(storage.started_at);
    }
    set_vendor_scan_active(false);
    Some(ScanCompletion {
        status: storage.hardware_status,
        psm: 0,
        num_channels: storage.num_channels,
        vendor_field: 0,
    })
}

/// Advance tuning and dwell for the cooperative scan engine.
pub fn service() -> Option<ScanCompletion> {
    let storage = unsafe { &mut *SCAN.0.get() };
    if !storage.active {
        return None;
    }

    #[cfg(target_arch = "arm")]
    if storage.finish_state != 0 {
        if !unsafe { service_unjoined_scan_finish(storage) } {
            return None;
        }
        return publish_scan_completion(storage);
    }

    #[cfg(target_arch = "arm")]
    if storage.hardware_status == 0 && !storage.transition.is_idle() {
        let now = vendor_timer();
        match unsafe { storage.transition.service(now) } {
            Ok(Some(_result)) => {
                let channel = storage.channels[usize::from(storage.current_channel_index)];
                // Vendor arms the dwell after the transition has enabled RX.
                let armed = vendor_timer();
                storage.dwell_deadline =
                    armed.wrapping_add(channel.max_channel_time.saturating_mul(0x400));
                {
                    storage.probe_deadline = armed.wrapping_add(u32::from(storage.probe_delay));
                    storage.probe_phase =
                        if storage.num_probes == 0 || storage.probe_publication_budget == 0 {
                            ActiveProbePhase::Disabled
                        } else {
                            ActiveProbePhase::WaitingForDelay
                        };
                    storage.probe_round = 0;
                    storage.probe_ssid_index = 0;
                }
                storage.dwell_arm_now = armed;
                storage.dwell_armed_deadline = storage.dwell_deadline;
            }
            Ok(None) => return None,
            Err(error) => {
                storage.hardware_error_code = transition_error_code(error);
                storage.hardware_status = 1;
                {
                    storage.probe_phase = ActiveProbePhase::Failed;
                }
            }
        }
    }

    if storage.hardware_status == 0 && storage.hardware_tune_pending {
        storage.hardware_tune_pending = false;
        let index = usize::from(storage.current_channel_index);
        let _channel = storage.channels[index];
        #[cfg(target_arch = "arm")]
        {
            if let Err(error) = unsafe { storage.transition.start(_channel.number, 100_000) } {
                storage.hardware_error_code = transition_error_code(error);
                storage.hardware_status = 1;
            } else {
                storage.transition.arm_settle(vendor_timer());
            }
        }
        #[cfg(not(target_arch = "arm"))]
        {
            storage.completion_delay = 1;
        }
        return None;
    }

    if storage.hardware_status == 0 {
        if !matches!(
            storage.probe_phase,
            ActiveProbePhase::Disabled | ActiveProbePhase::Done | ActiveProbePhase::Failed
        ) {
            return None;
        }

        #[cfg(target_arch = "arm")]
        {
            let now = vendor_timer();
            storage.dwell_last_now = now;
            if !deadline_reached(now, storage.dwell_deadline) {
                storage.dwell_waits = storage.dwell_waits.wrapping_add(1);
                return None;
            }
        }
        #[cfg(not(target_arch = "arm"))]
        {
            if storage.completion_delay != 0 {
                storage.completion_delay -= 1;
                return None;
            }
        }

        storage.current_channel_index = storage.current_channel_index.wrapping_add(1);
        if storage.current_channel_index < storage.num_channels {
            storage.hardware_tune_pending = true;
            {
                storage.probe_phase =
                    if storage.num_probes == 0 || storage.probe_publication_budget == 0 {
                        ActiveProbePhase::Disabled
                    } else {
                        ActiveProbePhase::WaitingForTune
                    };
            }
            return None;
        }
    }

    #[cfg(target_arch = "arm")]
    if storage.num_probes != 0 && storage.hardware_status == 0 {
        // JOIN is not accepted yet, but distinguish the vendor active-VIF
        // branch now so future activation can never destructively execute the
        // no-VIF radio-stop path. The restore implementation lands with JOIN.
        if crate::vif::any_active() {
            storage.hardware_status = 1;
            storage.hardware_error_code = 9;
            return publish_scan_completion(storage);
        }
        storage.finish_state = 1;
        return None;
    }

    publish_scan_completion(storage)
}

pub fn elapsed_ticks() -> u32 {
    let storage = unsafe { &*SCAN.0.get() };
    storage.elapsed_ticks
}

pub fn diagnostic_plan() -> (u8, u32) {
    let storage = unsafe { &*SCAN.0.get() };
    let max_time = if storage.num_channels == 0 {
        0
    } else {
        storage.channels[0].max_channel_time
    };
    (storage.num_channels, max_time)
}

pub fn diagnostic_error() -> (u32, u32) {
    let storage = unsafe { &*SCAN.0.get() };
    (storage.hardware_status, storage.hardware_error_code)
}

pub fn diagnostic_dwell() -> (u32, u32, u32, u32) {
    let storage = unsafe { &*SCAN.0.get() };
    (
        storage.dwell_arm_now,
        storage.dwell_armed_deadline,
        storage.dwell_last_now,
        storage.dwell_waits,
    )
}

pub fn active_interface() -> Option<u8> {
    let storage = unsafe { &*SCAN.0.get() };
    storage.active.then_some(storage.if_id)
}

pub fn active_channel() -> Option<u16> {
    let storage = unsafe { &*SCAN.0.get() };
    (storage.active
        && storage.hardware_status == 0
        && !storage.hardware_tune_pending
        && storage.transition.is_idle()
        && storage.current_channel_index < storage.num_channels)
        .then_some(storage.channels[usize::from(storage.current_channel_index)].number)
}

/// Claims the next probe opportunity for the current channel. An opportunity
/// is visible only after tuning, RX enable, and the scheduled probe delay.
fn claim_probe_opportunity_at(storage: &mut ScanStorage, now: u32) -> Option<ProbeOpportunity> {
    if !storage.active
        || storage.hardware_status != 0
        || storage.hardware_tune_pending
        || !storage.transition.is_idle()
        || storage.current_channel_index >= storage.num_channels
        || storage.num_probes == 0
        || storage.probe_phase != ActiveProbePhase::WaitingForDelay
        || !deadline_reached(now, storage.probe_deadline)
    {
        return None;
    }

    let channel =
        u8::try_from(storage.channels[usize::from(storage.current_channel_index)].number).ok()?;
    storage.probe_phase = ActiveProbePhase::InFlight;
    Some(ProbeOpportunity {
        generation: storage.probe_generation,
        if_id: storage.if_id,
        channel,
        ssid_index: storage.probe_ssid_index,
    })
}

#[cfg(target_arch = "arm")]
pub fn claim_probe_opportunity() -> Option<ProbeOpportunity> {
    let storage = unsafe { &mut *SCAN.0.get() };
    claim_probe_opportunity_at(storage, vendor_timer())
}

#[cfg(not(target_arch = "arm"))]
pub const fn claim_probe_opportunity() -> Option<ProbeOpportunity> {
    None
}

pub fn copy_probe_ssid(
    opportunity: ProbeOpportunity,
    output: &mut [u8; MAX_SSID_LEN],
) -> Option<usize> {
    let storage = unsafe { &*SCAN.0.get() };
    if storage.probe_generation != opportunity.generation
        || storage.probe_phase != ActiveProbePhase::InFlight
        || storage.if_id != opportunity.if_id
        || storage.probe_ssid_index != opportunity.ssid_index
    {
        return None;
    }
    if storage.num_ssids == 0 {
        output.fill(0);
        return Some(0);
    }
    let index = usize::from(opportunity.ssid_index);
    if index >= usize::from(storage.num_ssids) {
        return None;
    }
    let length = usize::from(storage.ssid_lengths[index]);
    output[..length].copy_from_slice(&storage.ssids[index][..length]);
    output[length..].fill(0);
    Some(length)
}

fn complete_probe_at(
    storage: &mut ScanStorage,
    opportunity: ProbeOpportunity,
    status: u16,
    now: u32,
) -> bool {
    if storage.probe_generation != opportunity.generation
        || storage.probe_phase != ActiveProbePhase::InFlight
        || storage.if_id != opportunity.if_id
        || storage.probe_ssid_index != opportunity.ssid_index
    {
        return false;
    }

    storage.probe_last_status = status;
    storage.probe_publication_budget = storage.probe_publication_budget.saturating_sub(1);
    let ssid_count = storage.num_ssids.max(1);
    storage.probe_ssid_index = storage.probe_ssid_index.wrapping_add(1);
    if storage.probe_ssid_index >= ssid_count {
        storage.probe_ssid_index = 0;
        storage.probe_round = storage.probe_round.wrapping_add(1);
    }

    if storage.probe_round >= storage.num_probes || storage.probe_publication_budget == 0 {
        storage.probe_phase = ActiveProbePhase::Done;
    } else {
        // The XR819 driver's dwell calculation budgets two milliseconds per
        // SSID/probe pair. Serialize publications through legal context return
        // and schedule the next pair at that same conservative interval.
        storage.probe_deadline = now.wrapping_add(2 * 0x400);
        storage.probe_phase = ActiveProbePhase::WaitingForDelay;
    }
    true
}

#[cfg(target_arch = "arm")]
pub fn complete_probe(opportunity: ProbeOpportunity, status: u16) -> bool {
    let storage = unsafe { &mut *SCAN.0.get() };
    complete_probe_at(storage, opportunity, status, vendor_timer())
}

#[cfg(not(target_arch = "arm"))]
pub fn complete_probe(opportunity: ProbeOpportunity, status: u16) -> bool {
    let storage = unsafe { &mut *SCAN.0.get() };
    complete_probe_at(storage, opportunity, status, storage.probe_deadline)
}

pub fn fail_probe(opportunity: ProbeOpportunity) -> bool {
    let storage = unsafe { &mut *SCAN.0.get() };
    if storage.probe_generation != opportunity.generation
        || storage.probe_phase != ActiveProbePhase::InFlight
    {
        return false;
    }
    storage.probe_phase = ActiveProbePhase::Failed;
    storage.hardware_status = 1;
    true
}

pub fn channel(index: usize) -> Option<RetainedScanChannel> {
    let storage = unsafe { &*SCAN.0.get() };
    (index < usize::from(storage.num_channels)).then_some(storage.channels[index])
}

pub fn channel_control(index: usize) -> Option<u16> {
    let storage = unsafe { &*SCAN.0.get() };
    (index < usize::from(storage.num_channels))
        .then(|| channel_control_word(storage.band, 0, storage.channels[index].number))
}

pub fn channel_program_request(index: usize) -> Option<ChannelProgramRequestWire> {
    let storage = unsafe { &*SCAN.0.get() };
    (index < usize::from(storage.num_channels)).then(|| {
        build_scan_channel_program_request(
            storage.band,
            storage.flags,
            storage.channels[index].number,
        )
    })
}

pub fn channel_tuning(index: usize) -> Option<ChannelTunePlan> {
    channel_program_request(index)
        .map(|request| channel_tune_plan(request.operation, request.control.get()))
}

pub fn channel_frequency_khz(index: usize) -> Option<u32> {
    let storage = unsafe { &*SCAN.0.get() };
    (storage.band == 0 && index < usize::from(storage.num_channels))
        .then(|| channel_frequency_khz_2ghz(storage.channels[index].number))
}

pub fn channel_pll(index: usize) -> Option<PllDivider> {
    pll_divider(channel_frequency_khz(index)?, 1250, 26_000)
}

pub fn channel_calibration(index: usize) -> Option<Mode0ChannelCalibration> {
    let channel = channel(index)?;
    configuration::mode0_channel_calibration(channel.number as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retains_scan_records_after_request_buffer_is_gone() {
        let mut payload = [0; 12 + 32];
        payload[9] = 2;
        payload[12..14].copy_from_slice(&6_u16.to_le_bytes());
        payload[16..20].copy_from_slice(&10_u32.to_le_bytes());
        payload[20..24].copy_from_slice(&30_u32.to_le_bytes());
        payload[28..30].copy_from_slice(&11_u16.to_le_bytes());
        payload[32..36].copy_from_slice(&20_u32.to_le_bytes());
        payload[36..40].copy_from_slice(&40_u32.to_le_bytes());
        let Ok(request) = StartScanRequest::parse(&payload) else {
            panic!("valid scan request did not parse");
        };

        assert_eq!(begin(&request, 1), Ok(()));
        assert_eq!(active_interface(), Some(1));
        assert_eq!(channel(0).map(|channel| channel.number), Some(6));
        assert_eq!(channel_control(0), Some(0x0117));
        assert_eq!(
            channel_program_request(0).map(|request| request.control.get()),
            Some(0x0117)
        );
        assert_eq!(
            channel_tuning(0),
            Some(ChannelTunePlan {
                phy_mode: 2,
                recalibrate: true,
            })
        );
        assert_eq!(channel_frequency_khz(0), Some(2_437_000));
        assert_eq!(
            channel_pll(0).map(|divider| divider.register),
            Some(0x356e_c4ec)
        );
        assert_eq!(channel(1).map(|channel| channel.number), Some(11));
        assert_eq!(service(), None);
        assert_eq!(service(), None);
        assert_eq!(service(), None);
        assert_eq!(service(), None);
        assert_eq!(service(), None);
        assert_eq!(
            service(),
            Some(ScanCompletion {
                status: 0,
                psm: 0,
                num_channels: 2,
                vendor_field: 0,
            })
        );

        payload[20..24].copy_from_slice(&0_u32.to_le_bytes());
        let Ok(request) = StartScanRequest::parse(&payload) else {
            panic!("invalid-timing scan request did not parse");
        };
        assert_eq!(begin(&request, 1), Err(ScanError::InvalidChannelTiming));
    }

    #[test]
    fn dwell_deadline_comparison_handles_timer_wrap() {
        assert!(!deadline_reached(0xffff_fff0, 0x0000_0010));
        assert!(deadline_reached(0x0000_0010, 0xffff_fff0));
        assert!(deadline_reached(1234, 1234));
    }

    #[test]
    fn active_probe_state_serializes_requested_publications() {
        let mut storage = ScanStorage::new();
        storage.active = true;
        storage.if_id = 1;
        storage.num_channels = 1;
        storage.channels[0].number = 1;
        storage.num_probes = 2;
        storage.probe_generation = 7;
        storage.probe_deadline = 100;
        storage.probe_phase = ActiveProbePhase::WaitingForDelay;
        storage.probe_publication_budget = 2;

        assert_eq!(claim_probe_opportunity_at(&mut storage, 99), None);
        let first = claim_probe_opportunity_at(&mut storage, 100).unwrap_or(ProbeOpportunity {
            generation: 0,
            if_id: 0,
            channel: 0,
            ssid_index: 0,
        });
        assert_eq!(first.generation, 7);
        assert_eq!(storage.probe_phase, ActiveProbePhase::InFlight);
        assert!(complete_probe_at(&mut storage, first, 0, 200));
        assert_eq!(storage.probe_round, 1);
        assert_eq!(storage.probe_deadline, 200 + 2 * 0x400);
        assert_eq!(storage.probe_phase, ActiveProbePhase::WaitingForDelay);

        assert_eq!(claim_probe_opportunity_at(&mut storage, 2247), None);
        let second = claim_probe_opportunity_at(&mut storage, 2248).unwrap_or(ProbeOpportunity {
            generation: 0,
            if_id: 0,
            channel: 0,
            ssid_index: 0,
        });
        assert!(complete_probe_at(&mut storage, second, 0, 2300));
        assert_eq!(storage.probe_round, 2);
        assert_eq!(storage.probe_publication_budget, 0);
        assert_eq!(storage.probe_phase, ActiveProbePhase::Done);
        assert_eq!(claim_probe_opportunity_at(&mut storage, u32::MAX), None);
    }
}
