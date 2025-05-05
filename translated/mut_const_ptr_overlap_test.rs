fn main() -> () {
let mut n: i32 = 0;
	let g: &i32 = &n;
	let _t: i32 = *g;
	let m: &mut i32  = &mut n;
	*m = 4;
}