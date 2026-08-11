use aes::Aes128;
use ccm::{
    Ccm,
    aead::{AeadInPlace, KeyInit, generic_array::GenericArray},
    consts::{U8, U13},
};
use core::cell::UnsafeCell;

const MAX_KEYS: usize = 24;
const CCMP_HEADER_LEN: usize = 8;
pub(crate) const CCMP_MIC_LEN: usize = 8;
const AES_GROUP: u8 = 4;
const AES_PAIRWISE: u8 = 5;

type AesCcmp = Ccm<Aes128, U8, U13>;

#[derive(Clone, Copy)]
struct KeyRecord {
    active: bool,
    key_type: u8,
    if_id: u8,
    peer: [u8; 6],
    key_id: u8,
    key: [u8; 16],
    tx_pn: u64,
}

impl KeyRecord {
    const EMPTY: Self = Self {
        active: false,
        key_type: 0,
        if_id: 0,
        peer: [0; 6],
        key_id: 0,
        key: [0; 16],
        tx_pn: 0,
    };
}

struct SharedKeys(UnsafeCell<[KeyRecord; MAX_KEYS]>);
unsafe impl Sync for SharedKeys {}

static KEYS: SharedKeys = SharedKeys(UnsafeCell::new([KeyRecord::EMPTY; MAX_KEYS]));

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeyError {
    InvalidIndex,
    InvalidLength,
    UnsupportedType,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CcmpError {
    MalformedFrame,
    MissingKey,
    Authentication,
}

pub fn add_key(if_id: u8, request: &crate::wsm::AddKeyRequest<'_>) -> Result<(), KeyError> {
    let index = usize::from(request.index);
    if index >= MAX_KEYS {
        return Err(KeyError::InvalidIndex);
    }
    let mut record = KeyRecord {
        active: true,
        key_type: request.key_type,
        if_id,
        peer: [0; 6],
        key_id: 0,
        key: [0; 16],
        tx_pn: 0,
    };
    match request.key_type {
        AES_GROUP => {
            record.key.copy_from_slice(&request.data[..16]);
            record.key_id = request.data[16] & 3;
        }
        AES_PAIRWISE => {
            record.peer.copy_from_slice(&request.data[..6]);
            record.key.copy_from_slice(&request.data[8..24]);
        }
        _ => return Err(KeyError::UnsupportedType),
    }
    unsafe { (*KEYS.0.get())[index] = record };
    Ok(())
}

pub fn remove_key(index: u8) -> Result<(), KeyError> {
    let index = usize::from(index);
    if index >= MAX_KEYS {
        return Err(KeyError::InvalidIndex);
    }
    unsafe { (*KEYS.0.get())[index] = KeyRecord::EMPTY };
    Ok(())
}

fn frame_control(frame: &[u8]) -> Result<u16, CcmpError> {
    frame
        .get(..2)
        .map(|value| u16::from_le_bytes([value[0], value[1]]))
        .ok_or(CcmpError::MalformedFrame)
}

fn header_length(frame_control: u16) -> usize {
    let mut length = if frame_control & 0x0300 == 0x0300 {
        30
    } else {
        24
    };
    if frame_control & 0x008c == 0x0088 {
        length += if frame_control & 0x8000 != 0 { 6 } else { 2 };
    }
    length
}

pub fn is_protected_eapol(frame: &[u8]) -> bool {
    let Ok(control) = frame_control(frame) else {
        return false;
    };
    if control & 0x400c != 0x4008 {
        return false;
    }
    let llc = header_length(control) + CCMP_HEADER_LEN;
    frame.get(llc..llc + 8).is_some_and(|value| {
        value[..6] == [0xaa, 0xaa, 0x03, 0x00, 0x00, 0x00] && value[6..] == [0x88, 0x8e]
    })
}

pub fn strip_qos_control(frame: &mut [u8]) -> Result<usize, CcmpError> {
    let control = frame_control(frame)?;
    if control & 0x008f != 0x0088 {
        return Ok(frame.len());
    }
    let base = if control & 0x0300 == 0x0300 { 30 } else { 24 };
    let qos_length = if control & 0x8000 != 0 { 6 } else { 2 };
    if frame.len() < base + qos_length {
        return Err(CcmpError::MalformedFrame);
    }
    frame.copy_within(base + qos_length.., base);
    frame[0] &= !0x80;
    frame[1] &= !0x80;
    Ok(frame.len() - qos_length)
}

pub fn strip_ccmp_reservation(frame: &mut [u8]) -> Result<usize, CcmpError> {
    let control = frame_control(frame)?;
    if control & 0x400c != 0x4008 {
        return Ok(frame.len());
    }
    let header = header_length(control);
    if frame.len() < header + CCMP_HEADER_LEN + CCMP_MIC_LEN {
        return Err(CcmpError::MalformedFrame);
    }
    let plaintext_end = frame.len() - CCMP_MIC_LEN;
    frame.copy_within(header + CCMP_HEADER_LEN..plaintext_end, header);
    frame[1] &= !0x40;
    Ok(frame.len() - CCMP_HEADER_LEN - CCMP_MIC_LEN)
}

fn qos_tid(frame: &[u8], frame_control: u16) -> Result<u8, CcmpError> {
    if frame_control & 0x008c != 0x0088 {
        return Ok(0);
    }
    let offset = if frame_control & 0x0300 == 0x0300 {
        30
    } else {
        24
    };
    frame
        .get(offset)
        .map(|value| value & 0x0f)
        .ok_or(CcmpError::MalformedFrame)
}

fn build_aad(frame: &[u8], frame_control: u16) -> Result<([u8; 30], usize), CcmpError> {
    if frame.len() < 24 {
        return Err(CcmpError::MalformedFrame);
    }
    let mut aad = [0_u8; 30];
    let mut masked = frame_control & !(0x0800 | 0x1000 | 0x2000);
    if frame_control & 0x000c != 0 {
        masked &= !0x0070;
    }
    masked |= 0x4000;
    let qos = frame_control & 0x008c == 0x0088;
    if qos {
        masked &= !0x8000;
    }
    aad[..2].copy_from_slice(&masked.to_le_bytes());
    aad[2..20].copy_from_slice(&frame[4..22]);
    aad[20] = frame[22] & 0x0f;
    aad[21] = 0;
    let mut length = 22;
    if frame_control & 0x0300 == 0x0300 {
        aad[22..28].copy_from_slice(frame.get(24..30).ok_or(CcmpError::MalformedFrame)?);
        length = 28;
    }
    if qos {
        aad[length] = qos_tid(frame, frame_control)?;
        aad[length + 1] = 0;
        length += 2;
    }
    Ok((aad, length))
}

fn build_nonce(frame: &[u8], frame_control: u16, pn: [u8; 6]) -> Result<[u8; 13], CcmpError> {
    let mut nonce = [0_u8; 13];
    nonce[0] = qos_tid(frame, frame_control)?;
    nonce[1..7].copy_from_slice(frame.get(10..16).ok_or(CcmpError::MalformedFrame)?);
    nonce[7..].copy_from_slice(&pn);
    Ok(nonce)
}

fn pn_bytes(value: u64) -> [u8; 6] {
    [
        (value >> 40) as u8,
        (value >> 32) as u8,
        (value >> 24) as u8,
        (value >> 16) as u8,
        (value >> 8) as u8,
        value as u8,
    ]
}

fn tx_key(if_id: u8, receiver: &[u8]) -> Option<(usize, KeyRecord)> {
    unsafe {
        (*KEYS.0.get())
            .iter()
            .copied()
            .enumerate()
            .find(|(_, key)| {
                key.active
                    && key.if_id == if_id
                    && ((key.key_type == AES_PAIRWISE && key.peer == receiver)
                        || key.key_type == AES_GROUP)
            })
    }
}

fn rx_key(if_id: u8, transmitter: &[u8], key_id: u8) -> Option<KeyRecord> {
    unsafe {
        (*KEYS.0.get()).iter().copied().find(|key| {
            key.active
                && key.if_id == if_id
                && ((key.key_type == AES_PAIRWISE && key_id == 0 && key.peer == transmitter)
                    || (key.key_type == AES_GROUP && key.key_id == key_id))
        })
    }
}

pub fn encrypt_tx_frame(frame: &mut [u8], if_id: u8) -> Result<(), CcmpError> {
    let control = frame_control(frame)?;
    if control & 0x400c != 0x4008 {
        return Ok(());
    }
    let header = header_length(control);
    if frame.len() < header + CCMP_HEADER_LEN + CCMP_MIC_LEN {
        return Err(CcmpError::MalformedFrame);
    }
    let receiver: [u8; 6] = frame[4..10]
        .try_into()
        .map_err(|_| CcmpError::MalformedFrame)?;
    let (index, key) = tx_key(if_id, &receiver).ok_or(CcmpError::MissingKey)?;
    let next_pn = key.tx_pn.wrapping_add(1) & 0x0000_ffff_ffff_ffff;
    unsafe { (*KEYS.0.get())[index].tx_pn = next_pn };
    let pn = pn_bytes(next_pn);
    let ccmp = &mut frame[header..header + CCMP_HEADER_LEN];
    ccmp.copy_from_slice(&[
        pn[5],
        pn[4],
        0,
        0x20 | (key.key_id << 6),
        pn[3],
        pn[2],
        pn[1],
        pn[0],
    ]);
    let (aad, aad_length) = build_aad(frame, control)?;
    let nonce = build_nonce(frame, control, pn)?;
    let cipher = AesCcmp::new_from_slice(&key.key).map_err(|_| CcmpError::Authentication)?;
    let payload_end = frame.len() - CCMP_MIC_LEN;
    let tag = cipher
        .encrypt_in_place_detached(
            GenericArray::from_slice(&nonce),
            &aad[..aad_length],
            &mut frame[header + CCMP_HEADER_LEN..payload_end],
        )
        .map_err(|_| CcmpError::Authentication)?;
    frame[payload_end..].copy_from_slice(&tag);
    Ok(())
}

pub fn decrypt_rx_frame(frame: &mut [u8], if_id: u8) -> Result<bool, CcmpError> {
    let control = frame_control(frame)?;
    if control & 0x400c != 0x4008 {
        return Ok(false);
    }
    let header = header_length(control);
    if frame.len() < header + CCMP_HEADER_LEN + CCMP_MIC_LEN {
        return Err(CcmpError::MalformedFrame);
    }
    let ccmp: [u8; 8] = frame[header..header + CCMP_HEADER_LEN]
        .try_into()
        .map_err(|_| CcmpError::MalformedFrame)?;
    let pn = [ccmp[7], ccmp[6], ccmp[5], ccmp[4], ccmp[1], ccmp[0]];
    let key_id = ccmp[3] >> 6;
    let transmitter: [u8; 6] = frame[10..16]
        .try_into()
        .map_err(|_| CcmpError::MalformedFrame)?;
    let key = rx_key(if_id, &transmitter, key_id).ok_or(CcmpError::MissingKey)?;
    let (aad, aad_length) = build_aad(frame, control)?;
    let nonce = build_nonce(frame, control, pn)?;
    let cipher = AesCcmp::new_from_slice(&key.key).map_err(|_| CcmpError::Authentication)?;
    let payload_end = frame.len() - CCMP_MIC_LEN;
    let tag = GenericArray::clone_from_slice(&frame[payload_end..]);
    cipher
        .decrypt_in_place_detached(
            GenericArray::from_slice(&nonce),
            &aad[..aad_length],
            &mut frame[header + CCMP_HEADER_LEN..payload_end],
            &tag,
        )
        .map_err(|_| CcmpError::Authentication)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    extern crate std;

    use self::std::sync::Mutex;
    use super::*;

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn pairwise_request<'a>(
        payload: &'a mut [u8; 44],
        index: u8,
        peer: [u8; 6],
    ) -> crate::wsm::AddKeyRequest<'a> {
        payload[0] = AES_PAIRWISE;
        payload[1] = index;
        payload[4..10].copy_from_slice(&peer);
        payload[12..28].copy_from_slice(&[0x11; 16]);
        crate::wsm::AddKeyRequest::parse(payload).unwrap()
    }

    #[test]
    fn ccmp_matches_independent_aesccm_known_answer() {
        let _guard = TEST_LOCK.lock().unwrap();
        unsafe {
            *KEYS.0.get() = [KeyRecord::EMPTY; MAX_KEYS];
            (*KEYS.0.get())[0] = KeyRecord {
                active: true,
                key_type: AES_PAIRWISE,
                if_id: 0,
                peer: [0x20, 0x05, 0xb6, 0xff, 0x01, 0x43],
                key_id: 0,
                key: [
                    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c,
                    0x0d, 0x0e, 0x0f,
                ],
                tx_pn: 0x0123_4567_89aa,
            };
            (*KEYS.0.get())[1] = KeyRecord {
                peer: [0x12, 0x42, 0x2a, 0x37, 0x70, 0x07],
                tx_pn: 0,
                ..(*KEYS.0.get())[0]
            };
        }
        let mut frame = [0_u8; 26 + 8 + 16 + 8];
        frame[..26].copy_from_slice(&[
            0x88, 0x41, 0x00, 0x00, 0x20, 0x05, 0xb6, 0xff, 0x01, 0x43, 0x12, 0x42, 0x2a, 0x37,
            0x70, 0x07, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x30, 0x12, 0x03, 0x00,
        ]);
        frame[34..50].copy_from_slice(&[
            0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d,
            0x1e, 0x1f,
        ]);

        encrypt_tx_frame(&mut frame, 0).unwrap();
        assert_eq!(
            &frame[26..34],
            &[0xab, 0x89, 0x00, 0x20, 0x67, 0x45, 0x23, 0x01]
        );
        assert_eq!(
            &frame[34..50],
            &[
                0xc7, 0x40, 0x95, 0x9b, 0xf5, 0x9a, 0x96, 0x31, 0xa8, 0xc2, 0xc9, 0xd9, 0x91, 0x0d,
                0x11, 0x4b
            ]
        );
        assert_eq!(
            &frame[50..58],
            &[0x11, 0x85, 0x85, 0x5b, 0xce, 0x30, 0xf3, 0xe9]
        );

        let encrypted = frame;
        assert!(decrypt_rx_frame(&mut frame, 0).unwrap());
        assert_eq!(
            &frame[34..50],
            &[
                0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d,
                0x1e, 0x1f
            ]
        );

        let mut bad_ciphertext = encrypted;
        bad_ciphertext[34] ^= 1;
        assert_eq!(
            decrypt_rx_frame(&mut bad_ciphertext, 0),
            Err(CcmpError::Authentication)
        );
        let mut bad_mic = encrypted;
        bad_mic[57] ^= 1;
        assert_eq!(
            decrypt_rx_frame(&mut bad_mic, 0),
            Err(CcmpError::Authentication)
        );
    }

    #[test]
    fn qos_control_can_be_removed_before_ccmp_processing() {
        let mut frame = [0_u8; 40];
        frame[..2].copy_from_slice(&0x4188_u16.to_le_bytes());
        frame[24..26].copy_from_slice(&[5, 0]);
        frame[26..40].copy_from_slice(b"reserved+data!");

        let length = strip_qos_control(&mut frame).unwrap();
        assert_eq!(length, 38);
        assert_eq!(u16::from_le_bytes([frame[0], frame[1]]), 0x4108);
        assert_eq!(&frame[24..38], b"reserved+data!");
    }

    #[test]
    fn ccmp_round_trip_preserves_reserved_header_and_mic_space() {
        let _guard = TEST_LOCK.lock().unwrap();
        unsafe { *KEYS.0.get() = [KeyRecord::EMPTY; MAX_KEYS] };
        let mut tx_request = [0_u8; 44];
        add_key(
            0,
            &pairwise_request(&mut tx_request, 0, [0x20, 0x05, 0xb6, 0xff, 0x01, 0x43]),
        )
        .unwrap();
        let mut rx_request = [0_u8; 44];
        add_key(
            0,
            &pairwise_request(&mut rx_request, 1, [0x12, 0x42, 0x2a, 0x37, 0x70, 0x07]),
        )
        .unwrap();
        let mut frame = [0_u8; 26 + 8 + 12 + 8];
        frame[..2].copy_from_slice(&0x4188_u16.to_le_bytes());
        frame[4..10].copy_from_slice(&[0x20, 0x05, 0xb6, 0xff, 0x01, 0x43]);
        frame[10..16].copy_from_slice(&[0x12, 0x42, 0x2a, 0x37, 0x70, 0x07]);
        frame[16..22].copy_from_slice(&[0x20, 0x05, 0xb6, 0xff, 0x01, 0x43]);
        frame[24] = 0;
        frame[34..46].copy_from_slice(b"hello world!");
        let plaintext: [u8; 12] = frame[34..46].try_into().unwrap();

        encrypt_tx_frame(&mut frame, 0).unwrap();
        assert_ne!(&frame[34..46], &plaintext);
        assert!(decrypt_rx_frame(&mut frame, 0).unwrap());
        assert_eq!(&frame[34..46], &plaintext);
    }
}
