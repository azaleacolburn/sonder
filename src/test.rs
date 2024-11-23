use std::collections::HashMap;

use crate::{
    analyzer::{self, VarData},
    parse_c,
};

#[test]
fn analyze() {
    let code = String::from("int main() { int n = 0; int* p = &n; int** m = &p; *p = 3; *m = 5; }");
    let _rust_code =
        String::from("fn main() -> i16 { let mut n = 0; let mut p = &mut n; let m = &mut p; **m = 6; return **m;}");
    let ast = parse_c(code);
    ast.print(&mut 0);
    let map: HashMap<String, VarData> = HashMap::new();

    let var_info = analyzer::determine_var_mutability(&ast, &map);
    println!("{:?}", var_info);
    let annotated_ast = analyzer::annotate_ast(&ast, &var_info);
    annotated_ast.print(&mut 0);
}
// fn test() -> i16 {
//     let mut n = 0;
//     let mut p = &mut n;
//     let m = &mut p;
//     **m = 6;
//     return **m;
// }
