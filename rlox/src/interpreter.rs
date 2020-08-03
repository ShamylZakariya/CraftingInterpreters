use std::{cell::RefCell, fmt, rc::Rc};

use crate::environment::Environment;
use crate::error;
use crate::expr::*;
use crate::scanner::{Literal, Token, TokenType};
use crate::stmt::*;

#[derive(PartialEq, Debug, Clone)]
pub enum LoxObject {
    Boolean(bool),
    Nil,
    Number(f64),
    Str(String),
    Undefined,
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
            LoxObject::Undefined => write!(f, "<undefined>"),
            LoxObject::Nil => write!(f, "nil"),
        }
    }
}

//-------------

use error::RuntimeError;
pub type Result<T> = std::result::Result<T, RuntimeError>;

pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
}
impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            environment: Rc::new(RefCell::new(Environment::new())),
        }
    }

    pub fn interpret(&mut self, statements: &Vec<Box<Stmt>>) -> Result<()> {
        for statement in statements {
            if let Err(e) = self.execute(statement) {
                error::report::runtime_error(&e);
                return Err(e);
            }
        }
        Ok(())
    }

    pub fn evaluate(&mut self, expr: &Box<Expr>) -> Result<LoxObject> {
        match expr.accept(self) {
            Ok(result) => Ok(result),
            Err(e) => {
                error::report::runtime_error(&e);
                Err(e)
            }
        }
    }

    pub fn execute(&mut self, stmt: &Box<Stmt>) -> Result<()> {
        stmt.accept(self)
    }

    pub fn execute_block(
        &mut self,
        statements: &Vec<Box<Stmt>>,
        env: Rc<RefCell<Environment>>,
    ) -> Result<()> {
        let previous_env = self.environment.clone();
        self.environment = env;

        for statement in statements {
            if let Err(e) = self.execute(statement) {
                // had an error, restore parent env and bail
                self.environment = previous_env;
                return Err(e);
            }
        }

        // completed successfully, restore parent env.
        self.environment = previous_env;
        Ok(())
    }
}

impl ExprVisitor<Result<LoxObject>> for Interpreter {
    fn visit_assign_expr(&mut self, name: &Token, value: &Box<Expr>) -> Result<LoxObject> {
        let value = self.evaluate(value)?;
        self.environment.borrow_mut().assign(name, &value)?;
        Ok(value)
    }

    fn visit_binary_expr(
        &mut self,
        left: &Box<Expr>,
        operator: &Token,
        right: &Box<Expr>,
    ) -> Result<LoxObject> {
        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;
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

    fn visit_grouping_expr(&mut self, expr: &Box<Expr>) -> Result<LoxObject> {
        self.evaluate(expr)
    }

    fn visit_literal_expr(&mut self, literal: &crate::scanner::Literal) -> Result<LoxObject> {
        Ok(LoxObject::from_literal(literal))
    }

    fn visit_logical_expr(
        &mut self,
        left: &Box<Expr>,
        operator: &Token,
        right: &Box<Expr>,
    ) -> Result<LoxObject> {
        let left = self.evaluate(left)?;
        match operator.token_type {
            TokenType::Or => {
                // left side of Or is truthy, result of expr is the left side
                if left.is_truthy() {
                    return Ok(left);
                }
            }
            TokenType::And => {
                // if left side of And is not truthy, the expression result is left side
                if !left.is_truthy() {
                    return Ok(left);
                }
            }
            _ => {
                return Err(RuntimeError::new(
                    operator,
                    "Only And and Or are supported conditional operators.",
                ))
            }
        }
        // expression result is right side because logical expr wasn't short circuited
        self.evaluate(right)
    }

    fn visit_unary_expr(&mut self, operator: &Token, right: &Box<Expr>) -> Result<LoxObject> {
        let right = self.evaluate(right)?;
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

    fn visit_variable_expr(&mut self, name: &Token) -> Result<LoxObject> {
        let value = self.environment.borrow().get(name)?;
        if let LoxObject::Undefined = value {
            return Err(RuntimeError::new(
                name,
                "Attempt to read from undefined variable.",
            ));
        }
        Ok(value)
    }
}

impl StmtVisitor<Result<()>> for Interpreter {
    fn visit_block_stmt(&mut self, statements: &Vec<Box<Stmt>>) -> Result<()> {
        let env = Rc::new(RefCell::new(Environment::as_child_of(
            self.environment.clone(),
        )));
        self.execute_block(statements, env)
    }
    fn visit_expression_stmt(&mut self, expression: &Box<Expr>) -> Result<()> {
        match self.evaluate(expression) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    fn visit_if_stmt(
        &mut self,
        condition: &Box<Expr>,
        then_branch: &Box<Stmt>,
        else_branch: &Option<Box<Stmt>>,
    ) -> Result<()> {
        if self.evaluate(condition)?.is_truthy() {
            self.execute(then_branch)?;
        } else if let Some(else_branch) = else_branch {
            self.execute(else_branch)?;
        }
        Ok(())
    }

    fn visit_print_stmt(&mut self, expression: &Box<Expr>) -> Result<()> {
        let value = self.evaluate(expression)?;
        println!("{}", value);
        Ok(())
    }

    fn visit_var_stmt(&mut self, name: &Token, initializer: &Option<Box<Expr>>) -> Result<()> {
        let mut value = LoxObject::Undefined;
        if let Some(initializer) = initializer {
            value = self.evaluate(initializer)?;
        }
        self.environment.borrow_mut().define(&name.lexeme, &value);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use crate::scanner;

    #[test]
    fn evaluates_expressions() {
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
            let expr = parser.parse_expression().unwrap();

            let mut interpreter = Interpreter::new();
            let result = interpreter.evaluate(&expr).unwrap();
            assert_eq!(result, expected_result);
        }
    }

    #[test]
    fn bad_expressions_are_errors() {
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
            let expr = parser.parse_expression().unwrap();
            let mut interpreter = Interpreter::new();
            match interpreter.evaluate(&expr) {
                Err(_) => (),
                Ok(r) => panic!("Expected expression to return runtime error, not: {}", r),
            }
        }
    }

    #[test]
    fn logical_expressions_evaluate() {
        let inputs = vec![
            ("1 and 2", LoxObject::Number(1.0)),
            ("\"hi\" and 2", LoxObject::Str(String::from("hi"))),
            ("0 and 2", LoxObject::Number(0.0)),
            ("0 or 2", LoxObject::Number(2.0)),
        ];
        for (expression, expected_result) in inputs {
            let mut scanner = scanner::Scanner::new(expression);
            let tokens = scanner.scan_tokens();
            let mut parser = parser::Parser::new(tokens);
            let expr = parser.parse_expression().unwrap();

            let mut interpreter = Interpreter::new();
            let result = interpreter.evaluate(&expr).unwrap();
            assert_eq!(result, expected_result);
        }
    }
}
