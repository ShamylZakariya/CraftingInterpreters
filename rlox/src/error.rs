use std::fmt;

use crate::scanner::Token;

// --------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ParseError {
    pub token: Token,
    pub message: String,
}

impl ParseError {
    pub fn new(token: Token, message: &str) -> Self {
        Self {
            token: token,
            message: message.to_owned(),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "token: {} error:\"{}\"", self.token, self.message)
    }
}

// --------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct RuntimeError {
    pub token: Token,
    pub message: String,
}

impl RuntimeError {
    pub fn new(token: &Token, message: &str) -> Self {
        Self {
            token: token.to_owned(),
            message: message.to_owned(),
        }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "token: {} error:\"{}\"", self.token, self.message)
    }
}

// --------------------------------------------------------------------------------------------------------------------

pub mod report {
    use super::RuntimeError;
    use crate::scanner::{Token, TokenType};

    pub fn error(line: i32, context: &str, message: &str) {
        eprintln!("[line {}] Error {}: {}", line, context, message);
    }

    pub fn parse_error_at_line(line: i32, message: &str) {
        error(line, "", message);
    }

    pub fn parse_error_at_token(token: &Token, message: &str) {
        match token.token_type {
            TokenType::Eof => error(token.line, " at end", message),
            _ => error(
                token.line,
                format!(" at '{}'", token.lexeme).as_str(),
                message,
            ),
        }
    }

    pub fn runtime_error(e: &RuntimeError) {
        eprintln!("{}\n[line {}]", e.message, e.token.line);
    }
}
