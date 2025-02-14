use std::{rc::Rc, cell::RefCell};
fn main() -> () {
	let n: Rc<RefCell<i32>> = Rc::new(RefCell::new(0));
	let g: Rc<RefCell<i32>> = n.clone();
	let b: Rc<RefCell<i32>> = n.clone();
	let k: i32 = *g.borrow();
	let y: i32 = 9;
	*b.borrow_mut() = y;
}