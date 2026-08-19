ENTRY(_start)

MEMORY
{
    SRAM (rwx) : ORIGIN = 0x08000000, LENGTH = 0x10000
}

SECTIONS
{
    __download_control_base = 0x0900ff80;
    __download_trace_pc = 0x0900ff8c;

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
        *(.packet_ram.*)
        *(.ARM.exidx*)
        *(.ARM.extab*)
        *(.comment*)
    }
}
