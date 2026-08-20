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
        Self::from_offset_unchecked(offset)
    }

    const fn from_offset_unchecked(offset: usize) -> Self {
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
/// One packed host TX retry policy as consumed by the retained PAS retry
/// algorithm. Rust has a separate native policy table for translated class-0
/// retry decisions, so these bytes remain volatile shared ABI state.
#[repr(C)]
struct PasRatePolicy {
    policy_index: SharedU8,
    short_retry_limit: SharedU8,
    long_retry_limit: SharedU8,
    control_byte_03: SharedU8,
    counter_byte_04: SharedU8,
    reserved_05: OpaqueBytes<3>,
    packed_rate_retries: [SharedU8; 12],
}

/// Four bytes addressed as one cached retry-walk entry. The retained code
/// proves the `0x04` stride, but the complete value domains are not decoded.
#[repr(C)]
struct PasRateWalkState {
    byte_00: SharedU8,
    byte_01: SharedU8,
    byte_02: SharedU8,
    byte_03: SharedU8,
}

/// Shared MAC runtime bytes rooted at family offset `0x3e0`. These are split
/// into scalar fields only where current Rust and raw vendor instructions agree
/// on width and role. The opaque bytes include timer/beacon state whose exact
/// overlays remain unresolved.
#[repr(C)]
struct LowMacRuntimeState {
    opaque_00: OpaqueBytes<0x10>,
    current_channel: SharedU16,
    optional_pipe_object_word: SharedU16,
    active_tx_count: SharedU8,
    receive_gate_bits: SharedU8,
    receive_state_byte: SharedU8,
    opaque_17: OpaqueBytes<1>,
    response_control_byte: SharedU8,
    opaque_19: OpaqueBytes<7>,
}

/// Exact 12-byte WSM SET_TX_QUEUE_PARAMS payload retained per access category.
/// It is distinct from the per-PAS EDCA arrays below.
#[repr(C)]
struct TxQueueParameters {
    storage: OpaqueBytes<0x0c>,
}

/// Layout template for fields observed relative to each of three `0x98`-spaced
/// PAS view starts. This type is not embedded three times in the family: the
/// alternate root at family `+0x620` starts inside the third view and accesses
/// `+0x19`/`+0x1a` beyond that view's nominal `0x98` extent. It is therefore a
/// field-offset schema only, not proof of three independent records.
#[repr(C)]
struct PasStrideLayout {
    activity_state: SharedU8,
    slot_bits: SharedU8,
    mode_byte: SharedU8,
    path_selector_byte: SharedU8,
    next_tbtt_low: SharedU32,
    basic_rate_bits: SharedU32,
    own_mac: [SharedU8; 6],
    bssid: [SharedU8; 6],
    tsf_adjust_low: SharedU32,
    tsf_adjust_high: SharedU32,
    rate_class_byte: SharedU8,
    rate_table_column: SharedU8,
    nonzero_block_byte: SharedU8,
    tbtt_window_control_byte: SharedU8,
    rate_map: [SharedU8; 22],
    opaque_3a: OpaqueBytes<2>,
    retry_counts: [SharedU32; 4],
    contention_windows: [SharedU32; 4],
    cw_min: [SharedU16; 4],
    cw_max: [SharedU16; 4],
    aifs: [SharedU8; 4],
    txop_limits: [SharedU16; 4],
    max_rx_lifetimes: [SharedU32; 4],
    slot_timing_word: SharedU32,
    packed_aifs: SharedU32,
    duration_extension_bits: SharedU32,
    opaque_94: OpaqueBytes<4>,
}

/// Offset schema for the alternate raw root at family `+0x620`
/// (`0x04003c98`). Its start overlaps the final `0x18` bytes of the third PAS
/// stride view; `byte_19` and `byte_1a` lie immediately beyond that view's
/// nominal end. The complete extent and semantics remain unresolved.
#[repr(C)]
struct AlternatePasRootLayout {
    byte_00: SharedU8,
    opaque_01: OpaqueBytes<0x18>,
    byte_19: SharedU8,
    byte_1a: SharedU8,
}

/// Shared low-MAC/PAS family at `0x04003678..0x04003e78`.
///
/// The prefix contains two packet-buffer boundary publications, eight raw
/// retry policies, eight cached retry-walk states, MAC runtime controls, queue
/// parameters, two six-byte address vectors, and four response-enable bytes.
/// Three observed PAS view starts occur at `+0x470 + n*0x98`. They are not
/// represented as three fields: the alternate raw root at `+0x620` overlaps
/// the third view, and its `+0x19`/`+0x1a` accesses cross that view's nominal
/// end. The complete `+0x470..+0x800` region is one private overlapping raw
/// view that also contains link timers and the BA root at `+0x648`; the latter
/// has computed extents crossing into the following pre-VIF header.
#[repr(C, align(4))]
struct LowMacPasFamily {
    opaque_000: OpaqueBytes<0x08>,
    beacon_interval_ticks: SharedU32,
    active_band_mask: SharedU32,
    internal_buffer_end_primary: SharedU32,
    opaque_014: OpaqueBytes<0x6c>,
    internal_buffer_end_mirror: SharedU32,
    opaque_084: OpaqueBytes<0x6c>,
    rate_policies: [PasRatePolicy; 8],
    opaque_190: OpaqueBytes<0x1e0>,
    rate_walks: [PasRateWalkState; 8],
    opaque_390: OpaqueBytes<0x50>,
    runtime: LowMacRuntimeState,
    queue_parameters: [TxQueueParameters; 4],
    txop_budgets: [SharedU32; 4],
    opaque_440: OpaqueBytes<0x14>,
    own_mac_addresses: [[SharedU8; 6]; 2],
    peer_addresses: [[SharedU8; 6]; 2],
    response_enabled: [SharedU8; 4],
    overlapping_pas_link_ba_views: OpaqueBytes<0x390>,
}
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

const LOW_MAC_PAS_OFFSET: usize = core::mem::offset_of!(DtcmLayout, low_mac_pas);
pub const LOW_MAC_PAS_SIZE: usize = core::mem::size_of::<LowMacPasFamily>();
pub const PAS_VIEW_COUNT: usize = 3;
pub const PAS_VIEW_STRIDE: usize = core::mem::size_of::<PasStrideLayout>();
const PAS_VIEWS_OFFSET: usize =
    core::mem::offset_of!(LowMacPasFamily, overlapping_pas_link_ba_views);
const ALTERNATE_PAS_ROOT_OFFSET: usize = PAS_VIEWS_OFFSET + 0x1b0;

/// Address of one of three observed `0x98`-spaced PAS raw views. The type uses
/// `PasStrideLayout` only to derive proven relative offsets; it does not imply
/// that the views are disjoint records or yield any reference.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PasStrideViewAddress(DtcmAddress);

impl PasStrideViewAddress {
    const fn field_unchecked(self, offset: usize) -> DtcmAddress {
        DtcmAddress::from_offset_unchecked(self.0.offset() + offset)
    }

    pub(crate) const fn activity_state(self) -> DtcmAddress {
        self.field_unchecked(core::mem::offset_of!(PasStrideLayout, activity_state))
    }
    pub(crate) const fn slot_bits(self) -> DtcmAddress {
        self.field_unchecked(core::mem::offset_of!(PasStrideLayout, slot_bits))
    }
    pub(crate) const fn mode_byte(self) -> DtcmAddress {
        self.field_unchecked(core::mem::offset_of!(PasStrideLayout, mode_byte))
    }
    pub(crate) const fn path_selector_byte(self) -> DtcmAddress {
        self.field_unchecked(core::mem::offset_of!(PasStrideLayout, path_selector_byte))
    }
    pub(crate) const fn next_tbtt_low(self) -> DtcmAddress {
        self.field_unchecked(core::mem::offset_of!(PasStrideLayout, next_tbtt_low))
    }
    pub(crate) const fn basic_rate_bits(self) -> DtcmAddress {
        self.field_unchecked(core::mem::offset_of!(PasStrideLayout, basic_rate_bits))
    }
    pub(crate) const fn own_mac_byte(self, index: usize) -> Option<DtcmAddress> {
        if index < 6 { Some(self.own_mac_byte_unchecked(index)) } else { None }
    }
    /// Intentionally preserves unbounded vendor base-plus-index arithmetic.
    pub(crate) const fn own_mac_byte_unchecked(self, index: usize) -> DtcmAddress {
        self.field_unchecked(core::mem::offset_of!(PasStrideLayout, own_mac) + index)
    }
    pub(crate) const fn bssid_byte(self, index: usize) -> Option<DtcmAddress> {
        if index < 6 { Some(self.bssid_byte_unchecked(index)) } else { None }
    }
    /// Intentionally preserves unbounded vendor base-plus-index arithmetic.
    pub(crate) const fn bssid_byte_unchecked(self, index: usize) -> DtcmAddress {
        self.field_unchecked(core::mem::offset_of!(PasStrideLayout, bssid) + index)
    }
    pub(crate) const fn tsf_adjust_low(self) -> DtcmAddress {
        self.field_unchecked(core::mem::offset_of!(PasStrideLayout, tsf_adjust_low))
    }
    pub(crate) const fn tsf_adjust_high(self) -> DtcmAddress {
        self.field_unchecked(core::mem::offset_of!(PasStrideLayout, tsf_adjust_high))
    }
    pub(crate) const fn rate_table_column(self) -> DtcmAddress {
        self.field_unchecked(core::mem::offset_of!(PasStrideLayout, rate_table_column))
    }
    pub(crate) const fn nonzero_block_byte(self) -> DtcmAddress {
        self.field_unchecked(core::mem::offset_of!(PasStrideLayout, nonzero_block_byte))
    }
    pub(crate) const fn tbtt_window_control_byte(self) -> DtcmAddress {
        self.field_unchecked(core::mem::offset_of!(PasStrideLayout, tbtt_window_control_byte))
    }
    pub(crate) const fn rate_map(self, rate: usize) -> Option<DtcmAddress> {
        if rate < 22 { Some(self.rate_map_unchecked(rate)) } else { None }
    }
    /// Intentionally preserves unbounded vendor base-plus-rate arithmetic.
    pub(crate) const fn rate_map_unchecked(self, rate: usize) -> DtcmAddress {
        self.field_unchecked(core::mem::offset_of!(PasStrideLayout, rate_map) + rate)
    }
    pub(crate) const fn retry_count(self, queue: usize) -> Option<DtcmAddress> {
        if queue < 4 { Some(self.retry_count_unchecked(queue)) } else { None }
    }
    /// Intentionally preserves unbounded vendor base-plus-queue arithmetic.
    pub(crate) const fn retry_count_unchecked(self, queue: usize) -> DtcmAddress {
        self.field_unchecked(core::mem::offset_of!(PasStrideLayout, retry_counts) + queue * 4)
    }
    pub(crate) const fn contention_window(self, queue: usize) -> Option<DtcmAddress> {
        if queue < 4 { Some(self.contention_window_unchecked(queue)) } else { None }
    }
    /// Intentionally preserves unbounded vendor base-plus-queue arithmetic.
    pub(crate) const fn contention_window_unchecked(self, queue: usize) -> DtcmAddress {
        self.field_unchecked(core::mem::offset_of!(PasStrideLayout, contention_windows) + queue * 4)
    }
    pub(crate) const fn cw_min(self, queue: usize) -> Option<DtcmAddress> {
        if queue < 4 { Some(self.cw_min_unchecked(queue)) } else { None }
    }
    /// Intentionally preserves unbounded vendor base-plus-queue arithmetic.
    pub(crate) const fn cw_min_unchecked(self, queue: usize) -> DtcmAddress {
        self.field_unchecked(core::mem::offset_of!(PasStrideLayout, cw_min) + queue * 2)
    }
    pub(crate) const fn cw_max(self, queue: usize) -> Option<DtcmAddress> {
        if queue < 4 { Some(self.cw_max_unchecked(queue)) } else { None }
    }
    /// Intentionally preserves unbounded vendor base-plus-queue arithmetic.
    pub(crate) const fn cw_max_unchecked(self, queue: usize) -> DtcmAddress {
        self.field_unchecked(core::mem::offset_of!(PasStrideLayout, cw_max) + queue * 2)
    }
    pub(crate) const fn aifs(self, queue: usize) -> Option<DtcmAddress> {
        if queue < 4 { Some(self.aifs_unchecked(queue)) } else { None }
    }
    /// Intentionally preserves unbounded vendor base-plus-queue arithmetic.
    pub(crate) const fn aifs_unchecked(self, queue: usize) -> DtcmAddress {
        self.field_unchecked(core::mem::offset_of!(PasStrideLayout, aifs) + queue)
    }
    pub(crate) const fn txop_limit(self, queue: usize) -> Option<DtcmAddress> {
        if queue < 4 { Some(self.txop_limit_unchecked(queue)) } else { None }
    }
    /// Intentionally preserves unbounded vendor base-plus-queue arithmetic.
    pub(crate) const fn txop_limit_unchecked(self, queue: usize) -> DtcmAddress {
        self.field_unchecked(core::mem::offset_of!(PasStrideLayout, txop_limits) + queue * 2)
    }
    pub(crate) const fn max_rx_lifetime(self, queue: usize) -> Option<DtcmAddress> {
        if queue < 4 { Some(self.max_rx_lifetime_unchecked(queue)) } else { None }
    }
    /// Intentionally preserves unbounded vendor base-plus-queue arithmetic.
    pub(crate) const fn max_rx_lifetime_unchecked(self, queue: usize) -> DtcmAddress {
        self.field_unchecked(core::mem::offset_of!(PasStrideLayout, max_rx_lifetimes) + queue * 4)
    }
    pub(crate) const fn slot_timing_word(self) -> DtcmAddress {
        self.field_unchecked(core::mem::offset_of!(PasStrideLayout, slot_timing_word))
    }
    pub(crate) const fn packed_aifs(self) -> DtcmAddress {
        self.field_unchecked(core::mem::offset_of!(PasStrideLayout, packed_aifs))
    }
    pub(crate) const fn duration_extension_bits(self) -> DtcmAddress {
        self.field_unchecked(core::mem::offset_of!(PasStrideLayout, duration_extension_bits))
    }
}

pub(crate) const fn pas_stride_view(interface: usize) -> Option<PasStrideViewAddress> {
    if interface < PAS_VIEW_COUNT { Some(pas_stride_view_unchecked(interface)) } else { None }
}

/// Compatibility accessor that intentionally preserves unbounded vendor
/// `base + interface*0x98` arithmetic. Callers with production interface
/// semantics should reject values outside `0..3` before using it.
pub(crate) const fn pas_stride_view_unchecked(interface: usize) -> PasStrideViewAddress {
    PasStrideViewAddress(DtcmAddress::from_offset_unchecked(
        LOW_MAC_PAS_OFFSET + PAS_VIEWS_OFFSET + interface * PAS_VIEW_STRIDE,
    ))
}

/// Explicit overlapping raw view rooted at family `+0x620` (`0x04003c98`).
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AlternatePasRootAddress(DtcmAddress);

impl AlternatePasRootAddress {
    pub(crate) const fn byte_00(self) -> DtcmAddress {
        DtcmAddress::from_offset_unchecked(
            self.0.offset() + core::mem::offset_of!(AlternatePasRootLayout, byte_00),
        )
    }
    pub(crate) const fn byte_19(self) -> DtcmAddress {
        DtcmAddress::from_offset_unchecked(
            self.0.offset() + core::mem::offset_of!(AlternatePasRootLayout, byte_19),
        )
    }
    pub(crate) const fn byte_1a(self) -> DtcmAddress {
        DtcmAddress::from_offset_unchecked(
            self.0.offset() + core::mem::offset_of!(AlternatePasRootLayout, byte_1a),
        )
    }
}

pub(crate) const ALTERNATE_PAS_ROOT: AlternatePasRootAddress = AlternatePasRootAddress(
    DtcmAddress::from_offset(LOW_MAC_PAS_OFFSET + ALTERNATE_PAS_ROOT_OFFSET),
);

const fn low_mac_pas_field(offset: usize) -> DtcmAddress {
    assert!(offset < LOW_MAC_PAS_SIZE);
    DtcmAddress::from_offset(LOW_MAC_PAS_OFFSET + offset)
}

pub(crate) const LOW_MAC_BEACON_INTERVAL: DtcmAddress = low_mac_pas_field(
    core::mem::offset_of!(LowMacPasFamily, beacon_interval_ticks),
);
pub(crate) const LOW_MAC_BAND_BITS: DtcmAddress = low_mac_pas_field(
    core::mem::offset_of!(LowMacPasFamily, active_band_mask),
);
pub(crate) const INTERNAL_BUFFER_END_PRIMARY: DtcmAddress = low_mac_pas_field(
    core::mem::offset_of!(LowMacPasFamily, internal_buffer_end_primary),
);
pub(crate) const INTERNAL_BUFFER_END_MIRROR: DtcmAddress = low_mac_pas_field(
    core::mem::offset_of!(LowMacPasFamily, internal_buffer_end_mirror),
);
pub(crate) const LOW_MAC_RUNTIME_ROOT: DtcmAddress = low_mac_pas_field(
    core::mem::offset_of!(LowMacPasFamily, runtime),
);
const LOW_MAC_RUNTIME_OFFSET: usize = core::mem::offset_of!(LowMacPasFamily, runtime);
pub const LOW_MAC_CURRENT_CHANNEL: DtcmAddress = low_mac_pas_field(
    LOW_MAC_RUNTIME_OFFSET + core::mem::offset_of!(LowMacRuntimeState, current_channel),
);
pub(crate) const LOW_MAC_OPTIONAL_PIPE_OBJECT_WORD: DtcmAddress = low_mac_pas_field(
    LOW_MAC_RUNTIME_OFFSET + core::mem::offset_of!(LowMacRuntimeState, optional_pipe_object_word),
);
pub(crate) const LOW_MAC_ACTIVE_TX_COUNT: DtcmAddress = low_mac_pas_field(
    LOW_MAC_RUNTIME_OFFSET + core::mem::offset_of!(LowMacRuntimeState, active_tx_count),
);
pub(crate) const LOW_MAC_RECEIVE_GATE_BITS: DtcmAddress = low_mac_pas_field(
    LOW_MAC_RUNTIME_OFFSET + core::mem::offset_of!(LowMacRuntimeState, receive_gate_bits),
);
pub(crate) const LOW_MAC_RECEIVE_STATE_BYTE: DtcmAddress = low_mac_pas_field(
    LOW_MAC_RUNTIME_OFFSET + core::mem::offset_of!(LowMacRuntimeState, receive_state_byte),
);
pub(crate) const LOW_MAC_RESPONSE_CONTROL_BYTE: DtcmAddress = low_mac_pas_field(
    LOW_MAC_RUNTIME_OFFSET + core::mem::offset_of!(LowMacRuntimeState, response_control_byte),
);

pub(crate) const fn low_mac_own_mac_byte(interface: usize, index: usize) -> Option<DtcmAddress> {
    if interface < 2 && index < 6 {
        Some(low_mac_own_mac_byte_unchecked(interface, index))
    } else {
        None
    }
}

/// Intentionally preserves unbounded vendor base-plus-interface/index arithmetic.
pub(crate) const fn low_mac_own_mac_byte_unchecked(interface: usize, index: usize) -> DtcmAddress {
    DtcmAddress::from_offset_unchecked(
        LOW_MAC_PAS_OFFSET
            + core::mem::offset_of!(LowMacPasFamily, own_mac_addresses)
            + interface * 6
            + index,
    )
}

pub(crate) const fn low_mac_peer_address_byte(
    interface: usize,
    index: usize,
) -> Option<DtcmAddress> {
    if interface < 2 && index < 6 {
        Some(low_mac_peer_address_byte_unchecked(interface, index))
    } else {
        None
    }
}

/// Intentionally preserves unbounded vendor base-plus-interface/index arithmetic.
pub(crate) const fn low_mac_peer_address_byte_unchecked(
    interface: usize,
    index: usize,
) -> DtcmAddress {
    DtcmAddress::from_offset_unchecked(
        LOW_MAC_PAS_OFFSET
            + core::mem::offset_of!(LowMacPasFamily, peer_addresses)
            + interface * 6
            + index,
    )
}

pub(crate) const fn low_mac_response_enabled(interface: usize) -> Option<DtcmAddress> {
    if interface < 4 {
        Some(low_mac_response_enabled_unchecked(interface))
    } else {
        None
    }
}

/// Intentionally preserves unbounded vendor base-plus-interface arithmetic.
pub(crate) const fn low_mac_response_enabled_unchecked(interface: usize) -> DtcmAddress {
    DtcmAddress::from_offset_unchecked(
        LOW_MAC_PAS_OFFSET
            + core::mem::offset_of!(LowMacPasFamily, response_enabled)
            + interface,
    )
}

/// Compatibility address used by completion accounting. The frame policy byte
/// can be the vendor sentinel `0x0f`, whose historical read lands beyond the
/// eight typed host policies in unresolved prefix storage. Keeping this
/// address-only view preserves that branch without falsely typing 16 entries.
pub(crate) const fn rate_policy_short_retry_limit_compat(policy: usize) -> DtcmAddress {
    DtcmAddress::from_offset_unchecked(
        LOW_MAC_PAS_OFFSET
            + core::mem::offset_of!(LowMacPasFamily, rate_policies)
            + core::mem::offset_of!(PasRatePolicy, short_retry_limit)
            + policy * core::mem::size_of::<PasRatePolicy>(),
    )
}

pub(crate) const fn rate_policy_word(policy: usize, word: usize) -> Option<DtcmAddress> {
    if policy < 8 && word < 5 {
        Some(rate_policy_word_unchecked(policy, word))
    } else {
        None
    }
}

/// Intentionally preserves unbounded vendor policy/word arithmetic.
pub(crate) const fn rate_policy_word_unchecked(policy: usize, word: usize) -> DtcmAddress {
    DtcmAddress::from_offset_unchecked(
        LOW_MAC_PAS_OFFSET
            + core::mem::offset_of!(LowMacPasFamily, rate_policies)
            + policy * core::mem::size_of::<PasRatePolicy>()
            + word * 4,
    )
}

/// Root of an unresolved `0x38`-stride BA overlay. Entry seven crosses the
/// `0x3e78` family boundary, so this is an address-only compatibility view.
pub(crate) const fn ba_pipe_record_address(pipe: usize) -> Option<DtcmAddress> {
    if pipe < 8 {
        Some(ba_pipe_record_address_unchecked(pipe))
    } else {
        None
    }
}

/// Intentionally preserves the unbounded vendor BA base-plus-pipe arithmetic,
/// including the historically observed crossing into the pre-VIF header.
pub(crate) const fn ba_pipe_record_address_unchecked(pipe: usize) -> DtcmAddress {
    DtcmAddress::from_offset_unchecked(
        LOW_MAC_PAS_OFFSET + PAS_VIEWS_OFFSET + 0x1d8 + pipe * 0x38,
    )
}

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
    assert_type_layout!(PasRatePolicy, 0x14, 1);
    assert_type_layout!(PasRateWalkState, 0x4, 1);
    assert_type_layout!(LowMacRuntimeState, 0x20, 2);
    assert_type_layout!(TxQueueParameters, 0x0c, 1);
    assert_type_layout!(PasStrideLayout, PAS_VIEW_STRIDE, 4);
    assert_type_layout!(AlternatePasRootLayout, 0x1b, 1);
    assert_type_layout!(PasStrideViewAddress, 4, 4);
    assert_type_layout!(AlternatePasRootAddress, 4, 4);
    assert_type_layout!(LowMacPasFamily, LOW_MAC_PAS_SIZE, 4);
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

    assert!(core::mem::offset_of!(PasRatePolicy, policy_index) == 0x00);
    assert!(core::mem::offset_of!(PasRatePolicy, short_retry_limit) == 0x01);
    assert!(core::mem::offset_of!(PasRatePolicy, long_retry_limit) == 0x02);
    assert!(core::mem::offset_of!(PasRatePolicy, control_byte_03) == 0x03);
    assert!(core::mem::offset_of!(PasRatePolicy, counter_byte_04) == 0x04);
    assert!(core::mem::offset_of!(PasRatePolicy, reserved_05) == 0x05);
    assert!(core::mem::offset_of!(PasRatePolicy, packed_rate_retries) == 0x08);
    assert!(core::mem::offset_of!(PasRateWalkState, byte_00) == 0x00);
    assert!(core::mem::offset_of!(PasRateWalkState, byte_01) == 0x01);
    assert!(core::mem::offset_of!(PasRateWalkState, byte_02) == 0x02);
    assert!(core::mem::offset_of!(PasRateWalkState, byte_03) == 0x03);
    assert!(core::mem::offset_of!(LowMacRuntimeState, current_channel) == 0x10);
    assert!(core::mem::offset_of!(LowMacRuntimeState, optional_pipe_object_word) == 0x12);
    assert!(core::mem::offset_of!(LowMacRuntimeState, active_tx_count) == 0x14);
    assert!(core::mem::offset_of!(LowMacRuntimeState, receive_gate_bits) == 0x15);
    assert!(core::mem::offset_of!(LowMacRuntimeState, receive_state_byte) == 0x16);
    assert!(core::mem::offset_of!(LowMacRuntimeState, response_control_byte) == 0x18);
    assert!(core::mem::offset_of!(PasStrideLayout, activity_state) == 0x00);
    assert!(core::mem::offset_of!(PasStrideLayout, slot_bits) == 0x01);
    assert!(core::mem::offset_of!(PasStrideLayout, mode_byte) == 0x02);
    assert!(core::mem::offset_of!(PasStrideLayout, path_selector_byte) == 0x03);
    assert!(core::mem::offset_of!(PasStrideLayout, next_tbtt_low) == 0x04);
    assert!(core::mem::offset_of!(PasStrideLayout, basic_rate_bits) == 0x08);
    assert!(core::mem::offset_of!(PasStrideLayout, own_mac) == 0x0c);
    assert!(core::mem::offset_of!(PasStrideLayout, bssid) == 0x12);
    assert!(core::mem::offset_of!(PasStrideLayout, tsf_adjust_low) == 0x18);
    assert!(core::mem::offset_of!(PasStrideLayout, tsf_adjust_high) == 0x1c);
    assert!(core::mem::offset_of!(PasStrideLayout, rate_class_byte) == 0x20);
    assert!(core::mem::offset_of!(PasStrideLayout, rate_table_column) == 0x21);
    assert!(core::mem::offset_of!(PasStrideLayout, nonzero_block_byte) == 0x22);
    assert!(core::mem::offset_of!(PasStrideLayout, tbtt_window_control_byte) == 0x23);
    assert!(core::mem::offset_of!(PasStrideLayout, rate_map) == 0x24);
    assert!(core::mem::offset_of!(PasStrideLayout, retry_counts) == 0x3c);
    assert!(core::mem::offset_of!(PasStrideLayout, contention_windows) == 0x4c);
    assert!(core::mem::offset_of!(PasStrideLayout, cw_min) == 0x5c);
    assert!(core::mem::offset_of!(PasStrideLayout, cw_max) == 0x64);
    assert!(core::mem::offset_of!(PasStrideLayout, aifs) == 0x6c);
    assert!(core::mem::offset_of!(PasStrideLayout, txop_limits) == 0x70);
    assert!(core::mem::offset_of!(PasStrideLayout, max_rx_lifetimes) == 0x78);
    assert!(core::mem::offset_of!(PasStrideLayout, slot_timing_word) == 0x88);
    assert!(core::mem::offset_of!(PasStrideLayout, packed_aifs) == 0x8c);
    assert!(core::mem::offset_of!(PasStrideLayout, duration_extension_bits) == 0x90);
    assert!(core::mem::offset_of!(PasStrideLayout, opaque_94) == 0x94);
    assert!(core::mem::offset_of!(AlternatePasRootLayout, byte_00) == 0x00);
    assert!(core::mem::offset_of!(AlternatePasRootLayout, byte_19) == 0x19);
    assert!(core::mem::offset_of!(AlternatePasRootLayout, byte_1a) == 0x1a);
    assert!(core::mem::offset_of!(LowMacPasFamily, beacon_interval_ticks) == 0x008);
    assert!(core::mem::offset_of!(LowMacPasFamily, active_band_mask) == 0x00c);
    assert!(core::mem::offset_of!(LowMacPasFamily, internal_buffer_end_primary) == 0x010);
    assert!(core::mem::offset_of!(LowMacPasFamily, internal_buffer_end_mirror) == 0x080);
    assert!(core::mem::offset_of!(LowMacPasFamily, rate_policies) == 0x0f0);
    assert!(core::mem::offset_of!(LowMacPasFamily, rate_walks) == 0x370);
    assert!(core::mem::offset_of!(LowMacPasFamily, runtime) == 0x3e0);
    assert!(core::mem::offset_of!(LowMacPasFamily, queue_parameters) == 0x400);
    assert!(core::mem::offset_of!(LowMacPasFamily, txop_budgets) == 0x430);
    assert!(core::mem::offset_of!(LowMacPasFamily, own_mac_addresses) == 0x454);
    assert!(core::mem::offset_of!(LowMacPasFamily, peer_addresses) == 0x460);
    assert!(core::mem::offset_of!(LowMacPasFamily, response_enabled) == 0x46c);
    assert!(
        core::mem::offset_of!(LowMacPasFamily, overlapping_pas_link_ba_views) == PAS_VIEWS_OFFSET
    );
    assert!(PAS_VIEWS_OFFSET == 0x470);
    assert!(ALTERNATE_PAS_ROOT_OFFSET == 0x620);

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
    fn low_mac_pas_stride_views_match_declared_field_offsets_and_bounds() {
        let first = pas_stride_view(0).unwrap();
        let last = pas_stride_view(2).unwrap();
        assert_eq!(first.activity_state().offset(), 0x3ae8);
        assert_eq!(last.activity_state().offset(), 0x3c18);
        assert_eq!(first.rate_map(21).unwrap().offset(), 0x3b21);
        assert_eq!(first.retry_count(3).unwrap().offset(), 0x3b30);
        assert_eq!(first.contention_window(3).unwrap().offset(), 0x3b40);
        assert_eq!(first.cw_min(3).unwrap().offset(), 0x3b4a);
        assert_eq!(first.cw_max(3).unwrap().offset(), 0x3b52);
        assert_eq!(first.aifs(3).unwrap().offset(), 0x3b57);
        assert_eq!(first.txop_limit(3).unwrap().offset(), 0x3b5e);
        assert_eq!(first.max_rx_lifetime(3).unwrap().offset(), 0x3b6c);
        assert_eq!(first.duration_extension_bits().offset(), 0x3b78);
        assert!(pas_stride_view(3).is_none());
        assert!(first.rate_map(22).is_none());
        assert!(first.contention_window(4).is_none());
    }

    #[test]
    fn alternate_pas_root_and_ba_views_make_crossing_overlays_explicit() {
        let third = pas_stride_view(2).unwrap();
        assert_eq!(third.activity_state().offset(), 0x3c18);
        assert_eq!(third.field_unchecked(PAS_VIEW_STRIDE).offset(), 0x3cb0);
        assert_eq!(ALTERNATE_PAS_ROOT.byte_00().offset(), 0x3c98);
        assert_eq!(ALTERNATE_PAS_ROOT.byte_19().offset(), 0x3cb1);
        assert_eq!(ALTERNATE_PAS_ROOT.byte_1a().offset(), 0x3cb2);
        assert!(ALTERNATE_PAS_ROOT.byte_00().offset() < 0x3cb0);
        assert!(ALTERNATE_PAS_ROOT.byte_19().offset() >= 0x3cb0);
        assert_eq!(ba_pipe_record_address(0).unwrap().offset(), 0x3cc0);
        assert_eq!(ba_pipe_record_address(7).unwrap().offset(), 0x3e48);
        assert!(ba_pipe_record_address(7).unwrap().get() + 0x38 > DTCM_STATE_BASE + 0x3e78);
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
