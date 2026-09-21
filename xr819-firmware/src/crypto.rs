#[cfg(not(target_arch = "arm"))]
use aes::Aes128;
#[cfg(not(target_arch = "arm"))]
use ccm::{
    Ccm,
    aead::{AeadInPlace, KeyInit, generic_array::GenericArray},
    consts::{U8, U13},
};
use core::cell::UnsafeCell;
#[cfg(target_arch = "arm")]
use tock_registers::{
    interfaces::{Readable, Writeable},
    register_structs,
    registers::ReadWrite,
};

const MAX_KEYS: usize = 24;
const CCMP_HEADER_LEN: usize = 8;
pub(crate) const CCMP_MIC_LEN: usize = 8;
const AES_GROUP: u8 = 4;
const AES_PAIRWISE: u8 = 5;

mod hardware_kat {
    include!(concat!(env!("OUT_DIR"), "/hardware_ccmp_kat.rs"));
}

#[cfg(not(target_arch = "arm"))]
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
    HardwareTimeout,
    InvalidDmaAddress,
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

/// Return the 48-bit CCMP packet number already embedded in a protected TX
/// frame. Diagnostics use this as the non-wrapping on-air identity; unlike the
/// 12-bit sequence number it remains unique across a sustained flow.
pub(crate) fn tx_frame_packet_number(frame: &[u8]) -> Option<u64> {
    let control = frame_control(frame).ok()?;
    if control & 0x400c != 0x4008 {
        return None;
    }
    let header = header_length(control);
    let ccmp = frame.get(header..header + CCMP_HEADER_LEN)?;
    Some(
        u64::from(ccmp[0])
            | (u64::from(ccmp[1]) << 8)
            | (u64::from(ccmp[4]) << 16)
            | (u64::from(ccmp[5]) << 24)
            | (u64::from(ccmp[6]) << 32)
            | (u64::from(ccmp[7]) << 40),
    )
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

fn rx_key(
    if_id: u8,
    transmitter: &[u8],
    key_id: u8,
    group_addressed: bool,
) -> Option<KeyRecord> {
    unsafe {
        (*KEYS.0.get()).iter().copied().find(|key| {
            key.active
                && key.if_id == if_id
                // Key ID zero is valid for a GTK too. Choose the key class from
                // Address 1 before lookup, rather than letting table order decide.
                && if group_addressed {
                    key.key_type == AES_GROUP && key.key_id == key_id
                } else {
                    key.key_type == AES_PAIRWISE && key_id == 0 && key.peer == transmitter
                }
        })
    }
}

const fn hardware_aad_tail_command(aad_length: usize) -> Option<u32> {
    match aad_length {
        22 => Some(0x148b),
        24 => Some(0x14ab),
        28 => Some(0x14eb),
        30 => Some(0x140b),
        _ => None,
    }
}

fn hardware_ccm_context(
    frame: &[u8],
    frame_control: u16,
    pn: [u8; 6],
    payload_length: usize,
) -> Result<[u8; 16], CcmpError> {
    if payload_length > u16::MAX as usize || frame.len() < 16 {
        return Err(CcmpError::MalformedFrame);
    }
    let mut context = [0_u8; 16];
    context[0] = 1;
    context[1] = qos_tid(frame, frame_control)?;
    context[2..8].copy_from_slice(&frame[10..16]);
    context[8..14].copy_from_slice(&pn);
    context[14..16].copy_from_slice(&(payload_length as u16).to_be_bytes());
    Ok(context)
}

fn hardware_aad_stream(aad: &[u8; 30], aad_length: usize) -> Result<[u8; 32], CcmpError> {
    if hardware_aad_tail_command(aad_length).is_none() {
        return Err(CcmpError::MalformedFrame);
    }
    let mut stream = [0_u8; 32];
    stream[..2].copy_from_slice(&(aad_length as u16).to_be_bytes());
    stream[2..2 + aad_length].copy_from_slice(&aad[..aad_length]);
    Ok(stream)
}

#[cfg(target_arch = "arm")]
register_structs! {
    AesRegisters {
        (0x00 => command_status: ReadWrite<u32>),
        (0x04 => fifo: ReadWrite<u32>),
        (0x08 => _debug_port: ReadWrite<u32>),
        (0x0c => _reserved),
        (0x10 => source: ReadWrite<u32>),
        (0x14 => destination: ReadWrite<u32>),
        (0x18 => length: ReadWrite<u32>),
        (0x1c => @END),
    }
}

#[cfg(target_arch = "arm")]
struct SharedHardwareCompletion(UnsafeCell<u32>);

#[cfg(target_arch = "arm")]
unsafe impl Sync for SharedHardwareCompletion {}

#[cfg(target_arch = "arm")]
static HARDWARE_COMPLETION: SharedHardwareCompletion = SharedHardwareCompletion(UnsafeCell::new(0));

#[cfg(target_arch = "arm")]
static HARDWARE_IRQ_MASK: SharedHardwareCompletion = SharedHardwareCompletion(UnsafeCell::new(0));

#[cfg(target_arch = "arm")]
static HARDWARE_COMPLETION_STATUS: SharedHardwareCompletion =
    SharedHardwareCompletion(UnsafeCell::new(0));

struct SharedHardwareSelftest(UnsafeCell<[u32; 22]>);

unsafe impl Sync for SharedHardwareSelftest {}

static HARDWARE_SELFTEST: SharedHardwareSelftest = SharedHardwareSelftest(UnsafeCell::new([0; 22]));

#[cfg(target_arch = "arm")]
fn aes_registers() -> &'static AesRegisters {
    unsafe { &*(crate::platform::aes_register_base() as *const AesRegisters) }
}

