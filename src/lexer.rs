use std::num::ParseIntError;

/// Each index is a new line, the value is the token_i that starts that line
#[derive(Debug, Clone)]
pub struct LineNumHandler {
    pub token_lines: Vec<usize>,
}

impl LineNumHandler {
    pub fn new() -> LineNumHandler {
        LineNumHandler {
            token_lines: vec![],
        }
    }

    /// Creates a new line with the start of the line being this token_number
    fn new_line(&mut self, token_number: usize) {
        self.token_lines.push(token_number);
    }

    /// Given a token index, returns the line that token was on
    /// For external use only
    pub fn _get_line(&self, token_number: usize) -> usize {
        self.token_lines
            .iter()
            .position(|n| *n < token_number)
            .expect("Invalid Token Number For Getting Line Number")
    }
}

/// This is where the lexical analysis happens
pub fn string_to_tokens(
    buff: impl ToString,
) -> Result<(Vec<Token>, LineNumHandler), ParseIntError> {
    let mut ret: Vec<Token> = vec![];
    let chars = buff.to_string().trim().chars().collect::<Vec<char>>();
    let mut curr: String = String::from("");
    let mut i: usize = 0;
    let mut line_tracker = LineNumHandler::new();
    line_tracker.new_line(1);
    while i < chars.len() {
        // Handles num literals but we don't actually know if it is a literal yet
        if chars[i].is_numeric() {
            let mut is_dec = true;
            let mut num = String::from("");
            for c in chars.iter().skip(i) {
                if !c.is_alphanumeric() {
                    break;
                }
                if c.is_alphabetic() && c.is_uppercase() {
                    is_dec = false;
                }
                num.push(*c);
            }
            if chars[i] == '0' {
                // handles literals // TODO: DO LITERAL SHIT
                // let string = chars.into_iter().collect::<String>();

                let mut radix = 0; // 0 is not extranious base value
                match chars[i + 1] {
                    'x' => {
                        // hex
                        radix = 12;
                    }
                    'o' => {
                        // octal
                        radix = 8;
                    }
                    'b' => {
                        // binary
                        radix = 2;
                    }
                    _ => {
                        if chars[i + 1].is_alphabetic() {
                            panic!("Not supported base")
                        }
                    }
                }
                if radix != 0 {
                    match usize::from_str_radix(&num, radix) {
                        Ok(value) => {
                            ret.push(Token::NumLiteral(value));
                        }
                        Err(_err) => {
                            continue;
                        }
                    };
                    i += 1;
                    continue;
                }
            }
            if is_dec {
                ret.push(Token::NumLiteral(num.parse::<usize>().unwrap()));
                i += num.len();
                continue;
            }
        }

        match chars[i] {
            'a' if chars[i + 1] == 's'
                && chars[i + 2] == 's'
                && chars[i + 3] == 'e'
                && chars[i + 4] == 'r'
                && chars[i + 5] == 't'
                && chars[i + 6] == ' ' =>
            {
                ret.push(Token::Assert);
                i += 5;
            }
            'a' if chars[i + 1] == 's'
                && chars[i + 2] == 'm'
                && (chars[i + 3] == ' ' || chars[i + 3] == '(') =>
            {
                ret.push(Token::Asm);
                i += 2;
            }
            'p' if chars[i + 1] == 'u'
                && chars[i + 2] == 't'
                && (chars[i + 3] == '(' || chars[i + 3] == ' ') =>
            {
                ret.push(Token::PutChar);
                i += 2;
            }
            's' if chars[i + 1] == 't'
                && chars[i + 2] == 'r'
                && chars[i + 3] == 'u'
                && chars[i + 4] == 'c'
                && chars[i + 5] == 't'
                && chars[i + 6] == ' ' =>
            {
                ret.push(Token::Struct);
                i += 6;
            }

            'L' if chars[i + 1] == 'A'
                && chars[i + 2] == 'B'
                && chars[i + 3] == 'E'
                && chars[i + 4] == 'L'
                && chars[i + 5] == ':' =>
            {
                for c in chars.iter().skip(i) {
                    if !c.is_alphabetic() && *c != '_' {
                        break;
                    }
                    curr.push(*c);
                }
                ret.push(Token::Label(curr.clone()));
                i += curr.len() - 1;
                curr = String::from("");
            }
            'g' if chars[i + 1] == 'o' && chars[i + 2] == 't' && chars[i + 3] == 'o' => {
                i += 4;
                for c in chars.iter().skip(i) {
                    if !c.is_alphabetic() && *c != '_' {
                        break;
                    }
                    curr.push(*c);
                }
                ret.push(Token::Goto(curr.clone()));
                i += curr.len() - 1;
                curr = String::from("");
            }
            'r' if chars[i + 1] == 'e'
                && chars[i + 2] == 't'
                && chars[i + 3] == 'u'
                && chars[i + 4] == 'r'
                && chars[i + 5] == 'n'
                && (chars[i + 6] == ' ' || chars[i + 6] == '(') =>
            {
                ret.push(Token::Return);
                i += 5;
            }
            'i' if chars[i + 1] == 'n' && chars[i + 2] == 't' && chars[i + 3] == ' ' => {
                ret.push(Token::Type(CType::Int));
                i += 3; // NOTE Saves an iteration
            }
            'i' if chars[i + 1] == 'n' && chars[i + 2] == 't' && chars[i + 3] == '*' => {
                ret.push(Token::Type(CType::Int));
                i += 2; // NOTE Not a bug, don't move past a star
            }
            'i' if chars[i + 1] == 'f' && (chars[i + 2] == ' ' || chars[i + 2] == '(') => {
                ret.push(Token::If);
                i += 1;
            }
            'w' if chars[i + 1] == 'h'
                && chars[i + 2] == 'i'
                && chars[i + 3] == 'l'
                && chars[i + 4] == 'e' =>
            {
                ret.push(Token::While);
                i += 4;
            }
            'v' if chars[i + 1] == 'o'
                && chars[i + 2] == 'i'
                && chars[i + 3] == 'd'
                && chars[i + 4] == ' ' =>
            {
                ret.push(Token::Type(CType::Void));
                i += 4;
            }
            'v' if chars[i + 1] == 'o'
                && chars[i + 2] == 'i'
                && chars[i + 3] == 'd'
                && chars[i + 4] == '*' =>
            {
                ret.push(Token::Type(CType::Void));
                i += 3; // NOTE Saves an iteration
            }

            '+' if chars[i + 1] == '=' => {
                ret.push(Token::AddEq);
                i += 1;
            }
            '+' if chars[i + 1] == '+' => {
                ret.push(Token::AddO);
                i += 1;
            }
            '+' => {
                ret.push(Token::Add);
            }
            '-' if chars[i + 1] == '=' => {
                ret.push(Token::SubEq);
                i += 1;
            }
            '-' if chars[i + 1] == '-' => {
                ret.push(Token::SubO);
                i += 1;
            }
            '-' if chars[i + 1] == '>' => {
                ret.push(Token::Arrow);
                i += 1;
            }
            '-' if chars[i + 1].is_numeric() => {
                let mut is_dec = true;
                // chars.into_iter().for_each(|x| if !x.is_numeric() { is_dec = false; });
                let mut num = String::from("-");
                for c in chars.iter().skip(i) {
                    if !c.is_alphanumeric() {
                        break;
                    }
                    if c.is_alphabetic() && c.is_uppercase() {
                        is_dec = false;
                    }
                    num.push(*c);
                }
                if chars[i + 1] == '0' {
                    i += 1;
                    // handles literals // TODO: DO LITERAL SHIT
                    // let string = chars.into_iter().collect::<String>();

                    let mut radix = 0; // 0 is not extranious base value
                    match chars[i + 1] {
                        'x' => {
                            // hex
                            radix = 12;
                        }
                        'o' => {
                            // octal
                            radix = 8;
                        }
                        'b' => {
                            // binary
                            radix = 2;
                        }
                        _ => {
                            if chars[i + 1].is_alphabetic() {
                                panic!("Not supported base")
                            }
                        }
                    }
                    if radix != 0 {
                        match usize::from_str_radix(&num, radix) {
                            Ok(value) => {
                                ret.push(Token::NumLiteral(value));
                            }
                            Err(_) => {
                                continue;
                            }
                        };
                        i += 1;
                        continue;
                    }
                }
                if is_dec {
                    ret.push(Token::NumLiteral(num.parse::<usize>().unwrap()));
                    i += num.len();
                    continue;
                }
            }
            '-' => {
                ret.push(Token::Sub);
            }
            '/' if chars[i + 1] == '=' => {
                ret.push(Token::DivEq);
                i += 1;
            }
            '/' if chars[i + 1] == '/' => {
                i += 1;
                while chars[i] != '\n' {
                    i += 1;
                }
            }
            '/' => {
                ret.push(Token::Div);
            }
            '*' if chars[i + 1] == '=' => {
                ret.push(Token::MulEq);
                i += 1;
            }
            '*' => {
                ret.push(Token::Star); // The lexer can probably determine whether this is a mul or deref
            }
            // obviously none of this can be included in ids
            '(' => {
                ret.push(Token::OParen);
            }
            ')' => {
                ret.push(Token::CParen);
            }
            '{' => {
                ret.push(Token::OCurl);
            }
            '}' => ret.push(Token::CCurl),
            '[' => ret.push(Token::OSquare),
            ']' => ret.push(Token::CSquare),
            '&' if chars[i + 1] == '=' => {
                ret.push(Token::BAndEq);
                i += 1;
            }
            '&' if chars[i + 1] == '&' => {
                ret.push(Token::AndCmp);
                i += 1;
            }
            '&' => {
                ret.push(Token::BAnd);
            }
            '^' if chars[i + 1] == '=' => {
                ret.push(Token::BXorEq);
                i += 1;
            }
            '^' => {
                ret.push(Token::BXor);
            }
            '%' if chars[i + 1] == '=' => {
                ret.push(Token::ModEq);
                i += 1;
            }
            '%' => {
                ret.push(Token::Mod);
            }
            '!' if chars[i + 1] == '=' => {
                ret.push(Token::NeqCmp);
                i += 1;
            }
            '!' => {
                ret.push(Token::Neq);
            }
            '|' if chars[i + 1] == '=' => {
                ret.push(Token::BOrEq);
                i += 1;
            }
            '|' if chars[i + 1] == '|' => {
                ret.push(Token::OrCmp);
                i += 1;
            }
            '|' => {
                ret.push(Token::BOr);
            }
            '~' if chars[i + 1] == '=' => {
                ret.push(Token::BNotEq);
                i += 1;
            }
            '~' => {
                ret.push(Token::BNot);
            }
            '<' if chars[i + 1] == '=' => {
                ret.push(Token::LsEq);
                i += 1;
            }
            '<' if chars[i + 1] == '<' => {
                if chars[i + 2] == '=' {
                    ret.push(Token::BlsEq);
                } else {
                    ret.push(Token::Bls);
                }
            }
            '<' => {
                ret.push(Token::Ls);
            }
            '>' if chars[i + 1] == '=' => {
                ret.push(Token::GrEq);
                i += 1;
            }
            '>' if chars[i + 1] == '>' => {
                if chars[i + 2] == '=' {
                    ret.push(Token::BrsEq);
                } else {
                    ret.push(Token::Brs);
                }
            }
            '>' => {
                ret.push(Token::Gr);
            }
            '.' => {
                ret.push(Token::Dot);
            }
            ',' => {
                ret.push(Token::Comma);
            }
            ';' => {
                ret.push(Token::Semi);
            }
            ':' => {
                ret.push(Token::Colon);
            }
            '=' if chars[i + 1] == '=' => {
                ret.push(Token::EqCmp);
                i += 1;
            }
            '=' => {
                ret.push(Token::Eq);
            }
            '\n' => {
                line_tracker.new_line(ret.len());
            }
            '\'' => {
                if chars[i + 1].is_ascii() {
                    let val: usize = if chars[i + 1] == '\\' {
                        i += 1;
                        if chars[i + 1].is_ascii_digit() {
                            chars[i + 1].to_digit(10).expect("Invalid literal digit") as usize
                        } else {
                            match chars[i + 1] {
                                'n' => 10,
                                't' => 9,
                                _ => 0,
                            }
                        }
                    } else {
                        chars[i + 1] as usize
                    };
                    ret.push(Token::NumLiteral(val));
                    i += 2;
                }
            }
            ' ' => {}
            _ => {
                // NOTE if we'e here it's an identifier

                lex_id(&chars, &mut curr, &mut i);
                println!("curr: {}\npost {}", curr, chars[i]);

                if chars[i] == '.' {
                    i += 1;
                    let struct_id = curr.clone();
                    curr = String::new();

                    lex_id(&chars, &mut curr, &mut i);

                    let field_id = curr.clone();
                    ret.push(Token::StructFieldId {
                        struct_id,
                        field_id,
                    });
                } else {
                    ret.push(Token::Id(curr.clone()));
                }
                curr = String::from("");
                i -= 1; // NOTE Next, i += 1 happens
            }
        }
        i += 1;
    }
    Ok((ret, line_tracker))
}

