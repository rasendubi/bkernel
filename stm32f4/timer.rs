//! General-purpose timers (TIM2-TIM5)

use volatile::RW;

extern {
    pub static TIM2: Tim;
    pub static TIM3: Tim;
    pub static TIM4: Tim;
    pub static TIM5: Tim;
}

#[repr(C)]
pub struct Tim {
    cr1:   RW<u32>, // 0x00
    cr2:   RW<u32>, // 0x04
    smcr:  RW<u32>, // 0x08
    dier:  RW<u32>, // 0x0C
    sr:    RW<u32>, // 0x10
    egr:   RW<u32>, // 0x14
    ccmr1: RW<u32>, // 0x18
    ccmr2: RW<u32>, // 0x1C
    ccer:  RW<u32>, // 0x20
    cnt:   RW<u32>, // 0x24
    psc:   RW<u32>, // 0x28
    arr:   RW<u32>, // 0x2c
    _0:    RW<u32>, // 0x30
    ccr1:  RW<u32>, // 0x34
    ccr2:  RW<u32>, // 0x38
    ccr3:  RW<u32>, // 0x3C
    ccr4:  RW<u32>, // 0x40
    _1:    RW<u32>, // 0x44
    dcr:   RW<u32>, // 0x48
    dmar:  RW<u32>, // 0x4C
    /// Unique to TIM2 and TIM5
    or:    RW<u32>, // 0x50
}

#[repr(u32)]
enum Cr1 {
    CEN  = 1 << 0,
    UDIS = 1 << 1,
    URS  = 1 << 2,
    OPM  = 1 << 3,
    DIR  = 1 << 4,
    CMS  = 3 << 5,
    ARPE = 1 << 7,
    CKD  = 3 << 8,
}

#[repr(u32)]
enum Egr {
    UG   = 1 << 0,
    CC1G = 1 << 1,
    CC2G = 1 << 2,
    CC3G = 1 << 3,
    CC4G = 1 << 4,
    TG   = 1 << 6,
}

pub struct TimInit {
    pub prescaler: u16,
    pub counter_mode: CounterMode,
    pub period: u32,
    pub clock_division: ClockDivision,
    pub repetition_counter: u8,
}

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum CounterMode {
    Up             = 0x0000,
    Down           = 0x0010,
    CenterAligned1 = 0x0020,
    CenterAligned2 = 0x0040,
    CenterAligned3 = 0x0060,
}

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum ClockDivision {
    Div1 = 0x0000,
    Div2 = 0x0100,
    Div3 = 0x0200,
}

impl Tim {
    pub fn init(&self, tim: &TimInit) {
        unsafe {
            let mut tmpcr1: u16 = self.cr1.get() as u16;
            tmpcr1 &= !(Cr1::DIR as u32 | Cr1::CMS as u32) as u16;
            tmpcr1 = (tmpcr1 as u32 | tim.counter_mode as u32) as u16;
            self.cr1.set(tmpcr1 as u32);

            self.arr.set(tim.period);
            self.psc.set(tim.prescaler as u32);
            self.egr.set(Egr::TG as u32);
        }
    }

    pub fn enable(&self) {
        unsafe {
            self.cr1.set_flag(Cr1::CEN as u32);
        }
    }

    pub fn disable(&self) {
        unsafe {
            self.cr1.clear_flag(Cr1::CEN as u32);
        }
    }

    pub fn get_counter(&self) -> u32 {
        unsafe {
            self.cnt.get()
        }
    }
}
