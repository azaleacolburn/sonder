struct Point<'a> {
	ptr: &'a mut i32,
}
fn main() -> () {
	let mut t: i32 = 4;
	let l = Point { ptr: &mut t,};
	*l.ptr = 5;
}