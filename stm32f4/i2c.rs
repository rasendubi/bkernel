//! Inter-integrated circuit (I2C) interface.

use volatile::{RO, RW};

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
    sr2:   RO<u32>, // 0x18
    ccr:   RW<u32>, // 0x1C
    trise: RW<u32>, // 0x20

    // Available on STM32F42xxx and STM32F43xxx only.
    fltr:  RW<u32>, // 0x24
}

#[test]
fn test_register_size() {
    assert_eq!(0x28, ::core::mem::size_of::<I2c>());
}

const FLAG_MASK: u32 = 0x00FFFFFF; // I2C FLAG mask

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
enum Sr1Masks {
    /// Start bit (Master mode)
    ///
    /// 0: No Start condition
    /// 1: Start condition
    ///
    /// - Set when a Start condition generated.
    /// - Cleared by software by reading the SR1 register followed by
    /// writing the DR register, or by hardware when PE=0
    SB = 0x1 << 0,

    /// Address sent (master mode)/matched (slave mode)
    ///
    /// This bit is cleared by software reading SR1 register followed
    /// reading SR2, or by hardware when PE=0.
    ///
    /// ## Address matched (Slave)
    /// 0: Address mismatched or not received.
    /// 1: Received address matched.
    ///
    /// - Set by hardware as soon as the received slave address
    /// matched with the OAR registers content or a general call or a
    /// SMBus Device Default Address or SMBus Host or SMBus Alert is
    /// recongnized. (When enabled depending on configuration.)
    ///
    /// Note: In slave more, it is recommended to perform the complete
    /// clearing sequence (READ SR1 then READ SR2) after ADDR is set.
    ///
    /// ## Address sent (Master)
    /// 0: No end of address transmission
    /// 1: End of address transmission
    ///
    /// - For 10-bit addressing, the bit is set after the ACK of the
    /// 2nd byte.
    /// - For 7-bit addressing, the bit is set after the ACK of the
    /// byte.
    ///
    /// Note: ADDR is not set after a NACK reception.
    ADDR = 0x1 << 1,

    /// Byte transfer finished
    ///
    /// 0: Data byte transfer not done
    /// 1: Data byte transfer secceeded
    ///
    /// - Set by hardware when NOSTRETCH=0 and:
    /// - In reception when a new byte is received (including ACK
    /// pulse) and DR has not been read yet (RxNE=1).
    /// - In transmission when a new byte should be sent and DR has
    /// not been written yet (TxE=1).
    /// - Cleared by software by either a read or write in the DR
    /// register or by hardware after a start or a stop condition in
    /// transmission of when PE=0.
    ///
    /// Note: The BTF bit is not set after a NACK reception. The BTF
    /// bit is not set if next byte to be transmitted is the PEC
    /// (TRA=1 in SR2 register and PEC=1 in CR1 register).
    BTF = 0x1 << 2,

    /// 10-bit header sent (Master mode)
    ///
    /// 0: No ADD10 event occured
    /// 1: Master has sent first address byte (header)
    ///
    /// - Set by hardware when the master has sent the first byte in
    /// 10-bit address mode.
    /// - Cleared by software reading the SR1 register followed by a
    /// write in the DR register of the second address byte, or by
    /// hardware when PE=0.
    ///
    /// Note: ADD10 bit is not set after a NACK reception.
    ADD10 = 0x1 << 3,

    /// Stop detection (slave mode)
    STOPF = 0x1 << 4,

    // 5: Reserved

    /// Data register not empty (receivers)
    RxNE = 0x1 << 6,

    /// Data register empty (transmitters)
    TxE = 0x1 << 7,

    /// Bus error
    BERR = 0x1 << 8,

    /// Arbitration lost (master mode)
    ARLO = 0x1 << 9,

    /// Acknowledge failure
    AF = 0x1 << 10,

    /// Overrun/Underrun
    OVR = 0x1 << 11,

    /// PEC Error in reception
    PECERR = 0x1 << 12,

    // 13: Reserved

    /// Timeout or Tlow error
    TIMEOUT = 0x1 << 14,

    /// SMBus alert
    SMBALERT = 0x1 << 15,
}

