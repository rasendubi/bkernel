//! Future-based drivers for hardware peripherals.
#![no_std]
#![feature(const_fn)]
#![feature(integer_atomics)]
#![feature(conservative_impl_trait)]

extern crate breactor;
#[macro_use]
extern crate futures;
extern crate stm32f4;

pub mod i2c;
pub mod rng;
