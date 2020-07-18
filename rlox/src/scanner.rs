use std::fmt;

use crate::error;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    Str,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone)]
pub enum Literal {
    Number(f64),
    Str(String),
    Bool(bool),
}

impl PartialEq for Literal {
    // TODO: There must be a better way to compare enums bearing payloads.
    fn eq(&self, other: &Self) -> bool {
        match self {
            Literal::Number(v) => {
                match other {
                    Literal::Number(v2) => (v - v2).abs() < 1e-10,
                    _ => false,
                }
            },
            Literal::Str(s) => {
                match other {
                    Literal::Str(s2) => s == s2,
                    _ => false,
                }
            }
            Literal::Bool(b) => {
                match other {
                    Literal::Bool(b2) => b == b2,
                    _ => false
                }
            }
        }
    }
}

impl Eq for Literal{}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    token_type: TokenType,
    lexeme: String,
    literal: Option<Literal>,
    line: i32,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, literal:Option<Literal>, line: i32) -> Token {
        Token {
            token_type: token_type,
            lexeme: lexeme,
            literal: literal,
            line: line,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(literal) = &self.literal {
            write!(f, "{} {} {}", self.token_type.to_string(), self.lexeme, literal.to_string())
        } else {
            write!(f, "{} {}", self.token_type.to_string(), self.lexeme)
        }
    }
}


pub struct Scanner<'a> {
    source: &'a str,
    current_grapheme: &'a str,
    remainder: &'a str,
    line: i32,
}

fn car_cdr(s: &str) -> (&str, &str) {
    match s.chars().next() {
        Some(c) => s.split_at(c.len_utf8()),
        None => s.split_at(0),
    }
}

impl Scanner<'_> {
    pub fn new<'a>(source: &'a str) -> Scanner {
        Scanner { source: source, current_grapheme: "", remainder: source, line: 0 }
    }

    // returns the next grapheme in the source string, or None if exhausted.
    fn next_grapheme(&mut self) -> Option<String> {
        let (grapheme, remainder) = car_cdr(self.remainder);
        self.current_grapheme = grapheme;
        self.remainder = remainder;
        if self.current_grapheme.len() > 0 {
            Some(String::from(self.current_grapheme))
        } else {
            None
        }
    }

    // if the next unread grapheme in the source string == query, return true
    // and advance, consuming that grapheme. Else leave state alone and return false.
    fn match_next_grapheme(&mut self, query: &str) -> bool {
        for i in 1..5 {
            let r = self.remainder.get(0..i);
            match r {
                Some(x) => {
                    if x == query {
                        self.advance();
                        return true;
                    }
                }
                None => continue,
            }
        }
        false
    }

    // COnsumes next grapheme in source.
    fn advance(&mut self) {
        let (fc, r) = car_cdr(self.remainder);
        self.current_grapheme = fc;
        self.remainder = r;
    }

    // Returns next unread grapheme, or EOF
    fn peek(&self) -> &str {
        for i in 1..5 {
            let r = self.remainder.get(0..i);
            match r {
                Some(x) => {
                    return x;
                }
                None => continue,
            }
        }
        "\0"
    }

    // Returns true if at end of source.
    fn is_at_end(&self) -> bool {
        return self.remainder.len() == 0;
    }

    fn string(&mut self, tokens:&mut Vec<Token>) {
        let mut string_value = String::new();
        while self.peek() != "\"" && !self.is_at_end() {
            if self.peek() == "\n" {
                self.line += 1;
            }
            string_value.push_str(self.peek());
            self.advance();
        }

        if self.is_at_end() {
            error::error(self.line, "Unterminated string");
            return;
        }

        // consume the closing "
        self.advance();

        tokens.push(Token::new(TokenType::Str,
            String::from(string_value.as_str()),
            Some(Literal::Str(string_value)),
            self.line));
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        let mut tokens:Vec<Token> = vec![];

        loop {
            if let Some(g) = self.next_grapheme() {
                match g.as_str() {
                    "(" => tokens.push(Token::new(TokenType::LeftParen, g, None, self.line)),
                    ")" => tokens.push(Token::new(TokenType::RightParen, g, None, self.line)),
                    "{" => tokens.push(Token::new(TokenType::LeftBrace, g, None, self.line)),
                    "}" => tokens.push(Token::new(TokenType::RightBrace, g, None, self.line)),
                    "," => tokens.push(Token::new(TokenType::Comma, g, None, self.line)),
                    "." => tokens.push(Token::new(TokenType::Dot, g, None, self.line)),
                    "-" => tokens.push(Token::new(TokenType::Minus, g, None, self.line)),
                    "+" => tokens.push(Token::new(TokenType::Plus, g, None, self.line)),
                    ";" => tokens.push(Token::new(TokenType::Semicolon, g, None, self.line)),
                    "*" => tokens.push(Token::new(TokenType::Star, g, None, self.line)),

                    "!" => {
                        if self.match_next_grapheme("=") {
                            tokens.push(Token::new(TokenType::BangEqual, g, None, self.line));
                        } else {
                            tokens.push(Token::new(TokenType::Bang, g, None, self.line));
                        }
                    },

                    "=" => {
                        if self.match_next_grapheme("=") {
                            tokens.push(Token::new(TokenType::EqualEqual, g, None, self.line));
                        } else {
                            tokens.push(Token::new(TokenType::Equal, g, None, self.line));
                        }
                    },

                    "<" => {
                        if self.match_next_grapheme("=") {
                            tokens.push(Token::new(TokenType::LessEqual, g, None, self.line));
                        } else {
                            tokens.push(Token::new(TokenType::Less, g, None, self.line));
                        }
                    },

                    ">" => {
                        if self.match_next_grapheme("=") {
                            tokens.push(Token::new(TokenType::GreaterEqual, g, None, self.line));
                        } else {
                            tokens.push(Token::new(TokenType::Greater, g, None, self.line));
                        }
                    },

                    "/" => {
                        if self.match_next_grapheme("/") {
                            // Comments go to the end of the line.
                            while self.peek() != "\n" && !self.is_at_end() {
                                self.advance();
                            }
                        } else {
                            tokens.push(Token::new(TokenType::Slash, g, None, self.line));
                        }
                    }

                    // Ignore whitespace
                    " " | "\r" | "\t" => (),

                    // Advance current self.line
                    "\n" => self.line += 1,

                    "\"" => self.string(&mut tokens),

                    _ => panic!("Unexpected character: \"{}\"", g),
                }
            } else {
                break;
            }
        }

        tokens
    }
}

