use crate::expr::*;
use crate::scanner::*;
use crate::stmt::*;

pub struct AstPrinter;

impl AstPrinter {
    pub fn new() -> Self {
        AstPrinter {
        }
    }

    pub fn generate(&mut self, statements: &Vec<Box<Stmt>>) -> String {
        let mut buffer = String::new();
        for statement in statements {
            buffer.push_str(statement.accept(self).as_str());
        }

        buffer
    }
}

impl ExprVisitor<String> for AstPrinter {
    fn visit_assign_expr(&mut self, __expr: &Expr, name: &Token, value: &Box<Expr>) -> String {
         parenthesize_exprs(self, &name.lexeme, &vec![value])
    }
    fn visit_binary_expr(
        &mut self,
        _expr: &Expr,
        left: &Box<Expr>,
        operator: &Token,
        right: &Box<Expr>,
    ) -> String {
        parenthesize_exprs(self, &operator.lexeme, &vec![left, right])
    }
    fn visit_call_expr(
        &mut self,
        _expr: &Expr,
        callee: &Box<Expr>,
        _paren: &Token,
        arguments: &Vec<Box<Expr>>,
    ) -> String {
        let mut exprs = vec![];
        for arg in arguments {
            exprs.push(arg);
        }
        exprs.push(callee);
        parenthesize_exprs(self, "call", &exprs)
    }
    fn visit_grouping_expr(&mut self, _expr: &Expr, content: &Box<Expr>) -> String {
        parenthesize_exprs(self, "group", &vec![content])
    }
    fn visit_lambda_expr(
        &mut self,
        _expr: &Expr,
        _parameters: &Vec<Token>,
        body: &Vec<Box<Stmt>>,
    ) -> String {
        parenthesize_statements(self, "lambda", body)
    }
    fn visit_literal_expr(&mut self, _expr: &Expr, literal: &crate::scanner::Literal) -> String {
        literal.to_string()
    }
    fn visit_logical_expr(
        &mut self,
        _expr: &Expr,
        left: &Box<Expr>,
        operator: &Token,
        right: &Box<Expr>,
    ) -> String {
        parenthesize_exprs(self, &operator.lexeme, &vec![left, right])
    }
    fn visit_ternary_expr(
        &mut self,
        _expr: &Expr,
        condition: &Box<Expr>,
        then_value: &Box<Expr>,
        else_value: &Box<Expr>,
    ) -> String {
        parenthesize_exprs(self, "ternary", &vec![condition, then_value, else_value])
    }
    fn visit_unary_expr(&mut self, _expr: &Expr, operator: &Token, right: &Box<Expr>) -> String {
        parenthesize_exprs(self, &operator.lexeme, &vec![right])
    }
    fn visit_variable_expr(&mut self, _expr: &Expr, name: &Token) -> String {
        parenthesize_exprs(self, &format!("variable \"{}\"", name.lexeme), &vec![])
    }
}

impl StmtVisitor<String> for AstPrinter {
    fn visit_block_stmt(&mut self, _stmt: &Stmt, statements: &Vec<Box<Stmt>>) -> String {
        parenthesize_statements(self, "block", statements)
    }
    fn visit_break_stmt(&mut self, _stmt: &Stmt) -> String {
        parenthesize_statements(self, "break", &vec![])
    }
    fn visit_expression_stmt(&mut self, _stmt: &Stmt, expression: &Box<Expr>) -> String {
        parenthesize_exprs(self, "expression", &vec![expression])
    }
    fn visit_function_stmt(
        &mut self,
        _stmt: &Stmt,
        name: &Token,
        _parameters: &Vec<Token>,
        body: &Vec<Box<Stmt>>,
    ) -> String {
        parenthesize_statements(self, &format!("fn \"{}\"", name.lexeme), body)
    }
    fn visit_if_stmt(
        &mut self,
        _stmt: &Stmt,
        condition: &Box<Expr>,
        then_branch: &Box<Stmt>,
        else_branch: &Option<Box<Stmt>>,
    ) -> String {
        let mut statements = vec![then_branch.clone()];
        if let Some(else_branch) = else_branch {
            statements.push(else_branch.clone())
        }
        let name = parenthesize_exprs(self, "if", &vec![condition]);
        parenthesize_statements(self, &name, &statements)
    }

    fn visit_print_stmt(&mut self, _stmt: &Stmt, expression: &Box<Expr>) -> String {
        parenthesize_exprs(self, "print", &vec![expression])
    }

    fn visit_return_stmt(&mut self, _stmt: &Stmt, _keyword: &Token, value: &Option<Box<Expr>>) -> String {
        if let Some(expr) = value {
            parenthesize_exprs(self, "return", &vec![expr])
        } else {
            parenthesize_exprs(self, "return", &vec![])
        }
    }

    fn visit_var_stmt(&mut self, _stmt: &Stmt, _name: &Token, initializer: &Option<Box<Expr>>) -> String {
        if let Some(expr) = initializer {
            parenthesize_exprs(self, "var", &vec![expr])
        } else {
            parenthesize_exprs(self, "var", &vec![])
        }
    }

    fn visit_while_stmt(&mut self, _stmt: &Stmt, condition: &Box<Expr>, body: &Box<Stmt>) -> String {
        let name = parenthesize_exprs(self, "while", &vec![condition]);
        parenthesize_statements(self, &name, &vec![body.clone()])
    }
}

fn parenthesize_exprs(ast_printer: &mut AstPrinter, name: &str, expressions: &Vec<&Box<Expr>>) -> String {
    let mut sequence = String::from("(");
    sequence.push_str(name);

    for expr in expressions {
        sequence.push_str(" ");
        sequence.push_str(expr.accept(ast_printer).as_str());
    }

    sequence.push_str(")");
    return sequence;
}

fn parenthesize_statements(ast_printer: &mut AstPrinter, name: &str, statements: &Vec<Box<Stmt>>) -> String {
    let mut sequence = String::from("(");
    sequence.push_str(name);

    for stmt in statements {
        sequence.push_str(" ");
        sequence.push_str(stmt.accept(ast_printer).as_str());
    }

    sequence.push_str(")");
    return sequence;
}

