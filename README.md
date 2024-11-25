# sonder

## Scratch repo for potential science fair submission

Sonder is a static analysis tool for C, for determining the necessary sementics for converting the analyzed C programs to Rust.

## Assumptions

1. The address of only one variable at a time is taken

```c
&(foo + bar) // illegal
```

2. Only one pointer is dereferenced at a time

```c
*(foo + bar) // illegal
```

3. Only pointers are dereferenced

```c
*not_ptr // illegal
```

4. UNSAFE ASSUMPTION: Mutatble references can't be made unless they're tied to a ptr declaration.
   Adresses are always immutable unless explicitely annotated otherwise by the ptr declaration.

```rust
list.append(&mut other_list) // not something we're going to worry about for now
```

In the future, we will treat arguments as being bound to parameters as variables
So,`&mut other_list` will be treated as if it's bound to the variable value inside the function

Any of these will immediantly result in raw pointers being used, although at the moment, they panic

## TODO

- Figure out how to represent lifetimes
  - This seems to rely on line numbers, which are lost information.
  - Perhaps we could find a way to represent lifetimes through position on the ast, but that seems exceedingly difficult
  - Including line-numbers in the initial ast seems to be the way to go

```rust
let t = 8;
let n = &t;
// n dropped here
let m = &mut t;
// m dropped here
```

- Figure out how to represent scope

## Steps

1. Get list of all owned data
2. For each piece of owned data:

   1. Get list of all usages of all refs to it in each scope
   2. Iterate through scope to check if each one is mutable, or immutable
      - this may include parsing called scopes
      - some escape analysis may be will here
      - whether full borrow-checking will be needed is still unclear
      - we will need a way of checking branching
      - this includes being passed mutably, or mutating
   3. Count to make sure at any given point, only a single mutable, or any number of immutable references are used (otherwise leave it as a raw pointer)
   4. If we have time, check for common Smart Pointer semantics

3. Generate a new AST with previously implicit pointer semantics made explicit
4. Try converting the new AST to semantically identical, idiomatic Rust.

### Branch handing

- Branches are counted as their own scopes
- Exclusive branches can hold different references to an object
- But they all must consider the higher-scoped references to higher-scoped data
- Assume all non-exclusive branches occur
