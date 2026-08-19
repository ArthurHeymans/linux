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

#[repr(C, align(4))]
struct OpaqueStorage<const N: usize>(UnsafeCell<MaybeUninit<[u8; N]>>);

unsafe impl<const N: usize> Sync for OpaqueStorage<N> {}

impl<const N: usize> OpaqueStorage<N> {
    const fn uninit() -> Self {
        Self(UnsafeCell::new(MaybeUninit::uninit()))
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
    ".packet_ram.host_frame_states",
    HOST_FRAME_STATE_COUNT * HOST_FRAME_STATE_SIZE
);
packet_object!(
    RESPONSE_POINTERS,
    ".packet_ram.response_pointers",
    RESPONSE_POINTER_COUNT * 4
);
packet_object!(
    TX_COMMANDS,
    ".packet_ram.tx_commands",
    TX_PIPE_COUNT * TX_COMMANDS_PER_PIPE * TX_COMMAND_SIZE
);
packet_object!(
    RATE_RAM,
    ".packet_ram.rate_ram",
    RATE_ENTRY_COUNT * RATE_ENTRY_SIZE
);
packet_object!(
    DURATION_WORDS,
    ".packet_ram.duration_words",
    DURATION_WORD_COUNT * 2
);
packet_object!(
    RESPONSE_COMMANDS,
    ".packet_ram.response_commands",
    RESPONSE_COMMAND_COUNT * TX_COMMAND_SIZE
);
packet_object!(
    INTERFACE_METADATA,
    ".packet_ram.interface_metadata",
    INTERFACE_METADATA_SIZE
);
packet_object!(
    HIF_INPUTS,
    ".packet_ram.hif_inputs",
    HIF_INPUT_COUNT * HIF_INPUT_SIZE
);
packet_object!(
    HIF_OUTPUTS,
    ".packet_ram.hif_outputs",
    HIF_OUTPUT_COUNT * HIF_OUTPUT_SIZE
);
packet_object!(
    INTERNAL_TX_BUFFERS,
    ".packet_ram.internal_tx_buffers",
    INTERNAL_TX_BUFFER_COUNT * INTERNAL_TX_BUFFER_SIZE
);
packet_object!(
    SOFTWARE_RECORDS,
    ".packet_ram.software_records",
    SOFTWARE_RECORD_COUNT * SOFTWARE_RECORD_SIZE
);
packet_object!(
    AUTOMATIC_RESPONSE_LIST,
    ".packet_ram.automatic_response_list",
    AUTOMATIC_RESPONSE_LIST_SIZE
);
packet_object!(
    RX_FIFO_BACKING,
    ".packet_ram.rx_fifo_backing",
    RX_FIFO_STORAGE_SIZE
);

#[cfg(target_arch = "arm")]
unsafe extern "C" {
    static __packet_ram_internal_tx_buffers_end: u8;
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
    #[cfg(target_arch = "arm")]
    {
        addr_of!(__packet_ram_internal_tx_buffers_end) as usize
    }
    #[cfg(not(target_arch = "arm"))]
    {
        internal_tx_buffers().end
    }
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
pub fn contains_owned_storage(address: usize) -> bool {
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
        || rx_fifo_backing().contains(&address)
}
