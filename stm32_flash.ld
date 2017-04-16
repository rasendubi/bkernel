INCLUDE isr_vector.ld;
INCLUDE peripherals.ld;

__stack_end = 0x20020000;    /* end of 128K RAM on AHB bus*/

MEMORY
{
    FLASH (rx)      : ORIGIN = 0x08000000, LENGTH = 1024K
    RAM (xrw)       : ORIGIN = 0x20000000, LENGTH = 192K
    MEMORY_B1 (rx)  : ORIGIN = 0x60000000, LENGTH = 0K
}

SECTIONS
{
    .isr_vector :
    {
        . = ALIGN(4);
        KEEP(*(.stack_end))
        KEEP(*(.isr_vector))
        . = ALIGN(4);
    } >FLASH

    .text :
    {
        . = ALIGN(4);
        *(.text)
        *(.text*)
        *(.rodata)
        *(.rodata*)
        *(.glue_7)
        *(.glue_7t)
        *(.eh_frame)

        KEEP (*(.init))
        KEEP (*(.fini))

        . = ALIGN(4);
    } >FLASH


    .ARM.extab   : { *(.ARM.extab* .gnu.linkonce.armextab.*) } >FLASH
    .ARM : {
        __exidx_start = .;
        *(.ARM.exidx*)
        __exidx_end = .;
    } >FLASH

    . = ALIGN(4);
    __init_data_start = .;

    .data : AT ( __init_data_start )
    {
        . = ALIGN(4);
        __data_start = .;
        *(.data)
        *(.data*)

        . = ALIGN(4);
        __data_end = .;
    } >RAM

    .bss :
    {
        . = ALIGN(4);
        __bss_start = .;
        *(.bss)
        *(.bss*)
        *(COMMON)

        . = ALIGN(4);
        __bss_end = .;
    } >RAM

    .ARM.attributes 0 : { *(.ARM.attributes) }
}
