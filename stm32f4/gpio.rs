//! General-Purpose Input/Output driver

#![allow(non_camel_case_types)]

use volatile::RW;

extern {
    pub static GPIO_A: Gpio;
    pub static GPIO_B: Gpio;
    pub static GPIO_C: Gpio;
    pub static GPIO_D: Gpio;
    pub static GPIO_E: Gpio;
    pub static GPIO_F: Gpio;
    pub static GPIO_G: Gpio;
    pub static GPIO_H: Gpio;
    pub static GPIO_I: Gpio;
    pub static GPIO_J: Gpio;
    pub static GPIO_K: Gpio;
}

#[repr(C)]
pub struct Gpio {
    moder:   RW<u32>, // 0x0
    otyper:  RW<u32>, // 0x4
    ospeedr: RW<u32>, // 0x8
    pupdr:   RW<u32>, // 0xC
    idr:     RW<u32>, // 0x10
    odr:     RW<u32>, // 0x14
    bsrr:    RW<u32>, // 0x18
    lckr:    RW<u32>, // 0x1C
    afrl:    RW<u32>, // 0x20
    afrh:    RW<u32>, // 0x24
}

#[test]
fn test_register_size() {
    assert_eq!(0x28, ::core::mem::size_of::<Gpio>());
}

pub struct GpioConfig {
    pub mode: GpioMode,
    pub otype: GpioOType,
    pub ospeed: GpioOSpeed,
    pub pupd: GpioPuPd,
    pub af: GpioAF,
}

#[derive(Copy, Clone)]
#[repr(u32)]
pub enum GpioMode {
    INPUT  = 0x0,
    OUTPUT = 0x1,
    AF     = 0x2,
    ANALOG = 0x3,
}

#[derive(Copy, Clone)]
#[repr(u32)]
pub enum GpioOType {
    PUSH_PULL  = 0x0,
    OPEN_DRAIN = 0x1,
}

#[derive(Copy, Clone)]
#[repr(u32)]
pub enum GpioOSpeed {
    LOW_SPEED    = 0x0,
    MEDIUM_SPEED = 0x1,
    FAST_SPEED   = 0x2,
    HIGH_SPEED   = 0x3,
}

#[derive(Copy, Clone)]
#[repr(u32)]
pub enum GpioPuPd {
    NO        = 0x0,
    PULL_UP   = 0x1,
    PULL_DOWN = 0x2,
}

/// Alternate Function
#[derive(Copy, Clone)]
#[repr(u32)]
pub enum GpioAF {
    AF0  = 0x0,
    AF1  = 0x1,
    AF2  = 0x2,
    AF3  = 0x3,
    AF4  = 0x4,
    AF5  = 0x5,
    AF6  = 0x6,
    AF7  = 0x7,
    AF8  = 0x8,
    AF9  = 0x9,
    AF10 = 0xA,
    AF11 = 0xB,
    AF12 = 0xC,
    AF13 = 0xD,
    AF14 = 0xE,
    AF15 = 0xF,
}

impl Gpio {
    /// Enables a given pin on GPIO. Pins are numbered starting from 0.
    ///
    /// # Examples
    ///
    /// Enable PB6 with Alternate Function 7 (USART1), fast speed, open-drain.
    ///
    /// ```no_run
    /// use stm32f4::gpio;
    ///
    /// gpio::GPIO_B.enable(6, gpio::GpioConfig {
    ///     mode: gpio::GpioMode::AF,
    ///     ospeed: gpio::GpioOSpeed::FAST_SPEED,
    ///     otype: gpio::GpioOType::OPEN_DRAIN,
    ///     pupd: gpio::GpioPuPd::PULL_UP,
    ///     af: gpio::GpioAF::AF7,
    /// });
    /// ```
    pub fn enable(&self, pin: u32, config: GpioConfig) {
        unsafe {
            self.ospeedr.update_with_mask(0x3 << pin*2, (config.ospeed as u32) << pin*2);
            self.otyper.update_with_mask(0x1 << pin, (config.otype as u32) << pin);
            self.pupdr.update_with_mask(0x3 << pin*2, (config.pupd as u32) << pin*2);

            if pin < 8 {
                self.afrl.update_with_mask(0xf << (pin*4), (config.af as u32) << pin*4);
            } else {
                self.afrh.update_with_mask(0xf << (pin-8)*4, (config.af as u32) << (pin-8)*4);
            }
            self.moder.update_with_mask(0x3 << pin*2, (config.mode as u32) << pin*2);
        }
    }

    pub fn set_bit(&self, pin: u32) {
        unsafe {
            self.bsrr.set(0x1 << pin);
        }
    }

    pub fn clear_bit(&self, pin: u32) {
        unsafe {
            self.bsrr.set(0x1 << (pin + 16));
        }
    }
}
