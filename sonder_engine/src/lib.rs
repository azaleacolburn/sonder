use proc_macro::TokenStream;
use regex::Regex;
use std::fs;
use std::num::ParseIntError;
use syn::{parse_macro_input, LitStr};

// #[proc_macro]
// pub fn import_c_static(input: TokenStream) -> TokenStream {
//     let parsed_input: Vec<String> = input
//         .to_string()
//         .split(", ")
//         .map(|n| n.to_owned())
//         .collect();
//     let raw_path = parsed_input[0].clone().parse().unwrap();
//     let raw_item = parsed_input[1].clone().parse().unwrap();
//     let path = parse_macro_input!(raw_path as LitStr).value();
//     let item = parse_macro_input!(raw_item as LitStr).value();
//
//     let fc = fs::read_to_string(path).unwrap();
//
//     let pattern = Regex::new(format!(
//         r#"static\s+const((int|char)\s+{item}*\s*=\s*([0-9]+|'([^'\\\n]|\\.))|char*\s+{item}*\s*=\s*(".*"));"#
//     ).as_str()).expect("Invalid regex");
//
//     let declaration = pattern
//         .find(fc.as_str())
//         .expect("No matches found")
//         .as_str();
//
//     let parsing_pattern = Regex::new(format!(r"(char*?\s*|int\s+)"))
//
//     static THIS: &str = "test";
//     // static const char*this="test";
//     // static char this="test";
//
//     let split: Vec<&str> = declaration.split("=").collect();
//     let mut first_half: Vec<&str> = split[0].split(" ").collect();
//     let type_name = first_half[2];
//
//     let lexed = c_string_to_tokens(declaration).expect("Failed to lex static");
//     let parsed = parse_c_static_to_rust(lexed);
//
//     println!("{parsed}");
//     "println!(\"test\")".parse().unwrap()
// }

