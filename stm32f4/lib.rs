//! STM32F4xx drivers.
#![crate_name = "stm32f4"]
#![crate_type = "lib"]

#![feature(lang_items)]
#![feature(no_std)]
#![feature(core_intrinsics, core_str_ext)]

#![no_std]

#![allow(dead_code)]

#[macro_use]
pub mod volatile;
pub mod rcc;
pub mod gpio;

pub mod lang_items;

use core::str::*;

const USART1_BASE: u32 = 0x40011000;
registers! {
    USART1_BASE, u32 => {
        USART1_SR  = 0x0,
        USART1_DR  = 0x4,
        USART1_BRR = 0x8,
        USART1_CR1 = 0xC,
        USART1_CR2 = 0x10,
        USART1_CR3 = 0x14
    }
}

pub fn puts(s: &str) {
    for c in s.bytes() {
        unsafe {
            while USART1_SR.get() & 0x40 == 0 {}
            USART1_DR.set(c as u32);
        }
    }
}

pub fn init_usart1() {
    unsafe {
        rcc::apb2_clock_enable(rcc::Apb2Enable::USART1);

        /* enable the peripheral clock for the pins used by
         * USART1, PB6 for TX and PB7 for RX
         */
        rcc::ahb1_clock_enable(rcc::Ahb1Enable::GPIOB);

        /* This sequence sets up the TX pin
         * so they work correctly with the USART1 peripheral
         */
        gpio::GPIO_B.enable(6, gpio::GpioConfig {
            mode: gpio::GpioMode::AF,
            ospeed: gpio::GpioOSpeed::FAST_SPEED,
            otype: gpio::GpioOType::OPEN_DRAIN,
            pupd: gpio::GpioPuPd::PULL_UP,
            af: gpio::GpioAF::AF7,
        });

        /* The RX and TX pins are now connected to their AF
         * so that the USART1 can take over control of the
         * pins
         */
        USART1_CR2.set(USART1_CR2.get() & !(0x3 << 12)); // 1 stop-bit
        USART1_CR1.set(USART1_CR1.get() & !(0x1 << 12 | 0x1 << 10) | (0x1 << 3)); // 8N + enable transmitter
        USART1_CR3.set(USART1_CR3.get() & !0x3FF); // No Hardware Flow-Control
        USART1_BRR.set(0x683); // 9600 baud-rate

        // finally this enables the complete USART1 peripheral
        USART1_CR1.set(USART1_CR1.get() | (1 << 13));
    }
}
