# sonder

## Scratch repo for potential science fair submission

Sonder is a static analyser and transpiler, for converting well-written C code to Rust

## What does well-written mean?

For the purposes of sonder, well-written means that for any given pointer, the C code in question either

- Treats it like a Rust reference in accordance with borrow-checking rules or
- Treats it a cloned `Rc<RefCell<T>>`

  If the code isn't well-written, sonder will fall back on unsafe raw pointers in the generated code, although at the moment, it panics

## Examples of "not-well-written" C code

1. The adding addresses

```c
&foo + &bar // illegal
```

2. Dereferencing pointer arithmatic

```c
*(foo + bar) // illegal
```

I'm aware that this includes legal array indexing, however, the sonder ast treats that as a totally seperate thing. So `arr[foo]` is legal while `*(arr + foo)` or `foo[arr]` aren't, even though they're equivalent.

3. Dereferencing non-ptrs

```c
*not_ptr // illegal
```

4. TEMPORARY ASSUMPTION: Mutable references can't be made unless they're tied to a ptr declaration.
   Adresses are always immutable unless explicitely annotated otherwise by the ptr declaration.

```rust
list.append(&mut other_list) // not something we're going to worry about for now
```

In the future, we will treat arguments as being bound to parameters as variables
So,`&mut other_list` will be treated as if it's bound to the variable value inside the function

Any of these will immediantly result in raw pointers being used, although at the moment, they panic

## TODO

- Write more test cases for the current prototype
- Figure out how to represent scope
- Create a system for managing scope
- Write scope-based borrowing checking

## How does all this work?

Sonder is broken up into a few differenc components for performing different tasks; they are run sequentially.
Note that all pointers are assumed to be either immutable or mutable references unless the checker deems otherwise

### Analyser

The Analyser determines the necessary variable semantics for performing borrow-checking on C code.
The primary data-structure involved is a map related variable names to information that would be aparent in Rust code, but must be inferred in C code. This includes:

- Is the variable mutated by via pointer, directly, or not at all?
- Is the variable a pointer? If so, where does it point to, and does it mutate that variable.
- For what line-ranges does the variable appear to be in scope and not "behind a reference" (note that whether or not this actually follows borrow-checking rules is irrelevent to the analyser, it simply collects information)

### Checker

The Checker performs a rudimentary, lexical form of borrow-checking, by validating the "lexical-lifetimes" of each mutable reference.
If any mutable reference to a piece of data overlaps with an immutable reference to that data or with the usage of the underlying value, the underlying variable, reference, and all other references to that variable are assumed to not follow borrow-checking rules, but still be "well-written," and are marked as `Rc<RefCell>>`s.
This isn't comprehensive borrow-checking and must be extended in numerous ways, most importantly to include function-based move semantics.

### Annotater

The Annotater takes the information about variables produced by the Analyser and Checker and creates a new AST that includes this information in necessary places, for example:

- All declarations are annotated with whether the variable is mutable
- PtrDeclarations are annotated with the pointer type (`Rc<RefCell<T>>`, `&mut`, `&`, `*mut`, or `*const`)
  The generated AST is essentially a rudimentary Rust AST.

### Converter

The Converter takes the annotated AST and uses it to generate a corresponding Rust program.
