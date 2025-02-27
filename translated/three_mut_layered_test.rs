fn main() -> () {
	let n: i32 = 0;
	let mut g: &i32 = &n;
	let mut p: &mut i32  = &mut n;
	let mut m: &mut i32  = &mut p;
	**m = 5;
}