//! Linker-owned XR819 packet-RAM objects.
//!
//! These objects are opaque hardware storage. Code may derive device addresses
//! from them, but must not create Rust references while the HIF, MAC, crypto,
//! or packet-DMA hardware can own the corresponding bytes.

use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::ops::Range;
#[cfg(target_arch = "arm")]
use core::ptr::addr_of;

pub const HOST_FRAME_STATE_COUNT: usize = 30;
pub const HOST_FRAME_STATE_SIZE: usize = 0x54;
pub const RESPONSE_POINTER_COUNT: usize = 32;
pub const TX_PIPE_COUNT: usize = 4;
pub const TX_COMMANDS_PER_PIPE: usize = 4;
pub const TX_COMMAND_SIZE: usize = 0x54;
pub const RATE_ENTRY_COUNT: usize = 80;
pub const RATE_ENTRY_SIZE: usize = 0x10;
pub const DURATION_WORD_COUNT: usize = 2;
pub const RESPONSE_COMMAND_COUNT: usize = 13;
pub const INTERFACE_METADATA_SIZE: usize = 4;
pub const HIF_INPUT_COUNT: usize = 30;
pub const HIF_INPUT_SIZE: usize = 0x660;
pub const HIF_OUTPUT_COUNT: usize = 4;
pub const HIF_OUTPUT_SIZE: usize = 0x180;
pub const INTERNAL_TX_BUFFER_COUNT: usize = 3;
pub const INTERNAL_TX_BUFFER_SIZE: usize = 0x400;
pub const SOFTWARE_RECORD_COUNT: usize = 4;
pub const SOFTWARE_RECORD_SIZE: usize = 0x2a0;
pub const AUTOMATIC_RESPONSE_LIST_SIZE: usize = 0x8c;
pub const RX_FIFO_LOGICAL_SIZE: usize = 0x7000;
pub const RX_FIFO_STORAGE_SIZE: usize = 0x8000;

/// CPU-visible physical extent of the linker-owned runtime packet-RAM pool.
/// Hardware command words use narrower bus encodings and must never be treated
/// as values in this range.
pub const RUNTIME_CPU_START: u32 = 0x0900_7000;
pub const RUNTIME_CPU_END: u32 = RUNTIME_CPU_START + 0x0000_f630;
pub const RX_FIFO_CPU_START: u32 = 0x0940_0000;
pub const RX_FIFO_CPU_END: u32 = RX_FIFO_CPU_START + RX_FIFO_STORAGE_SIZE as u32;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuntimePacketAddress(u32);

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MacPacketOffset(u32);

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TxPayloadBusAddress(u32);

impl RuntimePacketAddress {
    pub fn new(address: u32, length: usize) -> Option<Self> {
        let address_usize = address as usize;
        address_usize.checked_add(length)?;
        #[cfg(not(target_arch = "arm"))]
        let in_physical_runtime = address_usize >= RUNTIME_CPU_START as usize
            && address_usize + length <= RUNTIME_CPU_END as usize;
        #[cfg(target_arch = "arm")]
        let owned = contains_runtime_range(address_usize, length);
        #[cfg(not(target_arch = "arm"))]
        let owned = in_physical_runtime || contains_runtime_range(address_usize, length);

        (length != 0 && owned).then_some(Self(address))
    }

    #[inline(always)]
    pub const fn raw(self) -> u32 { self.0 }

    #[inline(always)]
    pub const fn mac_offset(self) -> MacPacketOffset {
        MacPacketOffset(encode_mac_packet_offset_u32(self.0))
    }

    #[inline(always)]
    pub const fn tx_payload_bus_address(self, address_mask: u32) -> TxPayloadBusAddress {
        TxPayloadBusAddress(encode_tx_payload_bus_address(self.0, address_mask))
    }
}

impl MacPacketOffset {
    #[inline(always)]
    pub const fn raw(self) -> u32 { self.0 }
}

impl TxPayloadBusAddress {
    #[inline(always)]
    pub const fn raw(self) -> u32 { self.0 }
}

#[repr(C, align(4))]
struct OpaqueStorage<const N: usize>(UnsafeCell<MaybeUninit<[u8; N]>>);

unsafe impl<const N: usize> Sync for OpaqueStorage<N> {}

impl<const N: usize> OpaqueStorage<N> {
    const fn uninit() -> Self {
        Self(UnsafeCell::new(MaybeUninit::uninit()))
    }
}

