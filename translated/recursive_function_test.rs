fn main() -> () {
let i: i32 = 0;
	let _g: i32 = count_four(i);
}
fn count_four(i: i32) -> i32 {
if i == 3 {
return(4);
}
	return(count_four(i + 1));
}