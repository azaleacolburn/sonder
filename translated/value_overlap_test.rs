fn main() -> () {
    let mut t: i32 = 0;
    t = 1;
    let mut g: &mut i32 = &mut t;
    *g = 2;
}

