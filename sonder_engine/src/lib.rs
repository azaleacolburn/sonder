use proc_macro::{TokenStream, TokenTree};
use regex::Regex;
use std::num::ParseIntError;
use std::{fs, str::FromStr};
use syn::{parse_macro_input, LitStr};

#[proc_macro]
pub fn import_c(input: TokenStream) -> TokenStream {
    let parsed_input: Vec<String> = input
        .to_string()
        .split(", ")
        .map(|n| n.to_owned())
        .collect();
    let raw_path = parsed_input[0].clone().parse().unwrap();
    let raw_item = parsed_input[1].clone().parse().unwrap();
    let path = parse_macro_input!(raw_path as LitStr).value();
    let item = parse_macro_input!(raw_item as LitStr).value();

    let fc = fs::read_to_string(path).unwrap();

    // TODO: Figure out how to allow typedefs, maybe preprocess c file first?
    let pattern = Regex::new(
        format!(r"^ *(int|char|void)( \*|\*|\* | ){item} ?\(((int|char|void)( \*|\*|\* | )[a-zA-Z0-9_]+,? ?)*\)")
            .as_str(),
    )
    .expect("Invalid regex");

    let declaration = pattern
        .find(fc.as_str())
        .expect("No matches found")
        .as_str();
    println!("{}", declaration);

    let output = format!("println!(\"this macro needs to output something for now\");");
    output.parse().unwrap()
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
            'i' => {
                if chars[i + 1] == 'n' && chars[i + 2] == 't' && chars[i + 3] == ' ' {
                    // split.push(String::from("int"));
                    ret.push(Token::Type(RhTypes::Int));
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
                    ret.push(Token::Type(RhTypes::Char));
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
                }
            }
            '-' => {
                ret.push(Token::Dash);
            }

            '*' => {
                // split.push(String::from("*"));
                ret.push(Token::Star); // The lexer can probably determine whether this is a mul or deref
            }
            // obviously none of this can be included in ids
            '(' => {
                ret.push(Token::OParen);
            }
            ')' => {
                ret.push(Token::CParen);
            }
            '[' => ret.push(Token::OSquare),
            ']' => ret.push(Token::CSquare),

            ',' => {
                // split.push(String::from(","));
                ret.push(Token::Comma);
            }
            'v' => {
                if chars[i + 1] == 'o'
                    && chars[i + 2] == 'i'
                    && chars[i + 3] == 'd'
                    && (chars[i + 4] == ' ' || chars[i + 4] == '*')
                {
                    ret.push(Token::Type(RhTypes::Void));
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
    Fn,
    Type(RhTypes),
    Struct,
    Star,
    Id(String),
    OParen,
    CParen,
    OSquare,
    CSquare,
    Dot,
    Comma,
    Dash,
}

#[derive(Debug, PartialEq, Clone)]
enum RhTypes {
    Char,
    Int,
    Void,
}

fn parsed_c_declaration_to_rust(tokens: Vec<Tokens>) -> TokenStream {
    let 
    let parsed = format!("fn {}");
}
