ENTRY(_start)

/*
 * The low Thumb image executes from the ITCM mapping enabled by
 * download_boot_low's CP15 c9 setup. Instruction address 0 is therefore TCM,
 * not the host-visible staging SRAM at 0x08000000. Read-only data and the small
 * Rust data/BSS image currently share this low TCM address space; the runtime
 * stack and vendor-compatible fixed software state live in the DTCM mapping at
 * 0x04000000.
 */
MEMORY
{
    ITCM (rwx) : ORIGIN = 0x00000000, LENGTH = 0x1c000
}

SECTIONS
{
    .text ORIGIN(ITCM) : ALIGN(4)
    {
        KEEP(*(.text.entry))
        *(.text .text.*)
        *(.rodata .rodata.*)
    } > ITCM

    .data : ALIGN(4)
    {
        *(.data .data.*)
    } > ITCM

    .bss (NOLOAD) : ALIGN(4)
    {
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        __bss_end = .;
    } > ITCM

    /DISCARD/ :
    {
        *(.ARM.exidx*)
        *(.ARM.extab*)
        *(.comment*)
    }
}
