fn main() -> () {
let mut n: i32 = 0;
	let g: &i32 = &n;
	let mut p: &mut i32  = &mut n;
	let m: &mut &mut i32   = &mut p;
	**m = 5;
}