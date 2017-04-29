//! This crate is a Rust part of the kernel. It should be linked with
//! the bootstrap that will jump to the `kmain` function.
#![feature(lang_items, alloc, core_intrinsics, const_fn)]
#![feature(conservative_impl_trait)]
#![feature(integer_atomics)]

#![cfg_attr(target_os = "none", no_std)]

#[cfg(not(target_os = "none"))]
extern crate core;

extern crate stm32f4;
extern crate dev;

extern crate smalloc;
#[cfg(target_os = "none")]
extern crate linkmem;
extern crate alloc;

#[macro_use]
extern crate futures;

extern crate breactor;

mod led;
mod led_music;
mod terminal;
mod start_send_all;
mod start_send_all_string;
mod log;
mod lock_free;

use stm32f4::{rcc, gpio, usart, timer, nvic};
use stm32f4::rcc::RCC;
use stm32f4::gpio::GPIO_B;
use stm32f4::usart::USART1;
use stm32f4::timer::TIM2;

use futures::Future;

use start_send_all_string::StartSendAllString;

pub use log::__isr_usart1;

use breactor::Reactor;

static REACTOR: Reactor = Reactor::new();

#[cfg(target_os = "none")]
fn init_memory() {
    const HEAP_SIZE: usize = 64*1024;
    static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

    ::linkmem::init(smalloc::Smalloc {
        start: unsafe { ::core::mem::transmute(&mut HEAP) },
        size: HEAP_SIZE,
    });
}

#[cfg(not(target_os = "none"))]
fn init_memory() {}

/// The main entry of the kernel.
#[no_mangle]
pub extern fn kmain() -> ! {
    init_memory();
    unsafe {
        init_usart1();
        init_leds();
        init_timer();
        init_i2c();
    }

    // Test that allocator works
    let mut b = ::alloc::boxed::Box::new(5);
    unsafe { ::core::intrinsics::volatile_store(&mut *b as *mut _, 4); }

    let stdin = unsafe {&mut log::STDIN};
    let stdout = unsafe {&mut log::STDOUT};

    let mut terminal = StartSendAllString::new(
        stdout,
        "\r\nWelcome to bkernel!\r\nType 'help' to get a list of available commands.\r\n"
    ).and_then(|stdout| terminal::run_terminal(stdin, stdout))
        .map(|_| ())
        .map_err(|_| ());

    unsafe {
        let reactor = &REACTOR;

        reactor.add_task(
            5,
            // Trust me, I know what I'm doing.
            //
            // The infinite loop below makes all values above it
            // effectively 'static.
            ::core::mem::transmute::<&mut Future<Item=(), Error=()>,
                                     &'static mut Future<Item=(), Error=()>>(&mut terminal)
        );

        loop {
            reactor.run();

            // SEV is issued when any task sets readiness to true.
            stm32f4::__wait_for_event();
        }
    }
}

unsafe fn init_timer() {
    RCC.apb1_clock_enable(rcc::Apb1Enable::TIM2);

    TIM2.init(&timer::TimInit {
        prescaler: 40000,
        counter_mode: timer::CounterMode::Up,
        period: 128,
        clock_division: timer::ClockDivision::Div1,
        repetition_counter: 0,
    });

    TIM2.it_enable(timer::Dier::UIE);

    TIM2.enable();

    nvic::init(&nvic::NvicInit {
        irq_channel: nvic::IrqChannel::TIM2,
        priority: 0,
        subpriority: 1,
        enable: true,
    });
}

unsafe fn init_leds() {
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

unsafe fn init_usart1() {
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
        baud_rate: 115200,
    });

    USART1.it_enable(usart::Interrupt::RXNE);

    nvic::init(&nvic::NvicInit {
        irq_channel: nvic::IrqChannel::USART1,
        priority: 0,
        subpriority: 1,
        enable: true,
    });
}

#[cfg(target_os = "none")]
pub mod panicking {
    use core::fmt::{self, Write};
    use stm32f4::usart::USART1;

    #[lang = "panic_fmt"]
    extern fn panic_fmt(fmt: fmt::Arguments, file: &str, line: u32) -> ! {
        let _ = write!(unsafe{&USART1}, "\r\nPANIC\r\n{}:{} {}", file, line, fmt);
        loop {
            unsafe { ::stm32f4::__wait_for_interrupt() };
        }
    }
}

#[no_mangle]
pub unsafe extern fn __isr_tim2() {
    static mut LED3_VALUE: bool = false;

    if TIM2.it_status(timer::Dier::UIE) {
        TIM2.it_clear_pending(timer::Dier::UIE);

        LED3_VALUE = !LED3_VALUE;
        if LED3_VALUE {
            led::LD3.turn_on();
        } else {
            led::LD3.turn_off();
        }
    }
}

unsafe fn init_i2c() {
    use stm32f4::i2c;

    i2c::I2C1.init(&i2c::I2C_INIT);
}
