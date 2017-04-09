//! This crate is a Rust part of the kernel. It should be linked with
//! the bootstrap that will jump to the `kmain` function.
#![feature(lang_items, alloc, core_intrinsics, const_fn)]
#![feature(conservative_impl_trait)]

#![cfg_attr(target_os = "none", no_std)]

#[cfg(not(target_os = "none"))]
extern crate core;

#[cfg(target_os = "none")]
extern crate linkmem;

extern crate stm32f4;
extern crate smalloc;
extern crate alloc;
#[macro_use]
extern crate futures;

mod led;
mod led_music;
mod terminal;
mod start_send_all;
mod start_send_all_string;
mod log;

use stm32f4::{rcc, gpio, usart, timer, nvic};
use stm32f4::rcc::RCC;
use stm32f4::gpio::GPIO_B;
use stm32f4::usart::USART1;
use stm32f4::timer::TIM2;

use futures::stream::Stream;
use futures::{Async, Future};

pub use log::__isr_usart1;

#[cfg(target_os = "none")]
const HEAP_SIZE: usize = 64*1024;

#[cfg(target_os = "none")]
static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

#[cfg(target_os = "none")]
fn init_memory() {
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
    }

    // Test that allocator works
    let mut b = ::alloc::boxed::Box::new(5);
    unsafe { ::core::intrinsics::volatile_store(&mut *b as *mut _, 4); }

    let f = ::futures::stream::iter("\nWelcome to bkernel!\nType 'help' to get a list of available commands.".as_bytes().into_iter().map(|x| Ok(*x) as Result<u8, ()>)).forward(unsafe{&mut log::LOGGER});

    let mut f = f.and_then(|(_stream, sink)| terminal::run_terminal(unsafe { &mut log::INPUT },
                                                                    sink));

    loop {
        match f.poll() {
            Ok(Async::NotReady) => {
                continue;
            }
            _ => {
                break;
            }
        }
    }

    loop { }
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
        baud_rate: 9600,
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
    use core::fmt;
    use stm32f4::usart::{USART1, UsartProxy};

    #[lang = "panic_fmt"]
    extern fn panic_fmt(fmt: fmt::Arguments, file: &str, line: u32) -> ! {
        use core::fmt::Write;
        unsafe{&USART1}.puts_synchronous("\r\nPANIC\r\n");
        let _ = write!(UsartProxy(unsafe{&USART1}), "{}:{} {}", file, line, fmt);
        loop {}
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
