//! Future-based drivers for hardware peripherals.
#![no_std]
#![feature(const_fn)]
#![feature(integer_atomics)]
#![feature(conservative_impl_trait)]
#![feature(core_intrinsics)]
#![feature(fixed_size_array)]

extern crate breactor;
#[macro_use]
extern crate futures;
extern crate stm32f4;

pub mod circular_buffer;

pub mod usart;
pub mod i2c;
pub mod rng;
pub mod htu21d;
pub mod cs43l22;
