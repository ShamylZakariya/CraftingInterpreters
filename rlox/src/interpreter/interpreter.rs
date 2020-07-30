use std::fmt;

use crate::error;
use crate::parser::expr::*;
use crate::parser::parser::*;
use crate::parser::scanner::{Literal, Scanner, Token, TokenType};

#[derive(PartialEq, Debug, Clone)]
pub enum LoxObject {
    Number(f64),
    Str(String),
    Boolean(bool),
    Nil,
}
impl LoxObject {
    fn from_literal(literal: &crate::parser::scanner::Literal) -> Self {
        match literal {
            Literal::Number(n) => LoxObject::Number(*n),
            Literal::Str(s) => LoxObject::Str(s.clone()),
            Literal::False => LoxObject::Boolean(false),
            Literal::True => LoxObject::Boolean(true),
            Literal::Nil => LoxObject::Nil,
        }
    }
    fn is_truthy(&self) -> bool {
        match self {
            LoxObject::Nil => false,
            LoxObject::Boolean(b) => *b,
            _ => false,
        }
    }
}

impl Eq for LoxObject {}

impl fmt::Display for LoxObject {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LoxObject::Number(n) => {
                if *n < 0.0 {
                    if n.ceil() - n < 1e-8 {
                        write!(f, "{}", *n as i32)
                    } else {
                        write!(f, "{}", n)
                    }
                } else {
                    if n - n.floor() < 1e-8 {
                        write!(f, "{}", *n as i32)
                    } else {
                        write!(f, "{}", n)
                    }
                }
            },
            LoxObject::Str(s) => write!(f, "{}", s),
            LoxObject::Boolean(v) => {
                if *v {
                    write!(f, "true")
                } else {
                    write!(f, "false")
                }
            }
            LoxObject::Nil => write!(f, "nil"),
        }
    }
}

//-------------

pub type Result<T> = std::result::Result<T, RuntimeError>;

#[derive(Debug, Clone)]
pub struct RuntimeError {
    pub token: Token,
    pub message: String,
}

impl RuntimeError {
    fn new(token: &Token, message: &str) -> Self {
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

//-------------

pub struct Interpreter;
impl Visitor<Result<LoxObject>> for Interpreter {
    fn visit_binary_expr(
        &self,
        left: &Box<Expr>,
        operator: &Token,
        right: &Box<Expr>,
    ) -> Result<LoxObject> {
        let left = _evaluate(self, left)?;
        let right = _evaluate(self, right)?;
        match operator.token_type {
            TokenType::Minus => {
                if let LoxObject::Number(l) = left {
                    if let LoxObject::Number(r) = right {
                        Ok(LoxObject::Number(l - r))
                    } else {
                        Err(RuntimeError::new(operator, "Right operand not a number"))
                    }
                } else {
                    Err(RuntimeError::new(operator, "Left operand not a number"))
                }
            }
            TokenType::Slash => {
                if let LoxObject::Number(l) = left {
                    if let LoxObject::Number(r) = right {
                        Ok(LoxObject::Number(l / r))
                    } else {
                        Err(RuntimeError::new(operator, "Right operand not a number"))
                    }
                } else {
                    Err(RuntimeError::new(operator, "Left operand not a number"))
                }
            }
            TokenType::Star => {
                if let LoxObject::Number(l) = left {
                    if let LoxObject::Number(r) = right {
                        Ok(LoxObject::Number(l * r))
                    } else {
                        Err(RuntimeError::new(operator, "Right operand not a number"))
                    }
                } else {
                    Err(RuntimeError::new(operator, "Left operand not a number"))
                }
            }
            TokenType::Plus => {
                if let LoxObject::Number(l) = left {
                    if let LoxObject::Number(r) = right {
                        Ok(LoxObject::Number(l + r))
                    } else {
                        Err(RuntimeError::new(operator, "Right operand not a number"))
                    }
                } else if let LoxObject::Str(l) = left {
                    if let LoxObject::Str(r) = right {
                        Ok(LoxObject::Str(format!("{}{}", l, r)))
                    } else {
                        Err(RuntimeError::new(operator, "Right operand not a string"))
                    }
                } else {
                    Err(RuntimeError::new(
                        operator,
                        "Left operand not a number or string",
                    ))
                }
            }

            _ => Err(RuntimeError::new(operator, "Unrecognized binary operator.")),
        }
    }

    fn visit_grouping_expr(&self, expr: &Box<Expr>) -> Result<LoxObject> {
        _evaluate(self, expr)
    }

    fn visit_literal_expr(&self, literal: &crate::parser::scanner::Literal) -> Result<LoxObject> {
        Ok(LoxObject::from_literal(literal))
    }

    fn visit_unary_expr(&self, operator: &Token, right: &Box<Expr>) -> Result<LoxObject> {
        let right = _evaluate(self, right)?;
        match operator.token_type {
            TokenType::Bang => Ok(LoxObject::Boolean(!right.is_truthy())),
            TokenType::Minus => match right {
                LoxObject::Number(n) => Ok(LoxObject::Number(-n)),
                _ => Err(RuntimeError::new(
                    operator,
                    "Unary negative can only be applied to numbers.",
                )),
            },
            _ => Err(RuntimeError::new(operator, "Unsupported unary operator.")),
        }
    }
}

pub fn _evaluate(interpreter: &Interpreter, expr: &Box<Expr>) -> Result<LoxObject> {
    accept(expr, interpreter)
}

pub fn interpret(expr: &Box<Expr>) -> Result<LoxObject> {
    let interpreter = Interpreter;
    match accept(expr, &interpreter) {
        Ok(result) => Ok(result),
        Err(e) => {
            error::runtime_error(&e);
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluate_works() {
        let inputs = vec![
            ("1+2*3", LoxObject::Number(7.0)),
            ("(1+2)*3", LoxObject::Number(9.0)),
            ("1+(2*3)", LoxObject::Number(7.0)),
            ("(3*4)/2", LoxObject::Number(6.0)),
            ("3*4/2", LoxObject::Number(6.0)),
        ];
        for (expression, expected_result) in inputs {
            let mut scanner = Scanner::new(expression);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);
            let expr = parser.parse().unwrap();
            let result = interpret(&expr).unwrap();
            assert_eq!(result, expected_result);
        }
    }
}
