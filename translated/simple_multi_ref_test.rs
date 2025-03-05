fn main() -> () {
	let mut n: i32 = 0;
	let mut g: &i32 = &n;
	let _k: i32 = *g;
	let mut b: &mut i32  = &mut n;
	let y: i32 = 9;
	*b = y;
}