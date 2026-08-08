//! Retained host configuration and opaque XR819 SDD/DPD calibration data.

use core::cell::UnsafeCell;

use crate::wsm::ConfigurationRequest;

pub const MAX_DPD_DATA_LEN: usize = 1536;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfigurationError {
    DpdTooLarge,
    MalformedSdd,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChannelCalibrationValues {
    pub first: i16,
    pub second: i16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Mode0ChannelCalibration {
    pub base: i16,
    pub first: i16,
    pub second: i16,
}

struct ConfigurationStorage {
    configured: bool,
    max_tx_msdu_lifetime: u32,
    max_rx_lifetime: u32,
    rts_threshold: u32,
    dpd_version: u16,
    station_id: [u8; 6],
    dpd_flags: u16,
    dpd_len: usize,
    dpd_data: [u8; MAX_DPD_DATA_LEN],
}

impl ConfigurationStorage {
    const fn new() -> Self {
        Self {
            configured: false,
            max_tx_msdu_lifetime: 0,
            max_rx_lifetime: 0,
            rts_threshold: 0,
            dpd_version: 0,
            station_id: [0; 6],
            dpd_flags: 0,
            dpd_len: 0,
            dpd_data: [0; MAX_DPD_DATA_LEN],
        }
    }
}

struct SharedConfiguration(UnsafeCell<ConfigurationStorage>);

unsafe impl Sync for SharedConfiguration {}

static XR819_CONFIGURATION: SharedConfiguration =
    SharedConfiguration(UnsafeCell::new(ConfigurationStorage::new()));

#[derive(Clone, Copy)]
pub struct ConfigurationSnapshot {
    pub max_tx_msdu_lifetime: u32,
    pub max_rx_lifetime: u32,
    pub rts_threshold: u32,
    pub dpd_version: u16,
    pub station_id: [u8; 6],
    pub dpd_flags: u16,
    pub dpd_data: &'static [u8],
}

pub fn retain(request: &ConfigurationRequest<'_>) -> Result<(), ConfigurationError> {
    if request.dpd_data.len() > MAX_DPD_DATA_LEN {
        return Err(ConfigurationError::DpdTooLarge);
    }
    validate_sdd(request.dpd_data)?;

    let storage = unsafe { &mut *XR819_CONFIGURATION.0.get() };
    storage.max_tx_msdu_lifetime = request.max_tx_msdu_lifetime;
    storage.max_rx_lifetime = request.max_rx_lifetime;
    storage.rts_threshold = request.rts_threshold;
    storage.dpd_version = request.dpd_version;
    storage.station_id = request.station_id;
    storage.dpd_flags = request.dpd_flags;
    storage.dpd_len = request.dpd_data.len();
    storage.dpd_data[..storage.dpd_len].copy_from_slice(request.dpd_data);
    storage.configured = true;
    Ok(())
}

pub fn snapshot() -> Option<ConfigurationSnapshot> {
    let storage = unsafe { &*XR819_CONFIGURATION.0.get() };
    if !storage.configured {
        return None;
    }

    Some(ConfigurationSnapshot {
        max_tx_msdu_lifetime: storage.max_tx_msdu_lifetime,
        max_rx_lifetime: storage.max_rx_lifetime,
        rts_threshold: storage.rts_threshold,
        dpd_version: storage.dpd_version,
        station_id: storage.station_id,
        dpd_flags: storage.dpd_flags,
        dpd_data: unsafe {
            core::slice::from_raw_parts(storage.dpd_data.as_ptr(), storage.dpd_len)
        },
    })
}

pub fn reference_frequency_khz() -> Option<u16> {
    let element = find_sdd_element(0xc5)?;
    let bytes: [u8; 2] = element.try_into().ok()?;
    Some(u16::from_le_bytes(bytes))
}

/// Vendor `0x176bc` loads SDD elements `0x30` and `0x31` as a signed default
/// followed by four-byte `(upper_channel, correction)` records. `0x19dd0`
/// selects the last correction whose threshold is not greater than the channel.
pub fn channel_threshold_correction(element_id: u8, channel: u16) -> Option<i16> {
    let element = find_sdd_element(element_id)?;
    if element.len() < 2 || (element.len() - 2) % 4 != 0 {
        return None;
    }
    let mut correction = i16::from_le_bytes(element[..2].try_into().ok()?);
    for record in element[2..].chunks_exact(4) {
        let upper_channel = u16::from_le_bytes(record[..2].try_into().ok()?);
        if upper_channel > channel {
            break;
        }
        correction = i16::from_le_bytes(record[2..4].try_into().ok()?);
    }
    Some(correction)
}

/// Vendor `0x17614` loads SDD element `0xec` as a 16-bit count followed by
/// three-byte `(upper_channel, first, second)` records consumed by `0x1a112`.
pub fn channel_calibration_values(channel: u8, base: i16) -> Option<ChannelCalibrationValues> {
    let element = find_sdd_element(0xec)?;
    if element.len() < 2 {
        return None;
    }
    let count = usize::from(u16::from_le_bytes(element[..2].try_into().ok()?));
    let records = element.get(2..2 + count.checked_mul(3)?)?;
    let mut selected = records.get(..3)?;
    for (index, record) in records.chunks_exact(3).enumerate() {
        if channel <= record[0] {
            selected = if channel < record[0] && index != 0 {
                &records[(index - 1) * 3..index * 3]
            } else {
                record
            };
            break;
        }
    }
    Some(ChannelCalibrationValues {
        first: base.wrapping_add(i16::from(selected[1]) * 4),
        second: base.wrapping_add(i16::from(selected[2]) * 4),
    })
}

pub fn mode0_channel_calibration(channel: u8) -> Option<Mode0ChannelCalibration> {
    let base = channel_threshold_correction(0x30, u16::from(channel))?;
    let values = channel_calibration_values(channel, base)?;
    Some(Mode0ChannelCalibration {
        base,
        first: values.first,
        second: values.second,
    })
}

pub fn find_sdd_element(id: u8) -> Option<&'static [u8]> {
    let configuration = snapshot()?;
    let mut remaining = configuration.dpd_data;
    while remaining.len() >= 2 {
        let element_id = remaining[0];
        let length = usize::from(remaining[1]);
        let end = 2 + length;
        if end > remaining.len() {
            return None;
        }
        if element_id == id {
            return Some(&remaining[2..end]);
        }
        remaining = &remaining[end..];
    }
    None
}

fn validate_sdd(mut data: &[u8]) -> Result<(), ConfigurationError> {
    while !data.is_empty() {
        if data.len() < 2 {
            return Err(ConfigurationError::MalformedSdd);
        }
        let length = usize::from(data[1]);
        let end = 2 + length;
        if end > data.len() {
            return Err(ConfigurationError::MalformedSdd);
        }
        data = &data[end..];
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_and_finds_sdd_elements() {
        let request = ConfigurationRequest {
            max_tx_msdu_lifetime: 1,
            max_rx_lifetime: 2,
            rts_threshold: 3,
            dpd_version: 1,
            station_id: [0, 1, 2, 3, 4, 5],
            dpd_flags: 5,
            dpd_data: &[
                0xc5, 2, 0xc0, 0x5d, 0xfe, 2, 0, 0, 0x30, 10, 0xff, 0xff, 3, 0, 10, 0, 8, 0, 20, 0,
                0xec, 8, 2, 0, 3, 2, 4, 8, 5, 7,
            ],
        };

        retain(&request).unwrap();
        assert_eq!(snapshot().unwrap().station_id, request.station_id);
        assert_eq!(find_sdd_element(0xc5), Some(&[0xc0, 0x5d][..]));
        assert_eq!(reference_frequency_khz(), Some(24_000));
        assert_eq!(channel_threshold_correction(0x30, 2), Some(-1));
        assert_eq!(channel_threshold_correction(0x30, 6), Some(10));
        assert_eq!(channel_threshold_correction(0x30, 11), Some(20));
        assert_eq!(
            channel_calibration_values(6, 100),
            Some(ChannelCalibrationValues {
                first: 108,
                second: 116,
            })
        );
        assert_eq!(
            mode0_channel_calibration(6),
            Some(Mode0ChannelCalibration {
                base: 10,
                first: 18,
                second: 26,
            })
        );
        assert_eq!(find_sdd_element(0xeb), None);
    }
}
