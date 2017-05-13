//! `lang_items` and functions needed to start Rust on bare metal.

#[cfg(target_os = "none")]
#[lang = "eh_personality"]
extern fn eh_personality() {}

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
