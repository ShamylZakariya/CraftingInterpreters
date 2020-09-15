use std::collections::HashMap;
use std::{cell::RefCell, rc::Rc};

use crate::ast::*;
use crate::callable::LoxCallable;
use crate::class::LoxClass;
use crate::environment::Environment;
use crate::error;
use crate::function::LoxFunction;
use crate::natives;
use crate::object::LoxObject;
use crate::scanner::{Token, TokenType};

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

pub struct Interpreter {
    globals: Environment,
    environment: Environment,
    locals: HashMap<Expr, usize>,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut globals = Environment::new();
        globals.define(
            "clock",
            &LoxObject::Callable(Rc::new(RefCell::new(natives::NativeClock::new()))),
        );

        Interpreter {
            globals: globals.clone(),
            environment: globals,
            locals: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    pub fn environment(&self) -> Environment {
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

    fn _process_error(&self, e: InterpretResultStatus) -> RuntimeError {
        match e {
            InterpretResultStatus::Error(e) => {
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

    pub fn execute_block(
        &mut self,
        statements: &Vec<Box<Stmt>>,
        env: Environment,
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

    pub fn resolve_local(&mut self, variable: &Expr, distance: usize) {
        self.locals.insert(variable.clone(), distance);
    }

    fn look_up_variable(&self, name: &Token, expr: &Expr) -> InterpretResult<LoxObject> {
        if let Some(distance) = self.locals.get(expr) {
            let v = self.environment.get_at(*distance, &name.lexeme)?;
            Ok(v)
        } else {
            let v = self.globals.get(name)?;
            Ok(v)
        }
    }
}

impl ExprVisitor<InterpretResult<LoxObject>> for Interpreter {
    fn visit_assign_expr(
        &mut self,
        expr: &Expr,
        name: &Token,
        value: &Box<Expr>,
    ) -> InterpretResult<LoxObject> {
        let value = self.evaluate(value)?;

        if let Some(distance) = self.locals.get(expr) {
            self.environment.assign_at(*distance, name, &value)?;
        } else {
            self.globals.assign(name, &value)?;
        }

        Ok(value)
    }

    fn visit_binary_expr(
        &mut self,
        _expr: &Expr,
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
                        "Left operand not a number.",
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
                    } else if let LoxObject::Number(r) = right {
                        Ok(LoxObject::Str(format!("{}{}", l, r)))
                    } else if let LoxObject::Boolean(r) = right {
                        Ok(LoxObject::Str(format!("{}{}", l, r)))
                    } else if let LoxObject::Nil = right {
                        Ok(LoxObject::Str(format!("{}nil", l)))
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
        _expr: &Expr,
        callee: &Box<Expr>,
        paren: &Token,
        arguments: &Vec<Box<Expr>>,
    ) -> InterpretResult<LoxObject> {
        let callee = self._evaluate(callee)?;
        let mut args = vec![];
        for arg in arguments {
            args.push(self._evaluate(arg)?);
        }

        match callee {
            LoxObject::Callable(callable) => {
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
                    Ok(v)
                } else {
                    Ok(LoxObject::Nil)
                }
            }

            LoxObject::Class(class) => {
                if args.len() != class.arity() {
                    return Err(InterpretResultStatus::Error(RuntimeError::new(
                        paren,
                        format!(
                            "Expected {} arguments but got {}",
                            class.arity(),
                            args.len()
                        )
                        .as_str(),
                    )));
                }

                if let Some(v) = class.call(self, &args)? {
                    Ok(v)
                } else {
                    Ok(LoxObject::Nil)
                }
            }

            _ => Err(InterpretResultStatus::Error(RuntimeError::new(
                paren,
                "Callee is not a callable expression (function, method, or class ctor).",
            ))),
        }
    }

    fn visit_get_expr(
        &mut self,
        _expr: &Expr,
        object: &Box<Expr>,
        name: &Token,
    ) -> InterpretResult<LoxObject> {
        let object = self.evaluate(object)?;

        match object {
            LoxObject::Instance(lox_instance) => {
                match lox_instance.get(name) {
                    Ok(obj) => {
                        if let LoxObject::Callable(callable) = &obj {
                            if callable.borrow().is_property() {
                                // this is a property field on a class instance, invoke it.
                                if let Some(r) = callable.borrow().call(self, &vec![])? {
                                    Ok(r)
                                } else {
                                    // Property didn't explicitly return anything - which is weird, but let's
                                    // allow it because it could be desired that the property invocation causes a
                                    // desired side-effect.
                                    Ok(LoxObject::Nil)
                                }
                            } else {
                                Ok(obj)
                            }
                        } else {
                            Ok(obj)
                        }
                    }
                    Err(e) => Err(InterpretResultStatus::Error(e)),
                }
            }
            LoxObject::Class(lox_class) => match lox_class.get(name) {
                Ok(obj) => Ok(obj),
                Err(e) => Err(InterpretResultStatus::Error(e)),
            },
            _ => Err(InterpretResultStatus::Error(RuntimeError::new(
                name,
                "Only instances have properties.",
            ))),
        }
    }

    fn visit_grouping_expr(
        &mut self,
        _expr: &Expr,
        contents: &Box<Expr>,
    ) -> InterpretResult<LoxObject> {
        self._evaluate(contents)
    }

    fn visit_lambda_expr(
        &mut self,
        _expr: &Expr,
        parameters: &Vec<Token>,
        body: &Vec<Box<Stmt>>,
    ) -> InterpretResult<LoxObject> {
        let fun = LoxFunction::new_lambda(parameters, body, self.environment.clone());
        let callable = LoxObject::Callable(Rc::new(RefCell::new(fun)));
        Ok(callable)
    }

    fn visit_literal_expr(
        &mut self,
        _expr: &Expr,
        literal: &crate::scanner::Literal,
    ) -> InterpretResult<LoxObject> {
        Ok(LoxObject::from_literal(literal))
    }

    fn visit_logical_expr(
        &mut self,
        _expr: &Expr,
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

    fn visit_set_expr(
        &mut self,
        _expr: &Expr,
        object: &Box<Expr>,
        name: &Token,
        value: &Box<Expr>,
    ) -> InterpretResult<LoxObject> {
        let object = self.evaluate(object)?;
        match object {
            LoxObject::Instance(instance) => {
                let value = self.evaluate(value)?;
                instance.set(name, &value);
                Ok(value)
            }
            LoxObject::Class(class) => {
                let value = self.evaluate(value)?;
                class.set(name, &value);
                Ok(value)
            }
            _ => Err(InterpretResultStatus::Error(RuntimeError::new(
                name,
                "Only object instances have fields.",
            ))),
        }
    }

    fn visit_this_expr(&mut self, expr: &Expr, keyword: &Token) -> InterpretResult<LoxObject> {
        self.look_up_variable(keyword, expr)
    }

    fn visit_ternary_expr(
        &mut self,
        _expr: &Expr,
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
        _expr: &Expr,
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

    fn visit_variable_expr(&mut self, expr: &Expr, name: &Token) -> InterpretResult<LoxObject> {
        self.look_up_variable(name, expr)
    }
}

impl StmtVisitor<InterpretResult<()>> for Interpreter {
    fn visit_block_stmt(
        &mut self,
        _stmt: &Stmt,
        statements: &Vec<Box<Stmt>>,
    ) -> InterpretResult<()> {
        let env = Environment::as_child_of(self.environment.clone());
        self.execute_block(statements, env)
    }

    fn visit_break_stmt(&mut self, _stmt: &Stmt, _keyword: &Token) -> InterpretResult<()> {
        Err(InterpretResultStatus::Break)
    }

    fn visit_class_stmt(
        &mut self,
        _stmt: &Stmt,
        name: &Token,
        methods: &Vec<Box<Stmt>>,
        class_methods: &Vec<Box<Stmt>>,
    ) -> InterpretResult<()> {
        self.environment.define(&name.lexeme, &LoxObject::Nil);

        let mut instance_method_fns: HashMap<String, Rc<RefCell<LoxFunction>>> = HashMap::new();
        for method in methods {
            match &**method {
                Stmt::Function {
                    name,
                    parameters,
                    body,
                    fn_type,
                } => {
                    // if this method is called "init" we know it is the class's init() method
                    // and need to flag it as such when creating the function.
                    let is_init = name.lexeme == "init";
                    let function = LoxFunction::new_function(
                        name,
                        parameters,
                        body,
                        self.environment.clone(),
                        is_init,
                        *fn_type,
                    );
                    let function = Rc::new(RefCell::new(function));
                    instance_method_fns.insert(name.lexeme.to_owned(), function);
                }
                _ => {
                    return Err(InterpretResultStatus::Error(RuntimeError::new(
                        name,
                        "Method in class somehow not a Stmt::Function instance.",
                    )));
                }
            }
        }

        let mut class_method_fns: HashMap<String, Rc<RefCell<LoxFunction>>> = HashMap::new();
        for method in class_methods {
            match &**method {
                Stmt::Function {
                    name,
                    parameters,
                    body,
                    fn_type,
                } => {
                    let function = LoxFunction::new_function(
                        name,
                        parameters,
                        body,
                        self.environment.clone(),
                        false,
                        *fn_type,
                    );
                    let function = Rc::new(RefCell::new(function));
                    class_method_fns.insert(name.lexeme.to_owned(), function);
                }
                _ => {
                    return Err(InterpretResultStatus::Error(RuntimeError::new(
                        name,
                        "Method in class somehow not a Stmt::Function instance.",
                    )));
                }
            }
        }

        let class_obj = LoxObject::Class(LoxClass::new(
            &name.lexeme,
            instance_method_fns,
            class_method_fns,
        ));
        self.environment.assign(name, &class_obj)?;

        Ok(())
    }

    fn visit_expression_stmt(
        &mut self,
        _stmt: &Stmt,
        expression: &Box<Expr>,
    ) -> InterpretResult<()> {
        match self.evaluate(expression) {
            Ok(_) => Ok(()),
            Err(e) => Err(InterpretResultStatus::Error(e)),
        }
    }

    fn visit_function_stmt(
        &mut self,
        _stmt: &Stmt,
        name: &Token,
        parameters: &Vec<Token>,
        body: &Vec<Box<Stmt>>,
        fn_type: CallableType,
    ) -> InterpretResult<()> {
        let fun = LoxFunction::new_function(
            name,
            parameters,
            body,
            self.environment.clone(),
            false,
            fn_type,
        );
        let callable = LoxObject::Callable(Rc::new(RefCell::new(fun)));
        self.environment.define(&name.lexeme, &callable);
        Ok(())
    }

    fn visit_if_stmt(
        &mut self,
        _stmt: &Stmt,
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

    fn visit_print_stmt(&mut self, _stmt: &Stmt, expression: &Box<Expr>) -> InterpretResult<()> {
        let value = self.evaluate(expression)?;
        println!("{}", value);
        Ok(())
    }

    fn visit_return_stmt(
        &mut self,
        _stmt: &Stmt,
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
        _stmt: &Stmt,
        name: &Token,
        initializer: &Option<Box<Expr>>,
    ) -> InterpretResult<()> {
        let mut value = LoxObject::Undefined;
        if let Some(initializer) = initializer {
            value = self.evaluate(initializer)?;
        }
        self.environment.define(&name.lexeme, &value);
        Ok(())
    }

    fn visit_while_stmt(
        &mut self,
        _stmt: &Stmt,
        condition: &Box<Expr>,
        body: &Box<Stmt>,
    ) -> InterpretResult<()> {
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
    use crate::resolver;
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
            "4 + \"Hello\"",
            "4.foo",
            "4.foo()",
            "\"Hello\".bar",
            "\"Hello\".bar()",
            "4.foo = 10",
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

            let mut r = resolver::Resolver::new(&mut interpreter);
            r.resolve(&statements)
                .expect("Expected successful resolve pass");

            interpreter.interpret(&statements).unwrap();

            let mut token_id = 0;
            for (name, value) in expected_results {
                let token = Token::new(
                    TokenType::Identifier,
                    String::from(*name),
                    None,
                    1,
                    token_id,
                );
                token_id += 1;
                assert_eq!(interpreter.environment().get(&token).unwrap(), *value);
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
                    b; // need to "access" b to prevent unused variable error :P
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
            (
                // `this` does what it should
                r#"
                class Cake {
                    init(type) {
                        this.type = type;
                    }
                    kind() {
                        return this.type + " cake";
                    }
                }
                var kind = Cake("Chocolate").kind();
                "#,
                vec![("kind", LoxObject::Str("Chocolate cake".to_owned()))],
            ),
            (
                // method bindings should work
                r#"
                class Foo {
                    init(w) {
                        this.w = w;
                    }
                    what() { return this.w; }
                }
                var foo = Foo("bar");
                var method = foo.what;
                var value_0 = method();
                foo.w = "baz";
                var value_1 = method();
                "#,
                vec![
                    ("value_0", LoxObject::Str("bar".to_owned())),
                    ("value_1", LoxObject::Str("baz".to_owned())),
                ],
            ),
            (
                // method bindings should work
                r#"
                class Person {
                    init(n) {
                        this.n = n;
                    }
                    name() { return this.n; }
                }
                var jane = Person("jane");
                var bill = Person("bill");
                bill.name = jane.name;
                var value_0 = jane.name();
                var value_1 = bill.name(); // uses jane's `this`
                "#,
                vec![
                    ("value_0", LoxObject::Str("jane".to_owned())),
                    ("value_1", LoxObject::Str("jane".to_owned())),
                ],
            ),
            (
                // property methods, simple 1
                r#"
                class Minimal {
                    foo {
                        return 1;
                    }
                }

                var minimal = Minimal();
                var value_0 = minimal.foo;
                "#,
                vec![("value_0", LoxObject::Number(1.0))],
            ),
            (
                // property methods, simple 2
                r#"
                class Minimal {
                    init(f) {
                        this._foo = f;
                    }
                    foo {
                        return this._foo;
                    }
                }

                var minimal = Minimal(123);
                var value_0 = minimal.foo;
                "#,
                vec![("value_0", LoxObject::Number(123.0))],
            ),
            (
                // property methods, simple 3
                r#"
                class Minimal {
                    init(f,b,z) {
                        this._foo = f;
                        this._bar = b;
                        this._baz = z;
                    }
                    foo {
                        return this._foo;
                    }
                    bar {
                        return this._bar;
                    }
                    baz {
                        return this._baz;
                    }
                    fbz { return this.foo + this.baz + this.bar; }
                }

                var minimal = Minimal(1,2,3);
                var value_0 = minimal.foo;
                var value_1 = minimal.bar;
                var value_2 = minimal.baz;
                var value_3 = minimal.fbz;
                "#,
                vec![
                    ("value_0", LoxObject::Number(1.0)),
                    ("value_1", LoxObject::Number(2.0)),
                    ("value_2", LoxObject::Number(3.0)),
                    ("value_3", LoxObject::Number(6.0)),
                ],
            ),
            (
                // class methods
                r#"
                class Math {
                    class square(x) { return x * x; }
                }

                var value_0 = Math.square(2);
                "#,
                vec![("value_0", LoxObject::Number(4.0))],
            ),
            (
                // class fields != instance fields
                r#"
                class Foo {
                }

                Foo.value = 10;
                var value_0 = Foo.value;

                var f = Foo();
                f.value = 11;
                var value_1 = f.value;
                "#,
                vec![
                    ("value_0", LoxObject::Number(10.0)),
                    ("value_1", LoxObject::Number(11.0)),
                ],
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
            (
                r#"
                class Foo {
                    init(a) {
                        this.a = a;
                    }
                    get_a() { return this.a; }
                    get_a_plus_b(b) { return this.a + b; }
                }
                var f = Foo(123);
                var a = f.get_a(); // 123
                var b = f.get_a_plus_b(1); //124
                "#,
                vec![
                    ("a", LoxObject::Number(123.0)),
                    ("b", LoxObject::Number(124.0)),
                ],
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
            r#"
            class Foo {
                init(a,b,c) {}
            }
            Foo(1,1);
            "#,
            r#"
            class Foo {
                init() {}
                bar(a,b){}
            }
            Foo().bar(1); // wrong arg count
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