#[repr(C, align(4))]
struct RateRam {
    entries: OpaqueStorage<{ RATE_ENTRY_COUNT * RATE_ENTRY_SIZE }>,
    // The MAC response family starts 0x100 bytes after the final known rate
    // entry. Keep that retained hardware spacing as part of the Rust layout.
    _retained_tail: OpaqueStorage<0x100>,
}

impl RateRam {
    const fn uninit() -> Self {
        Self {
            entries: OpaqueStorage::uninit(),
            _retained_tail: OpaqueStorage::uninit(),
        }
    }
}

macro_rules! packet_object {
    ($name:ident, $section:literal, $size:expr) => {
        #[cfg(all(target_arch = "arm", target_feature = "thumb-mode"))]
        #[used]
        #[unsafe(link_section = $section)]
        static $name: OpaqueStorage<{ $size }> = OpaqueStorage::uninit();

        #[cfg(not(all(target_arch = "arm", target_feature = "thumb-mode")))]
        static $name: OpaqueStorage<{ $size }> = OpaqueStorage::uninit();
    };
}

packet_object!(
    HOST_FRAME_STATES,
    ".packet_ram.runtime.120_host_frame_states",
    HOST_FRAME_STATE_COUNT * HOST_FRAME_STATE_SIZE
);
packet_object!(
    RESPONSE_POINTERS,
    ".packet_ram.runtime.010_response_pointers",
    RESPONSE_POINTER_COUNT * 4
);
packet_object!(
    TX_COMMANDS,
    ".packet_ram.runtime.020_tx_commands",
    TX_PIPE_COUNT * TX_COMMANDS_PER_PIPE * TX_COMMAND_SIZE
);
#[cfg(all(target_arch = "arm", target_feature = "thumb-mode"))]
#[used]
#[unsafe(link_section = ".packet_ram.runtime.030_rate_ram")]
static RATE_RAM: RateRam = RateRam::uninit();

#[cfg(not(all(target_arch = "arm", target_feature = "thumb-mode")))]
static RATE_RAM: RateRam = RateRam::uninit();
packet_object!(
    DURATION_WORDS,
    ".packet_ram.runtime.040_duration_words",
    DURATION_WORD_COUNT * 2
);
packet_object!(
    RESPONSE_COMMANDS,
    ".packet_ram.runtime.050_response_commands",
    RESPONSE_COMMAND_COUNT * TX_COMMAND_SIZE
);
packet_object!(
    INTERFACE_METADATA,
    ".packet_ram.runtime.060_interface_metadata",
    INTERFACE_METADATA_SIZE
);
packet_object!(
    HIF_INPUTS,
    ".packet_ram.runtime.070_hif_inputs",
    HIF_INPUT_COUNT * HIF_INPUT_SIZE
);
packet_object!(
    HIF_OUTPUTS,
    ".packet_ram.runtime.080_hif_outputs",
    HIF_OUTPUT_COUNT * HIF_OUTPUT_SIZE
);
packet_object!(
    INTERNAL_TX_BUFFERS,
    ".packet_ram.runtime.090_internal_tx_buffers",
    INTERNAL_TX_BUFFER_COUNT * INTERNAL_TX_BUFFER_SIZE
);
packet_object!(
    SOFTWARE_RECORDS,
    ".packet_ram.runtime.100_software_records",
    SOFTWARE_RECORD_COUNT * SOFTWARE_RECORD_SIZE
);
packet_object!(
    AUTOMATIC_RESPONSE_LIST,
    ".packet_ram.runtime.110_automatic_response_list",
    AUTOMATIC_RESPONSE_LIST_SIZE
);
packet_object!(
    RX_FIFO_BACKING,
    ".packet_ram.rx_fifo_backing",
    RX_FIFO_STORAGE_SIZE
);

#[cfg(target_arch = "arm")]
unsafe extern "C" {
    static __packet_ram_lmc_anchor_0: u8;
    static __packet_ram_lmc_anchor_1: u8;
}

#[cfg(not(target_arch = "arm"))]
static HOST_LMC_ANCHORS: [u8; 2] = [0; 2];

macro_rules! object_address {
    ($object:ident) => {
        core::ptr::addr_of!($object) as usize
    };
}

#[inline(always)]
pub fn host_frame_states() -> Range<usize> {
    let start = object_address!(HOST_FRAME_STATES);
    start..start + HOST_FRAME_STATE_COUNT * HOST_FRAME_STATE_SIZE
}

