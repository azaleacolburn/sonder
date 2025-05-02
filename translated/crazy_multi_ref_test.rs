fn main() -> () {
let mut n: i32 = 0;
	let mut k: &i32 = &n;
	let mut g: &mut i32  = &mut n;
	let mut h: &mut &mut i32   = &mut g;
	let p: i32 = 3;
	**h = p;
}