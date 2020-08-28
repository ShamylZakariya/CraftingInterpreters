use std::collections::HashMap;

use crate::error;
use crate::expr::*;
use crate::interpreter::Interpreter;
use crate::scanner::Token;
use crate::stmt::*;

pub type Result<T> = std::result::Result<T, error::ResolveError>;

#[derive(Copy, Clone, Debug)]
enum FunctionType {
    NoFunction,
    Function,
    Lambda,
}

pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Resolver {
        Resolver {
            interpreter,
            scopes: vec![],
            current_function: FunctionType::NoFunction,
        }
    }

    pub fn resolve(&mut self, statements: &Vec<Box<Stmt>>) -> Result<()> {
        self.resolve_statements(statements)
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn resolve_statements(&mut self, statements: &Vec<Box<Stmt>>) -> Result<()> {
        for statement in statements {
            self.resolve_statement(statement)?;
        }
        Ok(())
    }

    fn resolve_statement(&mut self, statement: &Box<Stmt>) -> Result<()> {
        statement.accept(self)
    }

    fn resolve_expression(&mut self, expression: &Box<Expr>) -> Result<()> {
        expression.accept(self)
    }

    fn declare(&mut self, name: &Token) -> Result<()> {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(&name.lexeme) {
                return Err(error::ResolveError::new(
                    Some(name.clone()),
                    &format!(
                        "Variable named \"{}\" already defined in this scope.",
                        name.lexeme
                    ),
                ));
            }
            scope.insert(name.lexeme.clone(), false);
        }
        Ok(())
    }

    fn define(&mut self, name: &Token) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.lexeme.clone(), true);
        }
    }

    fn resolve_local(&mut self, variable: &Expr, name: &Token) -> Result<()> {
        for i in (0..self.scopes.len()).rev() {
            if self.scopes[i].contains_key(&name.lexeme) {
                self.interpreter
                    .resolve_local(variable, self.scopes.len() - 1 - i);
                return Ok(());
            }
        }
        // not found, assume var is global.
        Ok(())
    }

    fn resolve_function(
        &mut self,
        parameters: &Vec<Token>,
        body: &Vec<Box<Stmt>>,
        function_type: FunctionType,
    ) -> Result<()> {
        let enclosing_function = self.current_function;
        self.current_function = function_type;

        self.begin_scope();
        for param in parameters {
            self.declare(param)?;
            self.define(param);
        }
        let r = self.resolve_statements(body);

        self.end_scope();
        self.current_function = enclosing_function;

        r
    }
}

impl<'a> ExprVisitor<Result<()>> for Resolver<'a> {
    fn visit_assign_expr(&mut self, expr: &Expr, name: &Token, value: &Box<Expr>) -> Result<()> {
        self.resolve_expression(value)?;
        self.resolve_local(expr, name)
    }

    fn visit_binary_expr(
        &mut self,
        _expr: &Expr,
        left: &Box<Expr>,
        _operator: &Token,
        right: &Box<Expr>,
    ) -> Result<()> {
        self.resolve_expression(left)?;
        self.resolve_expression(right)
    }

    fn visit_call_expr(
        &mut self,
        _expr: &Expr,
        callee: &Box<Expr>,
        _paren: &Token,
        arguments: &Vec<Box<Expr>>,
    ) -> Result<()> {
        self.resolve_expression(callee)?;
        for arg in arguments {
            self.resolve_expression(arg)?;
        }
        Ok(())
    }

    fn visit_grouping_expr(&mut self, _expr: &Expr, content: &Box<Expr>) -> Result<()> {
        self.resolve_expression(content)
    }

    fn visit_lambda_expr(
        &mut self,
        _expr: &Expr,
        parameters: &Vec<Token>,
        body: &Vec<Box<Stmt>>,
    ) -> Result<()> {
        self.resolve_function(parameters, body, FunctionType::Lambda)
    }

    fn visit_literal_expr(
        &mut self,
        _expr: &Expr,
        _literal: &crate::scanner::Literal,
    ) -> Result<()> {
        Ok(())
    }

    fn visit_logical_expr(
        &mut self,
        _expr: &Expr,
        left: &Box<Expr>,
        _operator: &Token,
        right: &Box<Expr>,
    ) -> Result<()> {
        self.resolve_expression(left)?;
        self.resolve_expression(right)
    }

    fn visit_ternary_expr(
        &mut self,
        _expr: &Expr,
        condition: &Box<Expr>,
        then_value: &Box<Expr>,
        else_value: &Box<Expr>,
    ) -> Result<()> {
        self.resolve_expression(condition)?;
        self.resolve_expression(then_value)?;
        self.resolve_expression(else_value)
    }

