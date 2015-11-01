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
pub unsafe extern "C" fn __aeabi_unwind_cpp_pr0() -> ! {
    loop {}
}

#[cfg(target_os = "none")]
#[no_mangle]
pub unsafe extern "C" fn __aeabi_memclr4(dest: *mut u32, n: usize) {
    let mut n = n;
    let mut dest = dest;
    while n != 0 {
        *dest = 0;
        dest = dest.offset(1);
        n -= 4;
    }
}