#[cfg(target_arch = "arm")]
const AES_MODE1_MICROCODE: &[u8; 430] = include_bytes!("../data/aes-mode1.bin");

#[cfg(target_arch = "arm")]
fn initialize_hardware_aes_engine() -> Result<(), CcmpError> {
    let registers = aes_registers();
    registers.command_status.set(0);
    wait_aes_status(|status| status & (1 << 12) == 0)?;
    registers
        ._debug_port
        .set(0x8000_0000 | u32::from(AES_MODE1_MICROCODE[0]));
    for byte in &AES_MODE1_MICROCODE[1..] {
        registers._debug_port.set(u32::from(*byte));
    }
    registers.command_status.set(0x9000);
    wait_aes_status(|status| status & (1 << 12) == 0)?;
    Ok(())
}

#[cfg(target_arch = "arm")]
fn record_hardware_selftest_word(index: usize, value: u32) {
    unsafe {
        if index < 22 && HARDWARE_SELFTEST.0.get().cast::<u32>().read_volatile() == 0x4857_434b {
            HARDWARE_SELFTEST
                .0
                .get()
                .cast::<u32>()
                .add(index)
                .write_volatile(value);
        }
    }
}

#[cfg(target_arch = "arm")]
fn increment_hardware_selftest_word(index: usize) -> u32 {
    unsafe {
        let result = HARDWARE_SELFTEST.0.get().cast::<u32>();
        if index >= 22 || result.read_volatile() != 0x4857_434b {
            return 0;
        }
        let word = result.add(index);
        let value = word.read_volatile().wrapping_add(1);
        word.write_volatile(value);
        value
    }
}

#[cfg(target_arch = "arm")]
fn hardware_crypto_irq(mask: u32) {
    unsafe {
        HARDWARE_COMPLETION_STATUS
            .0
            .get()
            .write_volatile(aes_registers().command_status.get());
        let irqs = HARDWARE_IRQ_MASK.0.get();
        irqs.write_volatile(irqs.read_volatile() | mask);
        HARDWARE_COMPLETION.0.get().write_volatile(1);
    }
}

