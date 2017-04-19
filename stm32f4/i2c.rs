//! Inter-integrated circuit (I2C) interface.

use volatile::RW;

use super::rcc::RCC;

extern {
    pub static I2C1: I2c;
    pub static I2C2: I2c;
    pub static I2C3: I2c;
}

#[repr(C)]
pub struct I2c {
    cr1:   RW<u32>, // 0x00
    cr2:   RW<u32>, // 0x04
    oar1:  RW<u32>, // 0x08
    oar2:  RW<u32>, // 0x0C
    dr:    RW<u32>, // 0x10
    sr1:   RW<u32>, // 0x14
    sr2:   RW<u32>, // 0x18
    ccr:   RW<u32>, // 0x1C
    trise: RW<u32>, // 0x20

    // Available on STM32F42xxx and STM32F43xxx only.
    fltr:  RW<u32>, // 0x24
}

#[test]
fn test_register_size() {
    assert_eq!(0x28, ::core::mem::size_of::<I2c>());
}

#[allow(non_camel_case_types)]
#[allow(dead_code)]
#[derive(Copy, Clone)]
#[repr(u32)]
enum Cr1Masks {
    // 0
    /// Peripheral enable.
    PE = 0x1 << 0,

    // 1
    /// SMBus mode.
    SMBUS = 0x1 << 1,

    // 2 Reserved

    // 3
    /// SMBus type.
    SMBTYPE = 0x1 << 3,

    // 4
    /// ARP enable.
    ENARP = 0x1 << 4,

    // 5
    /// PEC enable.
    ENPEC = 0x1 << 5,

    // 6
    /// General call enable.
    ENGC = 0x1 << 6,

    // 7
    /// Clock stretching disable (Slave mode).
    NOSTRETCH = 0x1 << 7,

    // 8
    /// Start generation.
    START = 0x1 << 8,

    // 9
    /// Stop generation.
    STOP = 0x1 << 9,

    // 10
    /// Acknowledge enable.
    ACK = 0x1 << 10,

    // 11
    /// Acknowledge/PEC Position (for data reception).
    POS = 0x1 << 11,

    // 12
    /// Packet error checking.
    PEC = 0x1 << 12,

    // 13
    /// SMBus alert.
    ALERT = 0x1 << 13,

    // 14 Reserved

    // 15
    /// Software reset.
    SWRST = 0x1 << 15,

    /// All allowed bits.
    CLEAR_MASK = 0xFBF5,
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
#[repr(u32)]
enum Cr2Masks {
    // 5:0
    /// Peripheral clock frequency.
    FREQ = 0x3F << 0,

    // 7:6 Reserved, must be kept at reset value

    // 8
    /// Error interrupt enable.
    ITERREN = 0x1 << 8,

    // 9
    /// Event interrupt enable.
    ITEVTEN = 0x1 << 9,

    // 10
    /// Buffer interrupt enable.
    ITBUFEN = 0x1 << 10,

    // 11
    /// DMA requests enable.
    DMAEN = 0x1 << 11,

    // 12
    /// DMA last transfer.
    LAST = 0x1 << 12,

    // 15:13 Reserved, must be kept at reset value
}

#[allow(non_camel_case_types)]
#[allow(dead_code)]
#[derive(Copy, Clone)]
#[repr(u32)]
enum CcrMasks {
    // 11:0
    /// Clock control register in Fm/Sm mode (Master mode).
    ///
    /// Controls the SCL clock in master mode.
    CCR = 0x0fff << 0,

    // 13:12 Reserved
    /// Fm mode duty cycle.
    ///
    /// 0: Fm mode Tlow/Thigh = 2
    /// 1: Fm mode Tlow/Thigh = 16/9
    DUTY = 0x1 << 14,

    // 15
    /// I2C master mode selection.
    ///
    /// 0: Sm mode I2C
    /// 1: Fm mode I2C
    F_S = 0x1 << 15,
}

pub struct I2cInit {
    /// Specifies the clock frequency.
    ///
    /// This parameter must be set to a value lower than 400kHz.
    clock_speed: u32,
    mode: Mode,
    duty_cycle: DutyCycle,

