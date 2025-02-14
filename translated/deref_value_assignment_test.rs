fn main() -> () {
	let mut k: i32 = 3;
	let k_clone: i32 = k;
	let y: &mut i32  = &mut k;
	*y = k_clone + 6;
}