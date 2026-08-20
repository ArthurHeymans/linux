//! Linker-owned view of the retained XR819 DTCM software-state ABI.
//!
//! This module describes the complete `0x0400_0000..0x0400_a000` quarantine
//! as one Rust layout without claiming exclusive Rust ownership. Vendor code,
//! IRQ/FIQ paths, and translated Rust can all mutate bytes in this range. The
//! backing symbol is therefore `MaybeUninit` inside `UnsafeCell`, fields expose
//! no safe references, and all consumers must use raw pointers with volatile
//! operations or narrower family APIs.
//!
//! Unknown bytes are occupied ABI bytes, not padding available for allocation.
//! The names below record the strongest presently supported family boundary;
//! private opaque storage deliberately makes no claim about undecoded fields.

use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::ptr::addr_of_mut;

pub const DTCM_STATE_BASE: usize = 0x0400_0000;
pub const DTCM_STATE_SIZE: usize = 0x0000_a000;
pub const DTCM_STATE_END: usize = DTCM_STATE_BASE + DTCM_STATE_SIZE;

pub const INTERNAL_TX_CONTEXT_SIZE: usize = 0x170;
pub const INTERNAL_TX_CONTEXT_COUNT: usize = 3;
pub const HOST_TX_CONTEXT_SIZE: usize = 0x170;
pub const HOST_TX_CONTEXT_COUNT: usize = 30;
pub const VIF_RECORD_SIZE: usize = 0x3b0;
pub const VIF_RECORD_COUNT: usize = 3;
/// Stride observed in per-interface power-save address calculations.
///
/// This is an address/view aid only. It does not prove that the family contains
/// two disjoint records: larger relative accesses leave its extent and overlap
/// semantics unresolved.
pub const POWER_SAVE_OBSERVED_STRIDE: usize = 0x104;
pub const LMC_MESSAGE_SIZE: usize = 0x2c;
pub const LMC_MESSAGE_COUNT: usize = 16;

/// A checked address in the linker-owned DTCM state quarantine.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DtcmAddress(u32);

impl DtcmAddress {
    pub const fn new(address: usize) -> Option<Self> {
        if address >= DTCM_STATE_BASE && address < DTCM_STATE_END {
            Some(Self(address as u32))
        } else {
            None
        }
    }

    pub const fn from_offset(offset: usize) -> Self {
        assert!(offset < DTCM_STATE_SIZE);
        Self((DTCM_STATE_BASE + offset) as u32)
    }

    pub const fn get(self) -> usize {
        self.0 as usize
    }

    pub const fn offset(self) -> usize {
        self.get() - DTCM_STATE_BASE
    }

    pub fn cast_mut<T>(self) -> *mut T {
        self.get() as *mut T
    }
}

/// Opaque storage with no safe byte-slice API.
#[repr(transparent)]
struct OpaqueBytes<const N: usize> {
    bytes: UnsafeCell<MaybeUninit<[u8; N]>>,
}

/// A shared scalar whose representation is known but whose ownership is not.
#[repr(transparent)]
struct SharedScalar<T> {
    value: UnsafeCell<MaybeUninit<T>>,
}

type SharedU8 = SharedScalar<u8>;
type SharedU16 = SharedScalar<u16>;
type SharedU32 = SharedScalar<u32>;

