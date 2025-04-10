fn main() -> () {
    let mut i: i32 = 0;
    while i == 0 {
        let mut k: &mut i32 = &mut i;
        *k = 1;
    }
}

