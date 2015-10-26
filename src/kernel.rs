//! This crate is a Rust part of the kernel. It should be linked with
//! the bootstrap that will jump to the `kmain` function.
#![crate_type = "staticlib"]

#![feature(no_std)]
#![no_std]

extern crate stm32f4;

/// The main entry of the kernel.
#[no_mangle]
pub extern fn kmain() -> ! {
    stm32f4::init_usart1();

    stm32f4::puts("Hello, world!\r\n");

    loop {}
}

