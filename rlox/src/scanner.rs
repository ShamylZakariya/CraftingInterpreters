use std::collections::HashMap;
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
    QuestionMark,
    Colon,

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
    Break,
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

#[derive(PartialEq, Debug, Clone)]
pub enum Literal {
    Number(f64),
    Str(String),
    False,
    True,
    Nil,
}

impl Eq for Literal {}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Literal::Number(n) => write!(f, "{}", n),
            Literal::Str(s) => write!(f, "{}", s),
            Literal::False => write!(f, "False"),
            Literal::True => write!(f, "True"),
            Literal::Nil => write!(f, "Nil"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Option<Literal>,
    pub line: i32,
}

impl Token {
    pub fn new(
        token_type: TokenType,
        lexeme: String,
        literal: Option<Literal>,
        line: i32,
    ) -> Token {
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
            write!(
                f,
                "{} lexeme: \"{}\" literal: {} line: {}",
                self.token_type.to_string(),
                self.lexeme,
                literal.to_string(),
                self.line
            )
        } else {
            write!(
                f,
                "{} lexeme: \"{}\" line: {}",
                self.token_type.to_string(),
                self.lexeme,
                self.line
            )
        }
    }
}

pub struct Scanner<'a> {
    _source: &'a str,
    current_grapheme: &'a str,
    remainder: &'a str,
    line: i32,
    keywords: HashMap<String, TokenType>,
}

fn car_cdr(s: &str) -> (&str, &str) {
    match s.chars().next() {
        Some(c) => s.split_at(c.len_utf8()),
        None => s.split_at(0),
    }
}

// Returns true if the grapheme is a digit.
fn is_digit(grapheme: &str) -> bool {
    let first_char = grapheme.chars().next().unwrap();
    match first_char {
        '0'..='9' => true,
        _ => false,
    }
}

// Returns true if the grapheme is a letter (a-z, A-Z)
fn is_alpha(grapheme: &str) -> bool {
    // TODO: Make this work with non-ascii text.
    let first_char = grapheme.chars().next().unwrap();
    match first_char {
        'a'..='z' | 'A'..='Z' | '_' => true,
        _ => false,
    }
}

// Returns true if the grapheme is_alpha and is_digit.
fn is_alpha_numeric(grapheme: &str) -> bool {
    is_alpha(grapheme) || is_digit(grapheme)
}

