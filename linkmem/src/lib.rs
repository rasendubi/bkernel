//! This module is a rust interface to smalloc.
//!
//! To get more info on custom allocators see:
//! https://doc.rust-lang.org/nightly/book/custom-allocators.html

#![no_std]

extern crate smalloc;

use smalloc::Smalloc;

#[cfg_attr(not(test), global_allocator)]
static mut ALLOCATOR: Smalloc = Smalloc {
    start: 0 as *mut u8,
    size: 0,
};

pub fn init(alloc: Smalloc) {
    unsafe {
        ALLOCATOR = alloc;
        ALLOCATOR.init();
    }
}
