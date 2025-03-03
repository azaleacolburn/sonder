use std::{cell::RefCell, rc::Rc};
fn main() -> () {
	let n: Rc<RefCell<i32>> = Rc::new(RefCell::new(0));
	let g: Rc<RefCell<i32>> = n.clone();
	let b: Rc<RefCell<i32>> = n.clone();
	let _k: i32 = *g.borrow();
	let y: i32 = 9;
	*b.borrow_mut() = y;
}