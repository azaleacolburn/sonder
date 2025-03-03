use std::fs::read_to_string;

use analysis_ctx::AnalysisContext;
use ast::TokenNode;

mod adjuster;
mod analysis_ctx;
#[allow(dead_code)]
mod analyzer;
mod annotater;
mod ast;
mod checker;
mod converter;
mod data_model;
mod error;
mod lexer;
mod parser;
#[cfg(test)]
mod test;
mod token_handler;

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

fn convert_to_rust_code(mut ast: TokenNode) -> String {
    ast.print(&mut 0);
    let mut ctx: AnalysisContext = AnalysisContext::new();

    analyzer::determine_var_mutability(&ast, &mut ctx, Box::new([]), 0);

    println!("variables: {:?}", ctx.variables);

    let temp_ctx = ctx.clone();
    let errors = checker::borrow_check(&temp_ctx);
    adjuster::adjust_ptr_type(errors, &mut ctx, &mut ast);

    let annotated_ast = annotater::annotate_ast(&ast, &ctx);
    // annotated_ast.print(&mut 0);

    let converted_rust = converter::convert_annotated_ast(&annotated_ast);
    println!("\n{converted_rust}");
    converted_rust
}
