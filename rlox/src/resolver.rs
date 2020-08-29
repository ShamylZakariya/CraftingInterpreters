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

#[derive(Copy, Clone, Debug)]
enum VariableState {
    Declared,
    Defined,
    Accessed,
}

struct Variable {
    state: VariableState,
    token: Token,
}

impl Variable {
    fn new(token: &Token) -> Self {
        Variable {
            state: VariableState::Declared,
            token: token.clone(),
        }
    }

    fn mark_defined(&mut self) {
        self.state = VariableState::Defined;
    }

    fn mark_accessed(&mut self) {
        self.state = VariableState::Accessed;
    }

    fn is_defined(&self) -> bool {
        match self.state {
            VariableState::Declared => false,
            _ => true,
        }
    }

    fn is_accessed(&self) -> bool {
        match self.state {
            VariableState::Accessed => true,
            _ => false,
        }
    }
}

pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    scopes: Vec<HashMap<String, Variable>>,
    current_function: FunctionType,
    loop_depths: Vec<i32>,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Resolver {
        Resolver {
            interpreter,
            scopes: vec![],
            current_function: FunctionType::NoFunction,
            loop_depths: vec![0],
        }
    }

    pub fn resolve(&mut self, statements: &Vec<Box<Stmt>>) -> Result<()> {
        self.resolve_statements(statements)
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) -> Result<()> {
        if let Some(scope) = self.scopes.last() {
            // look for variable definitions which were never accessed
            for var in scope.values() {
                if !var.is_accessed() {
                    return Err(error::ResolveError::new(
                        Some(var.token.clone()),
                        &format!("Variable \"{}\" defined but never accessed", &var.token.lexeme),
                    ));
                }
            }
        }
        self.scopes.pop();
        Ok(())
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
            // establish variable as defined
            scope.insert(name.lexeme.clone(), Variable::new(name));
        }
        Ok(())
    }

    fn define(&mut self, name: &Token) {
        if let Some(scope) = self.scopes.last_mut() {
            if let Some(variable) = scope.get_mut(&name.lexeme) {
                variable.mark_defined();
            }
        }
    }

    fn resolve_local(&mut self, variable: &Expr, name: &Token) -> Result<()> {
        for i in (0..self.scopes.len()).rev() {
            if let Some(var) = self.scopes[i].get_mut(&name.lexeme) {
                var.mark_accessed();
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

        self.loop_depths.push(0);
        self.begin_scope();
        for param in parameters {
            self.declare(param)?;
            self.define(param);
        }
        self.resolve_statements(body)?;
        self.end_scope()?;
        self.loop_depths.pop();
        self.current_function = enclosing_function;

        Ok(())
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
            if let Some(variable) = scope.get(&name.lexeme) {
                if !variable.is_defined() {
                    return Err(error::ResolveError::new(
                        Some(name.clone()),
                        "Cannot read local variable in its own initializer.",
                    ));
                }
            }
        }
        return self.resolve_local(&expr, name);
    }
}

impl<'a> StmtVisitor<Result<()>> for Resolver<'a> {
    fn visit_block_stmt(&mut self, _stmt: &Stmt, statements: &Vec<Box<Stmt>>) -> Result<()> {
        self.begin_scope();
        self.resolve_statements(statements)?;
        self.end_scope()?;
        Ok(())
    }

    fn visit_break_stmt(&mut self, _stmt: &Stmt, keyword: &Token) -> Result<()> {
        if let Some(loop_depth) = self.loop_depths.last() {
            if *loop_depth == 0 {
                return Err(error::ResolveError::new(
                    Some(keyword.clone()),
                    "Illegal \"break\" statement outside of a loop.",
                ));
            }
        }
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
        if let Some(loop_depth) = self.loop_depths.last_mut() {
            *loop_depth += 1;
        }

        let mut r = self.resolve_expression(condition);
        match r {
            Ok(()) => {
                r = self.resolve_statement(body);
            }
            _ => {}
        }

        if let Some(loop_depth) = self.loop_depths.last_mut() {
            *loop_depth -= 1;
        }

        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use crate::scanner;

    fn verify(program: &str, expected_error: bool) {
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
            (
                r#"
            var a = 10;
            return "hello"; // return at global scope
            "#,
                true,
            ),
            (
                r#"
            var a = 10;
            fun foo() {
                return "hello"; // this is OK
            }
            "#,
                false,
            ),
        ];

        for (program, is_error) in inputs {
            verify(program, is_error);
        }
    }

    #[test]
    fn redefined_variable_is_error() {
        let inputs = vec![
            (
                r#"
            var a = 10;
            var a = 20; // OK to redefine at global scope
            "#,
                false,
            ),
            (
                r#"
            fun foo() {
                var a = 10;
                var a = 11;
            }
            "#,
                true,
            ),
        ];

        for (program, is_error) in inputs {
            verify(program, is_error);
        }
    }

    #[test]
    fn break_outside_loop_is_error() {
        let inputs = vec![
            (
                r#"
                var a = 10;
                break; // no good, outside a loop
                "#,
                true,
            ),
            (
                r#"
                while(true) {
                    break; // OK, inside loop
                }
                "#,
                false,
            ),
            (
                r#"
                for(var i = 0; i < 10; i = i + 1) {
                    break; // OK, inside loop
                }
                "#,
                false,
            ),
            (
                r#"
                fun foo() {
                    break; // no good, not inside a loop.
                }
                for (var i = 0; i < 3; i = i + 1) {
                    foo();
                }
                "#,
                true,
            ),
            (
                r#"
                fun foo() {
                    fun bar() {
                        break; // no good, not inside a loop
                    }
                    for (var i = 0; i < 3; i = i + 1) {
                        bar();
                    }
                }
                foo();
                "#,
                true,
            ),
        ];

        for (program, is_error) in inputs {
            verify(program, is_error);
        }
    }

    #[test]
    fn variable_definition_without_access_is_error() {
        let inputs = vec![
            (
                r#"
                // fine, global scope
                var a = 10;
                var b = 20;
                "#,
                false,
            ),
            (
                r#"
                fun foo() {
                    var c = 3; // not ok, we're not in global scope
                }
                "#,
                true,
            ),
            (
                r#"
                fun foo() {
                    var c = 3;
                    return c; // fine, we access it here
                }
                "#,
                false,
            ),
            (
                r#"
                {
                    var c = 3; // not ok, we're not in global scope
                }
                "#,
                true,
            ),
            (
                r#"
                {
                    var c = 3;
                    print c; // fine, we access it here
                }
                "#,
                false,
            ),
        ];

        for (program, is_error) in inputs {
            verify(program, is_error);
        }
    }

}
