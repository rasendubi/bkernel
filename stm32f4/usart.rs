//! Universal Synchronous Asynchronous Receiver Transmitter

// Stupid compiler thinks Bits0_5 is not camel case, but Bits05 is.
#![allow(non_camel_case_types)]

use core::fmt;

use volatile::RW;

extern {
    pub static USART1: Usart;
}

#[repr(C)]
pub struct Usart {
    sr:   RW<u32>, // 0x00
    dr:   RW<u32>, // 0x04
    brr:  RW<u32>, // 0x08
    cr1:  RW<u32>, // 0x0C
    cr2:  RW<u32>, // 0x10
    cr3:  RW<u32>, // 0x14
    gtpr: RW<u32>, // 0x18
}

#[test]
fn test_register_size() {
    assert_eq!(0x1C, ::core::mem::size_of::<Usart>());
}

#[allow(dead_code)]
#[repr(u32)]
enum Sr {
    PE   = 1 << 0,
    FE   = 1 << 1,
    NF   = 1 << 2,
    ORE  = 1 << 3,
    IDLE = 1 << 4,
    RXNE = 1 << 5,
    TC   = 1 << 6,
    TXE  = 1 << 7,
    LBD  = 1 << 8,
    CTS  = 1 << 9,
}

#[allow(dead_code)]
#[repr(u32)]
enum Brr {
    DIV_Fraction = 0x000F,
    DIV_Mantissa = 0xFFF0,
}

#[allow(dead_code)]
#[repr(u32)]
enum Cr1 {
    SBK    = 1 << 0,
    RWU    = 1 << 1,
    RE     = 1 << 2,
    TE     = 1 << 3,
    IDLEIE = 1 << 4,
    RXNEIE = 1 << 5,
    TCIE   = 1 << 6,
    TXEIE  = 1 << 7,
    PEIE   = 1 << 8,
    PS     = 1 << 9,
    PCE    = 1 << 10,
    WAKE   = 1 << 11,
    M      = 1 << 12,
    /// USART Enable
    UE     = 1 << 13,
    OVER8  = 1 << 15,
}

#[allow(dead_code)]
#[repr(u32)]
enum Cr2 {
    ADD   = 0xF << 0,
    LBDL  = 1 << 5,
    LBDIE = 1 << 6,
    LBCL  = 1 << 8,
    CPHA  = 1 << 9,
    CPOL  = 1 << 10,
    CLKEN = 1 << 11,
    STOP  = 0x3 << 12,
    LINEN = 1 << 14,
}

#[derive(Copy, Clone)]
#[repr(u32)]
pub enum StopBits {
    Bits1   = 0x0,
    Bits0_5 = 0x1,
    Bits2   = 0x2,
    Bits1_5 = 0x3,
}

#[allow(dead_code)]
#[repr(u32)]
enum Cr3 {
    EIE    = 1 << 0,
    IREN   = 1 << 1,
    IRLP   = 1 << 2,
    HDSEL  = 1 << 3,
    NACK   = 1 << 4,
    SCEN   = 1 << 5,
    DMAR   = 1 << 6,
    DMAT   = 1 << 7,
    RTSE   = 1 << 8,
    CTSE   = 1 << 9,
    CTSIE  = 1 << 10,
    ONEBIT = 1 << 11,
}

#[allow(dead_code)]
#[repr(u32)]
enum Gtpr {
    PSC = 0x00FF,
    GT  = 0xFF00,
}

#[derive(Copy, Clone)]
pub enum FlowControl {
    No,
}

#[derive(Copy, Clone)]
#[repr(u32)]
pub enum DataBits {
    Bits8 = 0,
    Bits9 = Cr1::M as u32,
}

#[derive(Copy, Clone)]
pub struct UsartConfig {
    pub data_bits: DataBits,
    pub stop_bits: StopBits,
    pub flow_control: FlowControl,
    pub baud_rate: u32,
}

#[derive(Copy, Clone)]
#[repr(u32)]
pub enum Interrupt {
    PE     = 0x0028,
    TXE    = 0x0727,
    TC     = 0x0626,
    RXNE   = 0x0525,
    ORE_RX = 0x0325,
    IDLE   = 0x0424,
    LBD    = 0x0846,
    CTS    = 0x096A,
    ERR    = 0x0060,
    ORE_ER = 0x0360,
    NE     = 0x0260,
    FE     = 0x0160,
}

#[derive(Copy, Clone)]
#[repr(u32)]
pub enum InterruptFlag {
    CTS  = 0x0200,
    LBD  = 0x0100,
    TXE  = 0x0080,
    TC   = 0x0040,
    RXNE = 0x0020,
    IDLE = 0x0010,
    ORE  = 0x0008,
    NE   = 0x0004,
    FE   = 0x0002,
    PE   = 0x0001,
}