macro_rules! opaque_family {
    ($(#[$meta:meta])* $name:ident, $size:expr) => {
        $(#[$meta])*
        #[repr(C, align(4))]
        struct $name {
            storage: OpaqueBytes<$size>,
        }
    };
}

/// Vendor COPY image with exact initialized-data islands represented at their
/// qualified offsets. Bytes between islands remain occupied opaque data.
#[repr(C, align(4))]
struct InitializedVendorImage {
    pre_duration_tables: OpaqueBytes<0x138>,
    tx_duration_timing: [SharedU16; 10],
    pre_rate_tables: OpaqueBytes<0x48>,
    rate_encoding: OpaqueBytes<0x16>,
    rate_attributes: OpaqueBytes<0x16>,
    pre_completion_callback_words: OpaqueBytes<0xa0>,
    /// Ten visible words in the qualified initialized island. The evidence does
    /// not establish that every word is a complete callable entry.
    visible_completion_words: [SharedU32; 10],
    pre_ring_cursor_map: OpaqueBytes<0x50>,
    ring_cursor_map: SharedU32,
    pipe_status_maps: OpaqueBytes<0x8>,
    pre_command_dispatch: OpaqueBytes<0x42c>,
    command_dispatch: [SharedU32; 37],
    pre_aes_descriptors: OpaqueBytes<0x60>,
    aes_transfer_descriptors: OpaqueBytes<0x2c>,
    aes_mode1_microcode: OpaqueBytes<0x1ae>,
    pre_duration_quantum_pointers: OpaqueBytes<0x6f6>,
    duration_quantum_pointers: [SharedU32; 4],
    pre_hif_control_shadow: OpaqueBytes<0xc8>,
    hif_control_shadow: OpaqueBytes<0x8>,
    pre_irq_callbacks: OpaqueBytes<0x8>,
    irq_callbacks: [SharedU32; 32],
    pre_ampdu_counters: OpaqueBytes<0x64>,
    ampdu_counters: OpaqueBytes<0x28>,
    pre_control_words: OpaqueBytes<0x158>,
    control_words: OpaqueBytes<0x20>,
    pre_low_mac_root: OpaqueBytes<0x240>,
    initialized_low_mac_state: OpaqueBytes<0x94c>,
    scheduler_exclusion_words: [SharedU32; 2],
    scheduler_event_island: OpaqueBytes<0x44>,
    initialized_tail: OpaqueBytes<0x60>,
}
opaque_family!(
    /// Register-context, backoff, diagnostic, and other early zeroed state.
    RuntimePrefix,
    0x114
);
opaque_family!(
    /// Exact clock-parameter island. Individual fields remain vendor-shared.
    ClockParameterIsland,
    0x28
);

/// Reverse-indexed scheduler/IRQ callback words. Values are quarantined raw
/// addresses because not every callback target has native Rust ownership.
#[repr(C, align(4))]
struct SchedulerHandlerTable {
    handlers: [SharedU32; 32],
}

opaque_family!(
    /// PHY/template/beacon/filter state with only scattered decoded islands.
    PreConfigurationTables,
    0x127c
);
opaque_family!(
    /// SDD-derived channel, gain, and profile tables populated by startup.
    SddConfigurationTables,
    0x130
);
opaque_family!(
    /// Wake/context state whose exact field partition is not yet decoded.
    WakeContextState,
    0x90
);
opaque_family!(
    /// Two retained duration-source halfwords.
    DurationSources,
    0x4
);
opaque_family!(
    /// Undecoded occupied word before the low-MAC/PAS root.
    PreLowMacWord,
    0x4
);
opaque_family!(
    /// Shared low-MAC, PAS, rate, link, pipe, queue, and retry family. The
    /// `0x800` family extent is known, but internal overlays are not.
    LowMacPasFamily,
    0x800
);
opaque_family!(
    /// Link/aggregation header immediately preceding the VIF array.
    PreVifHeader,
    0x20
);

#[repr(C, align(4))]
struct VifRecord {
    /// Exact `0x3b0` stride; embedded timers and vendor writers prohibit safe
    /// field references even for decoded offsets.
    storage: OpaqueBytes<VIF_RECORD_SIZE>,
}

#[repr(C, align(4))]
struct VifRecords {
    records: [VifRecord; VIF_RECORD_COUNT],
}

opaque_family!(
    /// Occupied VIF-adjacent tables with no proven internal allocation units.
    PostVifQuarantine,
    0x107c
);

#[repr(C, align(4))]
struct HostTxContext {
    /// Exact host-context stride. Intrusive links, packet-RAM pointers, timers,
    /// and overlaid metadata remain accessible only through volatile raw views.
    storage: OpaqueBytes<HOST_TX_CONTEXT_SIZE>,
}

#[repr(C, align(4))]
struct HostTxContexts {
    contexts: [HostTxContext; HOST_TX_CONTEXT_COUNT],
}

opaque_family!(
    /// Occupied bytes between host contexts and the command upload buffer.
    PreCommandQuarantine,
    0x50
);

/// Deliberately opaque because the `0x68`-byte command-15 upload at `+0x00`
/// and the `0x20`-byte channel-switch control view at `+0x64` overlap by four
/// bytes. Two ordinary fields would falsely claim simultaneous ownership.
#[repr(C, align(4))]
struct CommandChannelSwitchOverlay {
    storage: OpaqueBytes<0x84>,
}

opaque_family!(
    /// LMC/encryption/free-list roots and mixed control state.
    LmcControlRoots,
    0x180
);
opaque_family!(
    /// Host-context and duplicate-cache accounting.
    HostContextAccounting,
    0x18
);

/// Host TX free-list root plus one adjacent unresolved word.
#[repr(C, align(4))]
struct HostContextFreeList {
    free_head: SharedU32,
    adjacent_state: SharedU32,
}

opaque_family!(
    /// Link state, link-map entries, and per-link/TID sequence storage.
    LinkAndSequenceState,
    0x220
);
opaque_family!(
    /// JOIN/scan timer and control objects.
    JoinScanControl,
    0x40
);
opaque_family!(
    /// WSM response, scan, indication scratch, and related control.
    WsmResponseScratch,
    0xa0
);
opaque_family!(
    /// BA/LMC global header.
    BaLmcHeader,
    0x20
);
opaque_family!(
    /// Shared pending-list, BA, LMC, scheduler, and radio state.
    PendingBaLmcState,
    0xe0
);

#[repr(C, align(4))]
struct LmcMessage {
    storage: OpaqueBytes<LMC_MESSAGE_SIZE>,
}

#[repr(C, align(4))]
struct LmcMessages {
    records: [LmcMessage; LMC_MESSAGE_COUNT],
}

opaque_family!(
    /// Occupied undecoded bytes after the LMC message records.
    PostLmcQuarantine,
    0xa0
);
opaque_family!(
    /// BA/link/event/timer state immediately before TALA.
    BaLinkEventState,
    0x30
);

/// Exact qualified TALA accounting shape. The semantic names describe the
/// translated algorithm, not exclusive ownership of these volatile words.
#[repr(C, align(4))]
struct TalaAccounting {
    growth_streaks: [SharedU8; 2],
    reserved_02: OpaqueBytes<2>,
    successes: [SharedU32; 2],
    failures: [SharedU32; 2],
    cumulative_tries: [SharedU32; 2],
    weighted_penalties: [SharedU32; 2],
}

/// Completion/context accounting anchor. Byte `+5` is the retained class-0
/// internal-context count; nearby bytes have mixed and negative-offset users.
#[repr(C, align(4))]
struct ContextCompletionPrefix {
    storage: OpaqueBytes<0x14>,
}

opaque_family!(
    /// Occupied undecoded bytes between completion accounting and the internal
    /// context global prefix.
    PreInternalContextQuarantine,
    0xec
);
opaque_family!(
    /// Internal-context global/header prefix. The pool free head follows this
    /// record at the historically qualified address.
    InternalContextPrefix,
    0x14
);

/// One exact internal TX context record. It remains quarantine because retained
/// teardown and diagnostics can mutate the same bytes.
#[repr(C, align(4))]
pub(crate) struct InternalTxContext {
    storage: OpaqueBytes<INTERNAL_TX_CONTEXT_SIZE>,
}

/// Existing typed internal TX pool, now embedded in the complete DTCM layout.
#[repr(C, align(4))]
pub(crate) struct InternalContextPoolState {
    free_head: SharedU32,
    contexts: [InternalTxContext; INTERNAL_TX_CONTEXT_COUNT],
}

opaque_family!(
    /// One occupied `0x208` power-save family. Per-interface calculations use
    /// an observed `0x104` stride, but larger relative accesses may be interior
    /// or overlapping views, so no record partition or ownership shape is asserted.
    PowerSaveFamily,
    0x208
);

opaque_family!(
    /// Occupied boundary bytes between power-save and HIF state.
    PowerSaveHifBoundary,
    0x44
);
opaque_family!(
    /// HIF buffer/free-list and deferred-transfer roots.
    HifBufferState,
    0x34
);
opaque_family!(
    /// Historical HIF software/ring state. Previous decoded views overlap, so
    /// this candidate intentionally exposes one opaque family only.
    LegacyHifSoftwareState,
    0x1d4
);
opaque_family!(
    /// MIC/HIF completion queue state.
    MicCompletionState,
    0x14
);
opaque_family!(
    /// PHY/RF/calibration/channel/gain core with mixed native/vendor history.
    PhyCoreState,
    0xd0
);
opaque_family!(
    /// Remaining PHY and unknown vendor-zeroed tail.
    PhyTail,
    0x238
);
opaque_family!(
    /// Bytes beyond the vendor zero-fill endpoint. They remain occupied
    /// research quarantine and are never exposed as spare capacity.
    ResearchMargin,
    0x3bc
);

/// Complete linker-owned retained-state view. Every field is private so the
/// type is an ABI map, not a safe ownership surface.
#[repr(C, align(4))]
struct DtcmLayout {
    initialized_vendor_image: InitializedVendorImage, // 0x0000
    runtime_prefix: RuntimePrefix,                    // 0x2078
    clock_parameters: ClockParameterIsland,           // 0x218c
    scheduler_handlers: SchedulerHandlerTable,        // 0x21b4
    pre_configuration_tables: PreConfigurationTables, // 0x2234
    sdd_configuration_tables: SddConfigurationTables, // 0x34b0
    wake_context_state: WakeContextState,             // 0x35e0
    duration_sources: DurationSources,                // 0x3670
    pre_low_mac_word: PreLowMacWord,                  // 0x3674
    low_mac_pas: LowMacPasFamily,                     // 0x3678
    pre_vif_header: PreVifHeader,                     // 0x3e78
    vifs: VifRecords,                                 // 0x3e98
    post_vif_quarantine: PostVifQuarantine,           // 0x49a8
    host_contexts: HostTxContexts,                    // 0x5a24
    pre_command_quarantine: PreCommandQuarantine,     // 0x8544
    command_channel_switch: CommandChannelSwitchOverlay, // 0x8594
    lmc_control_roots: LmcControlRoots,               // 0x8618
    host_context_accounting: HostContextAccounting,   // 0x8798
    host_context_free_list: HostContextFreeList,      // 0x87b0
    link_and_sequence: LinkAndSequenceState,          // 0x87b8
    join_scan_control: JoinScanControl,               // 0x89d8
    wsm_response_scratch: WsmResponseScratch,         // 0x8a18
    ba_lmc_header: BaLmcHeader,                       // 0x8ab8
    pending_ba_lmc: PendingBaLmcState,                // 0x8ad8
    lmc_messages: LmcMessages,                        // 0x8bb8
    post_lmc_quarantine: PostLmcQuarantine,           // 0x8e78
    ba_link_event_state: BaLinkEventState,            // 0x8f18
    tala: TalaAccounting,                             // 0x8f48
    context_completion_prefix: ContextCompletionPrefix, // 0x8f6c
    pre_internal_context_quarantine: PreInternalContextQuarantine, // 0x8f80
    internal_context_prefix: InternalContextPrefix,   // 0x906c
    internal_context_pool: InternalContextPoolState,  // 0x9080
    power_save: PowerSaveFamily,                      // 0x94d4
    power_save_hif_boundary: PowerSaveHifBoundary,    // 0x96dc
    hif_buffer_state: HifBufferState,                 // 0x9720
    legacy_hif_software_state: LegacyHifSoftwareState, // 0x9754
    mic_completion_state: MicCompletionState,         // 0x9928
    phy_core: PhyCoreState,                           // 0x993c
    phy_tail: PhyTail,                                // 0x9a0c
    research_margin: ResearchMargin,                  // 0x9c44
}

/// The only lower-DTCM allocation. The linker places this NOBITS object at
/// `DTCM_STATE_BASE`; loader COPY data below `+0x2078` and explicit runtime
/// zeroing from `+0x2078` retain their existing behavior.
#[repr(transparent)]
struct SharedDtcmState(UnsafeCell<MaybeUninit<DtcmLayout>>);

unsafe impl Sync for SharedDtcmState {}

#[unsafe(no_mangle)]
#[used]
#[unsafe(link_section = ".dtcm.state")]
static DTCM_STATE: SharedDtcmState = SharedDtcmState(UnsafeCell::new(MaybeUninit::uninit()));

#[cfg(any(test, not(target_arch = "arm")))]
#[inline(always)]
fn layout_ptr() -> *mut DtcmLayout {
    DTCM_STATE.0.get().cast::<DtcmLayout>()
}

#[cfg(target_arch = "arm")]
unsafe extern "C" {
    static mut __dtcm_context_pool_start: InternalContextPoolState;
}

/// Raw pointer to the internal-context pool. ARM code addresses the
/// linker-exported member symbol directly, preserving the parent symbol/addend
/// materialization while the bytes remain physically inside `DTCM_STATE`.
#[inline(always)]
pub(crate) fn internal_context_pool_ptr() -> *mut InternalContextPoolState {
    #[cfg(target_arch = "arm")]
    {
        addr_of_mut!(__dtcm_context_pool_start)
    }
    #[cfg(not(target_arch = "arm"))]
    {
        unsafe { addr_of_mut!((*layout_ptr()).internal_context_pool) }
    }
}

/// Raw pointer to the internal free-list head; callers must use volatile access.
#[inline(always)]
pub(crate) fn internal_context_free_head_ptr() -> *mut u32 {
    internal_context_pool_ptr().cast::<u32>()
}

/// Raw pointer to one internal context, checked against the exact three-record
/// pool but never converted into a safe reference.
#[inline(always)]
pub(crate) fn internal_context_ptr(index: usize) -> Option<*mut InternalTxContext> {
    if index >= INTERNAL_TX_CONTEXT_COUNT {
        return None;
    }
    #[cfg(target_arch = "arm")]
    let contexts = unsafe {
        internal_context_pool_ptr()
            .cast::<u8>()
            .add(4)
            .cast::<InternalTxContext>()
    };
    #[cfg(not(target_arch = "arm"))]
    let contexts = unsafe {
        addr_of_mut!((*internal_context_pool_ptr()).contexts).cast::<InternalTxContext>()
    };
    Some(unsafe { contexts.add(index) })
}

const LOW_MAC_PAS_OFFSET: usize = 0x3600 + 0x78;

pub const INITIALIZED_VENDOR_IMAGE: DtcmAddress = DtcmAddress::from_offset(0x0000);
pub const SCHEDULER_EVENT_ROOT: DtcmAddress = DtcmAddress::from_offset(0x1fd4);
pub const CLOCK_PARAMETERS: DtcmAddress = DtcmAddress::from_offset(0x218c);
pub const SCHEDULER_HANDLER_TABLE: DtcmAddress = DtcmAddress::from_offset(0x21b4);
pub const LOW_MAC_PAS_ROOT: DtcmAddress = DtcmAddress::from_offset(LOW_MAC_PAS_OFFSET);
pub const VIF_RECORDS: DtcmAddress = DtcmAddress::from_offset(0x3e98);
pub const HOST_TX_CONTEXTS: DtcmAddress = DtcmAddress::from_offset(0x5a24);
pub const COMMAND_CHANNEL_SWITCH_OVERLAY: DtcmAddress = DtcmAddress::from_offset(0x8594);
pub const HOST_TX_CONTEXT_FREE_HEAD: DtcmAddress = DtcmAddress::from_offset(0x87b0);
pub const LINK_SEQUENCE_ROOT: DtcmAddress = DtcmAddress::from_offset(0x87b8);
pub const TALA_ACCOUNTING: DtcmAddress = DtcmAddress::from_offset(0x8f48);
pub const CONTEXT_COMPLETION_PREFIX: DtcmAddress = DtcmAddress::from_offset(0x8f6c);
pub const INTERNAL_CONTEXT_PREFIX: DtcmAddress = DtcmAddress::from_offset(0x906c);
pub const INTERNAL_CONTEXT_POOL: DtcmAddress = DtcmAddress::from_offset(0x9080);
pub const POWER_SAVE_FAMILY: DtcmAddress = DtcmAddress::from_offset(0x94d4);

/// Returns one observed stride-based view start inside the opaque power-save
/// family. This does not confer record ownership or resolve overlapping extents.
pub fn power_save_observed_view(index: usize) -> Option<DtcmAddress> {
    let offset = index.checked_mul(POWER_SAVE_OBSERVED_STRIDE)?;
    (offset < 0x208).then(|| DtcmAddress::from_offset(POWER_SAVE_FAMILY.offset() + offset))
}
pub const HIF_BUFFER_STATE: DtcmAddress = DtcmAddress::from_offset(0x9720);
pub const MIC_COMPLETION_STATE: DtcmAddress = DtcmAddress::from_offset(0x9928);
pub const PHY_STATE: DtcmAddress = DtcmAddress::from_offset(0x993c);
pub const VENDOR_BSS_START: DtcmAddress = DtcmAddress::from_offset(0x2078);
pub const VENDOR_BSS_END: DtcmAddress = DtcmAddress::from_offset(0x9c44);

macro_rules! assert_type_layout {
    ($type:ty, $size:expr, $align:expr) => {
        assert!(core::mem::size_of::<$type>() == $size);
        assert!(core::mem::align_of::<$type>() == $align);
    };
}

const _: () = {
    assert_type_layout!(DtcmAddress, 4, 4);
    assert_type_layout!(InitializedVendorImage, 0x2078, 4);
    assert_type_layout!(RuntimePrefix, 0x114, 4);
    assert_type_layout!(ClockParameterIsland, 0x28, 4);
    assert_type_layout!(SchedulerHandlerTable, 0x80, 4);
    assert_type_layout!(PreConfigurationTables, 0x127c, 4);
    assert_type_layout!(SddConfigurationTables, 0x130, 4);
    assert_type_layout!(WakeContextState, 0x90, 4);
    assert_type_layout!(DurationSources, 0x4, 4);
    assert_type_layout!(PreLowMacWord, 0x4, 4);
    assert_type_layout!(LowMacPasFamily, 0x800, 4);
    assert_type_layout!(PreVifHeader, 0x20, 4);
    assert_type_layout!(VifRecord, VIF_RECORD_SIZE, 4);
    assert_type_layout!(VifRecords, 0xb10, 4);
    assert_type_layout!(PostVifQuarantine, 0x107c, 4);
    assert_type_layout!(HostTxContext, HOST_TX_CONTEXT_SIZE, 4);
    assert_type_layout!(HostTxContexts, 0x2b20, 4);
    assert_type_layout!(PreCommandQuarantine, 0x50, 4);
    assert_type_layout!(CommandChannelSwitchOverlay, 0x84, 4);
    assert_type_layout!(LmcControlRoots, 0x180, 4);
    assert_type_layout!(HostContextAccounting, 0x18, 4);
    assert_type_layout!(HostContextFreeList, 0x8, 4);
    assert_type_layout!(LinkAndSequenceState, 0x220, 4);
    assert_type_layout!(JoinScanControl, 0x40, 4);
    assert_type_layout!(WsmResponseScratch, 0xa0, 4);
    assert_type_layout!(BaLmcHeader, 0x20, 4);
    assert_type_layout!(PendingBaLmcState, 0xe0, 4);
    assert_type_layout!(LmcMessage, LMC_MESSAGE_SIZE, 4);
    assert_type_layout!(LmcMessages, 0x2c0, 4);
    assert_type_layout!(PostLmcQuarantine, 0xa0, 4);
    assert_type_layout!(BaLinkEventState, 0x30, 4);
    assert_type_layout!(TalaAccounting, 0x24, 4);
    assert_type_layout!(ContextCompletionPrefix, 0x14, 4);
    assert_type_layout!(PreInternalContextQuarantine, 0xec, 4);
    assert_type_layout!(InternalContextPrefix, 0x14, 4);
    assert_type_layout!(InternalTxContext, INTERNAL_TX_CONTEXT_SIZE, 4);
    assert_type_layout!(InternalContextPoolState, 0x454, 4);
    assert_type_layout!(PowerSaveFamily, 0x208, 4);
    assert_type_layout!(PowerSaveHifBoundary, 0x44, 4);
    assert_type_layout!(HifBufferState, 0x34, 4);
    assert_type_layout!(LegacyHifSoftwareState, 0x1d4, 4);
    assert_type_layout!(MicCompletionState, 0x14, 4);
    assert_type_layout!(PhyCoreState, 0xd0, 4);
    assert_type_layout!(PhyTail, 0x238, 4);
    assert_type_layout!(ResearchMargin, 0x3bc, 4);
    assert_type_layout!(DtcmLayout, DTCM_STATE_SIZE, 4);
    assert_type_layout!(SharedDtcmState, DTCM_STATE_SIZE, 4);

    assert!(core::mem::offset_of!(InitializedVendorImage, tx_duration_timing) == 0x0138);
    assert!(core::mem::offset_of!(InitializedVendorImage, rate_encoding) == 0x0194);
    assert!(core::mem::offset_of!(InitializedVendorImage, rate_attributes) == 0x01aa);
    assert!(core::mem::offset_of!(InitializedVendorImage, visible_completion_words) == 0x0260);
    assert!(core::mem::offset_of!(InitializedVendorImage, ring_cursor_map) == 0x02d8);
    assert!(core::mem::offset_of!(InitializedVendorImage, pipe_status_maps) == 0x02dc);
    assert!(core::mem::offset_of!(InitializedVendorImage, command_dispatch) == 0x0710);
    assert!(core::mem::offset_of!(InitializedVendorImage, aes_transfer_descriptors) == 0x0804);
    assert!(core::mem::offset_of!(InitializedVendorImage, aes_mode1_microcode) == 0x0830);
    assert!(core::mem::offset_of!(InitializedVendorImage, pre_duration_quantum_pointers) == 0x09de);
    assert!(core::mem::offset_of!(InitializedVendorImage, duration_quantum_pointers) == 0x10d4);
    assert!(core::mem::offset_of!(InitializedVendorImage, hif_control_shadow) == 0x11ac);
    assert!(core::mem::offset_of!(InitializedVendorImage, irq_callbacks) == 0x11bc);
    assert!(core::mem::offset_of!(InitializedVendorImage, ampdu_counters) == 0x12a0);
    assert!(core::mem::offset_of!(InitializedVendorImage, control_words) == 0x1420);
    assert!(core::mem::offset_of!(InitializedVendorImage, pre_low_mac_root) == 0x1440);
    assert!(core::mem::offset_of!(InitializedVendorImage, initialized_low_mac_state) == 0x1680);
    assert!(
        core::mem::offset_of!(InitializedVendorImage, initialized_low_mac_state) + 0x7ec == 0x1e6c
    );
    assert!(
        core::mem::offset_of!(InitializedVendorImage, initialized_low_mac_state) + 0x8f8 == 0x1f78
    );
    assert!(
        core::mem::offset_of!(InitializedVendorImage, initialized_low_mac_state) + 0x940 == 0x1fc0
    );
    assert!(core::mem::offset_of!(InitializedVendorImage, scheduler_exclusion_words) == 0x1fcc);
    assert!(core::mem::offset_of!(InitializedVendorImage, scheduler_event_island) == 0x1fd4);
    assert!(core::mem::offset_of!(InitializedVendorImage, initialized_tail) == 0x2018);

    assert!(core::mem::offset_of!(TalaAccounting, growth_streaks) == 0x00);
    assert!(core::mem::offset_of!(TalaAccounting, successes) == 0x04);
    assert!(core::mem::offset_of!(TalaAccounting, failures) == 0x0c);
    assert!(core::mem::offset_of!(TalaAccounting, cumulative_tries) == 0x14);
    assert!(core::mem::offset_of!(TalaAccounting, weighted_penalties) == 0x1c);
    assert!(core::mem::offset_of!(InternalContextPoolState, free_head) == 0x000);
    assert!(core::mem::offset_of!(InternalContextPoolState, contexts) == 0x004);

    assert!(core::mem::offset_of!(DtcmLayout, initialized_vendor_image) == 0x0000);
    assert!(core::mem::offset_of!(DtcmLayout, runtime_prefix) == 0x2078);
    assert!(core::mem::offset_of!(DtcmLayout, clock_parameters) == 0x218c);
    assert!(core::mem::offset_of!(DtcmLayout, scheduler_handlers) == 0x21b4);
    assert!(core::mem::offset_of!(DtcmLayout, pre_configuration_tables) == 0x2234);
    assert!(core::mem::offset_of!(DtcmLayout, sdd_configuration_tables) == 0x34b0);
    assert!(core::mem::offset_of!(DtcmLayout, wake_context_state) == 0x35e0);
    assert!(core::mem::offset_of!(DtcmLayout, duration_sources) == 0x3670);
    assert!(core::mem::offset_of!(DtcmLayout, pre_low_mac_word) == 0x3674);
    assert!(core::mem::offset_of!(DtcmLayout, low_mac_pas) == LOW_MAC_PAS_OFFSET);
    assert!(core::mem::offset_of!(DtcmLayout, pre_vif_header) == 0x3e78);
    assert!(core::mem::offset_of!(DtcmLayout, vifs) == 0x3e98);
    assert!(core::mem::offset_of!(DtcmLayout, post_vif_quarantine) == 0x49a8);
    assert!(core::mem::offset_of!(DtcmLayout, host_contexts) == 0x5a24);
    assert!(core::mem::offset_of!(DtcmLayout, pre_command_quarantine) == 0x8544);
    assert!(core::mem::offset_of!(DtcmLayout, command_channel_switch) == 0x8594);
    assert!(core::mem::offset_of!(DtcmLayout, lmc_control_roots) == 0x8618);
    assert!(core::mem::offset_of!(DtcmLayout, host_context_accounting) == 0x8798);
    assert!(core::mem::offset_of!(DtcmLayout, host_context_free_list) == 0x87b0);
    assert!(core::mem::offset_of!(DtcmLayout, link_and_sequence) == 0x87b8);
    assert!(core::mem::offset_of!(DtcmLayout, join_scan_control) == 0x89d8);
    assert!(core::mem::offset_of!(DtcmLayout, wsm_response_scratch) == 0x8a18);
    assert!(core::mem::offset_of!(DtcmLayout, ba_lmc_header) == 0x8ab8);
    assert!(core::mem::offset_of!(DtcmLayout, pending_ba_lmc) == 0x8ad8);
    assert!(core::mem::offset_of!(DtcmLayout, lmc_messages) == 0x8bb8);
    assert!(core::mem::offset_of!(DtcmLayout, post_lmc_quarantine) == 0x8e78);
    assert!(core::mem::offset_of!(DtcmLayout, ba_link_event_state) == 0x8f18);
    assert!(core::mem::offset_of!(DtcmLayout, tala) == 0x8f48);
    assert!(core::mem::offset_of!(DtcmLayout, context_completion_prefix) == 0x8f6c);
    assert!(core::mem::offset_of!(DtcmLayout, pre_internal_context_quarantine) == 0x8f80);
    assert!(core::mem::offset_of!(DtcmLayout, internal_context_prefix) == 0x906c);
    assert!(core::mem::offset_of!(DtcmLayout, internal_context_pool) == 0x9080);
    assert!(core::mem::offset_of!(DtcmLayout, power_save) == 0x94d4);
    assert!(core::mem::offset_of!(DtcmLayout, power_save_hif_boundary) == 0x96dc);
    assert!(core::mem::offset_of!(DtcmLayout, hif_buffer_state) == 0x9720);
    assert!(core::mem::offset_of!(DtcmLayout, legacy_hif_software_state) == 0x9754);
    assert!(core::mem::offset_of!(DtcmLayout, mic_completion_state) == 0x9928);
    assert!(core::mem::offset_of!(DtcmLayout, phy_core) == 0x993c);
    assert!(core::mem::offset_of!(DtcmLayout, phy_tail) == 0x9a0c);
    assert!(core::mem::offset_of!(DtcmLayout, research_margin) == 0x9c44);
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_addresses_reject_non_state_ranges() {
        assert_eq!(
            DtcmAddress::new(DTCM_STATE_BASE).map(DtcmAddress::offset),
            Some(0)
        );
        assert_eq!(
            DtcmAddress::new(DTCM_STATE_END - 1).map(DtcmAddress::offset),
            Some(DTCM_STATE_SIZE - 1)
        );
        assert!(DtcmAddress::new(DTCM_STATE_BASE - 1).is_none());
        assert!(DtcmAddress::new(DTCM_STATE_END).is_none());
    }

    #[test]
    fn internal_pool_views_are_derived_from_the_top_level_symbol() {
        let state = layout_ptr().addr();
        let pool = internal_context_pool_ptr().addr();
        let head = internal_context_free_head_ptr().addr();
        let first = internal_context_ptr(0).map(|pointer| pointer.addr());
        let last =
            internal_context_ptr(INTERNAL_TX_CONTEXT_COUNT - 1).map(|pointer| pointer.addr());

        assert_eq!(pool - state, 0x9080);
        assert_eq!(head - state, 0x9080);
        assert_eq!(first.map(|address| address - state), Some(0x9084));
        assert_eq!(
            last.zip(first).map(|(last, first)| last - first),
            Some(2 * INTERNAL_TX_CONTEXT_SIZE)
        );
        assert!(internal_context_ptr(INTERNAL_TX_CONTEXT_COUNT).is_none());
    }
}
