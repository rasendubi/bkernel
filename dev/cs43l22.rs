//! CS43L22 Low Power, Stereo DAC with Headphone and Speaker Amplifiers.
use crate::i2c;

use futures::{Future, FutureExt, TryFutureExt};

#[allow(missing_debug_implementations)]
pub struct Cs43l22 {
    i2c: &'static i2c::I2cBus,
    i2c_addr: u16,
    buffer: [u8; 8],
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum Error {
    /// An I2C error has occured.
    I2cError(i2c::Error),
}

impl From<i2c::Error> for Error {
    fn from(err: i2c::Error) -> Error {
        Error::I2cError(err)
    }
}

#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[repr(u8)]
enum Register {
    // 0x0: Reserved
    ID = 0x1,
    PowerCtl1 = 0x2,
    // 0x3: Reserved
    PowerCtl2 = 0x4,
    ClockingCtl = 0x5,
    InterfaceCtl1 = 0x6,
    InterfaceCtl2 = 0x7,
    PassthroughASelect = 0x8,
    PassthroughBSelect = 0x9,
    AnalogZCAndSRSettings = 0xA,
    // 0xB: Reserved
    PassthroughGangControl = 0xC,
    PlaybackCtl1 = 0xD,
    MiscCtl = 0xE,
    PlaybackCtl2 = 0xF,
    // 0x10 -- 0x13: Reserved
    PassthroughAVol = 0x14,
    PassthroughBVol = 0x15,
    // 0x16 -- 0x19: Reserved
    PCMAVol = 0x1A,
    PCMBVol = 0x1B,
    BEEPFreq_OnTime = 0x1C,
    BEEPFVol_OffTime = 0x1D,
    BEEP_ToneCfg = 0x1E,
    ToneCtl = 0x1F,
    MasterAVol = 0x20,
    MasterBVol = 0x21,
    HeadphoneAVol = 0x22,
    HeadphoneBVol = 0x23,
    SpeakerAVol = 0x24,
    SpeakerBVol = 0x25,
    ChannelMixer_Swap = 0x26,
    LimitCtl1_Thresholds = 0x27,
    LimitCtl2_ReleaseRate = 0x28,
    LimiterAttackRate = 0x29,
    // 0x2A -- 0x2D: Reserved
    Overflow_ClockStatus = 0x2E,
    BatteryCompensation = 0x2F,
    VPBatteryLevel = 0x30,
    SpeakerStatus = 0x31,
    // 0x32 -- 0x33: Reserved
    ChargePumpFrequency = 0x34,
}

impl Cs43l22 {
    /// Create new Cs43l22 instance.
    ///
    /// `ad0` is the LSB of the chip address.
    ///
    /// ## Example
    /// ```no_run
    /// let cs43l22 = dev::cs43l22::Cs43l22::new(&dev::i2c::I2C1_BUS, false);
    /// ```
    pub const fn new(i2c: &'static i2c::I2cBus, ad0: bool) -> Cs43l22 {
        Cs43l22 {
            i2c,
            i2c_addr: 0b1001_0100 | ((ad0 as u16) << 1),
            buffer: [0; 8],
        }
    }

    pub fn get_chip_id(&'static mut self) -> impl Future<Output = Result<u8, Error>> + 'static {
        let addr = self.i2c_addr;

        self.buffer[0] = 0x01; // ID register
        let buffer = self.buffer.as_mut_ptr();

        self.i2c
            .start_transfer()
            .then(move |i2c| i2c.master_transmitter_raw(addr, buffer, 1))
            .and_then(move |(i2c, _buffer)| i2c.master_receiver_raw(addr, buffer, 1))
            .map_ok(|(mut i2c, buffer)| {
                i2c.stop();
                buffer[0]
            })
            .map_err(|err| Error::I2cError(err))
    }
}
