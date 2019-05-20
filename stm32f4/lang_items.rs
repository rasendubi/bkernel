//! `lang_items` and functions needed to start Rust on bare metal.

// `loop {}` can't be replaced with `panic!()`
#![allow(clippy::empty_loop)]

#[cfg(target_os = "none")]
#[no_mangle]
pub unsafe extern "C" fn __aeabi_unwind_cpp_pr0() -> ! {
    loop {}
}

#[cfg(target_os = "none")]
#[no_mangle]
pub unsafe extern "C" fn __aeabi_unwind_cpp_pr1() -> ! {
    loop {}
}
