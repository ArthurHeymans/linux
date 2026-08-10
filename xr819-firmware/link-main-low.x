ENTRY(_start)

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
    /* Vendor bootstrap/runtime ownership observed below the mode stacks. */
    DTCM_RUNTIME (rw) : ORIGIN = 0x04000000, LENGTH = 0x0b000
    DTCM_STACKS  (rw) : ORIGIN = 0x0400b000, LENGTH = 0x01000
}

SECTIONS
{
    .text ORIGIN(ITCM_OBSERVED) : ALIGN(4)
    {
        __itcm_image_start = .;
        KEEP(*(.text.entry))
        *(.text .text.*)
        *(.rodata .rodata.*)
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

    __itcm_image_end = .;
    __itcm_observed_limit = ORIGIN(ITCM_OBSERVED) + LENGTH(ITCM_OBSERVED);
    __dtcm_base = ORIGIN(DTCM_RUNTIME);
    __dtcm_stack_floor = ORIGIN(DTCM_STACKS);
    __dtcm_stack_top = ORIGIN(DTCM_STACKS) + LENGTH(DTCM_STACKS);

    ASSERT(__itcm_image_end <= __itcm_observed_limit,
           "XR819 image exceeds the conservative vendor ITCM envelope")
    ASSERT(__bss_end <= __itcm_observed_limit,
           "XR819 ITCM-backed BSS exceeds the observed envelope")
    ASSERT(ORIGIN(DTCM_STACKS) == ORIGIN(DTCM_RUNTIME) + LENGTH(DTCM_RUNTIME),
           "XR819 observed DTCM runtime/stack windows overlap or have a gap")
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
