//! Retained WSM scan plan and cooperative scan state.
//!
//! This module owns the host request after the borrowed HIF RX buffer is
//! recycled, iterates every retained channel, and holds each successful tuning
//! result for the host-provided maximum dwell time.

use core::cell::UnsafeCell;

use crate::configuration::{self, Mode0ChannelCalibration};
#[cfg(target_arch = "arm")]
use crate::phy::run_channel_transition;
use crate::phy::{
    ChannelProgramRequestWire, ChannelTunePlan, PllDivider, build_scan_channel_program_request,
    channel_control_word, channel_frequency_khz_2ghz, channel_tune_plan, pll_divider,
};
use crate::wsm::{ScanChannel, StartScanRequest};

pub const MAX_SCAN_CHANNELS: usize = 48;
pub const VENDOR_MAX_SCAN_CHANNELS: usize = 34;
pub const MAX_SCAN_SSIDS: usize = 2;
pub const MAX_SSID_LEN: usize = 32;

#[cfg(any(target_arch = "arm", test))]
fn deadline_reached(now: u32, deadline: u32) -> bool {
    now.wrapping_sub(deadline) as i32 >= 0
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

struct ScanStorage {
    active: bool,
    completion_delay: u8,
    hardware_tune_pending: bool,
    hardware_status: u32,
    current_channel_index: u8,
    dwell_deadline: u32,
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
        let events = 0x0400_1fd4 as *mut u32;
        let value = events.read_volatile();
        events.write_volatile(if active {
            value | (1 << 10)
        } else {
            value & !(1 << 10)
        });
    }
}

#[cfg(not(target_arch = "arm"))]
fn set_vendor_scan_active(_active: bool) {}

pub fn begin(request: &StartScanRequest<'_>) -> Result<(), ScanError> {
    let num_channels = usize::from(request.num_channels);
    let num_ssids = usize::from(request.num_ssids);
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

    storage.band = request.band;
    storage.scan_type = request.scan_type;
    storage.flags = request.flags;
    storage.num_probes = request.num_probes;
    storage.probe_delay = request.probe_delay;
    storage.num_channels = request.num_channels;
    for index in 0..num_channels {
        storage.channels[index] = request
            .channel(index)
            .map(Into::into)
            .map_err(|_| ScanError::InvalidRecord)?;
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
    storage.active = true;
    set_vendor_scan_active(true);
    Ok(())
}

/// Advance tuning and dwell for the cooperative scan engine.
pub fn service() -> Option<ScanCompletion> {
    let storage = unsafe { &mut *SCAN.0.get() };
    if !storage.active {
        return None;
    }

    if storage.hardware_status == 0 && storage.hardware_tune_pending {
        storage.hardware_tune_pending = false;
        let index = usize::from(storage.current_channel_index);
        let _channel = storage.channels[index];
        #[cfg(target_arch = "arm")]
        match unsafe { run_channel_transition(_channel.number, 100_000) } {
            Ok(result) => unsafe {
                let now = vendor_timer();
                // Vendor scan deadlines use `channel_time * 0x400` against
                // `fw_read_timer()` (`0xe6b8`). The additive timer correction
                // at 0x0400143c cancels when comparing elapsed time.
                storage.dwell_deadline =
                    now.wrapping_add(_channel.max_channel_time.saturating_mul(0x400));
                (0x0900_ff98 as *mut u32).write_volatile(0x5455_4e4f);
                (0x0900_ff9c as *mut u32).write_volatile(result.divider.register);
            },
            Err(_) => {
                storage.hardware_status = 1;
                unsafe {
                    (0x0900_ff98 as *mut u32).write_volatile(0x5455_4e45);
                    (0x0900_ff9c as *mut u32).write_volatile(_channel.number.into());
                }
            }
        }
        #[cfg(not(target_arch = "arm"))]
        {
            storage.completion_delay = 1;
        }
        return None;
    }

    if storage.hardware_status == 0 {
        #[cfg(target_arch = "arm")]
        {
            let now = vendor_timer();
            if !deadline_reached(now, storage.dwell_deadline) {
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
            return None;
        }
    }

    storage.active = false;
    set_vendor_scan_active(false);
    Some(ScanCompletion {
        status: storage.hardware_status,
        psm: 0,
        num_channels: storage.num_channels,
        vendor_field: 0,
    })
}

pub fn active_channel() -> Option<u16> {
    let storage = unsafe { &*SCAN.0.get() };
    (storage.active
        && storage.hardware_status == 0
        && !storage.hardware_tune_pending
        && storage.current_channel_index < storage.num_channels)
        .then_some(storage.channels[usize::from(storage.current_channel_index)].number)
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
        let request = StartScanRequest::parse(&payload).unwrap();

        begin(&request).unwrap();
        assert_eq!(channel(0).unwrap().number, 6);
        assert_eq!(channel_control(0), Some(0x0117));
        assert_eq!(channel_program_request(0).unwrap().control.get(), 0x0117);
        assert_eq!(
            channel_tuning(0),
            Some(ChannelTunePlan {
                phy_mode: 2,
                recalibrate: true,
            })
        );
        assert_eq!(channel_frequency_khz(0), Some(2_437_000));
        assert_eq!(channel_pll(0).unwrap().register, 0x356e_c4ec);
        assert_eq!(channel(1).unwrap().number, 11);
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
        let request = StartScanRequest::parse(&payload).unwrap();
        assert_eq!(begin(&request), Err(ScanError::InvalidChannelTiming));
    }

    #[test]
    fn dwell_deadline_comparison_handles_timer_wrap() {
        assert!(!deadline_reached(0xffff_fff0, 0x0000_0010));
        assert!(deadline_reached(0x0000_0010, 0xffff_fff0));
        assert!(deadline_reached(1234, 1234));
    }
}