impl Scanner<'_> {
    pub fn new<'a>(source: &'a str) -> Scanner {
        Scanner {
            _source: source,
            current_grapheme: "",
            remainder: source,
            line: 1,
            keywords: Scanner::create_keywords(),
        }
    }

    fn create_keywords() -> HashMap<String, TokenType> {
        let mut keywords = HashMap::new();
        keywords.insert(String::from("and"), TokenType::And);
        keywords.insert(String::from("break"), TokenType::Break);
        keywords.insert(String::from("class"), TokenType::Class);
        keywords.insert(String::from("else"), TokenType::Else);
        keywords.insert(String::from("false"), TokenType::False);
        keywords.insert(String::from("for"), TokenType::For);
        keywords.insert(String::from("fun"), TokenType::Fun);
        keywords.insert(String::from("if"), TokenType::If);
        keywords.insert(String::from("nil"), TokenType::Nil);
        keywords.insert(String::from("or"), TokenType::Or);
        keywords.insert(String::from("print"), TokenType::Print);
        keywords.insert(String::from("return"), TokenType::Return);
        keywords.insert(String::from("super"), TokenType::Super);
        keywords.insert(String::from("this"), TokenType::This);
        keywords.insert(String::from("true"), TokenType::True);
        keywords.insert(String::from("var"), TokenType::Var);
        keywords.insert(String::from("while"), TokenType::While);
        keywords
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

    // Returns next+1 unread grapheme, or EOF
    fn peek_next(&self) -> &str {
        for i in 1..5 {
            let r = self.remainder.get(1..i + 1);
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

    // Consumes a string and writes it into tokens.
    fn string(&mut self, tokens: &mut Vec<Token>) {
        let mut string_value = String::new();
        while self.peek() != "\"" && !self.is_at_end() {
            if self.peek() == "\n" {
                self.line += 1;
            }
            string_value.push_str(self.peek());
            self.advance();
        }

        if self.is_at_end() {
            error::report::parse_error_at_line(self.line, "Unterminated string");
            return;
        }

        // consume the closing "
        self.advance();

        tokens.push(Token::new(
            TokenType::Str,
            String::from(string_value.as_str()),
            Some(Literal::Str(string_value)),
            self.line,
        ));
    }

    // Consumes a number, and writes it into tokens.
    fn number(&mut self, current_grapheme: &str, tokens: &mut Vec<Token>) {
        let mut string_value = String::new();
        string_value.push_str(current_grapheme);

        while is_digit(self.peek()) {
            string_value.push_str(self.peek());
            self.advance();
        }

        if self.peek() == "." && is_digit(self.peek_next()) {
            string_value.push('.');
            // consume.the "."
            self.advance();

            // add the fractional component
            while is_digit(self.peek()) {
                string_value.push_str(self.peek());
                self.advance();
            }
        }

        // now parse to double
        let d = string_value.parse::<f64>();
        if let Ok(v) = d {
            tokens.push(Token::new(
                TokenType::Number,
                String::from(string_value.as_str()),
                Some(Literal::Number(v)),
                self.line,
            ));
        } else {
            let error_message = format!("Unable to parse number literal \"{}\"", string_value);
            error::report::parse_error_at_line(self.line, &error_message);
        }
    }

    // Consumes an identifier and writes it into tokens.
    fn identifier(&mut self, current_grapheme: &str, tokens: &mut Vec<Token>) {
        let mut string_value = String::new();
        string_value.push_str(current_grapheme);

        while is_alpha_numeric(self.peek()) {
            string_value.push_str(self.peek());
            self.advance();
        }

        let identifier = String::from(string_value.as_str());
        let identifier_type = self.keywords.get(&identifier);
        match identifier_type {
            Some(token_type) => {
                tokens.push(Token::new(*token_type, identifier, None, self.line));
            }
            None => {
                tokens.push(Token::new(
                    TokenType::Identifier,
                    identifier,
                    None,
                    self.line,
                ));
            }
        };
    }

    // Scans the source string provided at construction and returns a vector of Token.
    pub fn scan_tokens(&mut self) -> Vec<Token> {
        let mut tokens: Vec<Token> = vec![];

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
                    "?" => tokens.push(Token::new(TokenType::QuestionMark, g, None, self.line)),
                    ":" => tokens.push(Token::new(TokenType::Colon, g, None, self.line)),

                    "!" => {
                        if self.match_next_grapheme("=") {
                            tokens.push(Token::new(
                                TokenType::BangEqual,
                                "!=".to_string(),
                                None,
                                self.line,
                            ));
                        } else {
                            tokens.push(Token::new(TokenType::Bang, g, None, self.line));
                        }
                    }

                    "=" => {
                        if self.match_next_grapheme("=") {
                            tokens.push(Token::new(
                                TokenType::EqualEqual,
                                "==".to_string(),
                                None,
                                self.line,
                            ));
                        } else {
                            tokens.push(Token::new(TokenType::Equal, g, None, self.line));
                        }
                    }

                    "<" => {
                        if self.match_next_grapheme("=") {
                            tokens.push(Token::new(
                                TokenType::LessEqual,
                                "<=".to_string(),
                                None,
                                self.line,
                            ));
                        } else {
                            tokens.push(Token::new(TokenType::Less, g, None, self.line));
                        }
                    }

                    ">" => {
                        if self.match_next_grapheme("=") {
                            tokens.push(Token::new(
                                TokenType::GreaterEqual,
                                ">=".to_string(),
                                None,
                                self.line,
                            ));
                        } else {
                            tokens.push(Token::new(TokenType::Greater, g, None, self.line));
                        }
                    }

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

                    _ => {
                        if is_digit(&g) {
                            self.number(&g, &mut tokens);
                        } else if is_alpha(&g) {
                            self.identifier(&g, &mut tokens);
                        } else {
                            panic!("Unexpected character: \"{}\"", g);
                        }
                    }
                }
            } else {
                break;
            }
        }

        tokens.push(Token::new(TokenType::Eof, String::new(), None, self.line));
        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn peek_and_peek_next_work() {
        let mut scanner = Scanner::new("test");
        assert_eq!(scanner.peek(), "t");
        assert_eq!(scanner.peek_next(), "e");

        scanner.advance();
        assert_eq!(scanner.peek(), "e");
        assert_eq!(scanner.peek_next(), "s");

        scanner.advance();
        assert_eq!(scanner.peek(), "s");
        assert_eq!(scanner.peek_next(), "t");

        scanner.advance();
        assert_eq!(scanner.peek(), "t");
        assert_eq!(scanner.peek_next(), "\0");

        scanner.advance();
        assert_eq!(scanner.peek(), "\0");
        assert_eq!(scanner.peek_next(), "\0");
    }

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
            TokenType::LeftParen,
            TokenType::RightParen,
            TokenType::LeftBrace,
            TokenType::RightBrace,
            TokenType::Less,
            TokenType::Greater,
            TokenType::LessEqual,
            TokenType::GreaterEqual,
            TokenType::Equal,
            TokenType::EqualEqual,
            TokenType::Bang,
            TokenType::BangEqual,
            TokenType::Eof,
        ];
        let token_types: Vec<TokenType> = tokens.into_iter().map(|t| t.token_type).collect();
        assert_eq!(token_types, expected_token_types);
    }

    #[test]
    fn produces_expected_string_literals() {
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

    #[test]
    fn produces_expected_int_literal() {
        let int_val = 12345;
        let source = format!("({})", int_val);
        let mut scanner = Scanner::new(&source);
        let tokens = scanner.scan_tokens();

        assert_eq!(tokens[0].token_type, TokenType::LeftParen);
        assert_eq!(tokens[1].token_type, TokenType::Number);
        assert_eq!(tokens[2].token_type, TokenType::RightParen);

        if let Some(literal) = &tokens[1].literal {
            let parsed_val = match literal {
                Literal::Number(numeric_val) => numeric_val,
                _ => &f64::NAN,
            };

            assert_eq!((int_val as f64), *parsed_val);
        }
    }

    #[test]
    fn produces_expected_f64_literal() {
        let double_val = 12345.6789;
        let source = format!("({})", double_val);
        let mut scanner = Scanner::new(&source);
        let tokens = scanner.scan_tokens();

        assert_eq!(tokens[0].token_type, TokenType::LeftParen);
        assert_eq!(tokens[1].token_type, TokenType::Number);
        assert_eq!(tokens[2].token_type, TokenType::RightParen);

        if let Some(literal) = &tokens[1].literal {
            let parsed_val = match literal {
                Literal::Number(numeric_val) => numeric_val,
                _ => &f64::NAN,
            };

            assert_eq!((double_val as f64), *parsed_val);
        }
    }

    #[test]
    fn produces_expected_keywords() {
        let mut scanner = Scanner::new(
            "and class else false for fun if nil or print return super this true var while",
        );
        let tokens = scanner.scan_tokens();
        let token_types: Vec<TokenType> = tokens.into_iter().map(|t| t.token_type).collect();
        assert_eq!(
            token_types,
            vec![
                TokenType::And,
                TokenType::Class,
                TokenType::Else,
                TokenType::False,
                TokenType::For,
                TokenType::Fun,
                TokenType::If,
                TokenType::Nil,
                TokenType::Or,
                TokenType::Print,
                TokenType::Return,
                TokenType::Super,
                TokenType::This,
                TokenType::True,
                TokenType::Var,
                TokenType::While,
                TokenType::Eof,
            ]
        );
    }

    #[test]
    fn produces_expected_identifiers() {
        let mut scanner = Scanner::new("{foo bar baz}");
        let tokens = scanner.scan_tokens();

        assert_eq!(tokens[0].token_type, TokenType::LeftBrace);
        assert_eq!(tokens[1].token_type, TokenType::Identifier);
        assert_eq!(tokens[2].token_type, TokenType::Identifier);
        assert_eq!(tokens[3].token_type, TokenType::Identifier);
        assert_eq!(tokens[4].token_type, TokenType::RightBrace);

        assert_eq!(tokens[1].lexeme, String::from("foo"));
        assert_eq!(tokens[2].lexeme, String::from("bar"));
        assert_eq!(tokens[3].lexeme, String::from("baz"));
    }
}
