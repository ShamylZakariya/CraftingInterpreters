use std::{cell::RefCell, fmt, rc::Rc};

use crate::environment::Environment;
use crate::error;
use crate::expr::*;
use crate::natives;
use crate::scanner::{Literal, Token, TokenType};
use crate::stmt::*;

//-----------------------------------------------------------------------------

use error::RuntimeError;
pub type Result<T> = std::result::Result<T, RuntimeError>;

pub enum InterpretResultStatus {
    // Returned when a runtime error occurs
    Error(RuntimeError),

    // Returned when control is flowing up the stack from a brack statement to the innermost loop.
    Break,

    // Return statement in a function, carrying optional return value payload.
    Return(Option<LoxObject>),
}

impl std::convert::From<error::RuntimeError> for InterpretResultStatus {
    fn from(error: RuntimeError) -> Self {
        InterpretResultStatus::Error(error)
    }
}

pub type InterpretResult<T> = std::result::Result<T, InterpretResultStatus>;

//-----------------------------------------------------------------------------
pub trait LoxCallable {
    fn arity(&self) -> usize;
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: &Vec<LoxObject>,
    ) -> InterpretResult<Option<LoxObject>>;
}

impl fmt::Debug for dyn LoxCallable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<callable arity {}>", self.arity())
    }
}

impl fmt::Display for dyn LoxCallable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<callable arity {}>", self.arity())
    }
}

impl PartialEq<dyn LoxCallable> for dyn LoxCallable {
    fn eq(&self, other: &Self) -> bool {
        &self == &other
    }
}

#[derive(Debug, Clone)]
pub enum LoxObject {
    Boolean(bool),
    Callable(Rc<RefCell<dyn LoxCallable>>),
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
            LoxObject::Nil => false,     // nil is falsey
            LoxObject::Boolean(b) => *b, // booleans are what they are
            _ => true,                   // everything else is *something*, which is truthy
        }
    }
}

impl PartialEq<LoxObject> for LoxObject {
    fn eq(&self, _other: &Self) -> bool {
        use LoxObject::*;
        match (self, _other) {
            (Boolean(b1), Boolean(b2)) => b1 == b2,
            (Callable(c1), Callable(c2)) => c1 == c2,
            (Nil, Nil) => true,
            (Number(n1), Number(n2)) => n1 == n2,
            (Str(s1), Str(s2)) => s1 == s2,
            (Undefined, Undefined) => true,
            _ => false,
        }
    }
}

impl Eq for LoxObject {}

impl fmt::Display for LoxObject {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LoxObject::Boolean(v) => {
                if *v {
                    write!(f, "true")
                } else {
                    write!(f, "false")
                }
            }
            LoxObject::Callable(c) => write!(f, "{}", c.borrow()),
            LoxObject::Nil => write!(f, "nil"),
            LoxObject::Number(n) => write!(f, "{}", n),
            LoxObject::Str(s) => write!(f, "{}", s),
            LoxObject::Undefined => write!(f, "<undefined>"),
        }
    }
}

//-----------------------------------------------------------------------------

struct LoxFunction {
    _name: Option<Token>,
    parameters: Vec<Token>,
    body: Vec<Box<Stmt>>,
    closure: Rc<RefCell<Environment>>,
}

impl LoxFunction {
    fn new_function(
        name: &Token,
        parameters: &Vec<Token>,
        body: &Vec<Box<Stmt>>,
        closure: Rc<RefCell<Environment>>,
    ) -> Self {
        LoxFunction {
            _name: Some(name.clone()),
            parameters: parameters.clone(),
            body: body.clone(),
            closure: closure,
        }
    }

    fn new_lambda(
        parameters: &Vec<Token>,
        body: &Vec<Box<Stmt>>,
        closure: Rc<RefCell<Environment>>,
    ) -> Self {
        LoxFunction {
            _name: None,
            parameters: parameters.clone(),
            body: body.clone(),
            closure: closure,
        }
    }

}

impl LoxCallable for LoxFunction {
    fn arity(&self) -> usize {
        return self.parameters.len();
    }

    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: &Vec<LoxObject>,
    ) -> InterpretResult<Option<LoxObject>> {
        let env = Rc::new(RefCell::new(Environment::as_child_of(self.closure.clone())));
        for i in 0..self.parameters.len() {
            env.borrow_mut()
                .define(&self.parameters[i].lexeme, &arguments[i]);
        }

        let ret = interpreter.execute_block(&self.body, env);
        match ret {
            // if function doesn't explicitly call return, we return None for it
            Ok(()) => Ok(None),
            Err(e) => match e {
                InterpretResultStatus::Return(v) => match v {
                    Some(v) => Ok(Some(v)),
                    None => Ok(None),
                },
                _ => Err(e),
            },
        }
    }
}

