use std::{fs, process::Command};

use crate::{convert_to_rust_code, parse_c};

#[test]
fn test_basic_assignment() {
    validate(
        "int main() {
            int n = 0;
            n = 2;
        }",
        "basic_assignment",
    )
}

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
/// We can only ever use `*t.borrow()` or `*t.borrow_mut()` when using t
/// Because it has to act exactly like the value t would in C.
/// eg.
/// ```rust
/// fn main() -> () {
///     let t: Rc<RefCell<i32>> = Rc::new(RefCell::new(0));
///     let g: Rc<RefCell<i32>> = t.clone();
///     *t.borrow_mut() = 1;
///     *g.borrow_mut() = 2;
///     // When function calls come about
///     f(*t.borrow())
///     f(g)
///     f(g, t.borrow_mut()) // this is fine
///     f(*g.borrow(), *t.borrow_mut) // this is not ok
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
/// WARNING THIS TEST PROBABLY NEEDS RAW PTRS
// /// ```
// #[test]
// fn deref_value_assignment() {
//     validate(
//         "void main() {
//             int k = 3;
//             int* y = &k;
//             *y = k + 6;
//         }",
//         "deref_value_assignment",
//     );
// }

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
/// This sets off the cloning system
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
fn struct_init() {
    validate(
        "struct Test {
            int m;
            int j;
        };

        int main() {
            struct Test my_test = { 0, 2 };
        }",
        "struct_init",
    );
}

#[test]
fn struct_field_assignment() {
    validate(
        "struct Test {
            int m;
            int j;
        };

        int main() {
            struct Test my_test = { 0, 2 };
            my_test.m = 1;
        }",
        "struct_field_assignment",
    );
}

#[test]
fn struct_with_ptr() {
    validate(
        "struct Point {
            int* ptr;
            int c;
        };

        int main() {
            int t = 4;
            struct Point l = { &t, 5 };
            *l.t = 5;
        }",
        "struct_with_ptr",
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
        .arg("./translated/bin")
        .spawn()
        .expect("Rust compilation failed to start")
        .wait()
    {
        Ok(o) if o.success() => println!("Test passed!"),
        Ok(_) => panic!("Compilation Failed"),
        Err(err) => panic!("RustC Panicked, {err}"),
    };
}
