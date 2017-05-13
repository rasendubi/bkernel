//! Reset and clock control.

use volatile::{RW, RES};

extern {
    pub static RCC: Rcc;
}

// TODO(rasen): allow changing this?
/// Value of the Internal oscillator in Hz.
const HSI_VALUE: u32 = 16000000;

// TODO(rasen): allow changing this?
/// Value of the External oscillator in Hz.
const HSE_VALUE: u32 = 25000000;

#[repr(C)]
#[allow(missing_debug_implementations)]
pub struct Rcc {
    cr:          RW<u32>,  // 0x00

    /// This register is used to configure the PLL clock outputs
    /// according to the formulas:
    ///
    /// - F_VCO_clock = F_PLL_clock_input * (PLLN / PLLM)
    /// - F_PLL_general_clock_output = F_VCO_clock / PLLP
    /// - F_USB_OTG_FS__SDIO__RNG_clock_output = F_VCO_clock / PLLQ
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

#[allow(dead_code)]
#[derive(Copy, Clone)]
#[repr(u32)]
enum PllCfgrMask {
    // 5:0
    /// Division factor for the main PLL (PLL) and audio PLL (PLLI2S)
    /// input clock.
    PLLM = 0x3F << 0,

    // 14:6
    /// Mail PLL (PLL) multiplication factor for VCO.
    PLLN = 0xFF << 6,

    // 17:16
    /// Main PLL (PLL) division factor for main system clock.
    PLLP = 0x3 << 16,

    // 21:18 Reserved, must be kept at reset value.

    // 22
    /// Main PLL (PLL) and audio PLL (PLLI2S) entry clock source.
    PLLSRC = 0x1 << 22,

    // 23 Reserver, must be kept at reset value.

    // 27:24
    /// Main PLL (PLL) division factor for USB OTG FS, SDIO and random
    /// number generator clocks.
    PLLQ = 0xF << 24,

    // 31:28 Reserver, must be kept at reset value.
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
#[repr(u32)]
enum CfgrMask {
    // 1:0
    /// System clock switch
    SW      = 0x3 << 0,

    // 3:2
    /// System clock switch status
    SWS     = 0x3 << 2,

    // 7:4
    /// AHB prescaler
    HPRE    = 0x7 << 4,

    // 9:8 reserved
    // 12:10
    /// APB Low speed prescaler (APB1)
    PPRE1   = 0x7 << 10,

    // 15:13
    /// APB high-speed prescaler (APB2)
    PPRE2   = 0x7 << 13,

    // 20:16
    /// HSE division factor for RTC clock
    RTCPRE  = 0x1F << 16,

    // 22:21
    /// Microcontroller clock output 1
    MCO1    = 0x3 << 21,

    // 23
    /// I2S clock selection
    I2SSRC  = 0x1 << 23,

    // 24:26
    /// MCO1 prescaler
    MCO1PRE = 0x7 << 24,

    // 27:29
    /// MCO2 prescaler
    MCO2PRE = 0x7 << 27,

    // 31:30
    /// Microcontroller clock output 2 [1:0]
    MCO2    = 0x3 << 30,
}

#[derive(Copy, Clone, Debug)]
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

#[derive(Copy, Clone, Debug)]
#[repr(u32)]
pub enum Ahb2Enable {
    DCMI       = 1 << 0,
    CRYP       = 1 << 4,
    HASH       = 1 << 5,
    RNG        = 1 << 6,
    OTGFS      = 1 << 7,
}

#[derive(Copy, Clone, Debug)]
#[repr(u32)]
pub enum Ahb3Enable {
    FMC        = 1 << 0,

    // This is added to avoid E0083: unsupported representation for
    // univariant enum
    __Dummy,
}

#[derive(Copy, Clone, Debug)]
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

#[derive(Copy, Clone, Debug)]
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

#[allow(missing_debug_implementations)]
pub struct Clocks {
    /// SYSCLK clock frequency expressed in Hz
    pub sysclk: u32,

    /// HCLK clock frequency expressed in Hz
    pub hclk:   u32,

    /// PCLK1 clock frequency expressed in Hz
    pub pclk1:  u32,

    /// PCLK2 clock frequency expressed in Hz
    pub pclk2:  u32,
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

    pub fn clock_freqs(&self) -> Clocks {
        let cfgr = unsafe { self.cfgr.get() };

        let sysclk = match cfgr & (CfgrMask::SWS as u32) {
            0x00 => {
                HSI_VALUE
            },
            0x04 => {
                HSE_VALUE
            },
            0x08 => {
                // PLL_VCO = (HSE_VALUE or HSI_VALUE / PLLM) * PLLN
                // SYSCLK = PLL_VCO / PLLP

                let pllcfgr = unsafe { self.pllcfgr.get() };

                let pllsource = (pllcfgr & PllCfgrMask::PLLSRC as u32) >> 22;
                let pllm = pllcfgr & PllCfgrMask::PLLM as u32;
                let plln = (pllcfgr & PllCfgrMask::PLLN as u32) >> 6;
                let pllp = (((pllcfgr & PllCfgrMask::PLLP as u32) >> 16) + 1) * 2;

                let pllvco_base = if pllsource != 0 { HSE_VALUE } else { HSI_VALUE };
                let pllvco = pllvco_base / pllm * plln;

                pllvco / pllp
            },
            _ => {
                debug_assert!(false);
                // TODO(rasen): not applicable (assert? unreachable?)
                HSI_VALUE
            },
        };

        // Compute HCLK, PCLK1 and PCLK2 clocks frequencies
        const APBAHB_PRESC_TABLE: [u8; 16] = [
            0, 0, 0, 0,
            1, 2, 3, 4,
            1, 2, 3, 4,
            6, 7, 8, 9,
        ];

        let hclk = {
            let presc = APBAHB_PRESC_TABLE[((cfgr & CfgrMask::HPRE as u32) >> 4) as usize];
            sysclk >> presc
        };

        let pclk1 = {
            let presc = APBAHB_PRESC_TABLE[((cfgr & CfgrMask::PPRE1 as u32) >> 10) as usize];
            hclk >> presc
        };

        let pclk2 = {
            let presc = APBAHB_PRESC_TABLE[((cfgr & CfgrMask::PPRE2 as u32) >> 13) as usize];
            hclk >> presc
        };

        Clocks {
            sysclk,
            hclk,
            pclk1,
            pclk2,
        }
    }
}