#[cfg(target_arch = "arm")]
pub(crate) extern "C" fn hardware_crypto_irq18() {
    hardware_crypto_irq(1 << 18);
}

#[cfg(target_arch = "arm")]
pub(crate) extern "C" fn hardware_crypto_irq20() {
    hardware_crypto_irq(1 << 20);
}

#[cfg(target_arch = "arm")]
fn wait_aes_status(predicate: impl Fn(u32) -> bool) -> Result<(), CcmpError> {
    let started = unsafe { crate::vendor_host_tx::vendor_timer_now() };
    while !predicate(aes_registers().command_status.get()) {
        if unsafe { crate::vendor_host_tx::vendor_timer_now() }.wrapping_sub(started) >= 50_000 {
            return Err(CcmpError::HardwareTimeout);
        }
        core::hint::spin_loop();
    }
    Ok(())
}

#[cfg(target_arch = "arm")]
fn write_fifo_block(block: &[u8]) {
    for word in block.chunks_exact(4) {
        aes_registers()
            .fifo
            .set(u32::from_le_bytes([word[0], word[1], word[2], word[3]]));
    }
}

#[cfg(target_arch = "arm")]
fn encrypt_tx_frame_hardware(
    frame: &mut [u8],
    frame_control: u16,
    header: usize,
    key: &[u8; 16],
    pn: [u8; 6],
    aad: &[u8; 30],
    aad_length: usize,
) -> Result<(), CcmpError> {
    let payload_start = header + CCMP_HEADER_LEN;
    let payload_end = frame.len() - CCMP_MIC_LEN;
    let payload_length = payload_end
        .checked_sub(payload_start)
        .ok_or(CcmpError::MalformedFrame)?;
    let payload_address = crate::packet_ram::packet_dma_bus_address(
        frame[payload_start..].as_mut_ptr() as usize,
    )
    .ok_or(CcmpError::InvalidDmaAddress)?;
    let context = hardware_ccm_context(frame, frame_control, pn, payload_length)?;
    let aad_stream = hardware_aad_stream(aad, aad_length)?;
    let registers = aes_registers();

    wait_aes_status(|status| status & (1 << 12) == 0)?;
    write_fifo_block(key);
    registers.command_status.set(0x1100);

    wait_aes_status(|status| status & (1 << 13) != 0)?;
    record_hardware_selftest_word(9, registers.command_status.get());
    write_fifo_block(&context);
    wait_aes_status(|status| status & (1 << 12) == 0)?;
    registers.command_status.set(0x1240);

    wait_aes_status(|status| status & (1 << 13) != 0)?;
    record_hardware_selftest_word(10, registers.command_status.get());
    write_fifo_block(&aad_stream[..16]);
    wait_aes_status(|status| status & (1 << 12) == 0)?;
    registers.command_status.set(0x1403);

    wait_aes_status(|status| status & (1 << 13) != 0)?;
    record_hardware_selftest_word(11, registers.command_status.get());
    write_fifo_block(&aad_stream[16..]);
    wait_aes_status(|status| status & (1 << 12) == 0)?;
    registers
        .command_status
        .set(hardware_aad_tail_command(aad_length).ok_or(CcmpError::MalformedFrame)?);

    registers.source.set(payload_address);
    registers.destination.set(payload_address);
    registers.length.set(payload_length as u32);
    record_hardware_selftest_word(14, registers.source.get());
    record_hardware_selftest_word(15, registers.destination.get());
    record_hardware_selftest_word(16, registers.length.get());
    unsafe {
        HARDWARE_COMPLETION.0.get().write_volatile(0);
        HARDWARE_IRQ_MASK.0.get().write_volatile(0);
        HARDWARE_COMPLETION_STATUS.0.get().write_volatile(0);
    }
    crate::hif::drain_write_buffer();
    wait_aes_status(|status| status & (1 << 12) == 0)?;
    record_hardware_selftest_word(12, registers.command_status.get());
    registers.command_status.set(0x3008_1009);
    record_hardware_selftest_word(13, registers.command_status.get());

    let started = unsafe { crate::vendor_host_tx::vendor_timer_now() };
    while unsafe { HARDWARE_COMPLETION.0.get().read_volatile() } == 0 {
        unsafe {
            const IRQ18: u32 = 1 << 18;
            const IRQ20: u32 = 1 << 20;
            let pending = (0x0a88_0020 as *const u32).read_volatile() & (IRQ18 | IRQ20);
            if pending != 0 {
                // CPU IRQ/FIQ remain masked in this firmware, so reproduce the
                // vendor demultiplexer's acknowledge-before-callback order in
                // the foreground. Per-transfer writes to controller +0x14 are
                // intentionally absent; vendor uses only +0x04 at runtime.
                (0x0a88_0004 as *mut u32).write_volatile(pending);
                if pending & IRQ18 != 0 {
                    hardware_crypto_irq18();
                }
                if pending & IRQ20 != 0 {
                    hardware_crypto_irq20();
                }
            }
        }
        if unsafe { crate::vendor_host_tx::vendor_timer_now() }.wrapping_sub(started) >= 50_000 {
            return Err(CcmpError::HardwareTimeout);
        }
        core::hint::spin_loop();
    }
    crate::hif::drain_write_buffer();
    // TX encryption completion is established by IRQ 20 and the KAT's exact
    // ciphertext/MIC match. AES status bit 0 remains clear before and after
    // acknowledgement for this mode; it is not a TX-success predicate.
    Ok(())
}