//-----------------------------------------------------------------------------

pub struct Interpreter {
    _globals: Rc<RefCell<Environment>>,
    environment: Rc<RefCell<Environment>>,
}
impl Interpreter {
    pub fn new() -> Self {
        let globals = Rc::new(RefCell::new(Environment::new()));
        globals.borrow_mut().define(
            "clock",
            &LoxObject::Callable(Rc::new(RefCell::new(natives::NativeClock::new()))),
        );

        Interpreter {
            _globals: globals.clone(),
            environment: globals,
        }
    }

    pub fn environment(&self) -> Rc<RefCell<Environment>> {
        self.environment.clone()
    }

    pub fn interpret(&mut self, statements: &Vec<Box<Stmt>>) -> Result<()> {
        for statement in statements {
            if let Err(e) = self.execute(statement) {
                return Err(self._process_error(e));
            }
        }
        Ok(())
    }

    pub fn evaluate(&mut self, expr: &Box<Expr>) -> Result<LoxObject> {
        match self._evaluate(expr) {
            Ok(result) => Ok(result),
            Err(e) => Err(self._process_error(e)),
        }
    }

    fn _process_error(&self, e:InterpretResultStatus)->RuntimeError{
        match e {
            InterpretResultStatus::Error(e) => {
                error::report::runtime_error(&e);
                return e;
            }
            InterpretResultStatus::Break => {
                // we're in big trouble
                return RuntimeError::with_message("A \"break\" statement trickled all the way up to root. Something is horribly wrong.");
            }
            InterpretResultStatus::Return(_) => {
                // we're in big trouble
                return RuntimeError::with_message("A \"return\" statement trickled all the way up to root. Something is horribly wrong.");
            }
        }
    }

    fn _evaluate(&mut self, expr: &Box<Expr>) -> InterpretResult<LoxObject> {
        expr.accept(self)
    }

    fn execute(&mut self, stmt: &Box<Stmt>) -> InterpretResult<()> {
        stmt.accept(self)
    }