#[allow(non_camel_case_types)]
#[allow(dead_code)]
#[derive(Copy, Clone)]
#[repr(u32)]
enum Sr2Masks {
    /// Master/slave
    ///
    /// 0: Slave Mode
    /// 1: Master Mode
    MSL = 0x1 << 0,

    /// Bus busy
    BUSY = 0x1 << 1,

    /// Transmitter/receiver
    ///
    /// 0: Data bytes received
    /// 1: Data bytes transmitted
    TRA = 0x1 << 2,

    // 3: Reserved

    /// Generall call address (Slave mode)
    GENCALL = 0x1 << 4,

    /// SMBus device default address (Slave mode)
    SMBDEFAULT = 0x1 << 5,

    /// SMBus host header (Slave mode)
    SMBHOST = 0x1 << 6,

    /// Dual flag (Slave mode)
    DUALF = 0x1 << 7,

    /// Packet error checking register.
    ///
    /// This register contains the internal PEC when ENPEC=1.
    PEC = 0xFF << 8,
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

#[allow(non_camel_case_types)]
#[allow(dead_code)]
#[derive(Copy, Clone)]
#[repr(u32)]
pub enum Interrupt {
    Buf = 0x0400,
    Evt = 0x0200,
    Err = 0x0100,
}

#[allow(non_camel_case_types)]
#[allow(dead_code)]
#[derive(Copy, Clone)]
#[repr(u32)]
pub enum InterruptFlag {
    SMBALERT = 0x01008000,
    TIMEOUT  = 0x01004000,
    PECERR   = 0x01001000,
    OVR      = 0x01000800,
    AF       = 0x01000400,
    ARLO     = 0x01000200,
    BERR     = 0x01000100,
    TXE      = 0x06000080,
    RXNE     = 0x06000040,
    STOPF    = 0x02000010,
    ADD10    = 0x02000008,
    BTF      = 0x02000004,
    ADDR     = 0x02000002,
    SB       = 0x02000001,
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

#[derive(Copy, Clone)]
#[repr(u32)]
pub enum Event {
    /// Communication start.
    ///
    /// After sending the START condition, the master has to wait for
    /// this event. It means that the Start condition has been
    /// correctly released on the I2C bus (the bus is free, no other
    /// devices is communicating).
    // EV5
    MasterModeSelect = 0x00030001, // BUSY, MSL, SB

    /// Address acknowledge.
    ///
    /// After checking on EV5 (start condition correctly released on
    /// the bus), the master sends the address of the slave(s) with
    /// which it will communicate. Then the master has to wait that a
    /// slave acknowledges his address. If an acknowledge is sent on
    /// the bus, one of the following event will be set:
    ///
    /// 1. In case of Master Receiver (7-bit addressing): the
    /// MasterReceiverModeSelected event is set.
    /// 2. In case of Master Transmitter (7-bit addressing): the
    /// MasterTransmitterModeSelected event is set.
    /// 3. In case of 10-bit addressing mode, the master (just after
    /// generating the START and checking on EV5) has to send the
    /// header of 10-bit addressing mode (I2c::send_data
    /// function). Then master should wait on EV9. It means that the
    /// 10-bit addressing header has been correctly sent on the
    /// bus. Then master should send the second part of the 10-bit
    /// address (LSB) using the function
    /// I2c::send_7bit_address(). Then master should wait for event
    /// EV6.
    // EV6
    MasterTransmitterModeSelected = 0x00070082, // BUSY, MSL, ADDR, TXE, TRA
    MasterReceiverModeSelected = 0x00030002, // BUSY, MSL, ADDR
    // EV9
    MasterModeAddress10 = 0x00030008, // BUSY, MSL, ADD10

