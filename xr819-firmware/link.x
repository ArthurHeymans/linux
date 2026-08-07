ENTRY(_start)

MEMORY
{
    SRAM (rwx) : ORIGIN = 0x08000000, LENGTH = 0x10000
}

SECTIONS
{
    .text ORIGIN(SRAM) : ALIGN(4)
    {
        KEEP(*(.text.entry))
        *(.text .text.*)
        *(.rodata .rodata.*)
    } > SRAM

    .data : ALIGN(4)
    {
        *(.data .data.*)
    } > SRAM

    .bss (NOLOAD) : ALIGN(4)
    {
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        __bss_end = .;
    } > SRAM

    /DISCARD/ :
    {
        *(.ARM.exidx*)
        *(.ARM.extab*)
        *(.comment*)
    }
}