    /// Must be a 7-bit or 10-bit address.
    own_address1: u16,
    ack: Acknowledgement,
    acknowledged_address: AcknowledgedAddress,
}

#[derive(Copy, Clone)]
#[repr(u16)]
pub enum Mode {
    I2C         = 0x0000,
    SMBusDevice = 0x0002,
    SMBusHost   = 0x000A,
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u16)]
pub enum DutyCycle {
    /// I2C fast mode Tlow/Thigh = 16/9
    DutyCycle_16_9 = 0x4000,

    /// I2C fast mode Tlow/Thigh = 2
    DutyCycle_2    = 0xBFFF,
}

#[derive(Copy, Clone)]
#[repr(u16)]
pub enum Acknowledgement {
    Enable  = 0x0400,
    Disable = 0x0000,
}

#[derive(Copy, Clone)]
#[repr(u8)]
pub enum Direction {
    Transmitter = 0x00,
    Receiver    = 0x01,
}

#[derive(Copy, Clone)]
#[repr(u16)]
pub enum AcknowledgedAddress {
    Bit7  = 0x4000,
    Bit10 = 0xC000,
}

pub const I2C_INIT: I2cInit = I2cInit {
    clock_speed: 5000,
    mode: Mode::I2C,
    duty_cycle: DutyCycle::DutyCycle_2,
    own_address1: 0,
    ack: Acknowledgement::Disable,
    acknowledged_address: AcknowledgedAddress::Bit7,
};

impl Default for I2cInit {
    fn default() -> I2cInit {
        I2C_INIT
    }
}

impl I2c {
    pub unsafe fn init(&self, init: &I2cInit) {
        debug_assert!(init.clock_speed >= 0x1 && init.clock_speed <= 400000);
        debug_assert!(init.own_address1 <= 0x3ff);

        let pclk1 = RCC.clock_freqs().pclk1;

        // Set frequency bits depending on pclk1 value
        let freqrange = pclk1 / 1000000;

        self.cr2.update_with_mask(Cr2Masks::FREQ as u32, freqrange);

        // Disable the selected I2C peripheral to configure TRISE
        self.cr1.clear_flag(Cr1Masks::PE as u32);

        // Clear F/S, DUTY and CCR[11:0] bits
        let mut ccr = 0;

        if init.clock_speed <= 100000 {
            // Standard mode

            let mut result = pclk1 / (init.clock_speed << 1);

            if result < 0x04 {
                // Set minimum allowed value
                result = 0x04;
            }

            ccr |= result;

            self.trise.set(freqrange + 1);
        } else { // i2c.clock_speed <= 400000
            // Fast mode.
            //
            // To use the I2C at 400 KHz (in fast mode), the PCLK1
            // frequency (I2C peripheral input clock) must be a
            // multiple of 10 MHz.

            let mut result = if init.duty_cycle == DutyCycle::DutyCycle_2 {
                // Fast mode speed calculate: Tlow/Thigh = 2
                pclk1 / (init.clock_speed * 3)
            } else {
                // Fast mode speed calculate: Tlow/Thigh = 16/9
                pclk1 / (init.clock_speed * 25) | (DutyCycle::DutyCycle_16_9 as u32)
            };

            // Test if CCR value is under 0x1
            if (result & (CcrMasks::CCR as u32)) == 0 {
                // Set minimum allowed value
                result |= 0x0001;
            }

            // Set speed value and set F/S bit for fast mode
            ccr |= result | (CcrMasks::F_S as u32);

            // Set Maximum Rise Time for fast mode
            self.trise.set(freqrange*300/1000 + 1);
        }

        // Write to CCR
        self.ccr.set(ccr);

        // Enable the selected I2C peripheral
        self.cr1.set_flag(Cr1Masks::PE as u32);

        // CR1 Configuration
        self.cr1.update(|cr1| {
            // Clear ACK, SMBTYPE and SMBUS bits
            cr1 & (Cr1Masks::CLEAR_MASK as u32) |
            // Configure mode and acknowledgement
            // Set SMBTYPE and SMBUS bits according to init.mode value
            // Set ACK bit according to init.ack value
            (init.mode as u32) | (init.ack as u32)
        });

        // Set Own Address1 and acknowledged address
        self.oar1.set((init.acknowledged_address as u32) | (init.own_address1 as u32));
    }
}
