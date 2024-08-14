extern crate proc_macro;
use std::fs;

use proc_macro::{TokenStream, TokenTree};
use quote::quote;
use syn::{parse_macro_input, ExprPath, ItemStatic, ItemStruct, LitStr};

#[proc_macro_attribute]
pub fn import_c(_metadata: TokenStream, input: TokenStream) -> TokenStream {
    let id_to_import = input.to_string().split(",").collect::<Vec<&str>>();
    let extern_file_contents = fs::read_to_string(input_path);
}
