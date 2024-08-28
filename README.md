# sonder

## Scratch repo for potential science fair submission

A macro system and linker that compiles c and rust to wasm and statically links them
or
Cross language compilation and linking with wasm as a shared target

## Plans

- Use proc macros in rust to be able to call C functions exactly like rust functions (including getting lsp completions, etc)
- Compile everything to wasm to avoid the issues of conforming rust code to the c_abi and deal with an ffi (or actually we're just inventing our own)
- Fully compile check the imported c source and have some assurance of semi-safe interop at compile time

### Proc Macros

- [x] Proc Macros for automatic extern "C" function declaration
- [x] Proc Macros for automatic extern "C" struct declaration
- [ ] Proc Macros for automatic extern "C" static declaration

### Linking

- [ ] Get list of all involved files (maybe just start with all file in root dir)
- [ ] Compile all relevent files to wasm
- [ ] Insert "linking markers" to where things need to be linked
- [ ] Inline into one "binary", or figure out how statically linked wasm binaries work :P

## Purpose

- Allow rust code to seamlessly interop with old C(++) code as it incrementally replaces a codebase, with static linking and code editor completions.

### Notes

- Compile Rust to wasm: `cargo build --target wasm32-unknown-unknown`
- Compile C to wasm: `./wasi-sdk-24.0/bin/clang --sysroot wasi-sdk-24.0/share/wasi-sysroot/ hello.c -o hello.wasm`