#[inline(always)]
pub fn host_frame_state(index: usize) -> usize {
    debug_assert!(index < HOST_FRAME_STATE_COUNT);
    object_address!(HOST_FRAME_STATES) + index * HOST_FRAME_STATE_SIZE
}

#[inline(always)]
pub fn host_frame_state_index(address: usize) -> Option<usize> {
    let range = host_frame_states();
    let offset = address.checked_sub(range.start)?;
    (address < range.end && offset % HOST_FRAME_STATE_SIZE == 0)
        .then_some(offset / HOST_FRAME_STATE_SIZE)
}

#[inline(always)]
pub fn response_pointers() -> Range<usize> {
    let start = object_address!(RESPONSE_POINTERS);
    start..start + RESPONSE_POINTER_COUNT * 4
}

#[inline(always)]
pub fn response_pointer(index: usize) -> usize {
    debug_assert!(index < RESPONSE_POINTER_COUNT);
    object_address!(RESPONSE_POINTERS) + index * 4
}

#[inline(always)]
pub fn ampdu_spacing_word_address(selector: u8) -> usize {
    automatic_response_list()
        .end
        .wrapping_sub(4)
        .wrapping_sub(usize::from(selector) * 4)
}

pub fn tx_commands() -> Range<usize> {
    let start = object_address!(TX_COMMANDS);
    start..start + TX_PIPE_COUNT * TX_COMMANDS_PER_PIPE * TX_COMMAND_SIZE
}

#[inline(always)]
pub fn tx_command(pipe: usize, slot: usize) -> usize {
    debug_assert!(pipe < TX_PIPE_COUNT);
    debug_assert!(slot < TX_COMMANDS_PER_PIPE);
    object_address!(TX_COMMANDS) + (pipe * TX_COMMANDS_PER_PIPE + slot) * TX_COMMAND_SIZE
}

#[inline(always)]
pub fn tx_command_index(address: usize) -> Option<(usize, usize)> {
    let range = tx_commands();
    let offset = address.checked_sub(range.start)?;
    if address >= range.end || offset % TX_COMMAND_SIZE != 0 {
        return None;
    }
    let index = offset / TX_COMMAND_SIZE;
    Some((index / TX_COMMANDS_PER_PIPE, index % TX_COMMANDS_PER_PIPE))
}

#[inline(always)]
pub fn rate_ram() -> Range<usize> {
    let start = object_address!(RATE_RAM);
    start..start + RATE_ENTRY_COUNT * RATE_ENTRY_SIZE
}

#[inline(always)]
pub fn rate_entry(index: usize) -> usize {
    debug_assert!(index < RATE_ENTRY_COUNT);
    object_address!(RATE_RAM) + index * RATE_ENTRY_SIZE
}

#[inline(always)]
pub fn duration_words() -> Range<usize> {
    let start = object_address!(DURATION_WORDS);
    start..start + DURATION_WORD_COUNT * 2
}

#[inline(always)]
pub fn duration_word(index: usize) -> usize {
    debug_assert!(index < DURATION_WORD_COUNT);
    object_address!(DURATION_WORDS) + index * 2
}

#[inline(always)]
pub fn response_commands() -> Range<usize> {
    let start = object_address!(RESPONSE_COMMANDS);
    start..start + RESPONSE_COMMAND_COUNT * TX_COMMAND_SIZE
}

#[inline(always)]
pub fn response_command(index: usize) -> usize {
    debug_assert!(index < RESPONSE_COMMAND_COUNT);
    object_address!(RESPONSE_COMMANDS) + index * TX_COMMAND_SIZE
}

#[inline(always)]
pub fn response_command_index(address: usize) -> Option<usize> {
    let range = response_commands();
    let offset = address.checked_sub(range.start)?;
    (address < range.end && offset % TX_COMMAND_SIZE == 0)
        .then_some(offset / TX_COMMAND_SIZE)
}

#[inline(always)]
pub fn interface_metadata() -> usize {
    object_address!(INTERFACE_METADATA)
}

#[inline(always)]
pub fn interface_metadata_byte(index: usize) -> usize {
    debug_assert!(index < INTERFACE_METADATA_SIZE);
    interface_metadata() + index
}

#[inline(always)]
pub fn hif_inputs() -> Range<usize> {
    let start = object_address!(HIF_INPUTS);
    start..start + HIF_INPUT_COUNT * HIF_INPUT_SIZE
}

#[inline(always)]
pub fn hif_input(index: usize) -> usize {
    debug_assert!(index < HIF_INPUT_COUNT);
    object_address!(HIF_INPUTS) + index * HIF_INPUT_SIZE
}

