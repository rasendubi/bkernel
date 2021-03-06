#![feature(lang_items, core_intrinsics, const_fn)]
#![feature(fixed_size_array)]
#![feature(alloc_error_handler)]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(target_os = "none", no_std)]

#[cfg(not(target_os = "none"))]
extern crate core;

extern crate dev;
extern crate stm32f4;

extern crate alloc;
#[cfg(target_os = "none")]
extern crate linkmem;
#[cfg(target_os = "none")]
extern crate smalloc;

#[macro_use]
extern crate futures;

extern crate breactor;

mod led;
mod led_music;
mod log;
mod terminal;

use core::pin::Pin;

use futures::future;
use futures::FutureExt;
use futures::Poll;
use futures::TryFutureExt;

use stm32f4::gpio::{GPIO_B, GPIO_D};
use stm32f4::rcc::RCC;
use stm32f4::timer::TIM2;
use stm32f4::{gpio, nvic, rcc, timer, usart};

use ::breactor::start_send_all_string::StartSendAllString;

use ::breactor::REACTOR;

use ::dev::usart::Usart;

use ::dev::htu21d::{Htu21d, Htu21dError};

use ::dev::cs43l22::Cs43l22;

use ::dev::esp8266::{AccessPoint, Esp8266};

pub static USART3: Usart<[u8; 32], [u8; 32]> =
    Usart::new(unsafe { &::stm32f4::usart::USART3 }, [0; 32], [0; 32]);

pub static mut ESP8266: Esp8266<&'static Usart<[u8; 32], [u8; 32]>> = Esp8266::new(&USART3);

pub static USART2: Usart<[u8; 128], [u8; 32]> =
    Usart::new(unsafe { &::stm32f4::usart::USART2 }, [0; 128], [0; 32]);

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
            let _ = write!(log::Logger::new(&USART2), $($x),*);
        }
    };
}

static HTU21D: Htu21d = Htu21d::new(&::dev::i2c::I2C1_BUS);

static mut CS43L22: Cs43l22 = Cs43l22::new(&::dev::i2c::I2C1_BUS, false);

#[cfg(target_os = "none")]
fn init_memory() {
    const HEAP_SIZE: usize = 64 * 1024;
    static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

    ::linkmem::init(smalloc::Smalloc {
        start: unsafe { &mut HEAP }.as_mut_ptr(),
        size: HEAP_SIZE,
    });
}

#[cfg(not(target_os = "none"))]
fn init_memory() {}

