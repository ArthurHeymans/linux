//! Internal management-frame TX preparation.
//!
//! This module intentionally stops before hardware publication. The vendor
//! queue/pipe ownership and IRQ completion path must be translated before a
//! prepared frame can become DMA-owned.

use crate::configuration::MAX_TEMPLATE_FRAME_LEN;

const TX_CONTEXT_BASE: usize = 0x0400_9084;
const TX_CONTEXT_SIZE: usize = 0x170;
const TX_CONTEXT_COUNT: usize = 3;
const TX_BUFFER_BASE: usize = 0x0901_4fa8;
const TX_BUFFER_SIZE: usize = 0x400;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProbeBuildError {
    MissingTemplate,
    WrongTemplateType,
    MalformedTemplate,
    SsidTooLong,
    FrameTooLarge,
}

pub struct PreparedProbe {
    bytes: [u8; MAX_TEMPLATE_FRAME_LEN],
    length: usize,
    rate: u8,
}

impl PreparedProbe {
    pub fn bytes(&self) -> &[u8] {
        &self.bytes[..self.length]
    }

    pub fn rate(&self) -> u8 {
        self.rate
    }
}

/// Initializes the three-entry internal management TX pool from vendor
/// `tx_ctx_pool_init` (`0x12574`). This does not publish anything to hardware.
///
/// # Safety
/// DTCM and packet RAM must be mapped and no internal TX context may be owned.
pub unsafe fn initialize_internal_pool() {
    unsafe {
        (0x0400_3688 as *mut u32).write_volatile(0x0901_5ba8);
        (0x0400_36f8 as *mut u32).write_volatile(0x0901_5ba8);

        let mut previous = 0_u32;
        for index in 0..TX_CONTEXT_COUNT {
            let context = TX_CONTEXT_BASE + index * TX_CONTEXT_SIZE;
            let buffer = TX_BUFFER_BASE + index * TX_BUFFER_SIZE;
            ((context + 0x1c) as *mut u32).write_volatile((buffer + 0x40) as u32);
            ((context + 0xc4) as *mut u32).write_volatile((buffer + 0x20) as u32);
            ((context + 0x70) as *mut u16).write_volatile(0x00ff);
            ((context + 0x04) as *mut u32).write_volatile(previous);
            previous = context as u32;
        }
        (0x0400_9080 as *mut u32).write_volatile(previous);
    }
}

/// Builds the 802.11 probe request portion of vendor
/// `syn_scan_build_probe_req` (`0x141b0`). The four-byte WSM template header is
/// consumed, the requested SSID is substituted, and every DS Parameter Set IE
/// is rewritten for the active channel.
pub fn prepare_probe(
    template: Option<&[u8]>,
    ssid: &[u8],
    channel: u8,
) -> Result<PreparedProbe, ProbeBuildError> {
    if ssid.len() > 32 {
        return Err(ProbeBuildError::SsidTooLong);
    }
    let template = template.ok_or(ProbeBuildError::MissingTemplate)?;
    if template.len() < 4 {
        return Err(ProbeBuildError::MalformedTemplate);
    }
    if template[0] != 0 {
        return Err(ProbeBuildError::WrongTemplateType);
    }
    let rate = template[1];
    let declared = usize::from(u16::from_le_bytes([template[2], template[3]]));
    if declared < 24 || declared != template.len() - 4 {
        return Err(ProbeBuildError::MalformedTemplate);
    }
    let frame = &template[4..];
    let mut bytes = [0_u8; MAX_TEMPLATE_FRAME_LEN];
    bytes[..24].copy_from_slice(&frame[..24]);
    let mut output = 24;
    let mut input = 24;
    let mut wrote_ssid = false;
    while input < frame.len() {
        if input + 2 > frame.len() {
            return Err(ProbeBuildError::MalformedTemplate);
        }
        let id = frame[input];
        let length = usize::from(frame[input + 1]);
        let next = input
            .checked_add(2 + length)
            .ok_or(ProbeBuildError::MalformedTemplate)?;
        if next > frame.len() {
            return Err(ProbeBuildError::MalformedTemplate);
        }
        if id == 0 {
            if !wrote_ssid {
                let end = output + 2 + ssid.len();
                if end > bytes.len() {
                    return Err(ProbeBuildError::FrameTooLarge);
                }
                bytes[output] = 0;
                bytes[output + 1] = ssid.len() as u8;
                bytes[output + 2..end].copy_from_slice(ssid);
                output = end;
                wrote_ssid = true;
            }
        } else if id == 3 {
            if output + 3 > bytes.len() {
                return Err(ProbeBuildError::FrameTooLarge);
            }
            bytes[output..output + 3].copy_from_slice(&[3, 1, channel]);
            output += 3;
        } else {
            let encoded = &frame[input..next];
            let end = output + encoded.len();
            if end > bytes.len() {
                return Err(ProbeBuildError::FrameTooLarge);
            }
            bytes[output..end].copy_from_slice(encoded);
            output = end;
        }
        input = next;
    }
    if !wrote_ssid {
        let end = output + 2 + ssid.len();
        if end > bytes.len() {
            return Err(ProbeBuildError::FrameTooLarge);
        }
        bytes[output] = 0;
        bytes[output + 1] = ssid.len() as u8;
        bytes[output + 2..end].copy_from_slice(ssid);
        output = end;
    }
    Ok(PreparedProbe {
        bytes,
        length: output,
        rate,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probe_builder_substitutes_ssid_and_channel() {
        let mut template = [0_u8; 4 + 24 + 2 + 3 + 4];
        template[0] = 0;
        template[1] = 6;
        let frame_len = (template.len() - 4) as u16;
        template[2..4].copy_from_slice(&frame_len.to_le_bytes());
        let ies = 4 + 24;
        template[ies..ies + 2].copy_from_slice(&[0, 0]);
        template[ies + 2..ies + 5].copy_from_slice(&[3, 1, 11]);
        template[ies + 5..ies + 9].copy_from_slice(&[1, 2, 0x82, 0x84]);

        let probe = match prepare_probe(Some(&template), b"test", 6) {
            Ok(probe) => probe,
            Err(error) => panic!("probe build failed: {error:?}"),
        };
        assert_eq!(probe.rate(), 6);
        assert_eq!(
            &probe.bytes()[24..],
            &[0, 4, b't', b'e', b's', b't', 3, 1, 6, 1, 2, 0x82, 0x84]
        );
    }

    #[test]
    fn probe_builder_rejects_non_probe_template() {
        assert_eq!(
            prepare_probe(Some(&[1, 0, 0, 0]), b"", 1).err(),
            Some(ProbeBuildError::WrongTemplateType)
        );
    }
}
