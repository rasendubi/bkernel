//! This crate is a Rust part of the kernel. It should be linked with
//! the bootstrap that will jump to the `kmain` function.
#![feature(lang_items, alloc, core_intrinsics, const_fn)]
#![feature(conservative_impl_trait)]
#![feature(integer_atomics)]

#![feature(compiler_builtins_lib)]

#![cfg_attr(target_os = "none", no_std)]

#[cfg(not(target_os = "none"))]
extern crate core;

extern crate stm32f4;
extern crate dev;

#[cfg(target_os = "none")]
extern crate smalloc;
#[cfg(target_os = "none")]
extern crate linkmem;
extern crate alloc;

#[macro_use]
extern crate futures;

extern crate breactor;

extern crate compiler_builtins;
pub use compiler_builtins::*;

mod led;
mod led_music;
mod terminal;
mod start_send_all;
mod start_send_all_string;
mod log;
mod lock_free;

use stm32f4::{rcc, gpio, usart, timer, nvic};
use stm32f4::rcc::RCC;
use stm32f4::gpio::{GPIO_B, GPIO_D};
use stm32f4::usart::USART2;
use stm32f4::timer::TIM2;

use futures::{Future, Stream};
use futures::future::{self, Loop};

use start_send_all_string::StartSendAllString;

pub use log::__isr_usart2;

use breactor::REACTOR;

use ::dev::htu21d::{Htu21d, Htu21dError};

macro_rules! debug_log {
    ( $( $x:expr ),* ) => {
        {
            use ::core::fmt::Write;
            let _lock = unsafe { ::stm32f4::IrqLock::new() };

            let _ = write!(unsafe{&::stm32f4::usart::USART2}, $($x),*);
        }
    };
}

macro_rules! log {
    ( $( $x:expr ),* ) => {
        {
            use ::core::fmt::Write;
            let _ = write!(unsafe{&mut log::STDOUT}, $($x),*);
        }
    };
}

static HTU21D: Htu21d = Htu21d::new(&::dev::i2c::I2C1_BUS);

