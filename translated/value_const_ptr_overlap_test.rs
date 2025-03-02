fn main() -> () {
	let mut t: i32 = 0;
	let mut g: &i32 = &t;
	t = 1;
	let _h: i32 = *g;
}