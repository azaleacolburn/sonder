# sonder

## Scratch repo for potential science fair submission

A macro system and linker that compiles c and rust to wasm and statically links them

## Plans

- Use macros in rust to be able to call C functions exactly like rust functions (including getting lsp completions, etc)
- Compile everything to wasi to avoid the issues of conforming rust code to the c_abi

## Purpose

- Allow rust code to seamlessly interop with old C(++) code as it incrementally replaces a codebase
