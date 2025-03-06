use std::{cell::RefCell, rc::Rc};
fn main() -> () {
    let n: Rc<RefCell<i32>> = Rc::new(RefCell::new(0));
    let g: Rc<RefCell<i32>> = n.clone();
    let m: Rc<RefCell<i32>> = n.clone();
    *m.borrow_mut() = 4;
    let _t: i32 = *g.borrow();
}