#[cfg(test)]
mod scanner_tests {
    use super::*;

    #[test]
    fn next_grapheme_advances() {
        let mut scanner = Scanner::new("test");
        assert_eq!(scanner.next_grapheme(), Some(String::from("t")));
        assert_eq!(scanner.next_grapheme(), Some(String::from("e")));
        assert_eq!(scanner.next_grapheme(), Some(String::from("s")));
        assert_eq!(scanner.next_grapheme(), Some(String::from("t")));
        assert_eq!(scanner.next_grapheme(), None);
    }

    #[test]
    fn advance_advances() {
        let mut scanner = Scanner::new("test");
        assert_eq!(scanner.peek(), "t");

        scanner.advance();
        assert_eq!(scanner.peek(), "e");

        scanner.advance();
        assert_eq!(scanner.peek(), "s");

        scanner.advance();
        assert_eq!(scanner.peek(), "t");
    }

    #[test]
    fn match_next_grapheme_advances_on_match() {
        let mut scanner = Scanner::new("test");

        assert_eq!(scanner.peek(), "t");
        assert!(scanner.match_next_grapheme("t"));

        assert_eq!(scanner.peek(), "e");
        assert!(scanner.match_next_grapheme("e"));

        assert_eq!(scanner.peek(), "s");
        assert!(scanner.match_next_grapheme("s"));

        assert_eq!(scanner.peek(), "t");
        assert!(scanner.match_next_grapheme("t"));
    }

    #[test]
    fn match_next_grapheme_does_nothing_without_match() {
        let mut scanner = Scanner::new("test");

        assert_eq!(scanner.peek(), "t");
        assert!(!scanner.match_next_grapheme("g"));

        assert_eq!(scanner.peek(), "t");
    }

    #[test]
    fn produces_expected_tokens() {
        let mut scanner = Scanner::new("( ) { } < > <= >= = == ! !=");
        let tokens = scanner.scan_tokens();
        let expected_token_types = vec![
            TokenType::LeftParen, TokenType::RightParen,
            TokenType::LeftBrace, TokenType::RightBrace,
            TokenType::Less, TokenType::Greater,
            TokenType::LessEqual, TokenType::GreaterEqual,
            TokenType::Equal, TokenType::EqualEqual,
            TokenType::Bang, TokenType::BangEqual,
        ];
        let token_types:Vec<TokenType> = tokens.into_iter().map(|t| t.token_type ).collect();
        assert_eq!(token_types, expected_token_types);
    }

    #[test]
    fn produces_expected_string_tokens() {
        let expected_val = "foo bar baz";
        let source = format!("(\"{}\")", expected_val);
        let mut scanner = Scanner::new(&source);
        let tokens = scanner.scan_tokens();

        assert_eq!(tokens[0].token_type, TokenType::LeftParen);
        assert_eq!(tokens[1].token_type, TokenType::Str);
        assert_eq!(tokens[2].token_type, TokenType::RightParen);

        if let Some(literal) = &tokens[1].literal {
            let parsed_val = match literal {
                Literal::Str(string_val) => string_val,
                _ => "",
            };

            assert_eq!(parsed_val, expected_val);
        }
    }
}