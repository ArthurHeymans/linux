ENTRY(xr819_reset_entry)

/*
 * The low Thumb image executes from the ITCM mapping enabled by
 * download_boot_low's CP15 c9 setup. Instruction address 0 is therefore TCM,
 * not the host-visible staging SRAM at 0x08000000. Read-only data and the small
 * Rust data/BSS image currently share this low TCM address space; the runtime
 * stack and vendor-compatible fixed software state live in the DTCM mapping at
 * 0x04000000.
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
    DTCM_LEGACY (rw) : ORIGIN = 0x04000000, LENGTH = 0x0a000
    DTCM_STACKS (rw) : ORIGIN = 0x0400a000, LENGTH = 0x02000
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
         * This is a no-op when the size is already aligned.
         */
        . = ALIGN(4);
    } > ITCM_OBSERVED

    .data : ALIGN(4)
    {
        *(.data .data.*)
    } > ITCM_OBSERVED

    .bss (NOLOAD) : ALIGN(4)
    {
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        __bss_end = .;
    } > ITCM_OBSERVED

    .noinit.exception (NOLOAD) : ALIGN(8)
    {
        KEEP(*(.noinit.exception))
    } > ITCM_OBSERVED

    __itcm_image_end = ADDR(.noinit.exception) + SIZEOF(.noinit.exception);
    __itcm_observed_limit = ORIGIN(ITCM_OBSERVED) + LENGTH(ITCM_OBSERVED);
    __dtcm_legacy_base = ORIGIN(DTCM_LEGACY);
    __dtcm_stack_floor = ORIGIN(DTCM_STACKS);
    __dtcm_stack_top = ORIGIN(DTCM_STACKS) + LENGTH(DTCM_STACKS);

    ASSERT(__itcm_image_end <= __itcm_observed_limit,
           "XR819 image exceeds the conservative vendor ITCM envelope")
    ASSERT(__bss_end <= __itcm_observed_limit,
           "XR819 ITCM-backed BSS exceeds the observed envelope")
    ASSERT(ORIGIN(DTCM_STACKS) == ORIGIN(DTCM_LEGACY) + LENGTH(DTCM_LEGACY),
           "XR819 compatibility DTCM and stacks are not contiguous")
    ASSERT(__dtcm_stack_floor == 0x0400a000,
           "XR819 observed stack floor changed")
    ASSERT(__dtcm_stack_top == 0x0400c000,
           "XR819 observed stack top changed")

    /DISCARD/ :
    {
        *(.ARM.exidx*)
        *(.ARM.extab*)
        *(.comment*)
    }
}
