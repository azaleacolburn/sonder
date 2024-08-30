# sonder

## Scratch repo for potential science fair submission

Cross language compilation and linking with wasm as a shared target.
Recently, there has been an outpour of new, compiled, imperative languages without a garbage collector. Sonder aims to make interop between any of these new languages and Rust (on the Rust end) trivial.

## Plans

### Preliminary work

- Use proc macros in rust to be able to call C functions exactly like rust functions (including getting lsp completions, etc)
- Compile and link everything together.

### Main project

- Create a proc macro or build tool for defining the header/declaration grammar of a new language.
  - Generate regex for grepping these declarations
  - Create macros that understand this and can seamlessly import functions, statics, and structs from any of there languages into Rust.
  - Track all involved files and compile them each to WASM
  - Link all the WASM files in the needed places

### If we really pop off this year

- Fully compile check the imported c source and have some assurance of semi-safe interop at compile time, without creating additional rust abstractions.
- Put the rhododendron parser into a macro and 

### Basic Bindgen

- [x] Proc Macros for automatic extern "C" function declaration
- [x] Proc Macros for automatic extern "C" struct declaration
- [ ] Proc Macros for automatic extern "C" static declaration

- [ ] Proc Macro to parse grammar and cache a novel regex for how to grep declarations in the header file of that language.
      - Note that header files can just be 

### Linking

- [ ] Get list of all involved files (maybe just start with all file in root dir)
- [ ] Compile all relevent files to wasm
- [ ] Insert "linking markers" to where things need to be linked
- [ ] Inline into one "binary", or figure out how statically linked wasm binaries work :P

## Purpose

- Allow Rust code to interop with any language given a header file grammar (and a compiler that supports either the C ABI or wasm as a target) with static linking and code editor completions in Rust.

## Notes

- Compile Rust to wasm: `cargo build --target wasm32-unknown-unknown`
- Compile C to wasm: `./wasi-sdk-24.0/bin/clang --sysroot wasi-sdk-24.0/share/wasi-sysroot/ hello.c -o hello.wasm`
