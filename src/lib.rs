//! This crate is a Rust part of the kernel. It should be linked with
//! the bootstrap that will jump to the `kmain` function.
#![feature(lang_items, alloc, core_intrinsics, collections, const_fn)]

#![cfg_attr(target_os = "none", no_std)]

#[cfg(not(target_os = "none"))]
extern crate core;

#[cfg(target_os = "none")]
extern crate linkmem;

extern crate stm32f4;
extern crate smalloc;
extern crate alloc;
extern crate collections;
extern crate bscheduler;

#[cfg(target_os = "none")]
mod global;

#[cfg(not(target_os = "none"))]
pub mod global;

mod led;
mod led_music;
mod terminal;
mod scheduler;
mod queue;
mod log;

use stm32f4::{rcc, gpio, usart, timer, nvic};
use stm32f4::rcc::RCC;
use stm32f4::gpio::GPIO_B;
use stm32f4::usart::USART1;
use stm32f4::timer::TIM2;

#[cfg(target_os = "none")]
const MEMORY_SIZE: usize = 64*1024;

#[cfg(target_os = "none")]
static mut MEMORY: [u8; MEMORY_SIZE] = [0; MEMORY_SIZE];

#[cfg(target_os = "none")]
fn init_memory() {
    ::linkmem::init(smalloc::Smalloc {
        start: unsafe { ::core::mem::transmute(&mut MEMORY) },
        size: MEMORY_SIZE,
    });
}

#[cfg(not(target_os = "none"))]
fn init_memory() {}

static mut STARTUP_TASK: scheduler::Task<'static> = unsafe { scheduler::Task::new(
    "terminal",
    5,
    &mut || {
        log::write_str("\r\nWelcome to bkernel!\r\n");
        log::write_str("Type 'help' to get a list of available commands.\r\n");

        terminal::run_terminal();
    }) };

/// The main entry of the kernel.
#[no_mangle]
pub extern fn kmain() -> ! {
    init_memory();
    init_usart1();
    init_leds();
    init_timer();
    log::init();

    scheduler::init();

    // Test that allocator works
    let mut b = ::alloc::boxed::Box::new(5);
    unsafe { ::core::intrinsics::volatile_store(&mut *b as *mut _, 4); }

    scheduler::add_task(unsafe { &mut STARTUP_TASK });

    scheduler::schedule();
}

fn init_timer() {
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
        USART1.puts_synchronous("\r\nPANIC\r\n");
        let _ = write!(UsartProxy(&USART1), "{}:{} {}", file, line, fmt);
        loop {}
    }
}

#[no_mangle]
pub unsafe extern fn __isr_tim2() {
    static mut led3_value: bool = false;

    if TIM2.it_status(timer::Dier::UIE) {
        TIM2.it_clear_pending(timer::Dier::UIE);

        led3_value = !led3_value;
        if led3_value {
            led::LD3.turn_on();
        } else {
            led::LD3.turn_off();
        }
    }
}

#[no_mangle]
pub unsafe extern fn __isr_usart1() {
    if USART1.it_status(usart::Interrupt::RXNE) {
        let c = USART1.get_unsafe();
        terminal::put_char(c);
    }

    if USART1.it_status(usart::Interrupt::TXE) {
        log::usart1_txe();
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn _Unwind_Resume(_ex_obj: *mut ()) {
}
