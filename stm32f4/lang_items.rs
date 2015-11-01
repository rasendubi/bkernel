//! lang_items and functions needed to start Rust on bare metal.

#[cfg(target_os = "none")]
#[lang = "eh_personality"]
extern fn eh_personality() {}

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

#[cfg(target_os = "none")]
#[no_mangle]
pub unsafe extern "C" fn memcmp(mut str1: *const u8, mut str2: *const u8, mut size: usize) -> i32 {
    while size != 0 {
        if *str1 < *str2 {
            return -1;
        } else if *str1 > *str2 {
            return 1;
        }

        str1 = str1.offset(1);
        str2 = str2.offset(1);
        size -= 1;
    }

    return 0;
}
