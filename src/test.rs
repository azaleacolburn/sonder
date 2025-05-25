use std::{fs, process::Command};

use crate::{convert_to_rust_code, parse_c};

#[test]
fn basic_assignment() {
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

/// ```rust
/// fn main() -> () {
///     let t = 0;
///     t = 1;
///     let g = &mut t;
///     *g = 2;
/// }
/// ```
#[test]
fn value_mut_ptr_overlap() {
    validate(
        "int main() {
            int t = 0;
            int* g = &t;
            t = 1;
            *g = 2;
        }",
        "value_mut_ptr_overlap",
    );
}

/// This test should be solvable by rearragement
/// Because within the reference lifetime the first usage of g is after the last usage of t
///
/// Because it's on the r_side, it requires rearragnment
///
/// ```rust
/// fn main() -> () {
///     let t: i32 = 0;
///     t = 1;
///     let g: &i32 = &t;
///     let h = *g;
/// }
#[test]
fn value_const_ptr_overlap() {
    validate(
        "int main() {
            int t = 0;
            int* g = &t;
            t = 1;
            int h = *g;
        }",
        "value_const_ptr_overlap",
    );
}

// #[test]
// fn basic_shared_ptr() {
//     validate(
//         "int main() {
//             int n = 0;
//             int* k = &n;
//             *k = 6;
//             int* h = &n;
//             *k = 3;
//             int y = *h;
//         }",
//         "basic_shared_ptr",
//     );
// }

/// Invalid Rust code if directly translated
/// Should be caught by the checker and a safe solution should be applied
///
/// See dilema in README.md
///
/// WARNING THIS TEST PROBABLY NEEDS RAW PTRS
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

/// ```rust
/// fn main() {
///     let n = 0;
///     let g = &n;
///     let t = *g;
///     let m = &mut n;
///     *m = 4;
/// }
#[test]
fn const_mut_ptr_overlap() {
    validate(
        "int main() {
            int n = 0;
            int* g = &n;
            int* m = &n;
            int t = *g;
            *m = 4;
        }",
        "const_mut_ptr_overlap",
    );
}

// For this case, we must move the const ptr down
#[test]
fn mut_const_ptr_overlap() {
    validate(
        "int main() {
            int n = 0;
            int* m = &n;
            int* g = &n;
            *m = 4;
            int t = *g;
        }",
        "mut_const_ptr_overlap",
    );
}

#[test]
fn mut_const_ptr_multi_overlap() {
    validate(
        "int main() {
            int n = 0;
            int* m = &n;
            int* g = &n;
            int j = *g;
            *m = 4;
            int t = *g;
        }",
        "mut_const_ptr_multi_overlap",
    );
}

#[test]
fn value_const_ptr_multi_overlap() {
    validate(
        "int main() {
            int t = 0;
            int* g = &t;
            t = 3;
            t = 1;
            int h = *g;
        }",
        "value_const_ptr_multi_overlap",
    );
}

/// ```rust
/// fn main() {
///     let n: i32  = 0;
///     let g: &i32 = &n;
///     let b: &mut i32 = &mut n;
///     let k: i32 = *g;
///     let y: i32 = 9;
///     *b = y;
// }
#[test]
fn simple_multi_ref() {
    validate(
        "int main() {
            int n = 0;
            int* g = &n;
            int* b = &n;
            int k = *g;
            int y = 9;
            *b = y;
        }",
        "simple_multi_ref",
    );
}

#[test]
fn unused_init_value() {
    validate(
        "int main() {
            int n = 0;
            n = 7;
        }",
        "unused_init_value",
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
            *l.ptr = 5;
        }",
        "struct_with_ptr",
    );
}

#[test]
fn struct_with_ptr_two() {
    validate(
        "struct Point {
            int* ptr;
            int* ptr2;
            int c;
        };

        int main() {
            int t = 4;
            int g = 8;
            struct Point l = { &t, &g, 5 };
            *l.ptr = 5;
            *l.ptr2 = 9;
        }",
        "struct_with_ptr_two",
    );
}

#[test]
fn struct_field_ptr_assignment() {
    validate(
        "struct Point {
            int* ptr;
        };

        int main() {
            int t = 4;
            struct Point l = { &t };
            *l.ptr = 5;
        }",
        "struct_field_ptr_assignment",
    );
}

#[test]
fn struct_with_ptr_multi() {
    validate(
        "struct Point {
            int* ptr;
            int** ptr2;
            int c;
        };

        int main() {
            int t = 4;
            int g = 8;
            int* h = &g;
            struct Point l = { &t, &h, 5 };
            *l.ptr = 5;
            **l.ptr2 = 9;
        }",
        "struct_with_ptr_multi",
    );
}

// #[test]
// fn struct_shared_ptr() {
//     validate(
//         "struct Point {
//             int* ptr;
//         };
//
//         int main() {
//             int t = 9;
//             struct Point l = { &t };
//             struct Point g = { &t };
//             *l.t = 8;
//             *g.t = 3;
//         }",
//         "struct_with_shared_ptr",
//     );
// }

#[test]
fn basic_loop() {
    validate(
        "int main() {
            int i = 0;
            while (i == 0) {
                i++;
            }
        }",
        "basic_loop",
    );
}

#[test]
fn ptr_loop() {
    validate(
        "int main() {
            int i = 0;
            while (i == 0) {
                int* k = &i;
                *k = 1;
            }
        }",
        "ptr_loop",
    );
}

#[test]
fn function_call() {
    validate(
        "int main() {
            test(1, 2);
        }
        void test(int a, int b) {
            int k = a + b;
        }",
        "function_call",
    );
}

#[test]
fn assignment_function_call() {
    validate(
        "int main() {
            int t = add(1, 2);
        }
        int add(int a, int b) {
            int k = a + b;
            return(k);
        }",
        "assignment_function_call",
    );
}

#[test]
fn recursive_function() {
    validate(
        "int main() {
            int i = 0;
            int g = count_four(i);
        }
        int count_four(int i) {
            if (i == 3) {
                return 4;
            }
            return(count_four(i + 1));
        }",
        "recursive_function",
    )
}

fn validate(c_code: &str, test_name: &str) {
    let ast = parse_c(c_code.to_string());
    let rust_code = convert_to_rust_code(ast);

    fs::create_dir_all("./translated/bin").expect("dir failed");
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