    fn execute_block(
        &mut self,
        statements: &Vec<Box<Stmt>>,
        env: Rc<RefCell<Environment>>,
    ) -> InterpretResult<()> {
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

impl ExprVisitor<InterpretResult<LoxObject>> for Interpreter {
    fn visit_assign_expr(&mut self, name: &Token, value: &Box<Expr>) -> InterpretResult<LoxObject> {
        let value = self.evaluate(value)?;
        self.environment.borrow_mut().assign(name, &value)?;
        Ok(value)
    }

    fn visit_binary_expr(
        &mut self,
        left: &Box<Expr>,
        operator: &Token,
        right: &Box<Expr>,
    ) -> InterpretResult<LoxObject> {
        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;
        match operator.token_type {
            TokenType::Minus => {
                if let LoxObject::Number(l) = left {
                    if let LoxObject::Number(r) = right {
                        Ok(LoxObject::Number(l - r))
                    } else {
                        Err(InterpretResultStatus::Error(RuntimeError::new(
                            operator,
                            "Right operand not a number.",
                        )))
                    }
                } else {
                    Err(InterpretResultStatus::Error(RuntimeError::new(
                        operator,
                        "Left operand not a numbe.",
                    )))
                }
            }
            TokenType::Slash => {
                if let LoxObject::Number(l) = left {
                    if let LoxObject::Number(r) = right {
                        if r.abs() > 1e-8 {
                            Ok(LoxObject::Number(l / r))
                        } else {
                            Err(InterpretResultStatus::Error(RuntimeError::new(
                                operator,
                                "Attempt to divide by zero.",
                            )))
                        }
                    } else {
                        Err(InterpretResultStatus::Error(RuntimeError::new(
                            operator,
                            "Right operand not a number.",
                        )))
                    }
                } else {
                    Err(InterpretResultStatus::Error(RuntimeError::new(
                        operator,
                        "Left operand not a number.",
                    )))
                }
            }
            TokenType::Star => {
                if let LoxObject::Number(l) = left {
                    if let LoxObject::Number(r) = right {
                        Ok(LoxObject::Number(l * r))
                    } else {
                        Err(InterpretResultStatus::Error(RuntimeError::new(
                            operator,
                            "Right operand not a number.",
                        )))
                    }
                } else {
                    Err(InterpretResultStatus::Error(RuntimeError::new(
                        operator,
                        "Left operand not a number.",
                    )))
                }
            }
            TokenType::Plus => {
                if let LoxObject::Number(l) = left {
                    if let LoxObject::Number(r) = right {
                        Ok(LoxObject::Number(l + r))
                    } else {
                        Err(InterpretResultStatus::Error(RuntimeError::new(
                            operator,
                            "Right operand not a number.",
                        )))
                    }
                } else if let LoxObject::Str(l) = left {
                    if let LoxObject::Str(r) = right {
                        Ok(LoxObject::Str(format!("{}{}", l, r)))
                    } else {
                        Err(InterpretResultStatus::Error(RuntimeError::new(
                            operator,
                            "Right operand not a string",
                        )))
                    }
                } else {
                    Err(InterpretResultStatus::Error(RuntimeError::new(
                        operator,
                        "Left operand not a number or string.",
                    )))
                }
            }

            TokenType::EqualEqual => {
                if let LoxObject::Number(l) = left {
                    if let LoxObject::Number(r) = right {
                        Ok(LoxObject::Boolean(l == r))
                    } else {
                        Err(InterpretResultStatus::Error(RuntimeError::new(
                            operator,
                            "Right operand not a number.",
                        )))
                    }
                } else if let LoxObject::Str(l) = left {
                    if let LoxObject::Str(r) = right {
                        Ok(LoxObject::Boolean(l == r))
                    } else {
                        Err(InterpretResultStatus::Error(RuntimeError::new(
                            operator,
                            "Right operand not a string",
                        )))
                    }
                } else {
                    Err(InterpretResultStatus::Error(RuntimeError::new(
                        operator,
                        "Left operand not a number or string.",
                    )))
                }
            }

            TokenType::Less => {
                if let LoxObject::Number(l) = left {
                    if let LoxObject::Number(r) = right {
                        Ok(LoxObject::Boolean(l < r))
                    } else {
                        Err(InterpretResultStatus::Error(RuntimeError::new(
                            operator,
                            "Right operand not a number.",
                        )))
                    }
                } else {
                    Err(InterpretResultStatus::Error(RuntimeError::new(
                        operator,
                        "Left operand not a number.",
                    )))
                }
            }
            TokenType::LessEqual => {
                if let LoxObject::Number(l) = left {
                    if let LoxObject::Number(r) = right {
                        Ok(LoxObject::Boolean(l <= r))
                    } else {
                        Err(InterpretResultStatus::Error(RuntimeError::new(
                            operator,
                            "Right operand not a number.",
                        )))
                    }
                } else {
                    Err(InterpretResultStatus::Error(RuntimeError::new(
                        operator,
                        "Left operand not a number.",
                    )))
                }
            }
            TokenType::Greater => {
                if let LoxObject::Number(l) = left {
                    if let LoxObject::Number(r) = right {
                        Ok(LoxObject::Boolean(l > r))
                    } else {
                        Err(InterpretResultStatus::Error(RuntimeError::new(
                            operator,
                            "Right operand not a number.",
                        )))
                    }
                } else {
                    Err(InterpretResultStatus::Error(RuntimeError::new(
                        operator,
                        "Left operand not a number.",
                    )))
                }
            }
            TokenType::GreaterEqual => {
                if let LoxObject::Number(l) = left {
                    if let LoxObject::Number(r) = right {
                        Ok(LoxObject::Boolean(l >= r))
                    } else {
                        Err(InterpretResultStatus::Error(RuntimeError::new(
                            operator,
                            "Right operand not a number.",
                        )))
                    }
                } else {
                    Err(InterpretResultStatus::Error(RuntimeError::new(
                        operator,
                        "Left operand not a number.",
                    )))
                }
            }

            _ => Err(InterpretResultStatus::Error(RuntimeError::new(
                operator,
                "Unrecognized binary operator.",
            ))),
        }
    }

    fn visit_call_expr(
        &mut self,
        callee: &Box<Expr>,
        paren: &Token,
        arguments: &Vec<Box<Expr>>,
    ) -> InterpretResult<LoxObject> {
        let callee = self._evaluate(callee)?;
        let mut args = vec![];
        for arg in arguments {
            args.push(self._evaluate(arg)?);
        }

        if let LoxObject::Callable(callable) = callee {
            if args.len() != callable.borrow().arity() {
                return Err(InterpretResultStatus::Error(RuntimeError::new(
                    paren,
                    format!(
                        "Expected {} arguments but got {}",
                        callable.borrow().arity(),
                        args.len()
                    )
                    .as_str(),
                )));
            }

            if let Some(v) = callable.borrow().call(self, &args)? {
                return Ok(v);
            } else {
                return Ok(LoxObject::Nil);
            }
        }
        return Err(InterpretResultStatus::Error(RuntimeError::new(
            paren,
            "Callee is not a callable expression (function or method or class ctor).",
        )));
    }

    fn visit_grouping_expr(&mut self, expr: &Box<Expr>) -> InterpretResult<LoxObject> {
        self._evaluate(expr)
    }

    fn visit_lambda_expr(&mut self, parameters: &Vec<Token>, body: &Vec<Box<Stmt>>) -> InterpretResult<LoxObject> {
        let fun = LoxFunction::new_lambda(parameters, body, self.environment.clone());
        let callable = LoxObject::Callable(Rc::new(RefCell::new(fun)));
        Ok(callable)
    }

    fn visit_literal_expr(
        &mut self,
        literal: &crate::scanner::Literal,
    ) -> InterpretResult<LoxObject> {
        Ok(LoxObject::from_literal(literal))
    }

    fn visit_logical_expr(
        &mut self,
        left: &Box<Expr>,
        operator: &Token,
        right: &Box<Expr>,
    ) -> InterpretResult<LoxObject> {
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
                return Err(InterpretResultStatus::Error(RuntimeError::new(
                    operator,
                    "Only And and Or are supported conditional operators.",
                )))
            }
        }
        // expression result is right side because logical expr wasn't short circuited
        self._evaluate(right)
    }

