//! General-Purpose Input/Output driver

#![allow(non_camel_case_types)]

use volatile::Volatile;

pub const GPIO_B: GPIO = GPIO { base: 0x40020400 };

pub struct GPIO {
    base: usize,
}

pub struct GpioConfig {
    pub mode: GpioMode,
    pub otype: GpioOType,
    pub ospeed: GpioOSpeed,
    pub pupd: GpioPuPd,
    pub af: GpioAF,
}

pub enum GpioMode {
    INPUT  = 0x0,
    OUTPUT = 0x1,
    AF     = 0x2,
    ANALOG = 0x3,
}

pub enum GpioOType {
    PUSH_PULL  = 0x0,
    OPEN_DRAIN = 0x1,
}

pub enum GpioOSpeed {
    LOW_SPEED    = 0x0,
    MEDIUM_SPEED = 0x1,
    FAST_SPEED   = 0x2,
    HIGH_SPEED   = 0x3,
}

pub enum GpioPuPd {
    NO        = 0x0,
    PULL_UP   = 0x1,
    PULL_DOWN = 0x2,
}

/// Alternate Function
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

unsafe fn update_with_mask(reg: Volatile<u32>, mask: u32, value: u32) {
    reg.set(reg.get() & !mask | value);
}

impl GPIO {
    /// Enables a given pin on GPIO. Pins are numbered starting from 0.
    ///
    /// # Examples
    ///
    /// Enable PB6 with Alternate Function 7 (USART1), fast speed, open-drain.
    ///
    /// ```no_run
    /// use kernel::stm32f4::gpio;
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
            update_with_mask(self.moder(), 0x3 << pin*2, (config.mode as u32) << pin*2);
            update_with_mask(self.ospeedr(), 0x3 << pin*2, (config.ospeed as u32) << pin*2);
            update_with_mask(self.otyper(), 0x1 << pin, config.otype as u32);
            update_with_mask(self.pupdr(), 0x3 << pin*2, (config.pupd as u32) << pin*2);

            /* The RX and TX pins are now connected to their AF
             * so that the USART1 can take over control of the
             * pins
             */
            if pin < 8 {
                update_with_mask(self.afrl(), 0xf << (pin*4), (config.af as u32) << pin*4);
            } else {
                update_with_mask(self.afrh(), 0xf << (pin-8)*4, (config.af as u32) << (pin-8)*4);
            }
        }
    }

    #[inline]
    fn moder(&self) -> Volatile<u32> {
        Volatile::new(self.base + 0x00)
    }

    #[inline]
    fn otyper(&self) -> Volatile<u32> {
        Volatile::new(self.base + 0x04)
    }

    #[inline]
    fn ospeedr(&self) -> Volatile<u32> {
        Volatile::new(self.base + 0x08)
    }

    #[inline]
    fn pupdr(&self) -> Volatile<u32> {
        Volatile::new(self.base + 0x0C)
    }

    #[inline]
    fn afrl(&self) -> Volatile<u32> {
        Volatile::new(self.base + 0x20)
    }

    #[inline]
    fn afrh(&self) -> Volatile<u32> {
        Volatile::new(self.base + 0x24)
    }
}
