fn main() -> () {
let mut n: i32 = 0;
	let k: &i32 = &n;
	let mut g: &mut i32  = &mut n;
	let h: &mut &mut i32   = &mut g;
	let p: i32 = 3;
	**h = p;
}