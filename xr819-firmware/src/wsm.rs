//! CW1200-compatible Wireless Station Management wire protocol.
//!
//! This module deliberately contains no XR819 hardware policy. It describes
//! only the byte-level ABI presented to the Linux `cw1200` driver.

use core::fmt;
use zerocopy::byteorder::little_endian::{U16, U32};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

pub const HEADER_LEN: usize = 4;
pub const STARTUP_IND_ID: u16 = 0x0801;
pub const CONFIGURATION_REQ_ID: u16 = 0x0009;
pub const CONFIGURATION_RESP_ID: u16 = 0x0409;
pub const RESET_REQ_ID: u16 = 0x000a;
pub const RESET_RESP_ID: u16 = 0x040a;
pub const READ_MIB_REQ_ID: u16 = 0x0005;
pub const READ_MIB_RESP_ID: u16 = 0x0405;
pub const WRITE_MIB_REQ_ID: u16 = 0x0006;
pub const WRITE_MIB_RESP_ID: u16 = 0x0406;
pub const START_SCAN_REQ_ID: u16 = 0x0007;
pub const START_SCAN_RESP_ID: u16 = 0x0407;
pub const TX_REQ_ID: u16 = 0x0004;
pub const TX_CONFIRM_ID: u16 = 0x0404;
pub const JOIN_REQ_ID: u16 = 0x000b;
pub const JOIN_RESP_ID: u16 = 0x040b;
pub const JOIN_COMPLETE_IND_ID: u16 = 0x080f;
pub const TX_QUEUE_PARAMS_REQ_ID: u16 = 0x0012;
pub const TX_QUEUE_PARAMS_RESP_ID: u16 = 0x0412;
pub const EDCA_PARAMS_REQ_ID: u16 = 0x0013;
pub const EDCA_PARAMS_RESP_ID: u16 = 0x0413;
pub const RECEIVE_IND_ID: u16 = 0x0804;

pub const STATUS_SUCCESS: u32 = 0;
pub const STATUS_FAILURE: u32 = 1;
pub const MIB_ID_DOT11_CURRENT_TX_POWER_LEVEL: u16 = 0x0006;
pub const MIB_ID_TEMPLATE_FRAME: u16 = 0x1002;
pub const MIB_ID_RX_FILTER: u16 = 0x1003;
pub const MIB_ID_BEACON_FILTER_TABLE: u16 = 0x1004;
pub const MIB_ID_BEACON_FILTER_ENABLE: u16 = 0x1005;
pub const MIB_ID_RCPI_RSSI_THRESHOLD: u16 = 0x1009;
pub const MIB_ID_SET_UAPSD_INFORMATION: u16 = 0x1013;
pub const MIB_ID_DISABLE_BSSID_FILTER: u16 = 0x1026;
pub const MIB_ID_GROUP_ADDRESSES_TABLE: u16 = 0x0004;

#[derive(Clone, Copy, Immutable, IntoBytes)]
#[repr(C)]
struct HeaderWire {
    len: U16,
    id: U16,
}

#[derive(Clone, Copy, FromBytes, Immutable, IntoBytes, KnownLayout)]
#[repr(C)]
struct ConfigurationRequestWire {
    max_tx_msdu_lifetime: U32,
    max_rx_lifetime: U32,
    rts_threshold: U32,
    dpd_block_len: U16,
    dpd_version: U16,
    station_id: [u8; 6],
    dpd_flags: U16,
}

