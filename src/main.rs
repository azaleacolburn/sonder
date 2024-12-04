use std::{collections::HashMap, fs::read_to_string, ops::Range};

use analyzer::{AnalysisContext, VarData};
use parser::TokenNode;

#[allow(dead_code)]
mod analyzer;
mod annotater;
mod checker;
mod converter;
mod error;
mod lexer;
mod parser;
#[cfg(test)]
mod test;

fn main() {
    let contents = read_to_string("test.c").expect("Please provide a valid file for parsing");

    let ast = parse_c(contents);
    let _rust_code = convert_to_rust_code(ast);
}

fn parse_c(contents: String) -> TokenNode {
    let (tokens, line_numbers) = lexer::string_to_tokens(contents)
        .expect("Failed to lex tokens, please provide valid C code");
    println!("{:?}", tokens);
    println!("{:?}", line_numbers);
    parser::program(tokens, line_numbers, true).expect("Failed to parse token stream")
}

fn convert_to_rust_code(ast: TokenNode) -> String {
    ast.print(&mut 0);
    let ctx: AnalysisContext = AnalysisContext::new();

    let mut var_info = analyzer::determine_var_mutability(&ast, ctx);
    // println!(
    //     "{:?}",
    //     var_info
    //         .iter()
    //         .map(|(id, data)| (id.clone(), data.non_borrowed_lines.clone()))
    //         .collect::<Vec<(String, Vec<Range<usize>>)>>()
    // );

    let errors = checker::borrow_check(&ctx);
    checker::adjust_ptr_type(errors, &mut ctx);
    let annotated_ast = annotater::annotate_ast(&ast, &var_info);
    annotated_ast.print(&mut 0);

    let converted_rust = converter::convert_annotated_ast(&annotated_ast);
    println!("{converted_rust}");
    converted_rust
}
