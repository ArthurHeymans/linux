ENTRY(_start)

MEMORY
{
    RAM (rwx) : ORIGIN = 0xfff00000, LENGTH = 0x20000
}

SECTIONS
{
    __diagnostic_checkpoint_base = 0x0900fd00;
    __diagnostic_download_control = 0x0900ff80;
    __diagnostic_main_mailbox = 0x0900ff98;
    __diagnostic_main_heartbeat = 0x0900ff9c;

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
        *(.packet_ram.*)
        *(.ARM.exidx*)
        *(.ARM.extab*)
        *(.comment*)
    }
}
