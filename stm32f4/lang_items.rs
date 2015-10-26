//! lang_items and functions needed to start Rust on bare metal.

#[cfg(target_os = "none")]
#[lang = "eh_personality"]
extern fn eh_personality() {}

#[cfg(target_os = "none")]
#[lang = "panic_fmt"]
fn panic_fmt() -> ! {
    loop {}
}

#[cfg(target_os = "none")]
#[lang = "stack_exhausted"]
fn stack_exhausted() -> ! {
    loop {}
}

#[cfg(target_os = "none")]
#[no_mangle]
pub unsafe fn __aeabi_unwind_cpp_pr0() -> ! {
    loop {}
}