#[proc_macro]
pub fn import_c_struct(input: TokenStream) -> TokenStream {
    let parsed_input: Vec<String> = input
        .to_string()
        .split(", ")
        .map(|n| n.to_owned())
        .collect();
    let raw_path = parsed_input[0]
        .clone()
        .parse()
        .expect("Please provide a file import path in the form of a string");
    let raw_item = parsed_input[1]
        .clone()
        .parse()
        .expect("Please provide an item to be imported in the form of a string");
    let path = parse_macro_input!(raw_path as LitStr).value();
    let item = parse_macro_input!(raw_item as LitStr).value();

    let fc = fs::read_to_string(path).expect("C file not found");

    let pattern = Regex::new(format!(r#"struct\s+{item}\s*\{{(\s*((int|char)\**|void\*+)\s+[a-zA-Z_][a-zA-Z0-9_]*\s*;)*\s*\}}\s*;"#).as_str())
        .expect("Invalid regex");

    let declaration = pattern
        .find(fc.as_str())
        .expect("Item not found in file. Did you mean to import a function?")
        .as_str();

    // It's fine that these functions panic because the regex only matches to valid function declarations
    let lexed = c_string_to_tokens(declaration).expect("Failed to lex struct");
    let parsed = parse_c_struct_to_rust(lexed);

    println!("{parsed}");
    parsed
}

#[proc_macro]
pub fn import_c_function(input: TokenStream) -> TokenStream {
    let parsed_input: Vec<String> = input
        .to_string()
        .split(", ")
        .map(|n| n.to_owned())
        .collect();
    let raw_path = parsed_input[0]
        .clone()
        .parse()
        .expect("Please provide a file import path in the form of a string");
    let raw_item = parsed_input[1]
        .clone()
        .parse()
        .expect("Please provide an item to be imported in the form of a string");
    let path = parse_macro_input!(raw_path as LitStr).value();
    let item = parse_macro_input!(raw_item as LitStr).value();

    let fc = fs::read_to_string(path).expect("C file not found");

    // TODO: Figure out how to allow typedefs, maybe preprocess c file first?
    let pattern = Regex::new(
        format!(r"((struct\s+{item}|int|char)\s*(\**\s)*|void\s*\*+(\s*\*)*)\s*([a-zA-Z0-9_]*)\s*\((((struct\s+[a-zA-Z0-9_]+|int|char)\s*\**|void\s*\*+)\s*[a-zA-Z_][a-zA-Z0-9_]*\s*,?\s*)*\s*\)")
            .as_str(),
    )
    .expect("Invalid regex");

    let declaration = pattern
        .find(fc.as_str())
        .expect("Item not found in file. Did you mean to import a struct?")
        .as_str();

    // It's fine that these functions panic because the regex only matches to valid function declarations
    let lexed = c_string_to_tokens(declaration).expect("Failed to lex declaration");
    let parsed = parse_c_function_declaration_to_rust(lexed);

    println!("{parsed}");
    parsed
}
/// This is where the lexical analysis happens
fn c_string_to_tokens(buff: impl ToString) -> Result<Vec<Token>, ParseIntError> {
    let mut ret: Vec<Token> = vec![];
    let chars = buff.to_string().chars().collect::<Vec<char>>();
    let mut curr: String = String::from("");
    let mut i: usize = 0;
    while i < chars.len() {
        match chars[i] {
            ' ' => {}
            '\t' => {}
            '\n' => {}
            'i' => {
                if chars[i + 1] == 'n' && chars[i + 2] == 't' && chars[i + 3] == ' ' {
                    // split.push(String::from("int"));
                    ret.push(Token::Int);
                    i += 2; // I think there's a problem with incrementing the iterator
                } else {
                    for j in i..chars.len() {
                        if !chars[j].is_alphabetic() && chars[j] != '_' {
                            break;
                        }
                        curr.push(chars[j]);
                    }
                    ret.push(Token::Id(curr.clone()));
                    i += curr.len() - 1;
                    curr = String::from("");
                }
            }
            'c' => {
                if chars[i + 1] == 'h'
                    && chars[i + 2] == 'a'
                    && chars[i + 3] == 'r'
                    && chars[i + 4] == ' '
                {
                    // split.push(String::from("char"));
                    ret.push(Token::Char);
                    i += 3;
                } else if chars[i + 1] == 'o'
                    && chars[i + 2] == 'n'
                    && chars[i + 3] == 's'
                    && chars[i + 4] == 't'
                {
                    ret.push(Token::Const);
                    i += 3;
                } else {
                    for j in i..chars.len() {
                        if !chars[j].is_alphabetic() && chars[j] != '_' {
                            break;
                        }
                        curr.push(chars[j]);
                    }
                    ret.push(Token::Id(curr.clone()));
                    i += curr.len() - 1;
                    curr = String::from("");
                }
            }
            's' => {
                if chars[i + 1] == 't'
                    && chars[i + 2] == 'r'
                    && chars[i + 3] == 'u'
                    && chars[i + 4] == 'c'
                    && chars[i + 5] == 't'
                {
                    ret.push(Token::Struct);
                    i += 5;
                } else if chars[i + 1] == 't'
                    && chars[i + 2] == 'a'
                    && chars[i + 3] == 't'
                    && chars[i + 4] == 'i'
                    && chars[i + 5] == 'c'
                {
                    ret.push(Token::Static);
                    i += 5;
                } else {
                    for j in i..chars.len() {
                        if !chars[j].is_alphabetic() && chars[j] != '_' {
                            break;
                        }
                        curr.push(chars[j]);
                    }
                    ret.push(Token::Id(curr.clone()));
                    i += curr.len() - 1;
                    curr = String::from("");
                }
            }
            '-' => ret.push(Token::Dash),
            '=' => ret.push(Token::Equals),
            '*' => ret.push(Token::Star),

            // obviously none of this can be included in ids
            '(' => ret.push(Token::OParen),
            ')' => ret.push(Token::CParen),
            '[' => ret.push(Token::OSquare),
            ']' => ret.push(Token::CSquare),
            '{' => ret.push(Token::OCurl),
            '}' => ret.push(Token::CCurl),
            ',' => ret.push(Token::Comma),
            ';' => ret.push(Token::Semi),
            'v' => {
                if chars[i + 1] == 'o'
                    && chars[i + 2] == 'i'
                    && chars[i + 3] == 'd'
                    && (chars[i + 4] == ' ' || chars[i + 4] == '*')
                {
                    ret.push(Token::Void);
                    i += 3;
                } else {
                    for j in i..chars.len() {
                        if !chars[j].is_alphabetic() && chars[j] != '_' {
                            break;
                        }
                        curr.push(chars[j]);
                    }
                    ret.push(Token::Id(curr.clone()));
                    i += curr.len() - 1;
                    curr = String::from("");
                }
            }
            _ => {
                // if we'e here it's an identifier
                for j in i..chars.len() {
                    if !chars[j].is_alphabetic() && chars[j] != '_' {
                        break;
                    }
                    curr.push(chars[j]);
                }
                ret.push(Token::Id(curr.clone()));
                i += curr.len() - 1;
                curr = String::from("");
            }
        }
        i += 1;
    }
    Ok(ret)
}

#[derive(Debug, PartialEq, Clone)]
enum Token {
    Struct,
    Semi,
    Star,
    Static,
    Const,
    Id(String),
    OParen,
    CParen,
    OSquare,
    CSquare,
    OCurl,
    CCurl,
    Dot,
    Comma,
    Dash,
    Equals,
    Char,
    Int,
    Void,
    Unsigned,
}

fn parse_c_function_declaration_to_rust(tokens: Vec<Token>) -> TokenStream {
    let mut token_stream = tokens.iter();
    let return_type: String = match token_stream.next().unwrap() {
        Token::Char => String::from(" -> i8"),
        Token::Int => String::from(" -> i16"),
        Token::Void => String::from(" "),
        Token::Struct => {
            if let Token::Id(id) = token_stream.next().unwrap() {
                format!(" -> {id}")
            } else {
                panic!("Expected id after struct keyword");
            }
        }
        // TODO: Port over structs
        _ => panic!("Invalid return type"),
    };
    let id: String = match token_stream.next().unwrap() {
        Token::Id(id) => id.to_string(),
        _ => panic!("Expected identifier"),
    };
    if *token_stream.next().unwrap() != Token::OParen {
        panic!("Expected open parenthesis");
    }
    let mut t = token_stream.next().unwrap();
    let mut args: Vec<String> = vec![];
    while *t == Token::Char || *t == Token::Int || *t == Token::Void || *t == Token::Struct {
        let next = token_stream.next().unwrap();
        let mut next_next = token_stream.next().unwrap();

        let arg: String = match (t, next, next_next) {
            (Token::Struct, Token::Id(c_type_id), Token::Star) => {
                let name = if let Token::Id(id) = token_stream.next().unwrap() {
                    id
                } else {
                    panic!("Expected id after for arg name");
                };
                next_next = token_stream.next().unwrap();
                format!("{name}: &{c_type_id}")
            }
            (Token::Struct, Token::Id(struct_id), Token::Id(arg_id)) => {
                next_next = token_stream.next().unwrap();
                format!("{arg_id}: {struct_id}")
            }
            (c_type, Token::Star, Token::Id(id)) => {
                let rust_type = match c_type {
                    Token::Int => "i16",
                    Token::Char => "i8",
                    Token::Void => "()",
                    _ => panic!("Invalid primitive type"),
                };
                next_next = token_stream.next().unwrap();
                format!("{id}: &{rust_type}")
            }
            (c_type, Token::Id(id), _after) => {
                let rust_type = match c_type {
                    Token::Int => "i16",
                    Token::Char => "i8",
                    Token::Void => "()",
                    _ => panic!("Invalid primitive type"),
                };
                format!("{id}: {rust_type}")
            }
            (Token::Void, _, _) => {
                panic!("Void type must be a void*");
            }
            (_, _, _) => {
                panic!("Expected declaration");
            }
        };

        args.push(arg);
        match next_next {
            Token::Comma => {
                t = token_stream.next().unwrap();
            }
            Token::CParen => {
                break;
            }
            _ => panic!("expected comma or cparen"),
        }
    }
    let formatted_args = args.join(", ");
    // This is just a trick to let us compile, the binaries will be statically linked dw
    let parsed = format!("extern \"C\" {{\nfn {id} ({formatted_args}){return_type};\n}}");
    return parsed.parse().unwrap();
}

fn parse_c_struct_to_rust(tokens: Vec<Token>) -> TokenStream {
    let mut token_stream = tokens.iter();
    if *token_stream.next().unwrap() != Token::Struct {
        panic!("Expected struct keyword");
    }

    let id = if let Token::Id(id) = token_stream.next().unwrap() {
        id
    } else {
        panic!("expected id token");
    };

    if *token_stream.next().unwrap() != Token::OCurl {
        panic!("Expected open curl");
    }

    let mut t = token_stream.next().unwrap();
    let mut fields: Vec<String> = vec![];
    while *t == Token::Int || *t == Token::Char || *t == Token::Void {
        let mut field_type = String::new();
        let mut symbol = token_stream.next().unwrap();
        if *symbol == Token::Star {
            field_type.push_str("&");
            symbol = token_stream.next().unwrap();
        }

        let c_type = match t {
            Token::Char => "i8",
            Token::Int => "i16",
            Token::Void => "()",
            _ => panic!("Invalid type token"),
        };

        let id = match symbol {
            Token::Id(id) => id,
            _ => panic!("Expected identifier"),
        };

        fields.push(format!("{id}: {c_type}"));

        match token_stream.next().unwrap() {
            Token::Semi => t = token_stream.next().unwrap(),
            Token::CCurl => break,
            _ => panic!("expected closing curl or semi"),
        }
    }

    let formatted_fields = fields.join(",\n");
    let parsed = format!("struct {id} {{\n{formatted_fields}\n}}");

    return parsed.parse().unwrap();
}

// fn parse_c_static_to_rust(tokens: Vec<Token>) -> TokenStream {
//     let mut token_stream = tokens.iter();
//     if *token_stream.next().unwrap() != Token::Static {
//         panic!("Expected static keyword");
//     }
//
//     if *token_stream.next().unwrap() != Token::Const {
//         panic!("Expected const keyword");
//     }
//
//     let id = if let Token::Id(id) = token_stream.next().unwrap() {
//         id
//     } else {
//         panic!("Expected id");
//     };
//
//     if *token_stream.next().unwrap() != Token::Equals {
//         panic!("Expected equals sign");
//     }
//
//     todo!()
// }
