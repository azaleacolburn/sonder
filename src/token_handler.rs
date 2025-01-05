use crate::error::{ErrType as ET, RhErr};
use crate::lexer::{LineNumHandler, Token};
use std::ops::Range;

pub struct TokenHandler {
    pub tokens: Vec<Token>,
    pub curr_token: usize,
    // Index is line number, range is start and end of line
    pub token_lines: Vec<Range<usize>>,
}

#[allow(dead_code)]
impl TokenHandler {
    pub fn new(tokens: Vec<Token>, line_tracker: LineNumHandler) -> Self {
        TokenHandler {
            tokens: tokens.clone(),
            curr_token: 0,
            token_lines: line_tracker
                .token_lines
                .iter()
                .enumerate()
                .map(|(line_number, start)| {
                    let end = match line_tracker.token_lines.len() - 1 == line_number {
                        true => tokens.len(),
                        false => line_tracker.token_lines[line_number + 1],
                    };

                    Range {
                        start: *start - 1,
                        end: end - 1,
                    }
                })
                .collect(),
        }
    }

    pub fn next_token(&mut self) {
        self.curr_token += 1;
    }

    pub fn peek(&self, i: usize) -> &Token {
        &self.tokens[self.curr_token + i]
    }

    pub fn prev_token(&mut self) {
        self.curr_token -= 1;
    }

    pub fn get_token(&self) -> &Token {
        &self.tokens[self.curr_token]
    }

    pub fn get_prev_token(&self) -> &Token {
        &self.tokens[self.curr_token - 1]
    }

    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    pub fn new_err(&self, err: ET) -> RhErr {
        println!("{:?}", err);
        RhErr {
            err,
            line: self.line(),
        }
    }

    pub fn line(&self) -> usize {
        println!("{:?}", self.tokens[self.curr_token]);
        println!("{:?}", self.token_lines);
        self.token_lines
            .iter()
            .position(|range| range.start <= self.curr_token && self.curr_token <= range.end)
            .expect("Token outside of line number range")
            + 1
    }
}
