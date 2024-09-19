# Mutate Basic

```c
int t = 0;
int ptr_t = &t;
void set_four(int* ptr_t) -> {
    *ptr_t = 4;
}
```

```rust
let t: i16 = 0;
let ptr_t: &mut i16 = &mut t;
fn set_four(ptr_t: &mut i16) -> () {
    *ptr_t = 4;
}
```
