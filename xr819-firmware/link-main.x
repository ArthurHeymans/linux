ENTRY(_start)

MEMORY
{
    RAM (rwx) : ORIGIN = 0xfff00000, LENGTH = 0x20000
}

SECTIONS
{
    .text ORIGIN(RAM) : ALIGN(4)
    {
        KEEP(*(.text.entry))
        *(.text .text.*)
        *(.rodata .rodata.*)
    } > RAM

    .data : ALIGN(4)
    {
        *(.data .data.*)
    } > RAM

    .bss (NOLOAD) : ALIGN(4)
    {
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        __bss_end = .;
    } > RAM

    /DISCARD/ :
    {
        *(.ARM.exidx*)
        *(.ARM.extab*)
        *(.comment*)
    }
}