    fn visit_ternary_expr(
        &mut self,
        condition: &Box<Expr>,
        then_value: &Box<Expr>,
        else_value: &Box<Expr>,
    ) -> InterpretResult<LoxObject> {
        if self.evaluate(condition)?.is_truthy() {
            self._evaluate(then_value)
        } else {
            self._evaluate(else_value)
        }
    }

    fn visit_unary_expr(
        &mut self,
        operator: &Token,
        right: &Box<Expr>,
    ) -> InterpretResult<LoxObject> {
        let right = self.evaluate(right)?;
        match operator.token_type {
            TokenType::Bang => Ok(LoxObject::Boolean(!right.is_truthy())),
            TokenType::Minus => match right {
                LoxObject::Number(n) => Ok(LoxObject::Number(-n)),
                _ => Err(InterpretResultStatus::Error(RuntimeError::new(
                    operator,
                    "Unary negative can only be applied to numbers.",
                ))),
            },
            _ => Err(InterpretResultStatus::Error(RuntimeError::new(
                operator,
                "Unsupported unary operator.",
            ))),
        }
    }

    fn visit_variable_expr(&mut self, name: &Token) -> InterpretResult<LoxObject> {
        let value = self.environment.borrow().get(name)?;
        if let LoxObject::Undefined = value {
            return Err(InterpretResultStatus::Error(RuntimeError::new(
                name,
                "Attempt to read from undefined variable.",
            )));
        }
        Ok(value)
    }
}

impl StmtVisitor<InterpretResult<()>> for Interpreter {
    fn visit_block_stmt(&mut self, statements: &Vec<Box<Stmt>>) -> InterpretResult<()> {
        let env = Rc::new(RefCell::new(Environment::as_child_of(
            self.environment.clone(),
        )));
        self.execute_block(statements, env)
    }

    fn visit_break_stmt(&mut self) -> InterpretResult<()> {
        Err(InterpretResultStatus::Break)
    }

    fn visit_expression_stmt(&mut self, expression: &Box<Expr>) -> InterpretResult<()> {
        match self.evaluate(expression) {
            Ok(_) => Ok(()),
            Err(e) => Err(InterpretResultStatus::Error(e)),
        }
    }

    fn visit_function_stmt(
        &mut self,
        name: &Token,
        parameters: &Vec<Token>,
        body: &Vec<Box<Stmt>>,
    ) -> InterpretResult<()> {
        let fun = LoxFunction::new_function(name, parameters, body, self.environment.clone());
        let callable = LoxObject::Callable(Rc::new(RefCell::new(fun)));
        self.environment()
            .borrow_mut()
            .define(&name.lexeme, &callable);
        Ok(())
    }

    fn visit_if_stmt(
        &mut self,
        condition: &Box<Expr>,
        then_branch: &Box<Stmt>,
        else_branch: &Option<Box<Stmt>>,
    ) -> InterpretResult<()> {
        if self.evaluate(condition)?.is_truthy() {
            self.execute(then_branch)?;
        } else if let Some(else_branch) = else_branch {
            self.execute(else_branch)?;
        }
        Ok(())
    }

    fn visit_print_stmt(&mut self, expression: &Box<Expr>) -> InterpretResult<()> {
        let value = self.evaluate(expression)?;
        println!("{}", value);
        Ok(())
    }

