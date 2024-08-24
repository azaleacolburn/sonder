use sonder_engine::import_c;

fn main() {
    import_c!("import_src/test.c", "test_func");
}
