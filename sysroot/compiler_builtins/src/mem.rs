//! C-ABI memory functions, byte-at-a-time. Correct over fast: at this stage
//! of the SDK a tight loop beats a clever one that needs core API we don't
//! have yet. Pointer arithmetic is done through usize casts on purpose —
//! scry-core does not yet provide the inherent pointer methods.

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let mut i = 0usize;
    while i < n {
        unsafe {
            *((dest as usize + i) as *mut u8) = *((src as usize + i) as *const u8);
        }
        i += 1;
    }
    dest
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memmove(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    if (dest as usize) < (src as usize) {
        // dest below src: forward copy is safe even when regions overlap.
        let mut i = 0usize;
        while i < n {
            unsafe {
                *((dest as usize + i) as *mut u8) = *((src as usize + i) as *const u8);
            }
            i += 1;
        }
    } else {
        // dest above (or equal to) src: copy backward.
        let mut i = n;
        while i > 0 {
            i -= 1;
            unsafe {
                *((dest as usize + i) as *mut u8) = *((src as usize + i) as *const u8);
            }
        }
    }
    dest
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memset(dest: *mut u8, c: i32, n: usize) -> *mut u8 {
    let byte = c as u8;
    let mut i = 0usize;
    while i < n {
        unsafe {
            *((dest as usize + i) as *mut u8) = byte;
        }
        i += 1;
    }
    dest
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    let mut i = 0usize;
    while i < n {
        let a = unsafe { *((s1 as usize + i) as *const u8) };
        let b = unsafe { *((s2 as usize + i) as *const u8) };
        if a != b {
            // Bytes compare as unsigned; the difference fits (and signs) i32.
            return a as i32 - b as i32;
        }
        i += 1;
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    unsafe { memcmp(s1, s2, n) }
}
