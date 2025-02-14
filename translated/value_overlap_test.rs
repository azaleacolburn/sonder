use std::{rc::Rc, cell::RefCell};
fn main() -> () {
	let t: Rc<RefCell<i32>> = Rc::new(RefCell::new(0));
	let g: Rc<RefCell<i32>> = t.clone();
	*t.borrow_mut() = 1;
	*g.borrow_mut() = 2;
}