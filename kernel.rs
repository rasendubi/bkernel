#![crate_type = "staticlib"]

#![feature(no_std)]
#![feature(core)]
#![feature(lang_items)]

#![no_std]

pub mod runtime;

extern crate core;

use core::str::*;

use core::intrinsics::volatile_store;

#[no_mangle]
pub extern fn kmain() -> ! {
    puts("Hello, world!\n");
    loop {}
}

fn puts(s: &str) {
    const REG: *mut u8 = 0x101f1000 as *mut u8;

    for c in s.bytes() {
        unsafe {
            volatile_store(REG, c);
        }
    }
}

