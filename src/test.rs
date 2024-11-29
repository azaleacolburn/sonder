use std::{collections::HashMap, fs, ops::Range, process::Command};

use crate::{
    analyzer::{self, VarData},
    annotater, checker, converter, parse_c,
};

/// Valid use of pointers as if they were Rust references
/// Translates one-to-one
#[test]
fn three_mut() {
    let rust_code = test(String::from(
        "int main() {
            int n = 0;
            int* g = &n;
            int* p = &n;
            int** m = &p;
            **m = 5;
        }",
    ));
    validate(String::from("three_mut"), rust_code);
}

/// Invalid rust code if directly translated
/// Should be caught by the checker and a safe solution should be applied
/// eg.
/// ```rust
/// fn main() -> () {
///     let t: Rc<RefCell<i32>> = Rc::new(RefCell::new(0));
///     let g: Rc<RefCell<i32>> = t.clone();
///     *t.borrow_mut() = 1;
///     *g.borrow_mut() = 2;
/// }
/// ```
#[test]
fn value_overlap() {
    let rust_code = test(String::from(
        "int main() {
            int t = 0;
            int* g = &t;
            t = 1;
            *g = 2;
        }",
    ));

    validate(String::from("value_overlap"), rust_code)
}

/// Invalid Rust code if directly translated
/// Should be caught by the checker and a safe solution should be applied
/// eg.
/// ```rust
/// fn main() -> () {
///     let n: i32 = 0;
///     let g: &i32 = 0;
/// }
/// fn test -> () {
///     let k: Rc<RefCell<i32>> = Rc::new(RefCell::new(3));
///     let y: Rc<RefCell<i32>> = k.clone();
///     *y.borrow_mut() = *k.borrow() + 6;
/// }
/// ```
#[test]
fn multi_function() {
    let rust_code = test(String::from(
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

    validate(String::from("multi_function"), rust_code);
}

fn test(code: String) -> String {
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
    let annotated_ast = annotater::annotate_ast(&ast, &var_info);
    annotated_ast.print(&mut 0);

    let converted_rust = converter::convert_annotated_ast(&annotated_ast);
    println!("{converted_rust}");
    converted_rust
}

fn validate(test_name: String, rust_code: String) {
    let file_name = format!("./translated/{test_name}_test.rs");
    fs::write(file_name.clone(), rust_code).expect("writing to succeed");
    match Command::new("rustc")
        .arg(file_name)
        .arg("--out-dir")
        .arg("./translated/exe")
        .spawn()
        .expect("Rust compilation failed")
        .wait()
    {
        Ok(o) => println!("Test passed: {o}"),
        Err(err) => panic!("Test failed, {err}"),
    };
}
