ENTRY(xr819_hif_extension_probe)

MEMORY
{
    HIGH_EXTENSION (rwx) : ORIGIN = 0xfff00000, LENGTH = 0x00010000
}

SECTIONS
{
    .text ORIGIN(HIGH_EXTENSION) : ALIGN(4)
    {
        KEEP(*(.text.entry))
        . = ORIGIN(HIGH_EXTENSION) + 0x100;
        KEEP(*(.text.gain_entry))
        *(.text .text.*)
        *(.rodata .rodata.*)
    } > HIGH_EXTENSION

    .data : ALIGN(4)
    {
        *(.data .data.*)
    } > HIGH_EXTENSION

    /DISCARD/ :
    {
        *(.ARM.exidx*)
        *(.ARM.extab*)
        *(.comment*)
    }
}
