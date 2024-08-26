use sonder_engine::{import_c_function, import_c_struct};

fn main() {
    import_c_struct!("import_src/test.c", "test_struct");
    import_c_function!("import_src/test.c", "test_func");
}
