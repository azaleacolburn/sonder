use std::{cell::RefCell, rc::Rc};
fn main() -> () {
	let n: Rc<RefCell<i32>> = Rc::new(RefCell::new(0));
	let k: Rc<RefCell<i32>> = n.clone();
	let mut g: Rc<RefCell<i32>> = n.clone();
	let mut h: &mut Rc<RefCell<i32>>  = &mut g;
	let p: i32 = 3;
	*h.borrow_mut() = p;
}