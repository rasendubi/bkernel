//! lang_items and functions needed to start Rust on bare metal.

// This file defines the builtin functions, so it would be a shame for
// LLVM to optimize these function calls to themselves!
#![no_builtins]

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

#[cfg(target_os = "none")]
#[no_mangle]
pub unsafe extern "C" fn __aeabi_memcpy(mut dst: *mut u8, mut src: *const u8, mut size: usize) {
    while size != 0 {
        *dst = *src;

        dst = dst.offset(1);
        src = src.offset(1);
        size -= 1;
    }
}

#[cfg(target_os = "none")]
#[no_mangle]
pub unsafe extern "C" fn __aeabi_memcpy4(mut dst: *mut u32, mut src: *const u32, mut size: usize) {
    while size != 0 {
        *dst = *src;

        dst = dst.offset(1);
        src = src.offset(1);
        size -= 4;
    }
}

#[cfg(target_os = "none")]
#[no_mangle]
pub unsafe extern "C" fn __aeabi_memmove4(mut dst: *mut u32, mut src: *mut u32, mut size: usize) {
    if dst == src {
        return;
    }

    let offset;
    if dst < src {
        offset = 1;
    } else {
        offset = -1;

        src = src.offset((size / 4 - 1) as isize);
        dst = src.offset((size / 4 - 1) as isize);
    }

    while size != 0 {
        *dst = *src;

        dst = dst.offset(offset);
        src = src.offset(offset);
        size -= 4;
    }
}