    fn visit_return_stmt(
        &mut self,
        _keyword: &Token,
        value: &Option<Box<Expr>>,
    ) -> InterpretResult<()> {
        let mut return_value = None;
        if let Some(value) = value {
            return_value = Some(self._evaluate(value)?);
        }
        Err(InterpretResultStatus::Return(return_value))
    }

    fn visit_var_stmt(
        &mut self,
        name: &Token,
        initializer: &Option<Box<Expr>>,
    ) -> InterpretResult<()> {
        let mut value = LoxObject::Undefined;
        if let Some(initializer) = initializer {
            value = self.evaluate(initializer)?;
        }
        self.environment.borrow_mut().define(&name.lexeme, &value);
        Ok(())
    }

    fn visit_while_stmt(&mut self, condition: &Box<Expr>, body: &Box<Stmt>) -> InterpretResult<()> {
        while self.evaluate(condition)?.is_truthy() {
            match self.execute(body) {
                Ok(_) => (),
                Err(status) => match status {
                    InterpretResultStatus::Error(runtime_error) => {
                        return Err(InterpretResultStatus::Error(runtime_error));
                    }
                    InterpretResultStatus::Break => {
                        // time to break from loop.
                        break;
                    }
                    InterpretResultStatus::Return(v) => {
                        // pass the return statement up
                        return Err(InterpretResultStatus::Return(v));
                    }
                },
            };
        }
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
            assert!(interpreter.evaluate(&expr).is_err());
        }
    }

    #[test]
    fn logical_expressions_evaluate() {
        let inputs = vec![
            ("1 and 2", LoxObject::Number(2.0)),
            ("\"hi\" and nil", LoxObject::Nil),
            ("0 and 2", LoxObject::Number(2.0)),
            ("0 or 2", LoxObject::Number(0.0)),
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

    /// The inputs
    fn execute(inputs: &Vec<(&str, Vec<(&str, LoxObject)>)>) {
        for (program, expected_results) in inputs {
            let mut scanner = scanner::Scanner::new(program);
            let tokens = scanner.scan_tokens();
            let mut parser = parser::Parser::new(tokens);
            let statements = parser.parse().unwrap();

            let mut interpreter = Interpreter::new();
            interpreter.interpret(&statements).unwrap();

            for (name, value) in expected_results {
                let token = Token::new(TokenType::Identifier, String::from(*name), None, 1);
                assert_eq!(
                    interpreter.environment().borrow().get(&token).unwrap(),
                    *value
                );
            }
        }
    }

    #[test]
    fn programs_produce_expected_results() {
        let inputs = vec![
            (
                r#"
                var a = 1;
                {
                    a = 2;
                }
                var b = 3;
                {
                    var b = 4;
                }
                "#,
                vec![("a", LoxObject::Number(2.0)), ("b", LoxObject::Number(3.0))],
            ),
            (
                r#"
                var a = 0;
                var b = 1;

                while (a < 10000) {
                  print a;
                  var temp = a;
                  a = b;
                  b = temp + b;
                  if (a == 377) {
                      break;
                  }
                }
                "#,
                vec![("a", LoxObject::Number(377.0))],
            ),
        ];
        execute(&inputs);
    }

    #[test]
    fn return_exits_functions_with_expected_values() {
        let inputs = vec![
            (
                r#"
                fun foo(t) {
                    if (t) {
                        return 1;
                    }
                    return 0;
                }
                var a = foo(true);
                var b = foo(false);
                "#,
                vec![("a", LoxObject::Number(1.0)), ("b", LoxObject::Number(0.0))],
            ),
            (
                r#"
                fun foo() {
                }
                var a = foo();
                "#,
                vec![("a", LoxObject::Nil)],
            ),
        ];
        execute(&inputs);
    }

    #[test]
    fn incorrect_function_arity_is_runtime_error() {
        let inputs = vec![
            r#"
            fun no_args() {}
            no_args(1);
            "#,
            r#"
            fun one_arg(a) {}
            one_arg();
            "#,
            r#"
            fun two_args(a,b) {}
            two_args();
            "#,
            r#"
            fun two_args(a,b) {}
            two_args(1,2,3);
            "#,
        ];

        for program in inputs {
            let mut scanner = scanner::Scanner::new(program);
            let tokens = scanner.scan_tokens();
            let mut parser = parser::Parser::new(tokens);
            let ast = parser.parse().unwrap();
            let mut interpreter = Interpreter::new();
            assert!(interpreter.interpret(&ast).is_err());
        }
    }
}
