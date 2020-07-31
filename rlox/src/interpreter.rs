use std::fmt;

use crate::error;
use crate::expr::*;
use crate::scanner::{Literal, Token, TokenType};

#[derive(PartialEq, Debug, Clone)]
pub enum LoxObject {
    Number(f64),
    Str(String),
    Boolean(bool),
    Nil,
}
impl LoxObject {
    fn from_literal(literal: &crate::scanner::Literal) -> Self {
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
            LoxObject::Number(n) => write!(f, "{}", n),
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

use error::RuntimeError as RuntimeError;
pub type Result<T> = std::result::Result<T, RuntimeError>;

pub struct Interpreter;
impl Interpreter {
    pub fn interpret(&self, expr: &Box<Expr>) -> Result<LoxObject> {
        match expr.accept(self) {
            Ok(result) => Ok(result),
            Err(e) => {
                error::report::runtime_error(&e);
                Err(e)
            }
        }
    }
}

impl Visitor<Result<LoxObject>> for Interpreter {
    fn visit_binary_expr(
        &self,
        left: &Box<Expr>,
        operator: &Token,
        right: &Box<Expr>,
    ) -> Result<LoxObject> {
        let left = self.interpret(left)?;
        let right = self.interpret(right)?;
        match operator.token_type {
            TokenType::Minus => {
                if let LoxObject::Number(l) = left {
                    if let LoxObject::Number(r) = right {
                        Ok(LoxObject::Number(l - r))
                    } else {
                        Err(RuntimeError::new(operator, "Right operand not a number."))
                    }
                } else {
                    Err(RuntimeError::new(operator, "Left operand not a numbe."))
                }
            }
            TokenType::Slash => {
                if let LoxObject::Number(l) = left {
                    if let LoxObject::Number(r) = right {
                        if r.abs() > 1e-8 {
                            Ok(LoxObject::Number(l / r))
                        } else {
                            Err(RuntimeError::new(operator, "Attempt to divide by zero."))
                        }
                    } else {
                        Err(RuntimeError::new(operator, "Right operand not a number."))
                    }
                } else {
                    Err(RuntimeError::new(operator, "Left operand not a number."))
                }
            }
            TokenType::Star => {
                if let LoxObject::Number(l) = left {
                    if let LoxObject::Number(r) = right {
                        Ok(LoxObject::Number(l * r))
                    } else {
                        Err(RuntimeError::new(operator, "Right operand not a number."))
                    }
                } else {
                    Err(RuntimeError::new(operator, "Left operand not a number."))
                }
            }
            TokenType::Plus => {
                if let LoxObject::Number(l) = left {
                    if let LoxObject::Number(r) = right {
                        Ok(LoxObject::Number(l + r))
                    } else {
                        Err(RuntimeError::new(operator, "Right operand not a number."))
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
                        "Left operand not a number or string.",
                    ))
                }
            }

            _ => Err(RuntimeError::new(operator, "Unrecognized binary operator.")),
        }
    }

    fn visit_grouping_expr(&self, expr: &Box<Expr>) -> Result<LoxObject> {
        self.interpret(expr)
    }

    fn visit_literal_expr(&self, literal: &crate::scanner::Literal) -> Result<LoxObject> {
        Ok(LoxObject::from_literal(literal))
    }

    fn visit_unary_expr(&self, operator: &Token, right: &Box<Expr>) -> Result<LoxObject> {
        let right = self.interpret(right)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use crate::scanner;

    #[test]
    fn evaluate_works() {
        let inputs = vec![
            ("1+2*3", LoxObject::Number(7.0)),
            ("(1+2)*3", LoxObject::Number(9.0)),
            ("1+(2*3)", LoxObject::Number(7.0)),
            ("(3*4)/2", LoxObject::Number(6.0)),
            ("3*4/2", LoxObject::Number(6.0)),
            (
                "\"Hello\" + \" \" + \"World\"",
                LoxObject::Str(String::from("Hello World")),
            ),
            ("true", LoxObject::Boolean(true)),
            ("false", LoxObject::Boolean(false)),
            ("nil", LoxObject::Nil),
        ];
        for (expression, expected_result) in inputs {
            let mut scanner = scanner::Scanner::new(expression);
            let tokens = scanner.scan_tokens();
            let mut parser = parser::Parser::new(tokens);
            let expr = parser.parse().unwrap();

            let interpreter = Interpreter;
            let result = interpreter.interpret(&expr).unwrap();
            assert_eq!(result, expected_result);
        }
    }

    #[test]
    fn bad_expressions_will_not_evaluate() {
        let inputs = vec![
            "\"Hello\" * 4",
            "4 * \"Hello\"",
            "\"Hello\" + 4",
            "4 + \"Hello\"",
        ];
        for expression in inputs {
            let mut scanner = scanner::Scanner::new(expression);
            let tokens = scanner.scan_tokens();
            let mut parser = parser::Parser::new(tokens);
            let expr = parser.parse().unwrap();
            let interpreter = Interpreter;
            match interpreter.interpret(&expr) {
                Err(_) => (),
                Ok(r) => panic!("Expected expression to return runtime error, not: {}", r),
            }
        }
    }
}
