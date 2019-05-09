//! Interrupt Service Routines vector.
//!
//! This module defines order of ISRs in the vector. The vector is
//! installed in appropriate place in the linker script.

// The Void type has 0 size. It's used here to allow aliasing of
// __data/bss_start/end. Otherwise, compiler may generate incorrect
// code and the board will hang.
enum Void {}

extern {
    static mut __init_data_start: u32;
    static mut __data_start: Void;
    static mut __data_end: Void;
    static mut __bss_start: Void;
    static mut __bss_end: Void;

    fn kmain();
}

/// Reset handler. It copies `.data` segment, initializes `.bss` to
/// zeros, and calls `kmain()`.
#[no_mangle]
// allow casting `*mut Void` to `*mut u32`
#[allow(clippy::cast_ptr_alignment)]
pub unsafe extern "C" fn __isr_reset() {
    let mut to = &mut __data_start as *mut Void as *mut u32;
    let data_end = &mut __data_end as *mut Void as *mut u32;

    let mut from = &mut __init_data_start as *mut u32;

    while to != data_end {
        *to = *from;

        to = to.offset(1);
        from = from.offset(1);
    }

    to = &mut __bss_start as *mut Void as *mut u32;
    let bss_end = &mut __bss_end as *mut Void as *mut u32;

    while to < bss_end {
        *to = 0;

        to = to.offset(1);
    }

    kmain();

    panic!("kmain returned!");
}

/// Default ISR. It just panics.
#[cfg(target_arch = "arm")]
#[no_mangle]
pub unsafe extern fn __isr_default() {
    let ipsr: u32;
    asm!("mrs $0, IPSR" : "=r" (ipsr));
    if ipsr >= 16 {
        panic!("Unknown ISR handler: {} (IRQ {})!", ipsr, ipsr - 16);
    } else {
        panic!("Unknown ISR handler: {}!", ipsr);
    }
}

extern {
    static __stack_end: u32;
    pub fn __isr_nmi();
    pub fn __isr_hardfault();
    pub fn __isr_memmanage();
    pub fn __isr_busfault();
    pub fn __isr_usagefault();
    pub fn __isr_svc();
    pub fn __isr_debugmon();
    pub fn __isr_pendsv();
    pub fn __isr_systick();
    pub fn __isr_wwdg();
    pub fn __isr_pvd();
    pub fn __isr_tamp_stamp();
    pub fn __isr_rtc_wkup();
    pub fn __isr_flash();
    pub fn __isr_rcc();
    pub fn __isr_exti0();
    pub fn __isr_exti1();
    pub fn __isr_exti2();
    pub fn __isr_exti3();
    pub fn __isr_exti4();
    pub fn __isr_dma1_stream0();
    pub fn __isr_dma1_stream1();
    pub fn __isr_dma1_stream2();
    pub fn __isr_dma1_stream3();
    pub fn __isr_dma1_stream4();
    pub fn __isr_dma1_stream5();
    pub fn __isr_dma1_stream6();
    pub fn __isr_adc();
    pub fn __isr_can1_tx();
    pub fn __isr_can1_rx0();
    pub fn __isr_can1_rx1();
    pub fn __isr_can1_sce();
    pub fn __isr_exti9_5();
    pub fn __isr_tim1_brk_tim9();
    pub fn __isr_tim1_up_tim10();
    pub fn __isr_tim1_trg_com_tim11();
    pub fn __isr_tim1_cc();
    pub fn __isr_tim2();
    pub fn __isr_tim3();
    pub fn __isr_tim4();
    pub fn __isr_i2c1_ev();
    pub fn __isr_i2c1_er();
    pub fn __isr_i2c2_ev();
    pub fn __isr_i2c2_er();
    pub fn __isr_spi1();
    pub fn __isr_spi2();
    pub fn __isr_usart1();
    pub fn __isr_usart2();
    pub fn __isr_usart3();
    pub fn __isr_exti15_10();
    pub fn __isr_rtc_alarm();
    pub fn __isr_otg_fs_wkup();
    pub fn __isr_tim8_brk_tim12();
    pub fn __isr_tim8_up_tim13();
    pub fn __isr_tim8_trg_com_tim14();
    pub fn __isr_tim8_cc();
    pub fn __isr_dma1_stream7();
    pub fn __isr_fsmc();
    pub fn __isr_sdio();
    pub fn __isr_tim5();
    pub fn __isr_spi3();
    pub fn __isr_uart4();
    pub fn __isr_uart5();
    pub fn __isr_tim6_dac();
    pub fn __isr_tim7();
    pub fn __isr_dma2_stream0();
    pub fn __isr_dma2_stream1();
    pub fn __isr_dma2_stream2();
    pub fn __isr_dma2_stream3();
    pub fn __isr_dma2_stream4();
    pub fn __isr_eth();
    pub fn __isr_eth_wkup();
    pub fn __isr_can2_tx();
    pub fn __isr_can2_rx0();
    pub fn __isr_can2_rx1();
    pub fn __isr_can2_sce();
    pub fn __isr_otg_fs();
    pub fn __isr_dma2_stream5();
    pub fn __isr_dma2_stream6();
    pub fn __isr_dma2_stream7();
    pub fn __isr_usart6();
    pub fn __isr_i2c3_ev();
    pub fn __isr_i2c3_er();
    pub fn __isr_otg_hs_ep1_out();
    pub fn __isr_otg_hs_ep1_in();
    pub fn __isr_otg_hs_wkup();
    pub fn __isr_otg_hs();
    pub fn __isr_dcmi();
    pub fn __isr_cryp();
    pub fn __isr_hash_rng();
    pub fn __isr_fpu();
}

