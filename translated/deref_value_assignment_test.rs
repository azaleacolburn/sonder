fn main() -> () {
	let mut k: i32 = 3;
	let y: *mut i32 = &mut k as *mut i32;
	unsafe { *y = k + 6 };
}