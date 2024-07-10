#[repr(C)]
pub struct div_t {
    quot: i16,
    rem: i16,
}

#[repr(C)]
pub struct ldiv_t {
    quot: i32,
    rem: i32,
}

#[no_mangle]
pub static NULL: i16 = 0;
#[no_mangle]
pub static EXIT_FAILURE: i16 = 0;
#[no_mangle]
pub static EXIT_SUCCESS: i16 = 1;
#[no_mangle]
pub static RAND_MAX: usize = 2147483647;

#[no_mangle]
pub unsafe extern "C" fn atof(mut str: *const u8) -> f64 {
    let anchor = str;
    let mut f: f64 = 0.0;

    let decimal_ptr: *const u8;
    while !(*str != b'.') {
        str = str.add(1);
    }
    decimal_ptr = str;

    let mut i: usize = anchor.offset_from(decimal_ptr) as usize;

    while !decimal_ptr.sub(i).is_null() {
        f += (*decimal_ptr.sub(i) - 48_u8) as f64 * 10_i64.pow(i as u32) as f64;
        i -= 1;
    }

    i = 0;

    while !decimal_ptr.add(i).is_null() {
        f += (*decimal_ptr.add(i) - 48_u8) as f64 / 10_i64.pow(i as u32) as f64;
        i += 1;
    }

    f
}

#[no_mangle]
pub unsafe extern "C" fn atoi(mut str: *const u8) -> i16 {
    let anchor = str;
    let mut int: i16 = 0;

    while !str.is_null() {
        str = str.add(1);
    }

    let mut i = anchor.offset_from(str) as usize;

    while !str.sub(i).is_null() {
        int += (*str.sub(i) - 48_u8) as i16 * 10_u8.pow(i as u32) as i16;
        i -= 1;
    }

    int
}

#[no_mangle]
pub unsafe extern "C" fn atol(mut str: *const u8) -> i32 {
    let anchor = str;
    let mut int: i32 = 0;

    while !str.is_null() {
        str = str.add(1);
    }

    let mut i = anchor.offset_from(str) as usize;

    while !str.sub(i).is_null() {
        int += (*str.sub(i) - 48_u8) as i32 * 10_u8.pow(i as u32) as i32;
        i -= 1;
    }

    int
}

#[no_mangle]
pub unsafe extern "C" fn strtod(mut str: *const u8, endptr: *const *mut u8) -> f64 {
    let anchor = str;
    let mut f: f64 = 0.0;

    let decimal_ptr: *const u8;
    while !(*str != b'.') {
        str = str.add(1);
    }
    decimal_ptr = str;

    let mut i: usize = anchor.offset_from(decimal_ptr) as usize;

    while !decimal_ptr.sub(i).is_null() {
        f += (*decimal_ptr.sub(i) - 48_u8) as f64 * 10_i64.pow(i as u32) as f64;
        i -= 1;
    }

    i = 0;

    while !decimal_ptr.add(i).is_null() && *decimal_ptr.add(i) > 47 && *decimal_ptr.add(i) < 58 {
        f += (*decimal_ptr.add(i) - 48_u8) as f64 / 10_i64.pow(i as u32) as f64;
        i += 1;
    }

    if endptr != core::ptr::null() {
        **endptr = *decimal_ptr.add(i);
    }

    f
}

#[no_mangle]
/// arg base must be between 2 and 32, or 0
pub unsafe extern "C" fn strtol(mut str: *const u8, endptr: *const *mut u8, mut base: u8) -> u32 {
    if (base < 2 || base > 32) && base != 0 {
        panic!("Base of function strtol must be between 2 and 32 (inclusive), or 0");
    } else if base == 0 {
        base = if *str.add(1) == 0 {
            if *str.add(2) == b'x' || *str.add(2) == b'X' {
                16
            } else {
                8
            }
        } else {
            0
        }
    }
    let anchor = str;
    let mut int: u32 = 0;

    while !str.is_null() && *str > 47 && *str < 58 {
        str = str.add(1);
    }

    let mut i = anchor.offset_from(str) as usize;

    while !str.sub(i).is_null() {
        int += (*str.sub(i) - 48_u8) as u32 * base.pow(i as u32) as u32;
        i -= 1;
    }

    **endptr = *str.add(1);

    int
}

struct MetaBlock {
    size: usize,
    next: Option<*mut MetaBlock>,
    free: bool,
    magic: u8,
}

const META_SIZE: usize = core::mem::size_of::<MetaBlock>();

static mut HEAP_HEAD: Option<*mut MetaBlock> = None;

pub unsafe extern "C" fn malloc(size: usize) -> Result<*mut MetaBlock, ()> {
    let mut block: Option<*mut MetaBlock>;

    if size <= 0 {
        return Err(());
    }

    if HEAP_HEAD.is_none() {
        block = Some(request_space(None, size)?);
        HEAP_HEAD = block;
    } else {
        let mut last: *mut MetaBlock = HEAP_HEAD.unwrap();
        block = find_free_block(&mut last as *mut *mut MetaBlock, size).ok();
        if block.is_some() {
            (*block.unwrap()).free = false;
            (*block.unwrap()).magic = 0x77777777;
        } else {
            block = Some(request_space(Some(last), size)?);
        }
    }

    Ok(block.unwrap().add(1))
}

unsafe extern "C" fn find_free_block(
    last: *mut *mut MetaBlock,
    size: usize,
) -> Result<*mut MetaBlock, ()> {
    let mut curr: Option<*mut MetaBlock> = HEAP_HEAD;
    while curr.is_some() && !((*curr.unwrap()).free && (*curr.unwrap()).size >= size) {
        *last = curr.unwrap();
        curr = (*curr.unwrap()).next;
    }

    Ok(curr.unwrap())
}

unsafe extern "C" fn request_space(
    last: Option<*const MetaBlock>,
    size: usize,
) -> Result<*mut MetaBlock, ()> {
    let block: *mut MetaBlock = sbrk(0);
    let request: *const () = sbrk(size + META_SIZE);
    if request == core::ptr::null() {
        return Err(());
    }

    if last.is_none() {
        (*last.unwrap()).next = Some(block);
    }

    (*block).size = size;
    (*block).next = None;
    (*block).free = false;
    (*block).magic = 0x12345678;

    Ok(block)
}

unsafe extern "C" fn sbrk(_size: usize) -> *const () {
    todo!()
}

pub unsafe extern "C" fn free(ptr: *mut ()) -> Result<(), ()> {
    if ptr.is_null() {
        return Err(());
    }

    let block_ptr = get_block_ptr(ptr);
    assert_eq!((*block_ptr).free, false);
    assert_eq!((*block_ptr).magic, 0x77777777);
    (*block_ptr).free = true;
    (*block_ptr).magic = 0x55555555;

    return Ok(());
}

unsafe extern "C" fn get_block_ptr(ptr: *mut ()) -> *mut MetaBlock {
    return (ptr as *mut MetaBlock).sub(1);
}
