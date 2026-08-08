//! Retained WSM scan plan and cooperative scan state.
//!
//! Channel tuning is intentionally not implemented here. This module owns the
//! host request after the borrowed HIF RX buffer is recycled and provides the
//! state boundary where the translated PHY channel operation will be attached.

use core::cell::UnsafeCell;

use crate::phy::{
    ChannelProgramRequestWire, build_scan_channel_program_request, channel_control_word,
};
use crate::wsm::{ScanChannel, StartScanRequest};

pub const MAX_SCAN_CHANNELS: usize = 48;
pub const VENDOR_MAX_SCAN_CHANNELS: usize = 34;
pub const MAX_SCAN_SSIDS: usize = 2;
pub const MAX_SSID_LEN: usize = 32;

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
    storage.active = true;
    set_vendor_scan_active(true);
    Ok(())
}

/// Advance the cooperative scan engine.
///
/// The one-tick delay ensures the start-scan response is published before the
/// completion indication. Real channel dwell and RX processing will replace
/// this placeholder transition.
pub fn service() -> Option<ScanCompletion> {
    let storage = unsafe { &mut *SCAN.0.get() };
    if !storage.active {
        return None;
    }
    if storage.completion_delay != 0 {
        storage.completion_delay -= 1;
        return None;
    }

    storage.active = false;
    set_vendor_scan_active(false);
    Some(ScanCompletion {
        status: 0,
        psm: 0,
        num_channels: storage.num_channels,
        vendor_field: 0,
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retains_scan_records_after_request_buffer_is_gone() {
        let mut payload = [0; 12 + 16];
        payload[9] = 1;
        payload[12..14].copy_from_slice(&6_u16.to_le_bytes());
        payload[16..20].copy_from_slice(&10_u32.to_le_bytes());
        payload[20..24].copy_from_slice(&30_u32.to_le_bytes());
        let request = StartScanRequest::parse(&payload).unwrap();

        begin(&request).unwrap();
        assert_eq!(channel(0).unwrap().number, 6);
        assert_eq!(channel_control(0), Some(0x0117));
        assert_eq!(channel_program_request(0).unwrap().control.get(), 0x0117);
        assert_eq!(
            service(),
            Some(ScanCompletion {
                status: 0,
                psm: 0,
                num_channels: 1,
                vendor_field: 0,
            })
        );

        payload[20..24].copy_from_slice(&0_u32.to_le_bytes());
        let request = StartScanRequest::parse(&payload).unwrap();
        assert_eq!(begin(&request), Err(ScanError::InvalidChannelTiming));
    }
}
