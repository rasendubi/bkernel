//! HTU21D temperature and humidity sensor.
//!
//! This module provides a driver for
//! [HTU21D](https://cdn-shop.adafruit.com/datasheets/1899_HTU21D.pdf)
//! sensor.
use super::i2c;

use ::futures::future::{self, Future, Either};

pub struct Htu21d {
    i2c: &'static i2c::I2cBus,
}

#[derive(Debug)]
pub struct Temperature(u16);

impl Temperature {
    /// Return raw sample from the sensor.
    ///
    /// The conversion formula must be applied to receive degrees
    /// celsius.
    pub const fn raw(&self) -> u16 {
        self.0
    }

    /// Return temperature in degrees celsius.
    pub const fn celsius(&self) -> f32 {
        -46.85 + 175.72 * ((self.0 & !0x3) as f32) / ((1 << 16) as f32)
    }

    /// Temperature in milliseconds.
    pub const fn millicelsius(&self) -> i64 {
        -46_850 + ((175_720 * ((self.0 & !0x3) as i64)) >> 16)
    }
}

impl ::core::fmt::Display for Temperature {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> Result<(), ::core::fmt::Error> {
        let mc = self.millicelsius();
        write!(f, "{}.{:03}", mc/1000, mc%1000)
    }
}

#[derive(Debug)]
pub struct Humidity(u16);

impl Humidity {
    pub const fn raw(&self) -> u16 {
        self.0
    }

    pub const fn percents(&self) -> f32 {
        -6.0 + 125.0*((self.0 & !0x3) as f32)/((1 << 16) as f32)
    }

    pub const fn millipercents(&self) -> i64 {
        -6_000 + (125_000*((self.0 & !0x3) as i64) >> 16)
    }
}

impl ::core::fmt::Display for Humidity {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> Result<(), ::core::fmt::Error> {
        let mp = self.millipercents();
        write!(f, "{}.{:03}", mp/1000, mp%1000)
    }
}

#[derive(Debug)]
pub enum Htu21dError {
    LockError,
    I2cError(u32),
}

const HTU21D_ADDRESS: u16 = 0x80;

const SOFT_RESET_CMD: [u8; 1] = [ 0xFE ];
const READ_TEMP_HOLD_MASTER_CMD: [u8; 1] = [ 0xE3 ];
const READ_HUM_HOLD_MASTER_CMD: [u8; 1] = [ 0xE5 ];

/// DO NOT USE.
///
/// It is public only to satisfy linker, so it doesn't delete the
/// variable.
pub static mut __READ_BUFFER: [u8; 3] = [0; 3];

impl Htu21d {
    pub const fn new(i2c: &'static i2c::I2cBus) -> Htu21d {
        Htu21d {
            i2c,
        }
    }

    pub fn soft_reset(&'static self) -> impl Future<Item=(), Error=Htu21dError> + 'static {
        self.i2c.start_transfer()
            .then(|res| {
                match res {
                    Ok(i2c) => Either::A(
                        i2c.master_transmitter_raw(
                            HTU21D_ADDRESS,
                            SOFT_RESET_CMD.as_ptr(),
                            SOFT_RESET_CMD.len()
                        )
                            .map(|(mut i2c, _buf)| {
                                i2c.stop();
                            })
                            .map_err(Htu21dError::I2cError)),
                    Err(_) => Either::B(future::err(Htu21dError::LockError)),
                }
            })
    }

    pub fn read_temperature_hold_master(&'static self) -> impl Future<Item=Temperature, Error=Htu21dError> + 'static {

        self.i2c.start_transfer()
            .then(|res| {
                match res {
                    Ok(i2c) => Either::A(
                        i2c.master_transmitter_raw(
                            HTU21D_ADDRESS,
                            READ_TEMP_HOLD_MASTER_CMD.as_ptr(),
                            READ_TEMP_HOLD_MASTER_CMD.len()
                        ).map_err(Htu21dError::I2cError)),
                    Err(_) => Either::B(future::err(Htu21dError::LockError)),
                }
            })
            .and_then(|(i2c, _buf)| {
                i2c.master_receiver_raw(
                    HTU21D_ADDRESS,
                    unsafe{&mut __READ_BUFFER}.as_mut_ptr(),
                    unsafe{&__READ_BUFFER}.len()
                ).map_err(Htu21dError::I2cError)
            })
            .and_then(|(mut i2c, buf)| {
                i2c.stop();

                Ok(Temperature(((buf[0] as u16) << 8) | (buf[1] as u16)))
            })
    }

    pub fn read_humidity_hold_master(&'static self) -> impl Future<Item=Humidity, Error=Htu21dError> + 'static {
        self.i2c.start_transfer()
            .then(|res| {
                match res {
                    Ok(i2c) => Either::A(
                        i2c.master_transmitter_raw(
                            HTU21D_ADDRESS,
                            READ_HUM_HOLD_MASTER_CMD.as_ptr(),
                            READ_HUM_HOLD_MASTER_CMD.len()
                        ).map_err(Htu21dError::I2cError)),
                    Err(_) => Either::B(future::err(Htu21dError::LockError)),
                }
            })
            .and_then(|(i2c, _buf)| {
                i2c.master_receiver_raw(
                    HTU21D_ADDRESS,
                    unsafe{&mut __READ_BUFFER}.as_mut_ptr(),
                    unsafe{&__READ_BUFFER}.len()
                ).map_err(Htu21dError::I2cError)
            })
            .and_then(|(mut i2c, buf)| {
                i2c.stop();

                Ok(Humidity(((buf[0] as u16) << 8) | (buf[1] as u16)))
            })
    }
}
