use sonder_engine::{import_c_function, import_c_struct};

import_c_struct!("import_src/test.c", "test_struct");
import_c_function!("import_src/test.c", "test_func");

fn main() {
    let test_s = test_struct { x: 10, y: 10 };
    test(test_s)
}
fn test(t: test_struct) {
    println!("{}", t.x);
}
