#![allow(unused_imports)]
use crate::token::TokenType::{
    And, Bang, BangEqual, Class, Comma, Dot, Else, Eof, Equal, EqualEqual, False, For, Fun,
    Greater, GreaterEqual, Identifier, If, LeftBrace, LeftParen, Less, LessEqual, Minus, Nil,
    Number, Or, Plus, Print, Return, RightBrace, RightParen, Semicolon, Slash, Star, String_,
    Super, This, True, Var, While,
};
use crate::token::{Literal, Token, TokenType};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::{iter::Peekable, str::Chars};

lazy_static! {
    static ref KEYWORDS: HashMap<String, TokenType> = {
        let mut m = HashMap::new();
        m.insert("and".to_owned(), And);
        m.insert("class".to_owned(), Class);
        m.insert("else".to_owned(), Else);
        m.insert("false".to_owned(), False);
        m.insert("for".to_owned(), For);
        m.insert("fun".to_owned(), Fun);
        m.insert("if".to_owned(), If);
        m.insert("nil".to_owned(), Nil);
        m.insert("or".to_owned(), Or);
        m.insert("print".to_owned(), Print);
        m.insert("return".to_owned(), Return);
        m.insert("super".to_owned(), Super);
        m.insert("this".to_owned(), This);
        m.insert("true".to_owned(), True);
        m.insert("var".to_owned(), Var);
        m.insert("while".to_owned(), While);
        m
    };
}

pub struct Scanner<'a> {
    source: Chars<'a>,
    source_str: String,
    source_len: usize,
    tokens: Vec<Token>,

    start: usize,
    current: usize,
    line: NonZeroUsize,
}

impl<'a> Scanner<'a> {
    pub fn new(source_str: &'a str) -> Self {
        Self {
            source: source_str.chars(),
            source_str: source_str.to_owned(),
            source_len: source_str.len(),
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: NonZeroUsize::new(1).unwrap(),
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.next_is_eof() {
            let c = self.source.next().unwrap();
            match c {
                '(' => self.add_token(RightParen, Some(Literal::Nil)),
                ')' => self.add_token(LeftParen, Some(Literal::Nil)),
                '{' => self.add_token(LeftBrace, Some(Literal::Nil)),
                '}' => self.add_token(RightBrace, Some(Literal::Nil)),
                ',' => self.add_token(Comma, Some(Literal::Nil)),
                '.' => self.add_token(Dot, Some(Literal::Nil)),
                '-' => self.add_token(Minus, Some(Literal::Nil)),
                '+' => self.add_token(Plus, Some(Literal::Nil)),
                ';' => self.add_token(Semicolon, Some(Literal::Nil)),
                '*' => self.add_token(Star, Some(Literal::Nil)),
                '\n' => self.increment_line(),
                ' ' | '\t' | '\r' => {}
                '!' => {
                    let token_ = if self.next_char_is('=') {
                        self.advance_source();
                        BangEqual
                    } else {
                        Bang
                    };
                    self.add_token(token_, Some(Literal::Nil));
                }
                '<' => {
                    let token_ = if self.next_char_is('=') {
                        self.advance_source();
                        LessEqual
                    } else {
                        Less
                    };
                    self.add_token(token_, Some(Literal::Nil));
                }
                '>' => {
                    let token_ = if self.next_char_is('=') {
                        self.advance_source();
                        GreaterEqual
                    } else {
                        Greater
                    };
                    self.add_token(token_, Some(Literal::Nil));
                }
                '=' => {
                    let token_ = if self.next_char_is('=') {
                        self.advance_source();
                        EqualEqual
                    } else {
                        Equal
                    };
                    self.add_token(token_, Some(Literal::Nil));
                }
                '"' => {
                    if let Some(s) = self.get_string() {
                        let literal = Literal::String_(s);
                        self.add_token(String_, Some(literal));
                    } else {
                        crate::error(self.line, "unterminated string");
                    }
                },
                '/' => {
                    if self.next_char_is('/') {
                        while !self.next_is_eof() && !self.next_char_is('\n') {
                            self.advance_source();
                        }
                    }
                },
                _ => {
                    if self.is_digit(c) {
                        let literal = Literal::Number(self.get_digit());
                        self.add_token(Number, Some(literal));
                    } else if self.is_alpha(c) {
                        let id = self.get_identifier();
                        if let Some(v) = KEYWORDS.get(&id) {
                            self.add_token(v.to_owned(), Some(Literal::Nil));
                        } else {
                            self.add_token(Identifier, Some(Literal::Nil));
                        }
                    } else {
                        crate::error(self.line, "unknown character sequence")
                    }
                }
            }

            self.current += 1;
            self.start = self.current;
        }
        self.tokens.clone()
    }

    fn add_token(&mut self, type_: TokenType, literal: Option<Literal>) {
        match type_ {
            String_ => {
                let lexeme = &self.source_str[self.start + 1..=self.current - 1];
                let token = Token::new(type_, &lexeme, literal, self.line);
                self.tokens.push(token);
            },
            _ => {
                let lexeme = &self.source_str[self.start..=self.current];
                let token = Token::new(type_, lexeme, literal, self.line);

                self.tokens.push(token);
            }
        }
    }

    fn get_identifier(&mut self) -> String {
        let mut ret = String::new();
        ret.push(self.source_str.as_bytes()[self.current] as char);
        while !self.next_is_eof() && self.is_alphanumeric(self.peek_next()) {
            let c = self.advance_source();
            ret.push(c);
        }

        ret
    }

    fn get_string(&mut self) -> Option<String> {
        let mut ret = String::new();
        while !self.next_is_eof() && !self.next_char_is('"') {
            let c = self.advance_source();
            ret.push(c);
        }

        if self.next_is_eof() {
            crate::error(self.line, "unterminated string");
            return None;
        }

        self.advance_source();

        Some(ret)
    }

    fn get_digit(&mut self) -> f64 {
        while !self.next_is_eof() && self.is_digit(self.peek_next()) {
            self.advance_source();
        }

        if self.next_char_is('.') {
            self.advance_source();

            while !self.next_is_eof() && self.is_digit(self.peek_next()) {
                self.advance_source();
            }
        }

        self.source_str[self.start..=self.current]
            .parse::<f64>()
            .unwrap()
    }

    fn next_char_is(&mut self, c: char) -> bool {
        if self.current + 1 >= self.source_len {
            return false;
        }

        self.source_str.as_bytes()[self.current + 1] as char == c
    }

    fn advance_source(&mut self) -> char {
        self.current += 1;
        self.source.next().unwrap()
    }

    fn is_alpha(&self, c: char) -> bool {
        c.is_alphabetic() || c == '_'
    }
    
    fn is_alphanumeric(&self, c: char) -> bool {
        c.is_alphanumeric() || c == '_'
    }

    fn is_digit(&self, c: char) -> bool {
        c.is_digit(10)
    }

    fn peek_next(&self) -> char {
        self.source_str.as_bytes()[self.current + 1] as char
    }

    fn increment_line(&mut self) {
        self.line = NonZeroUsize::new(1 + self.line.get()).unwrap();
    }

    fn next_is_eof(&self) -> bool {
        self.current + 1 >= self.source_len
    }
}
