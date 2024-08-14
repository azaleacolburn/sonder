extern crate proc_macro;
use proc_macro::TokenStream;
use regex::Regex;
use std::{fs, str::FromStr};

#[proc_macro_attribute]
pub fn import_c(metadata: TokenStream, input: TokenStream) -> TokenStream {
    let parsed_input: Vec<String> = metadata
        .to_string()
        .split(",")
        .map(|n| n.to_owned())
        .collect();
    let import_path = parsed_input[0].clone();
    let import_item = parsed_input[1].clone();
    let extern_file_contents = fs::read_to_string(import_path).expect("File doesn't exist");
    let pattern = Regex::new(format!(r"(?m)^\s*\b(int|char|void)\b\s+{import_item}\s").as_str())
        .expect("Invalid regex");
    let match_line = pattern
        .find(extern_file_contents.as_str())
        .expect("No matches found")
        .as_str();
    println!("{}", match_line);

    // TODO: figure out how to agree linking at comp time
    // let res = TokenStream::from_str("")

    input
}
