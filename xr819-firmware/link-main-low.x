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
     * Untranslated vendor-compatible state occupies the lower DTCM. The
     * matching vendor image initializes through 0x04009c44; round its temporary
     * compatibility envelope up to the next 1 KiB boundary. Rust-owned CPU
     * state lives in ITCM BSS, leaving the upper 8 KiB for mode stacks. This
     * boundary is transitional: translated vendor state should leave DTCM
     * rather than becoming a permanent compatibility ABI.
     */
    DTCM_LEGACY_LOW (rw) : ORIGIN = 0x04000000, LENGTH = 0x09080
    DTCM_CONTEXT_POOL (rw) : ORIGIN = 0x04009080, LENGTH = 0x00454
    DTCM_LEGACY_HIGH (rw) : ORIGIN = 0x040094d4, LENGTH = 0x00b2c
    DTCM_STACKS (rw) : ORIGIN = 0x0400a000, LENGTH = 0x02000

    PACKET_HOST_FRAME_STATES (rw) : ORIGIN = 0x09003678, LENGTH = 30 * 0x54
    PACKET_RESPONSE_POINTERS (rw) : ORIGIN = 0x09007000, LENGTH = 32 * 4
    PACKET_TX_COMMANDS (rw) : ORIGIN = 0x09007080, LENGTH = 4 * 4 * 0x54
    PACKET_RATE_RAM (rw) : ORIGIN = 0x090075c0, LENGTH = 80 * 0x10
    PACKET_DURATION_WORDS (rw) : ORIGIN = 0x09007bc0, LENGTH = 4
    PACKET_RESPONSE_COMMANDS (rw) : ORIGIN = 0x09007bc4, LENGTH = 13 * 0x54
    PACKET_INTERFACE_METADATA (rw) : ORIGIN = 0x09008008, LENGTH = 4
    PACKET_HIF_INPUTS (rw) : ORIGIN = 0x09008a68, LENGTH = 30 * 0x660
    PACKET_HIF_OUTPUTS (rw) : ORIGIN = 0x090149a8, LENGTH = 4 * 0x180
    PACKET_INTERNAL_TX (rw) : ORIGIN = 0x09014fa8, LENGTH = 3 * 0x400
    PACKET_SOFTWARE_RECORDS (rw) : ORIGIN = 0x09015fa8, LENGTH = 4 * 0x2a0
    PACKET_AUTO_RESPONSE_LIST (rw) : ORIGIN = 0x09016a28, LENGTH = 0x8c
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

    .packet_ram.host_frame_states 0x09003678 (NOLOAD) : ALIGN(4)
    {
        KEEP(*(.packet_ram.host_frame_states))
    } > PACKET_HOST_FRAME_STATES :NONE

    .packet_ram.response_pointers 0x09007000 (NOLOAD) : ALIGN(4)
    {
        KEEP(*(.packet_ram.response_pointers))
    } > PACKET_RESPONSE_POINTERS :NONE

    .packet_ram.tx_commands 0x09007080 (NOLOAD) : ALIGN(4)
    {
        KEEP(*(.packet_ram.tx_commands))
    } > PACKET_TX_COMMANDS :NONE

    .packet_ram.rate_ram 0x090075c0 (NOLOAD) : ALIGN(4)
    {
        KEEP(*(.packet_ram.rate_ram))
    } > PACKET_RATE_RAM :NONE

    .packet_ram.duration_words 0x09007bc0 (NOLOAD) : ALIGN(4)
    {
        KEEP(*(.packet_ram.duration_words))
    } > PACKET_DURATION_WORDS :NONE

    .packet_ram.response_commands 0x09007bc4 (NOLOAD) : ALIGN(4)
    {
        KEEP(*(.packet_ram.response_commands))
    } > PACKET_RESPONSE_COMMANDS :NONE

    .packet_ram.interface_metadata 0x09008008 (NOLOAD) : ALIGN(4)
    {
        KEEP(*(.packet_ram.interface_metadata))
    } > PACKET_INTERFACE_METADATA :NONE

    .packet_ram.hif_inputs 0x09008a68 (NOLOAD) : ALIGN(4)
    {
        KEEP(*(.packet_ram.hif_inputs))
    } > PACKET_HIF_INPUTS :NONE

    .packet_ram.hif_outputs 0x090149a8 (NOLOAD) : ALIGN(4)
    {
        KEEP(*(.packet_ram.hif_outputs))
    } > PACKET_HIF_OUTPUTS :NONE

    .packet_ram.internal_tx_buffers 0x09014fa8 (NOLOAD) : ALIGN(4)
    {
        KEEP(*(.packet_ram.internal_tx_buffers))
    } > PACKET_INTERNAL_TX :NONE
    __packet_ram_internal_tx_buffers_end =
        ADDR(.packet_ram.internal_tx_buffers) + SIZEOF(.packet_ram.internal_tx_buffers);

    .packet_ram.software_records 0x09015fa8 (NOLOAD) : ALIGN(4)
    {
        KEEP(*(.packet_ram.software_records))
    } > PACKET_SOFTWARE_RECORDS :NONE

    .packet_ram.automatic_response_list 0x09016a28 (NOLOAD) : ALIGN(4)
    {
        KEEP(*(.packet_ram.automatic_response_list))
    } > PACKET_AUTO_RESPONSE_LIST :NONE

    __packet_ram_lmc_anchor_0 = 0x09016ab4;
    __packet_ram_lmc_anchor_1 = 0x09017500;

    .packet_ram.rx_fifo_backing 0x09400000 (NOLOAD) : ALIGN(4)
    {
        KEEP(*(.packet_ram.rx_fifo_backing))
    } > PACKET_RX_FIFO :NONE

    .dtcm.context_pool (NOLOAD) : ALIGN(4)
    {
        __dtcm_context_pool_start = .;
        KEEP(*(.dtcm.context_pool))
        __dtcm_context_pool_end = .;
    } > DTCM_CONTEXT_POOL

    __itcm_image_end = ADDR(.noinit.exception) + SIZEOF(.noinit.exception);
    __itcm_observed_limit = ORIGIN(ITCM_OBSERVED) + LENGTH(ITCM_OBSERVED);
    __dtcm_legacy_base = ORIGIN(DTCM_LEGACY_LOW);
    __dtcm_stack_floor = ORIGIN(DTCM_STACKS);
    __dtcm_stack_top = ORIGIN(DTCM_STACKS) + LENGTH(DTCM_STACKS);

    ASSERT(__itcm_image_end <= __itcm_observed_limit,
           "XR819 image exceeds the conservative vendor ITCM envelope")
    ASSERT(__bss_end <= __itcm_observed_limit,
           "XR819 ITCM-backed BSS exceeds the observed envelope")
    ASSERT(ORIGIN(DTCM_CONTEXT_POOL) == ORIGIN(DTCM_LEGACY_LOW) + LENGTH(DTCM_LEGACY_LOW),
           "XR819 context pool must follow lower compatibility DTCM")
    ASSERT(ORIGIN(DTCM_LEGACY_HIGH) == ORIGIN(DTCM_CONTEXT_POOL) + LENGTH(DTCM_CONTEXT_POOL),
           "XR819 upper compatibility DTCM must follow the context pool")
    ASSERT(ORIGIN(DTCM_STACKS) == ORIGIN(DTCM_LEGACY_HIGH) + LENGTH(DTCM_LEGACY_HIGH),
           "XR819 compatibility DTCM and stacks are not contiguous")
    ASSERT(__dtcm_context_pool_start == 0x04009080,
           "XR819 internal context free head moved")
    ASSERT(__dtcm_context_pool_end == 0x040094d4,
           "XR819 internal context pool size changed")
    ASSERT(__dtcm_stack_floor == 0x0400a000,
           "XR819 observed stack floor changed")
    ASSERT(__dtcm_stack_top == 0x0400c000,
           "XR819 observed stack top changed")

    ASSERT(ADDR(.packet_ram.host_frame_states) == 0x09003678,
           "host frame-state pool moved")
    ASSERT(SIZEOF(.packet_ram.host_frame_states) == 30 * 0x54,
           "host frame-state pool size changed")
    ASSERT(ADDR(.packet_ram.response_pointers) == 0x09007000,
           "response-pointer table moved")
    ASSERT(SIZEOF(.packet_ram.response_pointers) == 32 * 4,
           "response-pointer table size changed")
    ASSERT(ADDR(.packet_ram.tx_commands) == 0x09007080,
           "TX command pool moved")
    ASSERT(SIZEOF(.packet_ram.tx_commands) == 4 * 4 * 0x54,
           "TX command pool size changed")
    ASSERT(ADDR(.packet_ram.rate_ram) == 0x090075c0,
           "rate RAM moved")
    ASSERT(SIZEOF(.packet_ram.rate_ram) == 80 * 0x10,
           "rate RAM size changed")
    ASSERT(ADDR(.packet_ram.duration_words) == 0x09007bc0,
           "duration words moved")
    ASSERT(SIZEOF(.packet_ram.duration_words) == 4,
           "duration-word size changed")
    ASSERT(ADDR(.packet_ram.response_commands) == 0x09007bc4,
           "response-command pool moved")
    ASSERT(SIZEOF(.packet_ram.response_commands) == 13 * 0x54,
           "response-command pool size changed")
    ASSERT(ADDR(.packet_ram.interface_metadata) == 0x09008008,
           "interface metadata moved")
    ASSERT(SIZEOF(.packet_ram.interface_metadata) == 4,
           "interface metadata size changed")
    ASSERT(ADDR(.packet_ram.hif_inputs) == 0x09008a68,
           "HIF input pool moved")
    ASSERT(SIZEOF(.packet_ram.hif_inputs) == 30 * 0x660,
           "HIF input pool size changed")
    ASSERT(ADDR(.packet_ram.hif_outputs) == 0x090149a8,
           "HIF output pool moved")
    ASSERT(SIZEOF(.packet_ram.hif_outputs) == 4 * 0x180,
           "HIF output pool size changed")
    ASSERT(ADDR(.packet_ram.internal_tx_buffers) == 0x09014fa8,
           "internal TX buffers moved")
    ASSERT(SIZEOF(.packet_ram.internal_tx_buffers) == 3 * 0x400,
           "internal TX buffer size changed")
    ASSERT(__packet_ram_internal_tx_buffers_end == 0x09015ba8,
           "internal TX boundary moved")
    ASSERT(ADDR(.packet_ram.software_records) == 0x09015fa8,
           "software records moved")
    ASSERT(SIZEOF(.packet_ram.software_records) == 4 * 0x2a0,
           "software record size changed")
    ASSERT(ADDR(.packet_ram.automatic_response_list) == 0x09016a28,
           "automatic-response list moved")
    ASSERT(SIZEOF(.packet_ram.automatic_response_list) == 0x8c,
           "automatic-response list size changed")
    ASSERT(__packet_ram_lmc_anchor_0 == 0x09016ab4,
           "first LMC anchor moved")
    ASSERT(__packet_ram_lmc_anchor_1 == 0x09017500,
           "second LMC anchor moved")
    ASSERT(ADDR(.packet_ram.rx_fifo_backing) == 0x09400000,
           "RX FIFO backing moved")
    ASSERT(SIZEOF(.packet_ram.rx_fifo_backing) == 0x8000,
           "RX FIFO backing size changed")

    ASSERT(ADDR(.packet_ram.tx_commands) ==
           ADDR(.packet_ram.response_pointers) + SIZEOF(.packet_ram.response_pointers),
           "response pointers and TX commands are not adjacent")
    ASSERT(ADDR(.packet_ram.rate_ram) ==
           ADDR(.packet_ram.tx_commands) + SIZEOF(.packet_ram.tx_commands),
           "TX commands and rate RAM are not adjacent")
    ASSERT(ADDR(.packet_ram.response_commands) ==
           ADDR(.packet_ram.duration_words) + SIZEOF(.packet_ram.duration_words),
           "duration and response commands are not adjacent")
    ASSERT(ADDR(.packet_ram.interface_metadata) ==
           ADDR(.packet_ram.response_commands) + SIZEOF(.packet_ram.response_commands),
           "response commands and interface metadata are not adjacent")
    ASSERT(ADDR(.packet_ram.hif_outputs) ==
           ADDR(.packet_ram.hif_inputs) + SIZEOF(.packet_ram.hif_inputs),
           "HIF input and output pools are not adjacent")
    ASSERT(ADDR(.packet_ram.internal_tx_buffers) ==
           ADDR(.packet_ram.hif_outputs) + SIZEOF(.packet_ram.hif_outputs),
           "HIF output and internal TX buffers are not adjacent")
    ASSERT(ADDR(.packet_ram.automatic_response_list) ==
           ADDR(.packet_ram.software_records) + SIZEOF(.packet_ram.software_records),
           "software records and response list are not adjacent")

    /DISCARD/ :
    {
        *(.ARM.exidx*)
        *(.ARM.extab*)
        *(.comment*)
    }
}
