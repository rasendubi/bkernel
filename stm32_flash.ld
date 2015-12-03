ENTRY(__isr_reset)

PROVIDE(__isr_nmi = __isr_default);
PROVIDE(__isr_hardfault = __isr_default);
PROVIDE(__isr_memmanage = __isr_default);
PROVIDE(__isr_busfault = __isr_default);
PROVIDE(__isr_usagefault = __isr_default);
PROVIDE(__isr_svc = __isr_default);
PROVIDE(__isr_debugmon = __isr_default);
PROVIDE(__isr_pendsv = __isr_default);
PROVIDE(__isr_systick = __isr_default);
PROVIDE(__isr_wwdg = __isr_default);
PROVIDE(__isr_pvd = __isr_default);
PROVIDE(__isr_tamp_stamp = __isr_default);
PROVIDE(__isr_rtc_wkup = __isr_default);
PROVIDE(__isr_flash = __isr_default);
PROVIDE(__isr_rcc = __isr_default);
PROVIDE(__isr_exti0 = __isr_default);
PROVIDE(__isr_exti1 = __isr_default);
PROVIDE(__isr_exti2 = __isr_default);
PROVIDE(__isr_exti3 = __isr_default);
PROVIDE(__isr_exti4 = __isr_default);
PROVIDE(__isr_dma1_stream0 = __isr_default);
PROVIDE(__isr_dma1_stream1 = __isr_default);
PROVIDE(__isr_dma1_stream2 = __isr_default);
PROVIDE(__isr_dma1_stream3 = __isr_default);
PROVIDE(__isr_dma1_stream4 = __isr_default);
PROVIDE(__isr_dma1_stream5 = __isr_default);
PROVIDE(__isr_dma1_stream6 = __isr_default);
PROVIDE(__isr_adc = __isr_default);
PROVIDE(__isr_can1_tx = __isr_default);
PROVIDE(__isr_can1_rx0 = __isr_default);
PROVIDE(__isr_can1_rx1 = __isr_default);
PROVIDE(__isr_can1_sce = __isr_default);
PROVIDE(__isr_exti9_5 = __isr_default);
PROVIDE(__isr_tim1_brk_tim9 = __isr_default);
PROVIDE(__isr_tim1_up_tim10 = __isr_default);
PROVIDE(__isr_tim1_trg_com_tim11 = __isr_default);
PROVIDE(__isr_tim1_cc = __isr_default);
PROVIDE(__isr_tim2 = __isr_default);
PROVIDE(__isr_tim3 = __isr_default);
PROVIDE(__isr_tim4 = __isr_default);
PROVIDE(__isr_i2c1_ev = __isr_default);
PROVIDE(__isr_i2c1_er = __isr_default);
PROVIDE(__isr_i2c2_ev = __isr_default);
PROVIDE(__isr_i2c2_er = __isr_default);
PROVIDE(__isr_spi1 = __isr_default);
PROVIDE(__isr_spi2 = __isr_default);
PROVIDE(__isr_usart1 = __isr_default);
PROVIDE(__isr_usart2 = __isr_default);
PROVIDE(__isr_usart3 = __isr_default);
PROVIDE(__isr_exti15_10 = __isr_default);
PROVIDE(__isr_rtc_alarm = __isr_default);
PROVIDE(__isr_otg_fs_wkup = __isr_default);
PROVIDE(__isr_tim8_brk_tim12 = __isr_default);
PROVIDE(__isr_tim8_up_tim13 = __isr_default);
PROVIDE(__isr_tim8_trg_com_tim14 = __isr_default);
PROVIDE(__isr_tim8_cc = __isr_default);
PROVIDE(__isr_dma1_stream7 = __isr_default);
PROVIDE(__isr_fsmc = __isr_default);
PROVIDE(__isr_sdio = __isr_default);
PROVIDE(__isr_tim5 = __isr_default);
PROVIDE(__isr_spi3 = __isr_default);
PROVIDE(__isr_uart4 = __isr_default);
PROVIDE(__isr_uart5 = __isr_default);
PROVIDE(__isr_tim6_dac = __isr_default);
PROVIDE(__isr_tim7 = __isr_default);
PROVIDE(__isr_dma2_stream0 = __isr_default);
PROVIDE(__isr_dma2_stream1 = __isr_default);
PROVIDE(__isr_dma2_stream2 = __isr_default);
PROVIDE(__isr_dma2_stream3 = __isr_default);
PROVIDE(__isr_dma2_stream4 = __isr_default);
PROVIDE(__isr_eth = __isr_default);
PROVIDE(__isr_eth_wkup = __isr_default);
PROVIDE(__isr_can2_tx = __isr_default);
PROVIDE(__isr_can2_rx0 = __isr_default);
PROVIDE(__isr_can2_rx1 = __isr_default);
PROVIDE(__isr_can2_sce = __isr_default);
PROVIDE(__isr_otg_fs = __isr_default);
PROVIDE(__isr_dma2_stream5 = __isr_default);
PROVIDE(__isr_dma2_stream6 = __isr_default);
PROVIDE(__isr_dma2_stream7 = __isr_default);
PROVIDE(__isr_usart6 = __isr_default);
PROVIDE(__isr_i2c3_ev = __isr_default);
PROVIDE(__isr_i2c3_er = __isr_default);
PROVIDE(__isr_otg_hs_ep1_out = __isr_default);
PROVIDE(__isr_otg_hs_ep1_in = __isr_default);
PROVIDE(__isr_otg_hs_wkup = __isr_default);
PROVIDE(__isr_otg_hs = __isr_default);
PROVIDE(__isr_dcmi = __isr_default);
PROVIDE(__isr_cryp = __isr_default);
PROVIDE(__isr_hash_rng = __isr_default);
PROVIDE(__isr_fpu = __isr_default);

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
