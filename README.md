# sonder
Sonder is a static analyzer and transpiler, for converting well-written C code to Rust.

## Scratch repo

This project is incredible WIP and not at all intended for production-grade work. It exists purely as a proof of concept.
## What does well-written mean?

For the purposes of sonder, well-written means that for any given pointer, the C code in question treats pointers only in the following ways:

- Like a Rust reference in accordance with borrow-checking rules
- Like a Rust refrence after trivial line rearrangement 
- As a cloned `Rc<RefCell<T>>`

- It also means that any mutable pointer to a value and the value itself that are used on the same line can be made to fit borrow checking rules by substituting the value used in it's own assignment for a clone taken before the mutable reference. This essentially means that `t` can only be modified once during `g`'s lifetime, and it must be the value-mut-same-line-overlap-case.  This allows the following edgecase to be resolve with cloning:
```c
int t = 0;
int* g = &t;
*g = t + 1;
```
by transpiling to:
```rust
let mut t: i32  = 0;
let t_clone = t; // This would be .clone() for non-copy types
let g: &mut i32 = &mut t;
*g = t_clone + 1;
```

If the code isn't "well-written", sonder will fall back on unsafe raw pointers in the generated code.

## Examples of "not-well-written" C code

1. The adding addresses

```c
&foo + &bar // illegal
```

2. Dereferencing pointer arithmatic

```c
*(foo + bar) // illegal
```

I'm aware that this includes legal array indexing if foo or bar aren't pointers, however, the sonder ast treats that as a totally seperate thing. So `arr[foo]` is legal while `*(arr + foo)` or `foo[arr]` aren't, even though they're equivalent.

3. Dereferencing non-ptrs

```c
*not_ptr // illegal
```

Any of these will immediantly result in raw pointers being used, although at the moment, they panic

### How to handle usage overlaps on the same line (see above as well)

```c
int main() {
    int t = 0;
    int* k = &t;
    *k = t + 1;
}
```

In this case, Rc<RefCell> won't work, since the borrow and mutable borrow are on the same line in the invalid translated code. Meaning we would get something like this:

```rust
fn main() -> () {
    let mut t = Rc::new(RefCell::new(0));
    let k = t.clone();
    *k.borrow_mut() = t.borrow() + 1; // On the same line, panics
}
```

If the &mut and (sub_value | &mut) are used on the same line (that isn't the borrowing line)
Cloning or raw ptrs are required.

We then need to decide if changing the semantics of the program any amount is valid, such as placing a new item on the stack for the sake of memory safety

This sounds really difficult, so

eg.

```rust
fn main() -> () {
   let mut t = 0;
   let t_tmp = t.clone();
   let k = &mut t;
   *k = t_tmp + 1;
}
```

or

```rust
fn main() -> () {
    let mut t = 0;
    let k = &mut t as *mut i32;
    unsafe {
        *k = t + 1;
    }
}
```

## Todo

- [x] Rethink reference tracking
- [x] Struct Support
- [x] Struct Analysis (wip)
- [x] Struct Checking, Annotation, Conversion (wip)
- [ ] Cloning solutions (wip)
- [ ] More test cases for the current prototype
- [ ] Figure out how to represent scope
- [ ] System for managing scope
- [ ] Scope-based borrowing checking

## How does all this work?

Sonder is broken up into a few differenc components for performing different tasks; they are run sequentially.
Note that all pointers are assumed to be either immutable or mutable references unless the checker deems otherwise

### Analyzer

The Analyzer determines the necessary variable semantics for performing borrow-checking on C code.
The primary data-structure involved is a map related variable names to information that would be aparent in Rust code, but must be inferred in C code. This includes:

- Is the variable mutated by via pointer, directly, or not at all?
- Is the variable a pointer? If so, where does it point to, and does it mutate that variable.
- For what line-ranges does the variable appear to be in scope and not "behind a reference" (note that whether or not this actually follows borrow-checking rules is irrelevent to the analyzer, it simply collects information)

### Checker

The Checker performs a rudimentary, lexical form of borrow-checking, by validating the "lexical-lifetimes" of each mutable reference.
If any mutable reference to a piece of data overlaps with an immutable reference to that data or with the usage of the underlying value, the underlying variable, reference, and all other references to that variable are assumed to not follow borrow-checking rules, but still be "well-written," and are marked as `Rc<RefCell>>`s.
This isn't comprehensive borrow-checking and must be extended in numerous ways, most importantly to include function-based move semantics.

### Annotater

The Annotater takes the information about variables produced by the Analyzer and Checker and creates a new AST that includes this information in necessary places, for example:

- All declarations are annotated with whether the variable is mutable
- PtrDeclarations are annotated with the pointer type (`Rc<RefCell<T>>`, `&mut`, `&`, `*mut`, or `*const`)
  The generated AST is essentially a rudimentary Rust AST.

### Converter

The Converter takes the annotated AST and uses it to generate a corresponding Rust program.
