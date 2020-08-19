use std::collections::HashMap;

use crate::error;
use crate::expr::*;
use crate::interpreter::InterpretResult;
use crate::interpreter::{InterpretResultStatus, Interpreter};
use crate::scanner::Token;
use crate::stmt::*;

pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    scopes: Vec<HashMap<String, bool>>,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Resolver {
        Resolver {
            interpreter,
            scopes: vec![],
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }
    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn resolve_statements(&mut self, statements: &Vec<Box<Stmt>>) -> InterpretResult<()> {
        for statement in statements {
            self.resolve_statement(statement)?;
        }
        Ok(())
    }

    fn resolve_statement(&mut self, statement: &Stmt) -> InterpretResult<()> {
        statement.accept(self)
    }

    fn resolve_expression(&mut self, expression: &Expr) -> InterpretResult<()> {
        expression.accept(self)
    }

    fn declare(&mut self, name: &Token) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.lexeme.clone(), false);
        }
    }

    fn define(&mut self, name: &Token) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.lexeme.clone(), true);
        }
    }

    fn resolve_function(
        &mut self,
        _function: Option<&Stmt>,
        parameters: &Vec<Token>,
        body: &Vec<Box<Stmt>>,
    ) -> InterpretResult<()> {
        self.begin_scope();
        for param in parameters {
            self.declare(param);
            self.define(param);
        }
        let r = self.resolve_statements(body);
        self.end_scope();
        r
    }

    fn resolve_local(&mut self, variable: &Expr, name: &Token) -> InterpretResult<()> {
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
}

impl<'a> ExprVisitor<InterpretResult<()>> for Resolver<'a> {
    fn visit_assign_expr(
        &mut self,
        expr: &Expr,
        name: &Token,
        value: &Box<Expr>,
    ) -> InterpretResult<()> {
        self.resolve_expression(value)?;
        self.resolve_local(expr, name)
    }

    fn visit_binary_expr(
        &mut self,
        _expr: &Expr,
        left: &Box<Expr>,
        _operator: &Token,
        right: &Box<Expr>,
    ) -> InterpretResult<()> {
        self.resolve_expression(left)?;
        self.resolve_expression(right)
    }

    fn visit_grouping_expr(&mut self, _expr: &Expr, contents: &Box<Expr>) -> InterpretResult<()> {
        self.resolve_expression(contents)
    }

    fn visit_call_expr(
        &mut self,
        _expr: &Expr,
        callee: &Box<Expr>,
        _paren: &Token,
        arguments: &Vec<Box<Expr>>,
    ) -> InterpretResult<()> {
        self.resolve_expression(callee)?;
        for arg in arguments {
            self.resolve_expression(arg)?;
        }
        Ok(())
    }

    fn visit_lambda_expr(
        &mut self,
        _expr: &Expr,
        parameters: &Vec<Token>,
        body: &Vec<Box<Stmt>>,
    ) -> InterpretResult<()> {
        self.resolve_function(None, parameters, body)
    }

    fn visit_literal_expr(
        &mut self,
        _expr: &Expr,
        _literal: &crate::scanner::Literal,
    ) -> InterpretResult<()> {
        Ok(())
    }

    fn visit_logical_expr(
        &mut self,
        _expr: &Expr,
        left: &Box<Expr>,
        _operator: &Token,
        right: &Box<Expr>,
    ) -> InterpretResult<()> {
        self.resolve_expression(left)?;
        self.resolve_expression(right)
    }

    fn visit_ternary_expr(
        &mut self,
        _expr: &Expr,
        condition: &Box<Expr>,
        then_value: &Box<Expr>,
        else_value: &Box<Expr>,
    ) -> InterpretResult<()> {
        self.resolve_expression(condition)?;
        self.resolve_expression(then_value)?;
        self.resolve_expression(else_value)
    }

    fn visit_unary_expr(
        &mut self,
        _expr: &Expr,
        _operator: &Token,
        right: &Box<Expr>,
    ) -> InterpretResult<()> {
        self.resolve_expression(right)
    }

    fn visit_variable_expr(&mut self, expr: &Expr, name: &Token) -> InterpretResult<()> {
        if let Some(scope) = self.scopes.last() {
            if let Some(is_defined) = scope.get(&name.lexeme) {
                if !is_defined {
                    let e = error::RuntimeError::new(
                        name,
                        "Cannot read local variable in its own initializer.",
                    );
                    error::report::runtime_error(&e);
                    return Err(InterpretResultStatus::Error(e));
                }
            }
        }
        self.resolve_local(&expr, name)
    }
}

impl<'a> StmtVisitor<InterpretResult<()>> for Resolver<'a> {
    fn visit_block_stmt(
        &mut self,
        _stmt: &Stmt,
        statements: &Vec<Box<Stmt>>,
    ) -> InterpretResult<()> {
        self.begin_scope();
        let r = self.resolve_statements(statements);
        self.end_scope();
        r
    }

    fn visit_break_stmt(&mut self, _stmt: &Stmt) -> InterpretResult<()> {
        Ok(())
    }

    fn visit_expression_stmt(
        &mut self,
        _stmt: &Stmt,
        expression: &Box<Expr>,
    ) -> InterpretResult<()> {
        self.resolve_expression(expression)
    }

    fn visit_function_stmt(
        &mut self,
        stmt: &Stmt,
        name: &Token,
        parameters: &Vec<Token>,
        body: &Vec<Box<Stmt>>,
    ) -> InterpretResult<()> {
        self.declare(name);
        self.define(name);
        self.resolve_function(Some(stmt), parameters, body)
    }

    fn visit_if_stmt(
        &mut self,
        _stmt: &Stmt,
        condition: &Box<Expr>,
        then_branch: &Box<Stmt>,
        else_branch: &Option<Box<Stmt>>,
    ) -> InterpretResult<()> {
        self.resolve_expression(condition)?;
        self.resolve_statement(then_branch)?;
        if let Some(else_branch) = else_branch {
            self.resolve_statement(else_branch)
        } else {
            Ok(())
        }
    }

    fn visit_print_stmt(&mut self, _stmt: &Stmt, expression: &Box<Expr>) -> InterpretResult<()> {
        self.resolve_expression(expression)
    }

    fn visit_return_stmt(
        &mut self,
        _stmt: &Stmt,
        _keyword: &Token,
        value: &Option<Box<Expr>>,
    ) -> InterpretResult<()> {
        if let Some(value) = value {
            self.resolve_expression(value)
        } else {
            Ok(())
        }
    }

    fn visit_var_stmt(
        &mut self,
        _stmt: &Stmt,
        name: &Token,
        initializer: &Option<Box<Expr>>,
    ) -> InterpretResult<()> {
        self.declare(name);
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
    ) -> InterpretResult<()> {
        self.resolve_expression(condition)?;
        self.resolve_statement(body)
    }
}
