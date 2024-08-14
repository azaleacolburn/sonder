# sonder

## Scratch repo for potential science fair submission

Cross language compilation and linking with wasm as a shared target

## Plans

- Use proc macros in rust to be able to call C functions exactly like rust functions (including getting lsp completions, etc)
- Compile everything to wasm to avoid the issues of conforming rust code to the c_abi and deal with an ffi (or actually we're just inventing our own)

## Purpose

- Allow rust code to seamlessly interop with old C(++) code as it incrementally replaces a codebase
