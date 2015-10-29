//! STM32F4xx drivers.
#![crate_name = "stm32f4"]
#![crate_type = "lib"]

#![cfg_attr(test, allow(unused_features))]

#![feature(lang_items)]
#![feature(no_std)]
#![feature(core_intrinsics, core_str_ext)]

#![no_std]

#![allow(dead_code)]

#[macro_use]
pub mod volatile;
pub mod rcc;
pub mod gpio;
pub mod usart;

pub mod lang_items;
