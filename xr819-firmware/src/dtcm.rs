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
pub(crate) const INTERNAL_TX_CONTEXT_NEXT_FREE_OFFSET: usize = core::mem::offset_of!(InternalTxContext, next_free);
pub(crate) const INTERNAL_TX_CONTEXT_HEADER_80211_OFFSET: usize = core::mem::offset_of!(InternalTxContext, header_80211);
pub(crate) const INTERNAL_TX_CONTEXT_RESULT_OFFSET: usize = core::mem::offset_of!(InternalTxContext, pas) + core::mem::offset_of!(InternalPasContext, terminal_status);
pub(crate) const INTERNAL_TX_CONTEXT_CIPHER_BUFFER_OFFSET: usize = core::mem::offset_of!(InternalTxContext, pas) + core::mem::offset_of!(InternalPasContext, cipher_buffer);
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
#[repr(C, align(2))] struct TkipSboxTables { low_byte: [SharedU16; 256], high_byte: [SharedU16; 256] } #[repr(C, align(4))] struct AesTransferClassTable { flags: [SharedU32; 11] } #[repr(C, align(4))] struct RegisterWrite { address: SharedU32, value: SharedU32 } #[repr(C, align(4))] struct RegisterWriteList { writes: [RegisterWrite; 10], terminator_address: SharedU32, terminator_opaque: SharedU32 } #[repr(C, align(4))] struct PhyInitRegisterWriteList0 { writes: [RegisterWrite; 1], terminator_address: SharedU32, terminator_opaque: SharedU32 } #[repr(C, align(4))] struct PhyInitRegisterWriteList1 { writes: [RegisterWrite; 4], terminator_address: SharedU32, terminator_opaque: SharedU32 } #[repr(C, align(4))] struct PhyInitRegisterWriteList2 { writes: [RegisterWrite; 2], terminator_address: SharedU32, terminator_opaque: SharedU32 } /* Retained code interprets lower and upper as signed 16-bit values. */ #[repr(C, align(2))] struct InitializedPhyGainSourceRecord { selector: SharedU8, opaque_01: SharedU8, lower: SharedU16, upper: SharedU16 } #[repr(C, align(4))] struct InitializedIqCalibrationGainIndices { entries: [SharedU32; 12] } #[repr(C, align(4))] struct MeasurementWorkspace { dwell_bound: SharedU16, opaque_02: OpaqueBytes<0x03>, measurement_type: SharedU8, opaque_06: OpaqueBytes<0x02>, completion_status: SharedU32, opaque_0c: OpaqueBytes<0x05>, dispatch_state: SharedU8, dispatch_argument: SharedU16, opaque_14: OpaqueBytes<0x04>, start_timestamp_words: [SharedU32; 2], elapsed_timestamp_words: [SharedU32; 2], scan_request_prefix: OpaqueBytes<0x01>, scan_request_mode: SharedU8, opaque_2a: OpaqueBytes<0x3e> } #[repr(C, align(4))] struct TxAggregateExpirationDelta { value: SharedU32 } #[repr(C, align(4))] struct DebugCommandDescriptor { command_name: SharedU32, help_text: SharedU32, handler: SharedU32 } #[repr(C, align(4))] struct InitializedDebugCommandDescriptors { records: [DebugCommandDescriptor; 6] } #[repr(C, align(4))] struct PhyWatchdogCounter { count: SharedU32 } #[repr(C, align(4))] struct InitializedMultiVifBeaconTimerTail { opaque_00: OpaqueBytes<0x0f>, interface_2_radio_latch: SharedU8, timer: TimerEntry, opaque_24: OpaqueBytes<0x18>, measurement_dwell_timer: TimerEntry, dtim_capture_latch: SharedU8, opaque_51: OpaqueBytes<0x03>, measurement_control_word_0: SharedU32, measurement_control_word_1: SharedU32, measurement_control_word_2: SharedU32 } #[repr(C, align(4))] struct RetryPathCounter { count: SharedU32 } #[repr(C, align(4))] struct PerTidTelemetryBank { table_00: [SharedU32; 8], table_01: [SharedU32; 8], table_02: [SharedU32; 8], table_03: [SharedU32; 8], table_04: [SharedU32; 8], table_05: [SharedU32; 8], table_06: [SharedU32; 8], table_07: [SharedU32; 8], table_08: [SharedU32; 8], table_09: [SharedU32; 8] } #[repr(C, align(4))] struct TxConfirmAggregationState { state: SharedU32, pending_message_raw: SharedU32, append_cursor_raw: SharedU32 } #[repr(C, align(4))] struct ConfigurationApplyFlags { flags: SharedU32 } #[repr(C, align(4))] struct DebugPlatformLocalTail { platform_local_word_2c: SharedU32, platform_local_word_30: SharedU32 } #[repr(C, align(4))] struct PreHostPasRingObserved { opaque_00: OpaqueBytes<0x06>, radio_stop_word_02: SharedU16, opaque_08: OpaqueBytes<0x04> } #[repr(C, align(2))] struct RfModeHalfwordTable { entries: [SharedU16; 36] } #[repr(C, align(4))] struct PhyCalSubstateRegisterWriteList { writes: [RegisterWrite; 8], terminator_address: SharedU32, terminator_opaque: SharedU32 } #[repr(C, align(4))] struct DbgExpandRegisterWriteList { writes: [RegisterWrite; 5], terminator_address: SharedU32, terminator_opaque: SharedU32 } #[repr(C, align(4))] struct RatePairTable { pairs: [[SharedU8; 2]; 31], opaque_3e: OpaqueBytes<0x02> } #[repr(C, align(4))] struct SchedulerTail { opaque_2018: OpaqueBytes<0x10>, hardware_timer_guard: SharedU32, rf_calibration_bytes: [SharedU8; 32], opaque_204c: OpaqueBytes<0x04>, error_event_counts: [SharedU32; 10] } #[repr(C, align(4))] struct MacAggregateSlotTables { queues: [[SharedU32; 16]; 8] } #[repr(C, align(4))] struct MacPipeTail { setup_word_6dc: SharedU16, setup_word_6de: SharedU16, setup_word_6e0: SharedU16, opaque_6e2: OpaqueBytes<0x02>, opaque_6e4: OpaqueBytes<0x1c>, type_byte_700: SharedU8, opaque_701: OpaqueBytes<0x03>, cleared_word_704: SharedU32, counter_word_708: SharedU32, counter_word_70c: SharedU32, counter_word_710: SharedU32, counter_word_714: SharedU32, frame_count_718: SharedU32, scratch_word_71c: SharedU32 } #[repr(C, align(4))] struct MacPipeTails { records: [MacPipeTail; 4] }
/// Vendor COPY image with exact initialized-data islands represented at their
/// qualified offsets. Bytes between islands remain occupied opaque data.
#[repr(C, align(4))]
struct InitializedVendorImage {
    pre_duration_tables: OpaqueBytes<0x138>,
    tx_duration_timing: [SharedU16; 10],
    pre_rate_tables: OpaqueBytes<0x48>,
    rate_encoding: [SharedU8; 22],
    rate_attributes: [SharedU8; 22],
    rate_pair_table: RatePairTable,
    initialized_rate_policies: InitializedRatePolicies,
    pre_completion_callback_words: OpaqueBytes<0x38>,
    /// Ten visible words in the qualified initialized island. The evidence does
    /// not establish that every word is a complete callable entry.
    visible_completion_words: [SharedU32; 10],
    pre_ring_cursor_map: OpaqueBytes<0x50>, queue_pipe_mappings: QueuePipeMappings,
    pre_tkip_sbox_tables: OpaqueBytes<0x2c>,
    tkip_sbox_tables: TkipSboxTables,
    command_dispatch: [SharedU32; 37],
    pre_aes_descriptors: OpaqueBytes<0x60>,
    aes_transfer_classes: AesTransferClassTable,
    aes_mode1_microcode: OpaqueBytes<0x1ae>,
    pre_phy_gain_register_write_lists: OpaqueBytes<0x182>,
    phy_gain_register_write_lists: [RegisterWriteList; 2],
    phy_init_register_write_list_0: PhyInitRegisterWriteList0, phy_init_register_write_list_1: PhyInitRegisterWriteList1, phy_init_register_write_list_2: PhyInitRegisterWriteList2, post_phy_init_register_write_lists_prefix: OpaqueBytes<0x46>, initialized_phy_gain_source_records: [InitializedPhyGainSourceRecord; 43], post_initialized_phy_gain_source_records: OpaqueBytes<0x28>, rf_mode_halfword_table: RfModeHalfwordTable, initialized_iq_calibration_gain_indices: InitializedIqCalibrationGainIndices, /* Retained rf_write_iq_corr_regs performs an unchecked u32 lookahead reading the first address word of the phy-cal substate list. */ phy_cal_substate_register_write_list: PhyCalSubstateRegisterWriteList, dbg_expand_register_write_list: DbgExpandRegisterWriteList, register_write_lists_suffix: OpaqueBytes<0x214>,
    duration_quantum_pointers: [SharedU32; 4],
    pre_measurement_workspace: OpaqueBytes<0x14>, measurement_workspace: MeasurementWorkspace, tx_aggregate_expiration_delta: TxAggregateExpirationDelta, debug_command_descriptors: InitializedDebugCommandDescriptors,
    hif_control: InitializedHifControl,
    irq_callbacks: [SharedU32; 32],
    phy_watchdog_counter: PhyWatchdogCounter, multi_vif_beacon_timer_tail: InitializedMultiVifBeaconTimerTail,
    ampdu_counters: AmpduTelemetryCounters,
    retry_path_counter: RetryPathCounter, per_tid_telemetry_bank: PerTidTelemetryBank,
    ampdu_completion_control: AmpduCompletionControl,
    tx_confirm_aggregation_state: TxConfirmAggregationState, configuration_apply_flags: ConfigurationApplyFlags,
    control_words: InitializedControlWords,
    pre_phy_channel_threshold_descriptors: OpaqueBytes<0x14>, debug_platform_local_tail: DebugPlatformLocalTail,
    phy_channel_threshold_descriptors: [PhyChannelThresholdDescriptor; 2],
    phy_gain_programming_records: [PhyGainProgrammingRecord; 16],
    pre_host_pas_ring: PreHostPasRingObserved,
    host_pas_ring: HostPasRing,
    initialized_low_mac_prefix: LowMacGlobalPrefix,
    mac_pipe_records: [MacPipeRecord; 4],
    mac_tx_queue_state: MacTxQueueState,
    pre_mac_beacon_state: OpaqueBytes<0x1a8>,
    mac_beacon_state: MacBeaconState,
    mac_wake_runtime_state: MacWakeRuntimeState,
    pre_mac_phy_command_state: OpaqueBytes<0x08>, mac_aggregate_slot_tables: MacAggregateSlotTables,
    mac_phy_command_state: MacPhyCommandState,
    mac_pipe_tails: MacPipeTails,
    mac_retry_hardware_state: MacRetryHardwareState,
    pre_mac_runtime_accounting: OpaqueBytes<0x108>,
    mac_runtime_accounting: MacRuntimeAccountingState,
    scheduler_exclusion_state: SchedulerExclusionState,
    scheduler_event_island: SchedulerEventIsland,
    initialized_tail: SchedulerTail,
}
#[repr(C, align(4))]
struct InitializedRatePolicies { policies: [[SharedU32; 5]; 2] }
#[repr(C, align(4))]
struct QueuePipeMappings {
    pipe_order: [SharedU8; 4],
    queue_to_access_category: [SharedU8; 4],
    access_category_to_queue: [SharedU8; 4],
}
#[repr(C, align(4))]
struct InitializedControlWords {
    beacon_state: SharedU32,
    rx_indication_state: SharedU32,
    tsf_resync_state: SharedU32,
    random_lfsr: SharedU32,
    tsf_accumulator_low: SharedU32,
    opaque_14: SharedU32,
    opaque_18: SharedU32,
    timer_counter: SharedU32,
}
#[repr(C, align(4))]
struct AmpduCompletionControl { enabled: SharedU32 }
#[repr(C, align(4))]
struct AmpduTelemetryCounters {
    tx_error_frames: SharedU32,
    tx_counted_frames: SharedU32,
    tx_duration_low: SharedU32,
    tx_duration_high: SharedU32,
    rx_management_0: SharedU32,
    rx_management_1: SharedU32,
    rx_management_2: SharedU32,
    rx_management_3: SharedU32,
    opaque_20: SharedU32,
    tx_retry_count: SharedU32,
}
#[repr(C, align(4))]
struct PhyChannelThresholdDescriptor { opaque_00: SharedU8, count: SharedU8, default_threshold: SharedU16, records: SharedU32 }
#[repr(C, align(4))]
struct PhyGainProgrammingRecord { rate: SharedU8, opaque_01: SharedU8, requested_offset: SharedU16, selected_power: SharedU16, reserved_06: SharedU16, cleared_word: SharedU32, gain_code: SharedU16, rssi_value: SharedU16 }
#[repr(C, align(4))]
struct InitializedHifControl {
    queued_depth: SharedU32,
    pending_count: SharedU32,
    coalesce_enabled: SharedU8,
    pending_threshold: SharedU8,
    ring_depth_threshold: SharedU8,
    count_threshold: SharedU8,
    coalesce_delay: SharedU32,
}
#[repr(C, align(4))]
struct HostPasRing { head: SharedU32, tail: SharedU32, slots: [SharedU32; 64] }
#[repr(C, align(4))]
struct LowMacGlobalPrefix { fifo_control: SharedU8, fifo_status: SharedU8, rate_config: SharedU16, control_04: SharedU8, legacy_mode: SharedU8, event_pending: SharedU8, pipe_busy: SharedU8, controller_config: SharedU16, control_0a: SharedU8, control_0b: SharedU8, selected_rate: SharedU8, opaque_0d: OpaqueBytes<0x03>, producer: SharedU32, producer_mirror: SharedU32, state_18: SharedU32, slot_time_base: SharedU32, slot_time_initial: SharedU32, slot_time_x1: SharedU32, slot_time_x2: SharedU32, slot_time_x3: SharedU32, slot_time_constant: SharedU32, slot_time_x8: SharedU32, slot_time_x16: SharedU32, slot_time_x24: SharedU32, opaque_40: SharedU32, ifs_duration: SharedU32, short_airtimes: [SharedU16; 22], long_airtimes: [SharedU16; 22] }
#[repr(C, align(4))]
struct MacPipeSlot { state_word: SharedU32, opaque_04: OpaqueBytes<0x08>, frame: SharedU32, auxiliary: SharedU32, command: SharedU32 }
#[repr(C, align(4))]
struct MacPipeRecord { current_slot: SharedU8, opaque_01: OpaqueBytes<0x02>, state: SharedU8, opaque_04: SharedU32, hardware_ring: SharedU32, slots: [MacPipeSlot; 4] }
#[repr(C, align(4))]
struct MacTxQueueState { head: SharedU32, tail: SharedU32 }
#[repr(C, align(4))]
struct MacBeaconState { opaque_00: OpaqueBytes<0x08>, response_commands: [SharedU32; 2], opaque_10: OpaqueBytes<0x18>, state: SharedU32, secondary_command: SharedU32, control: SharedU32, selector: SharedU32, mode: SharedU8, opaque_39: OpaqueBytes<0x03>, completion_word: SharedU32 }
#[repr(C, align(4))]
struct MacWakeRuntimeState { opaque_00: OpaqueBytes<0x08>, timer: TimerEntry, phy_state: SharedU8, transition_pending: SharedU8, restore_pending: SharedU8, opaque_1f: SharedU8, opaque_20: SharedU32, mode: SharedU32, control: SharedU32, retry_rate_map: [SharedU8; 22], opaque_42: OpaqueBytes<0x02>, edca_slot_timing: SharedU32 }
#[repr(C, align(4))]
struct MacPhyCommandState { opaque_00: OpaqueBytes<0x02>, radio_stop_state: SharedU8, opaque_03: SharedU8, sideband_capture: SharedU32, timer: TimerEntry, operation_state: SharedU32, operation_command: SharedU8, opaque_21: OpaqueBytes<0x07>, operation_output_state: SharedU8, opaque_29: OpaqueBytes<0x03>, operation_timeout: SharedU32, dispatch_command: [SharedU8; 8], dispatch_output_state: SharedU8, dispatch_output_flags: SharedU8, opaque_3a: OpaqueBytes<0x02>, dispatch_output_timeout: SharedU32, completion_status: SharedU32, opaque_44: SharedU32, interface: SharedU8, opaque_49: OpaqueBytes<0x03> }
#[repr(C, align(4))]
struct MacRetryHardwareState { control: SharedU8, opaque_01: OpaqueBytes<0x03> }
#[repr(C, align(4))]
struct SoftwareRecordNode { next: SharedU32, packet_record: SharedU32 }
#[repr(C, align(4))]
struct SoftwareRecordFreeList { head: SharedU32, nodes: [SoftwareRecordNode; 4] }
#[repr(C, align(4))]
struct MacRuntimeAccountingState { current_pipe: SharedU32, status_accounting: SharedU32, sample_count: SharedU32, current_pipe_record: SharedU32, current_slot: SharedU32, pipe_event_flags: [SharedU8; 4], software_records: SoftwareRecordFreeList, average: SharedU16, opaque_3e: OpaqueBytes<0x06>, silicon_control: SharedU8, opaque_45: OpaqueBytes<0x03>, parameter0: SharedU32, parameter1: SharedU32, opaque_50: SharedU32 }
#[repr(C, align(4))]
struct SchedulerExclusionState { exclusion_mask: SharedU32, secondary_exclusion: SharedU32 }
#[repr(C, align(4))]
struct SchedulerEventIsland { pending_events: SharedU32, runtime_flags: SharedU32, opaque_08: OpaqueBytes<0x0a>, startup_mode: SharedU16, opaque_14: OpaqueBytes<0x08>, analog_enabled: SharedU16, opaque_1e: OpaqueBytes<0x02>, remap_primary: SharedU32, opaque_24: OpaqueBytes<0x04>, remap_secondary: SharedU32, analog_words: [SharedU32; 3], opaque_38: OpaqueBytes<0x08>, timer_list_head: SharedU32 }

#[repr(C, align(4))]
struct RuntimeRegisterBackoffState { register_context: [SharedU32; 4], override_enabled: SharedU32, override_window: SharedU32, opaque_18: SharedU32 }
#[repr(C, align(4))]
struct DebugConsoleState { input_length: SharedU32, flags: SharedU32, timer: TimerEntry, memory_address: SharedU32, memory_value: SharedU32, command_count: SharedU32, commands: [SharedU32; 32], line_buffer: [SharedU8; 80] }
#[repr(C, align(4))]
struct RuntimePrefix { register_backoff: RuntimeRegisterBackoffState, debug_console_state: DebugConsoleState }
#[repr(C, align(4))]
struct ClockParameterIsland { opaque_00: SharedU32, mac_clock_snapshot: SharedU32, beacon_counter_snapshot: SharedU32, hardware_counter_cache: SharedU32, opaque_10: SharedU32, conversion_factor: SharedU32, opaque_18: SharedU32, conversion_mode: SharedU8, opaque_1d: OpaqueBytes<0x03>, opaque_20: SharedU32, correction_offset: SharedU32 }

/// Common retained intrusive timer entry initialized by `timer_entry_init`.
#[repr(C, align(4))]
struct TimerEntry { next: SharedU32, previous_link: SharedU32, deadline: SharedU32, callback: SharedU32, context: SharedU32 }

/// Reverse-indexed scheduler/IRQ callback words. Values are quarantined raw
/// addresses because not every callback target has native Rust ownership.
#[repr(C, align(4))]
struct SchedulerHandlerTable {
    handlers: [SharedU32; 32],
}

#[repr(C, align(2))]
struct PhyGainSourceRecord { selector: SharedU8, opaque_01: SharedU8, lower: SharedU16, upper: SharedU16 }
#[repr(C, align(4))]
struct TemplateFrameDescriptor { kind_00: SharedU8, flags_01: SharedU8, opaque_02: OpaqueBytes<0x02>, buffer_04: SharedU32, kind_08: SharedU8, flags_09: SharedU8, opaque_0a: OpaqueBytes<0x02>, pointer_0c: SharedU32, kind_10: SharedU8, flags_11: SharedU8, length_12: SharedU16, opaque_14: OpaqueBytes<0x04>, kind_18: SharedU8, flags_19: SharedU8, length_1a: SharedU16, opaque_1c: OpaqueBytes<0x04>, kind_20: SharedU8, flags_21: SharedU8, length_22: SharedU16, opaque_24: OpaqueBytes<0x04>, kind_28: SharedU8, flags_29: SharedU8, opaque_2a: OpaqueBytes<0x02>, pointer_2c: SharedU32, kind_30: SharedU8, opaque_31: OpaqueBytes<0x03>, pointer_34: SharedU32, kind_38: SharedU8, flags_39: SharedU8, length_3a: SharedU16, pointer_3c: SharedU32 }
/// Overlapping address schema used by retained RF initialization. The logical
/// view crosses beacon/filter storage and therefore is not embedded as an owner.
#[repr(C, align(4))]
struct RfInitializationObservedLayout { control_minus_ac: SharedU32, opaque_04: OpaqueBytes<0x04>, control_minus_a4: SharedU32, opaque_0c: OpaqueBytes<0x08>, table_minus_98: SharedU32, table_minus_94: SharedU32, control_minus_90: SharedU32, opaque_20: OpaqueBytes<0x38>, negative_words: [SharedU32; 7], opaque_74: OpaqueBytes<0x38>, root_prefix: OpaqueBytes<0x14>, table_14: SharedU32, table_18: SharedU32, table_1c: SharedU32, table_20: SharedU32, pointer_24: SharedU32, opaque_d4: OpaqueBytes<0x08>, control_30: SharedU32, opaque_e0: OpaqueBytes<0x04>, pointer_38: SharedU32 }
#[repr(C, align(4))]
struct BeaconIeOffsetIndex { count: SharedU32, offsets: [SharedU16; 256] }
#[repr(C, align(4))]
struct BeaconFilterStorage { stored_length: SharedU32, stored_beacon: [SharedU8; 700], active_index: SharedU32, indexes: [BeaconIeOffsetIndex; 2] }
#[repr(C, align(4))]
struct TemplateBackingStorage { primary: [[SharedU8; 0x100]; 2], secondary: [[SharedU8; 0x60]; 2], tertiary: [[SharedU8; 0x90]; 2] }
#[repr(C, align(4))]
struct PreConfigurationTables { gain_source_records: [PhyGainSourceRecord; 22], beacon_filter: BeaconFilterStorage, opaque_750: OpaqueBytes<0x06cc>, template_descriptors: [TemplateFrameDescriptor; 2], template_backing: TemplateBackingStorage }
#[repr(C)]
struct SddChannelRecord { bytes: [SharedU8; 3] }
#[repr(C, align(2))]
struct SddProfileBank { rate_limits: [SharedU16; 11], channel_records: [SddChannelRecord; 16], channel_count: SharedU8, opaque_47: SharedU8, agc_correction: SharedU16, calibration_coefficient: SharedU16, conversion_pair: [SharedU16; 2], rssi_coefficients: [SharedU16; 2], rssi_rate_scales: [SharedU16; 11], opaque_6a: OpaqueBytes<0x28> }
#[repr(C, align(4))]
struct SddConfigurationTables { profiles: [SddProfileBank; 2], opaque_124: OpaqueBytes<0x0c> }
#[repr(C, align(4))]
struct WakeContextState { clock_words: [SharedU32; 4], response_pointers: [SharedU32; 32] }
#[repr(C, align(2))]
struct DurationSources { values: [SharedU16; 2] }
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
/// Link/aggregation header immediately preceding the VIF array.
#[repr(C, align(4))]
struct PreVifHeader {
    opaque_00: OpaqueBytes<0x18>,
    link_bitmap: SharedU32,
    opaque_1c: OpaqueBytes<0x04>,
}

/// One physical VIF ABI record.
///
/// This is a layout description, not an ownership declaration. Retained vendor
/// routines and IRQ/FIQ code remain production-reachable writers, so decoded
/// scalars deliberately use shared wrappers and are accessed only through
/// volatile, field-derived addresses. In particular, no `&VifRecord` or
/// `&mut VifRecord` is ever created on target.
///
/// The byte at `+0x16` is also reached by vendor wake code as `+0x3c6` from the
/// preceding record. Keeping it as `wake_reinit_flag` makes that cross-record
/// overlay explicit. `rate_configuration` is an eight-byte overlay: its low
/// word is written as a unit while bytes `+2..+7` are independently consumed.
#[repr(C, align(4))]
struct VifRecord {
    scan_rate_config: SharedU16,
    scan_channel: SharedU16,
    scan_flags: SharedU8,
    reserved_05: OpaqueBytes<0x1>,
    host_contexts_in_flight: SharedU8,
    reserved_07: OpaqueBytes<0x0f>,
    wake_reinit_flag: SharedU8,
    reserved_17: OpaqueBytes<0x1>,
    mode: SharedU8,
    active: SharedU8,
    interface: SharedU8,
    role: SharedU8,
    flags: SharedU32,
    rate_configuration: OpaqueBytes<0x8>,
    basic_rates: SharedU32,
    allowed_links: SharedU16,
    effective_links: SharedU16,
    tx_busy: SharedU16,
    reserved_32: OpaqueBytes<0x2>,
    own_mac: [SharedU8; 0x6],
    reserved_3a: OpaqueBytes<0x2>,
    bssid: [SharedU8; 0x6],
    channel: SharedU16,
    radio_owner_overlay: OpaqueBytes<0x0c>,
    operating_state: SharedU8,
    owner_interface: SharedU8,
    owner_channel: SharedU16,
    owner_deadline: SharedU32,
    reserved_58: OpaqueBytes<0x4>,
    owner_flags: SharedU32,
    reserved_60: OpaqueBytes<0x6>,
    activity_state: SharedU8,
    opaque_067: OpaqueBytes<0x49>,
    operating_timers: [TimerEntry; 3],
    ssid_length: SharedU32,
    ssid: [SharedU8; 0x20],
    dtim_period: SharedU8,
    reserved_111: OpaqueBytes<0x5>,
    atim_window: SharedU16,
    beacon_interval: SharedU32,
    reserved_11c: OpaqueBytes<0x8>,
    rts_threshold: SharedU32,
    ampdu_length: SharedU16,
    internal_link: SharedU16,
    default_rates: [SharedU8; 0x2],
    reserved_12e: OpaqueBytes<0x0e>,
    link_object_flags: SharedU32,
    link_own_mac_0: [SharedU8; 0x6],
    link_own_mac_1: [SharedU8; 0x6],
    link_bssid: [SharedU8; 0x6],
    reserved_152: OpaqueBytes<0x0a>,
    sleeping_links: SharedU16,
    awake_links: SharedU16,
    buffered_links: SharedU16,
    reserved_162: OpaqueBytes<0x2>,
    link_gate: SharedU8,
    opaque_165: OpaqueBytes<0x1f>,
    link_timers: [TimerEntry; 2],
    opaque_1ac: OpaqueBytes<0x204>,
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

/// Address stored in a host context for the original borrowed HIF request.
///
/// The value remains a raw ABI word: this type distinguishes its address domain
/// but does not dereference it or transfer the `RequestBuffer` owner's lifetime.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HifRequestAddress(u32);

/// Packet-RAM address stored in a host context for an MPDU or its dedicated
/// per-context descriptor/work area.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PacketRamAddress(u32);

/// Address of one fixed-DTCM host-link mapping record.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LinkMapEntryAddress(DtcmAddress);

/// The decoded `ctx+0x54` PAS/frame-node overlay.
///
/// Names are limited to fields exercised by translated Rust or directly
/// supported by retained vendor evidence. The remaining bytes preserve live
/// overlays used by retry, encryption, aggregation, and completion code.
#[repr(C, align(4))]
struct HostPasContext {
    frame_address: SharedScalar<PacketRamAddress>, // +0x00 / outer +0x54
    control_bits: SharedU32,                       // +0x04
    frame_length: SharedU16,                       // +0x08
    frame_control: SharedU16,                      // +0x0a
    access_category: SharedU8,                     // +0x0c
    request_flag_rate_bits: SharedU8,              // +0x0d; request flags bits 1..3
    retry_policy: SharedU8,                        // +0x0e
    tx_rate: SharedU8,                             // +0x0f
    expiry_time: SharedU32,                        // +0x10
    completion_timestamp: SharedU32,               // +0x14
    scheduler_timestamp: SharedU32,                // +0x18
    terminal_status: SharedU16,                    // +0x1c
    try_count: SharedU16,                          // +0x1e
    opaque_20: OpaqueBytes<0x0c>,
    ownership_bits: SharedU32,                     // +0x2c
    opaque_30: OpaqueBytes<0x06>,
    duration: SharedU16,                           // +0x36
    opaque_38: OpaqueBytes<0x04>,
    descriptor_state: SharedU32,                   // +0x3c
    opaque_40: OpaqueBytes<0x08>,
    word_48: SharedU32,                            // +0x48; unresolved timing/accounting word
    frame_state_address: SharedScalar<PacketRamAddress>, // +0x4c
    auxiliary_state: SharedU16,                    // +0x50
    tid: SharedU8,                                 // +0x52
    insertion_mode: SharedU8,                      // +0x53
    sequence_number: SharedU16,                    // +0x54
    retry_rate: SharedU8,                          // +0x56
    byte_57: SharedU8,                             // +0x57; unresolved retry/encoding overlay
    opaque_58: OpaqueBytes<0x11>,
    interface: SharedU8,                           // +0x69
    duration_slot: SharedU8,                       // +0x6a
    host_link: SharedU8,                           // +0x6b
    completion_byte_6c: SharedU8,                  // +0x6c; copied into completion metadata
    opaque_6d: OpaqueBytes<0x07>,
    qos_control: SharedU16,                        // +0x74
    cipher_class: SharedU8,                        // +0x76
    opaque_77: OpaqueBytes<0x05>,
    word_7c: SharedU16,                            // +0x7c; unresolved crypto tail overlay
    opaque_7e: OpaqueBytes<0x02>,
}

/// One exact host WSM TX context at fixed DTCM identity.
///
/// This is a semantic ABI description, not a Rust ownership declaration.
/// Foreground code, retained callbacks, diagnostics, and IRQ/FIQ completion can
/// all access a live record. No ordinary reference to this type is exposed;
/// production access is field-derived raw volatile I/O, with multiword/list
/// transitions serialized by the MAC domain or explicit IRQ/FIQ exclusion.
#[repr(C, align(4))]
struct HostTxContext {
    request_buffer: SharedScalar<HifRequestAddress>, // +0x00
    intrusive_next: SharedU32,                       // +0x04
    packet_id: SharedU32,                            // +0x08
    requested_rate: SharedU8,                        // +0x0c
    queue_id: SharedU8,                              // +0x0d
    more: SharedU8,                                  // +0x0e
    request_flags: SharedU8,                         // +0x0f
    expiry_time: SharedU32,                          // +0x10
    ht_tx_parameters: SharedU32,                     // +0x14
    borrowed_frame_length: SharedU32,                // +0x18
    borrowed_frame_address: SharedScalar<PacketRamAddress>, // +0x1c
    completion_status: SharedU32,                    // +0x20
    rate_copy: SharedU8,                             // +0x24; duplicate request max-rate byte
    saved_status: SharedU8,                          // +0x25
    completion_flags: SharedU16,                     // +0x26
    rate_try: [SharedU32; 3],                        // +0x28
    opaque_34: OpaqueBytes<0x04>,
    timing_scratch: [SharedU32; 2],                  // +0x38
    submit_timer: SharedU32,                         // +0x40
    header_length: SharedU32,                        // +0x44
    payload_length: SharedU32,                       // +0x48
    optional_pipe_object: SharedU32,                 // +0x4c
    sequence_or_callback_state: SharedU16,           // +0x50
    submit_state: SharedU8,                          // +0x52; initialized to one at submit
    completion_class: SharedU8,                      // +0x53
    pas: HostPasContext,                             // +0x54
    /// Includes the crypto callback interior root at outer `+0x110` and all
    /// unresolved encryption/aggregation tail overlays through `+0x170`.
    opaque_d4: OpaqueBytes<0x9c>,
}

#[repr(C, align(4))]
struct HostTxContexts {
    contexts: [HostTxContext; HOST_TX_CONTEXT_COUNT],
}

#[repr(C, align(4))]
struct PeerPipeEntry { peer_mac: [SharedU8; 6], state_flags: SharedU8, age: SharedU8 }
#[repr(C, align(4))]
struct PreCommandQuarantine { peer_pipes: [PeerPipeEntry; 8], management_counters: [SharedU16; 4], scan_channel: SharedU16, opaque_4a: OpaqueBytes<0x02>, pending_root: SharedU32 }


/// Command-15 blob whose last four bytes overlap channel-switch control.
/// Later bytes are shared by JOIN, scan, register-save, and TX-buffer state.
#[repr(C, align(4))]
struct CommandChannelSwitchOverlay {
    upload_prefix: OpaqueBytes<0x64>, overlay_head: SharedU32, channel_switch_active: SharedU8, interface: SharedU8, opaque_6a: OpaqueBytes<0x02>, mode: SharedU8, countdown: SharedU8, channel: SharedU16,
    join_mode: SharedU8, join_flags: SharedU8, saved_register_context: SharedU16, rate_configuration: SharedU32, scan_state: SharedU8, scan_flags: SharedU8, opaque_7a: OpaqueBytes<0x01>, tx_buffer_free_count: SharedU8, scan_word: SharedU32, tail_word: SharedU32,
}

#[repr(C, align(4))]
struct DuplicateCacheEntry { peer_mac: [SharedU8; 6], identity: SharedU16, context: SharedU32 }
#[repr(C, align(4))]
struct LmcControlRoots { join_match_state: SharedU32, encryption_free_head: SharedU32, encryption_generation: SharedU32, duplicate_cache_prefix: [DuplicateCacheEntry; 31] }

/// Mixed accounting rooted at `0x04008798`.
///
/// Only direct vendor consumers are named. The auxiliary free list is distinct
/// from the host WSM context list at the following word and is not absorbed by
/// host-pool operations.
#[repr(C, align(4))]
struct HostContextAccounting {
    opaque_00: OpaqueBytes<0x0c>,
    duplicate_cache_cursor: SharedU32,
    deferred_event_owner: SharedU32,
    auxiliary_tx_buffer_free_head: SharedU32,
}

/// Host TX free-list root plus one adjacent unresolved word.
#[repr(C, align(4))]
struct HostContextFreeList {
    free_head: SharedU32,
    adjacent_state: SharedU32,
}

pub const LINK_MAP_ENTRY_COUNT: usize = 16;
pub const INTERNAL_LINK_SLOT_COUNT: usize = 10;
pub const LINK_TID_COUNT: usize = 16;

/// Header shared by host-link mapping and PAS release gating.
#[repr(C, align(4))]
struct LinkMapHeader {
    opaque_00: OpaqueBytes<0x14>,
    entry_count: SharedU16,           // +0x14
    release_blocked_links: SharedU16, // +0x16
}

/// One host-link to internal-link mapping record.
#[repr(C, align(4))]
struct LinkMapEntry {
    host_link: SharedU8,        // +0x00
    interface: SharedU8,        // +0x01
    internal_link: SharedU8,    // +0x02
    inactivity: SharedU8,       // +0x03
    release_flags: SharedU8,    // +0x04
    auxiliary_flags: SharedU8,  // +0x05
    mac_address: [SharedU8; 6], // +0x06
}

/// Link-map records, ten internal-link/TID sequence rows, and allocation state.
#[repr(C, align(4))]
struct LinkAndSequenceState {
    header: LinkMapHeader,
    entries: [LinkMapEntry; LINK_MAP_ENTRY_COUNT],
    sequences: [[SharedU16; LINK_TID_COUNT]; INTERNAL_LINK_SLOT_COUNT],
    internal_link_bitmap: SharedU16,
    opaque_tail: OpaqueBytes<0x06>,
}
#[repr(C, align(4))]
struct JoinScanControl { schedule_word: SharedU32, schedule_deadline: SharedU32, beacon_timer_active: SharedU8, beacon_interface: SharedU8, opaque_0a: OpaqueBytes<0x02>, channel_owner: SharedU32, opaque_10: OpaqueBytes<0x04>, channel_use_state: SharedU32, alternate_channel_owner: SharedU32, opaque_1c: OpaqueBytes<0x04>,
    join_timer: TimerEntry, join_status: SharedU16, start_state: SharedU8, interface_state: SharedU8, response_status: SharedU32, request_word: SharedU32 }
#[repr(C, align(4))]
struct WsmResponseScratch {
    scan_control: OpaqueBytes<0x0c>, request_pointers: [SharedU32; 30],
    request_status_prefix: [SharedU8; 28],
}


/// BA/LMC request accounting and protocol flags shared with retained code.
#[repr(C, align(4))]
struct BaLmcHeader {
    opaque_00: OpaqueBytes<0x02>, request_slot_index: SharedU8, pending_request_count: SharedU8,
    request_sequence: SharedU32, request_word_08: SharedU32, request_word_0c: SharedU32,
    request_meta_10: OpaqueBytes<0x04>, tim_flags: SharedU16, request_flags: SharedU16,
    join_retry_state: SharedU8, scan_state: SharedU8, opaque_1a: OpaqueBytes<0x01>,
    ba_policy_enabled: SharedU8, opaque_1c: OpaqueBytes<0x04>,
}
/// Pending TX list, radio arbitration, timers, and LMC ring cursors.
#[repr(C, align(4))]
struct PendingBaLmcState {
    pending_head: SharedU32, pending_tail: SharedU32, mac_bssid_mode: SharedU16, opaque_0a: OpaqueBytes<0x01>, pending_service_needed: SharedU8, opaque_0c: OpaqueBytes<0x3c>,
    radio_owner: SharedU32, radio_wait_head: SharedU32, opaque_50: OpaqueBytes<0x04>, deferred_radio_owner: SharedU32, opaque_58: OpaqueBytes<0x64>,
    radio_role_state: SharedU8, radio_timer_state: SharedU8, radio_timer_interface: SharedU8, power_state_complete: SharedU8, radio_timer_deadline: SharedU32, radio_timer_sample: SharedU32,
    opaque_c8: OpaqueBytes<0x08>, message_control: SharedU8, ba_session_count: SharedU8, ba_active_count: SharedU8, message_producer: SharedU8, message_consumer: SharedU8, opaque_d5: OpaqueBytes<0x0b>,
}
#[repr(C, align(4))]
struct LmcMessage { kind: SharedU8, flags: SharedU8, opaque_02: OpaqueBytes<0x02>, payload: OpaqueBytes<0x24>, interface: SharedU8, completion_state: SharedU8, opaque_2a: OpaqueBytes<0x02> }
#[repr(C, align(4))]
struct LmcMessages { records: [LmcMessage; LMC_MESSAGE_COUNT] }
#[repr(C, align(4))]
struct BaSession { activity: SharedU32, peer_mac: [SharedU8; 6], tid: SharedU8, interface: SharedU8, opaque_0c: OpaqueBytes<0x06>, timeout_1024us: SharedU16, timer: TimerEntry }
#[repr(C, align(4))]
struct BaSessions { records: [BaSession; 4] }

#[repr(C, align(4))]
struct BaLinkEventState { ba_deferred_action: SharedU8, ba_deferred_interface: SharedU8, opaque_02: OpaqueBytes<0x01>, periodic_timer_enabled: SharedU8,
    periodic_timer: TimerEntry, transition_timer: TimerEntry, current_network_flags: SharedU8, accumulated_network_flags: SharedU8,
    changed_network_flags: SharedU8, opaque_2f: OpaqueBytes<0x01> }


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
    root: SharedU32,
    external_count: SharedU8,
    class0_count: SharedU8,
    opaque_06: SharedU16,
    allocation_state: SharedU16,
    pending_count: SharedU16,
    coalesce_state: SharedU32,
    free_state: SharedU32,
}

/// Retained 64-entry completion ring. Its logical extent crosses the physical
/// pre-context/prefix boundary, so it remains an address-only shared view.
#[repr(C, align(4))]
struct CompletionRingObservedLayout { entries: [SharedU32; 64] }
/// Physical backing for completion-ring entries 0 through 58. The final five
/// entries overlap `InternalContextPrefix` and remain represented by the
/// address-only logical view above.
#[repr(C, align(4))]
struct PreInternalContextQuarantine { completion_entries: [SharedU32; 59] }
#[repr(C, align(4))]
struct InternalContextPrefix {
    iv_seed_or_completion_entry_59: SharedU32,
    completion_entries_60_63: [SharedU32; 4],
}

/// One exact internal TX context record. It remains quarantine because retained
/// teardown and diagnostics can mutate the same bytes.
#[repr(C, align(4))]
struct InternalPasContext {
    frame_address: SharedU32,
    control_bits: SharedU32,
    frame_length: SharedU16,
    frame_control: SharedU16,
    access_category: SharedU8,
    request_flag_rate_bits: SharedU8,
    retry_policy: SharedU8,
    tx_rate: SharedU8,
    expiry_time: SharedU32,
    completion_timestamp: SharedU32,
    scheduler_timestamp: SharedU32,
    terminal_status: SharedU16,
    try_count: SharedU16,
    opaque_20: OpaqueBytes<0x0c>,
    ownership_bits: SharedU32,
    opaque_30: OpaqueBytes<0x06>,
    duration: SharedU16,
    opaque_38: OpaqueBytes<0x04>,
    descriptor_state: SharedU32,
    opaque_40: OpaqueBytes<0x0c>,
    frame_state_address: SharedU32,
    auxiliary_state: SharedU16,
    tid: SharedU8,
    insertion_mode: SharedU8,
    sequence_number: SharedU16,
    retry_rate: SharedU8,
    byte_57: SharedU8,
    opaque_58: OpaqueBytes<0x11>,
    interface: SharedU8,
    duration_slot: SharedU8,
    host_link: SharedU8,
    completion_byte_6c: SharedU8,
    opaque_6d: OpaqueBytes<0x03>,
    cipher_buffer: SharedU32,
    qos_control: SharedU16,
    cipher_class: SharedU8,
    opaque_77: OpaqueBytes<0x05>,
    word_7c: SharedU16,
    opaque_7e: OpaqueBytes<0x02>,
}
#[repr(C, align(4))]
pub(crate) struct InternalTxContext {
    opaque_00: SharedU32,
    next_free: SharedU32,
    opaque_08: OpaqueBytes<0x14>,
    header_80211: SharedU32,
    completion_status: SharedU32,
    rate_copy: SharedU8,
    saved_status: SharedU8,
    completion_flags: SharedU16,
    opaque_28: OpaqueBytes<0x24>,
    optional_pipe_object: SharedU32,
    opaque_50: OpaqueBytes<0x03>,
    completion_class: SharedU8,
    pas: InternalPasContext,
    opaque_d4: OpaqueBytes<0x9c>,
}

/// Existing typed internal TX pool, now embedded in the complete DTCM layout.
#[repr(C, align(4))]
pub(crate) struct InternalContextPoolState {
    free_head: SharedU32,
    contexts: [InternalTxContext; INTERNAL_TX_CONTEXT_COUNT],
}

/// Overlapping address schema used by retained per-interface power-save code.
/// Its `0x140` aligned extent deliberately exceeds the observed `0x104` view stride.
#[repr(C, align(4))]
struct PowerSaveObservedLayout { wake_stats_phase: SharedU8, wake_stats_flag_01: SharedU8, wake_stats_flag_02: SharedU8, opaque_003: OpaqueBytes<0x03>, wake_duration: SharedU16, wake_register_min: SharedU32, wake_register_sum: SharedU32, wake_register_max: SharedU32, wake_elapsed_min: SharedU32, wake_elapsed_sum: SharedU32, wake_elapsed_max: SharedU32, tx_completion_state: SharedU32, next_tbtt: SharedU32, doze_state: SharedU8, requested_pm_mode: SharedU8, global_sleep_state: SharedU8, wake_stats_counter: SharedU8, opaque_02c: SharedU8, resume_state: SharedU8, opaque_02e: OpaqueBytes<0x02>, global_timer_duration: SharedU32, beacon_timing_reference: SharedU32, join_state_word: SharedU32, wake_lead_time: SharedU32, mode: SharedU8, active: SharedU8, tx_pending_state: SharedU8, timer_state: SharedU8, flags: SharedU16, sleep_transition_flags: SharedU16, wake_stats_timestamp: SharedU32, backoff_adjustment: SharedU32, reset_state: SharedU8, beacon_timing_valid: SharedU8, pending_control_kind: SharedU8, uapsd_state: SharedU8, pending: SharedU32, pending_flags: SharedU8, wake_reason: SharedU8, queue_mask: SharedU16, tx_followup_state: SharedU32, uapsd_interval: SharedU32, uapsd_timeout: SharedU32, uapsd_configuration: SharedU32, uapsd_restart_value: SharedU32, timers: [TimerEntry; 7], state_fc: SharedU8, tx_completion_pending: SharedU8, beacon_rx_state: SharedU8, beacon_rate: SharedU8, wake_timer_delay: SharedU32, beacon_airtime: SharedU32, pre_tbtt_offset: SharedU32, beacon_interval: SharedU16, backoff_interval: SharedU16, next_wake_deadline: SharedU32, tx_completion_state_114: SharedU8, wake_timer_active: SharedU8, opaque_116: SharedU8, beacon_timing_adjusted: SharedU8, duration_118: SharedU32, duration_11c: SharedU32, duration_120: SharedU32, mode_control_124: SharedU32, interval_128: SharedU32, maximum_backoff_12c: SharedU32, last_beacon_timestamp_130: SharedU32, counter_134: SharedU16, threshold_136: SharedU16, scan_completion_138: SharedU16, sleep_vote_count_13a: SharedU16, ps_mode_error_reported_13c: SharedU8, opaque_13d: OpaqueBytes<0x03> }

/// Physical `0x104` prefix at each observed power-save view start. The logical
/// view continues past this prefix and overlaps the next physical prefix.
#[repr(C, align(4))]
struct PowerSavePhysicalPrefix {
    wake_stats_phase: SharedU8,
    wake_stats_flag_01: SharedU8,
    wake_stats_flag_02: SharedU8,
    opaque_003: OpaqueBytes<0x03>,
    wake_duration: SharedU16,
    wake_register_min: SharedU32,
    wake_register_sum: SharedU32,
    wake_register_max: SharedU32,
    wake_elapsed_min: SharedU32,
    wake_elapsed_sum: SharedU32,
    wake_elapsed_max: SharedU32,
    tx_completion_state: SharedU32,
    next_tbtt: SharedU32,
    doze_state: SharedU8,
    requested_pm_mode: SharedU8,
    global_sleep_state: SharedU8,
    wake_stats_counter: SharedU8,
    opaque_02c: SharedU8,
    resume_state: SharedU8,
    opaque_02e: OpaqueBytes<0x02>,
    global_timer_duration: SharedU32,
    beacon_timing_reference: SharedU32,
    join_state_word: SharedU32,
    wake_lead_time: SharedU32,
    mode: SharedU8,
    active: SharedU8,
    tx_pending_state: SharedU8,
    timer_state: SharedU8,
    flags: SharedU16,
    sleep_transition_flags: SharedU16,
    wake_stats_timestamp: SharedU32,
    backoff_adjustment: SharedU32,
    reset_state: SharedU8,
    beacon_timing_valid: SharedU8,
    pending_control_kind: SharedU8,
    uapsd_state: SharedU8,
    pending: SharedU32,
    pending_flags: SharedU8,
    wake_reason: SharedU8,
    queue_mask: SharedU16,
    tx_followup_state: SharedU32,
    uapsd_interval: SharedU32,
    uapsd_timeout: SharedU32,
    uapsd_configuration: SharedU32,
    uapsd_restart_value: SharedU32,
    timers: [TimerEntry; 7],
    state_fc: SharedU8,
    tx_completion_pending: SharedU8,
    beacon_rx_state: SharedU8,
    beacon_rate: SharedU8,
    wake_timer_delay: SharedU32,
}
#[repr(C, align(4))]
struct PowerSaveFamily { physical_prefixes: [PowerSavePhysicalPrefix; 2] }

/// Physical PS/HIF boundary. Its first `0x38` bytes are the extension tail of
/// logical power-save view 1; the final word is retained beacon/TIM state.
#[repr(C, align(4))]
struct PowerSaveHifBoundary {
    opaque_00: OpaqueBytes<0x14>,
    duration_14: SharedU32,
    duration_18: SharedU32,
    duration_1c: SharedU32,
    opaque_20: OpaqueBytes<0x04>,
    interval_24: SharedU32,
    opaque_28: OpaqueBytes<0x08>,
    counter_30: SharedU16,
    threshold_32: SharedU16,
    scan_completion_34: SharedU16,
    sleep_vote_count_36: SharedU16,
    ps_mode_error_reported_38: SharedU8,
    opaque_39: OpaqueBytes<0x07>,
    beacon_tim_state_40: SharedU32,
}
#[repr(C, align(4))]
struct HostMessageFreeRing { producer: SharedU32, consumer: SharedU32, entries: [SharedU32; 4] }
#[repr(C, align(4))]
struct DeferredTransferQueue { active: SharedU32, pending_head: SharedU32, pending_tail: SharedU32, completed_head: SharedU32, completed_tail: SharedU32 }
#[repr(C, align(4))]
struct HifBufferState { host_message_ring: HostMessageFreeRing, opaque_18: SharedU32, transfer_queue: DeferredTransferQueue, control_word: SharedU32 }
#[repr(C, align(4))]
struct LegacyHifSoftwareState { mode: SharedU32, pending_count: SharedU32, coalesce_count: SharedU32, coalesce_timer: TimerEntry, rx_buffers: [SharedU32; 32], rx_consumer: SharedU32, rx_state: SharedU32, tx_queue: [SharedU32; 64], tx_producer: SharedU32, tx_consumer: SharedU32, queue_depth: SharedU32, queued_tx_producer: SharedU32, queued_tx_consumer: SharedU32, input_producer: SharedU32, input_consumer: SharedU32, input_ring_mask: SharedU32, input_descriptors: SharedU32, sequence_state: SharedU32, transport_state: SharedU32 }
#[repr(C, align(4))]
struct MicCompletionState { queue: DeferredTransferQueue }

#[repr(C, align(4))]
struct PhyCalibrationReferences { coefficient_i: SharedU32, coefficient_q: SharedU32, scale_i: SharedU32, scale_q: SharedU32 }
#[repr(C, align(4))]
struct PhyProfileState { state: SharedU8, opaque_01: SharedU8, profile: SharedU8, phase: SharedU8, opaque_04: OpaqueBytes<0x02>, channel: SharedU16, opaque_08: OpaqueBytes<0x05>, profile0_ready: SharedU8, opaque_0e: OpaqueBytes<0x02>, auxiliary_state: SharedU8, transition_gate: SharedU8, opaque_12: SharedU8, calibration_stage: SharedU8, profile0_state: SharedU8, profile1_ready: SharedU8, profile1_channel: SharedU16, opaque_18: OpaqueBytes<0x08>, reference_word: SharedU32, opaque_24: SharedU32 }
#[repr(C, align(4))]
struct PhyMeasurementState { frequency_khz: SharedU32, opaque_04: SharedU32, control_08: SharedU8, opaque_09: OpaqueBytes<0x05>, sample_width: SharedU16, offset_word: SharedU32, correction: SharedU16, opaque_16: SharedU8, override_value: SharedU8, silicon_variant: SharedU8, opaque_19: OpaqueBytes<0x03>, denominator: SharedU16, correction_offset: SharedU16, measured_a: SharedU32, measured_b: SharedU32, opaque_28: OpaqueBytes<0x0d>, retained_state: SharedU8, opaque_36: SharedU8, zero_select: SharedU8 }
#[repr(C, align(4))]
struct PhyChannelCacheState { configuration_cache: [SharedU32; 2], opaque_08: OpaqueBytes<0x10>, startup_observation: SharedU8, opaque_19: OpaqueBytes<0x09>, retained_channel: SharedU16, calibration_state: SharedU8, calibration_aux: SharedU8, opaque_26: OpaqueBytes<0x02>, control_word: SharedU32, table_pointer: SharedU32 }
#[repr(C, align(4))]
struct PhyTableControlState { opaque_00: OpaqueBytes<0x10>, table_a: SharedU32, table_b: SharedU32, state_scale: SharedU32, opaque_1c: OpaqueBytes<0x0c>, threshold: SharedU16, opaque_2a: OpaqueBytes<0x02>, extended_settle: SharedU8, control_2d: SharedU8, opaque_2e: OpaqueBytes<0x02> }
#[repr(C, align(4))]
struct PhyCoreState { references: PhyCalibrationReferences, profile_state: PhyProfileState, measurement_state: PhyMeasurementState, channel_cache_state: PhyChannelCacheState, table_control_state: PhyTableControlState }

#[repr(C, align(4))]
struct PhyIqCalibrationSlot { axis_a: SharedU32, axis_b: SharedU32, opaque_08: OpaqueBytes<0x08> }
#[repr(C, align(4))]
struct PhyIqCalibrationPage { opaque_00: OpaqueBytes<0x14>, slots: [PhyIqCalibrationSlot; 12], opaque_d4: OpaqueBytes<0x2c> }
#[repr(C, align(4))]
struct PhyIqCalibrationResults { opaque_00: OpaqueBytes<0x14>, values: [SharedU32; 9] }
#[repr(C, align(4))]
struct PhyTail { calibration_pages: [PhyIqCalibrationPage; 2], results: PhyIqCalibrationResults }
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
    ba_sessions: BaSessions,                         // 0x8e78
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
#[cfg(target_arch = "arm")]
static DTCM_STATE: SharedDtcmState = SharedDtcmState(UnsafeCell::new(MaybeUninit::uninit()));

// Host tests must never manufacture pointers into target DTCM. The complete
// process-local backing also makes offset and access-order tests deterministic.
#[cfg(not(target_arch = "arm"))]
static DTCM_STATE: SharedDtcmState = SharedDtcmState(UnsafeCell::new(MaybeUninit::zeroed()));

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

/// Identity of one of the three fixed internal TX contexts.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct InternalContextAddress(DtcmAddress);

/// Checked identity of one of the 30 fixed host WSM TX contexts.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HostContextAddress(DtcmAddress);

/// Checked identity of the intrusive PAS/frame node embedded at context `+0x54`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HostFrameNodeAddress(DtcmAddress);

/// Checked identity of the decoded PAS overlay beginning at context `+0x54`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HostPasAddress(DtcmAddress);

impl HifRequestAddress {
    pub(crate) const fn raw(self) -> u32 { self.0 }
}

impl PacketRamAddress {
    pub(crate) const fn raw(self) -> u32 { self.0 }
}

impl InternalContextAddress {
    pub(crate) const fn from_index(index: usize) -> Option<Self> {
        if index < INTERNAL_TX_CONTEXT_COUNT {
            Some(Self(DtcmAddress::from_offset_unchecked(
                core::mem::offset_of!(DtcmLayout, internal_context_pool)
                    + core::mem::offset_of!(InternalContextPoolState, contexts)
                    + index * core::mem::size_of::<InternalTxContext>(),
            )))
        } else {
            None
        }
    }

    /// Construct an internal-context address after the context family has
    /// already been distinguished from host contexts.
    ///
    /// # Safety
    /// `address` must identify the start of one `InternalTxContext` record.
    pub(crate) const unsafe fn from_raw_unchecked(address: u32) -> Self {
        Self(DtcmAddress::from_offset_unchecked(address as usize - DTCM_STATE_BASE))
    }

    pub(crate) const fn raw(self) -> u32 { self.0.get() as u32 }
    pub(crate) const INTRUSIVE_NEXT_OFFSET: u32 = core::mem::offset_of!(InternalTxContext, next_free) as u32;
    pub(crate) const BORROWED_FRAME_ADDRESS_OFFSET: u32 = core::mem::offset_of!(InternalTxContext, header_80211) as u32;
    pub(crate) const COMPLETION_STATUS_OFFSET: u32 = core::mem::offset_of!(InternalTxContext, completion_status) as u32;
    pub(crate) const SAVED_STATUS_OFFSET: u32 = core::mem::offset_of!(InternalTxContext, saved_status) as u32;
    pub(crate) const COMPLETION_FLAGS_OFFSET: u32 = core::mem::offset_of!(InternalTxContext, completion_flags) as u32;
    pub(crate) const OPTIONAL_PIPE_OBJECT_OFFSET: u32 = core::mem::offset_of!(InternalTxContext, optional_pipe_object) as u32;
    pub(crate) const COMPLETION_CLASS_OFFSET: u32 = core::mem::offset_of!(InternalTxContext, completion_class) as u32;
    pub(crate) const FRAME_ADDRESS_OFFSET: u32 = (core::mem::offset_of!(InternalTxContext, pas) + core::mem::offset_of!(InternalPasContext, frame_address)) as u32;
    pub(crate) const CONTROL_BITS_OFFSET: u32 = (core::mem::offset_of!(InternalTxContext, pas) + core::mem::offset_of!(InternalPasContext, control_bits)) as u32;
    pub(crate) const FRAME_LENGTH_OFFSET: u32 = (core::mem::offset_of!(InternalTxContext, pas) + core::mem::offset_of!(InternalPasContext, frame_length)) as u32;
    pub(crate) const FRAME_CONTROL_OFFSET: u32 = (core::mem::offset_of!(InternalTxContext, pas) + core::mem::offset_of!(InternalPasContext, frame_control)) as u32;
    pub(crate) const ACCESS_CATEGORY_OFFSET: u32 = (core::mem::offset_of!(InternalTxContext, pas) + core::mem::offset_of!(InternalPasContext, access_category)) as u32;
    pub(crate) const REQUEST_FLAG_RATE_BITS_OFFSET: u32 = (core::mem::offset_of!(InternalTxContext, pas) + core::mem::offset_of!(InternalPasContext, request_flag_rate_bits)) as u32;
    pub(crate) const RETRY_POLICY_OFFSET: u32 = (core::mem::offset_of!(InternalTxContext, pas) + core::mem::offset_of!(InternalPasContext, retry_policy)) as u32;
    pub(crate) const TX_RATE_OFFSET: u32 = (core::mem::offset_of!(InternalTxContext, pas) + core::mem::offset_of!(InternalPasContext, tx_rate)) as u32;
    pub(crate) const COMPLETION_TIMESTAMP_OFFSET: u32 = (core::mem::offset_of!(InternalTxContext, pas) + core::mem::offset_of!(InternalPasContext, completion_timestamp)) as u32;
    pub(crate) const SCHEDULER_TIMESTAMP_OFFSET: u32 = (core::mem::offset_of!(InternalTxContext, pas) + core::mem::offset_of!(InternalPasContext, scheduler_timestamp)) as u32;
    pub(crate) const TERMINAL_STATUS_OFFSET: u32 = (core::mem::offset_of!(InternalTxContext, pas) + core::mem::offset_of!(InternalPasContext, terminal_status)) as u32;
    pub(crate) const TRY_COUNT_OFFSET: u32 = (core::mem::offset_of!(InternalTxContext, pas) + core::mem::offset_of!(InternalPasContext, try_count)) as u32;
    pub(crate) const OWNERSHIP_BITS_OFFSET: u32 = (core::mem::offset_of!(InternalTxContext, pas) + core::mem::offset_of!(InternalPasContext, ownership_bits)) as u32;
    pub(crate) const DURATION_OFFSET: u32 = (core::mem::offset_of!(InternalTxContext, pas) + core::mem::offset_of!(InternalPasContext, duration)) as u32;
    pub(crate) const DESCRIPTOR_STATE_OFFSET: u32 = (core::mem::offset_of!(InternalTxContext, pas) + core::mem::offset_of!(InternalPasContext, descriptor_state)) as u32;
    pub(crate) const FRAME_STATE_ADDRESS_OFFSET: u32 = (core::mem::offset_of!(InternalTxContext, pas) + core::mem::offset_of!(InternalPasContext, frame_state_address)) as u32;
    pub(crate) const AUXILIARY_STATE_OFFSET: u32 = (core::mem::offset_of!(InternalTxContext, pas) + core::mem::offset_of!(InternalPasContext, auxiliary_state)) as u32;
    pub(crate) const TID_OFFSET: u32 = (core::mem::offset_of!(InternalTxContext, pas) + core::mem::offset_of!(InternalPasContext, tid)) as u32;
    pub(crate) const SEQUENCE_NUMBER_OFFSET: u32 = (core::mem::offset_of!(InternalTxContext, pas) + core::mem::offset_of!(InternalPasContext, sequence_number)) as u32;
    pub(crate) const RETRY_RATE_OFFSET: u32 = (core::mem::offset_of!(InternalTxContext, pas) + core::mem::offset_of!(InternalPasContext, retry_rate)) as u32;
    pub(crate) const INTERFACE_OFFSET: u32 = (core::mem::offset_of!(InternalTxContext, pas) + core::mem::offset_of!(InternalPasContext, interface)) as u32;
    pub(crate) const DURATION_SLOT_OFFSET: u32 = (core::mem::offset_of!(InternalTxContext, pas) + core::mem::offset_of!(InternalPasContext, duration_slot)) as u32;
    pub(crate) const HOST_LINK_OFFSET: u32 = (core::mem::offset_of!(InternalTxContext, pas) + core::mem::offset_of!(InternalPasContext, host_link)) as u32;
    pub(crate) const COMPLETION_BYTE_6C_OFFSET: u32 = (core::mem::offset_of!(InternalTxContext, pas) + core::mem::offset_of!(InternalPasContext, completion_byte_6c)) as u32;
    const fn field(self, offset: usize) -> DtcmAddress { DtcmAddress::from_offset_unchecked(self.0.offset() + offset) }
    const fn pas_field(self, offset: usize) -> DtcmAddress { self.field(core::mem::offset_of!(InternalTxContext, pas) + offset) }

    pub(crate) const fn intrusive_next(self) -> DtcmAddress { self.field(core::mem::offset_of!(InternalTxContext, next_free)) }
    pub(crate) const fn borrowed_frame_address(self) -> DtcmAddress { self.field(core::mem::offset_of!(InternalTxContext, header_80211)) }
    pub(crate) const fn completion_status(self) -> DtcmAddress { self.field(core::mem::offset_of!(InternalTxContext, completion_status)) }
    pub(crate) const fn saved_status(self) -> DtcmAddress { self.field(core::mem::offset_of!(InternalTxContext, saved_status)) }
    pub(crate) const fn completion_flags(self) -> DtcmAddress { self.field(core::mem::offset_of!(InternalTxContext, completion_flags)) }
    pub(crate) const fn optional_pipe_object(self) -> DtcmAddress { self.field(core::mem::offset_of!(InternalTxContext, optional_pipe_object)) }
    pub(crate) const fn completion_class(self) -> DtcmAddress { self.field(core::mem::offset_of!(InternalTxContext, completion_class)) }
    pub(crate) const fn frame_address(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(InternalPasContext, frame_address)) }
    pub(crate) const fn control_bits(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(InternalPasContext, control_bits)) }
    pub(crate) const fn frame_length(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(InternalPasContext, frame_length)) }
    pub(crate) const fn frame_control(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(InternalPasContext, frame_control)) }
    pub(crate) const fn access_category(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(InternalPasContext, access_category)) }
    pub(crate) const fn request_flag_rate_bits(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(InternalPasContext, request_flag_rate_bits)) }
    pub(crate) const fn retry_policy(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(InternalPasContext, retry_policy)) }
    pub(crate) const fn tx_rate(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(InternalPasContext, tx_rate)) }
    pub(crate) const fn completion_timestamp(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(InternalPasContext, completion_timestamp)) }
    pub(crate) const fn scheduler_timestamp(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(InternalPasContext, scheduler_timestamp)) }
    pub(crate) const fn terminal_status(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(InternalPasContext, terminal_status)) }
    pub(crate) const fn try_count(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(InternalPasContext, try_count)) }
    pub(crate) const fn ownership_bits(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(InternalPasContext, ownership_bits)) }
    pub(crate) const fn duration(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(InternalPasContext, duration)) }
    pub(crate) const fn descriptor_state(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(InternalPasContext, descriptor_state)) }
    pub(crate) const fn frame_state_address(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(InternalPasContext, frame_state_address)) }
    pub(crate) const fn auxiliary_state(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(InternalPasContext, auxiliary_state)) }
    pub(crate) const fn tid(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(InternalPasContext, tid)) }
    pub(crate) const fn sequence_number(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(InternalPasContext, sequence_number)) }
    pub(crate) const fn retry_rate(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(InternalPasContext, retry_rate)) }
    pub(crate) const fn interface(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(InternalPasContext, interface)) }
    pub(crate) const fn duration_slot(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(InternalPasContext, duration_slot)) }
    pub(crate) const fn host_link(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(InternalPasContext, host_link)) }
    pub(crate) const fn completion_byte_6c(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(InternalPasContext, completion_byte_6c)) }
    pub(crate) const fn cipher_buffer(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(InternalPasContext, cipher_buffer)) }
}

impl HostContextAddress {
    pub(crate) const fn from_index(index: usize) -> Option<Self> {
        if index < HOST_TX_CONTEXT_COUNT {
            Some(Self(DtcmAddress::from_offset_unchecked(
                core::mem::offset_of!(DtcmLayout, host_contexts)
                    + index * core::mem::size_of::<HostTxContext>(),
            )))
        } else {
            None
        }
    }

    pub(crate) const fn from_raw(address: u32) -> Option<Self> {
        let base = HOST_TX_CONTEXTS.get();
        let offset = (address as usize).wrapping_sub(base);
        if address as usize >= base
            && offset % core::mem::size_of::<HostTxContext>() == 0
            && offset / core::mem::size_of::<HostTxContext>() < HOST_TX_CONTEXT_COUNT
        {
            Some(Self(DtcmAddress::from_offset_unchecked(
                core::mem::offset_of!(DtcmLayout, host_contexts) + offset,
            )))
        } else {
            None
        }
    }

    /// Construct a host-context address after equivalent range and stride
    /// validation has already been performed.
    ///
    /// # Safety
    /// `address` must identify the start of one `HostTxContext` record.
    pub(crate) const unsafe fn from_raw_unchecked(address: u32) -> Self {
        Self(DtcmAddress::from_offset_unchecked(
            address as usize - DTCM_STATE_BASE,
        ))
    }

    pub(crate) const fn raw(self) -> u32 { self.0.get() as u32 }
    pub(crate) const fn index(self) -> usize {
        (self.0.offset() - core::mem::offset_of!(DtcmLayout, host_contexts))
            / core::mem::size_of::<HostTxContext>()
    }
    const fn field(self, offset: usize) -> DtcmAddress {
        DtcmAddress::from_offset_unchecked(self.0.offset() + offset)
    }
    const fn pas_field(self, offset: usize) -> DtcmAddress {
        self.field(core::mem::offset_of!(HostTxContext, pas) + offset)
    }

    pub(crate) const fn request_buffer(self) -> DtcmAddress { self.field(core::mem::offset_of!(HostTxContext, request_buffer)) }
    pub(crate) const fn intrusive_next(self) -> DtcmAddress { self.field(core::mem::offset_of!(HostTxContext, intrusive_next)) }
    pub(crate) const fn packet_id(self) -> DtcmAddress { self.field(core::mem::offset_of!(HostTxContext, packet_id)) }
    pub(crate) const fn requested_rate(self) -> DtcmAddress { self.field(core::mem::offset_of!(HostTxContext, requested_rate)) }
    pub(crate) const fn queue_id(self) -> DtcmAddress { self.field(core::mem::offset_of!(HostTxContext, queue_id)) }
    pub(crate) const fn more(self) -> DtcmAddress { self.field(core::mem::offset_of!(HostTxContext, more)) }
    pub(crate) const fn request_flags(self) -> DtcmAddress { self.field(core::mem::offset_of!(HostTxContext, request_flags)) }
    pub(crate) const fn expiry_time(self) -> DtcmAddress { self.field(core::mem::offset_of!(HostTxContext, expiry_time)) }
    pub(crate) const fn ht_tx_parameters(self) -> DtcmAddress { self.field(core::mem::offset_of!(HostTxContext, ht_tx_parameters)) }
    pub(crate) const fn borrowed_frame_length(self) -> DtcmAddress { self.field(core::mem::offset_of!(HostTxContext, borrowed_frame_length)) }
    pub(crate) const fn borrowed_frame_address(self) -> DtcmAddress { self.field(core::mem::offset_of!(HostTxContext, borrowed_frame_address)) }
    pub(crate) const fn completion_status(self) -> DtcmAddress { self.field(core::mem::offset_of!(HostTxContext, completion_status)) }
    pub(crate) const fn rate_copy(self) -> DtcmAddress { self.field(core::mem::offset_of!(HostTxContext, rate_copy)) }
    pub(crate) const fn saved_status(self) -> DtcmAddress { self.field(core::mem::offset_of!(HostTxContext, saved_status)) }
    pub(crate) const fn completion_flags(self) -> DtcmAddress { self.field(core::mem::offset_of!(HostTxContext, completion_flags)) }
    pub(crate) const fn rate_try(self, index: usize) -> Option<DtcmAddress> {
        if index < 3 { Some(self.field(core::mem::offset_of!(HostTxContext, rate_try) + index * 4)) } else { None }
    }
    pub(crate) const fn timing_scratch(self, index: usize) -> Option<DtcmAddress> {
        if index < 2 { Some(self.field(core::mem::offset_of!(HostTxContext, timing_scratch) + index * 4)) } else { None }
    }
    pub(crate) const fn submit_timer(self) -> DtcmAddress { self.field(core::mem::offset_of!(HostTxContext, submit_timer)) }
    pub(crate) const fn header_length(self) -> DtcmAddress { self.field(core::mem::offset_of!(HostTxContext, header_length)) }
    pub(crate) const fn payload_length(self) -> DtcmAddress { self.field(core::mem::offset_of!(HostTxContext, payload_length)) }
    pub(crate) const fn optional_pipe_object(self) -> DtcmAddress { self.field(core::mem::offset_of!(HostTxContext, optional_pipe_object)) }
    pub(crate) const fn sequence_or_callback_state(self) -> DtcmAddress { self.field(core::mem::offset_of!(HostTxContext, sequence_or_callback_state)) }
    pub(crate) const fn submit_state(self) -> DtcmAddress { self.field(core::mem::offset_of!(HostTxContext, submit_state)) }
    pub(crate) const fn completion_class(self) -> DtcmAddress { self.field(core::mem::offset_of!(HostTxContext, completion_class)) }
    pub(crate) const fn frame_address(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(HostPasContext, frame_address)) }
    pub(crate) const fn control_bits(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(HostPasContext, control_bits)) }
    pub(crate) const fn frame_length(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(HostPasContext, frame_length)) }
    pub(crate) const fn frame_control(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(HostPasContext, frame_control)) }
    pub(crate) const fn access_category(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(HostPasContext, access_category)) }
    pub(crate) const fn request_flag_rate_bits(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(HostPasContext, request_flag_rate_bits)) }
    pub(crate) const fn retry_policy(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(HostPasContext, retry_policy)) }
    pub(crate) const fn tx_rate(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(HostPasContext, tx_rate)) }
    pub(crate) const fn pas_expiry_time(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(HostPasContext, expiry_time)) }
    pub(crate) const fn completion_timestamp(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(HostPasContext, completion_timestamp)) }
    pub(crate) const fn scheduler_timestamp(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(HostPasContext, scheduler_timestamp)) }
    pub(crate) const fn terminal_status(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(HostPasContext, terminal_status)) }
    pub(crate) const fn try_count(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(HostPasContext, try_count)) }
    /// Three pool-initialized words with unresolved retry/aggregation overlay semantics.
    pub(crate) const fn pas_reset_word(self, index: usize) -> Option<DtcmAddress> {
        if index < 3 { Some(self.pas_field(core::mem::offset_of!(HostPasContext, opaque_20) + index * 4)) } else { None }
    }
    pub(crate) const fn ownership_bits(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(HostPasContext, ownership_bits)) }
    pub(crate) const fn duration(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(HostPasContext, duration)) }
    pub(crate) const fn descriptor_state(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(HostPasContext, descriptor_state)) }
    pub(crate) const fn word_48(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(HostPasContext, word_48)) }
    pub(crate) const fn frame_state_address(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(HostPasContext, frame_state_address)) }
    pub(crate) const fn auxiliary_state(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(HostPasContext, auxiliary_state)) }
    pub(crate) const fn tid(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(HostPasContext, tid)) }
    pub(crate) const fn insertion_mode(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(HostPasContext, insertion_mode)) }
    pub(crate) const fn sequence_number(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(HostPasContext, sequence_number)) }
    pub(crate) const fn retry_rate(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(HostPasContext, retry_rate)) }
    pub(crate) const fn byte_57(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(HostPasContext, byte_57)) }
    pub(crate) const fn interface(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(HostPasContext, interface)) }
    pub(crate) const fn duration_slot(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(HostPasContext, duration_slot)) }
    pub(crate) const fn host_link(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(HostPasContext, host_link)) }
    pub(crate) const fn completion_byte_6c(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(HostPasContext, completion_byte_6c)) }
    pub(crate) const fn qos_control(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(HostPasContext, qos_control)) }
    pub(crate) const fn cipher_class(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(HostPasContext, cipher_class)) }
    pub(crate) const fn word_7c(self) -> DtcmAddress { self.pas_field(core::mem::offset_of!(HostPasContext, word_7c)) }
    pub(crate) const fn pas(self) -> HostPasAddress { HostPasAddress(self.frame_address()) }
    pub(crate) const fn frame_node(self) -> HostFrameNodeAddress { HostFrameNodeAddress(self.frame_address()) }
    pub(crate) fn expected_frame_state(self) -> PacketRamAddress {
        PacketRamAddress(crate::packet_ram::host_frame_state(self.index()) as u32)
    }
}

impl HostFrameNodeAddress {
    pub(crate) const fn raw(self) -> u32 { self.0.get() as u32 }
    pub(crate) const fn context(self) -> HostContextAddress {
        HostContextAddress(DtcmAddress::from_offset_unchecked(
            self.0.offset() - core::mem::offset_of!(HostTxContext, pas),
        ))
    }
}

impl HostPasAddress {
    pub(crate) const fn raw(self) -> u32 { self.0.get() as u32 }
    pub(crate) const fn context(self) -> HostContextAddress {
        HostContextAddress(DtcmAddress::from_offset_unchecked(
            self.0.offset() - core::mem::offset_of!(HostTxContext, pas),
        ))
    }
}

pub(crate) const fn host_context(index: usize) -> Option<HostContextAddress> {
    HostContextAddress::from_index(index)
}

pub(crate) const fn host_context_from_raw(address: u32) -> Option<HostContextAddress> {
    HostContextAddress::from_raw(address)
}

pub(crate) const HOST_DUPLICATE_CACHE_CURSOR: DtcmAddress = DtcmAddress::from_offset(
    core::mem::offset_of!(DtcmLayout, host_context_accounting)
        + core::mem::offset_of!(HostContextAccounting, duplicate_cache_cursor),
);
pub(crate) const HOST_DEFERRED_EVENT_OWNER: DtcmAddress = DtcmAddress::from_offset(
    core::mem::offset_of!(DtcmLayout, host_context_accounting)
        + core::mem::offset_of!(HostContextAccounting, deferred_event_owner),
);
pub(crate) const AUXILIARY_TX_BUFFER_FREE_HEAD: DtcmAddress = DtcmAddress::from_offset(
    core::mem::offset_of!(DtcmLayout, host_context_accounting)
        + core::mem::offset_of!(HostContextAccounting, auxiliary_tx_buffer_free_head),
);
pub const HOST_CONTEXT_FREE_HEAD: DtcmAddress = DtcmAddress::from_offset(
    core::mem::offset_of!(DtcmLayout, host_context_free_list)
        + core::mem::offset_of!(HostContextFreeList, free_head),
);
pub(crate) const HOST_CONTEXT_ADJACENT_STATE: DtcmAddress = DtcmAddress::from_offset(
    core::mem::offset_of!(DtcmLayout, host_context_free_list)
        + core::mem::offset_of!(HostContextFreeList, adjacent_state),
);

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
pub const INITIALIZED_VENDOR_IMAGE: DtcmAddress = DtcmAddress::from_offset(0x0000); pub const TKIP_SBOX_TABLES: DtcmAddress = DtcmAddress::from_offset(core::mem::offset_of!(InitializedVendorImage, tkip_sbox_tables)); pub(crate) const AES_TRANSFER_CLASSES: DtcmAddress = DtcmAddress::from_offset(core::mem::offset_of!(InitializedVendorImage, aes_transfer_classes)); pub(crate) const PHY_GAIN_REGISTER_WRITE_LISTS: DtcmAddress = DtcmAddress::from_offset(core::mem::offset_of!(InitializedVendorImage, phy_gain_register_write_lists)); pub(crate) const PHY_INIT_REGISTER_WRITE_LISTS: DtcmAddress = DtcmAddress::from_offset(core::mem::offset_of!(InitializedVendorImage, phy_init_register_write_list_0)); pub(crate) const INITIALIZED_PHY_GAIN_SOURCE_RECORDS: DtcmAddress = DtcmAddress::from_offset(core::mem::offset_of!(InitializedVendorImage, initialized_phy_gain_source_records)); pub(crate) const INITIALIZED_IQ_CALIBRATION_GAIN_INDICES: DtcmAddress = DtcmAddress::from_offset(core::mem::offset_of!(InitializedVendorImage, initialized_iq_calibration_gain_indices)); pub(crate) const RF_MODE_HALFWORD_TABLE: DtcmAddress = DtcmAddress::from_offset(core::mem::offset_of!(InitializedVendorImage, rf_mode_halfword_table) + core::mem::offset_of!(RfModeHalfwordTable, entries));
pub const fn tkip_sbox_low_entry(index: usize) -> Option<DtcmAddress> { if index < 256 { Some(DtcmAddress::from_offset(TKIP_SBOX_TABLES.offset() + core::mem::offset_of!(TkipSboxTables, low_byte) + index * core::mem::size_of::<SharedU16>())) } else { None } }
pub const fn tkip_sbox_high_entry(index: usize) -> Option<DtcmAddress> { if index < 256 { Some(DtcmAddress::from_offset(TKIP_SBOX_TABLES.offset() + core::mem::offset_of!(TkipSboxTables, high_byte) + index * core::mem::size_of::<SharedU16>())) } else { None } } pub(crate) const fn aes_transfer_class(class: usize) -> Option<DtcmAddress> { if class < 11 { Some(DtcmAddress::from_offset(AES_TRANSFER_CLASSES.offset() + core::mem::offset_of!(AesTransferClassTable, flags) + class * core::mem::size_of::<SharedU32>())) } else { None } } pub(crate) const fn phy_gain_register_write_list(profile: usize) -> Option<DtcmAddress> { if profile < 2 { Some(DtcmAddress::from_offset(PHY_GAIN_REGISTER_WRITE_LISTS.offset() + profile * core::mem::size_of::<RegisterWriteList>())) } else { None } } pub(crate) const fn phy_gain_register_write_entry(profile: usize, entry: usize) -> Option<DtcmAddress> { if profile < 2 && entry < 10 { Some(DtcmAddress::from_offset(PHY_GAIN_REGISTER_WRITE_LISTS.offset() + profile * core::mem::size_of::<RegisterWriteList>() + core::mem::offset_of!(RegisterWriteList, writes) + entry * core::mem::size_of::<RegisterWrite>())) } else { None } } pub(crate) const fn phy_init_register_write_list(index: usize) -> Option<DtcmAddress> { match index { 0 => Some(PHY_INIT_REGISTER_WRITE_LISTS), 1 => Some(DtcmAddress::from_offset(core::mem::offset_of!(InitializedVendorImage, phy_init_register_write_list_1))), 2 => Some(DtcmAddress::from_offset(core::mem::offset_of!(InitializedVendorImage, phy_init_register_write_list_2))), _ => None } } pub(crate) const fn phy_init_register_write_entry(list: usize, entry: usize) -> Option<DtcmAddress> { let count = match list { 0 => 1, 1 => 4, 2 => 2, _ => return None }; if entry < count { Some(DtcmAddress::from_offset(phy_init_register_write_list(list).unwrap().offset() + entry * core::mem::size_of::<RegisterWrite>())) } else { None } } pub(crate) const fn initialized_phy_gain_source_physical_record(index: usize) -> Option<DtcmAddress> { if index < 43 { Some(DtcmAddress::from_offset(INITIALIZED_PHY_GAIN_SOURCE_RECORDS.offset() + index * core::mem::size_of::<InitializedPhyGainSourceRecord>())) } else { None } } pub(crate) const fn initialized_phy_gain_source_view_record(view: usize, index: usize) -> Option<DtcmAddress> { if view < 2 && index < 22 { initialized_phy_gain_source_physical_record(view * 21 + index) } else { None } } pub(crate) const fn initialized_iq_calibration_gain_index(index: usize) -> Option<DtcmAddress> { if index < 12 { Some(DtcmAddress::from_offset(INITIALIZED_IQ_CALIBRATION_GAIN_INDICES.offset() + core::mem::offset_of!(InitializedIqCalibrationGainIndices, entries) + index * core::mem::size_of::<SharedU32>())) } else { None } }
pub(crate) const MEASUREMENT_WORKSPACE: DtcmAddress = DtcmAddress::from_offset(core::mem::offset_of!(InitializedVendorImage, measurement_workspace)); pub(crate) const TX_AGGREGATE_EXPIRATION_DELTA: DtcmAddress = DtcmAddress::from_offset(core::mem::offset_of!(InitializedVendorImage, tx_aggregate_expiration_delta)); pub(crate) const DEBUG_COMMAND_DESCRIPTORS: DtcmAddress = DtcmAddress::from_offset(core::mem::offset_of!(InitializedVendorImage, debug_command_descriptors)); pub(crate) const fn debug_command_descriptor(index: usize) -> Option<DtcmAddress> { if index < 6 { Some(DtcmAddress::from_offset(DEBUG_COMMAND_DESCRIPTORS.offset() + core::mem::offset_of!(InitializedDebugCommandDescriptors, records) + index * core::mem::size_of::<DebugCommandDescriptor>())) } else { None } } pub(crate) const fn debug_command_name(index: usize) -> Option<DtcmAddress> { if index < 6 { Some(DtcmAddress::from_offset(DEBUG_COMMAND_DESCRIPTORS.offset() + core::mem::offset_of!(InitializedDebugCommandDescriptors, records) + index * core::mem::size_of::<DebugCommandDescriptor>() + core::mem::offset_of!(DebugCommandDescriptor, command_name))) } else { None } } pub(crate) const fn debug_command_help(index: usize) -> Option<DtcmAddress> { if index < 6 { Some(DtcmAddress::from_offset(DEBUG_COMMAND_DESCRIPTORS.offset() + core::mem::offset_of!(InitializedDebugCommandDescriptors, records) + index * core::mem::size_of::<DebugCommandDescriptor>() + core::mem::offset_of!(DebugCommandDescriptor, help_text))) } else { None } } pub(crate) const fn debug_command_handler(index: usize) -> Option<DtcmAddress> { if index < 6 { Some(DtcmAddress::from_offset(DEBUG_COMMAND_DESCRIPTORS.offset() + core::mem::offset_of!(InitializedDebugCommandDescriptors, records) + index * core::mem::size_of::<DebugCommandDescriptor>() + core::mem::offset_of!(DebugCommandDescriptor, handler))) } else { None } } const fn measurement_workspace_field(offset: usize) -> DtcmAddress { DtcmAddress::from_offset(MEASUREMENT_WORKSPACE.offset() + offset) } pub(crate) const fn measurement_dwell_bound() -> DtcmAddress { measurement_workspace_field(core::mem::offset_of!(MeasurementWorkspace, dwell_bound)) } pub(crate) const fn measurement_type() -> DtcmAddress { measurement_workspace_field(core::mem::offset_of!(MeasurementWorkspace, measurement_type)) } pub(crate) const fn measurement_completion_status() -> DtcmAddress { measurement_workspace_field(core::mem::offset_of!(MeasurementWorkspace, completion_status)) } pub(crate) const fn measurement_dispatch_state() -> DtcmAddress { measurement_workspace_field(core::mem::offset_of!(MeasurementWorkspace, dispatch_state)) } pub(crate) const fn measurement_dispatch_argument() -> DtcmAddress { measurement_workspace_field(core::mem::offset_of!(MeasurementWorkspace, dispatch_argument)) } pub(crate) const fn measurement_start_timestamp_word(index: usize) -> Option<DtcmAddress> { if index < 2 { Some(measurement_workspace_field(core::mem::offset_of!(MeasurementWorkspace, start_timestamp_words) + index * core::mem::size_of::<SharedU32>())) } else { None } } pub(crate) const fn measurement_elapsed_timestamp_word(index: usize) -> Option<DtcmAddress> { if index < 2 { Some(measurement_workspace_field(core::mem::offset_of!(MeasurementWorkspace, elapsed_timestamp_words) + index * core::mem::size_of::<SharedU32>())) } else { None } } pub(crate) const fn measurement_scan_request() -> DtcmAddress { measurement_workspace_field(core::mem::offset_of!(MeasurementWorkspace, scan_request_prefix)) } pub(crate) const fn measurement_scan_request_mode() -> DtcmAddress { measurement_workspace_field(core::mem::offset_of!(MeasurementWorkspace, scan_request_mode)) } pub(crate) const INITIALIZED_HIF_CONTROL: DtcmAddress = DtcmAddress::from_offset(core::mem::offset_of!(InitializedVendorImage, hif_control));
const fn initialized_hif_control_field(offset: usize) -> DtcmAddress { DtcmAddress::from_offset(INITIALIZED_HIF_CONTROL.offset() + offset) }
pub(crate) const fn initialized_hif_queued_depth() -> DtcmAddress { initialized_hif_control_field(core::mem::offset_of!(InitializedHifControl, queued_depth)) }
pub(crate) const fn initialized_hif_pending_count() -> DtcmAddress { initialized_hif_control_field(core::mem::offset_of!(InitializedHifControl, pending_count)) }
pub(crate) const fn initialized_hif_coalesce_enabled() -> DtcmAddress { initialized_hif_control_field(core::mem::offset_of!(InitializedHifControl, coalesce_enabled)) }
pub(crate) const fn initialized_hif_pending_threshold() -> DtcmAddress { initialized_hif_control_field(core::mem::offset_of!(InitializedHifControl, pending_threshold)) }
pub(crate) const fn initialized_hif_ring_depth_threshold() -> DtcmAddress { initialized_hif_control_field(core::mem::offset_of!(InitializedHifControl, ring_depth_threshold)) }
pub(crate) const fn initialized_hif_count_threshold() -> DtcmAddress { initialized_hif_control_field(core::mem::offset_of!(InitializedHifControl, count_threshold)) }
pub(crate) const fn initialized_hif_coalesce_delay() -> DtcmAddress { initialized_hif_control_field(core::mem::offset_of!(InitializedHifControl, coalesce_delay)) }
pub(crate) const AMPDU_COMPLETION_CONTROL: DtcmAddress = DtcmAddress::from_offset(core::mem::offset_of!(InitializedVendorImage, ampdu_completion_control));
pub(crate) const AMPDU_TELEMETRY_COUNTERS: DtcmAddress = DtcmAddress::from_offset(core::mem::offset_of!(InitializedVendorImage, ampdu_counters));
const fn ampdu_telemetry_field(offset: usize) -> DtcmAddress { DtcmAddress::from_offset(AMPDU_TELEMETRY_COUNTERS.offset() + offset) }
pub(crate) const fn ampdu_tx_error_frames() -> DtcmAddress { ampdu_telemetry_field(core::mem::offset_of!(AmpduTelemetryCounters, tx_error_frames)) }
pub(crate) const fn ampdu_tx_counted_frames() -> DtcmAddress { ampdu_telemetry_field(core::mem::offset_of!(AmpduTelemetryCounters, tx_counted_frames)) }
pub(crate) const fn ampdu_tx_duration_low() -> DtcmAddress { ampdu_telemetry_field(core::mem::offset_of!(AmpduTelemetryCounters, tx_duration_low)) }
pub(crate) const fn ampdu_tx_duration_high() -> DtcmAddress { ampdu_telemetry_field(core::mem::offset_of!(AmpduTelemetryCounters, tx_duration_high)) }
pub(crate) const fn ampdu_rx_management(index: usize) -> Option<DtcmAddress> { if index < 4 { Some(ampdu_telemetry_field(core::mem::offset_of!(AmpduTelemetryCounters, rx_management_0) + index * 4)) } else { None } }
pub(crate) const fn ampdu_tx_retry_count() -> DtcmAddress { ampdu_telemetry_field(core::mem::offset_of!(AmpduTelemetryCounters, tx_retry_count)) }
pub(crate) const PHY_CHANNEL_THRESHOLD_DESCRIPTORS: DtcmAddress = DtcmAddress::from_offset(core::mem::offset_of!(InitializedVendorImage, phy_channel_threshold_descriptors));
pub(crate) const fn phy_channel_threshold_descriptor(profile: usize) -> Option<DtcmAddress> { if profile < 2 { Some(DtcmAddress::from_offset(PHY_CHANNEL_THRESHOLD_DESCRIPTORS.offset() + profile * core::mem::size_of::<PhyChannelThresholdDescriptor>())) } else { None } }
pub(crate) const PHY_GAIN_PROGRAMMING_RECORDS: DtcmAddress = DtcmAddress::from_offset(core::mem::offset_of!(InitializedVendorImage, phy_gain_programming_records));
pub(crate) const fn phy_gain_programming_record(slot: usize) -> Option<DtcmAddress> { if slot < 16 { Some(DtcmAddress::from_offset(PHY_GAIN_PROGRAMMING_RECORDS.offset() + slot * core::mem::size_of::<PhyGainProgrammingRecord>())) } else { None } }
pub(crate) const INITIALIZED_RATE_POLICIES: DtcmAddress = DtcmAddress::from_offset(core::mem::offset_of!(InitializedVendorImage, initialized_rate_policies));
pub(crate) const fn initialized_rate_policy_word(policy: usize, word: usize) -> Option<DtcmAddress> { if policy < 2 && word < 5 { Some(DtcmAddress::from_offset(INITIALIZED_RATE_POLICIES.offset() + (policy * 5 + word) * core::mem::size_of::<SharedU32>())) } else { None } }
pub(crate) const MAC_TX_QUEUE_STATE: DtcmAddress = DtcmAddress::from_offset(core::mem::offset_of!(InitializedVendorImage, mac_tx_queue_state));
pub(crate) const MAC_TX_QUEUE_HEAD: DtcmAddress = DtcmAddress::from_offset(MAC_TX_QUEUE_STATE.offset() + core::mem::offset_of!(MacTxQueueState, head));
pub(crate) const MAC_TX_QUEUE_TAIL: DtcmAddress = DtcmAddress::from_offset(MAC_TX_QUEUE_STATE.offset() + core::mem::offset_of!(MacTxQueueState, tail));
pub(crate) const MAC_RETRY_HARDWARE_STATE: DtcmAddress = DtcmAddress::from_offset(core::mem::offset_of!(InitializedVendorImage, mac_retry_hardware_state));
pub(crate) const MAC_RUNTIME_ACCOUNTING: DtcmAddress = DtcmAddress::from_offset(core::mem::offset_of!(InitializedVendorImage, mac_runtime_accounting));
pub(crate) const MAC_CURRENT_PIPE: DtcmAddress = DtcmAddress::from_offset(MAC_RUNTIME_ACCOUNTING.offset() + core::mem::offset_of!(MacRuntimeAccountingState, current_pipe));
pub(crate) const MAC_STATUS_ACCOUNTING: DtcmAddress = DtcmAddress::from_offset(MAC_RUNTIME_ACCOUNTING.offset() + core::mem::offset_of!(MacRuntimeAccountingState, status_accounting));
pub(crate) const MAC_SAMPLE_COUNT: DtcmAddress = DtcmAddress::from_offset(MAC_RUNTIME_ACCOUNTING.offset() + core::mem::offset_of!(MacRuntimeAccountingState, sample_count));
pub(crate) const MAC_CURRENT_PIPE_RECORD: DtcmAddress = DtcmAddress::from_offset(MAC_RUNTIME_ACCOUNTING.offset() + core::mem::offset_of!(MacRuntimeAccountingState, current_pipe_record));
pub(crate) const MAC_CURRENT_SLOT: DtcmAddress = DtcmAddress::from_offset(MAC_RUNTIME_ACCOUNTING.offset() + core::mem::offset_of!(MacRuntimeAccountingState, current_slot));
pub(crate) const MAC_PIPE_EVENT_FLAGS: DtcmAddress = DtcmAddress::from_offset(MAC_RUNTIME_ACCOUNTING.offset() + core::mem::offset_of!(MacRuntimeAccountingState, pipe_event_flags));
pub(crate) const MAC_SOFTWARE_RECORDS: DtcmAddress = DtcmAddress::from_offset(MAC_RUNTIME_ACCOUNTING.offset() + core::mem::offset_of!(MacRuntimeAccountingState, software_records));
pub(crate) const MAC_ACCOUNTING_AVERAGE: DtcmAddress = DtcmAddress::from_offset(MAC_RUNTIME_ACCOUNTING.offset() + core::mem::offset_of!(MacRuntimeAccountingState, average));
pub(crate) const MAC_SILICON_CONTROL: DtcmAddress = DtcmAddress::from_offset(MAC_RUNTIME_ACCOUNTING.offset() + core::mem::offset_of!(MacRuntimeAccountingState, silicon_control));
pub(crate) const MAC_ACCOUNTING_PARAMETER0: DtcmAddress = DtcmAddress::from_offset(MAC_RUNTIME_ACCOUNTING.offset() + core::mem::offset_of!(MacRuntimeAccountingState, parameter0));
pub(crate) const MAC_ACCOUNTING_PARAMETER1: DtcmAddress = DtcmAddress::from_offset(MAC_RUNTIME_ACCOUNTING.offset() + core::mem::offset_of!(MacRuntimeAccountingState, parameter1));
pub(crate) const MAC_PHY_COMMAND_STATE: DtcmAddress = DtcmAddress::from_offset(core::mem::offset_of!(InitializedVendorImage, mac_phy_command_state));
pub(crate) const MAC_RADIO_STOP_STATE: DtcmAddress = DtcmAddress::from_offset(MAC_PHY_COMMAND_STATE.offset() + core::mem::offset_of!(MacPhyCommandState, radio_stop_state));
pub(crate) const MAC_SIDEBAND_CAPTURE: DtcmAddress = DtcmAddress::from_offset(MAC_PHY_COMMAND_STATE.offset() + core::mem::offset_of!(MacPhyCommandState, sideband_capture));
pub(crate) const MAC_PHY_OPERATION_TIMER: DtcmAddress = DtcmAddress::from_offset(MAC_PHY_COMMAND_STATE.offset() + core::mem::offset_of!(MacPhyCommandState, timer));
pub(crate) const MAC_PHY_OPERATION_ROOT: DtcmAddress = DtcmAddress::from_offset(MAC_PHY_COMMAND_STATE.offset() + 0x10);
pub(crate) const MAC_PHY_OPERATION_STATE: DtcmAddress = DtcmAddress::from_offset(MAC_PHY_COMMAND_STATE.offset() + core::mem::offset_of!(MacPhyCommandState, operation_state));
pub(crate) const MAC_PHY_OPERATION_COMMAND: DtcmAddress = DtcmAddress::from_offset(MAC_PHY_COMMAND_STATE.offset() + core::mem::offset_of!(MacPhyCommandState, operation_command));
pub(crate) const MAC_PHY_OPERATION_OUTPUT: DtcmAddress = DtcmAddress::from_offset(MAC_PHY_COMMAND_STATE.offset() + core::mem::offset_of!(MacPhyCommandState, operation_output_state));
pub(crate) const MAC_PHY_OPERATION_TIMEOUT: DtcmAddress = DtcmAddress::from_offset(MAC_PHY_COMMAND_STATE.offset() + core::mem::offset_of!(MacPhyCommandState, operation_timeout));
pub(crate) const MAC_PHY_DISPATCH_COMMAND: DtcmAddress = DtcmAddress::from_offset(MAC_PHY_COMMAND_STATE.offset() + core::mem::offset_of!(MacPhyCommandState, dispatch_command));
pub(crate) const MAC_PHY_DISPATCH_OUTPUT: DtcmAddress = DtcmAddress::from_offset(MAC_PHY_COMMAND_STATE.offset() + core::mem::offset_of!(MacPhyCommandState, dispatch_output_state));
pub(crate) const MAC_PHY_COMPLETION_STATUS: DtcmAddress = DtcmAddress::from_offset(MAC_PHY_COMMAND_STATE.offset() + core::mem::offset_of!(MacPhyCommandState, completion_status));
pub(crate) const MAC_PHY_INTERFACE: DtcmAddress = DtcmAddress::from_offset(MAC_PHY_COMMAND_STATE.offset() + core::mem::offset_of!(MacPhyCommandState, interface));
pub(crate) const MAC_WAKE_RUNTIME_STATE: DtcmAddress = DtcmAddress::from_offset(core::mem::offset_of!(InitializedVendorImage, mac_wake_runtime_state));
pub(crate) const MAC_WAKE_TIMER: DtcmAddress = DtcmAddress::from_offset(MAC_WAKE_RUNTIME_STATE.offset() + core::mem::offset_of!(MacWakeRuntimeState, timer));
pub(crate) const MAC_WAKE_PHY_STATE: DtcmAddress = DtcmAddress::from_offset(MAC_WAKE_RUNTIME_STATE.offset() + core::mem::offset_of!(MacWakeRuntimeState, phy_state));
pub(crate) const MAC_WAKE_TRANSITION_PENDING: DtcmAddress = DtcmAddress::from_offset(MAC_WAKE_RUNTIME_STATE.offset() + core::mem::offset_of!(MacWakeRuntimeState, transition_pending));
pub(crate) const MAC_WAKE_RESTORE_PENDING: DtcmAddress = DtcmAddress::from_offset(MAC_WAKE_RUNTIME_STATE.offset() + core::mem::offset_of!(MacWakeRuntimeState, restore_pending));
pub const MAC_WAKE_MODE: DtcmAddress = DtcmAddress::from_offset(MAC_WAKE_RUNTIME_STATE.offset() + core::mem::offset_of!(MacWakeRuntimeState, mode));
pub(crate) const MAC_WAKE_CONTROL: DtcmAddress = DtcmAddress::from_offset(MAC_WAKE_RUNTIME_STATE.offset() + core::mem::offset_of!(MacWakeRuntimeState, control));
pub(crate) const MAC_RETRY_RATE_MAP: DtcmAddress = DtcmAddress::from_offset(MAC_WAKE_RUNTIME_STATE.offset() + core::mem::offset_of!(MacWakeRuntimeState, retry_rate_map));
pub(crate) const MAC_EDCA_SLOT_TIMING: DtcmAddress = DtcmAddress::from_offset(MAC_WAKE_RUNTIME_STATE.offset() + core::mem::offset_of!(MacWakeRuntimeState, edca_slot_timing));
pub(crate) const MAC_BEACON_STATE: DtcmAddress = DtcmAddress::from_offset(core::mem::offset_of!(InitializedVendorImage, mac_beacon_state));
pub(crate) const MAC_BEACON_RESPONSE_COMMANDS: DtcmAddress = DtcmAddress::from_offset(MAC_BEACON_STATE.offset() + core::mem::offset_of!(MacBeaconState, response_commands));
pub(crate) const fn mac_beacon_response_command(index: usize) -> Option<DtcmAddress> { if index < 2 { Some(DtcmAddress::from_offset(MAC_BEACON_RESPONSE_COMMANDS.offset() + index * core::mem::size_of::<SharedU32>())) } else { None } }
pub(crate) const MAC_BEACON_CONTROL_STATE: DtcmAddress = DtcmAddress::from_offset(MAC_BEACON_STATE.offset() + core::mem::offset_of!(MacBeaconState, state));
pub(crate) const MAC_BEACON_SECONDARY_COMMAND: DtcmAddress = DtcmAddress::from_offset(MAC_BEACON_STATE.offset() + core::mem::offset_of!(MacBeaconState, secondary_command));
pub(crate) const MAC_BEACON_CONTROL: DtcmAddress = DtcmAddress::from_offset(MAC_BEACON_STATE.offset() + core::mem::offset_of!(MacBeaconState, control));
pub(crate) const MAC_BEACON_SELECTOR: DtcmAddress = DtcmAddress::from_offset(MAC_BEACON_STATE.offset() + core::mem::offset_of!(MacBeaconState, selector));
pub(crate) const MAC_BEACON_MODE: DtcmAddress = DtcmAddress::from_offset(MAC_BEACON_STATE.offset() + core::mem::offset_of!(MacBeaconState, mode));
pub(crate) const MAC_BEACON_COMPLETION_WORD: DtcmAddress = DtcmAddress::from_offset(MAC_BEACON_STATE.offset() + core::mem::offset_of!(MacBeaconState, completion_word));
pub(crate) const LOW_MAC_GLOBAL: DtcmAddress = DtcmAddress::from_offset(core::mem::offset_of!(InitializedVendorImage, initialized_low_mac_prefix));
pub(crate) const LOW_MAC_FIFO_STATUS: DtcmAddress = DtcmAddress::from_offset(LOW_MAC_GLOBAL.offset() + core::mem::offset_of!(LowMacGlobalPrefix, fifo_status));
pub(crate) const LOW_MAC_LEGACY_MODE: DtcmAddress = DtcmAddress::from_offset(LOW_MAC_GLOBAL.offset() + core::mem::offset_of!(LowMacGlobalPrefix, legacy_mode));
pub(crate) const LOW_MAC_SHORT_AIRTIME_TABLE: DtcmAddress = DtcmAddress::from_offset(LOW_MAC_GLOBAL.offset() + core::mem::offset_of!(LowMacGlobalPrefix, short_airtimes));
pub(crate) const LOW_MAC_LONG_AIRTIME_TABLE: DtcmAddress = DtcmAddress::from_offset(LOW_MAC_GLOBAL.offset() + core::mem::offset_of!(LowMacGlobalPrefix, long_airtimes));
pub(crate) const MAC_PIPE_RECORDS: DtcmAddress = DtcmAddress::from_offset(core::mem::offset_of!(InitializedVendorImage, mac_pipe_records));
pub(crate) const fn mac_pipe_record(pipe: usize) -> Option<DtcmAddress> { if pipe < 4 { Some(DtcmAddress::from_offset(MAC_PIPE_RECORDS.offset() + pipe * core::mem::size_of::<MacPipeRecord>())) } else { None } }
pub(crate) const RADIO_STOP_WORD_02: DtcmAddress = DtcmAddress::from_offset(core::mem::offset_of!(InitializedVendorImage, pre_host_pas_ring) + core::mem::offset_of!(PreHostPasRingObserved, radio_stop_word_02)); pub(crate) const HOST_PAS_RING: DtcmAddress = DtcmAddress::from_offset(core::mem::offset_of!(InitializedVendorImage, host_pas_ring));
pub(crate) const HOST_PAS_RING_HEAD: DtcmAddress = HOST_PAS_RING;
pub(crate) const HOST_PAS_RING_TAIL: DtcmAddress = DtcmAddress::from_offset(HOST_PAS_RING.offset() + core::mem::offset_of!(HostPasRing, tail));
pub(crate) const HOST_PAS_RING_SLOTS: DtcmAddress = DtcmAddress::from_offset(HOST_PAS_RING.offset() + core::mem::offset_of!(HostPasRing, slots));
pub(crate) const fn host_pas_ring_slot(index: usize) -> Option<DtcmAddress> { if index < 64 { Some(DtcmAddress::from_offset(HOST_PAS_RING_SLOTS.offset() + index * core::mem::size_of::<SharedU32>())) } else { None } }
pub(crate) const IRQ_CALLBACK_TABLE: DtcmAddress = DtcmAddress::from_offset(core::mem::offset_of!(InitializedVendorImage, irq_callbacks)); pub(crate) const PHY_WATCHDOG_COUNTER: DtcmAddress = DtcmAddress::from_offset(core::mem::offset_of!(InitializedVendorImage, phy_watchdog_counter) + core::mem::offset_of!(PhyWatchdogCounter, count)); pub(crate) const MULTI_VIF_BEACON_TIMER: DtcmAddress = DtcmAddress::from_offset(core::mem::offset_of!(InitializedVendorImage, multi_vif_beacon_timer_tail) + core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, timer)); pub(crate) const MEASUREMENT_DWELL_TIMER: DtcmAddress = DtcmAddress::from_offset(core::mem::offset_of!(InitializedVendorImage, multi_vif_beacon_timer_tail) + core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, measurement_dwell_timer));
pub(crate) const fn irq_callback(index: usize) -> Option<DtcmAddress> { if index < 32 { Some(DtcmAddress::from_offset(IRQ_CALLBACK_TABLE.offset() + index * core::mem::size_of::<SharedU32>())) } else { None } }
pub(crate) const VISIBLE_COMPLETION_WORDS: DtcmAddress = DtcmAddress::from_offset(core::mem::offset_of!(InitializedVendorImage, visible_completion_words));
pub(crate) const fn visible_completion_word(index: usize) -> Option<DtcmAddress> { if index < 10 { Some(DtcmAddress::from_offset(VISIBLE_COMPLETION_WORDS.offset() + index * core::mem::size_of::<SharedU32>())) } else { None } }
pub(crate) const TX_DURATION_TIMING_TABLE: DtcmAddress = DtcmAddress::from_offset(core::mem::offset_of!(InitializedVendorImage, tx_duration_timing));
pub(crate) const RATE_ENCODING_TABLE: DtcmAddress = DtcmAddress::from_offset(core::mem::offset_of!(InitializedVendorImage, rate_encoding));
pub(crate) const RATE_ATTRIBUTE_TABLE: DtcmAddress = DtcmAddress::from_offset(core::mem::offset_of!(InitializedVendorImage, rate_attributes));
pub(crate) const fn tx_duration_timing(rate: usize) -> Option<DtcmAddress> { if rate < 10 { Some(DtcmAddress::from_offset(TX_DURATION_TIMING_TABLE.offset() + rate * core::mem::size_of::<SharedU16>())) } else { None } }
pub(crate) const fn rate_encoding(rate: usize) -> Option<DtcmAddress> { if rate < 22 { Some(DtcmAddress::from_offset(RATE_ENCODING_TABLE.offset() + rate)) } else { None } }
pub(crate) const fn rate_attribute(rate: usize) -> Option<DtcmAddress> { if rate < 22 { Some(DtcmAddress::from_offset(RATE_ATTRIBUTE_TABLE.offset() + rate)) } else { None } }
pub(crate) const DURATION_QUANTUM_POINTERS: DtcmAddress = DtcmAddress::from_offset(core::mem::offset_of!(InitializedVendorImage, duration_quantum_pointers));
pub(crate) const fn duration_quantum_pointer(pipe: usize) -> Option<DtcmAddress> { if pipe < 4 { Some(DtcmAddress::from_offset(DURATION_QUANTUM_POINTERS.offset() + pipe * core::mem::size_of::<SharedU32>())) } else { None } }
pub(crate) const QUEUE_PIPE_MAPPINGS: DtcmAddress = DtcmAddress::from_offset(core::mem::offset_of!(InitializedVendorImage, queue_pipe_mappings));
pub(crate) const QUEUE_TO_ACCESS_CATEGORY: DtcmAddress = DtcmAddress::from_offset(QUEUE_PIPE_MAPPINGS.offset() + core::mem::offset_of!(QueuePipeMappings, queue_to_access_category));
pub(crate) const ACCESS_CATEGORY_TO_QUEUE: DtcmAddress = DtcmAddress::from_offset(QUEUE_PIPE_MAPPINGS.offset() + core::mem::offset_of!(QueuePipeMappings, access_category_to_queue));
pub(crate) const fn pipe_order_byte(index: usize) -> Option<DtcmAddress> { if index < 4 { Some(DtcmAddress::from_offset(QUEUE_PIPE_MAPPINGS.offset() + core::mem::offset_of!(QueuePipeMappings, pipe_order) + index)) } else { None } }
pub(crate) const fn queue_to_access_category(queue: usize) -> Option<DtcmAddress> { if queue < 4 { Some(DtcmAddress::from_offset(QUEUE_PIPE_MAPPINGS.offset() + core::mem::offset_of!(QueuePipeMappings, queue_to_access_category) + queue)) } else { None } }
pub(crate) const fn access_category_to_queue(access_category: usize) -> Option<DtcmAddress> { if access_category < 4 { Some(DtcmAddress::from_offset(QUEUE_PIPE_MAPPINGS.offset() + core::mem::offset_of!(QueuePipeMappings, access_category_to_queue) + access_category)) } else { None } }
pub(crate) const INITIALIZED_CONTROL_WORDS: DtcmAddress = DtcmAddress::from_offset(core::mem::offset_of!(InitializedVendorImage, control_words));
const fn initialized_control_field(offset: usize) -> DtcmAddress { DtcmAddress::from_offset(INITIALIZED_CONTROL_WORDS.offset() + offset) }
pub(crate) const fn initialized_beacon_state() -> DtcmAddress { initialized_control_field(core::mem::offset_of!(InitializedControlWords, beacon_state)) }
pub(crate) const fn initialized_rx_indication_state() -> DtcmAddress { initialized_control_field(core::mem::offset_of!(InitializedControlWords, rx_indication_state)) }
pub(crate) const fn initialized_tsf_resync_state() -> DtcmAddress { initialized_control_field(core::mem::offset_of!(InitializedControlWords, tsf_resync_state)) }
pub(crate) const fn initialized_random_lfsr() -> DtcmAddress { initialized_control_field(core::mem::offset_of!(InitializedControlWords, random_lfsr)) }
pub(crate) const fn initialized_tsf_accumulator_low() -> DtcmAddress { initialized_control_field(core::mem::offset_of!(InitializedControlWords, tsf_accumulator_low)) }
pub(crate) const fn initialized_timer_counter() -> DtcmAddress { initialized_control_field(core::mem::offset_of!(InitializedControlWords, timer_counter)) }
pub const SCHEDULER_EVENT_ROOT: DtcmAddress = DtcmAddress::from_offset(0x1fd4);
pub(crate) const RUNTIME_REGISTER_BACKOFF_STATE: DtcmAddress = DtcmAddress::from_offset(0x2078);
const fn runtime_register_backoff_field(offset: usize) -> DtcmAddress { DtcmAddress::from_offset(RUNTIME_REGISTER_BACKOFF_STATE.offset() + offset) }
pub(crate) const fn runtime_register_context(index: usize) -> Option<DtcmAddress> { if index < 4 { Some(runtime_register_context_unchecked(index)) } else { None } }
pub(crate) const fn runtime_register_context_unchecked(index: usize) -> DtcmAddress { runtime_register_backoff_field(core::mem::offset_of!(RuntimeRegisterBackoffState, register_context) + index * core::mem::size_of::<SharedU32>()) }
pub(crate) const fn pas_backoff_override_enabled() -> DtcmAddress { runtime_register_backoff_field(core::mem::offset_of!(RuntimeRegisterBackoffState, override_enabled)) }
pub(crate) const fn pas_backoff_override_window() -> DtcmAddress { runtime_register_backoff_field(core::mem::offset_of!(RuntimeRegisterBackoffState, override_window)) }
pub(crate) const DEBUG_CONSOLE_STATE: DtcmAddress = DtcmAddress::from_offset(0x2094);
pub(crate) const fn debug_console_command(index: usize) -> Option<DtcmAddress> { if index < 32 { Some(DtcmAddress::from_offset(DEBUG_CONSOLE_STATE.offset() + core::mem::offset_of!(DebugConsoleState, commands) + index * core::mem::size_of::<SharedU32>())) } else { None } }
pub(crate) const fn debug_console_line_byte(index: usize) -> Option<DtcmAddress> { if index < 80 { Some(DtcmAddress::from_offset(DEBUG_CONSOLE_STATE.offset() + core::mem::offset_of!(DebugConsoleState, line_buffer) + index)) } else { None } }
pub const CLOCK_PARAMETERS: DtcmAddress = DtcmAddress::from_offset(0x218c);
pub const SCHEDULER_HANDLER_TABLE: DtcmAddress = DtcmAddress::from_offset(0x21b4);
pub(crate) const PHY_GAIN_SOURCE_RECORDS: DtcmAddress = DtcmAddress::from_offset(0x2234);
pub(crate) const fn phy_gain_source_record(index: usize) -> Option<DtcmAddress> { if index < 22 { Some(DtcmAddress::from_offset(PHY_GAIN_SOURCE_RECORDS.offset() + index * core::mem::size_of::<PhyGainSourceRecord>())) } else { None } }
pub(crate) const RF_INITIALIZATION_VIEW: DtcmAddress = DtcmAddress::from_offset(0x2684);
pub(crate) const RF_INITIALIZATION_ROOT: DtcmAddress = DtcmAddress::from_offset(0x2730);
pub(crate) const BEACON_FILTER_STORAGE: DtcmAddress = DtcmAddress::from_offset(0x22b8);
pub(crate) const fn beacon_stored_byte(index: usize) -> Option<DtcmAddress> { if index < 700 { Some(DtcmAddress::from_offset(BEACON_FILTER_STORAGE.offset() + core::mem::offset_of!(BeaconFilterStorage, stored_beacon) + index)) } else { None } }
pub(crate) const BEACON_IE_INDEX_SELECTOR: DtcmAddress = DtcmAddress::from_offset(0x2578);
pub(crate) const fn beacon_ie_index(index: usize) -> Option<DtcmAddress> { match index { 0 => Some(DtcmAddress::from_offset(0x257c)), 1 => Some(DtcmAddress::from_offset(0x2780)), _ => None } }
pub(crate) const fn beacon_ie_offset(index: usize, entry: usize) -> Option<DtcmAddress> { if entry >= 256 { return None; } match beacon_ie_index(index) { Some(base) => Some(DtcmAddress::from_offset(base.offset() + core::mem::offset_of!(BeaconIeOffsetIndex, offsets) + entry * core::mem::size_of::<SharedU16>())), None => None } }
pub(crate) const TEMPLATE_FRAME_DESCRIPTORS: DtcmAddress = DtcmAddress::from_offset(0x3050);
pub(crate) const fn template_frame_descriptor(index: usize) -> Option<DtcmAddress> { if index < 2 { Some(DtcmAddress::from_offset(TEMPLATE_FRAME_DESCRIPTORS.offset() + index * core::mem::size_of::<TemplateFrameDescriptor>())) } else { None } }
pub(crate) const TEMPLATE_BACKING_STORAGE: DtcmAddress = DtcmAddress::from_offset(0x30d0);
pub(crate) const fn template_primary_buffer(index: usize) -> Option<DtcmAddress> { if index < 2 { Some(DtcmAddress::from_offset(TEMPLATE_BACKING_STORAGE.offset() + index * 0x100)) } else { None } }
pub(crate) const fn template_secondary_buffer(index: usize) -> Option<DtcmAddress> { if index < 2 { Some(DtcmAddress::from_offset(TEMPLATE_BACKING_STORAGE.offset() + core::mem::offset_of!(TemplateBackingStorage, secondary) + index * 0x60)) } else { None } }
pub(crate) const fn template_tertiary_buffer(index: usize) -> Option<DtcmAddress> { if index < 2 { Some(DtcmAddress::from_offset(TEMPLATE_BACKING_STORAGE.offset() + core::mem::offset_of!(TemplateBackingStorage, tertiary) + index * 0x90)) } else { None } }
pub(crate) const SDD_CONFIGURATION_TABLES: DtcmAddress = DtcmAddress::from_offset(0x34b0);
const fn sdd_profile_field(profile: usize, offset: usize) -> DtcmAddress { DtcmAddress::from_offset_unchecked(SDD_CONFIGURATION_TABLES.offset().wrapping_add(profile.wrapping_mul(core::mem::size_of::<SddProfileBank>())).wrapping_add(offset)) }
pub(crate) const fn sdd_profile(profile: usize) -> Option<DtcmAddress> { if profile < 2 { Some(sdd_profile_unchecked(profile)) } else { None } }
pub(crate) const fn sdd_profile_unchecked(profile: usize) -> DtcmAddress { sdd_profile_field(profile, 0) }
pub(crate) const fn sdd_rate_limit_unchecked(profile: usize, rate: usize) -> DtcmAddress { sdd_profile_field(profile, core::mem::offset_of!(SddProfileBank, rate_limits).wrapping_add(rate.wrapping_mul(core::mem::size_of::<SharedU16>()))) }
pub(crate) const fn sdd_channel_record_byte_unchecked(profile: usize, record: usize, byte: usize) -> DtcmAddress { sdd_profile_field(profile, core::mem::offset_of!(SddProfileBank, channel_records).wrapping_add(record.wrapping_mul(core::mem::size_of::<SddChannelRecord>())).wrapping_add(byte)) }
pub(crate) const fn sdd_channel_byte_unchecked(profile: usize, index: usize) -> DtcmAddress { sdd_profile_field(profile, core::mem::offset_of!(SddProfileBank, channel_records).wrapping_add(index)) }
pub(crate) const fn sdd_channel_count(profile: usize) -> Option<DtcmAddress> { if profile < 2 { Some(sdd_channel_count_unchecked(profile)) } else { None } }
pub(crate) const fn sdd_channel_count_unchecked(profile: usize) -> DtcmAddress { sdd_profile_field(profile, core::mem::offset_of!(SddProfileBank, channel_count)) }
pub(crate) const fn sdd_agc_correction_unchecked(profile: usize) -> DtcmAddress { sdd_profile_field(profile, core::mem::offset_of!(SddProfileBank, agc_correction)) }
pub(crate) const fn sdd_calibration_coefficient_unchecked(profile: usize) -> DtcmAddress { sdd_profile_field(profile, core::mem::offset_of!(SddProfileBank, calibration_coefficient)) }
pub(crate) const fn sdd_conversion_value_unchecked(profile: usize, index: usize) -> DtcmAddress { sdd_profile_field(profile, core::mem::offset_of!(SddProfileBank, conversion_pair).wrapping_add(index.wrapping_mul(core::mem::size_of::<SharedU16>()))) }
pub(crate) const fn sdd_rssi_coefficient_unchecked(profile: usize, index: usize) -> DtcmAddress { sdd_profile_field(profile, core::mem::offset_of!(SddProfileBank, rssi_coefficients).wrapping_add(index.wrapping_mul(core::mem::size_of::<SharedU16>()))) }
pub(crate) const fn sdd_rssi_rate_scale_unchecked(profile: usize, rate: usize) -> DtcmAddress { sdd_profile_field(profile, core::mem::offset_of!(SddProfileBank, rssi_rate_scales).wrapping_add(rate.wrapping_mul(core::mem::size_of::<SharedU16>()))) }
pub(crate) const fn sdd_gain_coefficient_unchecked(index: usize) -> DtcmAddress { sdd_profile_field(1, core::mem::offset_of!(SddProfileBank, opaque_6a).wrapping_add(index.wrapping_mul(core::mem::size_of::<SharedU16>()))) }
pub(crate) const WAKE_CONTEXT_STATE: DtcmAddress = DtcmAddress::from_offset(0x35e0);
pub(crate) const fn wake_clock_word(index: usize) -> Option<DtcmAddress> { if index < 4 { Some(DtcmAddress::from_offset(WAKE_CONTEXT_STATE.offset() + core::mem::offset_of!(WakeContextState, clock_words) + index * core::mem::size_of::<SharedU32>())) } else { None } }
pub(crate) const fn wake_response_pointer_unchecked(index: usize) -> DtcmAddress { DtcmAddress::from_offset_unchecked(WAKE_CONTEXT_STATE.offset().wrapping_add(core::mem::offset_of!(WakeContextState, response_pointers)).wrapping_add(index.wrapping_mul(core::mem::size_of::<SharedU32>()))) }
pub(crate) const DURATION_SOURCES: DtcmAddress = DtcmAddress::from_offset(0x3670);
pub(crate) const fn duration_source(index: usize) -> Option<DtcmAddress> { if index < 2 { Some(DtcmAddress::from_offset(DURATION_SOURCES.offset() + index * core::mem::size_of::<SharedU16>())) } else { None } }
pub const LOW_MAC_PAS_ROOT: DtcmAddress = DtcmAddress::from_offset(LOW_MAC_PAS_OFFSET);
pub(crate) const PRE_VIF_LINK_BITMAP: DtcmAddress = DtcmAddress::from_offset(
    core::mem::offset_of!(DtcmLayout, pre_vif_header)
        + core::mem::offset_of!(PreVifHeader, link_bitmap),
);
pub const VIF_RECORDS: DtcmAddress = DtcmAddress::from_offset(0x3e98);
pub const VIF_RECORD_END: usize = VIF_RECORDS.get() + VIF_RECORD_COUNT * VIF_RECORD_SIZE;
pub const HOST_TX_CONTEXTS: DtcmAddress = DtcmAddress::from_offset(0x5a24);
pub const COMMAND_CHANNEL_SWITCH_OVERLAY: DtcmAddress = DtcmAddress::from_offset(0x8594);
pub const HOST_TX_CONTEXT_FREE_HEAD: DtcmAddress = HOST_CONTEXT_FREE_HEAD;
pub const LINK_SEQUENCE_ROOT: DtcmAddress = DtcmAddress::from_offset(0x87b8);

const fn link_state_field(offset: usize) -> DtcmAddress {
    DtcmAddress::from_offset_unchecked(
        core::mem::offset_of!(DtcmLayout, link_and_sequence) + offset,
    )
}

pub(crate) const fn link_map_entry_count() -> DtcmAddress {
    link_state_field(
        core::mem::offset_of!(LinkAndSequenceState, header)
            + core::mem::offset_of!(LinkMapHeader, entry_count),
    )
}

pub(crate) const fn link_release_blocked_links() -> DtcmAddress {
    link_state_field(
        core::mem::offset_of!(LinkAndSequenceState, header)
            + core::mem::offset_of!(LinkMapHeader, release_blocked_links),
    )
}

pub(crate) const fn link_map_entry(index: usize) -> Option<LinkMapEntryAddress> {
    if index < LINK_MAP_ENTRY_COUNT {
        Some(link_map_entry_unchecked(index))
    } else {
        None
    }
}

/// Intentionally preserves vendor count-driven record arithmetic.
pub(crate) const fn link_map_entry_unchecked(index: usize) -> LinkMapEntryAddress {
    LinkMapEntryAddress(link_state_field(
        core::mem::offset_of!(LinkAndSequenceState, entries)
            + index * core::mem::size_of::<LinkMapEntry>(),
    ))
}

impl LinkMapEntryAddress {
    const fn field(self, offset: usize) -> DtcmAddress {
        DtcmAddress::from_offset_unchecked(self.0.offset() + offset)
    }

    pub(crate) const fn host_link(self) -> DtcmAddress {
        self.field(core::mem::offset_of!(LinkMapEntry, host_link))
    }
    pub(crate) const fn interface(self) -> DtcmAddress {
        self.field(core::mem::offset_of!(LinkMapEntry, interface))
    }
    pub(crate) const fn internal_link(self) -> DtcmAddress {
        self.field(core::mem::offset_of!(LinkMapEntry, internal_link))
    }
    pub(crate) const fn inactivity(self) -> DtcmAddress {
        self.field(core::mem::offset_of!(LinkMapEntry, inactivity))
    }
    pub(crate) const fn release_flags(self) -> DtcmAddress {
        self.field(core::mem::offset_of!(LinkMapEntry, release_flags))
    }
    pub(crate) const fn auxiliary_flags(self) -> DtcmAddress {
        self.field(core::mem::offset_of!(LinkMapEntry, auxiliary_flags))
    }
    pub(crate) const fn mac_byte(self, index: usize) -> Option<DtcmAddress> {
        if index < 6 {
            Some(self.field(core::mem::offset_of!(LinkMapEntry, mac_address) + index))
        } else {
            None
        }
    }
}

pub(crate) const fn link_sequence_counter(
    internal_link: usize,
    tid: usize,
) -> Option<DtcmAddress> {
    if internal_link < INTERNAL_LINK_SLOT_COUNT && tid < LINK_TID_COUNT {
        Some(link_sequence_counter_unchecked(internal_link, tid))
    } else {
        None
    }
}

/// Intentionally preserves vendor internal-link/TID address arithmetic.
pub(crate) const fn link_sequence_counter_unchecked(
    internal_link: usize,
    tid: usize,
) -> DtcmAddress {
    link_state_field(
        core::mem::offset_of!(LinkAndSequenceState, sequences)
            + internal_link * core::mem::size_of::<[SharedU16; LINK_TID_COUNT]>()
            + tid * core::mem::size_of::<SharedU16>(),
    )
}

pub(crate) const fn internal_link_bitmap() -> DtcmAddress {
    link_state_field(core::mem::offset_of!(
        LinkAndSequenceState,
        internal_link_bitmap
    ))
}

pub const TALA_ACCOUNTING: DtcmAddress = DtcmAddress::from_offset(0x8f48);
pub const CONTEXT_COMPLETION_PREFIX: DtcmAddress = DtcmAddress::from_offset(0x8f6c);
const fn context_completion_field(offset: usize) -> DtcmAddress { DtcmAddress::from_offset(CONTEXT_COMPLETION_PREFIX.offset() + offset) }
pub(crate) const fn class0_internal_context_count() -> DtcmAddress { context_completion_field(core::mem::offset_of!(ContextCompletionPrefix, class0_count)) }
pub(crate) const COMPLETION_RING_VIEW: DtcmAddress = DtcmAddress::from_offset(0x8f80);
pub(crate) const fn completion_ring_entry(index: usize) -> Option<DtcmAddress> { if index < 64 { Some(DtcmAddress::from_offset(COMPLETION_RING_VIEW.offset() + index * core::mem::size_of::<SharedU32>())) } else { None } }
pub const INTERNAL_CONTEXT_PREFIX: DtcmAddress = DtcmAddress::from_offset(0x906c);
pub(crate) const fn internal_context_iv_seed() -> DtcmAddress { INTERNAL_CONTEXT_PREFIX }
pub const INTERNAL_CONTEXT_POOL: DtcmAddress = DtcmAddress::from_offset(0x9080);
pub const POWER_SAVE_FAMILY: DtcmAddress = DtcmAddress::from_offset(0x94d4);

/// Returns one observed stride-based view start inside the opaque power-save
/// family. This does not confer record ownership or resolve overlapping extents.
pub fn power_save_observed_view(index: usize) -> Option<DtcmAddress> {
    let offset = index.checked_mul(POWER_SAVE_OBSERVED_STRIDE)?;
    (offset < 0x208).then(|| DtcmAddress::from_offset(POWER_SAVE_FAMILY.offset() + offset))
}
fn power_save_observed_field(interface: usize, offset: usize) -> Option<DtcmAddress> {
    power_save_observed_view(interface).map(|base| DtcmAddress::from_offset(base.offset() + offset))
}
pub(crate) fn power_save_wake_duration(interface: usize) -> Option<DtcmAddress> { power_save_observed_field(interface, core::mem::offset_of!(PowerSaveObservedLayout, wake_duration)) }
pub(crate) fn power_save_wake_register_min(interface: usize) -> Option<DtcmAddress> { power_save_observed_field(interface, core::mem::offset_of!(PowerSaveObservedLayout, wake_register_min)) }
pub(crate) fn power_save_wake_elapsed_max(interface: usize) -> Option<DtcmAddress> { power_save_observed_field(interface, core::mem::offset_of!(PowerSaveObservedLayout, wake_elapsed_max)) }
pub(crate) fn power_save_tx_completion_state(interface: usize) -> Option<DtcmAddress> { power_save_observed_field(interface, core::mem::offset_of!(PowerSaveObservedLayout, tx_completion_state)) }
pub(crate) fn power_save_next_tbtt(interface: usize) -> Option<DtcmAddress> { power_save_observed_field(interface, core::mem::offset_of!(PowerSaveObservedLayout, next_tbtt)) }
pub(crate) fn power_save_doze_state(interface: usize) -> Option<DtcmAddress> { power_save_observed_field(interface, core::mem::offset_of!(PowerSaveObservedLayout, doze_state)) }
pub(crate) fn power_save_requested_pm_mode(interface: usize) -> Option<DtcmAddress> { power_save_observed_field(interface, core::mem::offset_of!(PowerSaveObservedLayout, requested_pm_mode)) }
pub(crate) fn power_save_wake_stats_counter(interface: usize) -> Option<DtcmAddress> { power_save_observed_field(interface, core::mem::offset_of!(PowerSaveObservedLayout, wake_stats_counter)) }
pub(crate) fn power_save_beacon_timing_reference(interface: usize) -> Option<DtcmAddress> { power_save_observed_field(interface, core::mem::offset_of!(PowerSaveObservedLayout, beacon_timing_reference)) }
pub(crate) fn power_save_join_state_word(interface: usize) -> Option<DtcmAddress> { power_save_observed_field(interface, core::mem::offset_of!(PowerSaveObservedLayout, join_state_word)) }
pub(crate) fn power_save_wake_lead_time(interface: usize) -> Option<DtcmAddress> { power_save_observed_field(interface, core::mem::offset_of!(PowerSaveObservedLayout, wake_lead_time)) }
pub(crate) fn power_save_active(interface: usize) -> Option<DtcmAddress> { power_save_observed_field(interface, core::mem::offset_of!(PowerSaveObservedLayout, active)) }
pub(crate) fn power_save_sleep_transition_flags(interface: usize) -> Option<DtcmAddress> { power_save_observed_field(interface, core::mem::offset_of!(PowerSaveObservedLayout, sleep_transition_flags)) }
pub(crate) fn power_save_wake_stats_timestamp(interface: usize) -> Option<DtcmAddress> { power_save_observed_field(interface, core::mem::offset_of!(PowerSaveObservedLayout, wake_stats_timestamp)) }
pub(crate) fn power_save_backoff_adjustment(interface: usize) -> Option<DtcmAddress> { power_save_observed_field(interface, core::mem::offset_of!(PowerSaveObservedLayout, backoff_adjustment)) }
pub(crate) fn power_save_pending_control_kind(interface: usize) -> Option<DtcmAddress> { power_save_observed_field(interface, core::mem::offset_of!(PowerSaveObservedLayout, pending_control_kind)) }
pub(crate) fn power_save_pending_flags(interface: usize) -> Option<DtcmAddress> { power_save_observed_field(interface, core::mem::offset_of!(PowerSaveObservedLayout, pending_flags)) }
pub(crate) fn power_save_uapsd_restart_value(interface: usize) -> Option<DtcmAddress> { power_save_observed_field(interface, core::mem::offset_of!(PowerSaveObservedLayout, uapsd_restart_value)) }
pub(crate) fn power_save_tx_completion_pending(interface: usize) -> Option<DtcmAddress> { power_save_observed_field(interface, core::mem::offset_of!(PowerSaveObservedLayout, tx_completion_pending)) }
pub(crate) fn power_save_beacon_rx_state(interface: usize) -> Option<DtcmAddress> { power_save_observed_field(interface, core::mem::offset_of!(PowerSaveObservedLayout, beacon_rx_state)) }
pub(crate) fn power_save_beacon_rate(interface: usize) -> Option<DtcmAddress> { power_save_observed_field(interface, core::mem::offset_of!(PowerSaveObservedLayout, beacon_rate)) }
pub(crate) fn power_save_wake_timer_delay(interface: usize) -> Option<DtcmAddress> { power_save_observed_field(interface, core::mem::offset_of!(PowerSaveObservedLayout, wake_timer_delay)) }
pub(crate) fn power_save_beacon_airtime(interface: usize) -> Option<DtcmAddress> { power_save_observed_field(interface, core::mem::offset_of!(PowerSaveObservedLayout, beacon_airtime)) }
pub(crate) fn power_save_pre_tbtt_offset(interface: usize) -> Option<DtcmAddress> { power_save_observed_field(interface, core::mem::offset_of!(PowerSaveObservedLayout, pre_tbtt_offset)) }
pub(crate) fn power_save_beacon_interval(interface: usize) -> Option<DtcmAddress> { power_save_observed_field(interface, core::mem::offset_of!(PowerSaveObservedLayout, beacon_interval)) }
pub(crate) fn power_save_next_wake_deadline(interface: usize) -> Option<DtcmAddress> { power_save_observed_field(interface, core::mem::offset_of!(PowerSaveObservedLayout, next_wake_deadline)) }
pub(crate) fn power_save_wake_timer_active(interface: usize) -> Option<DtcmAddress> { power_save_observed_field(interface, core::mem::offset_of!(PowerSaveObservedLayout, wake_timer_active)) }
pub(crate) fn power_save_beacon_timing_adjusted(interface: usize) -> Option<DtcmAddress> { power_save_observed_field(interface, core::mem::offset_of!(PowerSaveObservedLayout, beacon_timing_adjusted)) }
pub(crate) fn power_save_mode_control(interface: usize) -> Option<DtcmAddress> { power_save_observed_field(interface, core::mem::offset_of!(PowerSaveObservedLayout, mode_control_124)) }
pub(crate) fn power_save_maximum_backoff(interface: usize) -> Option<DtcmAddress> { power_save_observed_field(interface, core::mem::offset_of!(PowerSaveObservedLayout, maximum_backoff_12c)) }
pub(crate) fn power_save_last_beacon_timestamp(interface: usize) -> Option<DtcmAddress> { power_save_observed_field(interface, core::mem::offset_of!(PowerSaveObservedLayout, last_beacon_timestamp_130)) }
pub(crate) fn power_save_ps_mode_error_reported(interface: usize) -> Option<DtcmAddress> { power_save_observed_field(interface, core::mem::offset_of!(PowerSaveObservedLayout, ps_mode_error_reported_13c)) }
pub(crate) fn power_save_timer(interface: usize, timer: usize) -> Option<DtcmAddress> {
    if timer < 7 {
        power_save_observed_field(interface, core::mem::offset_of!(PowerSaveObservedLayout, timers) + timer * core::mem::size_of::<TimerEntry>())
    } else { None }
}
pub(crate) const fn power_save_global_sleep_state() -> DtcmAddress {
    DtcmAddress::from_offset(POWER_SAVE_FAMILY.offset() + core::mem::offset_of!(PowerSaveObservedLayout, global_sleep_state))
}
pub(crate) const fn power_save_global_timer_duration() -> DtcmAddress {
    DtcmAddress::from_offset(POWER_SAVE_FAMILY.offset() + core::mem::offset_of!(PowerSaveObservedLayout, global_timer_duration))
}
pub(crate) fn power_save_scan_completion(interface: usize) -> Option<DtcmAddress> {
    power_save_observed_field(interface, core::mem::offset_of!(PowerSaveObservedLayout, scan_completion_138))
}
pub(crate) fn power_save_sleep_vote_count(interface: usize) -> Option<DtcmAddress> {
    power_save_observed_field(interface, core::mem::offset_of!(PowerSaveObservedLayout, sleep_vote_count_13a))
}
pub(crate) const POWER_SAVE_BEACON_TIM_STATE: DtcmAddress = DtcmAddress::from_offset(
    core::mem::offset_of!(DtcmLayout, power_save_hif_boundary)
        + core::mem::offset_of!(PowerSaveHifBoundary, beacon_tim_state_40),
);
#[cfg(test)]
fn power_save_extension_field(interface: usize, offset: usize) -> Option<DtcmAddress> {
    power_save_observed_field(interface, offset)
}
pub const HIF_BUFFER_STATE: DtcmAddress = DtcmAddress::from_offset(0x9720);
#[cfg(test)]
const fn hif_buffer_field(offset: usize) -> DtcmAddress {
    DtcmAddress::from_offset(HIF_BUFFER_STATE.offset() + offset)
}
#[cfg(test)]
const fn legacy_hif_field(offset: usize) -> DtcmAddress {
    DtcmAddress::from_offset(core::mem::offset_of!(DtcmLayout, legacy_hif_software_state) + offset)
}
pub const MIC_COMPLETION_STATE: DtcmAddress = DtcmAddress::from_offset(0x9928);
#[cfg(test)]
const fn mic_completion_field(offset: usize) -> DtcmAddress {
    DtcmAddress::from_offset(MIC_COMPLETION_STATE.offset() + offset)
}
pub const PHY_STATE: DtcmAddress = DtcmAddress::from_offset(0x993c);
const fn phy_reference_field(offset: usize) -> DtcmAddress {
    DtcmAddress::from_offset(PHY_STATE.offset() + core::mem::offset_of!(PhyCoreState, references) + offset)
}
pub(crate) const fn phy_coefficient_i() -> DtcmAddress { phy_reference_field(core::mem::offset_of!(PhyCalibrationReferences, coefficient_i)) }
pub(crate) const fn phy_coefficient_q() -> DtcmAddress { phy_reference_field(core::mem::offset_of!(PhyCalibrationReferences, coefficient_q)) }
pub(crate) const fn phy_scale_i() -> DtcmAddress { phy_reference_field(core::mem::offset_of!(PhyCalibrationReferences, scale_i)) }
pub(crate) const fn phy_scale_i_byte(index: usize) -> Option<DtcmAddress> { if index < 4 { Some(phy_scale_i_byte_unchecked(index)) } else { None } }
pub(crate) const fn phy_scale_i_byte_unchecked(index: usize) -> DtcmAddress { phy_reference_field(core::mem::offset_of!(PhyCalibrationReferences, scale_i) + index) }
pub(crate) const fn phy_scale_q() -> DtcmAddress { phy_reference_field(core::mem::offset_of!(PhyCalibrationReferences, scale_q)) }
pub(crate) const PHY_PROFILE_STATE: DtcmAddress = DtcmAddress::from_offset(0x994c);
const fn phy_profile_field(offset: usize) -> DtcmAddress { DtcmAddress::from_offset(PHY_PROFILE_STATE.offset() + offset) }
pub(crate) const fn phy_profile() -> DtcmAddress { phy_profile_field(core::mem::offset_of!(PhyProfileState, profile)) }
pub(crate) const fn phy_phase() -> DtcmAddress { phy_profile_field(core::mem::offset_of!(PhyProfileState, phase)) }
pub(crate) const fn phy_channel() -> DtcmAddress { phy_profile_field(core::mem::offset_of!(PhyProfileState, channel)) }
pub const fn phy_profile0_ready() -> DtcmAddress { phy_profile_field(core::mem::offset_of!(PhyProfileState, profile0_ready)) }
pub const fn phy_auxiliary_state() -> DtcmAddress { phy_profile_field(core::mem::offset_of!(PhyProfileState, auxiliary_state)) }
pub(crate) const fn phy_transition_gate() -> DtcmAddress { phy_profile_field(core::mem::offset_of!(PhyProfileState, transition_gate)) }
pub(crate) const fn phy_calibration_stage() -> DtcmAddress { phy_profile_field(core::mem::offset_of!(PhyProfileState, calibration_stage)) }
pub(crate) const fn phy_profile0_state() -> DtcmAddress { phy_profile_field(core::mem::offset_of!(PhyProfileState, profile0_state)) }
pub(crate) const fn phy_profile1_ready() -> DtcmAddress { phy_profile_field(core::mem::offset_of!(PhyProfileState, profile1_ready)) }
pub(crate) const fn phy_profile1_channel() -> DtcmAddress { phy_profile_field(core::mem::offset_of!(PhyProfileState, profile1_channel)) }
pub(crate) const fn phy_reference_word() -> DtcmAddress { phy_profile_field(core::mem::offset_of!(PhyProfileState, reference_word)) }
pub(crate) const PHY_MEASUREMENT_STATE: DtcmAddress = DtcmAddress::from_offset(0x9974);
const fn phy_measurement_field(offset: usize) -> DtcmAddress { DtcmAddress::from_offset(PHY_MEASUREMENT_STATE.offset() + offset) }
pub(crate) const fn phy_frequency_khz() -> DtcmAddress { phy_measurement_field(core::mem::offset_of!(PhyMeasurementState, frequency_khz)) }
pub(crate) const fn phy_measurement_control() -> DtcmAddress { phy_measurement_field(core::mem::offset_of!(PhyMeasurementState, control_08)) }
pub(crate) const fn phy_sample_width() -> DtcmAddress { phy_measurement_field(core::mem::offset_of!(PhyMeasurementState, sample_width)) }
pub(crate) const fn phy_offset_word() -> DtcmAddress { phy_measurement_field(core::mem::offset_of!(PhyMeasurementState, offset_word)) }
pub(crate) const fn phy_correction() -> DtcmAddress { phy_measurement_field(core::mem::offset_of!(PhyMeasurementState, correction)) }
pub(crate) const fn phy_override_value() -> DtcmAddress { phy_measurement_field(core::mem::offset_of!(PhyMeasurementState, override_value)) }
pub const fn phy_silicon_variant() -> DtcmAddress { phy_measurement_field(core::mem::offset_of!(PhyMeasurementState, silicon_variant)) }
pub(crate) const fn phy_denominator() -> DtcmAddress { phy_measurement_field(core::mem::offset_of!(PhyMeasurementState, denominator)) }
pub(crate) const fn phy_correction_offset() -> DtcmAddress { phy_measurement_field(core::mem::offset_of!(PhyMeasurementState, correction_offset)) }
pub(crate) const fn phy_measured_a() -> DtcmAddress { phy_measurement_field(core::mem::offset_of!(PhyMeasurementState, measured_a)) }
pub(crate) const fn phy_measured_b() -> DtcmAddress { phy_measurement_field(core::mem::offset_of!(PhyMeasurementState, measured_b)) }
pub(crate) const fn phy_retained_state() -> DtcmAddress { phy_measurement_field(core::mem::offset_of!(PhyMeasurementState, retained_state)) }
pub(crate) const fn phy_zero_select() -> DtcmAddress { phy_measurement_field(core::mem::offset_of!(PhyMeasurementState, zero_select)) }
pub(crate) const PHY_CHANNEL_CACHE_STATE: DtcmAddress = DtcmAddress::from_offset(0x99ac);
const fn phy_channel_cache_field(offset: usize) -> DtcmAddress { DtcmAddress::from_offset(PHY_CHANNEL_CACHE_STATE.offset() + offset) }
pub(crate) const fn phy_channel_cache(slot: usize) -> Option<DtcmAddress> { if slot < 2 { Some(phy_channel_cache_unchecked(slot)) } else { None } }
pub(crate) const fn phy_channel_cache_unchecked(slot: usize) -> DtcmAddress { phy_channel_cache_field(core::mem::offset_of!(PhyChannelCacheState, configuration_cache) + slot * core::mem::size_of::<SharedU32>()) }
pub const fn phy_startup_observation() -> DtcmAddress { phy_channel_cache_field(core::mem::offset_of!(PhyChannelCacheState, startup_observation)) }
pub(crate) const fn phy_retained_channel() -> DtcmAddress { phy_channel_cache_field(core::mem::offset_of!(PhyChannelCacheState, retained_channel)) }
pub(crate) const fn phy_calibration_state() -> DtcmAddress { phy_channel_cache_field(core::mem::offset_of!(PhyChannelCacheState, calibration_state)) }
pub(crate) const fn phy_calibration_aux() -> DtcmAddress { phy_channel_cache_field(core::mem::offset_of!(PhyChannelCacheState, calibration_aux)) }
pub(crate) const fn phy_control_word() -> DtcmAddress { phy_channel_cache_field(core::mem::offset_of!(PhyChannelCacheState, control_word)) }
pub(crate) const fn phy_table_pointer() -> DtcmAddress { phy_channel_cache_field(core::mem::offset_of!(PhyChannelCacheState, table_pointer)) }
pub(crate) const PHY_TABLE_CONTROL_STATE: DtcmAddress = DtcmAddress::from_offset(0x99dc);
const fn phy_table_control_field(offset: usize) -> DtcmAddress { DtcmAddress::from_offset(PHY_TABLE_CONTROL_STATE.offset() + offset) }
pub(crate) const fn phy_calibration_table_a() -> DtcmAddress { phy_table_control_field(core::mem::offset_of!(PhyTableControlState, table_a)) }
pub(crate) const fn phy_calibration_table_b() -> DtcmAddress { phy_table_control_field(core::mem::offset_of!(PhyTableControlState, table_b)) }
pub(crate) const fn phy_state_scale() -> DtcmAddress { phy_table_control_field(core::mem::offset_of!(PhyTableControlState, state_scale)) }
pub(crate) const fn phy_threshold() -> DtcmAddress { phy_table_control_field(core::mem::offset_of!(PhyTableControlState, threshold)) }
pub(crate) const fn phy_extended_settle() -> DtcmAddress { phy_table_control_field(core::mem::offset_of!(PhyTableControlState, extended_settle)) }
pub(crate) const fn phy_table_control() -> DtcmAddress { phy_table_control_field(core::mem::offset_of!(PhyTableControlState, control_2d)) }
pub(crate) const PHY_IQ_CALIBRATION_STATE: DtcmAddress = DtcmAddress::from_offset(0x9a0c);
pub(crate) const fn phy_iq_calibration_slot(page: usize, slot: usize) -> Option<DtcmAddress> { if page < 2 && slot < 12 { Some(phy_iq_calibration_slot_unchecked(page, slot)) } else { None } }
pub(crate) const fn phy_iq_calibration_slot_unchecked(page: usize, slot: usize) -> DtcmAddress { DtcmAddress::from_offset(PHY_IQ_CALIBRATION_STATE.offset() + page * core::mem::size_of::<PhyIqCalibrationPage>() + core::mem::offset_of!(PhyIqCalibrationPage, slots) + slot * core::mem::size_of::<PhyIqCalibrationSlot>()) }
pub(crate) const fn phy_iq_calibration_result(index: usize) -> Option<DtcmAddress> { if index < 9 { Some(DtcmAddress::from_offset(PHY_IQ_CALIBRATION_STATE.offset() + core::mem::offset_of!(PhyTail, results) + core::mem::offset_of!(PhyIqCalibrationResults, values) + index * core::mem::size_of::<SharedU32>())) } else { None } }
pub const VENDOR_BSS_START: DtcmAddress = DtcmAddress::from_offset(0x2078);
pub const VENDOR_BSS_END: DtcmAddress = DtcmAddress::from_offset(0x9c44);

/// Field-derived address of one physical VIF record. This carries no reference
/// and therefore makes no exclusive ownership claim over vendor-shared bytes.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct VifRecordAddress(DtcmAddress);

impl VifRecordAddress {
    const fn field(self, offset: usize) -> DtcmAddress {
        DtcmAddress::from_offset_unchecked(self.0.offset() + offset)
    }

    pub(crate) const fn scan_rate_config(self) -> DtcmAddress { self.field(core::mem::offset_of!(VifRecord, scan_rate_config)) }
    pub(crate) const fn scan_channel(self) -> DtcmAddress { self.field(core::mem::offset_of!(VifRecord, scan_channel)) }
    pub(crate) const fn scan_flags(self) -> DtcmAddress { self.field(core::mem::offset_of!(VifRecord, scan_flags)) }
    pub(crate) const fn host_contexts_in_flight(self) -> DtcmAddress { self.field(core::mem::offset_of!(VifRecord, host_contexts_in_flight)) }
    pub(crate) const fn wake_reinit_flag(self) -> DtcmAddress { self.field(core::mem::offset_of!(VifRecord, wake_reinit_flag)) }
    pub(crate) const fn mode(self) -> DtcmAddress { self.field(core::mem::offset_of!(VifRecord, mode)) }
    pub(crate) const fn active(self) -> DtcmAddress { self.field(core::mem::offset_of!(VifRecord, active)) }
    pub(crate) const fn interface(self) -> DtcmAddress { self.field(core::mem::offset_of!(VifRecord, interface)) }
    pub(crate) const fn role(self) -> DtcmAddress { self.field(core::mem::offset_of!(VifRecord, role)) }
    pub(crate) const fn flags(self) -> DtcmAddress { self.field(core::mem::offset_of!(VifRecord, flags)) }
    pub(crate) const fn rate_configuration(self) -> DtcmAddress { self.field(core::mem::offset_of!(VifRecord, rate_configuration)) }
    pub(crate) const fn rate_byte(self, index: usize) -> Option<DtcmAddress> { if index < 8 { Some(self.field(core::mem::offset_of!(VifRecord, rate_configuration) + index)) } else { None } }
    pub(crate) const fn basic_rates(self) -> DtcmAddress { self.field(core::mem::offset_of!(VifRecord, basic_rates)) }
    pub(crate) const fn allowed_links(self) -> DtcmAddress { self.field(core::mem::offset_of!(VifRecord, allowed_links)) }
    pub(crate) const fn effective_links(self) -> DtcmAddress { self.field(core::mem::offset_of!(VifRecord, effective_links)) }
    pub(crate) const fn tx_busy(self) -> DtcmAddress { self.field(core::mem::offset_of!(VifRecord, tx_busy)) }
    pub(crate) const fn own_mac_byte(self, index: usize) -> Option<DtcmAddress> { if index < 6 { Some(self.field(core::mem::offset_of!(VifRecord, own_mac) + index)) } else { None } }
    pub(crate) const fn bssid_byte(self, index: usize) -> Option<DtcmAddress> { if index < 6 { Some(self.field(core::mem::offset_of!(VifRecord, bssid) + index)) } else { None } }
    pub(crate) const fn channel(self) -> DtcmAddress { self.field(core::mem::offset_of!(VifRecord, channel)) }
    pub(crate) const fn radio_owner(self) -> DtcmAddress { self.field(core::mem::offset_of!(VifRecord, radio_owner_overlay)) }
    pub(crate) const fn operating_state(self) -> DtcmAddress { self.field(core::mem::offset_of!(VifRecord, operating_state)) }
    pub(crate) const fn owner_interface(self) -> DtcmAddress { self.field(core::mem::offset_of!(VifRecord, owner_interface)) }
    pub(crate) const fn owner_channel(self) -> DtcmAddress { self.field(core::mem::offset_of!(VifRecord, owner_channel)) }
    pub(crate) const fn owner_deadline(self) -> DtcmAddress { self.field(core::mem::offset_of!(VifRecord, owner_deadline)) }
    pub(crate) const fn owner_flags(self) -> DtcmAddress { self.field(core::mem::offset_of!(VifRecord, owner_flags)) }
    pub(crate) const fn activity_state(self) -> DtcmAddress { self.field(core::mem::offset_of!(VifRecord, activity_state)) }
    pub(crate) const fn operating_timer(self, index: usize) -> Option<DtcmAddress> { if index < 3 { Some(self.field(core::mem::offset_of!(VifRecord, operating_timers) + index * core::mem::size_of::<TimerEntry>())) } else { None } }
    pub(crate) const fn ssid_length(self) -> DtcmAddress { self.field(core::mem::offset_of!(VifRecord, ssid_length)) }
    pub(crate) const fn ssid_byte(self, index: usize) -> Option<DtcmAddress> { if index < 32 { Some(self.field(core::mem::offset_of!(VifRecord, ssid) + index)) } else { None } }
    pub(crate) const fn dtim_period(self) -> DtcmAddress { self.field(core::mem::offset_of!(VifRecord, dtim_period)) }
    pub(crate) const fn atim_window(self) -> DtcmAddress { self.field(core::mem::offset_of!(VifRecord, atim_window)) }
    pub(crate) const fn beacon_interval(self) -> DtcmAddress { self.field(core::mem::offset_of!(VifRecord, beacon_interval)) }
    pub(crate) const fn rts_threshold(self) -> DtcmAddress { self.field(core::mem::offset_of!(VifRecord, rts_threshold)) }
    pub(crate) const fn ampdu_length(self) -> DtcmAddress { self.field(core::mem::offset_of!(VifRecord, ampdu_length)) }
    pub(crate) const fn internal_link(self) -> DtcmAddress { self.field(core::mem::offset_of!(VifRecord, internal_link)) }
    pub(crate) const fn default_rate(self, index: usize) -> Option<DtcmAddress> { if index < 2 { Some(self.field(core::mem::offset_of!(VifRecord, default_rates) + index)) } else { None } }
    pub(crate) const fn link_object_flags(self) -> DtcmAddress { self.field(core::mem::offset_of!(VifRecord, link_object_flags)) }
    pub(crate) const fn link_own_mac_0_byte(self, index: usize) -> Option<DtcmAddress> { if index < 6 { Some(self.field(core::mem::offset_of!(VifRecord, link_own_mac_0) + index)) } else { None } }
    pub(crate) const fn link_own_mac_1_byte(self, index: usize) -> Option<DtcmAddress> { if index < 6 { Some(self.field(core::mem::offset_of!(VifRecord, link_own_mac_1) + index)) } else { None } }
    pub(crate) const fn link_bssid_byte(self, index: usize) -> Option<DtcmAddress> { if index < 6 { Some(self.field(core::mem::offset_of!(VifRecord, link_bssid) + index)) } else { None } }
    pub(crate) const fn sleeping_links(self) -> DtcmAddress { self.field(core::mem::offset_of!(VifRecord, sleeping_links)) }
    pub(crate) const fn awake_links(self) -> DtcmAddress { self.field(core::mem::offset_of!(VifRecord, awake_links)) }
    pub(crate) const fn buffered_links(self) -> DtcmAddress { self.field(core::mem::offset_of!(VifRecord, buffered_links)) }
    pub(crate) const fn link_gate(self) -> DtcmAddress { self.field(core::mem::offset_of!(VifRecord, link_gate)) }
    pub(crate) const fn link_timer(self, index: usize) -> Option<DtcmAddress> { if index < 2 { Some(self.field(core::mem::offset_of!(VifRecord, link_timers) + index * core::mem::size_of::<TimerEntry>())) } else { None } }
}

pub(crate) const fn vif_record(interface: usize) -> Option<VifRecordAddress> {
    if interface < VIF_RECORD_COUNT {
        Some(vif_record_unchecked(interface))
    } else {
        None
    }
}

/// Preserve parent-shaped base-plus-stride access where the caller has already
/// obtained a vendor-owned interface byte and the original code did not bound it.
#[inline(always)]
pub(crate) const fn vif_record_unchecked(interface: usize) -> VifRecordAddress {
    VifRecordAddress(DtcmAddress::from_offset_unchecked(
        core::mem::offset_of!(DtcmLayout, vifs) + interface * VIF_RECORD_SIZE,
    ))
}

pub const fn vif_record_address(interface: u8) -> Option<usize> {
    match vif_record(interface as usize) {
        Some(record) => Some(record.0.get()),
        None => None,
    }
}

/// Translate a typed DTCM address to its target pointer or process-local host
/// backing. The returned raw pointer confers no reference or ownership.
#[inline(always)]
pub(crate) fn shared_ptr<T>(address: DtcmAddress) -> *mut T {
    #[cfg(target_arch = "arm")]
    {
        address.cast_mut::<T>()
    }
    #[cfg(not(target_arch = "arm"))]
    {
        unsafe { layout_ptr().cast::<u8>().add(address.offset()).cast::<T>() }
    }
}

/// Address of one fixed-DTCM polymorphic LMC message record.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LmcMessageAddress(DtcmAddress);

const fn ba_lmc_header_field(offset: usize) -> DtcmAddress {
    DtcmAddress::from_offset_unchecked(core::mem::offset_of!(DtcmLayout, ba_lmc_header) + offset)
}

const fn pending_ba_lmc_field(offset: usize) -> DtcmAddress {
    DtcmAddress::from_offset_unchecked(core::mem::offset_of!(DtcmLayout, pending_ba_lmc) + offset)
}

#[cfg(test)]
pub(crate) const fn ba_policy_enabled() -> DtcmAddress {
    ba_lmc_header_field(core::mem::offset_of!(BaLmcHeader, ba_policy_enabled))
}

pub(crate) const fn pending_tx_head() -> DtcmAddress {
    pending_ba_lmc_field(core::mem::offset_of!(PendingBaLmcState, pending_head))
}

pub(crate) const fn pending_tx_tail() -> DtcmAddress {
    pending_ba_lmc_field(core::mem::offset_of!(PendingBaLmcState, pending_tail))
}

pub(crate) const fn mac_bssid_mode() -> DtcmAddress {
    pending_ba_lmc_field(core::mem::offset_of!(PendingBaLmcState, mac_bssid_mode))
}

pub(crate) const fn pending_service_needed() -> DtcmAddress {
    pending_ba_lmc_field(core::mem::offset_of!(
        PendingBaLmcState,
        pending_service_needed
    ))
}

pub(crate) const fn radio_owner() -> DtcmAddress {
    pending_ba_lmc_field(core::mem::offset_of!(PendingBaLmcState, radio_owner))
}

pub(crate) const fn radio_wait_head() -> DtcmAddress {
    pending_ba_lmc_field(core::mem::offset_of!(PendingBaLmcState, radio_wait_head))
}

pub(crate) const fn deferred_radio_owner() -> DtcmAddress {
    pending_ba_lmc_field(core::mem::offset_of!(
        PendingBaLmcState,
        deferred_radio_owner
    ))
}

pub(crate) const fn radio_timer_state() -> DtcmAddress {
    pending_ba_lmc_field(core::mem::offset_of!(PendingBaLmcState, radio_timer_state))
}

pub(crate) const fn lmc_message_control() -> DtcmAddress {
    pending_ba_lmc_field(core::mem::offset_of!(PendingBaLmcState, message_control))
}

pub(crate) const fn lmc_message_producer() -> DtcmAddress {
    pending_ba_lmc_field(core::mem::offset_of!(PendingBaLmcState, message_producer))
}

pub(crate) const fn lmc_message_consumer() -> DtcmAddress {
    pending_ba_lmc_field(core::mem::offset_of!(PendingBaLmcState, message_consumer))
}

#[cfg(test)]
pub(crate) const fn lmc_message(index: usize) -> Option<LmcMessageAddress> {
    if index < LMC_MESSAGE_COUNT {
        Some(lmc_message_unchecked(index))
    } else {
        None
    }
}

/// Intentionally preserves the vendor ring-index address calculation.
pub(crate) const fn lmc_message_unchecked(index: usize) -> LmcMessageAddress {
    LmcMessageAddress(DtcmAddress::from_offset_unchecked(
        core::mem::offset_of!(DtcmLayout, lmc_messages)
            + core::mem::offset_of!(LmcMessages, records)
            + index * core::mem::size_of::<LmcMessage>(),
    ))
}

impl LmcMessageAddress {
    const fn field(self, offset: usize) -> DtcmAddress {
        DtcmAddress::from_offset_unchecked(self.0.offset() + offset)
    }

    #[cfg(test)]
    pub(crate) const fn raw(self) -> u32 {
        self.0.get() as u32
    }

    pub(crate) const fn kind(self) -> DtcmAddress {
        self.field(core::mem::offset_of!(LmcMessage, kind))
    }

    #[cfg(test)]
    pub(crate) const fn flags(self) -> DtcmAddress {
        self.field(core::mem::offset_of!(LmcMessage, flags))
    }

    pub(crate) const fn completion_tid(self) -> DtcmAddress {
        self.field(core::mem::offset_of!(LmcMessage, payload))
    }

    pub(crate) const fn completion_queue(self) -> DtcmAddress {
        self.field(core::mem::offset_of!(LmcMessage, payload) + 1)
    }

    pub(crate) const fn completion_sequence(self) -> DtcmAddress {
        self.field(core::mem::offset_of!(LmcMessage, payload) + 2)
    }

    pub(crate) const fn completion_mac_word(self, index: usize) -> Option<DtcmAddress> {
        if index < 3 {
            Some(self.field(
                core::mem::offset_of!(LmcMessage, payload) + 4 + index * 2,
            ))
        } else {
            None
        }
    }

    pub(crate) const fn interface(self) -> DtcmAddress {
        self.field(core::mem::offset_of!(LmcMessage, interface))
    }

    pub(crate) const fn completion_state(self) -> DtcmAddress {
        self.field(core::mem::offset_of!(LmcMessage, completion_state))
    }
}

#[cfg(test)]
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct BaSessionAddress(DtcmAddress);

#[cfg(test)]
const fn ba_session(index: usize) -> Option<BaSessionAddress> {
    if index < 4 { Some(ba_session_unchecked(index)) } else { None }
}

#[cfg(test)]
const fn ba_session_unchecked(index: usize) -> BaSessionAddress {
    BaSessionAddress(DtcmAddress::from_offset_unchecked(
        core::mem::offset_of!(DtcmLayout, ba_sessions)
            + core::mem::offset_of!(BaSessions, records)
            + index * core::mem::size_of::<BaSession>(),
    ))
}

#[cfg(test)]
impl BaSessionAddress {
    const fn field(self, offset: usize) -> DtcmAddress {
        DtcmAddress::from_offset_unchecked(self.0.offset() + offset)
    }
    const fn raw(self) -> u32 { self.0.get() as u32 }
    const fn activity(self) -> DtcmAddress { self.field(core::mem::offset_of!(BaSession, activity)) }
    const fn peer_mac_byte(self, index: usize) -> Option<DtcmAddress> {
        if index < 6 { Some(self.field(core::mem::offset_of!(BaSession, peer_mac) + index)) } else { None }
    }
    const fn tid(self) -> DtcmAddress { self.field(core::mem::offset_of!(BaSession, tid)) }
    const fn interface(self) -> DtcmAddress { self.field(core::mem::offset_of!(BaSession, interface)) }
    const fn timeout_1024us(self) -> DtcmAddress { self.field(core::mem::offset_of!(BaSession, timeout_1024us)) }
    const fn timer(self) -> DtcmAddress { self.field(core::mem::offset_of!(BaSession, timer)) }
}

#[cfg(test)]
const fn ba_link_event_field(offset: usize) -> DtcmAddress {
    DtcmAddress::from_offset_unchecked(
        core::mem::offset_of!(DtcmLayout, ba_link_event_state) + offset,
    )
}

#[cfg(test)]
const fn ba_deferred_action() -> DtcmAddress { ba_link_event_field(core::mem::offset_of!(BaLinkEventState, ba_deferred_action)) }
#[cfg(test)]
const fn ba_deferred_interface() -> DtcmAddress { ba_link_event_field(core::mem::offset_of!(BaLinkEventState, ba_deferred_interface)) }
#[cfg(test)]
const fn ba_periodic_timer_enabled() -> DtcmAddress { ba_link_event_field(core::mem::offset_of!(BaLinkEventState, periodic_timer_enabled)) }
#[cfg(test)]
const fn ba_periodic_timer() -> DtcmAddress { ba_link_event_field(core::mem::offset_of!(BaLinkEventState, periodic_timer)) }
#[cfg(test)]
const fn ba_transition_timer() -> DtcmAddress { ba_link_event_field(core::mem::offset_of!(BaLinkEventState, transition_timer)) }
#[cfg(test)]
const fn current_network_flags() -> DtcmAddress { ba_link_event_field(core::mem::offset_of!(BaLinkEventState, current_network_flags)) }
#[cfg(test)]
const fn accumulated_network_flags() -> DtcmAddress { ba_link_event_field(core::mem::offset_of!(BaLinkEventState, accumulated_network_flags)) }
#[cfg(test)]
const fn changed_network_flags() -> DtcmAddress { ba_link_event_field(core::mem::offset_of!(BaLinkEventState, changed_network_flags)) }

#[cfg(test)]
const fn join_scan_field(offset: usize) -> DtcmAddress {
    DtcmAddress::from_offset_unchecked(core::mem::offset_of!(DtcmLayout, join_scan_control) + offset)
}
#[cfg(test)]
const fn join_timer() -> DtcmAddress { join_scan_field(core::mem::offset_of!(JoinScanControl, join_timer)) }
#[cfg(test)]
const fn join_status() -> DtcmAddress { join_scan_field(core::mem::offset_of!(JoinScanControl, join_status)) }
#[cfg(test)]
const fn join_interface_state() -> DtcmAddress { join_scan_field(core::mem::offset_of!(JoinScanControl, interface_state)) }

#[cfg(test)]
const fn lmc_request_pointer(index: usize) -> Option<DtcmAddress> {
    if index < 30 {
        Some(DtcmAddress::from_offset_unchecked(
            core::mem::offset_of!(DtcmLayout, wsm_response_scratch)
                + core::mem::offset_of!(WsmResponseScratch, request_pointers)
                + index * core::mem::size_of::<SharedU32>(),
        ))
    } else { None }
}
#[cfg(test)]
const fn lmc_request_status(index: usize) -> Option<DtcmAddress> {
    if index < 30 {
        Some(DtcmAddress::from_offset_unchecked(
            core::mem::offset_of!(DtcmLayout, wsm_response_scratch)
                + core::mem::offset_of!(WsmResponseScratch, request_status_prefix)
                + index,
        ))
    } else { None }
}

const fn command_channel_field(offset: usize) -> DtcmAddress {
    DtcmAddress::from_offset_unchecked(
        core::mem::offset_of!(DtcmLayout, command_channel_switch) + offset,
    )
}

pub(crate) const fn saved_register_context() -> DtcmAddress {
    command_channel_field(core::mem::offset_of!(
        CommandChannelSwitchOverlay,
        saved_register_context
    ))
}

pub(crate) const fn vendor_scan_state() -> DtcmAddress {
    command_channel_field(core::mem::offset_of!(CommandChannelSwitchOverlay, scan_state))
}

#[cfg(test)]
const fn command_upload_start() -> DtcmAddress {
    command_channel_field(core::mem::offset_of!(CommandChannelSwitchOverlay, upload_prefix))
}
#[cfg(test)]
const fn channel_switch_active() -> DtcmAddress {
    command_channel_field(core::mem::offset_of!(CommandChannelSwitchOverlay, channel_switch_active))
}
#[cfg(test)]
const fn channel_switch_channel() -> DtcmAddress {
    command_channel_field(core::mem::offset_of!(CommandChannelSwitchOverlay, channel))
}
#[cfg(test)]
const fn command_overlay_tail() -> DtcmAddress {
    command_channel_field(core::mem::offset_of!(CommandChannelSwitchOverlay, tail_word))
}

const fn lmc_control_field(offset: usize) -> DtcmAddress {
    DtcmAddress::from_offset_unchecked(
        core::mem::offset_of!(DtcmLayout, lmc_control_roots) + offset,
    )
}

pub(crate) const fn encryption_free_head() -> DtcmAddress {
    lmc_control_field(core::mem::offset_of!(LmcControlRoots, encryption_free_head))
}

pub(crate) const fn encryption_generation() -> DtcmAddress {
    lmc_control_field(core::mem::offset_of!(LmcControlRoots, encryption_generation))
}

#[cfg(test)]
const fn duplicate_cache_entry(index: usize) -> Option<DtcmAddress> {
    if index < 32 {
        Some(DtcmAddress::from_offset_unchecked(
            core::mem::offset_of!(DtcmLayout, lmc_control_roots)
                + core::mem::offset_of!(LmcControlRoots, duplicate_cache_prefix)
                + index * core::mem::size_of::<DuplicateCacheEntry>(),
        ))
    } else { None }
}

#[cfg(test)]
const fn pre_command_field(offset: usize) -> DtcmAddress {
    DtcmAddress::from_offset_unchecked(
        core::mem::offset_of!(DtcmLayout, pre_command_quarantine) + offset,
    )
}
#[cfg(test)]
const fn peer_pipe(index: usize) -> Option<DtcmAddress> {
    if index < 8 {
        Some(pre_command_field(
            core::mem::offset_of!(PreCommandQuarantine, peer_pipes)
                + index * core::mem::size_of::<PeerPipeEntry>(),
        ))
    } else { None }
}
#[cfg(test)]
const fn management_counter(index: usize) -> Option<DtcmAddress> {
    if index < 4 {
        Some(pre_command_field(
            core::mem::offset_of!(PreCommandQuarantine, management_counters)
                + index * core::mem::size_of::<SharedU16>(),
        ))
    } else { None }
}
#[cfg(test)]
const fn pre_command_scan_channel() -> DtcmAddress {
    pre_command_field(core::mem::offset_of!(PreCommandQuarantine, scan_channel))
}
#[cfg(test)]
const fn pre_command_pending_root() -> DtcmAddress {
    pre_command_field(core::mem::offset_of!(PreCommandQuarantine, pending_root))
}

const fn scheduler_exclusion_field(offset: usize) -> DtcmAddress {
    DtcmAddress::from_offset_unchecked(
        core::mem::offset_of!(DtcmLayout, initialized_vendor_image)
            + core::mem::offset_of!(InitializedVendorImage, scheduler_exclusion_state)
            + offset,
    )
}
const fn scheduler_event_field(offset: usize) -> DtcmAddress {
    DtcmAddress::from_offset_unchecked(
        core::mem::offset_of!(DtcmLayout, initialized_vendor_image)
            + core::mem::offset_of!(InitializedVendorImage, scheduler_event_island)
            + offset,
    )
}
pub(crate) const fn scheduler_exclusion_mask() -> DtcmAddress { scheduler_exclusion_field(core::mem::offset_of!(SchedulerExclusionState, exclusion_mask)) }
pub(crate) const fn scheduler_secondary_exclusion() -> DtcmAddress { scheduler_exclusion_field(core::mem::offset_of!(SchedulerExclusionState, secondary_exclusion)) }
pub(crate) const fn scheduler_pending_events() -> DtcmAddress { scheduler_event_field(core::mem::offset_of!(SchedulerEventIsland, pending_events)) }
pub(crate) const fn scheduler_runtime_flags() -> DtcmAddress { scheduler_event_field(core::mem::offset_of!(SchedulerEventIsland, runtime_flags)) }
pub(crate) const fn scheduler_startup_mode() -> DtcmAddress { scheduler_event_field(core::mem::offset_of!(SchedulerEventIsland, startup_mode)) }
pub(crate) const fn scheduler_analog_enabled() -> DtcmAddress { scheduler_event_field(core::mem::offset_of!(SchedulerEventIsland, analog_enabled)) }
pub(crate) const fn scheduler_remap_primary() -> DtcmAddress { scheduler_event_field(core::mem::offset_of!(SchedulerEventIsland, remap_primary)) }
pub(crate) const fn scheduler_remap_secondary() -> DtcmAddress { scheduler_event_field(core::mem::offset_of!(SchedulerEventIsland, remap_secondary)) }
pub(crate) const fn scheduler_analog_word(index: usize) -> Option<DtcmAddress> {
    if index < 3 {
        Some(scheduler_event_field(
            core::mem::offset_of!(SchedulerEventIsland, analog_words)
                + index * core::mem::size_of::<SharedU32>(),
        ))
    } else { None }
}
pub(crate) const fn scheduler_timer_list_head() -> DtcmAddress { scheduler_event_field(core::mem::offset_of!(SchedulerEventIsland, timer_list_head)) }
pub(crate) const fn scheduler_handler(index: usize) -> Option<DtcmAddress> {
    if index < 32 { Some(scheduler_handler_unchecked(index)) } else { None }
}
/// Forms a handler slot after the caller has established `index < 32`.
pub(crate) const fn scheduler_handler_unchecked(index: usize) -> DtcmAddress {
    DtcmAddress::from_offset_unchecked(
        core::mem::offset_of!(DtcmLayout, scheduler_handlers)
            + core::mem::offset_of!(SchedulerHandlerTable, handlers)
            + index * core::mem::size_of::<SharedU32>(),
    )
}
#[cfg(test)]
const fn clock_parameter_field(offset: usize) -> DtcmAddress {
    DtcmAddress::from_offset_unchecked(
        core::mem::offset_of!(DtcmLayout, clock_parameters) + offset,
    )
}

macro_rules! assert_type_layout {
    ($type:ty, $size:expr, $align:expr) => {
        assert!(core::mem::size_of::<$type>() == $size);
        assert!(core::mem::align_of::<$type>() == $align);
    };
}

const _: () = {
    assert_type_layout!(DtcmAddress, 4, 4);
    assert_type_layout!(InitializedVendorImage, 0x2078, 4);
    assert_type_layout!(TkipSboxTables, 0x400, 2); assert_type_layout!(AesTransferClassTable, 0x2c, 4); assert!(core::mem::offset_of!(AesTransferClassTable, flags) == 0); assert_type_layout!(InitializedIqCalibrationGainIndices, 0x30, 4); assert!(core::mem::offset_of!(InitializedIqCalibrationGainIndices, entries) == 0x00); assert!(core::mem::size_of::<[SharedU32; 12]>() == 0x30); assert_type_layout!(MeasurementWorkspace, 0x68, 4); assert_type_layout!(TxAggregateExpirationDelta, 0x04, 4); assert!(core::mem::offset_of!(TxAggregateExpirationDelta, value) == 0); assert_type_layout!(DebugCommandDescriptor, 0x0c, 4); assert!(core::mem::offset_of!(DebugCommandDescriptor, command_name) == 0x00); assert!(core::mem::offset_of!(DebugCommandDescriptor, help_text) == 0x04); assert!(core::mem::offset_of!(DebugCommandDescriptor, handler) == 0x08); assert_type_layout!(InitializedDebugCommandDescriptors, 0x48, 4); assert!(core::mem::offset_of!(InitializedDebugCommandDescriptors, records) == 0x00); assert!(core::mem::size_of::<[DebugCommandDescriptor; 6]>() == 0x48); assert!(core::mem::offset_of!(MeasurementWorkspace, dwell_bound) == 0x00); assert!(core::mem::offset_of!(MeasurementWorkspace, measurement_type) == 0x05); assert!(core::mem::offset_of!(MeasurementWorkspace, completion_status) == 0x08); assert!(core::mem::offset_of!(MeasurementWorkspace, dispatch_state) == 0x11); assert!(core::mem::offset_of!(MeasurementWorkspace, dispatch_argument) == 0x12); assert!(core::mem::offset_of!(MeasurementWorkspace, start_timestamp_words) == 0x18); assert!(core::mem::offset_of!(MeasurementWorkspace, elapsed_timestamp_words) == 0x20); assert!(core::mem::offset_of!(MeasurementWorkspace, scan_request_prefix) == 0x28); assert!(core::mem::offset_of!(MeasurementWorkspace, scan_request_mode) == 0x29); assert_type_layout!(RegisterWrite, 0x08, 4); assert!(core::mem::offset_of!(RegisterWrite, address) == 0x00); assert!(core::mem::offset_of!(RegisterWrite, value) == 0x04); assert_type_layout!(RegisterWriteList, 0x58, 4); assert!(core::mem::offset_of!(RegisterWriteList, writes) == 0x00); assert!(core::mem::offset_of!(RegisterWriteList, terminator_address) == 0x50); assert!(core::mem::offset_of!(RegisterWriteList, terminator_opaque) == 0x54); assert_type_layout!(PhyInitRegisterWriteList0, 0x10, 4); assert!(core::mem::offset_of!(PhyInitRegisterWriteList0, writes) == 0x00); assert!(core::mem::offset_of!(PhyInitRegisterWriteList0, terminator_address) == 0x08); assert!(core::mem::offset_of!(PhyInitRegisterWriteList0, terminator_opaque) == 0x0c); assert_type_layout!(PhyInitRegisterWriteList1, 0x28, 4); assert!(core::mem::offset_of!(PhyInitRegisterWriteList1, writes) == 0x00); assert!(core::mem::offset_of!(PhyInitRegisterWriteList1, terminator_address) == 0x20); assert!(core::mem::offset_of!(PhyInitRegisterWriteList1, terminator_opaque) == 0x24); assert_type_layout!(PhyInitRegisterWriteList2, 0x18, 4); assert!(core::mem::offset_of!(PhyInitRegisterWriteList2, writes) == 0x00); assert!(core::mem::offset_of!(PhyInitRegisterWriteList2, terminator_address) == 0x10); assert!(core::mem::offset_of!(PhyInitRegisterWriteList2, terminator_opaque) == 0x14); assert_type_layout!(InitializedPhyGainSourceRecord, 0x06, 2); assert!(core::mem::offset_of!(InitializedPhyGainSourceRecord, selector) == 0x00); assert!(core::mem::offset_of!(InitializedPhyGainSourceRecord, opaque_01) == 0x01); assert!(core::mem::offset_of!(InitializedPhyGainSourceRecord, lower) == 0x02); assert!(core::mem::offset_of!(InitializedPhyGainSourceRecord, upper) == 0x04); assert!(core::mem::size_of::<[InitializedPhyGainSourceRecord; 43]>() == 0x102); assert!(core::mem::align_of::<[InitializedPhyGainSourceRecord; 43]>() == 2);
    assert!(core::mem::offset_of!(TkipSboxTables, low_byte) == 0x000);
    assert!(core::mem::offset_of!(TkipSboxTables, high_byte) == 0x200);
    assert_type_layout!(RuntimeRegisterBackoffState, 0x1c, 4);
    assert!(core::mem::offset_of!(RuntimeRegisterBackoffState, override_enabled) == 0x10);
    assert!(core::mem::offset_of!(RuntimeRegisterBackoffState, override_window) == 0x14);
    assert_type_layout!(DebugConsoleState, 0xf8, 4);
    assert!(core::mem::offset_of!(DebugConsoleState, timer) == 0x08);
    assert!(core::mem::offset_of!(DebugConsoleState, memory_address) == 0x1c);
    assert!(core::mem::offset_of!(DebugConsoleState, command_count) == 0x24);
    assert!(core::mem::offset_of!(DebugConsoleState, commands) == 0x28);
    assert!(core::mem::offset_of!(DebugConsoleState, line_buffer) == 0xa8);
    assert_type_layout!(RuntimePrefix, 0x114, 4);
    assert!(core::mem::offset_of!(RuntimePrefix, register_backoff) == 0x00);
    assert!(core::mem::offset_of!(RuntimePrefix, debug_console_state) == 0x1c);
    assert_type_layout!(ClockParameterIsland, 0x28, 4);
    assert!(core::mem::offset_of!(ClockParameterIsland, mac_clock_snapshot) == 0x04);
    assert!(core::mem::offset_of!(ClockParameterIsland, beacon_counter_snapshot) == 0x08);
    assert!(core::mem::offset_of!(ClockParameterIsland, hardware_counter_cache) == 0x0c);
    assert!(core::mem::offset_of!(ClockParameterIsland, conversion_factor) == 0x14);
    assert!(core::mem::offset_of!(ClockParameterIsland, conversion_mode) == 0x1c);
    assert!(core::mem::offset_of!(ClockParameterIsland, correction_offset) == 0x24);
    assert_type_layout!(TimerEntry, 0x14, 4);
    assert!(core::mem::offset_of!(TimerEntry, next) == 0x00);
    assert!(core::mem::offset_of!(TimerEntry, previous_link) == 0x04);
    assert!(core::mem::offset_of!(TimerEntry, deadline) == 0x08);
    assert!(core::mem::offset_of!(TimerEntry, callback) == 0x0c);
    assert!(core::mem::offset_of!(TimerEntry, context) == 0x10);
    assert_type_layout!(SchedulerHandlerTable, 0x80, 4);
    assert_type_layout!(PhyGainSourceRecord, 0x06, 2);
    assert!(core::mem::offset_of!(PhyGainSourceRecord, lower) == 0x02);
    assert!(core::mem::offset_of!(PhyGainSourceRecord, upper) == 0x04);
    assert_type_layout!(BeaconIeOffsetIndex, 0x204, 4);
    assert_type_layout!(BeaconFilterStorage, 0x6cc, 4);
    assert!(core::mem::offset_of!(BeaconFilterStorage, stored_beacon) == 0x004);
    assert!(core::mem::offset_of!(BeaconFilterStorage, active_index) == 0x2c0);
    assert!(core::mem::offset_of!(BeaconFilterStorage, indexes) == 0x2c4);
    assert!(core::mem::offset_of!(BeaconIeOffsetIndex, offsets) == 0x04);
    assert_type_layout!(RfInitializationObservedLayout, 0xe8, 4);
    assert!(core::mem::offset_of!(RfInitializationObservedLayout, table_minus_98) == 0x14);
    assert!(core::mem::offset_of!(RfInitializationObservedLayout, negative_words) == 0x58);
    assert!(core::mem::offset_of!(RfInitializationObservedLayout, root_prefix) == 0xac);
    assert!(core::mem::offset_of!(RfInitializationObservedLayout, table_14) == 0xc0);
    assert!(core::mem::offset_of!(RfInitializationObservedLayout, control_30) == 0xdc);
    assert!(core::mem::offset_of!(RfInitializationObservedLayout, pointer_38) == 0xe4);
    assert_type_layout!(TemplateBackingStorage, 0x3e0, 4);
    assert!(core::mem::offset_of!(TemplateBackingStorage, secondary) == 0x200);
    assert!(core::mem::offset_of!(TemplateBackingStorage, tertiary) == 0x2c0);
    assert_type_layout!(TemplateFrameDescriptor, 0x40, 4);
    assert!(core::mem::offset_of!(TemplateFrameDescriptor, buffer_04) == 0x04);
    assert!(core::mem::offset_of!(TemplateFrameDescriptor, pointer_0c) == 0x0c);
    assert!(core::mem::offset_of!(TemplateFrameDescriptor, pointer_2c) == 0x2c);
    assert!(core::mem::offset_of!(TemplateFrameDescriptor, pointer_34) == 0x34);
    assert!(core::mem::offset_of!(TemplateFrameDescriptor, pointer_3c) == 0x3c);
    assert_type_layout!(PreConfigurationTables, 0x127c, 4);
    assert!(core::mem::offset_of!(PreConfigurationTables, gain_source_records) == 0x000);
    assert!(core::mem::offset_of!(PreConfigurationTables, beacon_filter) == 0x084);
    assert!(core::mem::offset_of!(PreConfigurationTables, opaque_750) == 0x750);
    assert!(core::mem::offset_of!(PreConfigurationTables, template_descriptors) == 0xe1c);
    assert!(core::mem::offset_of!(PreConfigurationTables, template_backing) == 0xe9c);
    assert_type_layout!(SddChannelRecord, 0x03, 1);
    assert_type_layout!(SddProfileBank, 0x92, 2);
    assert!(core::mem::offset_of!(SddProfileBank, channel_records) == 0x16);
    assert!(core::mem::offset_of!(SddProfileBank, channel_count) == 0x46);
    assert!(core::mem::offset_of!(SddProfileBank, agc_correction) == 0x48);
    assert!(core::mem::offset_of!(SddProfileBank, calibration_coefficient) == 0x4a);
    assert!(core::mem::offset_of!(SddProfileBank, conversion_pair) == 0x4c);
    assert!(core::mem::offset_of!(SddProfileBank, rssi_coefficients) == 0x50);
    assert!(core::mem::offset_of!(SddProfileBank, rssi_rate_scales) == 0x54);
    assert!(core::mem::offset_of!(SddProfileBank, opaque_6a) == 0x6a);
    assert_type_layout!(SddConfigurationTables, 0x130, 4);
    assert!(core::mem::offset_of!(SddConfigurationTables, profiles) == 0x00);
    assert!(core::mem::offset_of!(SddConfigurationTables, opaque_124) == 0x124);
    assert_type_layout!(WakeContextState, 0x90, 4);
    assert!(core::mem::offset_of!(WakeContextState, response_pointers) == 0x10);
    assert_type_layout!(DurationSources, 0x4, 2);
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
    assert!(core::mem::offset_of!(PreVifHeader, link_bitmap) == 0x18);
    assert_type_layout!(VifRecord, VIF_RECORD_SIZE, 4);
    assert_type_layout!(VifRecordAddress, 4, 4);
    assert!(core::mem::offset_of!(VifRecord, scan_rate_config) == 0x000);
    assert!(core::mem::offset_of!(VifRecord, scan_channel) == 0x002);
    assert!(core::mem::offset_of!(VifRecord, scan_flags) == 0x004);
    assert!(core::mem::offset_of!(VifRecord, reserved_05) == 0x005);
    assert!(core::mem::offset_of!(VifRecord, host_contexts_in_flight) == 0x006);
    assert!(core::mem::offset_of!(VifRecord, reserved_07) == 0x007);
    assert!(core::mem::offset_of!(VifRecord, wake_reinit_flag) == 0x016);
    assert!(core::mem::offset_of!(VifRecord, reserved_17) == 0x017);
    assert!(core::mem::offset_of!(VifRecord, mode) == 0x018);
    assert!(core::mem::offset_of!(VifRecord, active) == 0x019);
    assert!(core::mem::offset_of!(VifRecord, interface) == 0x01a);
    assert!(core::mem::offset_of!(VifRecord, role) == 0x01b);
    assert!(core::mem::offset_of!(VifRecord, flags) == 0x01c);
    assert!(core::mem::offset_of!(VifRecord, rate_configuration) == 0x020);
    assert!(core::mem::offset_of!(VifRecord, basic_rates) == 0x028);
    assert!(core::mem::offset_of!(VifRecord, allowed_links) == 0x02c);
    assert!(core::mem::offset_of!(VifRecord, effective_links) == 0x02e);
    assert!(core::mem::offset_of!(VifRecord, tx_busy) == 0x030);
    assert!(core::mem::offset_of!(VifRecord, reserved_32) == 0x032);
    assert!(core::mem::offset_of!(VifRecord, own_mac) == 0x034);
    assert!(core::mem::offset_of!(VifRecord, reserved_3a) == 0x03a);
    assert!(core::mem::offset_of!(VifRecord, bssid) == 0x03c);
    assert!(core::mem::offset_of!(VifRecord, channel) == 0x042);
    assert!(core::mem::offset_of!(VifRecord, radio_owner_overlay) == 0x044);
    assert!(core::mem::offset_of!(VifRecord, operating_state) == 0x050);
    assert!(core::mem::offset_of!(VifRecord, owner_interface) == 0x051);
    assert!(core::mem::offset_of!(VifRecord, owner_channel) == 0x052);
    assert!(core::mem::offset_of!(VifRecord, owner_deadline) == 0x054);
    assert!(core::mem::offset_of!(VifRecord, reserved_58) == 0x058);
    assert!(core::mem::offset_of!(VifRecord, owner_flags) == 0x05c);
    assert!(core::mem::offset_of!(VifRecord, reserved_60) == 0x060);
    assert!(core::mem::offset_of!(VifRecord, activity_state) == 0x066);
    assert!(core::mem::offset_of!(VifRecord, opaque_067) == 0x067);
    assert!(core::mem::offset_of!(VifRecord, operating_timers) == 0x0b0);
    assert!(core::mem::offset_of!(VifRecord, ssid_length) == 0x0ec);
    assert!(core::mem::offset_of!(VifRecord, ssid) == 0x0f0);
    assert!(core::mem::offset_of!(VifRecord, dtim_period) == 0x110);
    assert!(core::mem::offset_of!(VifRecord, reserved_111) == 0x111);
    assert!(core::mem::offset_of!(VifRecord, atim_window) == 0x116);
    assert!(core::mem::offset_of!(VifRecord, beacon_interval) == 0x118);
    assert!(core::mem::offset_of!(VifRecord, reserved_11c) == 0x11c);
    assert!(core::mem::offset_of!(VifRecord, rts_threshold) == 0x124);
    assert!(core::mem::offset_of!(VifRecord, ampdu_length) == 0x128);
    assert!(core::mem::offset_of!(VifRecord, internal_link) == 0x12a);
    assert!(core::mem::offset_of!(VifRecord, default_rates) == 0x12c);
    assert!(core::mem::offset_of!(VifRecord, reserved_12e) == 0x12e);
    assert!(core::mem::offset_of!(VifRecord, link_object_flags) == 0x13c);
    assert!(core::mem::offset_of!(VifRecord, link_own_mac_0) == 0x140);
    assert!(core::mem::offset_of!(VifRecord, link_own_mac_1) == 0x146);
    assert!(core::mem::offset_of!(VifRecord, link_bssid) == 0x14c);
    assert!(core::mem::offset_of!(VifRecord, reserved_152) == 0x152);
    assert!(core::mem::offset_of!(VifRecord, sleeping_links) == 0x15c);
    assert!(core::mem::offset_of!(VifRecord, awake_links) == 0x15e);
    assert!(core::mem::offset_of!(VifRecord, buffered_links) == 0x160);
    assert!(core::mem::offset_of!(VifRecord, reserved_162) == 0x162);
    assert!(core::mem::offset_of!(VifRecord, link_gate) == 0x164);
    assert!(core::mem::offset_of!(VifRecord, opaque_165) == 0x165);
    assert!(core::mem::offset_of!(VifRecord, link_timers) == 0x184);
    assert!(core::mem::offset_of!(VifRecord, opaque_1ac) == 0x1ac);
    assert_type_layout!(VifRecords, 0xb10, 4);
    assert_type_layout!(PostVifQuarantine, 0x107c, 4);
    assert_type_layout!(HifRequestAddress, 4, 4);
    assert_type_layout!(PacketRamAddress, 4, 4);
    assert_type_layout!(HostContextAddress, 4, 4);
    assert_type_layout!(HostFrameNodeAddress, 4, 4);
    assert_type_layout!(HostPasAddress, 4, 4);
    assert_type_layout!(HostPasContext, 0x80, 4);
    assert!(core::mem::offset_of!(HostPasContext, frame_address) == 0x00);
    assert!(core::mem::offset_of!(HostPasContext, control_bits) == 0x04);
    assert!(core::mem::offset_of!(HostPasContext, frame_length) == 0x08);
    assert!(core::mem::offset_of!(HostPasContext, frame_control) == 0x0a);
    assert!(core::mem::offset_of!(HostPasContext, access_category) == 0x0c);
    assert!(core::mem::offset_of!(HostPasContext, request_flag_rate_bits) == 0x0d);
    assert!(core::mem::offset_of!(HostPasContext, retry_policy) == 0x0e);
    assert!(core::mem::offset_of!(HostPasContext, tx_rate) == 0x0f);
    assert!(core::mem::offset_of!(HostPasContext, expiry_time) == 0x10);
    assert!(core::mem::offset_of!(HostPasContext, completion_timestamp) == 0x14);
    assert!(core::mem::offset_of!(HostPasContext, scheduler_timestamp) == 0x18);
    assert!(core::mem::offset_of!(HostPasContext, terminal_status) == 0x1c);
    assert!(core::mem::offset_of!(HostPasContext, try_count) == 0x1e);
    assert!(core::mem::offset_of!(HostPasContext, opaque_20) == 0x20);
    assert!(core::mem::offset_of!(HostPasContext, ownership_bits) == 0x2c);
    assert!(core::mem::offset_of!(HostPasContext, opaque_30) == 0x30);
    assert!(core::mem::offset_of!(HostPasContext, duration) == 0x36);
    assert!(core::mem::offset_of!(HostPasContext, opaque_38) == 0x38);
    assert!(core::mem::offset_of!(HostPasContext, descriptor_state) == 0x3c);
    assert!(core::mem::offset_of!(HostPasContext, opaque_40) == 0x40);
    assert!(core::mem::offset_of!(HostPasContext, word_48) == 0x48);
    assert!(core::mem::offset_of!(HostPasContext, frame_state_address) == 0x4c);
    assert!(core::mem::offset_of!(HostPasContext, auxiliary_state) == 0x50);
    assert!(core::mem::offset_of!(HostPasContext, tid) == 0x52);
    assert!(core::mem::offset_of!(HostPasContext, insertion_mode) == 0x53);
    assert!(core::mem::offset_of!(HostPasContext, sequence_number) == 0x54);
    assert!(core::mem::offset_of!(HostPasContext, retry_rate) == 0x56);
    assert!(core::mem::offset_of!(HostPasContext, byte_57) == 0x57);
    assert!(core::mem::offset_of!(HostPasContext, opaque_58) == 0x58);
    assert!(core::mem::offset_of!(HostPasContext, interface) == 0x69);
    assert!(core::mem::offset_of!(HostPasContext, duration_slot) == 0x6a);
    assert!(core::mem::offset_of!(HostPasContext, host_link) == 0x6b);
    assert!(core::mem::offset_of!(HostPasContext, completion_byte_6c) == 0x6c);
    assert!(core::mem::offset_of!(HostPasContext, opaque_6d) == 0x6d);
    assert!(core::mem::offset_of!(HostPasContext, qos_control) == 0x74);
    assert!(core::mem::offset_of!(HostPasContext, cipher_class) == 0x76);
    assert!(core::mem::offset_of!(HostPasContext, opaque_77) == 0x77);
    assert!(core::mem::offset_of!(HostPasContext, word_7c) == 0x7c);
    assert!(core::mem::offset_of!(HostPasContext, opaque_7e) == 0x7e);
    assert_type_layout!(HostTxContext, HOST_TX_CONTEXT_SIZE, 4);
    assert!(core::mem::offset_of!(HostTxContext, request_buffer) == 0x00);
    assert!(core::mem::offset_of!(HostTxContext, intrusive_next) == 0x04);
    assert!(core::mem::offset_of!(HostTxContext, packet_id) == 0x08);
    assert!(core::mem::offset_of!(HostTxContext, requested_rate) == 0x0c);
    assert!(core::mem::offset_of!(HostTxContext, queue_id) == 0x0d);
    assert!(core::mem::offset_of!(HostTxContext, more) == 0x0e);
    assert!(core::mem::offset_of!(HostTxContext, request_flags) == 0x0f);
    assert!(core::mem::offset_of!(HostTxContext, expiry_time) == 0x10);
    assert!(core::mem::offset_of!(HostTxContext, ht_tx_parameters) == 0x14);
    assert!(core::mem::offset_of!(HostTxContext, borrowed_frame_length) == 0x18);
    assert!(core::mem::offset_of!(HostTxContext, borrowed_frame_address) == 0x1c);
    assert!(core::mem::offset_of!(HostTxContext, completion_status) == 0x20);
    assert!(core::mem::offset_of!(HostTxContext, rate_copy) == 0x24);
    assert!(core::mem::offset_of!(HostTxContext, saved_status) == 0x25);
    assert!(core::mem::offset_of!(HostTxContext, completion_flags) == 0x26);
    assert!(core::mem::offset_of!(HostTxContext, rate_try) == 0x28);
    assert!(core::mem::offset_of!(HostTxContext, opaque_34) == 0x34);
    assert!(core::mem::offset_of!(HostTxContext, timing_scratch) == 0x38);
    assert!(core::mem::offset_of!(HostTxContext, submit_timer) == 0x40);
    assert!(core::mem::offset_of!(HostTxContext, header_length) == 0x44);
    assert!(core::mem::offset_of!(HostTxContext, payload_length) == 0x48);
    assert!(core::mem::offset_of!(HostTxContext, optional_pipe_object) == 0x4c);
    assert!(core::mem::offset_of!(HostTxContext, sequence_or_callback_state) == 0x50);
    assert!(core::mem::offset_of!(HostTxContext, submit_state) == 0x52);
    assert!(core::mem::offset_of!(HostTxContext, completion_class) == 0x53);
    assert!(core::mem::offset_of!(HostTxContext, pas) == 0x54);
    assert!(core::mem::offset_of!(HostTxContext, opaque_d4) == 0xd4);
    assert_type_layout!(HostTxContexts, 0x2b20, 4);
    assert_type_layout!(PeerPipeEntry, 0x08, 4);
    assert!(core::mem::offset_of!(PeerPipeEntry, peer_mac) == 0x00);
    assert!(core::mem::offset_of!(PeerPipeEntry, state_flags) == 0x06);
    assert!(core::mem::offset_of!(PeerPipeEntry, age) == 0x07);
    assert_type_layout!(PreCommandQuarantine, 0x50, 4);
    assert!(core::mem::offset_of!(PreCommandQuarantine, peer_pipes) == 0x00);
    assert!(core::mem::offset_of!(PreCommandQuarantine, management_counters) == 0x40);
    assert!(core::mem::offset_of!(PreCommandQuarantine, scan_channel) == 0x48);
    assert!(core::mem::offset_of!(PreCommandQuarantine, pending_root) == 0x4c);
    assert_type_layout!(CommandChannelSwitchOverlay, 0x84, 4);
    assert!(core::mem::offset_of!(CommandChannelSwitchOverlay, upload_prefix) == 0x00);
    assert!(core::mem::offset_of!(CommandChannelSwitchOverlay, overlay_head) == 0x64);
    assert!(core::mem::offset_of!(CommandChannelSwitchOverlay, channel_switch_active) == 0x68);
    assert!(core::mem::offset_of!(CommandChannelSwitchOverlay, interface) == 0x69);
    assert!(core::mem::offset_of!(CommandChannelSwitchOverlay, mode) == 0x6c);
    assert!(core::mem::offset_of!(CommandChannelSwitchOverlay, countdown) == 0x6d);
    assert!(core::mem::offset_of!(CommandChannelSwitchOverlay, channel) == 0x6e);
    assert!(core::mem::offset_of!(CommandChannelSwitchOverlay, join_mode) == 0x70);
    assert!(core::mem::offset_of!(CommandChannelSwitchOverlay, join_flags) == 0x71);
    assert!(core::mem::offset_of!(CommandChannelSwitchOverlay, saved_register_context) == 0x72);
    assert!(core::mem::offset_of!(CommandChannelSwitchOverlay, rate_configuration) == 0x74);
    assert!(core::mem::offset_of!(CommandChannelSwitchOverlay, scan_state) == 0x78);
    assert!(core::mem::offset_of!(CommandChannelSwitchOverlay, scan_flags) == 0x79);
    assert!(core::mem::offset_of!(CommandChannelSwitchOverlay, tx_buffer_free_count) == 0x7b);
    assert!(core::mem::offset_of!(CommandChannelSwitchOverlay, scan_word) == 0x7c);
    assert!(core::mem::offset_of!(CommandChannelSwitchOverlay, tail_word) == 0x80);
    assert_type_layout!(DuplicateCacheEntry, 0x0c, 4);
    assert!(core::mem::offset_of!(DuplicateCacheEntry, peer_mac) == 0x00);
    assert!(core::mem::offset_of!(DuplicateCacheEntry, identity) == 0x06);
    assert!(core::mem::offset_of!(DuplicateCacheEntry, context) == 0x08);
    assert_type_layout!(LmcControlRoots, 0x180, 4);
    assert!(core::mem::offset_of!(LmcControlRoots, join_match_state) == 0x00);
    assert!(core::mem::offset_of!(LmcControlRoots, encryption_free_head) == 0x04);
    assert!(core::mem::offset_of!(LmcControlRoots, encryption_generation) == 0x08);
    assert!(core::mem::offset_of!(LmcControlRoots, duplicate_cache_prefix) == 0x0c);
    assert_type_layout!(HostContextAccounting, 0x18, 4);
    assert!(core::mem::offset_of!(HostContextAccounting, opaque_00) == 0x00);
    assert!(core::mem::offset_of!(HostContextAccounting, duplicate_cache_cursor) == 0x0c);
    assert!(core::mem::offset_of!(HostContextAccounting, deferred_event_owner) == 0x10);
    assert!(core::mem::offset_of!(HostContextAccounting, auxiliary_tx_buffer_free_head) == 0x14);
    assert_type_layout!(HostContextFreeList, 0x8, 4);
    assert!(core::mem::offset_of!(HostContextFreeList, free_head) == 0x00);
    assert!(core::mem::offset_of!(HostContextFreeList, adjacent_state) == 0x04);
    assert_type_layout!(LinkMapHeader, 0x18, 4);
    assert!(core::mem::offset_of!(LinkMapHeader, entry_count) == 0x14);
    assert!(core::mem::offset_of!(LinkMapHeader, release_blocked_links) == 0x16);
    assert_type_layout!(LinkMapEntry, 0x0c, 4);
    assert!(core::mem::offset_of!(LinkMapEntry, host_link) == 0x00);
    assert!(core::mem::offset_of!(LinkMapEntry, interface) == 0x01);
    assert!(core::mem::offset_of!(LinkMapEntry, internal_link) == 0x02);
    assert!(core::mem::offset_of!(LinkMapEntry, inactivity) == 0x03);
    assert!(core::mem::offset_of!(LinkMapEntry, release_flags) == 0x04);
    assert!(core::mem::offset_of!(LinkMapEntry, auxiliary_flags) == 0x05);
    assert!(core::mem::offset_of!(LinkMapEntry, mac_address) == 0x06);
    assert_type_layout!(LinkAndSequenceState, 0x220, 4);
    assert!(core::mem::offset_of!(LinkAndSequenceState, header) == 0x000);
    assert!(core::mem::offset_of!(LinkAndSequenceState, entries) == 0x018);
    assert!(core::mem::offset_of!(LinkAndSequenceState, sequences) == 0x0d8);
    assert!(core::mem::offset_of!(LinkAndSequenceState, internal_link_bitmap) == 0x218);
    assert!(core::mem::offset_of!(LinkAndSequenceState, opaque_tail) == 0x21a);
    assert_type_layout!(JoinScanControl, 0x40, 4);
    assert!(core::mem::offset_of!(JoinScanControl, schedule_word) == 0x00);
    assert!(core::mem::offset_of!(JoinScanControl, schedule_deadline) == 0x04);
    assert!(core::mem::offset_of!(JoinScanControl, beacon_timer_active) == 0x08);
    assert!(core::mem::offset_of!(JoinScanControl, beacon_interface) == 0x09);
    assert!(core::mem::offset_of!(JoinScanControl, channel_owner) == 0x0c);
    assert!(core::mem::offset_of!(JoinScanControl, channel_use_state) == 0x14);
    assert!(core::mem::offset_of!(JoinScanControl, alternate_channel_owner) == 0x18);
    assert!(core::mem::offset_of!(JoinScanControl, join_timer) == 0x20);
    assert!(core::mem::offset_of!(JoinScanControl, join_status) == 0x34);
    assert!(core::mem::offset_of!(JoinScanControl, start_state) == 0x36);
    assert!(core::mem::offset_of!(JoinScanControl, interface_state) == 0x37);
    assert!(core::mem::offset_of!(JoinScanControl, response_status) == 0x38);
    assert!(core::mem::offset_of!(JoinScanControl, request_word) == 0x3c);
    assert_type_layout!(WsmResponseScratch, 0xa0, 4);
    assert!(core::mem::offset_of!(WsmResponseScratch, scan_control) == 0x00);
    assert!(core::mem::offset_of!(WsmResponseScratch, request_pointers) == 0x0c);
    assert!(core::mem::offset_of!(WsmResponseScratch, request_status_prefix) == 0x84);
    assert_type_layout!(BaLmcHeader, 0x20, 4);
    assert!(core::mem::offset_of!(BaLmcHeader, request_slot_index) == 0x02);
    assert!(core::mem::offset_of!(BaLmcHeader, pending_request_count) == 0x03);
    assert!(core::mem::offset_of!(BaLmcHeader, request_sequence) == 0x04);
    assert!(core::mem::offset_of!(BaLmcHeader, request_word_08) == 0x08);
    assert!(core::mem::offset_of!(BaLmcHeader, request_word_0c) == 0x0c);
    assert!(core::mem::offset_of!(BaLmcHeader, tim_flags) == 0x14);
    assert!(core::mem::offset_of!(BaLmcHeader, request_flags) == 0x16);
    assert!(core::mem::offset_of!(BaLmcHeader, join_retry_state) == 0x18);
    assert!(core::mem::offset_of!(BaLmcHeader, scan_state) == 0x19);
    assert!(core::mem::offset_of!(BaLmcHeader, ba_policy_enabled) == 0x1b);
    assert_type_layout!(PendingBaLmcState, 0xe0, 4);
    assert!(core::mem::offset_of!(PendingBaLmcState, pending_head) == 0x00);
    assert!(core::mem::offset_of!(PendingBaLmcState, pending_tail) == 0x04);
    assert!(core::mem::offset_of!(PendingBaLmcState, mac_bssid_mode) == 0x08);
    assert!(core::mem::offset_of!(PendingBaLmcState, pending_service_needed) == 0x0b);
    assert!(core::mem::offset_of!(PendingBaLmcState, radio_owner) == 0x48);
    assert!(core::mem::offset_of!(PendingBaLmcState, radio_wait_head) == 0x4c);
    assert!(core::mem::offset_of!(PendingBaLmcState, deferred_radio_owner) == 0x54);
    assert!(core::mem::offset_of!(PendingBaLmcState, radio_role_state) == 0xbc);
    assert!(core::mem::offset_of!(PendingBaLmcState, radio_timer_state) == 0xbd);
    assert!(core::mem::offset_of!(PendingBaLmcState, radio_timer_interface) == 0xbe);
    assert!(core::mem::offset_of!(PendingBaLmcState, power_state_complete) == 0xbf);
    assert!(core::mem::offset_of!(PendingBaLmcState, radio_timer_deadline) == 0xc0);
    assert!(core::mem::offset_of!(PendingBaLmcState, radio_timer_sample) == 0xc4);
    assert!(core::mem::offset_of!(PendingBaLmcState, message_control) == 0xd0);
    assert!(core::mem::offset_of!(PendingBaLmcState, ba_session_count) == 0xd1);
    assert!(core::mem::offset_of!(PendingBaLmcState, ba_active_count) == 0xd2);
    assert!(core::mem::offset_of!(PendingBaLmcState, message_producer) == 0xd3);
    assert!(core::mem::offset_of!(PendingBaLmcState, message_consumer) == 0xd4);
    assert_type_layout!(LmcMessage, LMC_MESSAGE_SIZE, 4);
    assert!(core::mem::offset_of!(LmcMessage, kind) == 0x00);
    assert!(core::mem::offset_of!(LmcMessage, flags) == 0x01);
    assert!(core::mem::offset_of!(LmcMessage, payload) == 0x04);
    assert!(core::mem::offset_of!(LmcMessage, interface) == 0x28);
    assert!(core::mem::offset_of!(LmcMessage, completion_state) == 0x29);
    assert_type_layout!(LmcMessages, 0x2c0, 4);
    assert_type_layout!(BaSession, 0x28, 4);
    assert!(core::mem::offset_of!(BaSession, activity) == 0x00);
    assert!(core::mem::offset_of!(BaSession, peer_mac) == 0x04);
    assert!(core::mem::offset_of!(BaSession, tid) == 0x0a);
    assert!(core::mem::offset_of!(BaSession, interface) == 0x0b);
    assert!(core::mem::offset_of!(BaSession, timeout_1024us) == 0x12);
    assert!(core::mem::offset_of!(BaSession, timer) == 0x14);
    assert_type_layout!(BaSessions, 0xa0, 4);
    assert_type_layout!(BaLinkEventState, 0x30, 4);
    assert!(core::mem::offset_of!(BaLinkEventState, ba_deferred_action) == 0x00);
    assert!(core::mem::offset_of!(BaLinkEventState, ba_deferred_interface) == 0x01);
    assert!(core::mem::offset_of!(BaLinkEventState, periodic_timer_enabled) == 0x03);
    assert!(core::mem::offset_of!(BaLinkEventState, periodic_timer) == 0x04);
    assert!(core::mem::offset_of!(BaLinkEventState, transition_timer) == 0x18);
    assert!(core::mem::offset_of!(BaLinkEventState, current_network_flags) == 0x2c);
    assert!(core::mem::offset_of!(BaLinkEventState, accumulated_network_flags) == 0x2d);
    assert!(core::mem::offset_of!(BaLinkEventState, changed_network_flags) == 0x2e);
    assert_type_layout!(TalaAccounting, 0x24, 4);
    assert_type_layout!(ContextCompletionPrefix, 0x14, 4);
    assert!(core::mem::offset_of!(ContextCompletionPrefix, external_count) == 0x04);
    assert!(core::mem::offset_of!(ContextCompletionPrefix, class0_count) == 0x05);
    assert!(core::mem::offset_of!(ContextCompletionPrefix, allocation_state) == 0x08);
    assert!(core::mem::offset_of!(ContextCompletionPrefix, pending_count) == 0x0a);
    assert!(core::mem::offset_of!(ContextCompletionPrefix, coalesce_state) == 0x0c);
    assert!(core::mem::offset_of!(ContextCompletionPrefix, free_state) == 0x10);
    assert_type_layout!(CompletionRingObservedLayout, 0x100, 4);
    assert_type_layout!(PreInternalContextQuarantine, 0xec, 4);
    assert!(core::mem::offset_of!(PreInternalContextQuarantine, completion_entries) == 0x00);
    assert_type_layout!(InternalContextPrefix, 0x14, 4);
    assert!(core::mem::offset_of!(InternalContextPrefix, iv_seed_or_completion_entry_59) == 0x00);
    assert!(core::mem::offset_of!(InternalContextPrefix, completion_entries_60_63) == 0x04);
    assert_type_layout!(InternalPasContext, 0x80, 4);
    assert!(core::mem::offset_of!(InternalPasContext, completion_timestamp) == 0x14);
    assert!(core::mem::offset_of!(InternalPasContext, terminal_status) == 0x1c);
    assert!(core::mem::offset_of!(InternalPasContext, ownership_bits) == 0x2c);
    assert!(core::mem::offset_of!(InternalPasContext, descriptor_state) == 0x3c);
    assert!(core::mem::offset_of!(InternalPasContext, frame_state_address) == 0x4c);
    assert!(core::mem::offset_of!(InternalPasContext, interface) == 0x69);
    assert!(core::mem::offset_of!(InternalPasContext, cipher_buffer) == 0x70);
    assert_type_layout!(InternalTxContext, INTERNAL_TX_CONTEXT_SIZE, 4);
    assert!(core::mem::offset_of!(InternalTxContext, completion_status) == 0x20);
    assert!(core::mem::offset_of!(InternalTxContext, optional_pipe_object) == 0x4c);
    assert!(core::mem::offset_of!(InternalTxContext, completion_class) == 0x53);
    assert!(core::mem::offset_of!(InternalTxContext, pas) == 0x54);
    assert!(core::mem::offset_of!(InternalTxContext, opaque_d4) == 0xd4);
    assert!(INTERNAL_TX_CONTEXT_NEXT_FREE_OFFSET == 0x04);
    assert!(INTERNAL_TX_CONTEXT_HEADER_80211_OFFSET == 0x1c);
    assert!(INTERNAL_TX_CONTEXT_RESULT_OFFSET == 0x70);
    assert!(INTERNAL_TX_CONTEXT_CIPHER_BUFFER_OFFSET == 0xc4);
    assert_type_layout!(InternalContextPoolState, 0x454, 4);
    assert_type_layout!(PowerSaveObservedLayout, 0x140, 4);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, wake_duration) == 0x006);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, wake_register_min) == 0x008);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, wake_elapsed_max) == 0x01c);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, tx_completion_state) == 0x020);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, next_tbtt) == 0x024);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, doze_state) == 0x028);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, requested_pm_mode) == 0x029);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, global_sleep_state) == 0x02a);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, wake_stats_counter) == 0x02b);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, resume_state) == 0x02d);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, global_timer_duration) == 0x030);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, beacon_timing_reference) == 0x034);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, join_state_word) == 0x038);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, wake_lead_time) == 0x03c);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, mode) == 0x040);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, active) == 0x041);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, tx_pending_state) == 0x042);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, timer_state) == 0x043);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, flags) == 0x044);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, sleep_transition_flags) == 0x046);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, wake_stats_timestamp) == 0x048);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, backoff_adjustment) == 0x04c);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, reset_state) == 0x050);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, beacon_timing_valid) == 0x051);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, pending_control_kind) == 0x052);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, uapsd_state) == 0x053);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, pending) == 0x054);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, pending_flags) == 0x058);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, wake_reason) == 0x059);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, queue_mask) == 0x05a);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, tx_followup_state) == 0x05c);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, uapsd_restart_value) == 0x06c);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, timers) == 0x070);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, state_fc) == 0x0fc);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, tx_completion_pending) == 0x0fd);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, beacon_rx_state) == 0x0fe);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, beacon_rate) == 0x0ff);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, wake_timer_delay) == 0x100);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, beacon_airtime) == 0x104);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, pre_tbtt_offset) == 0x108);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, beacon_interval) == 0x10c);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, backoff_interval) == 0x10e);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, next_wake_deadline) == 0x110);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, tx_completion_state_114) == 0x114);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, wake_timer_active) == 0x115);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, beacon_timing_adjusted) == 0x117);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, duration_118) == 0x118);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, duration_120) == 0x120);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, mode_control_124) == 0x124);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, interval_128) == 0x128);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, maximum_backoff_12c) == 0x12c);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, last_beacon_timestamp_130) == 0x130);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, counter_134) == 0x134);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, threshold_136) == 0x136);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, scan_completion_138) == 0x138);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, sleep_vote_count_13a) == 0x13a);
    assert!(core::mem::offset_of!(PowerSaveObservedLayout, ps_mode_error_reported_13c) == 0x13c);
    assert_type_layout!(PowerSavePhysicalPrefix, 0x104, 4);
    assert!(core::mem::offset_of!(PowerSavePhysicalPrefix, wake_duration) == 0x006);
    assert!(core::mem::offset_of!(PowerSavePhysicalPrefix, wake_register_min) == 0x008);
    assert!(core::mem::offset_of!(PowerSavePhysicalPrefix, wake_elapsed_max) == 0x01c);
    assert!(core::mem::offset_of!(PowerSavePhysicalPrefix, tx_completion_state) == 0x020);
    assert!(core::mem::offset_of!(PowerSavePhysicalPrefix, next_tbtt) == 0x024);
    assert!(core::mem::offset_of!(PowerSavePhysicalPrefix, doze_state) == 0x028);
    assert!(core::mem::offset_of!(PowerSavePhysicalPrefix, requested_pm_mode) == 0x029);
    assert!(core::mem::offset_of!(PowerSavePhysicalPrefix, global_sleep_state) == 0x02a);
    assert!(core::mem::offset_of!(PowerSavePhysicalPrefix, wake_stats_counter) == 0x02b);
    assert!(core::mem::offset_of!(PowerSavePhysicalPrefix, resume_state) == 0x02d);
    assert!(core::mem::offset_of!(PowerSavePhysicalPrefix, global_timer_duration) == 0x030);
    assert!(core::mem::offset_of!(PowerSavePhysicalPrefix, beacon_timing_reference) == 0x034);
    assert!(core::mem::offset_of!(PowerSavePhysicalPrefix, join_state_word) == 0x038);
    assert!(core::mem::offset_of!(PowerSavePhysicalPrefix, wake_lead_time) == 0x03c);
    assert!(core::mem::offset_of!(PowerSavePhysicalPrefix, mode) == 0x040);
    assert!(core::mem::offset_of!(PowerSavePhysicalPrefix, active) == 0x041);
    assert!(core::mem::offset_of!(PowerSavePhysicalPrefix, tx_pending_state) == 0x042);
    assert!(core::mem::offset_of!(PowerSavePhysicalPrefix, timer_state) == 0x043);
    assert!(core::mem::offset_of!(PowerSavePhysicalPrefix, flags) == 0x044);
    assert!(core::mem::offset_of!(PowerSavePhysicalPrefix, sleep_transition_flags) == 0x046);
    assert!(core::mem::offset_of!(PowerSavePhysicalPrefix, wake_stats_timestamp) == 0x048);
    assert!(core::mem::offset_of!(PowerSavePhysicalPrefix, backoff_adjustment) == 0x04c);
    assert!(core::mem::offset_of!(PowerSavePhysicalPrefix, reset_state) == 0x050);
    assert!(core::mem::offset_of!(PowerSavePhysicalPrefix, beacon_timing_valid) == 0x051);
    assert!(core::mem::offset_of!(PowerSavePhysicalPrefix, pending_control_kind) == 0x052);
    assert!(core::mem::offset_of!(PowerSavePhysicalPrefix, uapsd_state) == 0x053);
    assert!(core::mem::offset_of!(PowerSavePhysicalPrefix, pending) == 0x054);
    assert!(core::mem::offset_of!(PowerSavePhysicalPrefix, pending_flags) == 0x058);
    assert!(core::mem::offset_of!(PowerSavePhysicalPrefix, wake_reason) == 0x059);
    assert!(core::mem::offset_of!(PowerSavePhysicalPrefix, queue_mask) == 0x05a);
    assert!(core::mem::offset_of!(PowerSavePhysicalPrefix, tx_followup_state) == 0x05c);
    assert!(core::mem::offset_of!(PowerSavePhysicalPrefix, uapsd_restart_value) == 0x06c);
    assert!(core::mem::offset_of!(PowerSavePhysicalPrefix, timers) == 0x070);
    assert!(core::mem::offset_of!(PowerSavePhysicalPrefix, state_fc) == 0x0fc);
    assert!(core::mem::offset_of!(PowerSavePhysicalPrefix, tx_completion_pending) == 0x0fd);
    assert!(core::mem::offset_of!(PowerSavePhysicalPrefix, beacon_rx_state) == 0x0fe);
    assert!(core::mem::offset_of!(PowerSavePhysicalPrefix, beacon_rate) == 0x0ff);
    assert!(core::mem::offset_of!(PowerSavePhysicalPrefix, wake_timer_delay) == 0x100);
    assert_type_layout!(PowerSaveFamily, 0x208, 4);
    assert_type_layout!(PowerSaveHifBoundary, 0x44, 4);
    assert!(core::mem::offset_of!(PowerSaveHifBoundary, duration_14) == 0x14);
    assert!(core::mem::offset_of!(PowerSaveHifBoundary, duration_1c) == 0x1c);
    assert!(core::mem::offset_of!(PowerSaveHifBoundary, interval_24) == 0x24);
    assert!(core::mem::offset_of!(PowerSaveHifBoundary, counter_30) == 0x30);
    assert!(core::mem::offset_of!(PowerSaveHifBoundary, threshold_32) == 0x32);
    assert!(core::mem::offset_of!(PowerSaveHifBoundary, scan_completion_34) == 0x34);
    assert!(core::mem::offset_of!(PowerSaveHifBoundary, sleep_vote_count_36) == 0x36);
    assert!(core::mem::offset_of!(PowerSaveHifBoundary, ps_mode_error_reported_38) == 0x38);
    assert!(core::mem::offset_of!(PowerSaveHifBoundary, beacon_tim_state_40) == 0x40);
    assert_type_layout!(HostMessageFreeRing, 0x18, 4);
    assert!(core::mem::offset_of!(HostMessageFreeRing, entries) == 0x08);
    assert_type_layout!(DeferredTransferQueue, 0x14, 4);
    assert!(core::mem::offset_of!(DeferredTransferQueue, pending_head) == 0x04);
    assert!(core::mem::offset_of!(DeferredTransferQueue, pending_tail) == 0x08);
    assert!(core::mem::offset_of!(DeferredTransferQueue, completed_head) == 0x0c);
    assert!(core::mem::offset_of!(DeferredTransferQueue, completed_tail) == 0x10);
    assert_type_layout!(HifBufferState, 0x34, 4);
    assert!(core::mem::offset_of!(HifBufferState, transfer_queue) == 0x1c);
    assert!(core::mem::offset_of!(HifBufferState, control_word) == 0x30);
    assert_type_layout!(LegacyHifSoftwareState, 0x1d4, 4);
    assert!(core::mem::offset_of!(LegacyHifSoftwareState, coalesce_timer) == 0x0c);
    assert!(core::mem::offset_of!(LegacyHifSoftwareState, rx_buffers) == 0x20);
    assert!(core::mem::offset_of!(LegacyHifSoftwareState, rx_consumer) == 0xa0);
    assert!(core::mem::offset_of!(LegacyHifSoftwareState, tx_queue) == 0xa8);
    assert!(core::mem::offset_of!(LegacyHifSoftwareState, tx_producer) == 0x1a8);
    assert!(core::mem::offset_of!(LegacyHifSoftwareState, tx_consumer) == 0x1ac);
    assert!(core::mem::offset_of!(LegacyHifSoftwareState, queue_depth) == 0x1b0);
    assert!(core::mem::offset_of!(LegacyHifSoftwareState, queued_tx_producer) == 0x1b4);
    assert!(core::mem::offset_of!(LegacyHifSoftwareState, queued_tx_consumer) == 0x1b8);
    assert!(core::mem::offset_of!(LegacyHifSoftwareState, input_producer) == 0x1bc);
    assert!(core::mem::offset_of!(LegacyHifSoftwareState, input_consumer) == 0x1c0);
    assert!(core::mem::offset_of!(LegacyHifSoftwareState, input_ring_mask) == 0x1c4);
    assert!(core::mem::offset_of!(LegacyHifSoftwareState, input_descriptors) == 0x1c8);
    assert!(core::mem::offset_of!(LegacyHifSoftwareState, sequence_state) == 0x1cc);
    assert!(core::mem::offset_of!(LegacyHifSoftwareState, transport_state) == 0x1d0);
    assert_type_layout!(MicCompletionState, 0x14, 4);
    assert_type_layout!(PhyCalibrationReferences, 0x10, 4);
    assert!(core::mem::offset_of!(PhyCalibrationReferences, coefficient_i) == 0x00);
    assert!(core::mem::offset_of!(PhyCalibrationReferences, coefficient_q) == 0x04);
    assert!(core::mem::offset_of!(PhyCalibrationReferences, scale_i) == 0x08);
    assert!(core::mem::offset_of!(PhyCalibrationReferences, scale_q) == 0x0c);
    assert_type_layout!(PhyProfileState, 0x28, 4);
    assert!(core::mem::offset_of!(PhyProfileState, profile) == 0x02);
    assert!(core::mem::offset_of!(PhyProfileState, phase) == 0x03);
    assert!(core::mem::offset_of!(PhyProfileState, channel) == 0x06);
    assert!(core::mem::offset_of!(PhyProfileState, profile0_ready) == 0x0d);
    assert!(core::mem::offset_of!(PhyProfileState, auxiliary_state) == 0x10);
    assert!(core::mem::offset_of!(PhyProfileState, transition_gate) == 0x11);
    assert!(core::mem::offset_of!(PhyProfileState, calibration_stage) == 0x13);
    assert!(core::mem::offset_of!(PhyProfileState, profile0_state) == 0x14);
    assert!(core::mem::offset_of!(PhyProfileState, profile1_ready) == 0x15);
    assert!(core::mem::offset_of!(PhyProfileState, profile1_channel) == 0x16);
    assert!(core::mem::offset_of!(PhyProfileState, reference_word) == 0x20);
    assert_type_layout!(PhyMeasurementState, 0x38, 4);
    assert!(core::mem::offset_of!(PhyMeasurementState, control_08) == 0x08);
    assert!(core::mem::offset_of!(PhyMeasurementState, sample_width) == 0x0e);
    assert!(core::mem::offset_of!(PhyMeasurementState, offset_word) == 0x10);
    assert!(core::mem::offset_of!(PhyMeasurementState, correction) == 0x14);
    assert!(core::mem::offset_of!(PhyMeasurementState, override_value) == 0x17);
    assert!(core::mem::offset_of!(PhyMeasurementState, silicon_variant) == 0x18);
    assert!(core::mem::offset_of!(PhyMeasurementState, denominator) == 0x1c);
    assert!(core::mem::offset_of!(PhyMeasurementState, correction_offset) == 0x1e);
    assert!(core::mem::offset_of!(PhyMeasurementState, measured_a) == 0x20);
    assert!(core::mem::offset_of!(PhyMeasurementState, measured_b) == 0x24);
    assert!(core::mem::offset_of!(PhyMeasurementState, retained_state) == 0x35);
    assert!(core::mem::offset_of!(PhyMeasurementState, zero_select) == 0x37);
    assert_type_layout!(PhyChannelCacheState, 0x30, 4);
    assert!(core::mem::offset_of!(PhyChannelCacheState, startup_observation) == 0x18);
    assert!(core::mem::offset_of!(PhyChannelCacheState, retained_channel) == 0x22);
    assert!(core::mem::offset_of!(PhyChannelCacheState, calibration_state) == 0x24);
    assert!(core::mem::offset_of!(PhyChannelCacheState, calibration_aux) == 0x25);
    assert!(core::mem::offset_of!(PhyChannelCacheState, control_word) == 0x28);
    assert!(core::mem::offset_of!(PhyChannelCacheState, table_pointer) == 0x2c);
    assert_type_layout!(PhyTableControlState, 0x30, 4);
    assert!(core::mem::offset_of!(PhyTableControlState, table_a) == 0x10);
    assert!(core::mem::offset_of!(PhyTableControlState, table_b) == 0x14);
    assert!(core::mem::offset_of!(PhyTableControlState, state_scale) == 0x18);
    assert!(core::mem::offset_of!(PhyTableControlState, threshold) == 0x28);
    assert!(core::mem::offset_of!(PhyTableControlState, extended_settle) == 0x2c);
    assert!(core::mem::offset_of!(PhyTableControlState, control_2d) == 0x2d);
    assert_type_layout!(PhyCoreState, 0xd0, 4);
    assert!(core::mem::offset_of!(PhyCoreState, references) == 0x00);
    assert!(core::mem::offset_of!(PhyCoreState, profile_state) == 0x10);
    assert!(core::mem::offset_of!(PhyCoreState, measurement_state) == 0x38);
    assert!(core::mem::offset_of!(PhyCoreState, channel_cache_state) == 0x70);
    assert!(core::mem::offset_of!(PhyCoreState, table_control_state) == 0xa0);
    assert_type_layout!(PhyIqCalibrationSlot, 0x10, 4);
    assert_type_layout!(PhyIqCalibrationPage, 0x100, 4);
    assert!(core::mem::offset_of!(PhyIqCalibrationPage, slots) == 0x14);
    assert!(core::mem::offset_of!(PhyIqCalibrationPage, opaque_d4) == 0xd4);
    assert_type_layout!(PhyIqCalibrationResults, 0x38, 4);
    assert!(core::mem::offset_of!(PhyIqCalibrationResults, values) == 0x14);
    assert_type_layout!(PhyTail, 0x238, 4);
    assert!(core::mem::offset_of!(PhyTail, calibration_pages) == 0x00);
    assert!(core::mem::offset_of!(PhyTail, results) == 0x200);
    assert_type_layout!(ResearchMargin, 0x3bc, 4);
    assert_type_layout!(DtcmLayout, DTCM_STATE_SIZE, 4);
    assert_type_layout!(SharedDtcmState, DTCM_STATE_SIZE, 4);

    assert!(core::mem::offset_of!(InitializedVendorImage, tx_duration_timing) == 0x0138);
    assert!(core::mem::offset_of!(InitializedVendorImage, rate_encoding) == 0x0194);
    assert!(core::mem::offset_of!(InitializedVendorImage, rate_attributes) == 0x01aa);
    assert!(core::mem::offset_of!(InitializedVendorImage, rate_pair_table) == 0x01c0); assert_type_layout!(RatePairTable, 0x40, 4); assert!(core::mem::offset_of!(RatePairTable, pairs) == 0); assert!(core::mem::size_of::<[[SharedU8; 2]; 31]>() == 0x3e); assert!(core::mem::offset_of!(RatePairTable, opaque_3e) == 0x3e); assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, rate_pair_table) == 0x0400_01c0); assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, rate_pair_table) + core::mem::size_of::<RatePairTable>() == 0x0400_0200); assert_type_layout!(SchedulerTail, 0x60, 4); assert!(core::mem::size_of::<SchedulerTail>() == 0x60); assert!(core::mem::align_of::<SchedulerTail>() == 4);
    assert!(core::mem::offset_of!(InitializedVendorImage, initialized_rate_policies) == 0x0200);
    assert!(core::mem::size_of::<InitializedRatePolicies>() == 0x28);
    assert!(core::mem::offset_of!(InitializedRatePolicies, policies) == 0x00);
    assert!(core::mem::offset_of!(InitializedVendorImage, pre_completion_callback_words) == 0x0228);
    assert!(core::mem::offset_of!(InitializedVendorImage, visible_completion_words) == 0x0260);
    assert_type_layout!(QueuePipeMappings, 0x0c, 4);
    assert!(core::mem::offset_of!(QueuePipeMappings, pipe_order) == 0x00);
    assert!(core::mem::offset_of!(QueuePipeMappings, queue_to_access_category) == 0x04);
    assert!(core::mem::offset_of!(QueuePipeMappings, access_category_to_queue) == 0x08);
    assert!(core::mem::offset_of!(InitializedVendorImage, queue_pipe_mappings) == 0x02d8);
    assert!(core::mem::offset_of!(InitializedVendorImage, pre_tkip_sbox_tables) == 0x02e4);
    assert!(core::mem::offset_of!(InitializedVendorImage, tkip_sbox_tables) == 0x0310);
    assert!(core::mem::offset_of!(InitializedVendorImage, command_dispatch) == 0x0710);
    assert!(core::mem::offset_of!(InitializedVendorImage, aes_transfer_classes) == 0x0804);
    assert!(core::mem::offset_of!(InitializedVendorImage, aes_mode1_microcode) == 0x0830);
    assert!(core::mem::offset_of!(InitializedVendorImage, pre_phy_gain_register_write_lists) == 0x09de);
    assert!(core::mem::offset_of!(InitializedVendorImage, phy_gain_register_write_lists) == 0x0b60);
    assert!(core::mem::offset_of!(InitializedVendorImage, phy_init_register_write_list_0) == 0x0c10); assert!(core::mem::offset_of!(InitializedVendorImage, phy_init_register_write_list_1) == 0x0c20); assert!(core::mem::offset_of!(InitializedVendorImage, phy_init_register_write_list_2) == 0x0c48); assert!(core::mem::offset_of!(InitializedVendorImage, post_phy_init_register_write_lists_prefix) == 0x0c60); assert!(core::mem::offset_of!(InitializedVendorImage, initialized_phy_gain_source_records) == 0x0ca6); assert!(core::mem::offset_of!(InitializedVendorImage, post_initialized_phy_gain_source_records) == 0x0da8); assert_type_layout!(RfModeHalfwordTable, 0x48, 2); assert!(core::mem::offset_of!(RfModeHalfwordTable, entries) == 0); assert!(core::mem::offset_of!(InitializedVendorImage, rf_mode_halfword_table) == 0x0dd0); assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, rf_mode_halfword_table) == 0x0400_0dd0); assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, rf_mode_halfword_table) + core::mem::size_of::<RfModeHalfwordTable>() == 0x0400_0e18); assert!(RF_MODE_HALFWORD_TABLE.get() + 12 * core::mem::size_of::<SharedU16>() == 0x0400_0de8); assert!(core::mem::offset_of!(InitializedVendorImage, initialized_iq_calibration_gain_indices) == 0x0e18); assert!(core::mem::offset_of!(InitializedVendorImage, phy_cal_substate_register_write_list) == 0x0e48); assert_type_layout!(PhyCalSubstateRegisterWriteList, 0x48, 4); assert!(core::mem::offset_of!(PhyCalSubstateRegisterWriteList, writes) == 0); assert!(core::mem::size_of::<[RegisterWrite; 8]>() == 0x40); assert!(core::mem::offset_of!(PhyCalSubstateRegisterWriteList, terminator_address) == 0x40); assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, phy_cal_substate_register_write_list) + core::mem::size_of::<PhyCalSubstateRegisterWriteList>() == 0x0400_0e90); assert!(core::mem::offset_of!(InitializedVendorImage, dbg_expand_register_write_list) == 0x0e90); assert_type_layout!(DbgExpandRegisterWriteList, 0x30, 4); assert!(core::mem::offset_of!(DbgExpandRegisterWriteList, writes) == 0); assert!(core::mem::size_of::<[RegisterWrite; 5]>() == 0x28); assert!(core::mem::offset_of!(DbgExpandRegisterWriteList, terminator_address) == 0x28); assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, dbg_expand_register_write_list) + core::mem::size_of::<DbgExpandRegisterWriteList>() == 0x0400_0ec0); assert!(core::mem::offset_of!(InitializedVendorImage, register_write_lists_suffix) == 0x0ec0); assert!(core::mem::size_of::<OpaqueBytes<0x214>>() == 0x214); assert!(0x182 + 0x0b0 + 0x50 + (0x46 + 0x102 + 0x28 + 0x48 + 0x30 + (0x48 + 0x30 + 0x214)) == 0x6f6);
    assert!(core::mem::offset_of!(InitializedVendorImage, duration_quantum_pointers) == 0x10d4); assert!(core::mem::offset_of!(InitializedVendorImage, pre_measurement_workspace) == 0x10e4); assert!(core::mem::offset_of!(InitializedVendorImage, measurement_workspace) == 0x10f8); assert!(core::mem::offset_of!(InitializedVendorImage, tx_aggregate_expiration_delta) == 0x1160); assert!(core::mem::offset_of!(InitializedVendorImage, debug_command_descriptors) == 0x1164);
    assert_type_layout!(InitializedHifControl, 0x10, 4);
    assert!(core::mem::offset_of!(InitializedHifControl, queued_depth) == 0x00);
    assert!(core::mem::offset_of!(InitializedHifControl, pending_count) == 0x04);
    assert!(core::mem::offset_of!(InitializedHifControl, coalesce_enabled) == 0x08);
    assert!(core::mem::offset_of!(InitializedHifControl, pending_threshold) == 0x09);
    assert!(core::mem::offset_of!(InitializedHifControl, ring_depth_threshold) == 0x0a);
    assert!(core::mem::offset_of!(InitializedHifControl, count_threshold) == 0x0b);
    assert!(core::mem::offset_of!(InitializedHifControl, coalesce_delay) == 0x0c);
    assert!(core::mem::offset_of!(InitializedVendorImage, hif_control) == 0x11ac);
    assert!(core::mem::offset_of!(InitializedVendorImage, irq_callbacks) == 0x11bc); assert_type_layout!(PhyWatchdogCounter, 0x04, 4); assert!(core::mem::offset_of!(PhyWatchdogCounter, count) == 0); assert!(core::mem::offset_of!(InitializedVendorImage, phy_watchdog_counter) == 0x123c); assert_type_layout!(InitializedMultiVifBeaconTimerTail, 0x60, 4); assert_type_layout!(SharedU8, 0x01, 1); assert_type_layout!(SharedU32, 0x04, 4); assert!(core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, opaque_00) == 0); assert!(core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, interface_2_radio_latch) == 0x0f); assert!(core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, timer) == 0x10); assert!(core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, opaque_24) == 0x24); assert!(core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, measurement_dwell_timer) == 0x3c); assert!(core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, dtim_capture_latch) == 0x50); assert!(core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, opaque_51) == 0x51); assert!(core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, measurement_control_word_0) == 0x54); assert!(core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, measurement_control_word_1) == 0x58); assert!(core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, measurement_control_word_2) == 0x5c); assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, multi_vif_beacon_timer_tail) + core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, measurement_dwell_timer) + core::mem::size_of::<TimerEntry>() == 0x0400_1290); assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, multi_vif_beacon_timer_tail) + core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, dtim_capture_latch) + core::mem::size_of::<SharedU8>() == 0x0400_1291); assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, multi_vif_beacon_timer_tail) + core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, opaque_51) + core::mem::size_of::<OpaqueBytes<0x03>>() == 0x0400_1294); assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, multi_vif_beacon_timer_tail) + core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, measurement_control_word_0) + core::mem::size_of::<SharedU32>() == 0x0400_1298); assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, multi_vif_beacon_timer_tail) + core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, measurement_control_word_1) + core::mem::size_of::<SharedU32>() == 0x0400_129c); assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, multi_vif_beacon_timer_tail) + core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, measurement_control_word_2) + core::mem::size_of::<SharedU32>() == 0x0400_12a0); assert!(core::mem::offset_of!(InitializedVendorImage, multi_vif_beacon_timer_tail) == 0x1240); assert!(core::mem::offset_of!(InitializedVendorImage, ampdu_counters) == 0x12a0); assert_type_layout!(InitializedVendorImage, 0x2078, 4); assert_type_layout!(DtcmLayout, DTCM_STATE_SIZE, 4); assert_type_layout!(SharedDtcmState, DTCM_STATE_SIZE, 4);
    assert_type_layout!(AmpduTelemetryCounters, 0x28, 4);
    assert!(core::mem::offset_of!(AmpduTelemetryCounters, tx_error_frames) == 0x00);
    assert!(core::mem::offset_of!(AmpduTelemetryCounters, tx_counted_frames) == 0x04);
    assert!(core::mem::offset_of!(AmpduTelemetryCounters, tx_duration_low) == 0x08);
    assert!(core::mem::offset_of!(AmpduTelemetryCounters, tx_duration_high) == 0x0c);
    assert!(core::mem::offset_of!(AmpduTelemetryCounters, rx_management_0) == 0x10);
    assert!(core::mem::offset_of!(AmpduTelemetryCounters, rx_management_3) == 0x1c);
    assert!(core::mem::offset_of!(AmpduTelemetryCounters, opaque_20) == 0x20);
    assert!(core::mem::offset_of!(AmpduTelemetryCounters, tx_retry_count) == 0x24);
    assert!(core::mem::offset_of!(InitializedVendorImage, ampdu_counters) == 0x12a0); assert_type_layout!(SharedU32, 0x04, 4); assert_type_layout!(PerTidTelemetryBank, 0x140, 4); assert!(core::mem::size_of::<[SharedU32; 8]>() == 0x20);
    assert_type_layout!(RetryPathCounter, 0x04, 4); assert!(core::mem::offset_of!(RetryPathCounter, count) == 0x00); assert!(core::mem::offset_of!(InitializedVendorImage, retry_path_counter) == 0x12c8); assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, retry_path_counter) + core::mem::offset_of!(RetryPathCounter, count) == 0x0400_12c8); assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, retry_path_counter) + core::mem::size_of::<RetryPathCounter>() == 0x0400_12cc); assert!(core::mem::offset_of!(InitializedVendorImage, per_tid_telemetry_bank) == 0x12cc); assert!(core::mem::offset_of!(PerTidTelemetryBank, table_00) == 0x000); assert!(core::mem::offset_of!(PerTidTelemetryBank, table_01) == 0x020); assert!(core::mem::offset_of!(PerTidTelemetryBank, table_02) == 0x040); assert!(core::mem::offset_of!(PerTidTelemetryBank, table_03) == 0x060); assert!(core::mem::offset_of!(PerTidTelemetryBank, table_04) == 0x080); assert!(core::mem::offset_of!(PerTidTelemetryBank, table_05) == 0x0a0); assert!(core::mem::offset_of!(PerTidTelemetryBank, table_06) == 0x0c0); assert!(core::mem::offset_of!(PerTidTelemetryBank, table_07) == 0x0e0); assert!(core::mem::offset_of!(PerTidTelemetryBank, table_08) == 0x100); assert!(core::mem::offset_of!(PerTidTelemetryBank, table_09) == 0x120);
    assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, per_tid_telemetry_bank) + core::mem::offset_of!(PerTidTelemetryBank, table_00) == 0x0400_12cc); assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, per_tid_telemetry_bank) + core::mem::offset_of!(PerTidTelemetryBank, table_01) == 0x0400_12ec); assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, per_tid_telemetry_bank) + core::mem::offset_of!(PerTidTelemetryBank, table_02) == 0x0400_130c); assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, per_tid_telemetry_bank) + core::mem::offset_of!(PerTidTelemetryBank, table_03) == 0x0400_132c); assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, per_tid_telemetry_bank) + core::mem::offset_of!(PerTidTelemetryBank, table_04) == 0x0400_134c); assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, per_tid_telemetry_bank) + core::mem::offset_of!(PerTidTelemetryBank, table_05) == 0x0400_136c); assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, per_tid_telemetry_bank) + core::mem::offset_of!(PerTidTelemetryBank, table_06) == 0x0400_138c); assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, per_tid_telemetry_bank) + core::mem::offset_of!(PerTidTelemetryBank, table_07) == 0x0400_13ac); assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, per_tid_telemetry_bank) + core::mem::offset_of!(PerTidTelemetryBank, table_08) == 0x0400_13cc); assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, per_tid_telemetry_bank) + core::mem::offset_of!(PerTidTelemetryBank, table_09) == 0x0400_13ec); assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, per_tid_telemetry_bank) + core::mem::size_of::<PerTidTelemetryBank>() == 0x0400_140c); assert!(core::mem::offset_of!(InitializedVendorImage, ampdu_completion_control) == 0x140c);
    assert!(core::mem::size_of::<AmpduCompletionControl>() == 0x04);
    assert!(core::mem::offset_of!(AmpduCompletionControl, enabled) == 0x00);
    assert_type_layout!(SharedU32, 0x04, 4); assert_type_layout!(TxConfirmAggregationState, 0x0c, 4); assert!(core::mem::offset_of!(TxConfirmAggregationState, state) == 0x00); assert!(core::mem::offset_of!(TxConfirmAggregationState, pending_message_raw) == 0x04); assert!(core::mem::offset_of!(TxConfirmAggregationState, append_cursor_raw) == 0x08); assert!(core::mem::offset_of!(InitializedVendorImage, tx_confirm_aggregation_state) == 0x1410); assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, tx_confirm_aggregation_state) + core::mem::offset_of!(TxConfirmAggregationState, state) == 0x0400_1410); assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, tx_confirm_aggregation_state) + core::mem::offset_of!(TxConfirmAggregationState, pending_message_raw) == 0x0400_1414); assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, tx_confirm_aggregation_state) + core::mem::offset_of!(TxConfirmAggregationState, append_cursor_raw) == 0x0400_1418); assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, tx_confirm_aggregation_state) + core::mem::size_of::<TxConfirmAggregationState>() == 0x0400_141c); assert_type_layout!(ConfigurationApplyFlags, 0x04, 4); assert!(core::mem::offset_of!(ConfigurationApplyFlags, flags) == 0x00); assert!(core::mem::offset_of!(InitializedVendorImage, configuration_apply_flags) == 0x141c); assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, configuration_apply_flags) == 0x0400_141c); assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, configuration_apply_flags) + core::mem::offset_of!(ConfigurationApplyFlags, flags) == 0x0400_141c); assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, configuration_apply_flags) + core::mem::size_of::<ConfigurationApplyFlags>() == 0x0400_1420); assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, control_words) == 0x0400_1420); assert_type_layout!(InitializedVendorImage, 0x2078, 4); assert_type_layout!(DtcmLayout, DTCM_STATE_SIZE, 4); assert_type_layout!(SharedDtcmState, DTCM_STATE_SIZE, 4);
    assert_type_layout!(InitializedControlWords, 0x20, 4);
    assert!(core::mem::offset_of!(InitializedControlWords, beacon_state) == 0x00);
    assert!(core::mem::offset_of!(InitializedControlWords, rx_indication_state) == 0x04);
    assert!(core::mem::offset_of!(InitializedControlWords, tsf_resync_state) == 0x08);
    assert!(core::mem::offset_of!(InitializedControlWords, random_lfsr) == 0x0c);
    assert!(core::mem::offset_of!(InitializedControlWords, tsf_accumulator_low) == 0x10);
    assert!(core::mem::offset_of!(InitializedControlWords, timer_counter) == 0x1c);
    assert!(core::mem::offset_of!(InitializedVendorImage, control_words) == 0x1420);
    assert_type_layout!(SharedU32, 0x04, 4); assert_type_layout!(DebugPlatformLocalTail, 0x08, 4); assert!(core::mem::offset_of!(DebugPlatformLocalTail, platform_local_word_2c) == 0x00); assert!(core::mem::offset_of!(DebugPlatformLocalTail, platform_local_word_30) == 0x04); assert!(core::mem::offset_of!(InitializedVendorImage, pre_phy_channel_threshold_descriptors) == 0x1440); assert!(core::mem::offset_of!(InitializedVendorImage, debug_platform_local_tail) == 0x1454); assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, debug_platform_local_tail) + core::mem::offset_of!(DebugPlatformLocalTail, platform_local_word_2c) == 0x0400_1454); assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, debug_platform_local_tail) + core::mem::offset_of!(DebugPlatformLocalTail, platform_local_word_30) == 0x0400_1458); assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, debug_platform_local_tail) + core::mem::size_of::<DebugPlatformLocalTail>() == 0x0400_145c); assert!(core::mem::offset_of!(InitializedVendorImage, phy_channel_threshold_descriptors) == 0x145c); assert_type_layout!(InitializedVendorImage, 0x2078, 4); assert_type_layout!(DtcmLayout, DTCM_STATE_SIZE, 4); assert_type_layout!(SharedDtcmState, DTCM_STATE_SIZE, 4);
    assert!(core::mem::offset_of!(InitializedVendorImage, phy_channel_threshold_descriptors) == 0x145c);
    assert!(core::mem::size_of::<PhyChannelThresholdDescriptor>() == 0x08);
    assert!(core::mem::offset_of!(PhyChannelThresholdDescriptor, count) == 0x01);
    assert!(core::mem::offset_of!(PhyChannelThresholdDescriptor, default_threshold) == 0x02);
    assert!(core::mem::offset_of!(PhyChannelThresholdDescriptor, records) == 0x04);
    assert!(core::mem::offset_of!(InitializedVendorImage, phy_gain_programming_records) == 0x146c);
    assert!(core::mem::size_of::<PhyGainProgrammingRecord>() == 0x10);
    assert!(core::mem::offset_of!(PhyGainProgrammingRecord, requested_offset) == 0x02);
    assert!(core::mem::offset_of!(PhyGainProgrammingRecord, selected_power) == 0x04);
    assert!(core::mem::offset_of!(PhyGainProgrammingRecord, cleared_word) == 0x08);
    assert!(core::mem::offset_of!(PhyGainProgrammingRecord, gain_code) == 0x0c);
    assert!(core::mem::offset_of!(PhyGainProgrammingRecord, rssi_value) == 0x0e);
    assert_type_layout!(SharedU16, 0x02, 2); assert_type_layout!(PreHostPasRingObserved, 0x0c, 4); assert!(core::mem::offset_of!(PreHostPasRingObserved, opaque_00) == 0x00); assert!(core::mem::offset_of!(PreHostPasRingObserved, radio_stop_word_02) == 0x06); assert!(core::mem::offset_of!(PreHostPasRingObserved, opaque_08) == 0x08); assert!(core::mem::offset_of!(InitializedVendorImage, pre_host_pas_ring) == 0x156c); assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, pre_host_pas_ring) + core::mem::offset_of!(PreHostPasRingObserved, radio_stop_word_02) == 0x0400_1572); assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, pre_host_pas_ring) + core::mem::offset_of!(PreHostPasRingObserved, radio_stop_word_02) + core::mem::size_of::<SharedU16>() == 0x0400_1574); assert!(core::mem::offset_of!(InitializedVendorImage, host_pas_ring) == 0x1578); assert_type_layout!(InitializedVendorImage, 0x2078, 4); assert_type_layout!(DtcmLayout, DTCM_STATE_SIZE, 4); assert_type_layout!(SharedDtcmState, DTCM_STATE_SIZE, 4);
    assert!(core::mem::size_of::<HostPasRing>() == 0x108);
    assert!(core::mem::offset_of!(HostPasRing, head) == 0x00);
    assert!(core::mem::offset_of!(HostPasRing, tail) == 0x04);
    assert!(core::mem::offset_of!(HostPasRing, slots) == 0x08);
    assert!(core::mem::offset_of!(InitializedVendorImage, initialized_low_mac_prefix) == 0x1680);
    assert!(core::mem::size_of::<LowMacGlobalPrefix>() == 0xa0);
    assert!(core::mem::offset_of!(LowMacGlobalPrefix, rate_config) == 0x02);
    assert!(core::mem::offset_of!(LowMacGlobalPrefix, legacy_mode) == 0x05);
    assert!(core::mem::offset_of!(LowMacGlobalPrefix, producer) == 0x10);
    assert!(core::mem::offset_of!(LowMacGlobalPrefix, slot_time_base) == 0x1c);
    assert!(core::mem::offset_of!(LowMacGlobalPrefix, ifs_duration) == 0x44);
    assert!(core::mem::offset_of!(LowMacGlobalPrefix, short_airtimes) == 0x48);
    assert!(core::mem::offset_of!(LowMacGlobalPrefix, long_airtimes) == 0x74);
    assert!(core::mem::offset_of!(InitializedVendorImage, mac_pipe_records) == 0x1720);
    assert!(core::mem::size_of::<MacPipeSlot>() == 0x18);
    assert!(core::mem::offset_of!(MacPipeSlot, frame) == 0x0c);
    assert!(core::mem::offset_of!(MacPipeSlot, auxiliary) == 0x10);
    assert!(core::mem::offset_of!(MacPipeSlot, command) == 0x14);
    assert!(core::mem::size_of::<MacPipeRecord>() == 0x6c);
    assert!(core::mem::offset_of!(MacPipeRecord, state) == 0x03);
    assert!(core::mem::offset_of!(MacPipeRecord, hardware_ring) == 0x08);
    assert!(core::mem::offset_of!(MacPipeRecord, slots) == 0x0c);
    assert!(core::mem::offset_of!(InitializedVendorImage, mac_tx_queue_state) == 0x18d0);
    assert!(core::mem::size_of::<MacTxQueueState>() == 0x08);
    assert!(core::mem::offset_of!(MacTxQueueState, head) == 0x00);
    assert!(core::mem::offset_of!(MacTxQueueState, tail) == 0x04);
    assert!(core::mem::offset_of!(InitializedVendorImage, pre_mac_beacon_state) == 0x18d8);
    assert!(core::mem::offset_of!(InitializedVendorImage, mac_beacon_state) == 0x1a80);
    assert!(core::mem::size_of::<MacBeaconState>() == 0x40);
    assert!(core::mem::offset_of!(MacBeaconState, response_commands) == 0x08);
    assert!(core::mem::offset_of!(MacBeaconState, state) == 0x28);
    assert!(core::mem::offset_of!(MacBeaconState, secondary_command) == 0x2c);
    assert!(core::mem::offset_of!(MacBeaconState, control) == 0x30);
    assert!(core::mem::offset_of!(MacBeaconState, selector) == 0x34);
    assert!(core::mem::offset_of!(MacBeaconState, mode) == 0x38);
    assert!(core::mem::offset_of!(MacBeaconState, completion_word) == 0x3c);
    assert!(core::mem::offset_of!(InitializedVendorImage, mac_wake_runtime_state) == 0x1ac0);
    assert!(core::mem::size_of::<MacWakeRuntimeState>() == 0x48);
    assert!(core::mem::offset_of!(MacWakeRuntimeState, timer) == 0x08);
    assert!(core::mem::offset_of!(MacWakeRuntimeState, phy_state) == 0x1c);
    assert!(core::mem::offset_of!(MacWakeRuntimeState, transition_pending) == 0x1d);
    assert!(core::mem::offset_of!(MacWakeRuntimeState, restore_pending) == 0x1e);
    assert!(core::mem::offset_of!(MacWakeRuntimeState, mode) == 0x24);
    assert!(core::mem::offset_of!(MacWakeRuntimeState, control) == 0x28);
    assert!(core::mem::offset_of!(MacWakeRuntimeState, retry_rate_map) == 0x2c);
    assert!(core::mem::offset_of!(MacWakeRuntimeState, edca_slot_timing) == 0x44);
    assert_type_layout!(MacAggregateSlotTables, 0x200, 4); assert!(core::mem::offset_of!(MacAggregateSlotTables, queues) == 0); assert!(core::mem::offset_of!(InitializedVendorImage, pre_mac_phy_command_state) == 0x1b08); assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, pre_mac_phy_command_state) + core::mem::size_of::<OpaqueBytes<0x08>>() == 0x0400_1b10); assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, mac_aggregate_slot_tables) == 0x0400_1b10); assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, mac_aggregate_slot_tables) + core::mem::size_of::<MacAggregateSlotTables>() == 0x0400_1d10); assert!(core::mem::offset_of!(InitializedVendorImage, mac_phy_command_state) == 0x1d10);
    assert!(core::mem::size_of::<MacPhyCommandState>() == 0x4c);
    assert!(core::mem::offset_of!(MacPhyCommandState, radio_stop_state) == 0x02);
    assert!(core::mem::offset_of!(MacPhyCommandState, sideband_capture) == 0x04);
    assert!(core::mem::offset_of!(MacPhyCommandState, timer) == 0x08);
    assert!(core::mem::offset_of!(MacPhyCommandState, operation_state) == 0x1c);
    assert!(core::mem::offset_of!(MacPhyCommandState, operation_command) == 0x20);
    assert!(core::mem::offset_of!(MacPhyCommandState, operation_output_state) == 0x28);
    assert!(core::mem::offset_of!(MacPhyCommandState, operation_timeout) == 0x2c);
    assert!(core::mem::offset_of!(MacPhyCommandState, dispatch_command) == 0x30);
    assert!(core::mem::offset_of!(MacPhyCommandState, dispatch_output_state) == 0x38);
    assert!(core::mem::offset_of!(MacPhyCommandState, dispatch_output_timeout) == 0x3c);
    assert!(core::mem::offset_of!(MacPhyCommandState, completion_status) == 0x40);
    assert!(core::mem::offset_of!(MacPhyCommandState, interface) == 0x48);
    assert_type_layout!(MacPipeTail, 0x44, 4); assert_type_layout!(MacPipeTails, 0x110, 4); assert!(core::mem::offset_of!(MacPipeTails, records) == 0); assert!(core::mem::offset_of!(MacPipeTail, setup_word_6dc) == 0x00); assert!(core::mem::offset_of!(MacPipeTail, setup_word_6de) == 0x02); assert!(core::mem::offset_of!(MacPipeTail, setup_word_6e0) == 0x04); assert!(core::mem::offset_of!(MacPipeTail, opaque_6e2) == 0x06); assert!(core::mem::offset_of!(MacPipeTail, opaque_6e4) == 0x08); assert!(core::mem::offset_of!(MacPipeTail, type_byte_700) == 0x24); assert!(core::mem::offset_of!(MacPipeTail, opaque_701) == 0x25); assert!(core::mem::offset_of!(MacPipeTail, cleared_word_704) == 0x28); assert!(core::mem::offset_of!(MacPipeTail, counter_word_708) == 0x2c); assert!(core::mem::offset_of!(MacPipeTail, counter_word_70c) == 0x30); assert!(core::mem::offset_of!(MacPipeTail, counter_word_710) == 0x34); assert!(core::mem::offset_of!(MacPipeTail, counter_word_714) == 0x38); assert!(core::mem::offset_of!(MacPipeTail, frame_count_718) == 0x3c); assert!(core::mem::offset_of!(MacPipeTail, scratch_word_71c) == 0x40); assert!(core::mem::offset_of!(InitializedVendorImage, mac_pipe_tails) == 0x1d5c); assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, mac_pipe_tails) + core::mem::size_of::<MacPipeTails>() == 0x0400_1e6c); assert!(core::mem::offset_of!(InitializedVendorImage, mac_retry_hardware_state) == 0x1e6c);
    assert!(core::mem::size_of::<MacRetryHardwareState>() == 0x04);
    assert!(core::mem::offset_of!(MacRetryHardwareState, control) == 0x00);
    assert!(core::mem::offset_of!(InitializedVendorImage, pre_mac_runtime_accounting) == 0x1e70);
    assert!(core::mem::offset_of!(InitializedVendorImage, mac_runtime_accounting) == 0x1f78);
    assert!(core::mem::size_of::<SoftwareRecordNode>() == 0x08);
    assert!(core::mem::size_of::<SoftwareRecordFreeList>() == 0x24);
    assert!(core::mem::offset_of!(SoftwareRecordFreeList, nodes) == 0x04);
    assert!(core::mem::size_of::<MacRuntimeAccountingState>() == 0x54);
    assert!(core::mem::offset_of!(MacRuntimeAccountingState, status_accounting) == 0x04);
    assert!(core::mem::offset_of!(MacRuntimeAccountingState, sample_count) == 0x08);
    assert!(core::mem::offset_of!(MacRuntimeAccountingState, current_pipe_record) == 0x0c);
    assert!(core::mem::offset_of!(MacRuntimeAccountingState, pipe_event_flags) == 0x14);
    assert!(core::mem::offset_of!(MacRuntimeAccountingState, software_records) == 0x18);
    assert!(core::mem::offset_of!(MacRuntimeAccountingState, average) == 0x3c);
    assert!(core::mem::offset_of!(MacRuntimeAccountingState, silicon_control) == 0x44);
    assert!(core::mem::offset_of!(MacRuntimeAccountingState, parameter0) == 0x48);
    assert!(core::mem::offset_of!(MacRuntimeAccountingState, parameter1) == 0x4c);
    assert!(
        core::mem::offset_of!(InitializedVendorImage, initialized_low_mac_prefix) + 0x7ec == 0x1e6c
    );
    assert!(
        core::mem::offset_of!(InitializedVendorImage, initialized_low_mac_prefix) + 0x8f8 == 0x1f78
    );
    assert!(
        core::mem::offset_of!(InitializedVendorImage, initialized_low_mac_prefix) + 0x940 == 0x1fc0
    );
    assert_type_layout!(SchedulerExclusionState, 0x08, 4);
    assert!(core::mem::offset_of!(SchedulerExclusionState, exclusion_mask) == 0x00);
    assert!(core::mem::offset_of!(SchedulerExclusionState, secondary_exclusion) == 0x04);
    assert_type_layout!(SchedulerEventIsland, 0x44, 4);
    assert!(core::mem::offset_of!(SchedulerEventIsland, pending_events) == 0x00);
    assert!(core::mem::offset_of!(SchedulerEventIsland, runtime_flags) == 0x04);
    assert!(core::mem::offset_of!(SchedulerEventIsland, startup_mode) == 0x12);
    assert!(core::mem::offset_of!(SchedulerEventIsland, analog_enabled) == 0x1c);
    assert!(core::mem::offset_of!(SchedulerEventIsland, remap_primary) == 0x20);
    assert!(core::mem::offset_of!(SchedulerEventIsland, remap_secondary) == 0x28);
    assert!(core::mem::offset_of!(SchedulerEventIsland, analog_words) == 0x2c);
    assert!(core::mem::offset_of!(SchedulerEventIsland, timer_list_head) == 0x40);
    assert!(core::mem::offset_of!(InitializedVendorImage, scheduler_exclusion_state) == 0x1fcc);
    assert!(core::mem::offset_of!(InitializedVendorImage, scheduler_event_island) == 0x1fd4);
    assert!(core::mem::offset_of!(InitializedVendorImage, initialized_tail) == 0x2018);
    assert!(core::mem::size_of::<SchedulerTail>() == 0x60); assert!(core::mem::align_of::<SchedulerTail>() == 4);
    assert!(core::mem::offset_of!(SchedulerTail, hardware_timer_guard) == 0x10);
    assert!(core::mem::offset_of!(SchedulerTail, rf_calibration_bytes) == 0x14);
    assert!(core::mem::size_of::<[SharedU8; 32]>() == 0x20);
    assert!(core::mem::offset_of!(SchedulerTail, opaque_204c) == 0x34); assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, initialized_tail) + core::mem::offset_of!(SchedulerTail, opaque_204c) == 0x0400_204c);
    assert!(core::mem::offset_of!(SchedulerTail, error_event_counts) == 0x38);
    assert!(core::mem::size_of::<[SharedU32; 10]>() == 0x28);

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
    assert!(core::mem::offset_of!(DtcmLayout, ba_sessions) == 0x8e78);
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
    fn vif_addresses_are_typed_bounded_and_cross_record_wake_is_explicit() {
        assert_eq!(PRE_VIF_LINK_BITMAP.get(), 0x0400_3e90);
        assert_eq!(PRE_VIF_LINK_BITMAP.get() + 8, VIF_RECORDS.get());
        let first = vif_record(0).unwrap();
        let second = vif_record(1).unwrap();
        let third = vif_record(2).unwrap();
        assert_eq!(first.scan_rate_config().get(), 0x0400_3e98);
        assert_eq!(second.scan_rate_config().get(), 0x0400_4248);
        assert_eq!(third.scan_rate_config().get(), 0x0400_45f8);
        assert_eq!(first.radio_owner().get(), 0x0400_3edc);
        assert_eq!(first.rate_byte(7).unwrap().get(), 0x0400_3ebf);
        assert_eq!(first.wake_reinit_flag().get() + 0x3b0, second.wake_reinit_flag().get());
        assert_eq!(first.scan_rate_config().get() + 0x3c6, second.wake_reinit_flag().get());
        assert_eq!(third.link_gate().get(), 0x0400_475c);
        assert_eq!(first.operating_timer(0).unwrap().get(), 0x0400_3f48);
        assert_eq!(first.operating_timer(2).unwrap().get(), 0x0400_3f70);
        assert_eq!(first.link_timer(0).unwrap().get(), 0x0400_401c);
        assert_eq!(first.link_timer(1).unwrap().get(), 0x0400_4030);
        assert!(first.operating_timer(3).is_none());
        assert!(first.link_timer(2).is_none());
        assert!(vif_record(3).is_none());
        assert!(first.rate_byte(8).is_none());
        assert!(first.own_mac_byte(6).is_none());
        assert!(first.ssid_byte(32).is_none());
    }

    #[test]
    fn host_context_addresses_round_trip_and_reject_interior_or_gap_pointers() {
        let first = host_context(0).unwrap();
        let last = host_context(HOST_TX_CONTEXT_COUNT - 1).unwrap();
        assert_eq!(first.raw(), 0x0400_5a24);
        assert_eq!(last.raw(), 0x0400_83d4);
        assert_eq!(last.raw() + HOST_TX_CONTEXT_SIZE as u32, 0x0400_8544);
        assert_eq!(HostContextAddress::from_raw(first.raw()), Some(first));
        assert_eq!(HostContextAddress::from_raw(last.raw()), Some(last));
        assert_eq!(first.frame_node().raw(), first.raw() + 0x54);
        assert_eq!(first.frame_node().context(), first);
        assert_eq!(first.pas().raw(), first.frame_node().raw());
        assert_eq!(first.pas().context(), first);
        assert!(host_context(HOST_TX_CONTEXT_COUNT).is_none());
        assert!(HostContextAddress::from_raw(first.raw() - 4).is_none());
        assert!(HostContextAddress::from_raw(first.raw() + 4).is_none());
        assert!(HostContextAddress::from_raw(0x0400_8544).is_none());
        assert!(HostContextAddress::from_raw(HOST_CONTEXT_FREE_HEAD.get() as u32).is_none());
    }

    #[test]
    fn host_context_field_addresses_follow_the_semantic_layout() {
        let context = host_context(7).unwrap();
        let base = context.raw() as usize;
        let offsets = [
            (context.request_buffer(), 0x00),
            (context.intrusive_next(), 0x04),
            (context.packet_id(), 0x08),
            (context.requested_rate(), 0x0c),
            (context.queue_id(), 0x0d),
            (context.more(), 0x0e),
            (context.request_flags(), 0x0f),
            (context.expiry_time(), 0x10),
            (context.ht_tx_parameters(), 0x14),
            (context.borrowed_frame_length(), 0x18),
            (context.borrowed_frame_address(), 0x1c),
            (context.completion_status(), 0x20),
            (context.rate_copy(), 0x24),
            (context.saved_status(), 0x25),
            (context.completion_flags(), 0x26),
            (context.rate_try(0).unwrap(), 0x28),
            (context.rate_try(2).unwrap(), 0x30),
            (context.submit_timer(), 0x40),
            (context.header_length(), 0x44),
            (context.payload_length(), 0x48),
            (context.optional_pipe_object(), 0x4c),
            (context.sequence_or_callback_state(), 0x50),
            (context.submit_state(), 0x52),
            (context.completion_class(), 0x53),
            (context.frame_address(), 0x54),
            (context.control_bits(), 0x58),
            (context.frame_length(), 0x5c),
            (context.frame_control(), 0x5e),
            (context.access_category(), 0x60),
            (context.request_flag_rate_bits(), 0x61),
            (context.retry_policy(), 0x62),
            (context.tx_rate(), 0x63),
            (context.pas_expiry_time(), 0x64),
            (context.completion_timestamp(), 0x68),
            (context.scheduler_timestamp(), 0x6c),
            (context.terminal_status(), 0x70),
            (context.try_count(), 0x72),
            (context.ownership_bits(), 0x80),
            (context.duration(), 0x8a),
            (context.descriptor_state(), 0x90),
            (context.word_48(), 0x9c),
            (context.frame_state_address(), 0xa0),
            (context.auxiliary_state(), 0xa4),
            (context.tid(), 0xa6),
            (context.insertion_mode(), 0xa7),
            (context.sequence_number(), 0xa8),
            (context.retry_rate(), 0xaa),
            (context.byte_57(), 0xab),
            (context.interface(), 0xbd),
            (context.duration_slot(), 0xbe),
            (context.host_link(), 0xbf),
            (context.completion_byte_6c(), 0xc0),
            (context.qos_control(), 0xc8),
            (context.cipher_class(), 0xca),
            (context.word_7c(), 0xd0),
        ];
        for (address, offset) in offsets {
            assert_eq!(address.get() - base, offset);
        }
        assert!(context.rate_try(3).is_none());
        assert_eq!(context.expected_frame_state().raw(), crate::packet_ram::host_frame_state(7) as u32);
    }

    #[test]
    fn link_map_and_sequence_addresses_follow_the_decoded_layout() {
        let first = link_map_entry(0).unwrap();
        let last = link_map_entry(LINK_MAP_ENTRY_COUNT - 1).unwrap();
        assert_eq!(LINK_SEQUENCE_ROOT.get(), 0x0400_87b8);
        assert_eq!(link_map_entry_count().get(), 0x0400_87cc);
        assert_eq!(link_release_blocked_links().get(), 0x0400_87ce);
        assert_eq!(first.host_link().get(), 0x0400_87d0);
        assert_eq!(first.interface().get(), 0x0400_87d1);
        assert_eq!(first.internal_link().get(), 0x0400_87d2);
        assert_eq!(first.inactivity().get(), 0x0400_87d3);
        assert_eq!(first.release_flags().get(), 0x0400_87d4);
        assert_eq!(first.auxiliary_flags().get(), 0x0400_87d5);
        assert_eq!(first.mac_byte(0).unwrap().get(), 0x0400_87d6);
        assert_eq!(first.mac_byte(5).unwrap().get(), 0x0400_87db);
        assert_eq!(last.host_link().get(), 0x0400_8884);
        assert_eq!(last.mac_byte(5).unwrap().get(), 0x0400_888f);
        assert!(link_map_entry(LINK_MAP_ENTRY_COUNT).is_none());
        assert!(first.mac_byte(6).is_none());

        assert_eq!(link_sequence_counter(0, 0).unwrap().get(), 0x0400_8890);
        assert_eq!(link_sequence_counter(0, 15).unwrap().get(), 0x0400_88ae);
        assert_eq!(link_sequence_counter(9, 0).unwrap().get(), 0x0400_89b0);
        assert_eq!(link_sequence_counter(9, 15).unwrap().get(), 0x0400_89ce);
        assert!(link_sequence_counter(10, 0).is_none());
        assert!(link_sequence_counter(0, 16).is_none());
        assert_eq!(internal_link_bitmap().get(), 0x0400_89d0);
        assert_eq!(internal_link_bitmap().get() + 8, 0x0400_89d8);
    }

    #[test]
    fn completion_ring_view_addresses_are_exact() {
        assert_eq!(COMPLETION_RING_VIEW.get(), 0x0400_8f80);
        assert_eq!(completion_ring_entry(0).unwrap().get(), 0x0400_8f80);
        assert_eq!(completion_ring_entry(58).unwrap().get(), 0x0400_9068);
        assert_eq!(completion_ring_entry(59).unwrap().get(), 0x0400_906c);
        assert_eq!(internal_context_iv_seed().get(), completion_ring_entry(59).unwrap().get());
        assert_eq!(completion_ring_entry(63).unwrap().get(), 0x0400_907c);
        assert!(completion_ring_entry(64).is_none());
        assert_eq!(completion_ring_entry(63).unwrap().get() + 4, 0x0400_9080);
    }

    #[test]
    fn context_completion_prefix_addresses_are_exact() {
        assert_eq!(CONTEXT_COMPLETION_PREFIX.get(), 0x0400_8f6c);
        assert_eq!(class0_internal_context_count().get(), 0x0400_8f71);
        assert_eq!(CONTEXT_COMPLETION_PREFIX.get() + 0x14, 0x0400_8f80);
    }

    #[test]
    fn phy_gain_source_record_addresses_are_exact() {
        assert_eq!(PHY_GAIN_SOURCE_RECORDS.get(), 0x0400_2234);
        assert_eq!(phy_gain_source_record(0).unwrap().get(), 0x0400_2234);
        assert_eq!(phy_gain_source_record(21).unwrap().get(), 0x0400_22b2);
        assert!(phy_gain_source_record(22).is_none());
        assert_eq!(phy_gain_source_record(21).unwrap().get() + 6, 0x0400_22b8);
    }

    #[test]
    fn beacon_ie_index_view_addresses_are_exact() {
        assert_eq!(BEACON_FILTER_STORAGE.get(), 0x0400_22b8);
        assert_eq!(beacon_stored_byte(0).unwrap().get(), 0x0400_22bc);
        assert_eq!(beacon_stored_byte(699).unwrap().get(), 0x0400_2577);
        assert!(beacon_stored_byte(700).is_none());
        assert_eq!(BEACON_IE_INDEX_SELECTOR.get(), 0x0400_2578);
        assert_eq!(beacon_ie_index(0).unwrap().get(), 0x0400_257c);
        assert_eq!(beacon_ie_index(1).unwrap().get(), 0x0400_2780);
        assert!(beacon_ie_index(2).is_none());
        assert_eq!(beacon_ie_offset(0, 0).unwrap().get(), 0x0400_2580);
        assert_eq!(beacon_ie_offset(0, 255).unwrap().get(), 0x0400_277e);
        assert_eq!(beacon_ie_offset(1, 255).unwrap().get(), 0x0400_2982);
        assert!(beacon_ie_offset(0, 256).is_none());
        assert_eq!(beacon_ie_offset(1, 255).unwrap().get() + 2, 0x0400_2984);
    }

    #[test]
    fn rf_initialization_view_addresses_are_exact() {
        assert_eq!(RF_INITIALIZATION_VIEW.get(), 0x0400_2684);
        assert_eq!(RF_INITIALIZATION_ROOT.get(), 0x0400_2730);
        assert_eq!(RF_INITIALIZATION_ROOT.get() - RF_INITIALIZATION_VIEW.get(), 0xac);
        assert_eq!(RF_INITIALIZATION_VIEW.get() + core::mem::size_of::<RfInitializationObservedLayout>(), 0x0400_276c);
    }

    #[test]
    fn template_frame_descriptor_addresses_are_exact() {
        assert_eq!(TEMPLATE_FRAME_DESCRIPTORS.get(), 0x0400_3050);
        assert_eq!(template_frame_descriptor(0).unwrap().get(), 0x0400_3050);
        assert_eq!(template_frame_descriptor(1).unwrap().get(), 0x0400_3090);
        assert!(template_frame_descriptor(2).is_none());
        assert_eq!(template_frame_descriptor(1).unwrap().get() + 0x40, 0x0400_30d0);
        assert_eq!(TEMPLATE_BACKING_STORAGE.get(), 0x0400_30d0);
        assert_eq!(template_primary_buffer(0).unwrap().get(), 0x0400_30d0);
        assert_eq!(template_primary_buffer(1).unwrap().get(), 0x0400_31d0);
        assert_eq!(template_secondary_buffer(0).unwrap().get(), 0x0400_32d0);
        assert_eq!(template_secondary_buffer(1).unwrap().get(), 0x0400_3330);
        assert_eq!(template_tertiary_buffer(0).unwrap().get(), 0x0400_3390);
        assert_eq!(template_tertiary_buffer(1).unwrap().get(), 0x0400_3420);
        assert!(template_primary_buffer(2).is_none());
        assert_eq!(template_tertiary_buffer(1).unwrap().get() + 0x90, 0x0400_34b0);
    }

    #[test]
    fn sdd_profile_addresses_are_exact() {
        assert_eq!(SDD_CONFIGURATION_TABLES.get(), 0x0400_34b0);
        assert_eq!(sdd_profile(0).unwrap().get(), 0x0400_34b0);
        assert_eq!(sdd_profile(1).unwrap().get(), 0x0400_3542);
        assert!(sdd_profile(2).is_none());
        assert_eq!(sdd_rate_limit_unchecked(0, 10).get(), 0x0400_34c4);
        assert_eq!(sdd_channel_record_byte_unchecked(0, 0, 0).get(), 0x0400_34c6);
        assert_eq!(sdd_channel_count(0).unwrap().get(), 0x0400_34f6);
        assert_eq!(sdd_agc_correction_unchecked(0).get(), 0x0400_34f8);
        assert_eq!(sdd_calibration_coefficient_unchecked(0).get(), 0x0400_34fa);
        assert_eq!(sdd_conversion_value_unchecked(0, 0).get(), 0x0400_34fc);
        assert_eq!(sdd_rssi_coefficient_unchecked(0, 0).get(), 0x0400_3500);
        assert_eq!(sdd_rssi_rate_scale_unchecked(0, 0).get(), 0x0400_3504);
        assert_eq!(sdd_channel_record_byte_unchecked(1, 0, 0).get(), 0x0400_3558);
        assert_eq!(sdd_channel_count(1).unwrap().get(), 0x0400_3588);
        assert_eq!(sdd_agc_correction_unchecked(1).get(), 0x0400_358a);
        assert_eq!(sdd_rssi_rate_scale_unchecked(1, 0).get(), 0x0400_3596);
        assert_eq!(sdd_gain_coefficient_unchecked(0).get(), 0x0400_35ac);
        assert_eq!(sdd_gain_coefficient_unchecked(3).get(), 0x0400_35b2);
        assert_eq!(SDD_CONFIGURATION_TABLES.get() + 0x130, 0x0400_35e0);
    }

    #[test]
    fn wake_context_and_duration_addresses_are_exact() {
        assert_eq!(WAKE_CONTEXT_STATE.get(), 0x0400_35e0);
        assert_eq!(wake_clock_word(0).unwrap().get(), 0x0400_35e0);
        assert_eq!(wake_clock_word(3).unwrap().get(), 0x0400_35ec);
        assert!(wake_clock_word(4).is_none());
        assert_eq!(wake_response_pointer_unchecked(0).get(), 0x0400_35f0);
        assert_eq!(wake_response_pointer_unchecked(31).get(), 0x0400_366c);
        assert_eq!(DURATION_SOURCES.get(), 0x0400_3670);
        assert_eq!(duration_source(0).unwrap().get(), 0x0400_3670);
        assert_eq!(duration_source(1).unwrap().get(), 0x0400_3672);
        assert!(duration_source(2).is_none());
        assert_eq!(duration_source(1).unwrap().get() + 2, 0x0400_3674);
    }

    #[test]
    fn debug_console_addresses_are_exact() {
        assert_eq!(DEBUG_CONSOLE_STATE.get(), 0x0400_2094);
        assert_eq!(debug_console_command(0).unwrap().get(), 0x0400_20bc);
        assert_eq!(debug_console_command(31).unwrap().get(), 0x0400_2138);
        assert!(debug_console_command(32).is_none());
        assert_eq!(debug_console_line_byte(0).unwrap().get(), 0x0400_213c);
        assert_eq!(debug_console_line_byte(79).unwrap().get(), 0x0400_218b);
        assert!(debug_console_line_byte(80).is_none());
        assert_eq!(debug_console_line_byte(79).unwrap().get() + 1, 0x0400_218c);
    }

    #[test]
    fn runtime_register_and_backoff_addresses_are_exact() {
        assert_eq!(RUNTIME_REGISTER_BACKOFF_STATE.get(), 0x0400_2078);
        assert_eq!(runtime_register_context(0).unwrap().get(), 0x0400_2078);
        assert_eq!(runtime_register_context(3).unwrap().get(), 0x0400_2084);
        assert!(runtime_register_context(4).is_none());
        assert_eq!(pas_backoff_override_enabled().get(), 0x0400_2088);
        assert_eq!(pas_backoff_override_window().get(), 0x0400_208c);
        assert_eq!(pas_backoff_override_window().get() + 8, 0x0400_2094);
    }

    #[test]
    fn phy_calibration_reference_words_are_exact() {
        assert_eq!(phy_coefficient_i().get(), 0x0400_993c);
        assert_eq!(phy_coefficient_q().get(), 0x0400_9940);
        assert_eq!(phy_scale_i().get(), 0x0400_9944);
        assert_eq!(phy_scale_i_byte(1).unwrap().get(), 0x0400_9945);
        assert!(phy_scale_i_byte(4).is_none());
        assert_eq!(phy_scale_q().get(), 0x0400_9948);
        assert_eq!(phy_scale_q().get() + 4, 0x0400_994c);
        assert_eq!(PHY_PROFILE_STATE.get(), 0x0400_994c);
        assert_eq!(phy_profile().get(), 0x0400_994e);
        assert_eq!(phy_phase().get(), 0x0400_994f);
        assert_eq!(phy_channel().get(), 0x0400_9952);
        assert_eq!(phy_profile0_ready().get(), 0x0400_9959);
        assert_eq!(phy_auxiliary_state().get(), 0x0400_995c);
        assert_eq!(phy_transition_gate().get(), 0x0400_995d);
        assert_eq!(phy_calibration_stage().get(), 0x0400_995f);
        assert_eq!(phy_profile0_state().get(), 0x0400_9960);
        assert_eq!(phy_profile1_ready().get(), 0x0400_9961);
        assert_eq!(phy_profile1_channel().get(), 0x0400_9962);
        assert_eq!(phy_reference_word().get(), 0x0400_996c);
        assert_eq!(phy_reference_word().get() + 8, 0x0400_9974);
        assert_eq!(PHY_MEASUREMENT_STATE.get(), 0x0400_9974);
        assert_eq!(phy_measurement_control().get(), 0x0400_997c);
        assert_eq!(phy_sample_width().get(), 0x0400_9982);
        assert_eq!(phy_offset_word().get(), 0x0400_9984);
        assert_eq!(phy_correction().get(), 0x0400_9988);
        assert_eq!(phy_override_value().get(), 0x0400_998b);
        assert_eq!(phy_silicon_variant().get(), 0x0400_998c);
        assert_eq!(phy_denominator().get(), 0x0400_9990);
        assert_eq!(phy_correction_offset().get(), 0x0400_9992);
        assert_eq!(phy_measured_a().get(), 0x0400_9994);
        assert_eq!(phy_measured_b().get(), 0x0400_9998);
        assert_eq!(phy_retained_state().get(), 0x0400_99a9);
        assert_eq!(phy_zero_select().get() + 1, 0x0400_99ac);
        assert_eq!(PHY_CHANNEL_CACHE_STATE.get(), 0x0400_99ac);
        assert_eq!(phy_channel_cache(0).unwrap().get(), 0x0400_99ac);
        assert_eq!(phy_channel_cache(1).unwrap().get(), 0x0400_99b0);
        assert!(phy_channel_cache(2).is_none());
        assert_eq!(phy_startup_observation().get(), 0x0400_99c4);
        assert_eq!(phy_retained_channel().get(), 0x0400_99ce);
        assert_eq!(phy_calibration_state().get(), 0x0400_99d0);
        assert_eq!(phy_calibration_aux().get(), 0x0400_99d1);
        assert_eq!(phy_control_word().get(), 0x0400_99d4);
        assert_eq!(phy_table_pointer().get(), 0x0400_99d8);
        assert_eq!(phy_table_pointer().get() + 4, 0x0400_99dc);
        assert_eq!(PHY_TABLE_CONTROL_STATE.get(), 0x0400_99dc);
        assert_eq!(phy_calibration_table_a().get(), 0x0400_99ec);
        assert_eq!(phy_calibration_table_b().get(), 0x0400_99f0);
        assert_eq!(phy_state_scale().get(), 0x0400_99f4);
        assert_eq!(phy_threshold().get(), 0x0400_9a04);
        assert_eq!(phy_extended_settle().get(), 0x0400_9a08);
        assert_eq!(phy_table_control().get(), 0x0400_9a09);
        assert_eq!(phy_table_control().get() + 3, 0x0400_9a0c);
        assert_eq!(PHY_IQ_CALIBRATION_STATE.get(), 0x0400_9a0c);
        assert_eq!(phy_iq_calibration_slot(0, 0).unwrap().get(), 0x0400_9a20);
        assert_eq!(phy_iq_calibration_slot(0, 11).unwrap().get(), 0x0400_9ad0);
        assert_eq!(phy_iq_calibration_slot(1, 0).unwrap().get(), 0x0400_9b20);
        assert_eq!(phy_iq_calibration_slot(1, 11).unwrap().get(), 0x0400_9bd0);
        assert!(phy_iq_calibration_slot(2, 0).is_none());
        assert!(phy_iq_calibration_slot(0, 12).is_none());
        assert_eq!(phy_iq_calibration_result(0).unwrap().get(), 0x0400_9c20);
        assert_eq!(phy_iq_calibration_result(8).unwrap().get(), 0x0400_9c40);
        assert!(phy_iq_calibration_result(9).is_none());
        assert_eq!(phy_iq_calibration_result(8).unwrap().get() + 4, 0x0400_9c44);
    }

    #[test]
    fn hif_and_mic_queue_layouts_follow_retained_roots() {
        assert_eq!(hif_buffer_field(core::mem::offset_of!(HifBufferState, host_message_ring)).get(), 0x0400_9720);
        assert_eq!(hif_buffer_field(core::mem::offset_of!(HifBufferState, host_message_ring) + core::mem::offset_of!(HostMessageFreeRing, entries)).get(), 0x0400_9728);
        assert_eq!(hif_buffer_field(core::mem::offset_of!(HifBufferState, transfer_queue)).get(), 0x0400_973c);
        assert_eq!(hif_buffer_field(core::mem::offset_of!(HifBufferState, control_word)).get(), 0x0400_9750);
        assert_eq!(legacy_hif_field(core::mem::offset_of!(LegacyHifSoftwareState, mode)).get(), 0x0400_9754);
        assert_eq!(legacy_hif_field(core::mem::offset_of!(LegacyHifSoftwareState, coalesce_timer)).get(), 0x0400_9760);
        assert_eq!(legacy_hif_field(core::mem::offset_of!(LegacyHifSoftwareState, rx_buffers)).get(), 0x0400_9774);
        assert_eq!(legacy_hif_field(core::mem::offset_of!(LegacyHifSoftwareState, rx_consumer)).get(), 0x0400_97f4);
        assert_eq!(legacy_hif_field(core::mem::offset_of!(LegacyHifSoftwareState, tx_queue)).get(), 0x0400_97fc);
        assert_eq!(legacy_hif_field(core::mem::offset_of!(LegacyHifSoftwareState, tx_producer)).get(), 0x0400_98fc);
        assert_eq!(legacy_hif_field(core::mem::offset_of!(LegacyHifSoftwareState, tx_consumer)).get(), 0x0400_9900);
        assert_eq!(legacy_hif_field(core::mem::offset_of!(LegacyHifSoftwareState, queue_depth)).get(), 0x0400_9904);
        assert_eq!(legacy_hif_field(core::mem::offset_of!(LegacyHifSoftwareState, queued_tx_producer)).get(), 0x0400_9908);
        assert_eq!(legacy_hif_field(core::mem::offset_of!(LegacyHifSoftwareState, queued_tx_consumer)).get(), 0x0400_990c);
        assert_eq!(legacy_hif_field(core::mem::offset_of!(LegacyHifSoftwareState, input_producer)).get(), 0x0400_9910);
        assert_eq!(legacy_hif_field(core::mem::offset_of!(LegacyHifSoftwareState, input_consumer)).get(), 0x0400_9914);
        assert_eq!(legacy_hif_field(core::mem::offset_of!(LegacyHifSoftwareState, input_ring_mask)).get(), 0x0400_9918);
        assert_eq!(legacy_hif_field(core::mem::offset_of!(LegacyHifSoftwareState, input_descriptors)).get(), 0x0400_991c);
        assert_eq!(legacy_hif_field(core::mem::offset_of!(LegacyHifSoftwareState, sequence_state)).get(), 0x0400_9920);
        assert_eq!(legacy_hif_field(core::mem::offset_of!(LegacyHifSoftwareState, transport_state)).get() + 4, 0x0400_9928);
        assert_eq!(mic_completion_field(core::mem::offset_of!(MicCompletionState, queue)).get(), 0x0400_9928);
        assert_eq!(mic_completion_field(core::mem::size_of::<MicCompletionState>()).get(), 0x0400_993c);
    }

    #[test]
    fn power_save_timer_views_preserve_overlapping_extents() {
        assert_eq!(core::mem::offset_of!(PowerSaveFamily, physical_prefixes), 0);
        assert_eq!(POWER_SAVE_FAMILY.get() + core::mem::size_of::<PowerSavePhysicalPrefix>(), power_save_observed_view(1).unwrap().get());
        assert_eq!(power_save_wake_duration(0).unwrap().get(), 0x0400_94da);
        assert_eq!(power_save_wake_register_min(0).unwrap().get(), 0x0400_94dc);
        assert_eq!(power_save_wake_elapsed_max(0).unwrap().get(), 0x0400_94f0);
        assert_eq!(power_save_tx_completion_state(0).unwrap().get(), 0x0400_94f4);
        assert_eq!(power_save_next_tbtt(0).unwrap().get(), 0x0400_94f8);
        assert_eq!(power_save_doze_state(0).unwrap().get(), 0x0400_94fc);
        assert_eq!(power_save_requested_pm_mode(0).unwrap().get(), 0x0400_94fd);
        assert_eq!(power_save_requested_pm_mode(1).unwrap().get(), 0x0400_9601);
        assert_eq!(power_save_global_sleep_state().get(), 0x0400_94fe);
        assert_eq!(power_save_wake_stats_counter(0).unwrap().get(), 0x0400_94ff);
        assert_eq!(power_save_global_timer_duration().get(), 0x0400_9504);
        assert_eq!(power_save_beacon_timing_reference(0).unwrap().get(), 0x0400_9508);
        assert_eq!(power_save_join_state_word(0).unwrap().get(), 0x0400_950c);
        assert_eq!(power_save_wake_lead_time(0).unwrap().get(), 0x0400_9510);
        assert_eq!(power_save_active(0).unwrap().get(), 0x0400_9515);
        assert_eq!(power_save_active(1).unwrap().get(), 0x0400_9619);
        assert_eq!(power_save_sleep_transition_flags(0).unwrap().get(), 0x0400_951a);
        assert_eq!(power_save_wake_stats_timestamp(0).unwrap().get(), 0x0400_951c);
        assert_eq!(power_save_backoff_adjustment(0).unwrap().get(), 0x0400_9520);
        assert_eq!(power_save_pending_control_kind(0).unwrap().get(), 0x0400_9526);
        assert_eq!(power_save_pending_flags(0).unwrap().get(), 0x0400_952c);
        assert_eq!(power_save_uapsd_restart_value(0).unwrap().get(), 0x0400_9540);
        assert_eq!(power_save_timer(0, 0).unwrap().get(), 0x0400_9544);
        assert_eq!(power_save_timer(0, 6).unwrap().get(), 0x0400_95bc);
        assert_eq!(power_save_timer(1, 0).unwrap().get(), 0x0400_9648);
        assert_eq!(power_save_timer(1, 6).unwrap().get(), 0x0400_96c0);
        assert_eq!(power_save_timer(1, 6).unwrap().get() + 0x14, 0x0400_96d4);
        assert_eq!(power_save_tx_completion_pending(0).unwrap().get(), 0x0400_95d1);
        assert_eq!(power_save_beacon_rx_state(0).unwrap().get(), 0x0400_95d2);
        assert_eq!(power_save_beacon_rate(0).unwrap().get(), 0x0400_95d3);
        assert_eq!(power_save_wake_timer_delay(0).unwrap().get(), 0x0400_95d4);
        assert_eq!(power_save_wake_timer_delay(1).unwrap().get(), 0x0400_96d8);
        assert_eq!(power_save_beacon_airtime(0).unwrap().get(), 0x0400_95d8);
        assert_eq!(power_save_pre_tbtt_offset(0).unwrap().get(), 0x0400_95dc);
        assert_eq!(power_save_beacon_interval(0).unwrap().get(), 0x0400_95e0);
        assert_eq!(power_save_next_wake_deadline(0).unwrap().get(), 0x0400_95e4);
        assert_eq!(power_save_wake_timer_active(0).unwrap().get(), 0x0400_95e9);
        assert_eq!(power_save_beacon_timing_adjusted(0).unwrap().get(), 0x0400_95eb);
        assert_eq!(power_save_beacon_timing_adjusted(1).unwrap().get(), 0x0400_96ef);
        assert!(power_save_timer(2, 0).is_none());
        assert!(power_save_timer(0, 7).is_none());
        assert_eq!(power_save_extension_field(0, core::mem::offset_of!(PowerSaveObservedLayout, duration_118)).unwrap().get(), 0x0400_95ec);
        assert_eq!(power_save_extension_field(1, core::mem::offset_of!(PowerSaveObservedLayout, duration_118)).unwrap().get(), 0x0400_96f0);
        assert_eq!(power_save_extension_field(1, core::mem::offset_of!(PowerSaveObservedLayout, threshold_136)).unwrap().get() + 2, 0x0400_9710);
        assert_eq!(power_save_mode_control(0).unwrap().get(), 0x0400_95f8);
        assert_eq!(power_save_maximum_backoff(0).unwrap().get(), 0x0400_9600);
        assert_eq!(power_save_last_beacon_timestamp(0).unwrap().get(), 0x0400_9604);
        assert_eq!(power_save_ps_mode_error_reported(0).unwrap().get(), 0x0400_9610);
        assert_eq!(power_save_ps_mode_error_reported(1).unwrap().get(), 0x0400_9714);
        assert_eq!(power_save_scan_completion(1).unwrap().get(), 0x0400_9710);
        assert_eq!(power_save_sleep_vote_count(1).unwrap().get(), 0x0400_9712);
        assert_eq!(POWER_SAVE_BEACON_TIM_STATE.get(), 0x0400_971c);
        assert_eq!(POWER_SAVE_BEACON_TIM_STATE.get() + 4, HIF_BUFFER_STATE.get());
    }

    #[test]
    fn clock_parameters_and_scheduler_handlers_are_exact() {
        assert_eq!(clock_parameter_field(core::mem::offset_of!(ClockParameterIsland, mac_clock_snapshot)).get(), 0x0400_2190);
        assert_eq!(clock_parameter_field(core::mem::offset_of!(ClockParameterIsland, hardware_counter_cache)).get(), 0x0400_2198);
        assert_eq!(clock_parameter_field(core::mem::offset_of!(ClockParameterIsland, conversion_factor)).get(), 0x0400_21a0);
        assert_eq!(clock_parameter_field(core::mem::offset_of!(ClockParameterIsland, conversion_mode)).get(), 0x0400_21a8);
        assert_eq!(clock_parameter_field(core::mem::offset_of!(ClockParameterIsland, correction_offset)).get(), 0x0400_21b0);
        assert_eq!(scheduler_handler(0).unwrap().get(), 0x0400_21b4);
        assert_eq!(scheduler_handler(31).unwrap().get(), 0x0400_2230);
        assert_eq!(scheduler_handler(31).unwrap().get() + 4, 0x0400_2234);
        assert!(scheduler_handler(32).is_none());
    }

    #[test]
    fn initialized_phy_descriptor_and_gain_record_addresses_are_exact() {
        assert_eq!(PHY_CHANNEL_THRESHOLD_DESCRIPTORS.get(), 0x0400_145c);
        assert_eq!(phy_channel_threshold_descriptor(0).unwrap().get(), 0x0400_145c);
        assert_eq!(phy_channel_threshold_descriptor(1).unwrap().get(), 0x0400_1464);
        assert!(phy_channel_threshold_descriptor(2).is_none());
        assert_eq!(PHY_GAIN_PROGRAMMING_RECORDS.get(), 0x0400_146c);
        assert_eq!(phy_gain_programming_record(0).unwrap().get(), 0x0400_146c);
        assert_eq!(phy_gain_programming_record(15).unwrap().get(), 0x0400_155c);
        assert_eq!(phy_gain_programming_record(15).unwrap().get() + 0x10, 0x0400_156c);
        assert!(phy_gain_programming_record(16).is_none());
    }

    #[test]
    fn initialized_rate_policy_word_addresses_are_exact() {
        assert_eq!(INITIALIZED_RATE_POLICIES.get(), 0x0400_0200);
        assert_eq!(initialized_rate_policy_word(0, 0).unwrap().get(), 0x0400_0200);
        assert_eq!(initialized_rate_policy_word(0, 4).unwrap().get(), 0x0400_0210);
        assert_eq!(initialized_rate_policy_word(1, 0).unwrap().get(), 0x0400_0214);
        assert_eq!(initialized_rate_policy_word(1, 4).unwrap().get(), 0x0400_0224);
        assert_eq!(initialized_rate_policy_word(1, 4).unwrap().get() + 4, 0x0400_0228);
        assert!(initialized_rate_policy_word(2, 0).is_none());
        assert!(initialized_rate_policy_word(0, 5).is_none());
    }

    #[test]
    fn initialized_mac_tx_queue_addresses_are_exact() {
        assert_eq!(MAC_TX_QUEUE_STATE.get(), 0x0400_18d0);
        assert_eq!(MAC_TX_QUEUE_HEAD.get(), 0x0400_18d0);
        assert_eq!(MAC_TX_QUEUE_TAIL.get(), 0x0400_18d4);
        assert_eq!(MAC_TX_QUEUE_TAIL.get() + 4, 0x0400_18d8);
    }

    #[test]
    fn initialized_mac_retry_hardware_state_address_is_exact() {
        assert_eq!(MAC_RETRY_HARDWARE_STATE.get(), 0x0400_1e6c);
        assert_eq!(MAC_RETRY_HARDWARE_STATE.get() + 4, 0x0400_1e70);
    }

    #[test]
    fn initialized_mac_runtime_accounting_addresses_are_exact() {
        assert_eq!(MAC_RUNTIME_ACCOUNTING.get(), 0x0400_1f78);
        assert_eq!(MAC_CURRENT_PIPE.get(), 0x0400_1f78);
        assert_eq!(MAC_STATUS_ACCOUNTING.get(), 0x0400_1f7c);
        assert_eq!(MAC_SAMPLE_COUNT.get(), 0x0400_1f80);
        assert_eq!(MAC_CURRENT_PIPE_RECORD.get(), 0x0400_1f84);
        assert_eq!(MAC_CURRENT_SLOT.get(), 0x0400_1f88);
        assert_eq!(MAC_PIPE_EVENT_FLAGS.get(), 0x0400_1f8c);
        assert_eq!(MAC_SOFTWARE_RECORDS.get(), 0x0400_1f90);
        assert_eq!(MAC_ACCOUNTING_AVERAGE.get(), 0x0400_1fb4);
        assert_eq!(MAC_SILICON_CONTROL.get(), 0x0400_1fbc);
        assert_eq!(MAC_ACCOUNTING_PARAMETER0.get(), 0x0400_1fc0);
        assert_eq!(MAC_ACCOUNTING_PARAMETER1.get(), 0x0400_1fc4);
        assert_eq!(MAC_RUNTIME_ACCOUNTING.get() + 0x54, 0x0400_1fcc);
    }

    #[test]
    fn initialized_mac_pipe_tails_are_exact() { let image = DTCM_STATE_BASE; let tails = image + core::mem::offset_of!(InitializedVendorImage, mac_pipe_tails); assert_eq!(core::mem::size_of::<MacPipeTail>(), 0x44); assert_eq!(core::mem::align_of::<MacPipeTail>(), 4); assert_eq!(core::mem::size_of::<MacPipeTails>(), 0x110); assert_eq!(core::mem::align_of::<MacPipeTails>(), 4); assert_eq!([core::mem::offset_of!(MacPipeTail, setup_word_6dc), core::mem::offset_of!(MacPipeTail, setup_word_6de), core::mem::offset_of!(MacPipeTail, setup_word_6e0), core::mem::offset_of!(MacPipeTail, opaque_6e2), core::mem::offset_of!(MacPipeTail, opaque_6e4), core::mem::offset_of!(MacPipeTail, type_byte_700), core::mem::offset_of!(MacPipeTail, opaque_701), core::mem::offset_of!(MacPipeTail, cleared_word_704), core::mem::offset_of!(MacPipeTail, counter_word_708), core::mem::offset_of!(MacPipeTail, counter_word_70c), core::mem::offset_of!(MacPipeTail, counter_word_710), core::mem::offset_of!(MacPipeTail, counter_word_714), core::mem::offset_of!(MacPipeTail, frame_count_718), core::mem::offset_of!(MacPipeTail, scratch_word_71c)], [0x00, 0x02, 0x04, 0x06, 0x08, 0x24, 0x25, 0x28, 0x2c, 0x30, 0x34, 0x38, 0x3c, 0x40]); assert_eq!(tails, 0x0400_1d5c); assert_eq!(tails + core::mem::size_of::<MacPipeTail>(), 0x0400_1da0); assert_eq!(tails + 3 * core::mem::size_of::<MacPipeTail>(), 0x0400_1e28); assert_eq!(tails + core::mem::size_of::<MacPipeTails>(), 0x0400_1e6c); assert_eq!(MAC_PHY_COMMAND_STATE.get() + core::mem::size_of::<MacPhyCommandState>(), 0x0400_1d5c); assert_eq!(MAC_RETRY_HARDWARE_STATE.get(), 0x0400_1e6c); assert_eq!(core::mem::size_of::<InitializedVendorImage>(), 0x2078); assert_eq!(core::mem::align_of::<InitializedVendorImage>(), 4); assert_eq!(core::mem::size_of::<DtcmLayout>(), DTCM_STATE_SIZE); assert_eq!(core::mem::align_of::<DtcmLayout>(), 4); assert_eq!(core::mem::size_of::<SharedDtcmState>(), DTCM_STATE_SIZE); assert_eq!(core::mem::align_of::<SharedDtcmState>(), 4); } #[test]
    fn initialized_mac_phy_command_addresses_are_exact() {
        assert_eq!(MAC_PHY_COMMAND_STATE.get(), 0x0400_1d10);
        assert_eq!(MAC_RADIO_STOP_STATE.get(), 0x0400_1d12);
        assert_eq!(MAC_SIDEBAND_CAPTURE.get(), 0x0400_1d14);
        assert_eq!(MAC_PHY_OPERATION_TIMER.get(), 0x0400_1d18);
        assert_eq!(MAC_PHY_OPERATION_ROOT.get(), 0x0400_1d20);
        assert_eq!(MAC_PHY_OPERATION_STATE.get(), 0x0400_1d2c);
        assert_eq!(MAC_PHY_OPERATION_COMMAND.get(), 0x0400_1d30);
        assert_eq!(MAC_PHY_OPERATION_OUTPUT.get(), 0x0400_1d38);
        assert_eq!(MAC_PHY_OPERATION_TIMEOUT.get(), 0x0400_1d3c);
        assert_eq!(MAC_PHY_DISPATCH_COMMAND.get(), 0x0400_1d40);
        assert_eq!(MAC_PHY_DISPATCH_OUTPUT.get(), 0x0400_1d48);
        assert_eq!(MAC_PHY_COMPLETION_STATUS.get(), 0x0400_1d50);
        assert_eq!(MAC_PHY_INTERFACE.get(), 0x0400_1d58);
        assert_eq!(MAC_PHY_COMMAND_STATE.get() + 0x4c, 0x0400_1d5c);
    }

    #[test]
    fn initialized_mac_wake_runtime_addresses_are_exact() {
        assert_eq!(MAC_WAKE_RUNTIME_STATE.get(), 0x0400_1ac0);
        assert_eq!(MAC_WAKE_TIMER.get(), 0x0400_1ac8);
        assert_eq!(MAC_WAKE_PHY_STATE.get(), 0x0400_1adc);
        assert_eq!(MAC_WAKE_TRANSITION_PENDING.get(), 0x0400_1add);
        assert_eq!(MAC_WAKE_RESTORE_PENDING.get(), 0x0400_1ade);
        assert_eq!(MAC_WAKE_MODE.get(), 0x0400_1ae4);
        assert_eq!(MAC_WAKE_CONTROL.get(), 0x0400_1ae8);
        assert_eq!(MAC_RETRY_RATE_MAP.get(), 0x0400_1aec);
        assert_eq!(MAC_RETRY_RATE_MAP.get() + 22, 0x0400_1b02);
        assert_eq!(MAC_EDCA_SLOT_TIMING.get(), 0x0400_1b04);
        assert_eq!(MAC_EDCA_SLOT_TIMING.get() + 4, 0x0400_1b08);
    }

    #[test]
    fn initialized_mac_beacon_state_addresses_are_exact() {
        assert_eq!(MAC_BEACON_STATE.get(), 0x0400_1a80);
        assert_eq!(mac_beacon_response_command(0).unwrap().get(), 0x0400_1a88);
        assert_eq!(mac_beacon_response_command(1).unwrap().get(), 0x0400_1a8c);
        assert!(mac_beacon_response_command(2).is_none());
        assert_eq!(MAC_BEACON_CONTROL_STATE.get(), 0x0400_1aa8);
        assert_eq!(MAC_BEACON_SECONDARY_COMMAND.get(), 0x0400_1aac);
        assert_eq!(MAC_BEACON_CONTROL.get(), 0x0400_1ab0);
        assert_eq!(MAC_BEACON_SELECTOR.get(), 0x0400_1ab4);
        assert_eq!(MAC_BEACON_MODE.get(), 0x0400_1ab8);
        assert_eq!(MAC_BEACON_COMPLETION_WORD.get(), 0x0400_1abc);
        assert_eq!(MAC_BEACON_COMPLETION_WORD.get() + 4, 0x0400_1ac0);
    }

    #[test]
    fn initialized_low_mac_global_addresses_are_exact() {
        assert_eq!(LOW_MAC_GLOBAL.get(), 0x0400_1680);
        assert_eq!(LOW_MAC_FIFO_STATUS.get(), 0x0400_1681);
        assert_eq!(LOW_MAC_LEGACY_MODE.get(), 0x0400_1685);
        assert_eq!(LOW_MAC_SHORT_AIRTIME_TABLE.get(), 0x0400_16c8);
        assert_eq!(LOW_MAC_LONG_AIRTIME_TABLE.get(), 0x0400_16f4);
        assert_eq!(LOW_MAC_GLOBAL.get() + 0xa0, MAC_PIPE_RECORDS.get());
    }

    #[test]
    fn initialized_mac_pipe_record_addresses_are_exact() {
        assert_eq!(MAC_PIPE_RECORDS.get(), 0x0400_1720);
        assert_eq!(mac_pipe_record(0).unwrap().get(), 0x0400_1720);
        assert_eq!(mac_pipe_record(3).unwrap().get(), 0x0400_1864);
        assert_eq!(mac_pipe_record(3).unwrap().get() + 0x6c, 0x0400_18d0);
        assert!(mac_pipe_record(4).is_none());
    }

    #[test]
    fn initialized_pre_host_pas_radio_stop_word_address_is_exact() { let image = DTCM_STATE_BASE; let observed = image + core::mem::offset_of!(InitializedVendorImage, pre_host_pas_ring); let prefix = observed + core::mem::offset_of!(PreHostPasRingObserved, opaque_00); let word = observed + core::mem::offset_of!(PreHostPasRingObserved, radio_stop_word_02); let suffix = observed + core::mem::offset_of!(PreHostPasRingObserved, opaque_08); assert_eq!(core::mem::size_of::<SharedU16>(), 0x02); assert_eq!(core::mem::align_of::<SharedU16>(), 2); assert_eq!([core::mem::offset_of!(PreHostPasRingObserved, opaque_00), core::mem::offset_of!(PreHostPasRingObserved, radio_stop_word_02), core::mem::offset_of!(PreHostPasRingObserved, opaque_08)], [0x00, 0x06, 0x08]); assert_eq!(core::mem::size_of::<PreHostPasRingObserved>(), 0x0c); assert_eq!(core::mem::align_of::<PreHostPasRingObserved>(), 4); assert_eq!(observed, 0x0400_156c); assert_eq!(prefix, 0x0400_156c); assert_eq!(core::mem::size_of::<OpaqueBytes<0x06>>(), 0x06); assert_eq!(prefix + core::mem::size_of::<OpaqueBytes<0x06>>(), 0x0400_1572); assert_eq!(RADIO_STOP_WORD_02.get(), 0x0400_1572); assert_eq!(word, RADIO_STOP_WORD_02.get()); assert_eq!(word + core::mem::size_of::<SharedU16>(), 0x0400_1574); assert_eq!(suffix, 0x0400_1574); assert_eq!(core::mem::size_of::<OpaqueBytes<0x04>>(), 0x04); assert_eq!(suffix + core::mem::size_of::<OpaqueBytes<0x04>>(), 0x0400_1578); assert_eq!(HOST_PAS_RING.get(), 0x0400_1578); assert_eq!(core::mem::size_of::<InitializedVendorImage>(), 0x2078); assert_eq!(core::mem::align_of::<InitializedVendorImage>(), 4); assert_eq!(core::mem::size_of::<DtcmLayout>(), DTCM_STATE_SIZE); assert_eq!(core::mem::align_of::<DtcmLayout>(), 4); assert_eq!(core::mem::size_of::<SharedDtcmState>(), DTCM_STATE_SIZE); assert_eq!(core::mem::align_of::<SharedDtcmState>(), 4); }

    #[test]
    fn initialized_host_pas_ring_addresses_are_exact() {
        assert_eq!(HOST_PAS_RING.get(), 0x0400_1578);
        assert_eq!(HOST_PAS_RING_HEAD.get(), 0x0400_1578);
        assert_eq!(HOST_PAS_RING_TAIL.get(), 0x0400_157c);
        assert_eq!(HOST_PAS_RING_SLOTS.get(), 0x0400_1580);
        assert_eq!(host_pas_ring_slot(0).unwrap().get(), 0x0400_1580);
        assert_eq!(host_pas_ring_slot(63).unwrap().get(), 0x0400_167c);
        assert_eq!(host_pas_ring_slot(63).unwrap().get() + 4, 0x0400_1680);
        assert!(host_pas_ring_slot(64).is_none());
    }

    #[test]
    fn initialized_irq_callback_addresses_are_exact() {
        assert_eq!(IRQ_CALLBACK_TABLE.get(), 0x0400_11bc);
        assert_eq!(irq_callback(0).unwrap().get(), 0x0400_11bc);
        assert_eq!(irq_callback(31).unwrap().get(), 0x0400_1238);
        assert_eq!(irq_callback(31).unwrap().get() + 4, 0x0400_123c);
        assert!(irq_callback(32).is_none());
    }

    #[test]
    fn initialized_phy_watchdog_counter_address_is_exact() { assert_eq!(core::mem::size_of::<PhyWatchdogCounter>(), 0x04); assert_eq!(core::mem::align_of::<PhyWatchdogCounter>(), 4); assert_eq!(core::mem::offset_of!(PhyWatchdogCounter, count), 0); assert_eq!(IRQ_CALLBACK_TABLE.get() + core::mem::size_of::<[SharedU32; 32]>(), 0x0400_123c); assert_eq!(PHY_WATCHDOG_COUNTER.get(), 0x0400_123c); assert_eq!(PHY_WATCHDOG_COUNTER.get() + core::mem::size_of::<PhyWatchdogCounter>(), 0x0400_1240); assert_eq!(core::mem::offset_of!(InitializedVendorImage, multi_vif_beacon_timer_tail), 0x1240); assert_eq!(core::mem::size_of::<InitializedMultiVifBeaconTimerTail>(), 0x60); assert_eq!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, multi_vif_beacon_timer_tail) + core::mem::size_of::<InitializedMultiVifBeaconTimerTail>(), 0x0400_12a0); assert_eq!(AMPDU_TELEMETRY_COUNTERS.get(), 0x0400_12a0); assert_eq!(core::mem::size_of::<InitializedVendorImage>(), 0x2078); assert_eq!(core::mem::align_of::<InitializedVendorImage>(), 4); assert_eq!(core::mem::size_of::<DtcmLayout>(), DTCM_STATE_SIZE); assert_eq!(core::mem::align_of::<DtcmLayout>(), 4); assert_eq!(core::mem::size_of::<SharedDtcmState>(), DTCM_STATE_SIZE); assert_eq!(core::mem::align_of::<SharedDtcmState>(), 4); } #[test] fn initialized_multi_vif_beacon_timer_address_is_exact() { assert_eq!(core::mem::size_of::<TimerEntry>(), 0x14); assert_eq!(core::mem::align_of::<TimerEntry>(), 4); assert_eq!([core::mem::offset_of!(TimerEntry, next), core::mem::offset_of!(TimerEntry, previous_link), core::mem::offset_of!(TimerEntry, deadline), core::mem::offset_of!(TimerEntry, callback), core::mem::offset_of!(TimerEntry, context)], [0x00, 0x04, 0x08, 0x0c, 0x10]); assert_eq!(core::mem::size_of::<InitializedMultiVifBeaconTimerTail>(), 0x60); assert_eq!(core::mem::align_of::<InitializedMultiVifBeaconTimerTail>(), 4); assert_eq!([core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, opaque_00), core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, interface_2_radio_latch), core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, timer), core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, opaque_24), core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, measurement_dwell_timer), core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, dtim_capture_latch), core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, opaque_51), core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, measurement_control_word_0), core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, measurement_control_word_1), core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, measurement_control_word_2)], [0x00, 0x0f, 0x10, 0x24, 0x3c, 0x50, 0x51, 0x54, 0x58, 0x5c]); assert_eq!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, multi_vif_beacon_timer_tail), 0x0400_1240); assert_eq!(MULTI_VIF_BEACON_TIMER.get(), 0x0400_1250); assert_eq!(MULTI_VIF_BEACON_TIMER.get() + core::mem::size_of::<TimerEntry>(), 0x0400_1264); assert_eq!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, multi_vif_beacon_timer_tail) + core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, opaque_51) + core::mem::size_of::<OpaqueBytes<0x03>>(), 0x0400_1294); assert_eq!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, multi_vif_beacon_timer_tail) + core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, measurement_control_word_0) + core::mem::size_of::<SharedU32>(), 0x0400_1298); assert_eq!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, multi_vif_beacon_timer_tail) + core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, measurement_control_word_1) + core::mem::size_of::<SharedU32>(), 0x0400_129c); assert_eq!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, multi_vif_beacon_timer_tail) + core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, measurement_control_word_2) + core::mem::size_of::<SharedU32>(), 0x0400_12a0); assert_eq!(AMPDU_TELEMETRY_COUNTERS.get(), 0x0400_12a0); assert_eq!(core::mem::size_of::<InitializedVendorImage>(), 0x2078); assert_eq!(core::mem::align_of::<InitializedVendorImage>(), 4); assert_eq!(core::mem::size_of::<DtcmLayout>(), DTCM_STATE_SIZE); assert_eq!(core::mem::align_of::<DtcmLayout>(), 4); assert_eq!(core::mem::size_of::<SharedDtcmState>(), DTCM_STATE_SIZE); assert_eq!(core::mem::align_of::<SharedDtcmState>(), 4); } #[test] fn initialized_measurement_dwell_timer_address_is_exact() { assert_eq!([core::mem::offset_of!(TimerEntry, next), core::mem::offset_of!(TimerEntry, previous_link), core::mem::offset_of!(TimerEntry, deadline), core::mem::offset_of!(TimerEntry, callback), core::mem::offset_of!(TimerEntry, context)], [0x00, 0x04, 0x08, 0x0c, 0x10]); assert_eq!(core::mem::size_of::<TimerEntry>(), 0x14); assert_eq!(core::mem::align_of::<TimerEntry>(), 4); assert_eq!([core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, opaque_00), core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, interface_2_radio_latch), core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, timer), core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, opaque_24), core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, measurement_dwell_timer), core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, dtim_capture_latch), core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, opaque_51), core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, measurement_control_word_0), core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, measurement_control_word_1), core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, measurement_control_word_2)], [0x00, 0x0f, 0x10, 0x24, 0x3c, 0x50, 0x51, 0x54, 0x58, 0x5c]); assert_eq!(core::mem::size_of::<InitializedMultiVifBeaconTimerTail>(), 0x60); assert_eq!(core::mem::align_of::<InitializedMultiVifBeaconTimerTail>(), 4); let tail = DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, multi_vif_beacon_timer_tail); assert_eq!(tail, 0x0400_1240); assert_eq!(tail + core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, opaque_24) + core::mem::size_of::<OpaqueBytes<0x18>>(), 0x0400_127c); assert_eq!(MEASUREMENT_DWELL_TIMER.get(), 0x0400_127c); assert_eq!(MEASUREMENT_DWELL_TIMER.get() + core::mem::size_of::<TimerEntry>(), 0x0400_1290); assert_eq!(tail + core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, opaque_51) + core::mem::size_of::<OpaqueBytes<0x03>>(), 0x0400_1294); assert_eq!(tail + core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, measurement_control_word_0) + core::mem::size_of::<SharedU32>(), 0x0400_1298); assert_eq!(tail + core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, measurement_control_word_1) + core::mem::size_of::<SharedU32>(), 0x0400_129c); assert_eq!(tail + core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, measurement_control_word_2) + core::mem::size_of::<SharedU32>(), 0x0400_12a0); assert_eq!(AMPDU_TELEMETRY_COUNTERS.get(), 0x0400_12a0); assert_eq!(core::mem::size_of::<InitializedVendorImage>(), 0x2078); assert_eq!(core::mem::align_of::<InitializedVendorImage>(), 4); assert_eq!(core::mem::size_of::<DtcmLayout>(), DTCM_STATE_SIZE); assert_eq!(core::mem::align_of::<DtcmLayout>(), 4); assert_eq!(core::mem::size_of::<SharedDtcmState>(), DTCM_STATE_SIZE); assert_eq!(core::mem::align_of::<SharedDtcmState>(), 4); } #[test] fn initialized_dtim_capture_latch_address_is_exact() { let tail = DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, multi_vif_beacon_timer_tail); let latch = tail + core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, dtim_capture_latch); let opaque = tail + core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, opaque_51); assert_eq!(tail, 0x0400_1240); assert_eq!(MEASUREMENT_DWELL_TIMER.get() + core::mem::size_of::<TimerEntry>(), 0x0400_1290); assert_eq!([core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, dtim_capture_latch), core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, opaque_51), core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, measurement_control_word_0), core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, measurement_control_word_1), core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, measurement_control_word_2)], [0x50, 0x51, 0x54, 0x58, 0x5c]); assert_eq!(core::mem::size_of::<SharedU8>(), 0x01); assert_eq!(core::mem::align_of::<SharedU8>(), 1); assert_eq!(latch, 0x0400_1290); assert_eq!(latch + core::mem::size_of::<SharedU8>(), 0x0400_1291); assert_eq!(core::mem::size_of::<OpaqueBytes<0x03>>(), 0x03); assert_eq!(opaque, 0x0400_1291); assert_eq!(opaque + core::mem::size_of::<OpaqueBytes<0x03>>(), 0x0400_1294); assert_eq!([tail + core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, measurement_control_word_0), tail + core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, measurement_control_word_1), tail + core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, measurement_control_word_2)], [0x0400_1294, 0x0400_1298, 0x0400_129c]); assert_eq!(tail + core::mem::size_of::<InitializedMultiVifBeaconTimerTail>(), 0x0400_12a0); assert_eq!(AMPDU_TELEMETRY_COUNTERS.get(), 0x0400_12a0); assert_eq!(core::mem::size_of::<InitializedMultiVifBeaconTimerTail>(), 0x60); assert_eq!(core::mem::align_of::<InitializedMultiVifBeaconTimerTail>(), 4); assert_eq!(core::mem::size_of::<InitializedVendorImage>(), 0x2078); assert_eq!(core::mem::align_of::<InitializedVendorImage>(), 4); assert_eq!(core::mem::size_of::<DtcmLayout>(), DTCM_STATE_SIZE); assert_eq!(core::mem::align_of::<DtcmLayout>(), 4); assert_eq!(core::mem::size_of::<SharedDtcmState>(), DTCM_STATE_SIZE); assert_eq!(core::mem::align_of::<SharedDtcmState>(), 4); } #[test] fn initialized_measurement_control_reset_words_are_exact() { let tail = DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, multi_vif_beacon_timer_tail); assert_eq!(tail, 0x0400_1240); assert_eq!(MEASUREMENT_DWELL_TIMER.get() + core::mem::size_of::<TimerEntry>(), 0x0400_1290); assert_eq!(tail + core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, dtim_capture_latch), 0x0400_1290); assert_eq!(tail + core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, opaque_51) + core::mem::size_of::<OpaqueBytes<0x03>>(), 0x0400_1294); assert_eq!([tail + core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, measurement_control_word_0), tail + core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, measurement_control_word_1), tail + core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, measurement_control_word_2)], [0x0400_1294, 0x0400_1298, 0x0400_129c]); assert_eq!(core::mem::size_of::<SharedU32>(), 0x04); assert_eq!(tail + core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, measurement_control_word_2) + core::mem::size_of::<SharedU32>(), 0x0400_12a0); assert_eq!(AMPDU_TELEMETRY_COUNTERS.get(), 0x0400_12a0); assert_eq!(core::mem::size_of::<InitializedMultiVifBeaconTimerTail>(), 0x60); assert_eq!(core::mem::align_of::<InitializedMultiVifBeaconTimerTail>(), 4); assert_eq!(core::mem::size_of::<InitializedVendorImage>(), 0x2078); assert_eq!(core::mem::align_of::<InitializedVendorImage>(), 4); assert_eq!(core::mem::size_of::<DtcmLayout>(), DTCM_STATE_SIZE); assert_eq!(core::mem::align_of::<DtcmLayout>(), 4); assert_eq!(core::mem::size_of::<SharedDtcmState>(), DTCM_STATE_SIZE); assert_eq!(core::mem::align_of::<SharedDtcmState>(), 4); } #[test] fn initialized_mac_aggregate_slot_tables_are_exact() { let image = DTCM_STATE_BASE; let prefix = image + core::mem::offset_of!(InitializedVendorImage, pre_mac_phy_command_state); let tables = image + core::mem::offset_of!(InitializedVendorImage, mac_aggregate_slot_tables); assert_eq!(core::mem::size_of::<MacAggregateSlotTables>(), 0x200); assert_eq!(core::mem::align_of::<MacAggregateSlotTables>(), 4); assert_eq!(core::mem::offset_of!(MacAggregateSlotTables, queues), 0); assert_eq!(prefix, 0x0400_1b08); assert_eq!(core::mem::size_of::<OpaqueBytes<0x08>>(), 0x08); assert_eq!(tables, 0x0400_1b10); assert_eq!(tables + 7 * 0x40, 0x0400_1cd0); assert_eq!(tables + 7 * 0x40 + 15 * core::mem::size_of::<SharedU32>(), 0x0400_1d0c); assert_eq!(tables + core::mem::size_of::<MacAggregateSlotTables>(), 0x0400_1d10); assert_eq!(MAC_PHY_COMMAND_STATE.get(), 0x0400_1d10); assert_eq!(core::mem::size_of::<InitializedVendorImage>(), 0x2078); assert_eq!(core::mem::align_of::<InitializedVendorImage>(), 4); assert_eq!(core::mem::size_of::<DtcmLayout>(), DTCM_STATE_SIZE); assert_eq!(core::mem::align_of::<DtcmLayout>(), 4); assert_eq!(core::mem::size_of::<SharedDtcmState>(), DTCM_STATE_SIZE); assert_eq!(core::mem::align_of::<SharedDtcmState>(), 4); } #[test] fn initialized_interface_2_radio_latch_address_is_exact() { let tail = DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, multi_vif_beacon_timer_tail); let latch = tail + core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, interface_2_radio_latch); let timer = tail + core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, timer); assert_eq!(tail, 0x0400_1240); assert_eq!(tail + core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, opaque_00), 0x0400_1240); assert_eq!(tail + core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, opaque_00) + core::mem::size_of::<OpaqueBytes<0x0f>>(), 0x0400_124f); assert_eq!(latch, 0x0400_124f); assert_eq!(latch + core::mem::size_of::<SharedU8>(), 0x0400_1250); assert_eq!(timer, 0x0400_1250); assert_eq!(timer + core::mem::size_of::<TimerEntry>(), 0x0400_1264); assert_eq!([core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, opaque_00), core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, interface_2_radio_latch), core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, timer), core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, opaque_24), core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, measurement_dwell_timer), core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, dtim_capture_latch), core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, opaque_51), core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, measurement_control_word_0), core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, measurement_control_word_1), core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, measurement_control_word_2)], [0x00, 0x0f, 0x10, 0x24, 0x3c, 0x50, 0x51, 0x54, 0x58, 0x5c]); assert_eq!(core::mem::size_of::<SharedU8>(), 0x01); assert_eq!(core::mem::align_of::<SharedU8>(), 1); assert_eq!(core::mem::size_of::<InitializedMultiVifBeaconTimerTail>(), 0x60); assert_eq!(core::mem::align_of::<InitializedMultiVifBeaconTimerTail>(), 4); assert_eq!(core::mem::offset_of!(InitializedVendorImage, multi_vif_beacon_timer_tail), 0x1240); assert_eq!(core::mem::offset_of!(InitializedVendorImage, ampdu_counters), 0x12a0); assert_eq!(core::mem::size_of::<InitializedVendorImage>(), 0x2078); assert_eq!(core::mem::align_of::<InitializedVendorImage>(), 4); assert_eq!(core::mem::size_of::<DtcmLayout>(), DTCM_STATE_SIZE); assert_eq!(core::mem::align_of::<DtcmLayout>(), 4); assert_eq!(core::mem::size_of::<SharedDtcmState>(), DTCM_STATE_SIZE); assert_eq!(core::mem::align_of::<SharedDtcmState>(), 4); }

    #[test]
    fn initialized_completion_word_addresses_are_exact() {
        assert_eq!(VISIBLE_COMPLETION_WORDS.get(), 0x0400_0260);
        assert_eq!(visible_completion_word(0).unwrap().get(), 0x0400_0260);
        assert_eq!(visible_completion_word(9).unwrap().get(), 0x0400_0284);
        assert_eq!(visible_completion_word(9).unwrap().get() + 4, 0x0400_0288);
        assert!(visible_completion_word(10).is_none());
    }

    #[test]
    fn initialized_scheduler_tail_is_exact() { let image = DTCM_STATE_BASE; let tail = image + core::mem::offset_of!(InitializedVendorImage, initialized_tail); assert_eq!(core::mem::size_of::<SchedulerTail>(), 0x60); assert_eq!(core::mem::align_of::<SchedulerTail>(), 4); assert_eq!(core::mem::offset_of!(SchedulerTail, hardware_timer_guard), 0x10); assert_eq!(tail + core::mem::offset_of!(SchedulerTail, hardware_timer_guard), 0x0400_2028); assert_eq!(core::mem::offset_of!(SchedulerTail, rf_calibration_bytes), 0x14); assert_eq!(core::mem::size_of::<[SharedU8; 32]>(), 0x20); assert_eq!(tail + core::mem::offset_of!(SchedulerTail, rf_calibration_bytes), 0x0400_202c); assert_eq!(core::mem::offset_of!(SchedulerTail, error_event_counts), 0x38); assert_eq!(core::mem::size_of::<[SharedU32; 10]>(), 0x28); assert_eq!(tail + core::mem::offset_of!(SchedulerTail, error_event_counts), 0x0400_2050); assert_eq!(tail + core::mem::offset_of!(SchedulerTail, error_event_counts) + core::mem::size_of::<[SharedU32; 10]>(), 0x0400_2078); assert_eq!(tail + core::mem::size_of::<SchedulerTail>(), 0x0400_2078); assert_eq!(scheduler_timer_list_head().get(), 0x0400_2014); assert_eq!(core::mem::size_of::<InitializedVendorImage>(), 0x2078); assert_eq!(core::mem::align_of::<InitializedVendorImage>(), 4); assert_eq!(core::mem::size_of::<DtcmLayout>(), DTCM_STATE_SIZE); assert_eq!(core::mem::align_of::<DtcmLayout>(), 4); assert_eq!(core::mem::size_of::<SharedDtcmState>(), DTCM_STATE_SIZE); assert_eq!(core::mem::align_of::<SharedDtcmState>(), 4); } #[test]
    fn register_write_lists_after_iq_gain_indices_are_exact() { let image = DTCM_STATE_BASE; let cal = image + core::mem::offset_of!(InitializedVendorImage, phy_cal_substate_register_write_list); let dbg = image + core::mem::offset_of!(InitializedVendorImage, dbg_expand_register_write_list); assert_eq!(core::mem::size_of::<PhyCalSubstateRegisterWriteList>(), 0x48); assert_eq!(core::mem::align_of::<PhyCalSubstateRegisterWriteList>(), 4); assert_eq!(core::mem::offset_of!(PhyCalSubstateRegisterWriteList, writes), 0); assert_eq!(core::mem::size_of::<[RegisterWrite; 8]>(), 0x40); assert_eq!(core::mem::offset_of!(PhyCalSubstateRegisterWriteList, terminator_address), 0x40); assert_eq!(cal, 0x0400_0e48); assert_eq!(cal + core::mem::size_of::<PhyCalSubstateRegisterWriteList>(), 0x0400_0e90); assert_eq!(core::mem::size_of::<DbgExpandRegisterWriteList>(), 0x30); assert_eq!(core::mem::align_of::<DbgExpandRegisterWriteList>(), 4); assert_eq!(core::mem::offset_of!(DbgExpandRegisterWriteList, writes), 0); assert_eq!(core::mem::size_of::<[RegisterWrite; 5]>(), 0x28); assert_eq!(core::mem::offset_of!(DbgExpandRegisterWriteList, terminator_address), 0x28); assert_eq!(dbg, 0x0400_0e90); assert_eq!(dbg + core::mem::size_of::<DbgExpandRegisterWriteList>(), 0x0400_0ec0); assert_eq!(image + core::mem::offset_of!(InitializedVendorImage, register_write_lists_suffix), 0x0400_0ec0); assert_eq!(DURATION_QUANTUM_POINTERS.get(), 0x0400_10d4); assert_eq!(core::mem::size_of::<InitializedVendorImage>(), 0x2078); assert_eq!(core::mem::align_of::<InitializedVendorImage>(), 4); assert_eq!(core::mem::size_of::<DtcmLayout>(), DTCM_STATE_SIZE); assert_eq!(core::mem::align_of::<DtcmLayout>(), 4); assert_eq!(core::mem::size_of::<SharedDtcmState>(), DTCM_STATE_SIZE); assert_eq!(core::mem::align_of::<SharedDtcmState>(), 4); } #[test]
    fn initialized_rate_pair_table_is_exact() { let image = DTCM_STATE_BASE; let table = image + core::mem::offset_of!(InitializedVendorImage, rate_pair_table); assert_eq!(core::mem::size_of::<RatePairTable>(), 0x40); assert_eq!(core::mem::align_of::<RatePairTable>(), 4); assert_eq!(core::mem::offset_of!(RatePairTable, pairs), 0); assert_eq!(core::mem::size_of::<[[SharedU8; 2]; 31]>(), 0x3e); assert_eq!(core::mem::offset_of!(RatePairTable, opaque_3e), 0x3e); assert_eq!(core::mem::size_of::<OpaqueBytes<0x02>>(), 0x02); assert_eq!(table, 0x0400_01c0); assert_eq!(table + core::mem::size_of::<RatePairTable>(), 0x0400_0200); assert_eq!(INITIALIZED_RATE_POLICIES.get(), 0x0400_0200); assert_eq!(core::mem::size_of::<InitializedVendorImage>(), 0x2078); assert_eq!(core::mem::align_of::<InitializedVendorImage>(), 4); assert_eq!(core::mem::size_of::<DtcmLayout>(), DTCM_STATE_SIZE); assert_eq!(core::mem::align_of::<DtcmLayout>(), 4); assert_eq!(core::mem::size_of::<SharedDtcmState>(), DTCM_STATE_SIZE); assert_eq!(core::mem::align_of::<SharedDtcmState>(), 4); } #[test]
    fn initialized_tx_rate_table_addresses_are_exact() {
        assert_eq!(TX_DURATION_TIMING_TABLE.get(), 0x0400_0138);
        assert_eq!(tx_duration_timing(0).unwrap().get(), 0x0400_0138);
        assert_eq!(tx_duration_timing(9).unwrap().get(), 0x0400_014a);
        assert!(tx_duration_timing(10).is_none());
        assert_eq!(RATE_ENCODING_TABLE.get(), 0x0400_0194);
        assert_eq!(rate_encoding(21).unwrap().get(), 0x0400_01a9);
        assert!(rate_encoding(22).is_none());
        assert_eq!(RATE_ATTRIBUTE_TABLE.get(), 0x0400_01aa);
        assert_eq!(rate_attribute(21).unwrap().get(), 0x0400_01bf);
        assert!(rate_attribute(22).is_none());
    }

    #[test]
    fn initialized_tkip_sbox_table_addresses_are_exact() {
        assert_eq!(TKIP_SBOX_TABLES.get(), 0x0400_0310);
        assert_eq!(tkip_sbox_low_entry(0).unwrap().get(), 0x0400_0310);
        assert_eq!(tkip_sbox_low_entry(255).unwrap().get(), 0x0400_050e);
        assert!(tkip_sbox_low_entry(256).is_none());
        assert_eq!(tkip_sbox_high_entry(0).unwrap().get(), 0x0400_0510);
        assert_eq!(tkip_sbox_high_entry(255).unwrap().get(), 0x0400_070e);
        assert!(tkip_sbox_high_entry(256).is_none());
        assert_eq!(tkip_sbox_high_entry(255).unwrap().get() + core::mem::size_of::<SharedU16>(), 0x0400_0710);
        assert_eq!(TKIP_SBOX_TABLES.get() + core::mem::size_of::<TkipSboxTables>(), DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, command_dispatch));
    }

    #[test]
    fn initialized_aes_transfer_class_addresses_are_exact() {
        assert_eq!(AES_TRANSFER_CLASSES.get(), 0x0400_0804);
        assert_eq!(aes_transfer_class(0).unwrap().get(), 0x0400_0804);
        assert_eq!(aes_transfer_class(6).unwrap().get(), 0x0400_081c);
        assert_eq!(aes_transfer_class(7).unwrap().get(), 0x0400_0820);
        assert_eq!(aes_transfer_class(10).unwrap().get(), 0x0400_082c);
        assert!(aes_transfer_class(11).is_none());
        assert_eq!(aes_transfer_class(10).unwrap().get() + core::mem::size_of::<SharedU32>(), 0x0400_0830);
        assert_eq!(AES_TRANSFER_CLASSES.get() + core::mem::size_of::<AesTransferClassTable>(), DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, aes_mode1_microcode));
    }

    #[test]
    fn initialized_phy_gain_register_write_list_addresses_are_exact() {
        assert_eq!(PHY_GAIN_REGISTER_WRITE_LISTS.get(), 0x0400_0b60);
        assert_eq!(phy_gain_register_write_list(0).unwrap().get(), 0x0400_0b60);
        assert_eq!(phy_gain_register_write_entry(0, 0).unwrap().get(), 0x0400_0b60);
        assert_eq!(phy_gain_register_write_entry(0, 9).unwrap().get(), 0x0400_0ba8);
        assert_eq!(phy_gain_register_write_list(1).unwrap().get(), 0x0400_0bb8);
        assert_eq!(phy_gain_register_write_entry(1, 0).unwrap().get(), 0x0400_0bb8);
        assert_eq!(phy_gain_register_write_entry(1, 9).unwrap().get(), 0x0400_0c00);
        assert!(phy_gain_register_write_list(2).is_none());
        assert!(phy_gain_register_write_entry(2, 0).is_none());
        assert!(phy_gain_register_write_entry(0, 10).is_none());
        assert_eq!(PHY_GAIN_REGISTER_WRITE_LISTS.get() + 2 * core::mem::size_of::<RegisterWriteList>(), PHY_INIT_REGISTER_WRITE_LISTS.get());
    }

    #[test]
    fn initialized_phy_init_register_write_list_addresses_are_exact() {
        assert_eq!(PHY_INIT_REGISTER_WRITE_LISTS.get(), 0x0400_0c10);
        assert_eq!([phy_init_register_write_list(0).unwrap().get(), phy_init_register_write_list(1).unwrap().get(), phy_init_register_write_list(2).unwrap().get()], [0x0400_0c10, 0x0400_0c20, 0x0400_0c48]);
        assert_eq!(phy_init_register_write_entry(0, 0).unwrap().get(), 0x0400_0c10);
        assert_eq!([phy_init_register_write_entry(1, 0).unwrap().get(), phy_init_register_write_entry(1, 1).unwrap().get(), phy_init_register_write_entry(1, 2).unwrap().get(), phy_init_register_write_entry(1, 3).unwrap().get()], [0x0400_0c20, 0x0400_0c28, 0x0400_0c30, 0x0400_0c38]);
        assert_eq!([phy_init_register_write_entry(2, 0).unwrap().get(), phy_init_register_write_entry(2, 1).unwrap().get()], [0x0400_0c48, 0x0400_0c50]);
        assert!(phy_init_register_write_list(3).is_none()); assert!(phy_init_register_write_entry(0, 1).is_none()); assert!(phy_init_register_write_entry(1, 4).is_none()); assert!(phy_init_register_write_entry(2, 2).is_none());
        assert_eq!(phy_init_register_write_list(0).unwrap().get() + core::mem::size_of::<PhyInitRegisterWriteList0>(), 0x0400_0c20); assert_eq!(phy_init_register_write_list(1).unwrap().get() + core::mem::size_of::<PhyInitRegisterWriteList1>(), 0x0400_0c48);
        assert_eq!(phy_init_register_write_list(2).unwrap().get() + core::mem::size_of::<PhyInitRegisterWriteList2>(), 0x0400_0c60); assert_eq!(DURATION_QUANTUM_POINTERS.get(), 0x0400_10d4);
    }

    #[test]
    fn initialized_phy_gain_source_record_addresses_are_exact() { assert_eq!(INITIALIZED_PHY_GAIN_SOURCE_RECORDS.get(), 0x0400_0ca6); assert_eq!(initialized_phy_gain_source_physical_record(0).unwrap().get(), 0x0400_0ca6); assert_eq!(initialized_phy_gain_source_physical_record(21).unwrap().get(), 0x0400_0d24); assert_eq!(initialized_phy_gain_source_physical_record(42).unwrap().get(), 0x0400_0da2); assert_eq!(initialized_phy_gain_source_physical_record(42).unwrap().get() + core::mem::size_of::<InitializedPhyGainSourceRecord>(), 0x0400_0da8); assert!(initialized_phy_gain_source_physical_record(43).is_none()); assert_eq!(initialized_phy_gain_source_view_record(0, 0).unwrap().get(), 0x0400_0ca6); assert_eq!(initialized_phy_gain_source_view_record(0, 21).unwrap().get(), 0x0400_0d24); assert_eq!(initialized_phy_gain_source_view_record(1, 0).unwrap().get(), 0x0400_0d24); assert_eq!(initialized_phy_gain_source_view_record(1, 21).unwrap().get(), 0x0400_0da2); assert!(initialized_phy_gain_source_view_record(2, 0).is_none()); assert!(initialized_phy_gain_source_view_record(0, 22).is_none()); assert_eq!(core::mem::size_of::<InitializedPhyGainSourceRecord>(), 0x06); assert_eq!(core::mem::align_of::<InitializedPhyGainSourceRecord>(), 2); assert_eq!(core::mem::size_of::<[InitializedPhyGainSourceRecord; 43]>(), 0x102); assert_eq!(core::mem::align_of::<[InitializedPhyGainSourceRecord; 43]>(), 2); assert_eq!([core::mem::offset_of!(InitializedPhyGainSourceRecord, selector), core::mem::offset_of!(InitializedPhyGainSourceRecord, opaque_01), core::mem::offset_of!(InitializedPhyGainSourceRecord, lower), core::mem::offset_of!(InitializedPhyGainSourceRecord, upper)], [0, 1, 2, 4]); assert_eq!(core::mem::size_of::<InitializedVendorImage>(), 0x2078); assert_eq!(DURATION_QUANTUM_POINTERS.get(), 0x0400_10d4); }
    #[test]
    fn initialized_rf_mode_halfword_table_is_exact() { let image = DTCM_STATE_BASE; let table = image + core::mem::offset_of!(InitializedVendorImage, rf_mode_halfword_table); let opaque = image + core::mem::offset_of!(InitializedVendorImage, post_initialized_phy_gain_source_records); assert_eq!(opaque, 0x0400_0da8); assert_eq!(core::mem::size_of::<OpaqueBytes<0x28>>(), 0x28); assert_eq!(table, 0x0400_0dd0); assert_eq!(core::mem::align_of::<RfModeHalfwordTable>(), 2); assert_eq!(core::mem::offset_of!(RfModeHalfwordTable, entries), 0); assert_eq!(core::mem::size_of::<RfModeHalfwordTable>(), 0x48); assert_eq!([RF_MODE_HALFWORD_TABLE.get() + 0 * 2, RF_MODE_HALFWORD_TABLE.get() + 12 * 2, RF_MODE_HALFWORD_TABLE.get() + 35 * 2], [0x0400_0dd0, 0x0400_0de8, 0x0400_0e16]); assert_eq!(table + core::mem::size_of::<RfModeHalfwordTable>(), 0x0400_0e18); assert_eq!(INITIALIZED_IQ_CALIBRATION_GAIN_INDICES.get(), 0x0400_0e18); assert_eq!(core::mem::size_of::<InitializedVendorImage>(), 0x2078); assert_eq!(core::mem::align_of::<InitializedVendorImage>(), 4); assert_eq!(core::mem::size_of::<DtcmLayout>(), DTCM_STATE_SIZE); assert_eq!(core::mem::align_of::<DtcmLayout>(), 4); assert_eq!(core::mem::size_of::<SharedDtcmState>(), DTCM_STATE_SIZE); assert_eq!(core::mem::align_of::<SharedDtcmState>(), 4); } #[test]
    fn initialized_iq_calibration_gain_index_addresses_are_exact() { assert_eq!(core::mem::size_of::<InitializedIqCalibrationGainIndices>(), 0x30); assert_eq!(core::mem::align_of::<InitializedIqCalibrationGainIndices>(), 4); assert_eq!(core::mem::offset_of!(InitializedIqCalibrationGainIndices, entries), 0); assert_eq!(core::mem::offset_of!(InitializedVendorImage, post_initialized_phy_gain_source_records), 0x0da8); assert_eq!(core::mem::size_of::<OpaqueBytes<0x28>>(), 0x28); assert_eq!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, rf_mode_halfword_table), 0x0400_0dd0); assert_eq!(core::mem::size_of::<RfModeHalfwordTable>(), 0x48); assert_eq!(RF_MODE_HALFWORD_TABLE.get(), 0x0400_0dd0); assert_eq!(RF_MODE_HALFWORD_TABLE.get() + core::mem::size_of::<RfModeHalfwordTable>(), 0x0400_0e18); assert_eq!(INITIALIZED_IQ_CALIBRATION_GAIN_INDICES.get(), 0x0400_0e18); assert_eq!(initialized_iq_calibration_gain_index(0).unwrap().get(), 0x0400_0e18); assert_eq!(initialized_iq_calibration_gain_index(1).unwrap().get(), 0x0400_0e1c); assert_eq!(initialized_iq_calibration_gain_index(11).unwrap().get(), 0x0400_0e44); assert!(initialized_iq_calibration_gain_index(12).is_none()); assert_eq!(INITIALIZED_IQ_CALIBRATION_GAIN_INDICES.get() + core::mem::size_of::<InitializedIqCalibrationGainIndices>(), 0x0400_0e48); assert_eq!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, phy_cal_substate_register_write_list), 0x0400_0e48); assert_eq!(core::mem::size_of::<PhyCalSubstateRegisterWriteList>(), 0x48); assert_eq!(core::mem::size_of::<OpaqueBytes<0x214>>(), 0x214); assert_eq!(DURATION_QUANTUM_POINTERS.get(), 0x0400_10d4); assert_eq!(core::mem::size_of::<InitializedVendorImage>(), 0x2078); }
    #[test]
    fn measurement_workspace_addresses_are_exact() { assert_eq!([MEASUREMENT_WORKSPACE.get(), measurement_dwell_bound().get(), measurement_type().get(), measurement_completion_status().get(), measurement_dispatch_state().get(), measurement_dispatch_argument().get(), measurement_start_timestamp_word(0).unwrap().get(), measurement_start_timestamp_word(1).unwrap().get(), measurement_elapsed_timestamp_word(0).unwrap().get(), measurement_elapsed_timestamp_word(1).unwrap().get(), measurement_scan_request().get(), measurement_scan_request_mode().get()], [0x0400_10f8, 0x0400_10f8, 0x0400_10fd, 0x0400_1100, 0x0400_1109, 0x0400_110a, 0x0400_1110, 0x0400_1114, 0x0400_1118, 0x0400_111c, 0x0400_1120, 0x0400_1121]); assert!(measurement_start_timestamp_word(2).is_none()); assert!(measurement_elapsed_timestamp_word(2).is_none()); assert_eq!(MEASUREMENT_WORKSPACE.get() + core::mem::size_of::<MeasurementWorkspace>(), 0x0400_1160); }
    #[test]
    fn initialized_debug_command_descriptor_addresses_are_exact() { assert_eq!(TX_AGGREGATE_EXPIRATION_DELTA.get() + core::mem::size_of::<TxAggregateExpirationDelta>(), 0x0400_1164); assert_eq!(DEBUG_COMMAND_DESCRIPTORS.get(), 0x0400_1164); assert_eq!(core::mem::size_of::<DebugCommandDescriptor>(), 0x0c); assert_eq!(core::mem::align_of::<DebugCommandDescriptor>(), 4); assert_eq!([core::mem::offset_of!(DebugCommandDescriptor, command_name), core::mem::offset_of!(DebugCommandDescriptor, help_text), core::mem::offset_of!(DebugCommandDescriptor, handler)], [0x00, 0x04, 0x08]); assert_eq!(core::mem::size_of::<InitializedDebugCommandDescriptors>(), 0x48); assert_eq!(core::mem::align_of::<InitializedDebugCommandDescriptors>(), 4); assert_eq!(core::mem::offset_of!(InitializedDebugCommandDescriptors, records), 0); assert_eq!([debug_command_descriptor(0).unwrap().get(), debug_command_descriptor(1).unwrap().get(), debug_command_descriptor(2).unwrap().get(), debug_command_descriptor(3).unwrap().get(), debug_command_descriptor(4).unwrap().get(), debug_command_descriptor(5).unwrap().get()], [0x0400_1164, 0x0400_1170, 0x0400_117c, 0x0400_1188, 0x0400_1194, 0x0400_11a0]); assert_eq!([debug_command_name(0).unwrap().get(), debug_command_help(0).unwrap().get(), debug_command_handler(0).unwrap().get(), debug_command_name(1).unwrap().get(), debug_command_help(1).unwrap().get(), debug_command_handler(1).unwrap().get(), debug_command_name(2).unwrap().get(), debug_command_help(2).unwrap().get(), debug_command_handler(2).unwrap().get()], [0x0400_1164, 0x0400_1168, 0x0400_116c, 0x0400_1170, 0x0400_1174, 0x0400_1178, 0x0400_117c, 0x0400_1180, 0x0400_1184]); assert_eq!([debug_command_name(3).unwrap().get(), debug_command_help(3).unwrap().get(), debug_command_handler(3).unwrap().get(), debug_command_name(4).unwrap().get(), debug_command_help(4).unwrap().get(), debug_command_handler(4).unwrap().get(), debug_command_name(5).unwrap().get(), debug_command_help(5).unwrap().get(), debug_command_handler(5).unwrap().get()], [0x0400_1188, 0x0400_118c, 0x0400_1190, 0x0400_1194, 0x0400_1198, 0x0400_119c, 0x0400_11a0, 0x0400_11a4, 0x0400_11a8]); assert!(debug_command_descriptor(6).is_none()); assert!(debug_command_name(6).is_none()); assert!(debug_command_help(6).is_none()); assert!(debug_command_handler(6).is_none()); assert_eq!(DEBUG_COMMAND_DESCRIPTORS.get() + core::mem::size_of::<InitializedDebugCommandDescriptors>(), 0x0400_11ac); assert_eq!(DEBUG_COMMAND_DESCRIPTORS.get() + core::mem::size_of::<InitializedDebugCommandDescriptors>(), INITIALIZED_HIF_CONTROL.get()); assert_eq!(core::mem::size_of::<InitializedVendorImage>(), 0x2078); assert_eq!(core::mem::size_of::<DtcmLayout>(), DTCM_STATE_SIZE); assert_eq!(core::mem::align_of::<DtcmLayout>(), 4); assert_eq!(core::mem::size_of::<SharedDtcmState>(), DTCM_STATE_SIZE); assert_eq!(core::mem::align_of::<SharedDtcmState>(), 4); }
    #[test]
    fn initialized_duration_quantum_pointer_addresses_are_exact() {
        assert_eq!(DURATION_QUANTUM_POINTERS.get(), 0x0400_10d4);
        assert_eq!(duration_quantum_pointer(0).unwrap().get(), 0x0400_10d4);
        assert_eq!(duration_quantum_pointer(3).unwrap().get(), 0x0400_10e0);
        assert_eq!(duration_quantum_pointer(3).unwrap().get() + 4, 0x0400_10e4);
        assert!(duration_quantum_pointer(4).is_none());
    }

    #[test]
    fn initialized_queue_pipe_mapping_addresses_are_exact() {
        assert_eq!(QUEUE_PIPE_MAPPINGS.get(), 0x0400_02d8);
        assert_eq!(pipe_order_byte(0).unwrap().get(), 0x0400_02d8);
        assert_eq!(pipe_order_byte(3).unwrap().get(), 0x0400_02db);
        assert_eq!(QUEUE_TO_ACCESS_CATEGORY.get(), 0x0400_02dc);
        assert_eq!(queue_to_access_category(3).unwrap().get(), 0x0400_02df);
        assert_eq!(ACCESS_CATEGORY_TO_QUEUE.get(), 0x0400_02e0);
        assert_eq!(access_category_to_queue(3).unwrap().get(), 0x0400_02e3);
        assert!(pipe_order_byte(4).is_none());
        assert!(queue_to_access_category(4).is_none());
        assert!(access_category_to_queue(4).is_none());
    }

    #[test]
    fn initialized_control_word_addresses_are_exact() {
        assert_eq!(INITIALIZED_CONTROL_WORDS.get(), 0x0400_1420);
        assert_eq!(initialized_beacon_state().get(), 0x0400_1420);
        assert_eq!(initialized_rx_indication_state().get(), 0x0400_1424);
        assert_eq!(initialized_tsf_resync_state().get(), 0x0400_1428);
        assert_eq!(initialized_random_lfsr().get(), 0x0400_142c);
        assert_eq!(initialized_tsf_accumulator_low().get(), 0x0400_1430);
        assert_eq!(initialized_timer_counter().get(), 0x0400_143c);
        assert_eq!(initialized_timer_counter().get() + 4, 0x0400_1440);
    }

    #[test]
    fn initialized_retry_path_counter_address_is_exact() { let image = DTCM_STATE_BASE; let record = image + core::mem::offset_of!(InitializedVendorImage, retry_path_counter); let field = record + core::mem::offset_of!(RetryPathCounter, count); assert_eq!(core::mem::size_of::<SharedU32>(), 0x04); assert_eq!(core::mem::align_of::<SharedU32>(), 4); assert_eq!(core::mem::size_of::<RetryPathCounter>(), 0x04); assert_eq!(core::mem::align_of::<RetryPathCounter>(), 4); assert_eq!(core::mem::offset_of!(RetryPathCounter, count), 0x00); assert_eq!(AMPDU_TELEMETRY_COUNTERS.get() + core::mem::size_of::<AmpduTelemetryCounters>(), 0x0400_12c8); assert_eq!(record, 0x0400_12c8); assert_eq!(field, 0x0400_12c8); assert_eq!(record + core::mem::size_of::<RetryPathCounter>(), 0x0400_12cc); assert_eq!(image + core::mem::offset_of!(InitializedVendorImage, per_tid_telemetry_bank), 0x0400_12cc); assert_eq!(core::mem::size_of::<InitializedVendorImage>(), 0x2078); assert_eq!(core::mem::align_of::<InitializedVendorImage>(), 4); assert_eq!(core::mem::size_of::<DtcmLayout>(), DTCM_STATE_SIZE); assert_eq!(core::mem::align_of::<DtcmLayout>(), 4); assert_eq!(core::mem::size_of::<SharedDtcmState>(), DTCM_STATE_SIZE); assert_eq!(core::mem::align_of::<SharedDtcmState>(), 4); } #[test] fn initialized_per_tid_telemetry_bank_addresses_are_exact() { let image = DTCM_STATE_BASE; let bank = image + core::mem::offset_of!(InitializedVendorImage, per_tid_telemetry_bank); let roots = [bank + core::mem::offset_of!(PerTidTelemetryBank, table_00), bank + core::mem::offset_of!(PerTidTelemetryBank, table_01), bank + core::mem::offset_of!(PerTidTelemetryBank, table_02), bank + core::mem::offset_of!(PerTidTelemetryBank, table_03), bank + core::mem::offset_of!(PerTidTelemetryBank, table_04), bank + core::mem::offset_of!(PerTidTelemetryBank, table_05), bank + core::mem::offset_of!(PerTidTelemetryBank, table_06), bank + core::mem::offset_of!(PerTidTelemetryBank, table_07), bank + core::mem::offset_of!(PerTidTelemetryBank, table_08), bank + core::mem::offset_of!(PerTidTelemetryBank, table_09)]; assert_eq!(core::mem::size_of::<SharedU32>(), 4); assert_eq!(core::mem::align_of::<SharedU32>(), 4); assert_eq!(core::mem::size_of::<PerTidTelemetryBank>(), 0x140); assert_eq!(core::mem::align_of::<PerTidTelemetryBank>(), 4); assert_eq!([core::mem::offset_of!(PerTidTelemetryBank, table_00), core::mem::offset_of!(PerTidTelemetryBank, table_01), core::mem::offset_of!(PerTidTelemetryBank, table_02), core::mem::offset_of!(PerTidTelemetryBank, table_03), core::mem::offset_of!(PerTidTelemetryBank, table_04), core::mem::offset_of!(PerTidTelemetryBank, table_05), core::mem::offset_of!(PerTidTelemetryBank, table_06), core::mem::offset_of!(PerTidTelemetryBank, table_07), core::mem::offset_of!(PerTidTelemetryBank, table_08), core::mem::offset_of!(PerTidTelemetryBank, table_09)], [0x000, 0x020, 0x040, 0x060, 0x080, 0x0a0, 0x0c0, 0x0e0, 0x100, 0x120]); assert_eq!(core::mem::size_of::<[SharedU32; 8]>(), 0x20); assert_eq!(core::mem::size_of::<[SharedU32; 8]>() / core::mem::size_of::<SharedU32>(), 8); assert_eq!(core::mem::size_of::<SharedU32>(), 4); assert_eq!(roots, [0x0400_12cc, 0x0400_12ec, 0x0400_130c, 0x0400_132c, 0x0400_134c, 0x0400_136c, 0x0400_138c, 0x0400_13ac, 0x0400_13cc, 0x0400_13ec]); for pair in roots.windows(2) { assert_eq!(pair[0] + core::mem::size_of::<[SharedU32; 8]>(), pair[1]); } assert_eq!(roots[9] + core::mem::size_of::<[SharedU32; 8]>(), 0x0400_140c); assert_eq!(image + core::mem::offset_of!(InitializedVendorImage, retry_path_counter), 0x0400_12c8); assert_eq!(core::mem::size_of::<RetryPathCounter>(), 4); assert_eq!(image + core::mem::offset_of!(InitializedVendorImage, retry_path_counter) + core::mem::size_of::<RetryPathCounter>(), 0x0400_12cc); assert_eq!(bank, 0x0400_12cc); assert_eq!(AMPDU_TELEMETRY_COUNTERS.get() + core::mem::size_of::<AmpduTelemetryCounters>(), 0x0400_12c8); assert_eq!(AMPDU_COMPLETION_CONTROL.get(), 0x0400_140c); assert_eq!(core::mem::size_of::<InitializedVendorImage>(), 0x2078); assert_eq!(core::mem::align_of::<InitializedVendorImage>(), 4); assert_eq!(core::mem::size_of::<DtcmLayout>(), DTCM_STATE_SIZE); assert_eq!(core::mem::align_of::<DtcmLayout>(), 4); assert_eq!(core::mem::size_of::<SharedDtcmState>(), DTCM_STATE_SIZE); assert_eq!(core::mem::align_of::<SharedDtcmState>(), 4); }

    #[test]
    fn initialized_ampdu_completion_control_address_is_exact() {
        assert_eq!(AMPDU_COMPLETION_CONTROL.get(), 0x0400_140c);
        assert_eq!(AMPDU_COMPLETION_CONTROL.get() + 4, 0x0400_1410);
    }

    #[test]
    fn initialized_tx_confirm_aggregation_state_addresses_are_exact() { let image = DTCM_STATE_BASE; let record = image + core::mem::offset_of!(InitializedVendorImage, tx_confirm_aggregation_state); let fields = [record + core::mem::offset_of!(TxConfirmAggregationState, state), record + core::mem::offset_of!(TxConfirmAggregationState, pending_message_raw), record + core::mem::offset_of!(TxConfirmAggregationState, append_cursor_raw)]; assert_eq!(core::mem::size_of::<SharedU32>(), 4); assert_eq!(core::mem::align_of::<SharedU32>(), 4); assert_eq!(core::mem::size_of::<TxConfirmAggregationState>(), 0x0c); assert_eq!(core::mem::align_of::<TxConfirmAggregationState>(), 4); assert_eq!([core::mem::offset_of!(TxConfirmAggregationState, state), core::mem::offset_of!(TxConfirmAggregationState, pending_message_raw), core::mem::offset_of!(TxConfirmAggregationState, append_cursor_raw)], [0x00, 0x04, 0x08]); assert_eq!(fields, [0x0400_1410, 0x0400_1414, 0x0400_1418]); for pair in fields.windows(2) { assert_eq!(pair[0] + core::mem::size_of::<SharedU32>(), pair[1]); } assert_eq!(record, 0x0400_1410); assert_eq!(record + core::mem::size_of::<TxConfirmAggregationState>(), 0x0400_141c); assert_eq!(AMPDU_COMPLETION_CONTROL.get(), 0x0400_140c); assert_eq!(AMPDU_COMPLETION_CONTROL.get() + core::mem::size_of::<AmpduCompletionControl>(), 0x0400_1410); assert_eq!(image + core::mem::offset_of!(InitializedVendorImage, configuration_apply_flags), 0x0400_141c); assert_eq!(image + core::mem::offset_of!(InitializedVendorImage, control_words), 0x0400_1420); assert_eq!(core::mem::size_of::<InitializedVendorImage>(), 0x2078); assert_eq!(core::mem::align_of::<InitializedVendorImage>(), 4); assert_eq!(core::mem::size_of::<DtcmLayout>(), DTCM_STATE_SIZE); assert_eq!(core::mem::align_of::<DtcmLayout>(), 4); assert_eq!(core::mem::size_of::<SharedDtcmState>(), DTCM_STATE_SIZE); assert_eq!(core::mem::align_of::<SharedDtcmState>(), 4); }

    #[test]
    fn initialized_configuration_apply_flags_address_is_exact() { let image = DTCM_STATE_BASE; let record = image + core::mem::offset_of!(InitializedVendorImage, configuration_apply_flags); let field = record + core::mem::offset_of!(ConfigurationApplyFlags, flags); assert_eq!(core::mem::size_of::<SharedU32>(), 4); assert_eq!(core::mem::align_of::<SharedU32>(), 4); assert_eq!(core::mem::size_of::<ConfigurationApplyFlags>(), 0x04); assert_eq!(core::mem::align_of::<ConfigurationApplyFlags>(), 4); assert_eq!(core::mem::offset_of!(ConfigurationApplyFlags, flags), 0x00); assert_eq!(record, 0x0400_141c); assert_eq!(field, 0x0400_141c); assert_eq!(image + core::mem::offset_of!(InitializedVendorImage, tx_confirm_aggregation_state) + core::mem::size_of::<TxConfirmAggregationState>(), 0x0400_141c); assert_eq!(record + core::mem::size_of::<ConfigurationApplyFlags>(), 0x0400_1420); assert_eq!(image + core::mem::offset_of!(InitializedVendorImage, control_words), 0x0400_1420); assert_eq!(core::mem::size_of::<InitializedVendorImage>(), 0x2078); assert_eq!(core::mem::align_of::<InitializedVendorImage>(), 4); assert_eq!(core::mem::size_of::<DtcmLayout>(), DTCM_STATE_SIZE); assert_eq!(core::mem::align_of::<DtcmLayout>(), 4); assert_eq!(core::mem::size_of::<SharedDtcmState>(), DTCM_STATE_SIZE); assert_eq!(core::mem::align_of::<SharedDtcmState>(), 4); } #[test] fn initialized_debug_platform_local_tail_addresses_are_exact() { let image = DTCM_STATE_BASE; let opaque = image + core::mem::offset_of!(InitializedVendorImage, pre_phy_channel_threshold_descriptors); let tail = image + core::mem::offset_of!(InitializedVendorImage, debug_platform_local_tail); let fields = [tail + core::mem::offset_of!(DebugPlatformLocalTail, platform_local_word_2c), tail + core::mem::offset_of!(DebugPlatformLocalTail, platform_local_word_30)]; let phy = image + core::mem::offset_of!(InitializedVendorImage, phy_channel_threshold_descriptors); assert_eq!(core::mem::size_of::<SharedU32>(), 0x04); assert_eq!(core::mem::align_of::<SharedU32>(), 4); assert_eq!(core::mem::size_of::<DebugPlatformLocalTail>(), 0x08); assert_eq!(core::mem::align_of::<DebugPlatformLocalTail>(), 4); assert_eq!([core::mem::offset_of!(DebugPlatformLocalTail, platform_local_word_2c), core::mem::offset_of!(DebugPlatformLocalTail, platform_local_word_30)], [0x00, 0x04]); assert_eq!(fields, [0x0400_1454, 0x0400_1458]); assert_eq!(fields[0] + core::mem::size_of::<SharedU32>(), fields[1]); assert_eq!(opaque, 0x0400_1440); assert_eq!(core::mem::size_of::<OpaqueBytes<0x14>>(), 0x14); assert_eq!(opaque + core::mem::size_of::<OpaqueBytes<0x14>>(), 0x0400_1454); assert_eq!(tail + core::mem::size_of::<DebugPlatformLocalTail>(), 0x0400_145c); assert_eq!(phy, 0x0400_145c); assert_eq!(core::mem::size_of::<InitializedVendorImage>(), 0x2078); assert_eq!(core::mem::align_of::<InitializedVendorImage>(), 4); assert_eq!(core::mem::size_of::<DtcmLayout>(), DTCM_STATE_SIZE); assert_eq!(core::mem::align_of::<DtcmLayout>(), 4); assert_eq!(core::mem::size_of::<SharedDtcmState>(), DTCM_STATE_SIZE); assert_eq!(core::mem::align_of::<SharedDtcmState>(), 4); }

    #[test]
    fn initialized_ampdu_telemetry_addresses_are_exact() {
        assert_eq!(AMPDU_TELEMETRY_COUNTERS.get(), 0x0400_12a0);
        assert_eq!(ampdu_tx_error_frames().get(), 0x0400_12a0);
        assert_eq!(ampdu_tx_counted_frames().get(), 0x0400_12a4);
        assert_eq!(ampdu_tx_duration_low().get(), 0x0400_12a8);
        assert_eq!(ampdu_tx_duration_high().get(), 0x0400_12ac);
        assert_eq!(ampdu_rx_management(0).unwrap().get(), 0x0400_12b0);
        assert_eq!(ampdu_rx_management(3).unwrap().get(), 0x0400_12bc);
        assert!(ampdu_rx_management(4).is_none());
        assert_eq!(ampdu_tx_retry_count().get(), 0x0400_12c4);
    }

    #[test]
    fn initialized_hif_control_addresses_are_exact() {
        assert_eq!(INITIALIZED_HIF_CONTROL.get(), 0x0400_11ac);
        assert_eq!(initialized_hif_queued_depth().get(), 0x0400_11ac);
        assert_eq!(initialized_hif_pending_count().get(), 0x0400_11b0);
        assert_eq!(initialized_hif_coalesce_enabled().get(), 0x0400_11b4);
        assert_eq!(initialized_hif_pending_threshold().get(), 0x0400_11b5);
        assert_eq!(initialized_hif_ring_depth_threshold().get(), 0x0400_11b6);
        assert_eq!(initialized_hif_count_threshold().get(), 0x0400_11b7);
        assert_eq!(initialized_hif_coalesce_delay().get(), 0x0400_11b8);
        assert_eq!(initialized_hif_coalesce_delay().get() + 4, 0x0400_11bc);
    }

    #[test]
    fn scheduler_event_and_timer_roots_are_exact() {
        assert_eq!(scheduler_exclusion_mask().get(), 0x0400_1fcc);
        assert_eq!(scheduler_secondary_exclusion().get(), 0x0400_1fd0);
        assert_eq!(scheduler_pending_events().get(), 0x0400_1fd4);
        assert_eq!(scheduler_runtime_flags().get(), 0x0400_1fd8);
        assert_eq!(scheduler_startup_mode().get(), 0x0400_1fe6);
        assert_eq!(scheduler_analog_enabled().get(), 0x0400_1ff0);
        assert_eq!(scheduler_remap_primary().get(), 0x0400_1ff4);
        assert_eq!(scheduler_remap_secondary().get(), 0x0400_1ffc);
        assert_eq!(scheduler_analog_word(0).unwrap().get(), 0x0400_2000);
        assert_eq!(scheduler_analog_word(2).unwrap().get(), 0x0400_2008);
        assert!(scheduler_analog_word(3).is_none());
        assert_eq!(scheduler_timer_list_head().get(), 0x0400_2014);
        assert_eq!(scheduler_timer_list_head().get() + 4, 0x0400_2018);
    }

    #[test]
    fn peer_pipe_table_and_pre_command_tail_are_exact() {
        assert_eq!(peer_pipe(0).unwrap().get(), 0x0400_8544);
        assert_eq!(peer_pipe(3).unwrap().get(), 0x0400_855c);
        assert_eq!(peer_pipe(4).unwrap().get(), 0x0400_8564);
        assert_eq!(peer_pipe(7).unwrap().get(), 0x0400_857c);
        assert!(peer_pipe(8).is_none());
        assert_eq!(management_counter(0).unwrap().get(), 0x0400_8584);
        assert_eq!(management_counter(3).unwrap().get(), 0x0400_858a);
        assert!(management_counter(4).is_none());
        assert_eq!(pre_command_scan_channel().get(), 0x0400_858c);
        assert_eq!(pre_command_pending_root().get(), 0x0400_8590);
        assert_eq!(pre_command_pending_root().get() + 4, 0x0400_8594);
    }

    #[test]
    fn encryption_roots_and_duplicate_cache_overlap_are_exact() {
        assert_eq!(encryption_free_head().get(), 0x0400_861c);
        assert_eq!(encryption_generation().get(), 0x0400_8620);
        assert_eq!(duplicate_cache_entry(0).unwrap().get(), 0x0400_8624);
        assert_eq!(duplicate_cache_entry(30).unwrap().get(), 0x0400_878c);
        assert_eq!(duplicate_cache_entry(31).unwrap().get(), 0x0400_8798);
        assert_eq!(duplicate_cache_entry(31).unwrap().get() + 0x0c, 0x0400_87a4);
        assert!(duplicate_cache_entry(32).is_none());
    }

    #[test]
    fn command_upload_and_channel_scan_overlay_addresses_are_exact() {
        assert_eq!(command_upload_start().get(), 0x0400_8594);
        assert_eq!(channel_switch_active().get(), 0x0400_85fc);
        assert_eq!(saved_register_context().get(), 0x0400_8606);
        assert_eq!(channel_switch_channel().get(), 0x0400_8602);
        assert_eq!(vendor_scan_state().get(), 0x0400_860c);
        assert_eq!(command_overlay_tail().get(), 0x0400_8614);
        assert_eq!(command_overlay_tail().get() + 4, 0x0400_8618);
    }

    #[test]
    fn join_scan_and_lmc_request_addresses_follow_decoded_layout() {
        assert_eq!(join_timer().get(), 0x0400_89f8);
        assert_eq!(join_status().get(), 0x0400_8a0c);
        assert_eq!(join_interface_state().get(), 0x0400_8a0f);
        assert_eq!(lmc_request_pointer(0).unwrap().get(), 0x0400_8a24);
        assert_eq!(lmc_request_pointer(29).unwrap().get(), 0x0400_8a98);
        assert_eq!(lmc_request_status(0).unwrap().get(), 0x0400_8a9c);
        assert_eq!(lmc_request_status(27).unwrap().get(), 0x0400_8ab7);
        assert_eq!(lmc_request_status(28).unwrap().get(), 0x0400_8ab8);
        assert_eq!(lmc_request_status(29).unwrap().get(), 0x0400_8ab9);
        assert!(lmc_request_pointer(30).is_none());
        assert!(lmc_request_status(30).is_none());
    }

    #[test]
    fn pending_ba_lmc_and_message_addresses_follow_the_decoded_layout() {
        assert_eq!(ba_policy_enabled().get(), 0x0400_8ad3);
        assert_eq!(pending_tx_head().get(), 0x0400_8ad8);
        assert_eq!(pending_tx_tail().get(), 0x0400_8adc);
        assert_eq!(mac_bssid_mode().get(), 0x0400_8ae0);
        assert_eq!(pending_service_needed().get(), 0x0400_8ae3);
        assert_eq!(radio_owner().get(), 0x0400_8b20);
        assert_eq!(radio_wait_head().get(), 0x0400_8b24);
        assert_eq!(deferred_radio_owner().get(), 0x0400_8b2c);
        assert_eq!(radio_timer_state().get(), 0x0400_8b95);
        assert_eq!(lmc_message_control().get(), 0x0400_8ba8);
        assert_eq!(lmc_message_producer().get(), 0x0400_8bab);
        assert_eq!(lmc_message_consumer().get(), 0x0400_8bac);

        let first = lmc_message(0).unwrap();
        let last = lmc_message(LMC_MESSAGE_COUNT - 1).unwrap();
        assert_eq!(first.raw(), 0x0400_8bb8);
        assert_eq!(first.kind().get(), 0x0400_8bb8);
        assert_eq!(first.flags().get(), 0x0400_8bb9);
        assert_eq!(first.completion_tid().get(), 0x0400_8bbc);
        assert_eq!(first.completion_queue().get(), 0x0400_8bbd);
        assert_eq!(first.completion_sequence().get(), 0x0400_8bbe);
        assert_eq!(first.completion_mac_word(0).unwrap().get(), 0x0400_8bc0);
        assert_eq!(first.completion_mac_word(2).unwrap().get(), 0x0400_8bc4);
        assert_eq!(first.interface().get(), 0x0400_8be0);
        assert_eq!(first.completion_state().get(), 0x0400_8be1);
        assert_eq!(last.raw(), 0x0400_8e4c);
        assert_eq!(last.completion_state().get(), 0x0400_8e75);
        assert_eq!(last.raw() + LMC_MESSAGE_SIZE as u32, 0x0400_8e78);
        assert!(lmc_message(LMC_MESSAGE_COUNT).is_none());
        assert!(first.completion_mac_word(3).is_none());
    }

    #[test]
    fn ba_session_records_follow_the_decoded_four_by_0x28_layout() {
        let first = ba_session(0).unwrap();
        let last = ba_session(3).unwrap();
        assert_eq!(first.raw(), 0x0400_8e78);
        assert_eq!(first.activity().get(), 0x0400_8e78);
        assert_eq!(first.peer_mac_byte(0).unwrap().get(), 0x0400_8e7c);
        assert_eq!(first.peer_mac_byte(5).unwrap().get(), 0x0400_8e81);
        assert_eq!(first.tid().get(), 0x0400_8e82);
        assert_eq!(first.interface().get(), 0x0400_8e83);
        assert_eq!(first.timeout_1024us().get(), 0x0400_8e8a);
        assert_eq!(first.timer().get(), 0x0400_8e8c);
        assert_eq!(last.raw(), 0x0400_8ef0);
        assert_eq!(last.timer().get(), 0x0400_8f04);
        assert_eq!(last.raw() + 0x28, 0x0400_8f18);
        assert!(ba_session(4).is_none());
        assert!(first.peer_mac_byte(6).is_none());
    }

    #[test]
    fn ba_link_timers_and_network_flag_bytes_follow_decoded_offsets() {
        assert_eq!(ba_deferred_action().get(), 0x0400_8f18);
        assert_eq!(ba_deferred_interface().get(), 0x0400_8f19);
        assert_eq!(ba_periodic_timer_enabled().get(), 0x0400_8f1b);
        assert_eq!(ba_periodic_timer().get(), 0x0400_8f1c);
        assert_eq!(ba_transition_timer().get(), 0x0400_8f30);
        assert_eq!(current_network_flags().get(), 0x0400_8f44);
        assert_eq!(accumulated_network_flags().get(), 0x0400_8f45);
        assert_eq!(changed_network_flags().get(), 0x0400_8f46);
        assert_eq!(changed_network_flags().get() + 2, 0x0400_8f48);
    }

    #[test]
    fn host_accounting_roots_and_cross_family_boundaries_are_exact() {
        assert_eq!(HOST_DUPLICATE_CACHE_CURSOR.get(), 0x0400_87a4);
        assert_eq!(HOST_DEFERRED_EVENT_OWNER.get(), 0x0400_87a8);
        assert_eq!(AUXILIARY_TX_BUFFER_FREE_HEAD.get(), 0x0400_87ac);
        assert_eq!(HOST_CONTEXT_FREE_HEAD.get(), 0x0400_87b0);
        assert_eq!(HOST_CONTEXT_ADJACENT_STATE.get(), 0x0400_87b4);
        assert_eq!(HOST_CONTEXT_ADJACENT_STATE.get() + 4, LINK_SEQUENCE_ROOT.get());
        assert_eq!(host_context(0).unwrap().expected_frame_state().raw(), crate::packet_ram::host_frame_state(0) as u32);
    }

    #[test]
    fn internal_context_field_addresses_are_exact() {
        let first = InternalContextAddress::from_index(0).unwrap();
        let second = InternalContextAddress::from_index(1).unwrap();
        let last = InternalContextAddress::from_index(2).unwrap();

        assert_eq!(first.raw(), 0x0400_9084);
        assert_eq!(second.raw(), 0x0400_91f4);
        assert_eq!(last.raw(), 0x0400_9364);
        assert_eq!(first.intrusive_next().get(), 0x0400_9088);
        assert_eq!(first.borrowed_frame_address().get(), 0x0400_90a0);
        assert_eq!(first.completion_status().get(), 0x0400_90a4);
        assert_eq!(first.frame_address().get(), 0x0400_90d8);
        assert_eq!(first.terminal_status().get(), 0x0400_90f4);
        assert_eq!(first.cipher_buffer().get(), 0x0400_9148);
        assert_eq!(last.completion_byte_6c().get(), 0x0400_9424);
        assert!(InternalContextAddress::from_index(3).is_none());
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
