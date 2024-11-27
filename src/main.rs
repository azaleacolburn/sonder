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

    parse_c(contents);
}

fn parse_c(contents: String) -> TokenNode {
    let (tokens, line_numbers) = lexer::string_to_tokens(contents)
        .expect("Failed to lex tokens, please provide valid C code");
    println!("{:?}", tokens);
    println!("{:?}", line_numbers);
    parser::program(tokens, line_numbers, true).expect("Failed to parse token stream")
}
