use std::{cell::RefCell, rc::Rc};
fn main() -> () {
	let t: Rc<RefCell<i32>> = Rc::new(RefCell::new(0));
	*t.borrow_mut() = 1;
	let mut g: Rc<RefCell<i32>> = t.clone();
	*g.borrow_mut() = 2;
}