/// The main entry of the kernel.
#[no_mangle]
pub extern "C" fn kmain() -> ! {
    init_memory();
    unsafe {
        init_usart2();
        init_esp8266();
        init_leds();
        init_timer();
        init_i2c();
        init_rng();
    }

    // Test that allocator works
    let mut b = ::alloc::boxed::Box::new(5);
    unsafe {
        ::core::intrinsics::volatile_store(&mut *b as *mut _, 4);
    }

    // unsafe { &mut ::dev::rng::RNG }.enable();
    // let mut print_rng = unsafe { &mut ::dev::rng::RNG }
    //     .for_each(|r| {
    //         use core::fmt::Write;
    //         let _ = writeln!(unsafe { &::stm32f4::usart::USART2 }, "RNG: {:?}\r", r);

    //         futures::future::ready(())
    //     })
    //     .map(|_| ());

    let mut terminal = StartSendAllString::new(
        &USART2,
        "\r\nWelcome to bkernel!\r\nType 'help' to get a list of available commands.\r\n",
    )
    .and_then(|stdout| terminal::run_terminal(&USART2, stdout))
    .map(|_| ());

    let mut htu21d = HTU21D
        .soft_reset()
        .and_then(|_| {
            // This is needed because device is not instantly up after
            // reset, so we poll it, untill it ready.
            future::poll_fn(|cx| {
                match ready!(HTU21D.read_temperature_hold_master().poll_unpin(cx)) {
                    Ok(temp) => Poll::Ready(Ok(temp)),
                    // Acknowledge failure -> device is not ready -> retry
                    Err(Htu21dError::I2cError(dev::i2c::Error::AcknowledgementFailure)) => {
                        Poll::Pending
                    }
                    Err(x) => Poll::Ready(Err(x)),
                }
            })
        })
        .and_then(|temp| {
            HTU21D
                .read_humidity_hold_master()
                .map_ok(move |hum| (temp, hum))
        })
        .then(|x| {
            match x {
                Ok((temp, hum)) => log!("Temperature: {} C      Humidity: {}%\r\n", temp, hum),
                Err(err) => log!("HTU21D error: {:?}\r\n", err),
            }

            future::ready(())
        });

    let mut cs43l22 = unsafe { &mut CS43L22 }.get_chip_id().then(|res| {
        match res {
            Ok(id) => {
                log!("CS43L22 CHIP ID: 0b{:b}\r\n", id);
            }
            Err(err) => {
                log!("Error: {:?}\r\n", err);
            }
        }

        future::ready(())
    });

    let mut esp8266 = unsafe { &mut ESP8266 }
        .check_at()
        .then(|x| {
            log!("\r\nESP CHECK AT: {:?}\r\n", x);
            future::ready(Ok(()) as Result<(), ()>)
        })
        .then(|_| unsafe { &mut ESP8266 }.list_aps::<[AccessPoint; 32]>())
        .and_then(|(aps, size)| {
            debug_log!("\r\nAccess points:\r\n");
            for ap in &aps[0..::core::cmp::min(size, aps.len())] {
                debug_log!("{:?}\r\n", ap);
            }

            future::ready(Ok(()))
        })
        .then(|_| unsafe { &mut ESP8266 }.join_ap("Rotem Indiana_Guest", "snickershock"))
        .and_then(|res| {
            debug_log!("Join status: {:?}", res);
            future::ready(Ok(()))
        })
        .map_err(|err| {
            log!("\r\nESP8266 error: {:?}\r\n", err);
        })
        .map(|_| ());

    unsafe {
        let reactor = &REACTOR;

        // Trust me, I know what I'm doing with lifetime loundary here.
        //
        // The infinite loop below makes all values above it
        // effectively 'static.
        reactor.add_task(5, Pin::new_unchecked(lifetime_loundary(&mut terminal)));
        // reactor.add_task(4, Pin::new_unchecked(lifetime_loundary(&mut print_rng)));
        reactor.add_task(6, Pin::new_unchecked(lifetime_loundary(&mut htu21d)));
        reactor.add_task(2, Pin::new_unchecked(lifetime_loundary(&mut cs43l22)));
        reactor.add_task(1, Pin::new_unchecked(lifetime_loundary(&mut esp8266)));

        loop {
            reactor.run();

            // SEV is issued when any task sets readiness to true.
            stm32f4::__wait_for_event();
        }
    }
}