fn lex_id(chars: &[char], curr: &mut String, i: &mut usize) {
    for c in chars.iter().skip(*i) {
        if !c.is_alphanumeric() && *c != '_' {
            break;
        }

        curr.push(*c);
    }
    *i += curr.len();
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    If,
    Struct,
    Break,
    For,
    While,
    Loop,
    Fn,
    Type(CType),
    // Assign(String),
    Star,
    // Var(String),
    NumLiteral(usize),
    StrLiteral(String),
    Add,
    AddO,
    // Mul,
    Sub,
    SubO,
    Div,
    Mod,
    ModEq,
    Eq,
    Id(String),
    StructFieldId { struct_id: String, field_id: String },
    EqCmp,
    NeqCmp,
    AndCmp,
    OrCmp,
    Neq,
    BOr,
    BAnd,
    BXor,
    BOrEq,
    BAndEq,
    BXorEq,
    BNot,
    BNotEq,
    SubEq,
    AddEq,
    DivEq,
    MulEq,
    LsEq,
    Ls,
    Gr,
    GrEq,
    Bls,
    Blsu,
    BlsEq,
    Brs,
    Brsu,
    BrsEq,
    OParen,
    CParen,
    OCurl,
    CCurl,
    OSquare,
    CSquare,
    Goto(String),
    Label(String),
    Asm,
    Dot,
    Comma,
    Semi,
    Colon,
    Arrow,
    Return,
    PutChar,
    Assert, // this might be to much for the lexer to do
            // FuncDeclare((String, Vec<String>, CType)), // function name, args, return type
            // FuncCall(String, Vec<String>), // function name, args
}

#[derive(Debug, Clone, PartialEq)]
pub enum CType {
    Void,
    Int,
    Char,
    Struct(String),
    Array(Box<CType>),
}

impl CType {
    pub fn to_rust_type(&self) -> String {
        match self {
            CType::Int => "i32".into(),
            CType::Char => "u8".into(),
            CType::Void => "()".into(),
            CType::Struct(id) => id.into(),
            CType::Array(sub_type) => format!("&[{}]", sub_type.to_rust_type()),
        }
    }
}
