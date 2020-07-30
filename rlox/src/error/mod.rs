use crate::parser::scanner::{Token, TokenType};
use crate::interpreter::interpreter::RuntimeError;

pub fn report(line: i32, context: &str, message: &str) {
    eprintln!("[line {}] Error {}: {}", line, context, message);
}

pub fn parse_error_at_line(line: i32, message: &str) {
    report(line, "", message);
}

pub fn parse_error_at_token(token: &Token, message: &str) {
    match token.token_type {
        TokenType::Eof => report(token.line, " at end", message),
        _ => report(
            token.line,
            format!(" at '{}'", token.lexeme).as_str(),
            message,
        ),
    }
}

pub fn runtime_error(e:&RuntimeError) {
    eprintln!("{}\n[line {}]", e.message, e.token.line);
}
