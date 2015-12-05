//! This crate is a Rust part of the kernel. It should be linked with
//! the bootstrap that will jump to the `kmain` function.
#![crate_type = "staticlib"]

#![feature(lang_items)]
#![feature(no_std)]
#![no_std]

extern crate stm32f4;

mod led;
mod terminal;

use stm32f4::{rcc, gpio, usart};
use stm32f4::rcc::RCC;
use stm32f4::gpio::GPIO_B;
use stm32f4::usart::USART1;

/// The main entry of the kernel.
#[no_mangle]
pub extern fn kmain() -> ! {
    init_usart1();
    init_leds();

    USART1.puts_synchronous("\r\nWelcome to bkernel!\r\n");
    USART1.puts_synchronous("Type 'help' to get a list of available commands.\r\n");

    terminal::run_terminal(&USART1);

    loop {}
}

fn init_leds() {
    RCC.ahb1_clock_enable(rcc::Ahb1Enable::GPIOD);
    led::LD3.init();
    led::LD4.init();
    led::LD5.init();
    led::LD6.init();

    led::LD3.turn_on();
    led::LD4.turn_on();
    led::LD5.turn_on();
    led::LD6.turn_on();
}

fn init_usart1() {
    RCC.apb2_clock_enable(rcc::Apb2Enable::USART1);

    /* enable the peripheral clock for the pins used by
     * USART1, PB6 for TX and PB7 for RX
     */
    RCC.ahb1_clock_enable(rcc::Ahb1Enable::GPIOB);

    /* This sequence sets up the TX pin
     * so they work correctly with the USART1 peripheral
     */
    GPIO_B.enable(6, gpio::GpioConfig {
        mode: gpio::GpioMode::AF,
        ospeed: gpio::GpioOSpeed::FAST_SPEED,
        otype: gpio::GpioOType::OPEN_DRAIN,
        pupd: gpio::GpioPuPd::PULL_UP,
        af: gpio::GpioAF::AF7,
    });
    GPIO_B.enable(7, gpio::GpioConfig {
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
    USART1.enable(&usart::UsartConfig {
        data_bits: usart::DataBits::Bits8,
        stop_bits: usart::StopBits::Bits1,
        flow_control: usart::FlowControl::No,
        baud_rate: 9600,
    });
}

#[cfg(target_os = "none")]
pub mod panicking {
    use core::fmt;
    use stm32f4::usart::{USART1, UsartProxy};

    #[lang = "panic_fmt"]
    extern fn panic_fmt(fmt: fmt::Arguments, file: &str, line: u32) -> ! {
        use core::fmt::Write;
        USART1.puts_synchronous("\r\nPANIC\r\n");
        let _ = write!(UsartProxy(&USART1), "{}:{} {}", file, line, fmt);
        loop {}
    }
}