#[cfg(target_arch = "arm")]
fn decrypt_rx_frame_hardware(
    frame: &mut [u8],
    frame_control: u16,
    header: usize,
    key: &[u8; 16],
    pn: [u8; 6],
    aad: &[u8; 30],
    aad_length: usize,
) -> Result<(), CcmpError> {
    let payload_start = header + CCMP_HEADER_LEN;
    let payload_end = frame.len() - CCMP_MIC_LEN;
    let payload_length = payload_end
        .checked_sub(payload_start)
        .ok_or(CcmpError::MalformedFrame)?;
    let payload_address = crate::packet_ram::packet_dma_bus_address(
        frame[payload_start..].as_mut_ptr() as usize,
    )
    .ok_or(CcmpError::InvalidDmaAddress)?;
    let context = hardware_ccm_context(frame, frame_control, pn, payload_length)?;
    let aad_stream = hardware_aad_stream(aad, aad_length)?;
    let registers = aes_registers();

    wait_aes_status(|status| status & (1 << 12) == 0)?;
    write_fifo_block(key);
    registers.command_status.set(0x1100);
    wait_aes_status(|status| status & (1 << 13) != 0)?;
    write_fifo_block(&context);
    wait_aes_status(|status| status & (1 << 12) == 0)?;
    registers.command_status.set(0x1240);
    wait_aes_status(|status| status & (1 << 13) != 0)?;
    write_fifo_block(&aad_stream[..16]);
    wait_aes_status(|status| status & (1 << 12) == 0)?;
    registers.command_status.set(0x1402);
    wait_aes_status(|status| status & (1 << 13) != 0)?;
    write_fifo_block(&aad_stream[16..]);
    wait_aes_status(|status| status & (1 << 12) == 0)?;
    registers.command_status.set(
        hardware_aad_tail_command(aad_length)
            .ok_or(CcmpError::MalformedFrame)?
            .wrapping_sub(1),
    );

    registers.source.set(payload_address);
    registers.destination.set(payload_address);
    registers.length.set(payload_length as u32);
    unsafe {
        HARDWARE_COMPLETION.0.get().write_volatile(0);
        HARDWARE_IRQ_MASK.0.get().write_volatile(0);
        HARDWARE_COMPLETION_STATUS.0.get().write_volatile(0);
    }
    crate::hif::drain_write_buffer();
    wait_aes_status(|status| status & (1 << 12) == 0)?;
    registers.command_status.set(0x3008_1008);

    let started = unsafe { crate::vendor_host_tx::vendor_timer_now() };
    while unsafe { HARDWARE_COMPLETION.0.get().read_volatile() } == 0 {
        unsafe {
            const IRQ18: u32 = 1 << 18;
            const IRQ20: u32 = 1 << 20;
            let pending = (0x0a88_0020 as *const u32).read_volatile() & (IRQ18 | IRQ20);
            if pending != 0 {
                (0x0a88_0004 as *mut u32).write_volatile(pending);
                if pending & IRQ18 != 0 {
                    hardware_crypto_irq18();
                }
                if pending & IRQ20 != 0 {
                    hardware_crypto_irq20();
                }
            }
        }
        if unsafe { crate::vendor_host_tx::vendor_timer_now() }.wrapping_sub(started) >= 50_000 {
            return Err(CcmpError::HardwareTimeout);
        }
        core::hint::spin_loop();
    }
    crate::hif::drain_write_buffer();
    if unsafe { HARDWARE_COMPLETION_STATUS.0.get().read_volatile() } & 1 == 0 {
        return Err(CcmpError::Authentication);
    }
    Ok(())
}

