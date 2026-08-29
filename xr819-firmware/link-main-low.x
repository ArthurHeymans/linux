ENTRY(xr819_reset_entry)

/*
 * The low Thumb image executes from the ITCM mapping enabled by
 * download_boot_low's CP15 c9 setup. Instruction address 0 is therefore TCM,
 * not the host-visible staging SRAM at 0x08000000. Read-only data and the small
 * Rust data/BSS image currently share this low TCM address space; the runtime
 * stack and vendor-compatible fixed software state live in the DTCM mapping at
 * 0x04000000.
 *
 * Packet-RAM sections below are independent address-only ownership objects.
 * They deliberately have no enclosing MEMORY region and no program header:
 * unknown holes and bootstrap overlays are not allocatable runtime storage.
 *
 * These bounds are conservative observations from the matching vendor image,
 * not claims about undocumented physical TCM capacities.
 */
MEMORY
{
    /* Vendor ITCM content ends at 0x1b3dc; round up to the next 4 KiB page. */
    ITCM_OBSERVED (rwx) : ORIGIN = 0x00000000, LENGTH = 0x1c000
    /*
     * One Rust-described quarantine ABI covers all retained software state.
     * It is NOLOAD: vendor COPY data below 0x04002078 remains retained, while
     * startup explicitly clears the historical BSS range. Unknown bytes are
     * occupied ABI state rather than linker-allocatable holes.
     */
    DTCM_STATE (rw) : ORIGIN = 0x04000000, LENGTH = 0x0a000
    DTCM_STACKS (rw) : ORIGIN = 0x0400a000, LENGTH = 0x02000

    /* Rust-owned runtime objects are packed in source-controlled section order. */
    PACKET_RUNTIME (rw) : ORIGIN = 0x09007000, LENGTH = 0xf630
    PACKET_RX_FIFO (rw) : ORIGIN = 0x09400000, LENGTH = 0x8000
}

PHDRS
{
    text PT_LOAD FLAGS(5);
    data PT_LOAD FLAGS(6);
}

