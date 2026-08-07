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
pub const RECEIVE_IND_ID: u16 = 0x0804;

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

    /// Command ID without the transport sequence and link-ID fields.
    pub const fn base_id(self) -> u16 {
        self.id & 0x0fff
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Truncated,
    InvalidLength,
    InvalidDpdLength,
    OutputTooSmall,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Truncated => "truncated WSM message",
            Self::InvalidLength => "invalid WSM message length",
            Self::InvalidDpdLength => "invalid WSM DPD block length",
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

pub fn encode_configuration_response(
    station_id: [u8; 6],
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
    // Two conservative power ranges in 0.1 dBm units.
    for offset in [20, 32] {
        write_u32(output, offset, 0);
        write_u32(output, offset + 4, 200);
        write_u32(output, offset + 8, 10);
    }
    Ok(LEN)
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, Error> {
    let value = bytes.get(offset..offset + 4).ok_or(Error::Truncated)?;
    Ok(u32::from_le_bytes([value[0], value[1], value[2], value[3]]))
}

#[cfg(test)]
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
        let header = Header { len: 8, id: 0xa409 };
        assert_eq!(header.base_id(), CONFIGURATION_RESP_ID);
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
            firmware_capabilities: 3,
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
    fn configuration_response_matches_driver_layout() {
        let mut output = [0xaa; 44];
        let station_id = [0, 1, 2, 3, 4, 5];
        let len = encode_configuration_response(station_id, &mut output).unwrap();

        assert_eq!(len, 44);
        assert_eq!(
            Header::parse(&output).unwrap().base_id(),
            CONFIGURATION_RESP_ID
        );
        assert_eq!(&output[4..8], &[0; 4]);
        assert_eq!(&output[8..14], &station_id);
        assert_eq!(output[14], 1);
        assert_eq!(&output[16..20], &0x3fff_u32.to_le_bytes());
        assert_eq!(&output[20..32], &[0, 0, 0, 0, 200, 0, 0, 0, 10, 0, 0, 0]);
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
