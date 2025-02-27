fn main() -> () {
	let k: i32 = 3;
	let mut y: &mut i32  = &mut k;
	*y = k + 6;
}