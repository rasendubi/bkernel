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


static mut allocator: Smalloc = Smalloc {
    start: 0 as *mut u8,
    size: 0,
};

pub fn init(alloc: Smalloc) {
    unsafe {
        allocator = alloc;
        allocator.init();
    }
}

#[no_mangle]
pub extern fn __rust_allocate(size: usize, _align: usize) -> *mut u8 {
    unsafe { allocator.alloc(size) }
}

#[no_mangle]
pub extern fn __rust_deallocate(ptr: *mut u8, _old_size: usize, _align: usize) {
    unsafe { allocator.free(ptr) }
}

#[no_mangle]
pub extern fn __rust_reallocate(ptr: *mut u8, old_size: usize, size: usize,
                                _align: usize) -> *mut u8 {
    // TODO(tailhook): optimize me
    unsafe {
        let nval = allocator.alloc(size);
        copy_nonoverlapping(ptr, nval, old_size);
        allocator.free(ptr);
        return nval;
    }
}

#[no_mangle]
pub extern fn __rust_reallocate_inplace(_ptr: *mut u8, old_size: usize,
                                        _size: usize, _align: usize) -> usize {
    old_size
}

#[no_mangle]
pub extern fn __rust_usable_size(size: usize, _align: usize) -> usize {
    return size;
}
