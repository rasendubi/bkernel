//! Reset and clock control.

use volatile::{RW, RES};

extern {
    pub static RCC: Rcc;
}

#[repr(C)]
pub struct Rcc {
    cr:          RW<u32>,  // 0x00
    pllcfgr:     RW<u32>,  // 0x04
    cfgr:        RW<u32>,  // 0x08
    cir:         RW<u32>,  // 0x0C
    ahb1rstr:    RW<u32>,  // 0x10
    ahb2rstr:    RW<u32>,  // 0x14
    ahb3rstr:    RW<u32>,  // 0x18
    _0:          RES<u32>, // 0x1C
    apb1rstr:    RW<u32>,  // 0x20
    apb2rstr:    RW<u32>,  // 0x24
    _1:          RES<u32>, // 0x28
    _2:          RES<u32>, // 0x2C
    ahb1enr:     RW<u32>,  // 0x30
    ahb2enr:     RW<u32>,  // 0x34
    ahb3enr:     RW<u32>,  // 0x38
    _3:          RES<u32>, // 0x3C
    apb1enr:     RW<u32>,  // 0x40
    apb2enr:     RW<u32>,  // 0x44
    _4:          RES<u32>, // 0x48
    _5:          RES<u32>, // 0x4C
    ahb1lpenr:   RW<u32>,  // 0x50
    ahb2lpenr:   RW<u32>,  // 0x54
    ahb3lpenr:   RW<u32>,  // 0x58
    _6:          RES<u32>, // 0x5C
    apb1lpenr:   RW<u32>,  // 0x60
    apb2lpenr:   RW<u32>,  // 0x64
    _7:          RES<u32>, // 0x68
    _8:          RES<u32>, // 0x6C
    bdcr:        RW<u32>,  // 0x70
    csr:         RW<u32>,  // 0x74
    _9:          RES<u32>, // 0x78
    _10:         RES<u32>, // 0x7C
    sscgr:       RW<u32>,  // 0x80
    plli2scfgr:  RW<u32>,  // 0x84
    pllsaicfgr:  RW<u32>,  // 0x88
    dckcfgr:     RW<u32>,  // 0x8C
}

#[test]
fn test_register_size() {
    assert_eq!(0x90, ::core::mem::size_of::<Rcc>());
}

#[derive(Copy, Clone)]
#[repr(u32)]
pub enum Ahb1Enable {
    GPIOA      = 1 << 0,
    GPIOB      = 1 << 1,
    GPIOC      = 1 << 2,
    GPIOD      = 1 << 3,
    GPIOE      = 1 << 4,
    GPIOF      = 1 << 5,
    GPIOG      = 1 << 6,
    GPIOH      = 1 << 7,
    GPIOI      = 1 << 8,
    GPIOJ      = 1 << 9,
    GPIOK      = 1 << 10,
    CRC        = 1 << 12,
    BKPSRAM    = 1 << 18,
    CCMDATARAM = 1 << 20,
    DMA1       = 1 << 21,
    DMA2       = 1 << 22,
    DMA2D      = 1 << 23,
    ETHMAC     = 1 << 25,
    ETHMACTX   = 1 << 26,
    ETHMACRX   = 1 << 27,
    ETHMACPTP  = 1 << 28,
    OTGHS      = 1 << 29,
    OTGHSULPI  = 1 << 30,
}

#[derive(Copy, Clone)]
#[repr(u32)]
pub enum Ahb2Enable {
    DCMI       = 1 << 0,
    CRYP       = 1 << 4,
    HASH       = 1 << 5,
    RNG        = 1 << 6,
    OTGFS      = 1 << 7,
}

#[derive(Copy, Clone)]
#[repr(u32)]
pub enum Ahb3Enable {
    FMC        = 1 << 0,

    // This is added to avoid E0083: unsupported representation for
    // univariant enum
    __Dummy,
}

#[derive(Copy, Clone)]
#[repr(u32)]
pub enum Apb1Enable {
    TIM2       = 1 << 0,
    TIM3       = 1 << 1,
    TIM4       = 1 << 2,
    TIM5       = 1 << 3,
    TIM6       = 1 << 4,
    TIM7       = 1 << 5,
    TIM12      = 1 << 6,
    TIM13      = 1 << 7,
    TIM14      = 1 << 8,
    WWDG       = 1 << 11,
    SPI2       = 1 << 14,
    SPI3       = 1 << 15,
    USART2     = 1 << 17,
    USART3     = 1 << 18,
    USART4     = 1 << 19,
    USART5     = 1 << 20,
    I2C1       = 1 << 21,
    I2C2       = 1 << 22,
    I2C3       = 1 << 23,
    CAN1       = 1 << 25,
    CAN2       = 1 << 26,
    PWR        = 1 << 28,
    DAC        = 1 << 29,
    UART7      = 1 << 30,
    UART8      = 1 << 31,
}

#[derive(Copy, Clone)]
#[repr(u32)]
pub enum Apb2Enable {
    TIM1       = 1 << 0,
    TIM8       = 1 << 1,
    USART1     = 1 << 4,
    USART6     = 1 << 5,
    ADC1       = 1 << 8,
    ADC2       = 1 << 9,
    ADC3       = 1 << 10,
    SDIO       = 1 << 11,
    SPI1       = 1 << 12,
    SPI4       = 1 << 13,
    SYSCFG     = 1 << 14,
    TIM9       = 1 << 16,
    TIM10      = 1 << 17,
    TIM11      = 1 << 18,
    SPI5       = 1 << 20,
    SPI6       = 1 << 21,
    SAI1       = 1 << 22,
    LTDC       = 1 << 26,
}

impl Rcc {
    pub fn ahb1_clock_enable(&self, value: Ahb1Enable) {
        unsafe {
            self.ahb1enr.update(|x| x | value as u32);
        }
    }

    pub fn ahb2_clock_enable(&self, value: Ahb2Enable) {
        unsafe {
            self.ahb2enr.update(|x| x | value as u32);
        }
    }

    pub fn ahb3_clock_enable(&self, value: Ahb3Enable) {
        unsafe {
            self.ahb3enr.update(|x| x | value as u32);
        }
    }

    pub fn apb1_clock_enable(&self, value: Apb1Enable) {
        unsafe {
            self.apb1enr.update(|x| x | value as u32);
        }
    }

    pub fn apb2_clock_enable(&self, value: Apb2Enable) {
        unsafe {
            self.apb2enr.update(|x| x | value as u32);
        }
    }
}