#[inline(always)]
pub fn hif_outputs() -> Range<usize> {
    let start = object_address!(HIF_OUTPUTS);
    start..start + HIF_OUTPUT_COUNT * HIF_OUTPUT_SIZE
}

#[inline(always)]
pub fn hif_output(index: usize) -> usize {
    debug_assert!(index < HIF_OUTPUT_COUNT);
    object_address!(HIF_OUTPUTS) + index * HIF_OUTPUT_SIZE
}

#[inline(always)]
pub fn internal_tx_buffers() -> Range<usize> {
    let start = object_address!(INTERNAL_TX_BUFFERS);
    start..start + INTERNAL_TX_BUFFER_COUNT * INTERNAL_TX_BUFFER_SIZE
}

#[inline(always)]
pub fn internal_tx_buffer(index: usize) -> usize {
    debug_assert!(index < INTERNAL_TX_BUFFER_COUNT);
    object_address!(INTERNAL_TX_BUFFERS) + index * INTERNAL_TX_BUFFER_SIZE
}

#[inline(always)]
pub fn internal_tx_buffers_end() -> usize {
    internal_tx_buffers().end
}

#[inline(always)]
pub fn software_records() -> Range<usize> {
    let start = object_address!(SOFTWARE_RECORDS);
    start..start + SOFTWARE_RECORD_COUNT * SOFTWARE_RECORD_SIZE
}

#[inline(always)]
pub fn software_record(index: usize) -> usize {
    debug_assert!(index < SOFTWARE_RECORD_COUNT);
    object_address!(SOFTWARE_RECORDS) + index * SOFTWARE_RECORD_SIZE
}

#[inline(always)]
pub fn software_record_index(address: usize) -> Option<usize> {
    let range = software_records();
    let offset = address.checked_sub(range.start)?;
    (address < range.end && offset % SOFTWARE_RECORD_SIZE == 0)
        .then_some(offset / SOFTWARE_RECORD_SIZE)
}

/// Encode one aligned CPU-form runtime packet-RAM address for MAC opcode 0x65.
/// The resulting bus word is intentionally one-way: software ownership must
/// retain or recover the original CPU address from a typed software record.
#[inline(always)]
pub fn ampdu_transfer_word(cpu_address: usize) -> Option<u32> {
    #[cfg(target_arch = "arm")]
    let in_runtime = (RUNTIME_CPU_START as usize..RUNTIME_CPU_END as usize)
        .contains(&cpu_address);
    #[cfg(not(target_arch = "arm"))]
    let in_runtime = (RUNTIME_CPU_START as usize..RUNTIME_CPU_END as usize)
        .contains(&cpu_address)
        || contains_runtime_storage(cpu_address);

    (in_runtime && cpu_address & 3 == 0)
        .then_some(0x6500_0000 | (cpu_address as u32 & 0x001f_fffc))
}

/// Encode a CPU-form address already proven to identify packet RAM for a DMA
/// descriptor.
///
/// # Safety
///
/// `cpu_address` must be inside linker-owned runtime packet RAM or the RX FIFO.
#[inline(always)]
pub const unsafe fn packet_dma_bus_address_unchecked(cpu_address: usize) -> u32 {
    cpu_address as u32 & 0xf6ff_ffff
}

/// Encode a CPU-form address already proven to identify runtime packet RAM for
/// a MAC register or command field that carries a 23-bit packet-memory offset.
///
/// # Safety
///
/// `cpu_address` must be derived from a linker-owned runtime packet-RAM object.
#[inline(always)]
pub const unsafe fn mac_packet_offset_unchecked(cpu_address: usize) -> u32 {
    cpu_address as u32 & 0x007f_ffff
}

/// Encode a raw 32-bit value as a MAC packet offset. This preserves the pure
/// descriptor-builder API; publication boundaries must validate CPU identity.
#[inline(always)]
pub const fn encode_mac_packet_offset_u32(cpu_address: u32) -> u32 {
    cpu_address & 0x007f_ffff
}

/// Apply the MAC TX payload bus encoding and the caller's qualified alignment
/// mask. This is a pure descriptor encoding; publication retains and validates
/// the separate CPU-form context identity.
#[inline(always)]
pub const fn encode_tx_payload_bus_address(
    cpu_address: u32,
    address_mask: u32,
) -> u32 {
    address_mask & cpu_address & 0xf6ff_ffff
}

