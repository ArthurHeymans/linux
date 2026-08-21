//! Retained host configuration and opaque XR819 SDD/DPD calibration data.

use core::cell::UnsafeCell;

use crate::wsm::{ConfigurationRequest, EdcaParameters, TxPowerRange, TxQueueParameters};

pub const MAX_DPD_DATA_LEN: usize = 1536;
pub const MAX_TEMPLATE_FRAME_LEN: usize = 2304;

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
    edca_valid: bool,
    edca: EdcaParameters,
    uapsd_information: [u8; 8],
    rcpi_rssi_threshold: [u8; 4],
    tx_queues: [TxQueueParameters; 4],
    template_frame_len: usize,
    template_frame: [u8; MAX_TEMPLATE_FRAME_LEN],
    rx_filter: u32,
    current_tx_power_encoded: u32,
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
            edca_valid: false,
            edca: EdcaParameters {
                queues: [crate::wsm::EdcaQueueParameters {
                    cwmin: 0,
                    cwmax: 0,
                    aifns: 0,
                    txop_limit: 0,
                    max_rx_lifetime: 0,
                }; 4],
            },
            uapsd_information: [0; 8],
            rcpi_rssi_threshold: [0; 4],
            tx_queues: [TxQueueParameters {
                queue_id: 0,
                ack_policy: 0,
                max_transmit_lifetime: 0,
                allowed_medium_time: 0,
            }; 4],
            template_frame_len: 0,
            template_frame: [0; MAX_TEMPLATE_FRAME_LEN],
            rx_filter: 0,
            current_tx_power_encoded: 0,
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
    #[cfg(target_arch = "arm")]
    unsafe {
        populate_vendor_calibration_state();
    }
    Ok(())
}

pub fn retain_edca(parameters: EdcaParameters) {
    let storage = unsafe { &mut *XR819_CONFIGURATION.0.get() };
    storage.edca = parameters;
    storage.edca_valid = true;
}

pub fn edca() -> Option<EdcaParameters> {
    let storage = unsafe { &*XR819_CONFIGURATION.0.get() };
    storage.edca_valid.then_some(storage.edca)
}

pub fn retain_tx_queue(parameters: TxQueueParameters) {
    let storage = unsafe { &mut *XR819_CONFIGURATION.0.get() };
    storage.tx_queues[usize::from(parameters.queue_id)] = parameters;
}

pub fn tx_queue(queue: usize) -> Option<TxQueueParameters> {
    let storage = unsafe { &*XR819_CONFIGURATION.0.get() };
    storage.tx_queues.get(queue).copied()
}

pub fn template_frame() -> Option<&'static [u8]> {
    let storage = unsafe { &*XR819_CONFIGURATION.0.get() };
    (storage.template_frame_len != 0)
        .then_some(&storage.template_frame[..storage.template_frame_len])
}

pub fn retain_interface_mib(mib_id: u16, data: &[u8]) -> bool {
    let storage = unsafe { &mut *XR819_CONFIGURATION.0.get() };
    match (mib_id, data) {
        (crate::wsm::MIB_ID_DOT11_CURRENT_TX_POWER_LEVEL, [a, b, c, d]) => {
            storage.current_tx_power_encoded = u32::from_le_bytes([*a, *b, *c, *d]) ^ 0x8000_0000;
            true
        }
        (crate::wsm::MIB_ID_SET_UAPSD_INFORMATION, [a, b, c, d, e, f, g, h]) => {
            storage.uapsd_information = [*a, *b, *c, *d, *e, *f, *g, *h];
            true
        }
        (crate::wsm::MIB_ID_RCPI_RSSI_THRESHOLD, [a, b, c, d]) => {
            storage.rcpi_rssi_threshold = [*a, *b, *c, *d];
            true
        }
        (crate::wsm::MIB_ID_RX_FILTER, [a, b, c, d]) => {
            storage.rx_filter = u32::from_le_bytes([*a, *b, *c, *d]);
            true
        }
        // The cooperative joined RX path currently delivers a superset and
        // lets mac80211 apply these optional filters. Accepting the standard
        // CW1200 MIBs is therefore honest even before hardware offload exists.
        (crate::wsm::MIB_ID_BEACON_FILTER_TABLE, _)
        | (crate::wsm::MIB_ID_BEACON_FILTER_ENABLE, _)
        | (crate::wsm::MIB_ID_DISABLE_BSSID_FILTER, _)
        | (crate::wsm::MIB_ID_GROUP_ADDRESSES_TABLE, _) => true,
        (crate::wsm::MIB_ID_TEMPLATE_FRAME, data) if data.len() <= MAX_TEMPLATE_FRAME_LEN => {
            let previous_len = storage.template_frame_len;
            storage.template_frame[..data.len()].copy_from_slice(data);
            if previous_len > data.len() {
                storage.template_frame[data.len()..previous_len].fill(0);
            }
            storage.template_frame_len = data.len();
            true
        }
        _ => false,
    }
}

