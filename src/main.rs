use std::fs::read_to_string;

use parser::TokenNode;

#[allow(dead_code)]
mod analyzer;
mod converter;
mod error;
mod lexer;
mod parser;
#[cfg(test)]
mod test;

fn main() {
    let contents = read_to_string("test.c").expect("Please provide a valid file for parsing");
    let _ast = parse_c(contents);
}

fn parse_c(contents: String) -> TokenNode {
    let (tokens, line_numbers) = lexer::string_to_tokens(contents)
        .expect("Failed to lex tokens, please provide valid C code");
    println!("{:?}", tokens);
    parser::program(tokens, line_numbers, true)
        .expect("Faild to parse tokens, plaes provide valid C code")
}