impl Usart {
    /// Enables USART with given config.
    /// # Known bugs
    /// - No hardware flow control is supported.
    /// - Only works with default sysclk.
    /// - Generally, this driver is a piece of crap.
    pub fn enable(&self, config: &UsartConfig) {

        unsafe {
            self.cr2.update_with_mask(Cr2::STOP as u32, config.stop_bits as u32);
            self.cr1.update_with_mask(Cr1::M as u32 | Cr1::PCE as u32 | Cr1::TE as u32 | Cr1::RE as u32,
                                      config.data_bits as u32 | Cr1::TE as u32 | Cr1::RE as u32);
            self.cr3.clear_flag(0x3FF); // No Hardware Flow-Control
            self.brr.set(0xF42400 / config.baud_rate); // Default SysClk Rate / Baud Rate

            // finally this enables the complete USART peripheral
            self.cr1.set_flag(Cr1::UE as u32);
        }
    }

    pub fn puts_synchronous(&self, s: &str) {
        for c in s.bytes() {
            self.put_char(c as u32);
        }
    }

    pub fn put_bytes(&self, bytes: &[u8]) {
        for b in bytes {
            self.put_char(*b as u32);
        }
    }

    pub fn put_char(&self, c: u32) {
        while !self.transmission_complete() {};
        unsafe { self.dr.set(c); }
    }

    pub fn transmission_complete(&self) -> bool {
        unsafe { self.sr.get() & Sr::TC as u32 != 0 }
    }

    pub fn receive_complete(&self) -> bool {
        unsafe { self.sr.get() & Sr::RXNE as u32 != 0 }
    }

    pub fn get_char(&self) -> u32 {
        while !self.receive_complete() {}
        unsafe { self.dr.get() & 0xff }
    }

    pub unsafe fn get_unsafe(&self) -> u32 {
        self.dr.get() & 0xFF
    }

    pub unsafe fn put_unsafe(&self, c: u32) {
        self.dr.set(c);
    }

    pub fn it_enable(&self, it: Interrupt) {
        self.it_set(it, true);
    }

    pub fn it_disable(&self, it: Interrupt) {
        self.it_set(it, false);
    }

    fn it_set(&self, it: Interrupt, enable: bool) {
        let itpos = it as u32 & 0x001F;
        let itmask = 0x01 << itpos;

        let usartreg = (it as u32 & 0xFF) >> 5;
        let reg = match usartreg {
            0x01 => &self.cr1,
            0x02 => &self.cr2,
            _    => &self.cr3,
        };

        unsafe {
            if enable {
                reg.set_flag(itmask);
            } else {
                reg.clear_flag(itmask);
            }
        }
    }

    pub fn it_flag_status(&self, it: InterruptFlag) -> bool {
        unsafe {
            self.sr.get() & it as u32 != 0
        }
    }

    pub fn it_clear_flag(&self, it: InterruptFlag) {
        unsafe {
            self.sr.set((!(it as u16)) as u32);
        }
    }

    pub fn it_enabled(&self, it: Interrupt) -> bool {
        unsafe {
            let itpos = it as u32 & 0x001F;
            let itmask = 0x01 << itpos;

            let usartreg = (it as u8) >> 5;
            let reg = match usartreg {
                0x01 => &self.cr1,
                0x02 => &self.cr2,
                _    => &self.cr3,
            };

            itmask & reg.get() != 0
        }
    }

    pub fn it_status(&self, it: Interrupt) -> bool {
        unsafe {
            let itpos = it as u32 & 0x001F;
            let mut itmask = 0x01 << itpos;

            let usartreg = (it as u8) >> 5;
            let reg = match usartreg {
                0x01 => &self.cr1,
                0x02 => &self.cr2,
                _    => &self.cr3,
            };

            itmask = itmask & reg.get();

            let mut bitpos = it as u32 >> 8;
            bitpos = 0x01 << bitpos;
            bitpos = bitpos & self.sr.get();

            bitpos != 0 && itmask != 0
        }
    }

    pub fn it_clear_pending(&self, it: Interrupt) {
        unsafe {
            let bitpos = it as u32 >> 8;
            let itmask = 1 << bitpos;
            self.sr.set(!(itmask as u16) as u32);
        }
    }
}

// This is needed because fmt::Write impl on Usart is useless.
// All USART instances are immutable, so methods on &mut self are
// useless.
pub struct UsartProxy<'a>(pub &'a Usart);

impl<'a> fmt::Write for UsartProxy<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.0.puts_synchronous(s);
        Ok(())
    }

    fn write_char(&mut self, c: char) -> fmt::Result {
        self.0.put_char(c as u32);
        Ok(())
    }
}
