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