#[cfg(target_arch = "arm")]
pub fn run_hardware_ccmp_selftest() {
    use hardware_kat::{
        HARDWARE_KAT_EXPECTED, HARDWARE_KAT_INPUT, HARDWARE_KAT_KEY, HARDWARE_KAT_MATRIX_CHECKSUMS,
        HARDWARE_KAT_MATRIX_LENGTHS, HARDWARE_KAT_MATRIX_MICS, HARDWARE_KAT_PN,
    };

    let scratch = crate::packet_ram::hif_output(0);
    unsafe {
        let result = HARDWARE_SELFTEST.0.get().cast::<u32>();
        result.write_volatile(0x4857_434b); // "HWCK"
        result.add(1).write_volatile(0xffff_0000);
        for (index, byte) in HARDWARE_KAT_INPUT.iter().copied().enumerate() {
            (scratch as *mut u8).add(index).write_volatile(byte);
        }
        let frame = core::slice::from_raw_parts_mut(scratch as *mut u8, HARDWARE_KAT_INPUT.len());
        let control = u16::from_le_bytes([frame[0], frame[1]]);
        let header = header_length(control);
        let (aad, aad_length) = match build_aad(frame, control) {
            Ok(value) => value,
            Err(_) => {
                result.add(1).write_volatile(0xe004);
                return;
            }
        };
        let started = crate::vendor_host_tx::vendor_timer_now();
        result
            .add(7)
            .write_volatile(aes_registers().command_status.get());
        let microcode_checksum = AES_MODE1_MICROCODE.iter().fold(0_u32, |value, byte| {
            value.rotate_left(5).wrapping_add(u32::from(*byte))
        });
        result.add(17).write_volatile(microcode_checksum);
        if initialize_hardware_aes_engine().is_err() {
            result.add(1).write_volatile(0xe005);
            result
                .add(8)
                .write_volatile(aes_registers().command_status.get());
            result
                .add(4)
                .write_volatile(crate::vendor_host_tx::vendor_timer_now().wrapping_sub(started));
            return;
        }
        result
            .add(8)
            .write_volatile(aes_registers().command_status.get());
        let operation = encrypt_tx_frame_hardware(
            frame,
            control,
            header,
            &HARDWARE_KAT_KEY,
            HARDWARE_KAT_PN,
            &aad,
            aad_length,
        );
        result
            .add(4)
            .write_volatile(crate::vendor_host_tx::vendor_timer_now().wrapping_sub(started));
        result
            .add(2)
            .write_volatile(HARDWARE_IRQ_MASK.0.get().read_volatile());
        result
            .add(3)
            .write_volatile(HARDWARE_COMPLETION_STATUS.0.get().read_volatile());
        result
            .add(18)
            .write_volatile(aes_registers().command_status.get());

        let mismatch = HARDWARE_KAT_EXPECTED
            .iter()
            .copied()
            .enumerate()
            .find(|(index, expected)| frame[*index] != *expected);
        match (operation, mismatch) {
            (Err(CcmpError::HardwareTimeout), _) => result.add(1).write_volatile(0xe001),
            (_, Some((index, expected))) => {
                result.add(1).write_volatile(0xe003);
                result.add(5).write_volatile(
                    ((index as u32) << 16) | (u32::from(expected) << 8) | u32::from(frame[index]),
                );
            }
            (Ok(()), None) => result.add(1).write_volatile(1),
            (Err(CcmpError::Authentication), None) => result.add(1).write_volatile(2),
            (Err(_), None) => result.add(1).write_volatile(0xe002),
        }
        let checksum = frame.iter().fold(0_u32, |value, byte| {
            value.rotate_left(5).wrapping_add(u32::from(*byte))
        });
        result.add(6).write_volatile(checksum);

        if result.add(1).read_volatile() == 1 {
            for (case, payload_length) in HARDWARE_KAT_MATRIX_LENGTHS.iter().copied().enumerate() {
                let frame_length = 26 + CCMP_HEADER_LEN + payload_length + CCMP_MIC_LEN;
                let frame = core::slice::from_raw_parts_mut(scratch as *mut u8, frame_length);
                frame[..34].copy_from_slice(&HARDWARE_KAT_INPUT[..34]);
                for (index, byte) in frame[34..34 + payload_length].iter_mut().enumerate() {
                    *byte = (index as u8).wrapping_mul(17).wrapping_add(3);
                }
                frame[34 + payload_length..].fill(0);
                let control = u16::from_le_bytes([frame[0], frame[1]]);
                let (aad, aad_length) = match build_aad(frame, control) {
                    Ok(value) => value,
                    Err(_) => {
                        result.add(1).write_volatile(0xe004);
                        result.add(20).write_volatile(payload_length as u32);
                        break;
                    }
                };
                let operation = encrypt_tx_frame_hardware(
                    frame,
                    control,
                    header_length(control),
                    &HARDWARE_KAT_KEY,
                    HARDWARE_KAT_PN,
                    &aad,
                    aad_length,
                );
                result.add(2).write_volatile(
                    result.add(2).read_volatile() | HARDWARE_IRQ_MASK.0.get().read_volatile(),
                );
                let actual_checksum = frame.iter().fold(0_u32, |value, byte| {
                    value.rotate_left(5).wrapping_add(u32::from(*byte))
                });
                let mic = &frame[34 + payload_length..];
                let expected_mic = &HARDWARE_KAT_MATRIX_MICS[case * 8..case * 8 + 8];
                if operation.is_err()
                    || actual_checksum != HARDWARE_KAT_MATRIX_CHECKSUMS[case]
                    || mic != expected_mic
                {
                    result.add(1).write_volatile(0xe006);
                    result
                        .add(5)
                        .write_volatile(HARDWARE_KAT_MATRIX_CHECKSUMS[case]);
                    result.add(6).write_volatile(actual_checksum);
                    result.add(20).write_volatile(payload_length as u32);
                    break;
                }
                result.add(19).write_volatile((case + 1) as u32);
            }
        }
        if result.add(1).read_volatile() == 1 {
            let frame =
                core::slice::from_raw_parts_mut(scratch as *mut u8, HARDWARE_KAT_EXPECTED.len());
            frame.copy_from_slice(&HARDWARE_KAT_EXPECTED);
            let control = u16::from_le_bytes([frame[0], frame[1]]);
            let header = header_length(control);
            let (aad, aad_length) = build_aad(frame, control).expect("fixed RX KAT AAD");
            let operation = decrypt_rx_frame_hardware(
                frame,
                control,
                header,
                &HARDWARE_KAT_KEY,
                HARDWARE_KAT_PN,
                &aad,
                aad_length,
            );
            result.add(2).write_volatile(
                result.add(2).read_volatile() | HARDWARE_IRQ_MASK.0.get().read_volatile(),
            );
            let payload_end = frame.len() - CCMP_MIC_LEN;
            if operation.is_err()
                || frame[header + CCMP_HEADER_LEN..payload_end]
                    != HARDWARE_KAT_INPUT[header + CCMP_HEADER_LEN..payload_end]
            {
                result.add(1).write_volatile(0xe007);
                result.add(20).write_volatile(1);
            } else {
                result.add(19).write_volatile(6);
                frame.copy_from_slice(&HARDWARE_KAT_EXPECTED);
                let last = frame.len() - 1;
                frame[last] ^= 1;
                let (aad, aad_length) = build_aad(frame, control).expect("fixed RX KAT AAD");
                if !matches!(
                    decrypt_rx_frame_hardware(
                        frame,
                        control,
                        header,
                        &HARDWARE_KAT_KEY,
                        HARDWARE_KAT_PN,
                        &aad,
                        aad_length,
                    ),
                    Err(CcmpError::Authentication)
                ) {
                    result.add(1).write_volatile(0xe007);
                    result.add(20).write_volatile(2);
                } else {
                    result.add(19).write_volatile(7);
                }
            }
        }
        result
            .add(3)
            .write_volatile(HARDWARE_COMPLETION_STATUS.0.get().read_volatile());
        result
            .add(18)
            .write_volatile(aes_registers().command_status.get());
        result
            .add(4)
            .write_volatile(crate::vendor_host_tx::vendor_timer_now().wrapping_sub(started));
        for index in 0..(26 + CCMP_HEADER_LEN + 1506 + CCMP_MIC_LEN) {
            (scratch as *mut u8).add(index).write_volatile(0);
        }
    }
}