pub fn current_tx_power_tenths_dbm() -> Option<i32> {
    let storage = unsafe { &*XR819_CONFIGURATION.0.get() };
    (storage.current_tx_power_encoded != 0)
        .then_some((storage.current_tx_power_encoded ^ 0x8000_0000) as i32)
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

#[cfg(target_arch = "arm")]
unsafe fn populate_vendor_calibration_state() {
    unsafe fn write_u8(address: usize, value: u8) {
        unsafe { (address as *mut u8).write_volatile(value) };
    }
    unsafe fn write_u16(address: usize, value: u16) {
        unsafe { (address as *mut u16).write_volatile(value) };
    }
    unsafe fn write_u32(address: usize, value: u32) {
        unsafe { (address as *mut u32).write_volatile(value) };
    }
    unsafe fn copy_u16_profile(id: u8, destination: usize) {
        if let Some(data) = find_sdd_element(id) {
            for (index, value) in data.chunks_exact(2).take(11).enumerate() {
                unsafe {
                    write_u16(
                        destination + index * 2,
                        u16::from_le_bytes([value[0], value[1]]),
                    )
                };
            }
        }
    }
    unsafe fn copy_u16_value(id: u8, destination: usize) {
        if let Some([first, second, ..]) = find_sdd_element(id) {
            unsafe { write_u16(destination, u16::from_le_bytes([*first, *second])) };
        }
    }
    unsafe fn copy_u16_pair(id: u8, destination: usize) {
        if let Some([a, b, c, d, ..]) = find_sdd_element(id) {
            unsafe {
                write_u16(destination, u16::from_le_bytes([*a, *b]));
                write_u16(destination + 2, u16::from_le_bytes([*c, *d]));
            }
        }
    }

    unsafe {
        if let Some(reference) = reference_frequency_khz() {
            write_u32(crate::dtcm::phy_reference_word().get(), u32::from(reference));
        }

        // Annotated callbacks 0x17626 and 0x17646.
        copy_u16_profile(0xe3, crate::dtcm::sdd_profile_unchecked(0).get());
        copy_u16_profile(0xe4, crate::dtcm::sdd_profile_unchecked(1).get());
        copy_u16_profile(0x48, crate::dtcm::sdd_rssi_rate_scale_unchecked(0, 0).get());
        copy_u16_profile(0x49, crate::dtcm::sdd_rssi_rate_scale_unchecked(1, 0).get());

        // Reference handlers 0x1774a: signed AGC threshold correction for
        // profile zero/one, consumed by `phy_build_gain_tables`.
        copy_u16_value(0xe0, crate::dtcm::sdd_agc_correction_unchecked(0).get());
        copy_u16_value(0xe1, crate::dtcm::sdd_agc_correction_unchecked(1).get());

        // Exact type-3 SDD callbacks at 0x176e2..0x177d4. These coefficients
        // are consumed directly by `phy_lookup_gain_pair` and
        // `phy_compute_rssi`; leaving them zero produces invalid TX-gain words.
        copy_u16_value(0x20, crate::dtcm::sdd_gain_coefficient_unchecked(0).get());
        copy_u16_value(0x21, crate::dtcm::sdd_gain_coefficient_unchecked(2).get());
        copy_u16_value(0x22, crate::dtcm::sdd_gain_coefficient_unchecked(1).get());
        copy_u16_value(0x23, crate::dtcm::sdd_gain_coefficient_unchecked(3).get());
        copy_u16_pair(0x40, crate::dtcm::sdd_rssi_coefficient_unchecked(0, 0).get());
        copy_u16_pair(0x41, crate::dtcm::sdd_rssi_coefficient_unchecked(1, 0).get());
        copy_u16_value(0x42, crate::dtcm::sdd_calibration_coefficient_unchecked(0).get());
        copy_u16_value(0x43, crate::dtcm::sdd_calibration_coefficient_unchecked(1).get());
        // At configuration time the vendor ADC conversion normally lacks
        // live samples and falls back to the first two SDD halfwords.
        copy_u16_pair(0x46, crate::dtcm::sdd_conversion_value_unchecked(0, 0).get());
        copy_u16_pair(0x47, crate::dtcm::sdd_conversion_value_unchecked(1, 0).get());

        // Annotated callback 0x17668: count plus three-byte channel records.
        if let Some(data) = find_sdd_element(0xec) {
            if data.len() >= 2 {
                let count = usize::from(u16::from_le_bytes([data[0], data[1]]));
                let records_len = count.saturating_mul(3);
                if count <= u8::MAX as usize && data.len() >= 2 + records_len {
                    for (index, value) in data[2..2 + records_len].iter().copied().enumerate() {
                        write_u8(crate::dtcm::sdd_channel_byte_unchecked(0, index).get(), value);
                    }
                    write_u8(crate::dtcm::sdd_channel_count_unchecked(0).get(), count as u8);
                    write_u32(crate::dtcm::phy_table_pointer().get(), crate::dtcm::SDD_CONFIGURATION_TABLES.get() as u32);
                }
            }
        }

        // Profile-one two-byte channel records from callback 0x176a8.
        if let Some(data) = find_sdd_element(0xed) {
            if data.len() >= 2 {
                let count = usize::from(u16::from_le_bytes([data[0], data[1]]));
                let records_len = count.saturating_mul(2);
                if count <= u8::MAX as usize && data.len() >= 2 + records_len {
                    for (index, record) in data[2..2 + records_len].chunks_exact(2).enumerate() {
                        write_u8(crate::dtcm::sdd_channel_byte_unchecked(1, index * 3).get(), record[0]);
                        write_u8(crate::dtcm::sdd_channel_byte_unchecked(1, index * 3 + 1).get(), record[1]);
                    }
                    write_u8(crate::dtcm::sdd_channel_count_unchecked(1).get(), count as u8);
                }
            }
        }

        // Parameter-zero temperature fallback supplied by SDD element 0x46.
        if let Some(data) = find_sdd_element(0x46) {
            if data.len() >= 4 {
                write_u16(crate::dtcm::phy_denominator().get(), u16::from_le_bytes([data[0], data[1]]));
                write_u16(crate::dtcm::phy_correction_offset().get(), u16::from_le_bytes([data[2], data[3]]));
            }
        }
    }
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

fn tx_power_profile_maximum(element_id: u8) -> Option<i32> {
    let element = find_sdd_element(element_id)?;
    // Annotated SDD callback `0x00017626` copies eleven little-endian u16
    // entries from TLVs e3/e4 to runtime profiles at 0x040034b0 and +0x92.
    // `phy_get_tx_power_range` reads entry ten at profile offsets 0x14/0xa6.
    let bytes: [u8; 2] = element.get(20..22)?.try_into().ok()?;
    Some(i32::from(u16::from_le_bytes(bytes)))
}

pub fn tx_power_ranges() -> Option<[TxPowerRange; 2]> {
    Some([
        TxPowerRange {
            min_power_level: -160,
            max_power_level: tx_power_profile_maximum(0xe3)?,
            stepping: 0,
        },
        TxPowerRange {
            min_power_level: -160,
            max_power_level: tx_power_profile_maximum(0xe4)?,
            stepping: 0,
        },
    ])
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

        let mut power_sdd = [0_u8; 56];
        power_sdd[0] = 0xe3;
        power_sdd[1] = 26;
        power_sdd[22..24].copy_from_slice(&272_u16.to_le_bytes());
        power_sdd[28] = 0xe4;
        power_sdd[29] = 26;
        power_sdd[50..52].copy_from_slice(&212_u16.to_le_bytes());
        retain(&ConfigurationRequest {
            dpd_data: &power_sdd,
            ..request
        })
        .unwrap();
        assert!(retain_interface_mib(
            crate::wsm::MIB_ID_DOT11_CURRENT_TX_POWER_LEVEL,
            &(-135_i32).to_le_bytes(),
        ));
        assert_eq!(current_tx_power_tenths_dbm(), Some(-135));

        assert_eq!(
            tx_power_ranges(),
            Some([
                TxPowerRange {
                    min_power_level: -160,
                    max_power_level: 272,
                    stepping: 0,
                },
                TxPowerRange {
                    min_power_level: -160,
                    max_power_level: 212,
                    stepping: 0,
                },
            ])
        );
    }
}
