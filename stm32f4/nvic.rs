//! Nested Vector Interrupt Controller

use volatile::{RW, RO};

extern {
    pub static ICTR: RO<u32>;
    pub static ISER: [RW<u32>; 8];
    pub static ICER: [RW<u32>; 8];
    pub static ISPR: [RW<u32>; 8];
    pub static ICPR: [RW<u32>; 8];
    pub static IABR: [RO<u32>; 8];
    pub static IPR:  [RW<u32>; 60];

    pub static AIRCR: RW<u32>;
}

pub struct NvicInit {
    pub irq_channel: IrqChannel,
    pub preemption_priority: u8,
    pub channel_subpriority: u8,
    pub enable: bool,
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum IrqChannel {
    // TODO rest
    TIM2 = 28,
}

pub fn init(nvic: &NvicInit) {
    unsafe {
        if nvic.enable {
            let mut tmppriority = (0x700 - AIRCR.get() & 0x700) >> 0x08;
            let tmppre = 0x4 - tmppriority;
            let tmpsub = 0x0F >> tmppriority;

            tmppriority = (nvic.preemption_priority as u32) << tmppre;
            tmppriority |= nvic.channel_subpriority as u32 & tmpsub;
            tmppriority = tmppriority << 0x04;

            IPR[nvic.irq_channel as usize].set(tmppriority);
            ISER[nvic.irq_channel as usize >> 5].set(0x1 << (nvic.irq_channel as u8 & 0x1F));
        } else {
            ICER[nvic.irq_channel as usize >> 5].set(0x1 << (nvic.irq_channel as u8 & 0x1F));
        }
    }
}
