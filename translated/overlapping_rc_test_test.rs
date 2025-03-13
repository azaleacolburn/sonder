use std::{cell::RefCell, rc::Rc};
fn main() -> () {
    let n: Rc<RefCell<i32>> = Rc::new(RefCell::new(0));
    let k: Rc<RefCell<i32>> = n.clone();
    *k.borrow_mut() = 6;
    let h: Rc<RefCell<i32>> = n.clone();
    *k.borrow_mut() = 3;
    let _y: i32 = *h.borrow();
}

