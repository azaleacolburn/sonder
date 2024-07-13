pub unsafe fn strlen(src: *const ()) -> usize {
    let mut i: usize = 0;

    while src.wrapping_add(i).is_null() {
        i += 1;
    }

    i
}

pub unsafe fn memchr(mut str: *const u8, c: u8, _n: usize) -> Option<*mut u8> {
    while !str.is_null() {
        if *str == c {
            return Some(str.clone() as *mut u8);
        }
        str = str.add(1);
    }
    None
}

pub unsafe fn memcmp(str1: *const u8, str2: *const u8, n: usize) -> bool {
    // Probably a better way to do this with bitshifting and xor
    for i in 0..n {
        if *str1.add(i) != *str2.add(i) {
            return false;
        }
    }
    return true;
}

pub unsafe fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    for i in 0..n {
        *dest.add(i) = *src.add(i);
    }

    return dest;
}

pub unsafe fn memmove(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let furthest_dest: *const u8 = dest.add(n);
    let furthest_src: *const u8 = src.add(n);
    if (furthest_dest >= src && furthest_dest <= furthest_src)
        || (furthest_src >= dest && furthest_src <= furthest_dest)
    {
        return core::ptr::null::<u8>() as *mut u8;
    }

    for i in 0..n {
        *dest.add(i) = *src.add(i);
    }

    return dest;
}

#[allow(unused_assignments)]
pub unsafe fn strcat(mut dest: *mut u8, src: *const u8) {
    while !dest.is_null() {
        dest = dest.add(1)
    }
    let mut i = 0;
    while !src.is_null() {
        *dest.add(i) = *src.add(i);
        i += 1;
    }
}

#[allow(unused_assignments)]
pub unsafe fn strncat(mut dest: *mut u8, src: *const u8, n: usize) {
    while !dest.is_null() {
        dest = dest.add(1)
    }
    for i in 0..n {
        if src.is_null() {
            break;
        }
        *dest.add(i) = *src.add(i);
    }
}

pub unsafe fn strchr(mut str: *const u8, c: u8) -> *mut u8 {
    while !str.is_null() {
        if *str == c {
            return str as *mut u8;
        }
        str = str.add(1);
    }

    return core::ptr::null::<u8>() as *mut u8;
}

pub unsafe fn strcmp(mut str1: *const u8, mut str2: *const u8) -> bool {
    while !str1.is_null() && !str2.is_null() {
        if *str1 != *str2 {
            return false;
        }
        str1 = str1.add(1);
        str2 = str2.add(1);
    }
    return true;
}