pub fn hardware_ccmp_selftest_snapshot() -> [u32; 22] {
    let mut values = [0; 22];
    unsafe {
        let source = HARDWARE_SELFTEST.0.get().cast::<u32>();
        for (index, value) in values.iter_mut().enumerate() {
            *value = source.add(index).read_volatile();
        }
    }
    values
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
    #[cfg(target_arch = "arm")]
    {
        encrypt_tx_frame_hardware(frame, control, header, &key.key, pn, &aad, aad_length)?;
        return Ok(());
    }
    #[cfg(not(target_arch = "arm"))]
    {
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
    let group_addressed = frame[4] & 1 != 0;
    let key = rx_key(if_id, &transmitter, key_id, group_addressed)
        .ok_or(CcmpError::MissingKey)?;
    let (aad, aad_length) = build_aad(frame, control)?;
    #[cfg(target_arch = "arm")]
    {
        decrypt_rx_frame_hardware(frame, control, header, &key.key, pn, &aad, aad_length)?;
        return Ok(true);
    }
    #[cfg(not(target_arch = "arm"))]
    {
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
    fn tx_packet_number_decodes_ccmp_wire_order() {
        let mut frame = [0_u8; 34];
        frame[..2].copy_from_slice(&0x4088_u16.to_le_bytes());
        frame[26..34].copy_from_slice(&[0x8c, 0x31, 0, 0x20, 0x67, 0x45, 0x23, 0x01]);
        assert_eq!(tx_frame_packet_number(&frame), Some(0x0123_4567_318c));
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

    // Seal a From-DS frame with the specified key, independently of TX key lookup.
    fn sealed_rx_frame(receiver: [u8; 6], key_id: u8, key: &[u8; 16]) -> [u8; 52] {
        let mut frame = [0; 52];
        frame[..2].copy_from_slice(&0x4208_u16.to_le_bytes());
        frame[4..10].copy_from_slice(&receiver);
        frame[10..16].copy_from_slice(&[0x02; 6]);
        frame[16..22].copy_from_slice(&[0x04; 6]);
        frame[24..32].copy_from_slice(&[1, 0, 0, 0x20 | (key_id << 6), 0, 0, 0, 0]);
        frame[32..44].copy_from_slice(b"hello world!");
        let pn = [0, 0, 0, 0, 0, 1];
        let nonce = build_nonce(&frame, 0x4208, pn).unwrap();
        let (aad, aad_length) = build_aad(&frame, 0x4208).unwrap();
        let tag = AesCcmp::new_from_slice(key)
            .unwrap()
            .encrypt_in_place_detached(
                GenericArray::from_slice(&nonce),
                &aad[..aad_length],
                &mut frame[32..44],
            )
            .unwrap();
        frame[44..].copy_from_slice(&tag);
        frame
    }

    #[test]
    fn rx_key_class_is_selected_by_receiver_not_table_order() {
        let _guard = TEST_LOCK.lock().unwrap();
        let pairwise = KeyRecord {
            active: true,
            key_type: AES_PAIRWISE,
            peer: [0x02; 6],
            key: [0x11; 16],
            ..KeyRecord::EMPTY
        };
        for group_id in 0..4 {
            let group = KeyRecord {
                active: true,
                key_type: AES_GROUP,
                key_id: group_id,
                key: [0x22; 16],
                ..KeyRecord::EMPTY
            };
            for records in [[pairwise, group], [group, pairwise]] {
                unsafe {
                    *KEYS.0.get() = [KeyRecord::EMPTY; MAX_KEYS];
                    (&mut *KEYS.0.get())[..2].copy_from_slice(&records);
                }
                for (receiver, key_id, key) in [
                    ([0x06; 6], 0, pairwise.key),
                    ([0xff; 6], group_id, group.key),
                    ([0x01, 0, 0x5e, 0, 0, 1], group_id, group.key),
                ] {
                    let mut frame = sealed_rx_frame(receiver, key_id, &key);
                    assert_eq!(decrypt_rx_frame(&mut frame, 0), Ok(true));
                    assert_eq!(&frame[32..44], b"hello world!");
                }
            }
        }
    }

    #[test]
    fn rx_does_not_fall_back_to_the_other_key_class() {
        let _guard = TEST_LOCK.lock().unwrap();
        for (key_type, receiver) in [(AES_GROUP, [0x06; 6]), (AES_PAIRWISE, [0xff; 6])] {
            unsafe {
                *KEYS.0.get() = [KeyRecord::EMPTY; MAX_KEYS];
                (*KEYS.0.get())[0] = KeyRecord {
                    active: true,
                    key_type,
                    peer: [0x02; 6],
                    key: [0x11; 16],
                    ..KeyRecord::EMPTY
                };
            }
            let mut frame = sealed_rx_frame(receiver, 0, &[0x11; 16]);
            assert_eq!(decrypt_rx_frame(&mut frame, 0), Err(CcmpError::MissingKey));
        }
    }

    #[test]
    fn build_generated_hardware_kat_matches_independent_vector() {
        assert_eq!(
            &hardware_kat::HARDWARE_KAT_EXPECTED[34..50],
            &[
                0xc7, 0x40, 0x95, 0x9b, 0xf5, 0x9a, 0x96, 0x31, 0xa8, 0xc2, 0xc9, 0xd9, 0x91, 0x0d,
                0x11, 0x4b,
            ]
        );
        assert_eq!(
            &hardware_kat::HARDWARE_KAT_EXPECTED[50..58],
            &[0x11, 0x85, 0x85, 0x5b, 0xce, 0x30, 0xf3, 0xe9]
        );
        assert_eq!(
            hardware_kat::HARDWARE_KAT_MATRIX_LENGTHS,
            [1, 15, 16, 17, 1506]
        );
        assert_eq!(
            hardware_kat::HARDWARE_KAT_MATRIX_MICS.len(),
            5 * CCMP_MIC_LEN
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
