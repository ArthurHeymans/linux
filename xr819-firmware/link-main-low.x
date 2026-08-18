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
     * matching vendor image initializes through 0x04009c44; round its legacy
     * ownership envelope up to the next 1 KiB boundary. Native Rust state is
     * linker-owned in the following 4 KiB page, while the existing exception
     * and system stacks retain the top 4 KiB.
     */
    DTCM_LEGACY (rw) : ORIGIN = 0x04000000, LENGTH = 0x0a000
    DTCM_NATIVE (rw) : ORIGIN = 0x0400a000, LENGTH = 0x01000
    DTCM_STACKS  (rw) : ORIGIN = 0x0400b000, LENGTH = 0x01000
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

    .dtcm.bss (NOLOAD) : ALIGN(8)
    {
        __dtcm_bss_start = .;
        *(.dtcm.bss .dtcm.bss.*)
        __dtcm_bss_end = .;
    } > DTCM_NATIVE

    __itcm_image_end = ADDR(.noinit.exception) + SIZEOF(.noinit.exception);
    __itcm_observed_limit = ORIGIN(ITCM_OBSERVED) + LENGTH(ITCM_OBSERVED);
    __dtcm_legacy_base = ORIGIN(DTCM_LEGACY);
    __dtcm_native_base = ORIGIN(DTCM_NATIVE);
    __dtcm_native_limit = ORIGIN(DTCM_NATIVE) + LENGTH(DTCM_NATIVE);
    __dtcm_stack_floor = ORIGIN(DTCM_STACKS);
    __dtcm_stack_top = ORIGIN(DTCM_STACKS) + LENGTH(DTCM_STACKS);

    ASSERT(__itcm_image_end <= __itcm_observed_limit,
           "XR819 image exceeds the conservative vendor ITCM envelope")
    ASSERT(__bss_end <= __itcm_observed_limit,
           "XR819 ITCM-backed BSS exceeds the observed envelope")
    ASSERT(ORIGIN(DTCM_NATIVE) == ORIGIN(DTCM_LEGACY) + LENGTH(DTCM_LEGACY),
           "XR819 legacy and native DTCM windows are not contiguous")
    ASSERT(ORIGIN(DTCM_STACKS) == ORIGIN(DTCM_NATIVE) + LENGTH(DTCM_NATIVE),
           "XR819 native DTCM and stack windows are not contiguous")
    ASSERT(__dtcm_bss_end <= __dtcm_native_limit,
           "XR819 native DTCM BSS exceeds its linker-owned page")
    ASSERT(__dtcm_native_base == 0x0400a000,
           "XR819 native DTCM base changed")
    ASSERT(__dtcm_stack_floor == 0x0400b000,
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
