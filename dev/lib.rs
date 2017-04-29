//! Future-based drivers for hardware peripherals.
#![no_std]
#![feature(const_fn)]
#![feature(integer_atomics)]

extern crate breactor;
extern crate futures;
extern crate stm32f4;

pub mod i2c;
pub mod rng;
