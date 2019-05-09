//! Random number generator.

// allow `<< 0`
#![allow(clippy::identity_op)]

use crate::volatile::{RW, RO};

extern {
    pub static RNG: Rng;
}

#[repr(C)]
#[allow(missing_debug_implementations)]
pub struct Rng {
    cr: RW<u32>,
    sr: RW<u32>,
    dr: RO<u32>,
}

#[derive(Copy, Clone)]
#[repr(u32)]
enum CrMask {
    /// Interrupt enable.
    ///
    /// 0: Interrupt is disabled.
    /// 1: Interrupt is enabled. An interrupt is pending soon as
    /// DRDY=1 or SEIS=1 or CEIS=1 in the SR register.
    IE = 0x1 << 3,

    /// Random number generator enable.
    ///
    /// 0: Random number generator is disabled.
    /// 1: Random number generator is enabled.
    RNDGEN = 0x1 << 2,
}

#[repr(u32)]
#[derive(Debug)]
pub enum SrMask {
    /// Seed error interrupt status.
    ///
    /// This bit is set at the same time as SECS, it is cleared by
    /// writing it to 0.
    ///
    /// 0: No faulty sequence detected.
    /// 1: One of the following faulty sequences has been detected:
    ///   - More than 64 consecutive bits as the same value (0 or 1)
    ///   - More than 32 consecutive alternances of 0 and 1
    ///   (01010101...01)
    ///
    /// An interrupt is pending if IE = 1 in the CR register.
    SEIS = 0x1 << 6,

    /// Clock error interrupt status.
    ///
    /// The bit is set at the same time as CECS, it is cleared by
    /// writing it to 0.
    ///
    /// 0: The CLK clock was correctly detected.
    /// 1: The CLK was not correctly detected (f_clk < f_hclk/16)
    ///
    /// An interrupt is pending if IE = 1 in the CR register.
    CEIS = 0x1 << 5,

    /// Seed error current status.
    ///
    /// 0: No faulty sequence has currently been detected. If the SEIS
    /// bit is set, this means that a faulty sequence was detected and
    /// the situation has been recovered.
    /// 1: One of the following faulty sequences has been detected:
    ///   - More than 64 consecutive bits at the same value (0 or 1)
    ///   - More than 32 consecutive alternatives of 0 and 1
    ///   (010101...01)
    SECS = 0x1 << 2,

    /// Clock error current status.
    ///
    /// 0: RNG_CLK clock has been correctly detected. If the CEIS bit
    /// is set, this means that a clock error was detected and the
    /// situation has been recovered.
    /// 1: RNG_CLK was not correctly detected (f_rng_clk < f_hclk/16).
    CECS = 0x1 << 1,

    /// Data ready.
    ///
    /// 0: The DR register is not yet valid, no random data is
    /// available.
    /// 1: The DR register contains valid random data.
    ///
    /// Note: An interrupt is pending if IE = 1 in the CR register.
    /// Once the DR register has been read, this bit returns to 0
    /// until a new valid value is computed.
    DRDY = 0x1 << 0,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Error {
    SeedError,
    ClockError,
}

impl Rng {
    pub fn enable(&self) {
        unsafe {
            self.cr.set_flag(CrMask::RNDGEN as u32);
        }
    }

    pub fn disable(&self) {
        unsafe {
            self.cr.clear_flag(CrMask::RNDGEN as u32);
        }
    }

    pub fn it_enable(&self) {
        unsafe {
            self.cr.set_flag(CrMask::IE as u32);
        }
    }

    pub fn it_disable(&self) {
        unsafe {
            self.cr.clear_flag(CrMask::IE as u32);
        }
    }

    pub fn sr_status(&self, mask: SrMask) -> bool {
        (unsafe { self.sr.get() }) & (mask as u32) != 0
    }

    pub fn get(&self) -> Result<Option<u32>, Error> {
        let sr = unsafe { self.sr.get() };
        if sr & (SrMask::SECS as u32) != 0 {
            // From reference manual (24.3.2):
            //
            // "In the case of a seed error, [...]. If a number is
            // available in the DR register, it must not be used
            // because it may not have enough entropy."
            Err(Error::SeedError)
        } else if sr & (SrMask::DRDY as u32) != 0 {
            Ok(Some(unsafe { self.dr.get() }))
        } else if sr & (SrMask::CECS as u32) != 0 {
            // From reference manual (24.3.2):
            //
            // "The clock error has no impact on the previously
            // generated random numbers, and the DR register contents
            // can be used."
            Err(Error::ClockError)
        } else {
            Ok(None)
        }
    }

    pub unsafe fn get_data_unchecked(&self) -> u32 {
        self.dr.get()
    }
}
