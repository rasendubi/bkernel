//! This crate is a Rust part of the kernel. It should be linked with
//! the bootstrap that will jump to the `kmain` function.
#![crate_type = "staticlib"]

#![cfg_attr(doc, allow(unused_features))]

#![feature(no_std)]
#![feature(core_intrinsics, core_str_ext)]
#![cfg_attr(not(target_os = "none"), feature(core))]
#![feature(lang_items)]

#![cfg_attr(target_os = "none", no_std)]

pub mod runtime;

#[cfg(not(target_os = "none"))]
extern crate core;

// We export volatile as pub for doc to document registers! macro
#[cfg(doc)]
#[macro_use]
pub mod volatile;

#[cfg(not(doc))]
#[macro_use]
mod volatile;

mod stm32f4;

/// The main entry of the kernel.
#[no_mangle]
pub extern fn kmain() -> ! {
    stm32f4::init_usart1();

    stm32f4::puts("Hello, world!\r\n");

    loop {}
}

