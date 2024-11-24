use std::collections::HashMap;

use crate::{
    analyzer::{self, VarData},
    converter, parse_c,
};

// #[test]
// fn sanity() {
//     test(String::from(
//         "int main() {
//             int n = 0;
//             int* g = &n;
//         }",
//     ));
// }

#[test]
fn three_mut() {
    test(String::from(
        "int main() {
            int n = 0;
            int* g = &n;
            int* p = &n;
            int** m = &p;
            **m = 5;
        }",
    ));
}

fn test(code: String) {
    let ast = parse_c(code);
    ast.print(&mut 0);
    let map: HashMap<String, VarData> = HashMap::new();

    let var_info = analyzer::determine_var_mutability(&ast, &map);
    println!("{:?}", var_info);
    let annotated_ast = analyzer::annotate_ast(&ast, &var_info);
    annotated_ast.print(&mut 0);

    let converted_rust = converter::convert_annotated_ast(&annotated_ast);
    println!("{converted_rust}");
}
// fn test() -> i16 {
//     let mut n = 0;
//     let mut p = &mut n;
//     let m = &mut p;
//     **m = 6;
//     return **m;
// }