    /// Communication events.
    ///
    /// If a communication is established (START condition generated
    /// and slave address acknowledged) then the master has to check
    /// on one of the following events for communication procedures:
    ///
    /// 1. Master Receiver mode: The master has to wait on the event
    ///    EV7 then to read the data received from the slave
    ///    (I2c::receive_data() function).
    ///
    /// 2. Master Transmitter mode: The master has to send data
    ///    (I2c::send_data() function) then to wait on event EV8 or
    ///    EV8_2.
    ///
    ///    These two events are similar:
    ///
    ///    - EV8 means that the data has been written it the data
    ///    register and is being shifted out.
    ///    - EV8_2 means that the data has been physically shifted out
    ///    and output on the bus.
    ///
    ///    In most cases, using EV8 is sufficient for the
    ///    application. EV8_2 leads to a slower communication but
    ///    ensure more reliable test. EV8_2 is also more suitable that
    ///    EV8 for testing on the last data transmission (before Stop
    ///    condition generation).
    ///
    /// In case the user software does not guarantee that this event
    /// EV7 is managed before the current byte end of transfer, then
    /// user may check on EV7 and BTF flag at the same time
    /// (i.e. MasterByteReceived | FlagBtf). In this case the
    /// communication may be slower.
    // Master RECEIVER mode
    // EV7
    MasterByteReceived = 0x00030040, // BUSY, MSL, RXNE

    // Master TRANSMITTER mode
    // EV8
    MasterByteTransmitting = 0x00070080, // TRA, BUSY, MSL, TXE
    MasterByteTransmitted = 0x00070084, // TRA, BUSY, MSL, TXE, BTF

    /// Communication start events.
    ///
    /// Wait on one of these events at the start of the
    /// communication. It means that the I2C periperal detected a
    /// Start condition on the bus (generated by master device)
    /// followed by the peripheral address. The peripheral generates
    /// an ACK condition on the bus (if the acknowledge feature is
    /// enabled) and the events listed above are set:
    ///
    /// 1. In normal case (only one address managed by the slave),
    /// when the address sent by the master matches the own address of
    /// the peripheral (configured by I2c::own_address1 field) the
    /// SlaveXxxAddressMatched event is set (where Xxx could be
    /// Transmitter or Receiver).
    /// 2. In case the address sent by the master matches the second
    /// address of the peripheral the events
    /// SlaveXxxSecondAddressMatched (where Xxx could be Transmitter
    /// or Receiver) are set.
    /// 3. In case the address sent by the master is General Call
    /// (address 0x00) and if the Generall Call is enabled for the
    /// peripheral the following event is set
    /// SlaveGenerallCallAddressMatched.
    // EV1 (all the events below are variants of EV1)
    // 1. Case of One Single Address managed by the slave
    SlaveReceiverAddressMatched = 0x00020002, // BUSY, ADDR
    SlaveTransmitterAddressMatched = 0x00060082, // TRA, BUSY, TXE, ADDR
    // 2. Case of Dual address managed by the slave
    SlaveReceiverSecondAddressMatched = 0x00820000, // DUALF, BUSY
    SlaveTransmitterSecondAddressMatched = 0x00860080, // DUALF, TRA, BUSY, TXE
    // 3. Case of Generall Call enabled for the slave
    SlaveGenerallCallAddressMatched = 0x00120000, // GENCALL, BUSY

    /// Communication events.
    ///
    /// Wait on one of these when EV1 has already been checked and:
    ///
    /// - Slave Receiver mod:
    ///   - EV2: When the application is expecting a data byte to be
    ///   received.
    ///   - EV4: When the application is expecting the end of the
    ///   communication: master sends a stop condition and data
    ///   transmission is stopped.
    ///
    /// - Slave Transmitter mode:
    ///   - EV3: When a byte has been transmitted by the slave an the
    ///   application is expecting the end of the byte
    ///   transmission. The two events SlaveByteTransmitted and
    ///   SlaveByteTransmitting are similar. The second one can
    ///   optionally be used when the user software doesn't guarantee
    ///   the EV3 is managed before the current byte end of transfer.
    ///   - EV3_2: When the master sends a NACK in order to tell slave
    ///   that ddata transmission shall end (before sending the STOP
    ///   condition). In this case slave has to stop sending data
    ///   bytes and expect a Stop condition on the bus.
    ///
    /// Note: In case the user software does not guarantee that event
    /// EV2 is managed before the current byte end of transfer, then
    /// user may check on EV2 and BTF flag at the same time. In this
    /// case the communication may be slower.
    // Slave Receiver mode
    // EV2
    SlaveByteReceived = 0x00020040, // BUSY, RXNE
    // EV4
    SlaveStopDetected = 0x00000010, // STOPF
    // Slave Transmitter mode
    // EV3
    SlaveByteTransmitted = 0x00060084, // TRA, BUSY, TXE, BTF
    SlaveByteTransmitting = 0x00060080, // TRA, BUSY, TXE
    // EV3_2
    SlaveAckFailure = 0x00000400, // AF
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

