ENTRY(_start)

SECTIONS
{
    __download_fifo_base = 0x09004000;
    __download_fifo_end = 0x0900c000;
    __download_checkpoint_base = 0x0900fd00;
    __download_control_base = 0x0900ff80;

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
        __relocated_bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        __relocated_bss_end = .;
    }

    ASSERT(__relocated_start == 0x09010000,
           "bootstrap relocation start moved")
    ASSERT(__relocated_bss_end <= 0x090149a8,
           "bootstrap image or BSS outgrew the documented HIF-input overlay")
    ASSERT(__download_fifo_end - __download_fifo_base == 0x8000,
           "bootstrap FIFO size changed")

    /DISCARD/ :
    {
        *(.packet_ram.*)
        *(.dtcm.*)
        *(.ARM.exidx*)
        *(.ARM.extab*)
        *(.comment*)
    }
}
