use std::{collections::HashMap, fs, ops::Range, process::Command};

use crate::{
    analyzer::{self, VarData},
    checker, converter, parse_c,
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

/// Valid use of pointers as if they were Rust references
/// Translates one-to-one
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

/// Invalid rust code if directly translated
/// Should be caught by the checker and a safe solution should be applied
/// eg.
/// ```rust
/// fn main() {
///     let t: Rc<RefCell<i32>> = Rc::new(RefCell::new(0));
///     let g = t.clone();
///     *t.borrow_mut() = 1;
///     *g.borrow_mut() = 2;
/// }
/// ```
#[test]
fn value_overlap() {
    test(String::from(
        "int main() {
            int t = 0;
            int* g = &t;
            t = 1;
            *g = 2;
        }",
    ))
}

#[test]
fn multi_function() {
    test(String::from(
        "int main() {
            int n = 0;
            int* g = &n;
        }
        void test() {
            int k = 3;
            int* y = &k;
            *y = k + 6;
        }",
    ));
}

fn test(code: String) {
    let ast = parse_c(code);
    ast.print(&mut 0);
    let map: HashMap<String, VarData> = HashMap::new();

    let mut var_info = analyzer::determine_var_mutability(&ast, &map);
    println!(
        "{:?}",
        var_info
            .iter()
            .map(|(id, data)| (id.clone(), data.non_borrowed_lines.clone()))
            .collect::<Vec<(String, Vec<Range<usize>>)>>()
    );

    let errors = checker::borrow_check(&var_info);
    checker::adjust_ptr_type(errors, &mut var_info);
    let annotated_ast = analyzer::annotate_ast(&ast, &var_info);
    annotated_ast.print(&mut 0);

    let converted_rust = converter::convert_annotated_ast(&annotated_ast);
    println!("{converted_rust}");
    validate(converted_rust);
}

fn validate(rust_code: String) {
    fs::write("./test.rs", rust_code).expect("writing to succeed");
    match Command::new("rustc")
        .arg("./test.rs")
        .spawn()
        .expect("Rust compilation failed")
        .wait()
    {
        Ok(o) => println!("Test passed: {o}"),
        Err(err) => panic!("Test failed, {err}"),
    };
}
// fn test() -> i16 {
//     let mut n = 0;
//     let mut p = &mut n;
//     let m = &mut p;
//     **m = 6;
//     return **m;
// }
