use std::{fs, process::Command};

use crate::{convert_to_rust_code, parse_c};

/// Valid use of pointers as if they were Rust references
/// Translates one-to-one
#[test]
fn three_mut() {
    let rust_code = to_rust(String::from(
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
    let rust_code = to_rust(String::from(
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
fn deref_value_assignment() {
    let rust_code = to_rust(String::from(
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

    validate(String::from("deref_value_assignment"), rust_code);
}

#[test]
fn multi_ref() {
    let rust_code = to_rust(String::from(
        "int main() {
            int n = 0;
            int* g = &n;
            int* b = &n;
            int k = *g;
            int y = 9;
            *b = y;
        }",
    ));

    validate(String::from("multi_ref"), rust_code);
}

fn to_rust(code: String) -> String {
    let ast = parse_c(code);
    convert_to_rust_code(ast)
}

fn validate(test_name: String, rust_code: String) {
    fs::create_dir_all("./translated/exe").expect("dir failed");
    let file_name = format!("./translated/{test_name}_test.rs");
    fs::write(file_name.clone(), rust_code).expect("writing code to file failed");
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