#[no_mangle]
#[link_section = ".stack_end"]
pub static STACK_END: &'static u32 = unsafe{&__stack_end};

#[no_mangle]
#[link_section = ".isr_vector"]
pub static ISR_VECTOR: [Option<unsafe extern fn()>; 97] = [
    Some(__isr_reset),
    Some(__isr_nmi),
    Some(__isr_hardfault),
    Some(__isr_memmanage),
    Some(__isr_busfault),
    Some(__isr_usagefault),
    None,
    None,
    None,
    None,
    Some(__isr_svc),
    Some(__isr_debugmon),
    None,
    Some(__isr_pendsv),
    Some(__isr_systick),

    Some(__isr_wwdg),
    Some(__isr_pvd),
    Some(__isr_tamp_stamp),
    Some(__isr_rtc_wkup),
    Some(__isr_flash),
    Some(__isr_rcc),
    Some(__isr_exti0),
    Some(__isr_exti1),
    Some(__isr_exti2),
    Some(__isr_exti3),
    Some(__isr_exti4),
    Some(__isr_dma1_stream0),
    Some(__isr_dma1_stream1),
    Some(__isr_dma1_stream2),
    Some(__isr_dma1_stream3),
    Some(__isr_dma1_stream4),
    Some(__isr_dma1_stream5),
    Some(__isr_dma1_stream6),
    Some(__isr_adc),
    Some(__isr_can1_tx),
    Some(__isr_can1_rx0),
    Some(__isr_can1_rx1),
    Some(__isr_can1_sce),
    Some(__isr_exti9_5),
    Some(__isr_tim1_brk_tim9),
    Some(__isr_tim1_up_tim10),
    Some(__isr_tim1_trg_com_tim11),
    Some(__isr_tim1_cc),
    Some(__isr_tim2),
    Some(__isr_tim3),
    Some(__isr_tim4),
    Some(__isr_i2c1_ev),
    Some(__isr_i2c1_er),
    Some(__isr_i2c2_ev),
    Some(__isr_i2c2_er),
    Some(__isr_spi1),
    Some(__isr_spi2),
    Some(__isr_usart1),
    Some(__isr_usart2),
    Some(__isr_usart3),
    Some(__isr_exti15_10),
    Some(__isr_rtc_alarm),
    Some(__isr_otg_fs_wkup),
    Some(__isr_tim8_brk_tim12),
    Some(__isr_tim8_up_tim13),
    Some(__isr_tim8_trg_com_tim14),
    Some(__isr_tim8_cc),
    Some(__isr_dma1_stream7),
    Some(__isr_fsmc),
    Some(__isr_sdio),
    Some(__isr_tim5),
    Some(__isr_spi3),
    Some(__isr_uart4),
    Some(__isr_uart5),
    Some(__isr_tim6_dac),
    Some(__isr_tim7),
    Some(__isr_dma2_stream0),
    Some(__isr_dma2_stream1),
    Some(__isr_dma2_stream2),
    Some(__isr_dma2_stream3),
    Some(__isr_dma2_stream4),
    Some(__isr_eth),
    Some(__isr_eth_wkup),
    Some(__isr_can2_tx),
    Some(__isr_can2_rx0),
    Some(__isr_can2_rx1),
    Some(__isr_can2_sce),
    Some(__isr_otg_fs),
    Some(__isr_dma2_stream5),
    Some(__isr_dma2_stream6),
    Some(__isr_dma2_stream7),
    Some(__isr_usart6),
    Some(__isr_i2c3_ev),
    Some(__isr_i2c3_er),
    Some(__isr_otg_hs_ep1_out),
    Some(__isr_otg_hs_ep1_in),
    Some(__isr_otg_hs_wkup),
    Some(__isr_otg_hs),
    Some(__isr_dcmi),
    Some(__isr_cryp),
    Some(__isr_hash_rng),
    Some(__isr_fpu),
];
