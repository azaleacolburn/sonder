use crate::{analyzer, parse_c};

#[test]
fn analyze() {
    let code = String::from("int main() {int n = 0; int* p = &n; int** m = &p; *p = 3; *m = 5;}");
    let ast = parse_c(code);
    ast.print(&mut 0);

    let mut ptrs = vec![];
    let pointers = analyzer::get_all_pointers_and_derefs(&ast, &mut ptrs);
    println!("{:?}", pointers);
}