/// Extremely unsafe (probably even UB)
unsafe fn lifetime_loundary<'a, 'b, T: ?Sized>(val: &'a mut T) -> &'b mut T {
    &mut *(val as *mut _)
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

unsafe fn init_usart2() {
    use ::stm32f4::usart::USART2;

    RCC.apb1_clock_enable(rcc::Apb1Enable::USART2);

    // Enable the peripheral clock for the pins used by USART2, PD5
    // for TX and PD6 for RX
    RCC.ahb1_clock_enable(rcc::Ahb1Enable::GPIOD);

    // This sequence sets up the TX and RX pins so they work correctly
    // with the USART2 peripheral
    GPIO_D.enable(
        5,
        gpio::GpioConfig {
            mode: gpio::GpioMode::AF,
            ospeed: gpio::GpioOSpeed::FAST_SPEED,
            otype: gpio::GpioOType::PUSH_PULL,
            pupd: gpio::GpioPuPd::PULL_UP,
            af: gpio::GpioAF::AF7,
        },
    );
    GPIO_D.enable(
        6,
        gpio::GpioConfig {
            mode: gpio::GpioMode::AF,
            ospeed: gpio::GpioOSpeed::FAST_SPEED,
            otype: gpio::GpioOType::PUSH_PULL,
            pupd: gpio::GpioPuPd::PULL_UP,
            af: gpio::GpioAF::AF7,
        },
    );

    // The RX and TX pins are now connected to their AF so that the
    // USART2 can take over control of the pins
    USART2.enable(&usart::UsartConfig {
        data_bits: usart::DataBits::Bits8,
        stop_bits: usart::StopBits::Bits1,
        flow_control: usart::FlowControl::No,
        baud_rate: 115_200,
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
    use core::fmt::Write;
    use core::panic::PanicInfo;
    use stm32f4::usart::USART2;

    // #[lang = "panic_fmt"]
    #[panic_handler]
    fn panic(info: &PanicInfo) -> ! {
        {
            let _lock = unsafe { ::stm32f4::IrqLock::new() };
            match info.location() {
                Some(loc) => {
                    let _ = write!(
                        unsafe { &USART2 },
                        "\r\nPANIC\r\n{}:{}",
                        loc.file(),
                        loc.line()
                    );
                }
                None => {
                    let _ = write!(unsafe { &USART2 }, "\r\nPANIC\r\n");
                }
            }
        }
        loop {
            unsafe { ::stm32f4::__wait_for_interrupt() };
        }
    }

    #[alloc_error_handler]
    fn alloc_error(_: core::alloc::Layout) -> ! {
        {
            let _lock = unsafe { ::stm32f4::IrqLock::new() };
            let _ = write!(unsafe { &USART2 }, "\r\nALLOC ERROR\r\n");
        }

        loop {
            unsafe { ::stm32f4::__wait_for_interrupt() };
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn __isr_tim2() {
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
    GPIO_D.enable(
        4,
        gpio::GpioConfig {
            mode: gpio::GpioMode::OUTPUT,
            ospeed: gpio::GpioOSpeed::FAST_SPEED,
            otype: gpio::GpioOType::PUSH_PULL,
            pupd: gpio::GpioPuPd::PULL_DOWN,
            af: gpio::GpioAF::AF0,
        },
    );

    GPIO_D.set_bit(4);

    rcc::RCC.ahb1_clock_enable(rcc::Ahb1Enable::GPIOB);

    GPIO_B.enable(
        6,
        gpio::GpioConfig {
            mode: gpio::GpioMode::AF,
            ospeed: gpio::GpioOSpeed::FAST_SPEED,
            otype: gpio::GpioOType::OPEN_DRAIN,
            pupd: gpio::GpioPuPd::NO,
            af: gpio::GpioAF::AF4,
        },
    );
    GPIO_B.enable(
        9,
        gpio::GpioConfig {
            mode: gpio::GpioMode::AF,
            ospeed: gpio::GpioOSpeed::FAST_SPEED,
            otype: gpio::GpioOType::OPEN_DRAIN,
            pupd: gpio::GpioPuPd::NO,
            af: gpio::GpioAF::AF4,
        },
    );

    rcc::RCC.apb1_clock_enable(rcc::Apb1Enable::I2C1);
    i2c::I2C1.init(&i2c::I2cInit {
        clock_speed: 10000,
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

unsafe fn init_esp8266() {
    use ::stm32f4::usart::USART3;

    RCC.apb1_clock_enable(rcc::Apb1Enable::USART3);

    // Enable the peripheral clock for the pins used by USART3, PD8
    // for TX and PD9 for RX
    RCC.ahb1_clock_enable(rcc::Ahb1Enable::GPIOD);

    GPIO_D.enable(
        8,
        gpio::GpioConfig {
            mode: gpio::GpioMode::AF,
            ospeed: gpio::GpioOSpeed::FAST_SPEED,
            otype: gpio::GpioOType::PUSH_PULL,
            pupd: gpio::GpioPuPd::PULL_UP,
            af: gpio::GpioAF::AF7,
        },
    );
    GPIO_D.enable(
        9,
        gpio::GpioConfig {
            mode: gpio::GpioMode::AF,
            ospeed: gpio::GpioOSpeed::FAST_SPEED,
            otype: gpio::GpioOType::PUSH_PULL,
            pupd: gpio::GpioPuPd::PULL_UP,
            af: gpio::GpioAF::AF7,
        },
    );

    // The RX and TX pins are now connected to their AF so that the
    // USART3 can take over control of the pins
    USART3.enable(&usart::UsartConfig {
        data_bits: usart::DataBits::Bits8,
        stop_bits: usart::StopBits::Bits1,
        flow_control: usart::FlowControl::No,
        baud_rate: 115_200,
    });

    USART3.it_enable(usart::Interrupt::RXNE);

    nvic::init(&nvic::NvicInit {
        irq_channel: nvic::IrqChannel::USART3,
        priority: 0,
        subpriority: 4,
        enable: true,
    });
}

#[no_mangle]
pub unsafe extern "C" fn __isr_usart2() {
    USART2.isr()
}

#[no_mangle]
pub unsafe extern "C" fn __isr_usart3() {
    USART3.isr()
}
