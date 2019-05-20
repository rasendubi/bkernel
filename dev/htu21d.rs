//! HTU21D temperature and humidity sensor.
//!
//! This module provides a driver for
//! [HTU21D](https://cdn-shop.adafruit.com/datasheets/1899_HTU21D.pdf)
//! sensor.
use super::i2c;

use core::marker::PhantomData;
use core::pin::Pin;
use core::task::Context;

use futures::{Future, Poll};

#[allow(missing_debug_implementations)]
pub struct Htu21d {
    i2c: &'static i2c::I2cBus,
}

impl Htu21d {
    pub const fn new(i2c: &'static i2c::I2cBus) -> Htu21d {
        Htu21d { i2c }
    }

    pub fn soft_reset(&'static self) -> Htu21dCommand<NoHoldMaster, Reset> {
        Htu21dCommand::StartTransfer(self.i2c.start_transfer(), SOFT_RESET_CMD.as_ptr())
    }

    pub fn read_temperature_hold_master(&'static self) -> Htu21dCommand<HoldMaster, Temperature> {
        Htu21dCommand::StartTransfer(
            self.i2c.start_transfer(),
            READ_TEMP_HOLD_MASTER_CMD.as_ptr(),
        )
    }

    pub fn read_humidity_hold_master(&'static self) -> Htu21dCommand<HoldMaster, Humidity> {
        Htu21dCommand::StartTransfer(self.i2c.start_transfer(), READ_HUM_HOLD_MASTER_CMD.as_ptr())
    }
}

/// A marker for a measurement that holds master.
#[derive(Debug)]
pub struct HoldMaster;

/// A marker for a measurement that does not holds master.
#[derive(Debug)]
pub struct NoHoldMaster;

#[derive(Debug, Copy, Clone)]
pub struct Reset;

#[derive(Debug, Copy, Clone)]
pub struct Temperature(u16);

impl Temperature {
    /// Return raw sample from the sensor.
    ///
    /// The conversion formula must be applied to receive degrees
    /// celsius.
    pub const fn raw(self) -> u16 {
        self.0
    }

    /// Return temperature in degrees celsius.
    #[allow(clippy::float_arithmetic)]
    // f32::from is not constant
    #[allow(clippy::cast_lossless)]
    pub const fn celsius(self) -> f32 {
        -46.85 + 175.72 * ((self.0 & !0x3) as f32) / ((1 << 16) as f32)
    }

    /// Temperature in milliseconds.
    // i64::from is not constant
    #[allow(clippy::cast_lossless)]
    pub const fn millicelsius(self) -> i64 {
        -46_850 + ((175_720 * ((self.0 & !0x3) as i64)) >> 16)
    }
}

impl From<u16> for Temperature {
    fn from(sample: u16) -> Temperature {
        Temperature(sample)
    }
}

impl ::core::fmt::Display for Temperature {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> Result<(), ::core::fmt::Error> {
        let mc = self.millicelsius();
        write!(f, "{}.{:03}", mc / 1000, mc % 1000)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Humidity(u16);

impl Humidity {
    pub const fn raw(self) -> u16 {
        self.0
    }

    #[allow(clippy::float_arithmetic)]
    // f32::from is not constant
    #[allow(clippy::cast_lossless)]
    pub const fn percents(self) -> f32 {
        -6.0 + 125.0 * ((self.0 & !0x3) as f32) / ((1 << 16) as f32)
    }

