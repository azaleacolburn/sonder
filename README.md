# sonder

## Scratch repo for potential science fair submission

Sonder is a static analysis tool for C, for determining the necessary sementics for converting the analyzed C programs to Rust.

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
