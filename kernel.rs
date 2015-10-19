#![crate_type = "staticlib"]

#![feature(no_std)]
#![feature(core)]
#![feature(lang_items)]

#![no_std]

extern crate core;

pub mod runtime;

#[macro_use]
mod volatile;
mod stm32f4;

#[no_mangle]
pub extern fn kmain() -> ! {
    stm32f4::init_usart1();

    stm32f4::puts("Hello, world!\r\n");

    loop {}
}

