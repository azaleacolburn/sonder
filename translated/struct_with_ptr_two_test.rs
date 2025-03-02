struct Point<'a> {
	ptr: &'a mut i32,
	ptr2: &'a mut i32,
	c: i32,
}
fn main() -> () {
	let mut t: i32 = 4;
	let mut g: i32 = 8;
	let l = Point { ptr: &mut t,ptr2: &mut g,c: 5,};
	*l.ptr = 5;
	*l.ptr2 = 9;
}