#[derive(Clone, Copy, Immutable, IntoBytes)]
#[repr(C)]
struct StartupIndicationWire {
    input_buffers: U16,
    input_buffer_size: U16,
    hardware_id: U16,
    hardware_sub_id: U16,
    status: U16,
    firmware_capabilities: U16,
    firmware_type: U16,
    firmware_api: U16,
    firmware_build: U16,
    firmware_version: U16,
    label: [u8; 128],
    config: [U32; 4],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Header {
    pub len: u16,
    pub id: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TxPowerRange {
    pub min_power_level: i32,
    pub max_power_level: i32,
    pub stepping: i32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EdcaQueueParameters {
    pub cwmin: u16,
    pub cwmax: u16,
    pub aifns: u8,
    pub txop_limit: u16,
    pub max_rx_lifetime: u32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EdcaParameters {
    pub queues: [EdcaQueueParameters; 4],
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TxQueueParameters {
    pub queue_id: u8,
    pub ack_policy: u8,
    pub max_transmit_lifetime: u32,
    pub allowed_medium_time: u16,
}

impl TxQueueParameters {
    pub fn parse(payload: &[u8]) -> Result<Self, Error> {
        if payload.len() != 12 || payload[0] > 3 {
            return Err(Error::InvalidLength);
        }
        Ok(Self {
            queue_id: payload[0],
            ack_policy: payload[2],
            max_transmit_lifetime: u32::from_le_bytes([
                payload[4], payload[5], payload[6], payload[7],
            ]),
            allowed_medium_time: u16::from_le_bytes([payload[8], payload[9]]),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WriteMibRequest<'a> {
    pub mib_id: u16,
    pub data: &'a [u8],
}

impl WriteMibRequest<'_> {
    pub fn parse(payload: &[u8]) -> Result<WriteMibRequest<'_>, Error> {
        if payload.len() < 4 {
            return Err(Error::Truncated);
        }
        let mib_id = u16::from_le_bytes([payload[0], payload[1]]);
        let length = usize::from(u16::from_le_bytes([payload[2], payload[3]]));
        if payload.len() != 4 + length {
            return Err(Error::InvalidLength);
        }
        Ok(WriteMibRequest {
            mib_id,
            data: &payload[4..],
        })
    }
}

impl EdcaParameters {
    pub fn parse(payload: &[u8]) -> Result<Self, Error> {
        if payload.len() != 44 {
            return Err(Error::InvalidLength);
        }
        let read_u16 = |offset: usize| u16::from_le_bytes([payload[offset], payload[offset + 1]]);
        let read_u32 = |offset: usize| {
            u32::from_le_bytes([
                payload[offset],
                payload[offset + 1],
                payload[offset + 2],
                payload[offset + 3],
            ])
        };
        let mut queues = [EdcaQueueParameters::default(); 4];
        for wire_index in 0..4 {
            let queue = 3 - wire_index;
            queues[queue] = EdcaQueueParameters {
                cwmin: read_u16(wire_index * 2),
                cwmax: read_u16(8 + wire_index * 2),
                aifns: payload[16 + wire_index],
                txop_limit: read_u16(20 + wire_index * 2),
                max_rx_lifetime: read_u32(28 + wire_index * 4),
            };
        }
        Ok(Self { queues })
    }
}

impl Header {
    pub fn parse(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() < HEADER_LEN {
            return Err(Error::Truncated);
        }

        let header = Self {
            len: u16::from_le_bytes([bytes[0], bytes[1]]),
            id: u16::from_le_bytes([bytes[2], bytes[3]]),
        };
        if usize::from(header.len) < HEADER_LEN || usize::from(header.len) > bytes.len() {
            return Err(Error::InvalidLength);
        }
        Ok(header)
    }

    pub fn encode(self, output: &mut [u8]) -> Result<(), Error> {
        if output.len() < HEADER_LEN {
            return Err(Error::Truncated);
        }
        let wire = HeaderWire {
            len: self.len.into(),
            id: self.id.into(),
        };
        output[..HEADER_LEN].copy_from_slice(wire.as_bytes());
        Ok(())
    }

    /// Vendor `wsm_dispatch_cmd` (`0x0000e5a0`) applies `MsgId & 0x0c3f`.
    /// This removes sequence bits 13..15 and the CW1200 link-routing field in
    /// bits 6..9 while retaining the request/confirm/indication class bits.
    pub const fn base_id(self) -> u16 {
        self.id & 0x0c3f
    }

    /// XR819 interprets the low two routing bits as its three-entry VIF index;
    /// value 2 is the P2P-device slot and value 3 is invalid. Upstream CW1200
    /// names the wider bits 6..9 field `link_id`, so do not use this accessor
    /// for per-station link-slot routing.
    pub const fn if_id(self) -> u8 {
        ((self.id >> 6) & 3) as u8
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Truncated,
    InvalidLength,
    InvalidDpdLength,
    InvalidScanRequest,
    InvalidJoinRequest,
    OutputTooSmall,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Truncated => "truncated WSM message",
            Self::InvalidLength => "invalid WSM message length",
            Self::InvalidDpdLength => "invalid WSM DPD block length",
            Self::InvalidScanRequest => "invalid WSM start-scan request",
            Self::InvalidJoinRequest => "invalid WSM join request",
            Self::OutputTooSmall => "WSM output buffer is too small",
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConfigurationRequest<'a> {
    pub max_tx_msdu_lifetime: u32,
    pub max_rx_lifetime: u32,
    pub rts_threshold: u32,
    pub dpd_version: u16,
    pub station_id: [u8; 6],
    pub dpd_flags: u16,
    pub dpd_data: &'a [u8],
}

impl<'a> ConfigurationRequest<'a> {
    pub fn parse(payload: &'a [u8]) -> Result<Self, Error> {
        let (wire, _) =
            ConfigurationRequestWire::ref_from_prefix(payload).map_err(|_| Error::Truncated)?;
        let dpd_block_len = usize::from(wire.dpd_block_len.get());
        if dpd_block_len < 12 || 12 + dpd_block_len > payload.len() {
            return Err(Error::InvalidDpdLength);
        }

        Ok(Self {
            max_tx_msdu_lifetime: wire.max_tx_msdu_lifetime.get(),
            max_rx_lifetime: wire.max_rx_lifetime.get(),
            rts_threshold: wire.rts_threshold.get(),
            dpd_version: wire.dpd_version.get(),
            station_id: wire.station_id,
            dpd_flags: wire.dpd_flags.get(),
            dpd_data: &payload[size_of::<ConfigurationRequestWire>()..12 + dpd_block_len],
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScanChannel {
    pub number: u16,
    pub min_channel_time: u32,
    pub max_channel_time: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StartScanRequest<'a> {
    pub band: u8,
    pub scan_type: u8,
    pub flags: u8,
    pub max_tx_rate: u8,
    pub auto_scan_interval: u32,
    pub num_probes: u8,
    pub num_channels: u8,
    pub num_ssids: u8,
    pub probe_delay: u8,
    channel_bytes: &'a [u8],
    ssid_bytes: &'a [u8],
}

impl<'a> StartScanRequest<'a> {
    const FIXED_LEN: usize = 12;
    const CHANNEL_LEN: usize = 16;
    const SSID_LEN: usize = 36;
    const MAX_CHANNELS: usize = 48;
    const MAX_SSIDS: usize = 2;

    pub fn parse(payload: &'a [u8]) -> Result<Self, Error> {
        if payload.len() < Self::FIXED_LEN {
            return Err(Error::Truncated);
        }

        let num_channels = usize::from(payload[9]);
        let num_ssids = usize::from(payload[10]);
        if num_channels > Self::MAX_CHANNELS || num_ssids > Self::MAX_SSIDS {
            return Err(Error::InvalidScanRequest);
        }

        let channels_len = num_channels
            .checked_mul(Self::CHANNEL_LEN)
            .ok_or(Error::InvalidScanRequest)?;
        let ssids_len = num_ssids
            .checked_mul(Self::SSID_LEN)
            .ok_or(Error::InvalidScanRequest)?;
        let ssids_offset = Self::FIXED_LEN
            .checked_add(channels_len)
            .ok_or(Error::InvalidScanRequest)?;
        let end = ssids_offset
            .checked_add(ssids_len)
            .ok_or(Error::InvalidScanRequest)?;
        if end != payload.len() {
            return Err(Error::InvalidScanRequest);
        }

        let request = Self {
            band: payload[0],
            scan_type: payload[1],
            flags: payload[2],
            max_tx_rate: payload[3],
            auto_scan_interval: read_u32(payload, 4)?,
            num_probes: payload[8],
            num_channels: payload[9],
            num_ssids: payload[10],
            probe_delay: payload[11],
            channel_bytes: &payload[Self::FIXED_LEN..ssids_offset],
            ssid_bytes: &payload[ssids_offset..end],
        };

        for index in 0..num_ssids {
            request.ssid(index)?;
        }
        Ok(request)
    }

    pub fn channel(&self, index: usize) -> Result<ScanChannel, Error> {
        if index >= usize::from(self.num_channels) {
            return Err(Error::InvalidScanRequest);
        }
        let offset = index * Self::CHANNEL_LEN;
        Ok(ScanChannel {
            number: read_u16(self.channel_bytes, offset)?,
            min_channel_time: read_u32(self.channel_bytes, offset + 4)?,
            max_channel_time: read_u32(self.channel_bytes, offset + 8)?,
        })
    }

    pub fn ssid(&self, index: usize) -> Result<&'a [u8], Error> {
        if index >= usize::from(self.num_ssids) {
            return Err(Error::InvalidScanRequest);
        }
        let offset = index * Self::SSID_LEN;
        let length = usize::try_from(read_u32(self.ssid_bytes, offset)?)
            .map_err(|_| Error::InvalidScanRequest)?;
        if length > 32 {
            return Err(Error::InvalidScanRequest);
        }
        Ok(&self.ssid_bytes[offset + 4..offset + 4 + length])
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct JoinRequest<'a> {
    pub mode: u8,
    pub band: u8,
    pub channel_number: u16,
    pub bssid: [u8; 6],
    pub atim_window: u16,
    pub preamble_type: u8,
    pub probe_for_join: bool,
    pub dtim_period: u8,
    pub flags: u8,
    pub ssid: &'a [u8],
    pub beacon_interval: u32,
    pub basic_rate_set: u32,
}

impl<'a> JoinRequest<'a> {
    pub const PAYLOAD_LEN: usize = 60;

    pub fn parse(payload: &'a [u8]) -> Result<Self, Error> {
        if payload.len() != Self::PAYLOAD_LEN {
            return Err(Error::InvalidJoinRequest);
        }
        let ssid_len = usize::try_from(read_u32(payload, 0x10)?)
            .map_err(|_| Error::InvalidJoinRequest)?;
        if ssid_len > 32 {
            return Err(Error::InvalidJoinRequest);
        }
        let mut bssid = [0_u8; 6];
        bssid.copy_from_slice(&payload[4..10]);
        Ok(Self {
            mode: payload[0],
            band: payload[1],
            channel_number: read_u16(payload, 2)?,
            bssid,
            atim_window: read_u16(payload, 0x0a)?,
            preamble_type: payload[0x0c],
            probe_for_join: payload[0x0d] != 0,
            dtim_period: payload[0x0e],
            flags: payload[0x0f],
            ssid: &payload[0x14..0x14 + ssid_len],
            beacon_interval: read_u32(payload, 0x34)?,
            basic_rate_set: read_u32(payload, 0x38)?,
        })
    }

    pub const fn supported_sta_shape(&self) -> bool {
        self.mode == 1
            && self.band == 0
            && self.channel_number >= 1
            && self.channel_number <= 14
            && self.atim_window == 0
            && self.preamble_type == 0
            && self.flags & 0x1b == 0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TxRequest<'a> {
    pub packet_id: u32,
    pub max_tx_rate: u8,
    pub queue_id: u8,
    pub more: bool,
    pub flags: u8,
    pub expire_time: u32,
    pub ht_tx_parameters: u32,
    pub frame: &'a [u8],
}

impl<'a> TxRequest<'a> {
    pub const PAYLOAD_HEADER_LEN: usize = 20;

    pub fn parse(payload: &'a [u8]) -> Result<Self, Error> {
        if payload.len() < Self::PAYLOAD_HEADER_LEN {
            return Err(Error::Truncated);
        }
        let shifted = payload[7] & 0x80 != 0;
        let frame_offset = Self::PAYLOAD_HEADER_LEN + if shifted { 2 } else { 0 };
        let frame = payload.get(frame_offset..).ok_or(Error::Truncated)?;
        if frame.len() < 24 {
            return Err(Error::Truncated);
        }
        Ok(Self {
            packet_id: read_u32(payload, 0)?,
            max_tx_rate: payload[4],
            queue_id: payload[5],
            more: payload[6] != 0,
            flags: payload[7],
            expire_time: read_u32(payload, 12)?,
            ht_tx_parameters: read_u32(payload, 16)?,
            frame,
        })
    }

    pub fn is_unicast_management(&self) -> bool {
        let frame_control = u16::from_le_bytes([self.frame[0], self.frame[1]]);
        frame_control & 0x000c == 0 && self.frame[4] & 1 == 0
    }

    pub fn is_unicast_eapol(&self) -> bool {
        const EAPOL_SNAP: [u8; 8] = [0xaa, 0xaa, 0x03, 0x00, 0x00, 0x00, 0x88, 0x8e];
        let frame_control = u16::from_le_bytes([self.frame[0], self.frame[1]]);
        frame_control & 0x000c == 0x0008
            && self.frame[4] & 1 == 0
            && self.frame[24..self.frame.len().min(48)]
                .windows(EAPOL_SNAP.len())
                .any(|window| window == EAPOL_SNAP)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResetRequest {
    /// The upstream driver sends zero to reset statistics and one to retain
    /// them. Link ID is encoded in the WSM header rather than this payload.
    pub reset_statistics: bool,
}

impl ResetRequest {
    pub fn parse(payload: &[u8]) -> Result<Self, Error> {
        Ok(Self {
            reset_statistics: read_u32(payload, 0)? == 0,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StartupIndication<'a> {
    pub input_buffers: u16,
    pub input_buffer_size: u16,
    pub status: u16,
    pub hardware_id: u16,
    pub hardware_sub_id: u16,
    pub firmware_capabilities: u16,
    pub firmware_type: u16,
    pub firmware_api: u16,
    pub firmware_build: u16,
    pub firmware_version: u16,
    pub label: &'a [u8],
    pub config: [u32; 4],
}

impl StartupIndication<'_> {
    pub const PAYLOAD_LEN: usize = 164;

    pub fn encode(self, output: &mut [u8]) -> Result<usize, Error> {
        let total_len = HEADER_LEN + Self::PAYLOAD_LEN;
        if output.len() < total_len {
            return Err(Error::OutputTooSmall);
        }

        Header {
            len: total_len as u16,
            id: STARTUP_IND_ID,
        }
        .encode(output)?;

        let mut label = [0; 128];
        let label_len = self.label.len().min(127);
        label[..label_len].copy_from_slice(&self.label[..label_len]);
        let wire = StartupIndicationWire {
            input_buffers: self.input_buffers.into(),
            input_buffer_size: self.input_buffer_size.into(),
            hardware_id: self.hardware_id.into(),
            hardware_sub_id: self.hardware_sub_id.into(),
            status: self.status.into(),
            firmware_capabilities: self.firmware_capabilities.into(),
            firmware_type: self.firmware_type.into(),
            firmware_api: self.firmware_api.into(),
            firmware_build: self.firmware_build.into(),
            firmware_version: self.firmware_version.into(),
            label,
            config: self.config.map(Into::into),
        };
        output[HEADER_LEN..total_len].copy_from_slice(wire.as_bytes());
        Ok(total_len)
    }
}

pub fn encode_status_response(id: u16, status: u32, output: &mut [u8]) -> Result<usize, Error> {
    const LEN: usize = HEADER_LEN + 4;
    if output.len() < LEN {
        return Err(Error::OutputTooSmall);
    }
    Header {
        len: LEN as u16,
        id,
    }
    .encode(output)?;
    write_u32(output, HEADER_LEN, status);
    Ok(LEN)
}

/// Confirmation layout consumed by mainline `wsm_read_mib_confirm`: status,
/// echoed MIB ID, returned byte count, then data. Unsupported reads use a zero
/// byte count but retain the complete fixed prefix to avoid parser underflow.
pub fn encode_read_mib_data_response(
    status: u32,
    mib_id: u16,
    data: &[u8],
    output: &mut [u8],
) -> Result<usize, Error> {
    let len = HEADER_LEN + 8 + data.len();
    if data.len() > u16::MAX as usize || output.len() < len {
        return Err(Error::OutputTooSmall);
    }
    Header {
        len: len as u16,
        id: READ_MIB_RESP_ID,
    }
    .encode(output)?;
    write_u32(output, HEADER_LEN, status);
    write_u16(output, HEADER_LEN + 4, mib_id);
    write_u16(output, HEADER_LEN + 6, data.len() as u16);
    output[HEADER_LEN + 8..len].copy_from_slice(data);
    Ok(len)
}

pub fn encode_read_mib_response(
    status: u32,
    mib_id: u16,
    output: &mut [u8],
) -> Result<usize, Error> {
    encode_read_mib_data_response(status, mib_id, &[], output)
}

/// Twelve-byte payload required by mainline `wsm_join_confirm`, even when the
/// operation is rejected before real association support is available.
pub fn encode_join_response(
    status: u32,
    min_power_level: i32,
    max_power_level: i32,
    output: &mut [u8],
) -> Result<usize, Error> {
    const LEN: usize = HEADER_LEN + 12;
    if output.len() < LEN {
        return Err(Error::OutputTooSmall);
    }
    Header {
        len: LEN as u16,
        id: JOIN_RESP_ID,
    }
    .encode(output)?;
    write_u32(output, HEADER_LEN, status);
    write_u32(output, HEADER_LEN + 4, min_power_level as u32);
    write_u32(output, HEADER_LEN + 8, max_power_level as u32);
    Ok(LEN)
}

pub fn encode_tx_confirm(
    packet_id: u32,
    status: u32,
    output: &mut [u8],
) -> Result<usize, Error> {
    encode_tx_confirm_details(packet_id, status, 0, 0, output)
}

pub fn encode_tx_confirm_details(
    packet_id: u32,
    status: u32,
    tx_rate: u8,
    ack_failures: u8,
    output: &mut [u8],
) -> Result<usize, Error> {
    const PAYLOAD_LEN: usize = 20;
    let total_len = HEADER_LEN + PAYLOAD_LEN;
    if output.len() < total_len {
        return Err(Error::OutputTooSmall);
    }
    Header {
        len: total_len as u16,
        id: TX_CONFIRM_ID,
    }
    .encode(output)?;
    output[HEADER_LEN..total_len].fill(0);
    write_u32(output, HEADER_LEN, packet_id);
    write_u32(output, HEADER_LEN + 4, status);
    output[HEADER_LEN + 8] = tx_rate;
    output[HEADER_LEN + 9] = ack_failures;
    Ok(total_len)
}

pub fn encode_xr819_tx_confirm(
    packet_id: u32,
    status: u32,
    output: &mut [u8],
) -> Result<usize, Error> {
    encode_xr819_tx_confirm_details(packet_id, status, 0, 0, output)
}

pub fn encode_xr819_tx_confirm_details(
    packet_id: u32,
    status: u32,
    tx_rate: u8,
    ack_failures: u8,
    output: &mut [u8],
) -> Result<usize, Error> {
    const PAYLOAD_LEN: usize = 32;
    let total_len = HEADER_LEN + PAYLOAD_LEN;
    if output.len() < total_len {
        return Err(Error::OutputTooSmall);
    }
    Header {
        len: total_len as u16,
        id: TX_CONFIRM_ID,
    }
    .encode(output)?;
    output[HEADER_LEN..total_len].fill(0);
    write_u32(output, HEADER_LEN, packet_id);
    write_u32(output, HEADER_LEN + 4, status);
    output[HEADER_LEN + 8] = tx_rate;
    output[HEADER_LEN + 9] = ack_failures;
    Ok(total_len)
}

pub fn encode_join_complete_indication(
    status: u32,
    output: &mut [u8],
) -> Result<usize, Error> {
    const LEN: usize = HEADER_LEN + 4;
    if output.len() < LEN {
        return Err(Error::OutputTooSmall);
    }
    Header {
        len: LEN as u16,
        id: JOIN_COMPLETE_IND_ID,
    }
    .encode(output)?;
    write_u32(output, HEADER_LEN, status);
    Ok(LEN)
}

pub fn encode_receive_indication(
    if_id: u8,
    status: u32,
    channel: u16,
    rate: u8,
    rcpi: u8,
    flags: u32,
    frame: &[u8],
    output: &mut [u8],
) -> Result<usize, Error> {
    const METADATA_LEN: usize = 12;
    let len = HEADER_LEN + METADATA_LEN + frame.len();
    if if_id > 2 || len > u16::MAX as usize || output.len() < len {
        return Err(Error::OutputTooSmall);
    }

    Header {
        len: len as u16,
        id: 0x0804 | (u16::from(if_id) << 6),
    }
    .encode(output)?;
    write_u32(output, 4, status);
    write_u16(output, 8, channel);
    output[10] = rate;
    output[11] = rcpi;
    write_u32(output, 12, flags);
    output[16..len].copy_from_slice(frame);
    Ok(len)
}

pub fn encode_scan_complete_indication(
    status: u32,
    psm: u8,
    num_channels: u8,
    vendor_field: u16,
    output: &mut [u8],
) -> Result<usize, Error> {
    const LEN: usize = HEADER_LEN + 8;
    if output.len() < LEN {
        return Err(Error::OutputTooSmall);
    }

    Header {
        len: LEN as u16,
        id: 0x0806,
    }
    .encode(output)?;
    write_u32(output, 4, status);
    output[8] = psm;
    output[9] = num_channels;
    output[10..12].copy_from_slice(&vendor_field.to_le_bytes());
    Ok(LEN)
}

pub fn encode_configuration_response(
    station_id: [u8; 6],
    tx_power_ranges: [TxPowerRange; 2],
    output: &mut [u8],
) -> Result<usize, Error> {
    const LEN: usize = HEADER_LEN + 40;
    if output.len() < LEN {
        return Err(Error::OutputTooSmall);
    }

    output[..LEN].fill(0);
    Header {
        len: LEN as u16,
        id: 0x0409,
    }
    .encode(output)?;
    write_u32(output, 4, 0);
    output[8..14].copy_from_slice(&station_id);
    output[14] = 1; // 2.4 GHz
    write_u32(output, 16, 0x0000_3fff);
    // Annotated `phy_get_tx_power_range` (`0x00016e9c`) publishes these as two
    // sign-extended triples. Their maxima are SDD profile fields, not constants.
    for (offset, range) in [20, 32].into_iter().zip(tx_power_ranges) {
        write_u32(output, offset, range.min_power_level as u32);
        write_u32(output, offset + 4, range.max_power_level as u32);
        write_u32(output, offset + 8, range.stepping as u32);
    }
    Ok(LEN)
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, Error> {
    let value = bytes.get(offset..offset + 2).ok_or(Error::Truncated)?;
    Ok(u16::from_le_bytes([value[0], value[1]]))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, Error> {
    let value = bytes.get(offset..offset + 4).ok_or(Error::Truncated)?;
    Ok(u32::from_le_bytes([value[0], value[1], value[2], value[3]]))
}

fn write_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_masks_transport_fields() {
        let header = Header { len: 8, id: 0xa449 };
        assert_eq!(header.base_id(), CONFIGURATION_RESP_ID);
        assert_eq!(header.if_id(), 1);
    }

    #[test]
    fn startup_indication_matches_cw1200_layout() {
        let mut output = [0xaa; 168];
        let len = StartupIndication {
            input_buffers: 30,
            input_buffer_size: 1632,
            status: 0,
            hardware_id: 7,
            hardware_sub_id: 9,
            firmware_capabilities: 1,
            firmware_type: 2,
            firmware_api: 1060,
            firmware_build: 5258,
            firmware_version: 8,
            label: b"XR819 open firmware",
            config: [1, 2, 3, 4],
        }
        .encode(&mut output)
        .unwrap();

        assert_eq!(len, 168);
        assert_eq!(Header::parse(&output).unwrap().base_id(), STARTUP_IND_ID);
        assert_eq!(&output[4..8], &[30, 0, 0x60, 0x06]);
        assert_eq!(&output[8..14], &[7, 0, 9, 0, 0, 0]);
        assert_eq!(&output[24..43], b"XR819 open firmware");
        assert_eq!(
            &output[152..168],
            &[1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0]
        );
    }

    #[test]
    fn start_scan_request_borrows_channel_and_ssid_records() {
        let mut payload = [0; 12 + 2 * 16 + 36];
        payload[..4].copy_from_slice(&[0, 1, 2, 6]);
        write_u32(&mut payload, 4, 30_000);
        payload[8..12].copy_from_slice(&[3, 2, 1, 5]);
        write_u16(&mut payload, 12, 1);
        write_u32(&mut payload, 16, 10);
        write_u32(&mut payload, 20, 40);
        write_u16(&mut payload, 28, 11);
        write_u32(&mut payload, 32, 20);
        write_u32(&mut payload, 36, 50);
        write_u32(&mut payload, 44, 4);
        payload[48..52].copy_from_slice(b"test");

        let request = StartScanRequest::parse(&payload).unwrap();
        assert_eq!(request.num_channels, 2);
        assert_eq!(request.channel(0).unwrap().number, 1);
        assert_eq!(request.channel(1).unwrap().number, 11);
        assert_eq!(request.channel(1).unwrap().max_channel_time, 50);
        assert_eq!(request.ssid(0).unwrap(), b"test");
    }

    #[test]
    fn edca_request_restores_linux_queue_order() {
        let mut payload = [0_u8; 44];
        for wire_index in 0..4 {
            write_u16(&mut payload, wire_index * 2, 10 + wire_index as u16);
            write_u16(&mut payload, 8 + wire_index * 2, 20 + wire_index as u16);
            payload[16 + wire_index] = 30 + wire_index as u8;
            write_u16(&mut payload, 20 + wire_index * 2, 40 + wire_index as u16);
            write_u32(&mut payload, 28 + wire_index * 4, 50 + wire_index as u32);
        }
        let parameters = EdcaParameters::parse(&payload).unwrap();
        assert_eq!(parameters.queues[3].cwmin, 10);
        assert_eq!(parameters.queues[0].cwmin, 13);
        assert_eq!(parameters.queues[2].max_rx_lifetime, 51);
        assert_eq!(parameters.queues[1].txop_limit, 42);
    }

    #[test]
    fn receive_indication_matches_driver_layout() {
        let frame = [0x80, 0x00, 1, 2, 3, 4];
        let mut output = [0; 32];
        let length =
            encode_receive_indication(0, 0, 6, 0, 120, 1 << 7, &frame, &mut output).unwrap();
        assert_eq!(length, 22);
        assert_eq!(read_u16(&output, 0), Ok(22));
        assert_eq!(read_u16(&output, 2), Ok(0x0804));
        assert_eq!(read_u32(&output, 4), Ok(0));
        assert_eq!(read_u16(&output, 8), Ok(6));
        assert_eq!(output[10], 0);
        assert_eq!(output[11], 120);
        assert_eq!(read_u32(&output, 12), Ok(1 << 7));
        assert_eq!(&output[16..22], &frame);
    }

    #[test]
    fn scan_complete_matches_driver_layout() {
        let mut output = [0xaa; 12];
        let len = encode_scan_complete_indication(0, 1, 13, 0x1234, &mut output).unwrap();

        assert_eq!(len, 12);
        assert_eq!(Header::parse(&output).unwrap().base_id(), 0x0806);
        assert_eq!(&output[4..8], &[0; 4]);
        assert_eq!(&output[8..12], &[1, 13, 0x34, 0x12]);
    }

    #[test]
    fn configuration_response_matches_driver_layout() {
        let mut output = [0xaa; 44];
        let station_id = [0, 1, 2, 3, 4, 5];
        let len = encode_configuration_response(
            station_id,
            [
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
            ],
            &mut output,
        )
        .unwrap();

        assert_eq!(len, 44);
        assert_eq!(
            Header::parse(&output).unwrap().base_id(),
            CONFIGURATION_RESP_ID
        );
        assert_eq!(&output[4..8], &[0; 4]);
        assert_eq!(&output[8..14], &station_id);
        assert_eq!(output[14], 1);
        assert_eq!(&output[16..20], &0x3fff_u32.to_le_bytes());
        assert_eq!(read_u32(&output, 20).unwrap(), (-160_i32) as u32);
        assert_eq!(read_u32(&output, 24).unwrap(), 272);
        assert_eq!(read_u32(&output, 28).unwrap(), 0);
        assert_eq!(read_u32(&output, 32).unwrap(), (-160_i32) as u32);
        assert_eq!(read_u32(&output, 36).unwrap(), 212);
        assert_eq!(read_u32(&output, 40).unwrap(), 0);
    }

    #[test]
    fn unsupported_read_mib_and_join_confirmations_are_length_correct() {
        let mut read_mib = [0xaa; 12];
        assert_eq!(
            encode_read_mib_response(STATUS_FAILURE, 0x1006, &mut read_mib).unwrap(),
            12
        );
        assert_eq!(
            Header::parse(&read_mib).unwrap().base_id(),
            READ_MIB_RESP_ID
        );
        assert_eq!(read_u32(&read_mib, 4).unwrap(), STATUS_FAILURE);
        assert_eq!(read_u16(&read_mib, 8).unwrap(), 0x1006);
        assert_eq!(read_u16(&read_mib, 10).unwrap(), 0);

        let mut join = [0xaa; 16];
        assert_eq!(
            encode_join_response(STATUS_FAILURE, -160, 200, &mut join).unwrap(),
            16
        );
        assert_eq!(Header::parse(&join).unwrap().base_id(), JOIN_RESP_ID);
        assert_eq!(read_u32(&join, 4).unwrap(), STATUS_FAILURE);
        assert_eq!(read_u32(&join, 8).unwrap(), (-160_i32) as u32);
        assert_eq!(read_u32(&join, 12).unwrap(), 200);
    }

    #[test]
    fn join_request_matches_linux_wire_layout() {
        let mut payload = [0_u8; JoinRequest::PAYLOAD_LEN];
        payload[0] = 1;
        payload[2..4].copy_from_slice(&6_u16.to_le_bytes());
        payload[4..10].copy_from_slice(&[0x02, 1, 2, 3, 4, 5]);
        payload[0x0d] = 1;
        payload[0x0e] = 2;
        payload[0x10..0x14].copy_from_slice(&4_u32.to_le_bytes());
        payload[0x14..0x18].copy_from_slice(b"test");
        payload[0x34..0x38].copy_from_slice(&100_u32.to_le_bytes());
        payload[0x38..0x3c].copy_from_slice(&7_u32.to_le_bytes());

        let request = JoinRequest::parse(&payload).unwrap_or(JoinRequest {
            mode: 0,
            band: 1,
            channel_number: 0,
            bssid: [0; 6],
            atim_window: 1,
            preamble_type: 1,
            probe_for_join: false,
            dtim_period: 0,
            flags: 0xff,
            ssid: &[],
            beacon_interval: 0,
            basic_rate_set: 0,
        });
        assert!(request.supported_sta_shape());
        assert_eq!(request.channel_number, 6);
        assert_eq!(request.ssid, b"test");
        assert_eq!(request.basic_rate_set, 7);
    }

    #[test]
    fn failed_tx_confirm_has_complete_driver_shape() {
        let mut output = [0_u8; 24];
        assert_eq!(
            encode_tx_confirm(0x1234_5678, STATUS_FAILURE, &mut output),
            Ok(24)
        );
        assert_eq!(&output[..4], &[24, 0, 4, 4]);
        assert_eq!(&output[4..8], &0x1234_5678_u32.to_le_bytes());
        assert_eq!(&output[8..12], &STATUS_FAILURE.to_le_bytes());
        assert!(output[12..].iter().all(|value| *value == 0));
    }

    #[test]
    fn tx_request_strips_optional_alignment_before_management_frame() {
        let mut payload = [0_u8; TxRequest::PAYLOAD_HEADER_LEN + 2 + 24];
        payload[..4].copy_from_slice(&0x1234_5678_u32.to_le_bytes());
        payload[4] = 6;
        payload[5] = 3;
        payload[7] = 0x80;
        payload[22..24].copy_from_slice(&0x00b0_u16.to_le_bytes());
        payload[26..32].copy_from_slice(&[0x02, 1, 2, 3, 4, 5]);

        let request = TxRequest::parse(&payload).unwrap_or_else(|error| panic!("{error}"));
        assert_eq!(request.packet_id, 0x1234_5678);
        assert_eq!(request.max_tx_rate, 6);
        assert_eq!(request.queue_id, 3);
        assert_eq!(&request.frame[..2], &0x00b0_u16.to_le_bytes());
        assert!(request.is_unicast_management());
    }

    #[test]
    fn tx_request_recognizes_qos_eapol_data() {
        let mut payload = [0_u8; TxRequest::PAYLOAD_HEADER_LEN + 34];
        payload[TxRequest::PAYLOAD_HEADER_LEN..][..2]
            .copy_from_slice(&0x0188_u16.to_le_bytes());
        payload[TxRequest::PAYLOAD_HEADER_LEN + 4..][..6]
            .copy_from_slice(&[0x20, 5, 0xb6, 0xff, 1, 0x43]);
        payload[TxRequest::PAYLOAD_HEADER_LEN + 26..][..8]
            .copy_from_slice(&[0xaa, 0xaa, 0x03, 0, 0, 0, 0x88, 0x8e]);

        let request = TxRequest::parse(&payload).unwrap();
        assert!(!request.is_unicast_management());
        assert!(request.is_unicast_eapol());
    }

    #[test]
    fn configuration_request_borrows_dpd_data() {
        let mut payload = [0; 27];
        write_u32(&mut payload, 0, 512);
        write_u32(&mut payload, 4, 1024);
        write_u32(&mut payload, 8, 2347);
        write_u16(&mut payload, 12, 15);
        write_u16(&mut payload, 14, 1);
        payload[16..22].copy_from_slice(&[0, 1, 2, 3, 4, 5]);
        write_u16(&mut payload, 22, 5);
        payload[24..].copy_from_slice(&[0xc0, 1, 2]);

        let request = ConfigurationRequest::parse(&payload).unwrap();
        assert_eq!(request.station_id, [0, 1, 2, 3, 4, 5]);
        assert_eq!(request.dpd_data, &[0xc0, 1, 2]);
    }
}