    // i64::from is not constant
    #[allow(clippy::cast_lossless)]
    pub const fn millipercents(self) -> i64 {
        -6_000 + ((125_000 * ((self.0 & !0x3) as i64)) >> 16)
    }
}

impl From<u16> for Humidity {
    fn from(sample: u16) -> Humidity {
        Humidity(sample)
    }
}

impl ::core::fmt::Display for Humidity {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> Result<(), ::core::fmt::Error> {
        let mp = self.millipercents();
        write!(f, "{}.{:03}", mp / 1000, mp % 1000)
    }
}

#[derive(Debug)]
pub enum Htu21dError {
    LockError,
    I2cError(i2c::Error),
}

impl From<()> for Htu21dError {
    fn from(_: ()) -> Htu21dError {
        Htu21dError::LockError
    }
}

impl From<i2c::Error> for Htu21dError {
    fn from(err: i2c::Error) -> Htu21dError {
        Htu21dError::I2cError(err)
    }
}

const HTU21D_ADDRESS: u16 = 0x80;

const READ_TEMP_HOLD_MASTER_CMD: [u8; 1] = [0xE3];
const READ_HUM_HOLD_MASTER_CMD: [u8; 1] = [0xE5];
#[allow(dead_code)]
const READ_TEMP_NO_HOLD_MASTER_CMD: [u8; 1] = [0xF3];
#[allow(dead_code)]
const READ_HUM_NO_HOLD_MASTER_CMD: [u8; 1] = [0xF5];
#[allow(dead_code)]
const WRITE_USER_CMD: [u8; 1] = [0xE6];
#[allow(dead_code)]
const READ_USER_CMD: [u8; 1] = [0xE7];
const SOFT_RESET_CMD: [u8; 1] = [0xFE];

static mut __READ_BUFFER: [u8; 3] = [0; 3];

#[allow(missing_debug_implementations)]
pub enum Htu21dCommand<H, R> {
    StartTransfer(i2c::StartTransferFuture, *const u8),
    CmdTransmission(i2c::Transmission<'static>),
    ResultTransmission(i2c::Transmission<'static>),
    Done(u16, PhantomData<(H, R)>),
}

impl<H, R> Unpin for Htu21dCommand<H, R> {}

impl<T> Future for Htu21dCommand<HoldMaster, T>
where
    T: From<u16> + Copy,
{
    type Output = Result<T, Htu21dError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<T, Htu21dError>> {
        use self::Htu21dCommand::*;

        let this = &mut *self;

        loop {
            *this = match this {
                StartTransfer(ref mut start_transfer, ref cmd) => {
                    let i2c = ready!(Pin::new(start_transfer).poll(cx));
                    CmdTransmission(i2c.master_transmitter_raw(HTU21D_ADDRESS, *cmd, 1))
                }
                CmdTransmission(ref mut transmission) => {
                    let (i2c, _buf) = try_ready!(Pin::new(transmission).poll(cx));
                    ResultTransmission(i2c.master_receiver_raw(
                        HTU21D_ADDRESS,
                        unsafe { &mut __READ_BUFFER }.as_mut_ptr(),
                        unsafe { &__READ_BUFFER }.len(),
                    ))
                }
                ResultTransmission(ref mut transmission) => {
                    let (mut i2c, buf) = try_ready!(Pin::new(transmission).poll(cx));
                    i2c.stop();
                    Done((u16::from(buf[0]) << 8) | u16::from(buf[1]), PhantomData)
                }
                Done(sample, _) => {
                    return Poll::Ready(Ok(<T>::from(*sample)));
                }
            };
        }
    }
}

impl Future for Htu21dCommand<NoHoldMaster, Reset> {
    type Output = Result<Reset, Htu21dError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<Reset, Htu21dError>> {
        use self::Htu21dCommand::*;

        let this = &mut *self;

        loop {
            *this = match this {
                StartTransfer(ref mut start_transfer, ref cmd) => {
                    let transfer = ready!(Pin::new(start_transfer).poll(cx));
                    CmdTransmission(transfer.master_transmitter_raw(HTU21D_ADDRESS, *cmd, 1))
                }
                CmdTransmission(ref mut transmission) => {
                    let (mut i2c, _buf) = try_ready!(Pin::new(transmission).poll(cx));
                    i2c.stop();
                    Done(0, PhantomData)
                }
                Done(_, _) => {
                    return Poll::Ready(Ok(Reset));
                }
                _ => unsafe {
                    ::core::intrinsics::unreachable();
                },
            };
        }
    }
}
