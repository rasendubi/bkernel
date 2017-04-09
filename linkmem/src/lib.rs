//! This module is a rust interface to smalloc.
//!
//! To get more info on custom allocators see:
//! https://doc.rust-lang.org/nightly/book/custom-allocators.html

#![feature(allocator)]
#![allocator]
#![no_std]

extern crate smalloc;

use smalloc::Smalloc;
use core::intrinsics::copy_nonoverlapping;

// TODO(rasen): allow importing this from different module (use weak linkage or extern)
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

#[no_mangle]
pub unsafe extern fn __rust_allocate(size: usize, _align: usize) -> *mut u8 {
    ALLOCATOR.alloc(size)
}

#[no_mangle]
pub unsafe extern fn __rust_deallocate(ptr: *mut u8, _old_size: usize, _align: usize) {
    ALLOCATOR.free(ptr)
}

// TODO(tailhook): optimize me
#[no_mangle]
pub unsafe extern fn __rust_reallocate(ptr: *mut u8, old_size: usize, size: usize,
                                       _align: usize) -> *mut u8 {
    let new = ALLOCATOR.alloc(size);
    if new.is_null() {
        return ::core::ptr::null_mut();
    }

    copy_nonoverlapping(ptr, new, old_size);
    ALLOCATOR.free(ptr);
    new
}

#[no_mangle]
pub extern fn __rust_reallocate_inplace(_ptr: *mut u8, old_size: usize,
                                        _size: usize, _align: usize) -> usize {
    old_size
}

#[no_mangle]
pub extern fn __rust_usable_size(size: usize, _align: usize) -> usize {
    size
}
