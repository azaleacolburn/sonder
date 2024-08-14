use sonder_engine::import_c;

fn main() {
    #[import_c(~/compsci/sonder/import_src/test.c, test_func)]
    let test = 5;
}
