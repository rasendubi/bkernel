const RCC_BASE: u32 = 0x40023800;
registers! {
    RCC_BASE, u32 => {
        RCC_CR         = 0x00,
        RCC_PLLCFGR    = 0x04,
        RCC_CFGR       = 0x08,
        RCC_CIR        = 0x0C,
        RCC_AHB1RSTR   = 0x10,
        RCC_AHB2RSTR   = 0x14,
        RCC_AHB3RSTR   = 0x18,
        RCC_APB1RSTR   = 0x20,
        RCC_APB2RSTR   = 0x24,
        RCC_AHB1ENR    = 0x30,
        RCC_AHB2ENR    = 0x34,
        RCC_AHB3ENR    = 0x38,
        RCC_APB1ENR    = 0x40,
        RCC_APB2ENR    = 0x44,
        RCC_AHB1LPENR  = 0x50,
        RCC_AHB2LPENR  = 0x54,
        RCC_AHB3LPENR  = 0x58,
        RCC_APB1LPENR  = 0x60,
        RCC_APB2LPENR  = 0x64,
        RCC_BDCR       = 0x70,
        RCC_CSR        = 0x74,
        RCC_SSCGR      = 0x80,
        RCC_PLLI2SCFGR = 0x84,
        RCC_PLLSAICFGR = 0x88,
        RCC_DCKCFGR    = 0x8C,
    }
}

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

pub enum Ahb2Enable {
    DCMI       = 1 << 0,
    CRYP       = 1 << 4,
    HASH       = 1 << 5,
    RNG        = 1 << 6,
    OTGFS      = 1 << 7,
}

pub enum Ahb3Enable {
    FMC        = 1 << 0,
}

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

pub fn ahb1_clock_enable(val: Ahb1Enable) {
    unsafe {
        RCC_AHB1ENR.set(RCC_AHB1ENR.get() | val as u32);
    }
}

pub fn ahb2_clock_enable(val: Ahb2Enable) {
    unsafe {
        RCC_AHB2ENR.set(RCC_AHB2ENR.get() | val as u32);
    }
}

pub fn ahb3_clock_enable(val: Ahb3Enable) {
    unsafe {
        RCC_AHB3ENR.set(RCC_AHB3ENR.get() | val as u32);
    }
}

pub fn apb1_clock_enable(val: Apb1Enable) {
    unsafe {
        RCC_APB1ENR.set(RCC_APB1ENR.get() | val as u32);
    }
}

pub fn apb2_clock_enable(val: Apb2Enable) {
    unsafe {
        RCC_APB2ENR.set(RCC_APB2ENR.get() | val as u32);
    }
}