#[cfg(target_os = "none")]
fn init_memory() {
    const HEAP_SIZE: usize = 64*1024;
    static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

    ::linkmem::init(smalloc::Smalloc {
        start: unsafe {&mut HEAP}.as_mut_ptr(),
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
        init_rng();
    }

    // Test that allocator works
    let mut b = ::alloc::boxed::Box::new(5);
    unsafe { ::core::intrinsics::volatile_store(&mut *b as *mut _, 4); }

    unsafe{&mut ::dev::rng::RNG}.enable();
    let mut print_rng = unsafe{&mut ::dev::rng::RNG}.for_each(|r| {
        use core::fmt::Write;
        let _ = write!(unsafe{&::stm32f4::usart::USART1}, "RNG: {}\n", r);
        Ok(())
    })
        .map(|_| ())
        .map_err(|_| ());

    let stdin = unsafe {&mut log::STDIN};
    let stdout = unsafe {&mut log::STDOUT};

    let mut terminal = StartSendAllString::new(
        stdout,
        "\r\nWelcome to bkernel!\r\nType 'help' to get a list of available commands.\r\n"
    ).and_then(|stdout| terminal::run_terminal(stdin, stdout))
        .map(|_| ())
        .map_err(|_| ());

    let mut htu21d = HTU21D.soft_reset()
        .and_then(|_| {
            // This is needed because device is not instantly up after
            // reset, so we poll it, untill it ready.
            future::loop_fn(
                (),
                |_| {
                    HTU21D.read_temperature_hold_master()
                        .then(|res| match res {
                            Ok(temp) => {
                                Ok(Loop::Break(temp))
                            },
                            // Acknowledge failure -> device is not ready -> retry
                            Err(Htu21dError::I2cError(dev::i2c::Error::AcknowledgementFailure)) => {
                                Ok(Loop::Continue(()))
                            },
                            Err(x) => {
                                Err(x)
                            },
                        })
                }
            )
        })
        .and_then(|temp| {
            HTU21D.read_humidity_hold_master()
                .map(move |hum| {
                    log!("Temperature: {} C      Humidity: {}%\r\n",
                         temp, hum);
                    ()
                })
        })
        .map_err(|err| {
            log!("HTU21D error: {:?}\r\n", err);
            ()
        });

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
        reactor.add_task(
            4,
            ::core::mem::transmute::<&mut Future<Item=(), Error=()>,
                                     &'static mut Future<Item=(), Error=()>>(&mut print_rng)
        );
        reactor.add_task(
            6,
            ::core::mem::transmute::<&mut Future<Item=(), Error=()>,
                                     &'static mut Future<Item=(), Error=()>>(&mut htu21d)
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
    RCC.apb1_clock_enable(rcc::Apb1Enable::USART2);

    // Enable the peripheral clock for the pins used by USART2, PD5
    // for TX and PD6 for RX
    RCC.ahb1_clock_enable(rcc::Ahb1Enable::GPIOD);

    // This sequence sets up the TX and RX pins so they work correctly
    // with the USART2 peripheral
    GPIO_D.enable(5, gpio::GpioConfig {
        mode: gpio::GpioMode::AF,
        ospeed: gpio::GpioOSpeed::FAST_SPEED,
        otype: gpio::GpioOType::PUSH_PULL,
        pupd: gpio::GpioPuPd::PULL_UP,
        af: gpio::GpioAF::AF7,
    });
    GPIO_D.enable(6, gpio::GpioConfig {
        mode: gpio::GpioMode::AF,
        ospeed: gpio::GpioOSpeed::FAST_SPEED,
        otype: gpio::GpioOType::PUSH_PULL,
        pupd: gpio::GpioPuPd::PULL_UP,
        af: gpio::GpioAF::AF7,
    });

    // The RX and TX pins are now connected to their AF so that the
    // USART2 can take over control of the pins
    USART2.enable(&usart::UsartConfig {
        data_bits: usart::DataBits::Bits8,
        stop_bits: usart::StopBits::Bits1,
        flow_control: usart::FlowControl::No,
        baud_rate: 115200,
    });

    USART2.it_enable(usart::Interrupt::RXNE);

    nvic::init(&nvic::NvicInit {
        irq_channel: nvic::IrqChannel::USART2,
        priority: 0,
        subpriority: 1,
        enable: true,
    });
}

#[cfg(target_os = "none")]
pub mod panicking {
    use core::fmt::{self, Write};
    use stm32f4::usart::USART2;

    #[lang = "panic_fmt"]
    extern fn panic_fmt(fmt: fmt::Arguments, file: &str, line: u32) -> ! {
        {
            let _lock = unsafe { ::stm32f4::IrqLock::new() };
            let _ = write!(unsafe{&USART2}, "\r\nPANIC\r\n{}:{} {}", file, line, fmt);
        }
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

    rcc::RCC.ahb1_clock_enable(rcc::Ahb1Enable::GPIOD);
    GPIO_D.enable(4, gpio::GpioConfig {
        mode: gpio::GpioMode::OUTPUT,
        ospeed: gpio::GpioOSpeed::FAST_SPEED,
        otype: gpio::GpioOType::PUSH_PULL,
        pupd: gpio::GpioPuPd::PULL_DOWN,
        af: gpio::GpioAF::AF0,
    });

    rcc::RCC.ahb1_clock_enable(rcc::Ahb1Enable::GPIOB);

    GPIO_B.enable(6, gpio::GpioConfig {
        mode: gpio::GpioMode::AF,
        ospeed: gpio::GpioOSpeed::FAST_SPEED,
        otype: gpio::GpioOType::OPEN_DRAIN,
        pupd: gpio::GpioPuPd::NO,
        af: gpio::GpioAF::AF4,
    });
    GPIO_B.enable(9, gpio::GpioConfig {
        mode: gpio::GpioMode::AF,
        ospeed: gpio::GpioOSpeed::FAST_SPEED,
        otype: gpio::GpioOType::OPEN_DRAIN,
        pupd: gpio::GpioPuPd::NO,
        af: gpio::GpioAF::AF4,
    });

    rcc::RCC.apb1_clock_enable(rcc::Apb1Enable::I2C1);
    i2c::I2C1.init(&i2c::I2cInit {
        clock_speed: 100000,
        mode: i2c::Mode::I2C,
        duty_cycle: i2c::DutyCycle::DutyCycle_2,
        own_address1: 0,
        ack: i2c::Acknowledgement::Disable,
        acknowledged_address: i2c::AcknowledgedAddress::Bit7,
    });

    nvic::init(&nvic::NvicInit {
        irq_channel: nvic::IrqChannel::I2C1_EV,
        priority: 4,
        subpriority: 1,
        enable: true,
    });
    nvic::init(&nvic::NvicInit {
        irq_channel: nvic::IrqChannel::I2C1_ER,
        priority: 4,
        subpriority: 1,
        enable: true,
    });
}

unsafe fn init_rng() {
    rcc::RCC.ahb2_clock_enable(rcc::Ahb2Enable::RNG);

    nvic::init(&nvic::NvicInit {
        irq_channel: nvic::IrqChannel::HASH_RNG,
        priority: 4,
        subpriority: 1,
        enable: true,
    });
}