    /// Generates I2C communication start condition.
    pub unsafe fn generate_start(&self) {
        self.cr1.set_flag(Cr1Masks::START as u32);
    }

    pub unsafe fn generate_stop(&self) {
        self.cr1.set_flag(Cr1Masks::STOP as u32);
    }

    pub unsafe fn send_7bit_address(&self, address: u8, direction: Direction) {
        self.dr.set(match direction {
            Direction::Transmitter => address | 0x1,
            Direction::Receiver => address & !0x1,
        } as u32);
    }

    pub unsafe fn send_data(&self, data: u8) {
        self.dr.set(data as u32);
    }

    pub unsafe fn receive_data(&self) -> u8 {
        self.dr.get() as u8
    }

    /// Returns the image of both status registers in a single word
    /// (u32) (SR2 value is shiftedd left by 16 bits and concatenated
    /// to SR1).
    ///
    /// The result event could be checked against `Event` enum.
    pub unsafe fn get_last_event(&self) -> u32 {

        // Do NOT inline these reads. They should be done in that
        // order.
        let sr1 = self.sr1.get();
        let sr2 = self.sr2.get();
        (sr1 | (sr2 << 16)) & FLAG_MASK
    }

    pub unsafe fn it_enable(&self, it: Interrupt) {
        self.cr2.set_flag(it as u32);
    }

    pub unsafe fn it_disable(&self, it: Interrupt) {
        self.cr2.clear_flag(it as u32);
    }

    /// Checks whether the specified I2C interrupt has occurred or
    /// not.
    pub unsafe fn it_status(&self, it: InterruptFlag) -> bool {
        const ITEN_MASK: u32 = 0x07000000; // I2C Interrupt Enable mask

        // Check if the interrupt source is enabled or not
        let enablestatus = (((it as u32) & ITEN_MASK) >> 16) & self.cr2.get();

        // Get bit[23:0] of the flag */
        let it = (it as u32) & FLAG_MASK;

        // Check the status of the specified I2C flag */
        ((self.sr1.get() as u32) & it) != 0 && enablestatus != 0
    }

    /// Clears the I2C's pending flags.
    ///
    /// `flag` specifies the flag to clear. This parameter can be any
    /// combination of the following values:
    ///
    /// - Sr1Masks::SMBALERT: SMBus Alert flag
    /// - Sr1Masks::TIMEOUT: Timeout or Tlow error flag
    /// - Sr1Masks::PECERR: PEC error in reception flag
    /// - Sr1Masks::OVR: Overrun/Underrun flag (Slave mode)
    /// - Sr1Masks::AF: Acknowledge failure flag
    /// - Sr1Masks::ARLO: Arbitration lost flag (Master mode)
    /// - Sr1Masks::BERR: Bus error flag
    ///
    /// STOPF (STOP detection) is cleared by software sequence: a read
    /// operation to SR1 register followed by a write operation to CR1
    /// register (cmd() to re-enable the I2C peripheral).
    ///
    /// ADD10 (10-bit header sent) is cleared by software sequence: a
    /// read operation to SR1 followed by writing the second byte of
    /// the address in DR register.
    ///
    /// BTF (Byte Transfer Finished) is cleared by software sequence:
    /// a read operation to SR1 register followed by a read/write to
    /// DR register (send_data()).
    ///
    /// ADDR (Address sent) is cleared by software sequence: a read
    /// operation to SR1 register followed by a read operation to SR2
    /// register.
    ///
    /// SB (Start Bit) is cleared by software sequence: a read
    /// operation to SR1 register followed by a write operation to DR
    /// register (send_data()).
    pub unsafe fn it_clear_pending(&self, flag: u32) {
        self.sr1.clear_flag((flag as u32) & FLAG_MASK);
    }
}
