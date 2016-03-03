//! STM32F4xx drivers.
#![crate_name = "stm32f4"]
#![crate_type = "lib"]

#![cfg_attr(test, allow(unused_features))]

#![feature(lang_items)]
#![feature(core_intrinsics)]

#![no_std]

#![allow(dead_code)]

pub mod isr_vector;

#[macro_use]
pub mod volatile;
pub mod rcc;
pub mod gpio;
pub mod usart;
pub mod timer;

pub mod lang_items;
