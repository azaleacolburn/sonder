use std::{fs, process::Command};

use crate::{convert_to_rust_code, parse_c};

/// Valid use of pointers as if they were Rust references
/// Translates one-to-one
#[test]
fn three_mut_layered() {
    validate(
        "int main() {
            int n = 0;
            int* g = &n;
            int* p = &n;
            int** m = &p;
            **m = 5;
        }",
        "three_mut_layered",
    );
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
    validate(
        "int main() {
            int t = 0;
            int* g = &t;
            t = 1;
            *g = 2;
        }",
        "value_overlap",
    );
}

/// Invalid Rust code if directly translated
/// Should be caught by the checker and a safe solution should be applied
///
/// See dilema in README.md
///
/// ```
#[test]
fn deref_value_assignment() {
    validate(
        "void main() {
            int k = 3;
            int* y = &k;
            *y = k + 6;
        }",
        "deref_value_assignment",
    );
}

#[test]
fn multi_ref() {
    validate(
        "int main() {
            int n = 0;
            int* g = &n;
            int* b = &n;
            int k = *g;
            int y = 9;
            *b = y;
        }",
        "multi_ref",
    );
}

/// This is actually an interesting case
/// Based on our current assumption, this is illegal, because we're assigning a reference in
/// something that isn't a ptr declaration
/// So the issue becomes, since during conversion, addresses just return their id and let the
/// higher node handle the rest, and deref_assignment_node isn't handling the address level, the
/// address never gets taken in the generated rust code
///
/// In addition to that, there's some cascading mutability issue that the checker isn't picking up
/// on
#[test]
fn crazy_multi_ref() {
    validate(
        "int main() {
            int n = 0;
            int* g = &n;
            int* k = &n;
            int** h = &g;
            int p = 3;
            *h = &p;
        }",
        "crazy_multi_ref",
    );
}

#[test]
fn struct_basic() {
    validate(
        "struct test {
            int m;
            int j;
        };",
        "struct_basic",
    );
}

#[test]
fn struct_init_test() {
    validate(
        "struct Test {
            int m;
            int j;
        };

        int main() {
            struct Test my_test = { 0, 2 };
        }",
        "struct_init_test",
    );
}

fn validate(c_code: &str, test_name: &str) {
    let ast = parse_c(c_code.to_string());
    let rust_code = convert_to_rust_code(ast);

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
