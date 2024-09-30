use std::{fs::read_to_string, path::Path};

use parser::TokenNode;

#[allow(dead_code)]
mod analyzer;
mod converter;
mod error;
mod lexer;
mod parser;
mod test;

fn main() {
    let path = Path::new("test");
    let parsed = parse_c(path);
}

fn parse_c(file: &Path) -> TokenNode {
    let contents = read_to_string(file).expect("Please provide a valid file for parsing");
    let (tokens, line_numbers) = lexer::string_to_tokens(contents)
        .expect("Failed to lex tokens, please provide valid C code");
    parser::program(tokens, line_numbers, true)
        .expect("Faild to parse tokens, plaes provide valid C code")
}