#[inline(always)]
pub fn response_command_bus_address(cpu_address: usize) -> Option<u32> {
    response_command_index(cpu_address)
        .map(|_| unsafe { packet_dma_bus_address_unchecked(cpu_address) })
}

/// Check and encode a CPU-form packet-RAM address for a DMA descriptor.
/// The bus value is intentionally not reversible; queue ownership retains the
/// CPU-form address until the descriptor is reclaimed.
pub fn packet_dma_bus_address(cpu_address: usize) -> Option<u32> {
    let in_physical_packet_ram =
        (RUNTIME_CPU_START as usize..RUNTIME_CPU_END as usize).contains(&cpu_address)
            || (RX_FIFO_CPU_START as usize..RX_FIFO_CPU_END as usize).contains(&cpu_address);
    #[cfg(target_arch = "arm")]
    let owned = contains_runtime_storage(cpu_address) || rx_fifo_backing().contains(&cpu_address);
    #[cfg(not(target_arch = "arm"))]
    let owned = in_physical_packet_ram
        || contains_runtime_storage(cpu_address)
        || rx_fifo_backing().contains(&cpu_address);

    owned.then(|| unsafe { packet_dma_bus_address_unchecked(cpu_address) })
}

#[inline(always)]
pub fn automatic_response_list() -> Range<usize> {
    let start = object_address!(AUTOMATIC_RESPONSE_LIST);
    start..start + AUTOMATIC_RESPONSE_LIST_SIZE
}

#[inline(always)]
pub fn lmc_anchor(index: usize) -> usize {
    #[cfg(target_arch = "arm")]
    {
        match index {
            0 => addr_of!(__packet_ram_lmc_anchor_0) as usize,
            1 => addr_of!(__packet_ram_lmc_anchor_1) as usize,
            _ => panic!("invalid LMC anchor"),
        }
    }
    #[cfg(not(target_arch = "arm"))]
    {
        HOST_LMC_ANCHORS
            .get(index)
            .map(|anchor| anchor as *const u8 as usize)
            .expect("invalid LMC anchor")
    }
}

#[inline(always)]
pub fn rx_fifo_backing() -> Range<usize> {
    let start = object_address!(RX_FIFO_BACKING);
    start..start + RX_FIFO_STORAGE_SIZE
}

#[inline(always)]
pub fn rx_fifo_base() -> usize {
    object_address!(RX_FIFO_BACKING)
}

/// Whether an address belongs to a qualified linker-owned runtime object.
/// Unknown packet-RAM holes and all boot/diagnostic overlays return false.
#[inline(always)]
pub fn contains_runtime_storage(address: usize) -> bool {
    host_frame_states().contains(&address)
        || response_pointers().contains(&address)
        || tx_commands().contains(&address)
        || rate_ram().contains(&address)
        || duration_words().contains(&address)
        || response_commands().contains(&address)
        || (interface_metadata()..interface_metadata() + INTERFACE_METADATA_SIZE).contains(&address)
        || hif_inputs().contains(&address)
        || hif_outputs().contains(&address)
        || internal_tx_buffers().contains(&address)
        || software_records().contains(&address)
        || automatic_response_list().contains(&address)
}

#[inline(always)]
pub fn contains_runtime_range(address: usize, length: usize) -> bool {
    let Some(end) = address.checked_add(length) else { return false };
    let contains = |range: Range<usize>| address >= range.start && end <= range.end;
    contains(host_frame_states())
        || contains(response_pointers())
        || contains(tx_commands())
        || contains(rate_ram())
        || contains(duration_words())
        || contains(response_commands())
        || contains(interface_metadata()..interface_metadata() + INTERFACE_METADATA_SIZE)
        || contains(hif_inputs())
        || contains(hif_outputs())
        || contains(internal_tx_buffers())
        || contains(software_records())
        || contains(automatic_response_list())
}