SECTIONS
{
    .text ORIGIN(ITCM_OBSERVED) : ALIGN(4)
    {
        __itcm_image_start = .;
        KEEP(*(.vectors))
        KEEP(*(.text.entry))
        *(.text .text.*)
        *(.rodata .rodata.*)
        /*
         * The image packer requires every PT_LOAD file size to be word
         * aligned. Without this the read-only segment ends wherever the last
         * rodata item happens to land and packing succeeds only by luck.
         */
        . = ALIGN(4);
    } > ITCM_OBSERVED :text

    .data : ALIGN(4)
    {
        *(.data .data.*)
    } > ITCM_OBSERVED :data

    .bss (NOLOAD) : ALIGN(4)
    {
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        __bss_end = .;
    } > ITCM_OBSERVED :data

    .noinit.exception (NOLOAD) : ALIGN(8)
    {
        KEEP(*(.noinit.exception))
    } > ITCM_OBSERVED :data

    .packet_ram.runtime (NOLOAD) : ALIGN(4)
    {
        __packet_ram_runtime_start = .;
        KEEP(*(SORT_BY_NAME(.packet_ram.runtime.*)))
        __packet_ram_runtime_end = .;
    } > PACKET_RUNTIME :NONE

    __packet_ram_lmc_anchor_0 = 0x09016ab4;
    __packet_ram_lmc_anchor_1 = 0x09017500;

    .packet_ram.rx_fifo_backing 0x09400000 (NOLOAD) : ALIGN(4)
    {
        KEEP(*(.packet_ram.rx_fifo_backing))
    } > PACKET_RX_FIFO :NONE

    /*
     * Lower DTCM is link-placed by startup contract rather than represented as
     * one monolithic opaque allocation. These remain NOLOAD while the existing
     * vendor COPY and explicit zero-fill paths own initialization.
     */
    .dtcm.data (NOLOAD) : ALIGN(4)
    {
        __dtcm_data_start = .;
        KEEP(*(SORT_BY_NAME(.dtcm.data.*)))
        __dtcm_data_end = .;
    } > DTCM_STATE :NONE

    .dtcm.bss (NOLOAD) : ALIGN(4)
    {
        __dtcm_bss_start = .;
        KEEP(*(.dtcm.bss.runtime_prefix))
        KEEP(*(.dtcm.bss.clock_parameters))
        KEEP(*(.dtcm.bss.scheduler_handlers))
        KEEP(*(.dtcm.bss.pre_configuration_tables))
        KEEP(*(.dtcm.bss.sdd_configuration_tables))
        KEEP(*(.dtcm.bss.wake_context_state))
        KEEP(*(.dtcm.bss.duration_sources))
        KEEP(*(.dtcm.bss.pre_low_mac_word))
        KEEP(*(.dtcm.bss.low_mac_pas))
        KEEP(*(.dtcm.bss.pre_vif_header))
        KEEP(*(.dtcm.bss.vifs))
        KEEP(*(.dtcm.bss.post_vif_quarantine))
        KEEP(*(.dtcm.bss.host_tx_contexts))
        KEEP(*(.dtcm.bss.pre_command_quarantine))
        KEEP(*(.dtcm.bss.command_channel_switch))
        KEEP(*(.dtcm.bss.lmc_control_roots))
        KEEP(*(.dtcm.bss.host_context_accounting))
        KEEP(*(.dtcm.bss.host_context_free_list))
        KEEP(*(.dtcm.bss.link_and_sequence))
        KEEP(*(.dtcm.bss.join_scan_control))
        KEEP(*(.dtcm.bss.wsm_response_scratch))
        KEEP(*(.dtcm.bss.ba_lmc_header))
        KEEP(*(.dtcm.bss.pending_ba_lmc))
        KEEP(*(.dtcm.bss.lmc_messages))
        KEEP(*(.dtcm.bss.ba_sessions))
        KEEP(*(.dtcm.bss.ba_link_event_state))
        KEEP(*(.dtcm.bss.tala))
        KEEP(*(.dtcm.bss.context_completion_prefix))
        KEEP(*(.dtcm.bss.pre_internal_context_quarantine))
        KEEP(*(.dtcm.bss.internal_context_prefix))
        KEEP(*(.dtcm.bss.internal_context_pool))
        KEEP(*(.dtcm.bss.power_save))
        KEEP(*(.dtcm.bss.power_save_hif_boundary))
        KEEP(*(.dtcm.bss.hif_buffer_state))
        KEEP(*(.dtcm.bss.legacy_hif_software_state))
        KEEP(*(.dtcm.bss.mic_completion_state))
        KEEP(*(.dtcm.bss.phy_core))
        KEEP(*(.dtcm.bss.phy_tail))
        __dtcm_bss_end = .;
    } > DTCM_STATE :NONE

    .dtcm.noinit (NOLOAD) : ALIGN(4)
    {
        __dtcm_noinit_start = .;
        KEEP(*(.dtcm.noinit))
        __dtcm_noinit_end = .;
    } > DTCM_STATE :NONE

    __dtcm_state_start = __dtcm_data_start;
    __dtcm_state_end = __dtcm_noinit_end;

    __itcm_image_end = ADDR(.noinit.exception) + SIZEOF(.noinit.exception);
    __itcm_observed_limit = ORIGIN(ITCM_OBSERVED) + LENGTH(ITCM_OBSERVED);
    /*
     * Export raw member-view symbols from the actual Rust object. Code can
     * materialize these addresses directly without creating separate sections.
     */
    __dtcm_state_object_start = ORIGIN(DTCM_STATE);
    __dtcm_state_object_end = __dtcm_state_end;
    __dtcm_context_pool_start = DTCM_INTERNAL_CONTEXT_POOL;
    __dtcm_context_pool_contexts = DTCM_INTERNAL_CONTEXT_POOL + 0x4;
    __dtcm_context_pool_end = __dtcm_context_pool_contexts + 3 * 0x170;
    __dtcm_stack_floor = ORIGIN(DTCM_STACKS);
    __dtcm_stack_top = ORIGIN(DTCM_STACKS) + LENGTH(DTCM_STACKS);

    ASSERT(__itcm_image_end <= __itcm_observed_limit,
           "XR819 image exceeds the conservative vendor ITCM envelope")
    ASSERT(__bss_end <= __itcm_observed_limit,
           "XR819 ITCM-backed BSS exceeds the observed envelope")
    ASSERT(ORIGIN(DTCM_STATE) == __dtcm_state_start,
           "XR819 Rust DTCM regions do not start at the state base")
    ASSERT(__dtcm_state_object_start == 0x04000000,
           "XR819 DTCM state object base moved")
    ASSERT(__dtcm_data_start == ORIGIN(DTCM_STATE),
           "XR819 DTCM initialized-data base moved")
    ASSERT(__dtcm_data_end == ORIGIN(DTCM_STATE) + 0x2078,
           "XR819 DTCM initialized-data extent changed")
    ASSERT(SIZEOF(.dtcm.data) == 0x2078,
           "XR819 DTCM initialized-data size changed")
    ASSERT(DTCM_DATA_MAC_SLOT_TIMING_PATCH_LIST == __dtcm_data_start,
           "XR819 first initialized-data object moved")
    ASSERT(DTCM_DATA_INITIALIZED_TAIL + 0x60 == __dtcm_data_end,
           "XR819 last initialized-data object moved")
    ASSERT(__dtcm_bss_start == ORIGIN(DTCM_STATE) + 0x2078,
           "XR819 DTCM BSS base moved")
    ASSERT(__dtcm_bss_end == ORIGIN(DTCM_STATE) + 0x9c44,
           "XR819 DTCM BSS extent changed")
    ASSERT(SIZEOF(.dtcm.bss) == 0x7bcc,
           "XR819 DTCM BSS size changed")
    ASSERT(DTCM_RUNTIME_PREFIX == __dtcm_bss_start,
           "XR819 typed runtime prefix moved")
    ASSERT(DTCM_CLOCK_PARAMETERS == DTCM_RUNTIME_PREFIX + 0x114,
           "XR819 typed clock parameters moved")
    ASSERT(DTCM_SCHEDULER_HANDLERS == DTCM_CLOCK_PARAMETERS + 0x28,
           "XR819 typed scheduler handlers moved")
    ASSERT(DTCM_PRE_CONFIGURATION_TABLES == DTCM_SCHEDULER_HANDLERS + 0x80,
           "XR819 typed pre-configuration tables moved")
    ASSERT(DTCM_SDD_CONFIGURATION_TABLES == DTCM_PRE_CONFIGURATION_TABLES + 0x127c,
           "XR819 typed SDD configuration tables moved")
    ASSERT(DTCM_WAKE_CONTEXT_STATE == DTCM_SDD_CONFIGURATION_TABLES + 0x130,
           "XR819 typed wake context moved")
    ASSERT(DTCM_DURATION_SOURCES == DTCM_WAKE_CONTEXT_STATE + 0x90,
           "XR819 typed duration sources moved")
    ASSERT(DTCM_PRE_LOW_MAC_WORD == DTCM_DURATION_SOURCES + 0x4,
           "XR819 typed pre-low-MAC word moved")
    ASSERT(DTCM_LOW_MAC_PAS == DTCM_PRE_LOW_MAC_WORD + 0x4,
           "XR819 typed low-MAC PAS state moved")
    ASSERT(DTCM_PRE_VIF_HEADER == DTCM_LOW_MAC_PAS + 0x800,
           "XR819 typed pre-VIF header moved")
    ASSERT(DTCM_VIFS == DTCM_PRE_VIF_HEADER + 0x20,
           "XR819 typed VIF records moved")
    ASSERT(DTCM_POST_VIF_QUARANTINE == DTCM_VIFS + 0xb10,
           "XR819 typed post-VIF quarantine moved")
    ASSERT(DTCM_HOST_TX_CONTEXTS == DTCM_POST_VIF_QUARANTINE + 0x107c,
           "XR819 typed host-TX contexts moved")
    ASSERT(DTCM_PRE_COMMAND_QUARANTINE == DTCM_HOST_TX_CONTEXTS + 30 * 0x170,
           "XR819 typed pre-command quarantine moved")
    ASSERT(DTCM_COMMAND_CHANNEL_SWITCH == DTCM_PRE_COMMAND_QUARANTINE + 0x50,
           "XR819 typed command/channel overlay moved")
    ASSERT(DTCM_LMC_CONTROL_ROOTS == DTCM_COMMAND_CHANNEL_SWITCH + 0x84,
           "XR819 typed LMC control roots moved")
    ASSERT(DTCM_HOST_CONTEXT_ACCOUNTING == DTCM_LMC_CONTROL_ROOTS + 0x180,
           "XR819 typed host-context accounting moved")
    ASSERT(DTCM_HOST_CONTEXT_FREE_LIST == DTCM_HOST_CONTEXT_ACCOUNTING + 0x18,
           "XR819 typed host-context free list moved")
    ASSERT(DTCM_LINK_AND_SEQUENCE == DTCM_HOST_CONTEXT_FREE_LIST + 0x8,
           "XR819 typed link/sequence state moved")
    ASSERT(DTCM_JOIN_SCAN_CONTROL == DTCM_LINK_AND_SEQUENCE + 0x220,
           "XR819 typed join/scan control moved")
    ASSERT(DTCM_WSM_RESPONSE_SCRATCH == DTCM_JOIN_SCAN_CONTROL + 0x40,
           "XR819 typed WSM response scratch moved")
    ASSERT(DTCM_BA_LMC_HEADER == DTCM_WSM_RESPONSE_SCRATCH + 0xa0,
           "XR819 typed BA/LMC header moved")
    ASSERT(DTCM_PENDING_BA_LMC == DTCM_BA_LMC_HEADER + 0x20,
           "XR819 typed pending BA/LMC state moved")
    ASSERT(DTCM_LMC_MESSAGES == DTCM_PENDING_BA_LMC + 0xe0,
           "XR819 typed LMC messages moved")
    ASSERT(DTCM_BA_SESSIONS == DTCM_LMC_MESSAGES + 0x2c0,
           "XR819 typed BA sessions moved")
    ASSERT(DTCM_BA_LINK_EVENT_STATE == DTCM_BA_SESSIONS + 0xa0,
           "XR819 typed BA link/event state moved")
    ASSERT(DTCM_TALA == DTCM_BA_LINK_EVENT_STATE + 0x30,
           "XR819 typed TALA accounting moved")
    ASSERT(DTCM_CONTEXT_COMPLETION_PREFIX == DTCM_TALA + 0x24,
           "XR819 typed context-completion prefix moved")
    ASSERT(DTCM_PRE_INTERNAL_CONTEXT_QUARANTINE == DTCM_CONTEXT_COMPLETION_PREFIX + 0x14,
           "XR819 typed pre-internal-context quarantine moved")
    ASSERT(DTCM_INTERNAL_CONTEXT_PREFIX == DTCM_PRE_INTERNAL_CONTEXT_QUARANTINE + 0xec,
           "XR819 typed internal-context prefix moved")
    ASSERT(DTCM_INTERNAL_CONTEXT_POOL == DTCM_INTERNAL_CONTEXT_PREFIX + 0x14,
           "XR819 typed internal-context pool moved")
    ASSERT(DTCM_POWER_SAVE == DTCM_INTERNAL_CONTEXT_POOL + 0x454,
           "XR819 typed power-save state moved")
    ASSERT(DTCM_POWER_SAVE_HIF_BOUNDARY == DTCM_POWER_SAVE + 0x208,
           "XR819 typed power-save/HIF boundary moved")
    ASSERT(DTCM_HIF_BUFFER_STATE == DTCM_POWER_SAVE_HIF_BOUNDARY + 0x44,
           "XR819 typed HIF buffer state moved")
    ASSERT(DTCM_LEGACY_HIF_SOFTWARE_STATE == DTCM_HIF_BUFFER_STATE + 0x34,
           "XR819 typed legacy HIF state moved")
    ASSERT(DTCM_MIC_COMPLETION_STATE == DTCM_LEGACY_HIF_SOFTWARE_STATE + 0x1d4,
           "XR819 typed MIC completion state moved")
    ASSERT(DTCM_PHY_CORE == DTCM_MIC_COMPLETION_STATE + 0x14,
           "XR819 typed PHY core state moved")
    ASSERT(DTCM_PHY_TAIL == DTCM_PHY_CORE + 0xd0,
           "XR819 typed PHY tail moved")
    ASSERT(__dtcm_noinit_start == DTCM_PHY_TAIL + 0x238,
           "XR819 DTCM no-init boundary does not follow PHY tail")
    ASSERT(__dtcm_noinit_start == ORIGIN(DTCM_STATE) + 0x9c44,
           "XR819 DTCM no-init base moved")
    ASSERT(__dtcm_noinit_end == ORIGIN(DTCM_STATE) + LENGTH(DTCM_STATE),
           "XR819 DTCM no-init extent changed")
    ASSERT(SIZEOF(.dtcm.noinit) == 0x3bc,
           "XR819 DTCM no-init size changed")
    ASSERT(__dtcm_state_object_end == 0x0400a000,
           "XR819 DTCM state object size changed")
    ASSERT(__dtcm_state_end == __dtcm_state_object_end,
           "XR819 DTCM regions do not fill the state object")
    ASSERT(ORIGIN(DTCM_STACKS) == ORIGIN(DTCM_STATE) + LENGTH(DTCM_STATE),
           "XR819 DTCM state and stacks are not contiguous")
    ASSERT(__dtcm_context_pool_start == 0x04009080,
           "XR819 internal context free head moved")
    ASSERT(__dtcm_context_pool_contexts == 0x04009084,
           "XR819 first internal context moved")
    ASSERT(__dtcm_context_pool_end == 0x040094d4,
           "XR819 internal context pool size changed")
    ASSERT(__dtcm_stack_floor == 0x0400a000,
           "XR819 observed stack floor changed")
    ASSERT(__dtcm_stack_top == 0x0400c000,
           "XR819 observed stack top changed")

    ASSERT(__packet_ram_runtime_start == ORIGIN(PACKET_RUNTIME),
           "Rust packet-RAM region moved")
    ASSERT(__packet_ram_runtime_end == ORIGIN(PACKET_RUNTIME) + LENGTH(PACKET_RUNTIME),
           "Rust packet-RAM region size changed")
    ASSERT(SIZEOF(.packet_ram.runtime) == 0xf630,
           "Rust packet-RAM objects no longer fill their region")
    ASSERT(__packet_ram_runtime_end <= __packet_ram_lmc_anchor_0,
           "Rust packet-RAM objects overlap retained LMC state")
    ASSERT(__packet_ram_lmc_anchor_0 == 0x09016ab4,
           "first LMC anchor moved")
    ASSERT(__packet_ram_lmc_anchor_1 == 0x09017500,
           "second LMC anchor moved")
    ASSERT(ADDR(.packet_ram.rx_fifo_backing) == 0x09400000,
           "RX FIFO backing moved")
    ASSERT(SIZEOF(.packet_ram.rx_fifo_backing) == 0x8000,
           "RX FIFO backing size changed")

    /DISCARD/ :
    {
        *(.ARM.exidx*)
        *(.ARM.extab*)
        *(.comment*)
    }
}