    fn visit_unary_expr(
        &mut self,
        _expr: &Expr,
        _operator: &Token,
        right: &Box<Expr>,
    ) -> Result<()> {
        self.resolve_expression(right)
    }

    fn visit_variable_expr(&mut self, expr: &Expr, name: &Token) -> Result<()> {
        if let Some(scope) = self.scopes.last() {
            if let Some(is_defined) = scope.get(&name.lexeme) {
                if !is_defined {
                    let e = error::ResolveError::new(
                        Some(name.clone()),
                        "Cannot read local variable in its own initializer.",
                    );
                    return Err(e);
                }
            }
        }
        self.resolve_local(&expr, name)
    }
}

impl<'a> StmtVisitor<Result<()>> for Resolver<'a> {
    fn visit_block_stmt(&mut self, _stmt: &Stmt, statements: &Vec<Box<Stmt>>) -> Result<()> {
        self.begin_scope();
        self.resolve_statements(statements)?;
        self.end_scope();
        Ok(())
    }

    fn visit_break_stmt(&mut self, _stmt: &Stmt) -> Result<()> {
        Ok(())
    }

    fn visit_expression_stmt(&mut self, _stmt: &Stmt, expression: &Box<Expr>) -> Result<()> {
        self.resolve_expression(expression)
    }

    fn visit_function_stmt(
        &mut self,
        _stmt: &Stmt,
        name: &Token,
        parameters: &Vec<Token>,
        body: &Vec<Box<Stmt>>,
    ) -> Result<()> {
        self.declare(name)?;
        self.define(name);
        self.resolve_function(parameters, body, FunctionType::Function)
    }

    fn visit_if_stmt(
        &mut self,
        _stmt: &Stmt,
        condition: &Box<Expr>,
        then_branch: &Box<Stmt>,
        else_branch: &Option<Box<Stmt>>,
    ) -> Result<()> {
        self.resolve_expression(condition)?;
        self.resolve_statement(then_branch)?;
        if let Some(else_branch) = else_branch {
            self.resolve_statement(else_branch)?;
        }
        Ok(())
    }

    fn visit_print_stmt(&mut self, _stmt: &Stmt, expression: &Box<Expr>) -> Result<()> {
        self.resolve_expression(expression)
    }

    fn visit_return_stmt(
        &mut self,
        _stmt: &Stmt,
        keyword: &Token,
        value: &Option<Box<Expr>>,
    ) -> Result<()> {
        match self.current_function {
            FunctionType::NoFunction => Err(error::ResolveError::new(
                Some(keyword.clone()),
                "Cannot return from top-level code.",
            )),
            _ => {
                if let Some(value) = value {
                    self.resolve_expression(value)?;
                }
                Ok(())
            }
        }
    }

    fn visit_var_stmt(
        &mut self,
        _stmt: &Stmt,
        name: &Token,
        initializer: &Option<Box<Expr>>,
    ) -> Result<()> {
        self.declare(name)?;
        if let Some(initializer) = initializer {
            self.resolve_expression(initializer)?;
        }
        self.define(name);
        Ok(())
    }

    fn visit_while_stmt(
        &mut self,
        _stmt: &Stmt,
        condition: &Box<Expr>,
        body: &Box<Stmt>,
    ) -> Result<()> {
        self.resolve_expression(condition)?;
        self.resolve_statement(body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use crate::scanner;

    fn verify(program: &str, expected_error:bool) {
        let mut scanner = scanner::Scanner::new(program);
        let tokens = scanner.scan_tokens();
        let mut parser = parser::Parser::new(tokens);
        let ast = parser.parse().unwrap();
        let mut interpreter = Interpreter::new();
        let mut resolver = Resolver::new(&mut interpreter);
        if expected_error {
            assert!(resolver.resolve(&ast).is_err());
        } else {
            assert!(resolver.resolve(&ast).is_ok());
        }
    }

    #[test]
    fn return_outside_function_is_error() {
        let inputs = vec![
            (r#"
            var a = 10;
            return "hello"; // return at global scope
            "#, true),
            (r#"
            var a = 10;
            fun foo() {
                return "hello"; // this is OK
            }
            "#, false)
        ];

        for (program, is_error) in inputs {
            verify(program, is_error);
        }
    }

    #[test]
    fn redefined_variable_is_error() {
        let inputs = vec![
            (r#"
            var a = 10;
            var a = 20; // OK to redefine at global scope
            "#, false),
            (r#"
            fun foo() {
                var a = 10;
                var a = 11;
            }
            "#, true)
        ];

        for (program, is_error) in inputs {
            verify(program, is_error);
        }
    }
}