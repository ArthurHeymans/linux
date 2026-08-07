ENTRY(_start)

SECTIONS
{
    .entry 0x08000000 : ALIGN(4)
    {
        KEEP(*(.text.entry))
    }

    .relocated 0x09010000 : AT(LOADADDR(.entry) + SIZEOF(.entry))
    {
        __relocated_start = .;
        *(.text .text.*)
        *(.rodata .rodata.*)
        *(.data .data.*)
        __relocated_end = .;
    }
    __relocated_load = LOADADDR(.relocated);

    .bss (NOLOAD) : ALIGN(4)
    {
        *(.bss .bss.*)
        *(COMMON)
    }

    /DISCARD/ :
    {
        *(.ARM.exidx*)
        *(.ARM.extab*)
        *(.comment*)
    }
}