#[inline(always)]
pub fn contains_owned_range(address: usize, length: usize) -> bool {
    contains_runtime_range(address, length) || {
        let Some(end) = address.checked_add(length) else { return false };
        let range = rx_fifo_backing();
        address >= range.start && end <= range.end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tx_command_addresses_require_exact_pipe_slot_bases() {
        for pipe in 0..TX_PIPE_COUNT {
            for slot in 0..TX_COMMANDS_PER_PIPE {
                let address = tx_command(pipe, slot);
                assert_eq!(tx_command_index(address), Some((pipe, slot)));
                assert_eq!(tx_command_index(address + 4), None);
            }
        }
        assert_eq!(tx_command_index(tx_commands().end), None);
    }

    #[test]
    fn software_record_addresses_require_exact_record_bases() {
        for index in 0..SOFTWARE_RECORD_COUNT {
            assert_eq!(software_record_index(software_record(index)), Some(index));
            assert_eq!(software_record_index(software_record(index) + 4), None);
        }
        assert_eq!(software_record_index(software_records().end), None);
        assert_eq!(
            software_record_index(software_record(0) & 0x001f_fffc),
            None,
        );
    }

    #[test]
    fn response_command_addresses_require_exact_bases_before_bus_encoding() {
        for index in 0..RESPONSE_COMMAND_COUNT {
            let address = response_command(index);
            assert_eq!(response_command_index(address), Some(index));
            assert_eq!(
                response_command_bus_address(address),
                Some(unsafe { packet_dma_bus_address_unchecked(address) }),
            );
            assert_eq!(response_command_index(address + 4), None);
            assert_eq!(response_command_bus_address(address + 4), None);
        }
        assert_eq!(response_command_index(response_commands().end), None);
        assert_eq!(
            response_command_bus_address(response_command(0) & 0xf6ff_ffff),
            None,
        );
    }

    #[test]
    fn packet_dma_encoding_accepts_cpu_packet_ram_but_rejects_bus_aliases() {
        assert_eq!(
            packet_dma_bus_address(RUNTIME_CPU_START as usize),
            Some(RUNTIME_CPU_START & 0xf6ff_ffff),
        );
        assert_eq!(
            packet_dma_bus_address((RX_FIFO_CPU_END - 1) as usize),
            Some((RX_FIFO_CPU_END - 1) & 0xf6ff_ffff),
        );
        assert_eq!(packet_dma_bus_address(RUNTIME_CPU_END as usize), None);
        assert_eq!(
            packet_dma_bus_address((RUNTIME_CPU_START & 0xf6ff_ffff) as usize),
            None,
        );
        assert!(packet_dma_bus_address(hif_output(0)).is_some());
        assert!(packet_dma_bus_address(rx_fifo_base()).is_some());
    }

    #[test]
    fn owned_range_checks_reject_cross_object_and_overflowing_reads() {
        assert!(contains_owned_range(tx_command(0, 0), TX_COMMAND_SIZE));
        assert!(!contains_owned_range(tx_commands().end - 2, 4));
        assert!(!contains_owned_range(usize::MAX - 1, 4));
    }

    #[test]
    fn typed_runtime_addresses_reject_aliases_ends_and_non_runtime_storage() {
        let address = RuntimePacketAddress::new(RUNTIME_CPU_START, 4).unwrap();
        assert_eq!(address.raw(), RUNTIME_CPU_START);
        assert_eq!(address.mac_offset().raw(), RUNTIME_CPU_START & 0x007f_ffff);
        assert_eq!(
            address.tx_payload_bus_address(0x007f_fffc).raw(),
            RUNTIME_CPU_START & 0x007f_fffc,
        );
        assert!(RuntimePacketAddress::new(RUNTIME_CPU_END - 4, 4).is_some());
        assert!(RuntimePacketAddress::new(RUNTIME_CPU_END - 2, 4).is_none());
        assert!(RuntimePacketAddress::new(RUNTIME_CPU_START, 0).is_none());
        assert!(RuntimePacketAddress::new(RUNTIME_CPU_START & 0x007f_ffff, 4).is_none());
        assert!(RuntimePacketAddress::new(RX_FIFO_CPU_START, 4).is_none());
    }

    #[test]
    fn ampdu_transfer_encoding_accepts_only_aligned_cpu_runtime_addresses() {
        assert_eq!(
            ampdu_transfer_word(RUNTIME_CPU_START as usize),
            Some(0x6500_0000 | (RUNTIME_CPU_START & 0x001f_fffc)),
        );
        assert_eq!(
            ampdu_transfer_word((RUNTIME_CPU_END - 4) as usize),
            Some(0x6500_0000 | ((RUNTIME_CPU_END - 4) & 0x001f_fffc)),
        );
        assert_eq!(ampdu_transfer_word((RUNTIME_CPU_START + 2) as usize), None);
        assert_eq!(ampdu_transfer_word(RUNTIME_CPU_END as usize), None);
        assert_eq!(ampdu_transfer_word((RUNTIME_CPU_START & 0x001f_fffc) as usize), None);
        assert!(ampdu_transfer_word(automatic_response_list().start).is_some());
    }
}
