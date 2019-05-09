//! Nested Vector Interrupt Controller

use crate::volatile::{RO, RW};

extern "C" {
    pub static ICTR: RO<u32>;
    pub static ISER: [RW<u32>; 8];
    pub static ICER: [RW<u32>; 8];
    pub static ISPR: [RW<u32>; 8];
    pub static ICPR: [RW<u32>; 8];
    pub static IABR: [RO<u32>; 8];
    pub static IPR: [RW<u32>; 82];

    pub static AIRCR: RW<u32>;
}

#[derive(Debug)]
pub struct NvicInit {
    pub irq_channel: IrqChannel,
    pub priority: u8,
    pub subpriority: u8,
    pub enable: bool,
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug)]
#[repr(u32)]
pub enum IrqChannel {
    WWDG = 0,
    PVD = 1,
    TAMP_STAMP = 2,
    RTC_WKUP = 3,
    FLASH = 4,
    RCC = 5,
    EXTI0 = 6,
    EXTI1 = 7,
    EXTI2 = 8,
    EXTI3 = 9,
    EXTI4 = 10,
    DMA1_Stream0 = 11,
    DMA1_Stream1 = 12,
    DMA1_Stream2 = 13,
    DMA1_Stream3 = 14,
    DMA1_Stream4 = 15,
    DMA1_Stream5 = 16,
    DMA1_Stream6 = 17,
    ADC = 18,
    CAN1_TX = 19,
    CAN1_RX0 = 20,
    CAN1_RX1 = 21,
    CAN1_SCE = 22,
    EXTI9_5 = 23,
    TIM1_BRK_TIM9 = 24,
    TIM1_UP_TIM1 = 25,
    TIM1_TRG_COM_TIM11 = 26,
    TIM1_CC = 27,
    TIM2 = 28,
    TIM3 = 29,
    TIM4 = 30,
    I2C1_EV = 31,
    I2C1_ER = 32,
    I2C2_EV = 33,
    I2C2_ER = 34,
    SPI1 = 35,
    SPI2 = 36,
    USART1 = 37,
    USART2 = 38,
    USART3 = 39,
    EXTI15_10 = 40,
    RCT_Alarm = 41,
    OTG_FS_WKUP = 42,
    TIM8_BRK_TIM12 = 43,
    TIM8_UP_TIM13 = 44,
    TIM8_TRG_COM_TIM14 = 45,
    TIM8_CC = 46,
    DMA1_Stream7 = 47,
    FSMC = 48,
    SDIO = 49,
    TIM5 = 50,
    SPI3 = 51,
    UART4 = 52,
    UART5 = 53,
    TIM6_DAC = 54,
    TIM7 = 55,
    DMA2_Stream0 = 56,
    DMA2_Stream1 = 57,
    DMA2_Stream2 = 58,
    DMA2_Stream3 = 59,
    DMA2_Stream4 = 60,
    ETH = 61,
    ETH_WKUP = 62,
    CAN2_TX = 63,
    CAN2_RX0 = 64,
    CAN2_RX1 = 65,
    CAN2_SCE = 66,
    OTG_FS = 67,
    DMA2_Stream5 = 68,
    DMA2_Stream6 = 69,
    DMA2_Stream7 = 70,
    USART6 = 71,
    I2C3_EV = 72,
    I2C3_ER = 73,
    OTG_HS_EP1_OUT = 74,
    OTG_HS_EP1_IN = 75,
    OTG_HS_WKUP = 76,
    OTG_HS = 77,
    DCMI = 78,
    CRYP = 79,
    HASH_RNG = 80,
    FPU = 81,
}

pub fn init(nvic: &NvicInit) {
    unsafe {
        if nvic.enable {
            let mut tmppriority = (0x700 - (AIRCR.get() & 0x700)) >> 0x08;
            let tmppre = 0x4 - tmppriority;
            let tmpsub = 0x0F >> tmppriority;

            tmppriority = u32::from(nvic.priority) << tmppre;
            tmppriority |= u32::from(nvic.subpriority) & tmpsub;
            tmppriority <<= 0x04;

            IPR[nvic.irq_channel as usize].set(tmppriority);
            ISER[nvic.irq_channel as usize >> 5].set(0x1 << (nvic.irq_channel as u8 & 0x1F));
        } else {
            ICER[nvic.irq_channel as usize >> 5].set(0x1 << (nvic.irq_channel as u8 & 0x1F));
        }
    }
}
