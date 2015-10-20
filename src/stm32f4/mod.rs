#![allow(dead_code)]

mod rcc;

use core::str::*;

const GPIOB_BASE: u32 = 0x40020400;
registers! {
    GPIOB_BASE, u32 => {
        GPIOB_MODER   = 0x0,
        GPIOB_TYPER   = 0x4,
        GPIOB_OSPEEDR = 0x8,
        GPIOB_PUPDR   = 0xC,
        GPIOB_AFRL    = 0x20
    }
}

const AF_MODE: u32 = 0x2;

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
        // RCC_APB2ENR.set(RCC_APB2ENR.get() | USART1_EN);

        /* enable the peripheral clock for the pins used by 
         * USART1, PB6 for TX and PB7 for RX
         */
        rcc::ahb1_clock_enable(rcc::Ahb1Enable::GPIOB);
        // RCC_AHB1ENR.set(RCC_AHB1ENR.get() | GPIOB_EN);

        /* This sequence sets up the TX and RX pins 
         * so they work correctly with the USART1 peripheral
         */
        const PIN: u32 = 6;
        GPIOB_MODER.set(GPIOB_MODER.get() & !(0x3 << (PIN*2)) | (AF_MODE << (PIN*2)));
        GPIOB_OSPEEDR.set(GPIOB_OSPEEDR.get() & !(0x3 << (PIN*2)) | (0x2 << (PIN*2)));
        GPIOB_TYPER.set(GPIOB_TYPER.get() & !(1 << PIN));
        GPIOB_PUPDR.set(GPIOB_PUPDR.get() & !(0x3 << (PIN*2)) | (1 << (PIN*2)));

        /* The RX and TX pins are now connected to their AF
         * so that the USART1 can take over control of the 
         * pins
         */
        GPIOB_AFRL.set(GPIOB_AFRL.get() & !(0xf << (PIN*4)) | (0x7 << (PIN*4)));

        USART1_CR2.set(USART1_CR2.get() & !(0x3 << 12)); // 1 stop-bit
        USART1_CR1.set(USART1_CR1.get() & !(0x1 << 12 | 0x1 << 10) | (0x1 << 3)); // 8N + enable transmitter
        USART1_CR3.set(USART1_CR3.get() & !0x3FF); // No Hardware Flow-Control
        USART1_BRR.set(0x683); // 9600 baud-rate
        
        // finally this enables the complete USART1 peripheral
        USART1_CR1.set(USART1_CR1.get() | (1 << 13));
    }
}
