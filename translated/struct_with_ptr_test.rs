struct Point<'a> {
	ptr: &'a mut i32,
	c: i32,
}
fn main() -> () {
let mut t: i32 = 4;
	let l = Point { ptr: &mut t,c: 5,};
	*l.ptr = 5;
}