use std::fmt;

#[derive(Debug, PartialEq, Eq)]
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

#[derive(Debug)]
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

#[derive(Debug, PartialEq, Eq)]
pub struct Token {
    token_type: TokenType,
    lexeme: String,
    literal: Option<Literal>,
    line: i32,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: &str, literal:Option<Literal>, line: i32) -> Token {
        Token {
            token_type: token_type,
            lexeme: String::from(lexeme),
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
}

fn car_cdr(s: &str) -> (&str, &str) {
    match s.chars().next() {
        Some(c) => s.split_at(c.len_utf8()),
        None => s.split_at(0),
    }
}

impl Scanner<'_> {
    pub fn new<'a>(source: &'a str) -> Scanner {
        Scanner { source: source, current_grapheme: "", remainder: source }
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

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        let mut tokens:Vec<Token> = vec![];
        let mut line:i32 = 0;

        loop {
            if let Some(g) = self.next_grapheme() {
                match g.as_str() {
                    "(" => tokens.push(Token::new(TokenType::LeftParen, g.as_str(), None, line)),
                    ")" => tokens.push(Token::new(TokenType::RightParen, g.as_str(), None, line)),
                    "{" => tokens.push(Token::new(TokenType::LeftBrace, g.as_str(), None, line)),
                    "}" => tokens.push(Token::new(TokenType::RightBrace, g.as_str(), None, line)),
                    "," => tokens.push(Token::new(TokenType::Comma, g.as_str(), None, line)),
                    "." => tokens.push(Token::new(TokenType::Dot, g.as_str(), None, line)),
                    "-" => tokens.push(Token::new(TokenType::Minus, g.as_str(), None, line)),
                    "+" => tokens.push(Token::new(TokenType::Plus, g.as_str(), None, line)),
                    ";" => tokens.push(Token::new(TokenType::Semicolon, g.as_str(), None, line)),
                    "*" => tokens.push(Token::new(TokenType::Star, g.as_str(), None, line)),

                    "!" => {
                        if self.match_next_grapheme("=") {
                            tokens.push(Token::new(TokenType::BangEqual, g.as_str(), None, line));
                        } else {
                            tokens.push(Token::new(TokenType::Bang, g.as_str(), None, line));
                        }
                    },

                    "=" => {
                        if self.match_next_grapheme("=") {
                            tokens.push(Token::new(TokenType::EqualEqual, g.as_str(), None, line));
                        } else {
                            tokens.push(Token::new(TokenType::Equal, g.as_str(), None, line));
                        }
                    },

                    "<" => {
                        if self.match_next_grapheme("=") {
                            tokens.push(Token::new(TokenType::LessEqual, g.as_str(), None, line));
                        } else {
                            tokens.push(Token::new(TokenType::Less, g.as_str(), None, line));
                        }
                    },

                    ">" => {
                        if self.match_next_grapheme("=") {
                            tokens.push(Token::new(TokenType::GreaterEqual, g.as_str(), None, line));
                        } else {
                            tokens.push(Token::new(TokenType::Greater, g.as_str(), None, line));
                        }
                    },

                    "/" => {
                        if self.match_next_grapheme("/") {
                            // Comments go to the end of the line.
                            while self.peek() != "\n" && !self.is_at_end() {
                                self.advance();
                            }
                        } else {
                            tokens.push(Token::new(TokenType::Slash, g.as_str(), None, line));
                        }
                    }

                    // Ignore whitespace
                    " " | "\r" | "\t" => (),

                    // Advance current line
                    "\n" => line += 1,


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
}
