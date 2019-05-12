//! Future-based drivers for hardware peripherals.
#![cfg_attr(not(test), no_std)]
#![feature(const_fn)]
#![feature(integer_atomics)]
#![feature(core_intrinsics)]
#![feature(fixed_size_array)]

extern crate breactor;
#[macro_use]
extern crate futures;
extern crate stm32f4;

mod circular_buffer;
mod resettable_stream;
#[cfg(test)]
mod debug;

pub mod cs43l22;
pub mod esp8266;
pub mod htu21d;
pub mod i2c;
pub mod rng;
pub mod usart;
