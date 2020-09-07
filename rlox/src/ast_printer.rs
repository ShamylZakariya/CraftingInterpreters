use crate::ast::*;
use crate::scanner::*;

pub struct AstPrinter;

impl AstPrinter {
    pub fn new() -> Self {
        AstPrinter {}
    }

    pub fn generate(&mut self, statements: &Vec<Box<Stmt>>) -> String {
        let mut buffer = String::new();
        for statement in statements {
            buffer.push_str(statement.accept(self).as_str());
        }

        buffer
    }

    fn parenthesize_exprs(&mut self, name: &str, expressions: &Vec<&Box<Expr>>) -> String {
        let mut sequence = String::from("(");
        sequence.push_str(name);

        for expr in expressions {
            sequence.push_str(" ");
            sequence.push_str(expr.accept(self).as_str());
        }

        // sequence.push_str("\n");
        sequence.push_str(")");
        return sequence;
    }

    fn parenthesize_stmts(&mut self, name: &str, statements: &Vec<Box<Stmt>>) -> String {
        let mut sequence = String::from("(");
        sequence.push_str(name);

        for stmt in statements {
            sequence.push_str(" ");
            sequence.push_str(stmt.accept(self).as_str());
        }

        // sequence.push_str("\n");
        sequence.push_str(")");
        return sequence;
    }
}

impl ExprVisitor<String> for AstPrinter {
    fn visit_assign_expr(&mut self, __expr: &Expr, name: &Token, value: &Box<Expr>) -> String {
        let name = format!("assign \"{}\"", name.lexeme);
        self.parenthesize_exprs(&name, &vec![value])
    }

    fn visit_binary_expr(
        &mut self,
        _expr: &Expr,
        left: &Box<Expr>,
        operator: &Token,
        right: &Box<Expr>,
    ) -> String {
        self.parenthesize_exprs(&operator.lexeme, &vec![left, right])
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
        self.parenthesize_exprs("call", &exprs)
    }

    fn visit_grouping_expr(&mut self, _expr: &Expr, content: &Box<Expr>) -> String {
        self.parenthesize_exprs("group", &vec![content])
    }

    fn visit_lambda_expr(
        &mut self,
        _expr: &Expr,
        _parameters: &Vec<Token>,
        body: &Vec<Box<Stmt>>,
    ) -> String {
        self.parenthesize_stmts("lambda", body)
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
        self.parenthesize_exprs(&operator.lexeme, &vec![left, right])
    }

    fn visit_ternary_expr(
        &mut self,
        _expr: &Expr,
        condition: &Box<Expr>,
        then_value: &Box<Expr>,
        else_value: &Box<Expr>,
    ) -> String {
        self.parenthesize_exprs("ternary", &vec![condition, then_value, else_value])
    }

    fn visit_unary_expr(&mut self, _expr: &Expr, operator: &Token, right: &Box<Expr>) -> String {
        self.parenthesize_exprs(&operator.lexeme, &vec![right])
    }

    fn visit_variable_expr(&mut self, _expr: &Expr, name: &Token) -> String {
        self.parenthesize_exprs(&format!("var_expr \"{}\"", name.lexeme), &vec![])
    }
}

impl StmtVisitor<String> for AstPrinter {
    fn visit_block_stmt(&mut self, _stmt: &Stmt, statements: &Vec<Box<Stmt>>) -> String {
        self.parenthesize_stmts("block", statements)
    }

    fn visit_break_stmt(&mut self, _stmt: &Stmt, _keyword: &Token) -> String {
        self.parenthesize_stmts("break", &vec![])
    }

    fn visit_class_stmt(&mut self, _stmt: &Stmt, name: &Token, methods: &Vec<Box<Stmt>>) -> String {
        self.parenthesize_stmts(&name.lexeme, methods)
    }

    fn visit_expression_stmt(&mut self, _stmt: &Stmt, expression: &Box<Expr>) -> String {
        self.parenthesize_exprs("expression", &vec![expression])
    }

    fn visit_function_stmt(
        &mut self,
        _stmt: &Stmt,
        name: &Token,
        _parameters: &Vec<Token>,
        body: &Vec<Box<Stmt>>,
    ) -> String {
        self.parenthesize_stmts(&format!("fn \"{}\"", name.lexeme), body)
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
        let name = self.parenthesize_exprs("if", &vec![condition]);
        self.parenthesize_stmts(&name, &statements)
    }

    fn visit_print_stmt(&mut self, _stmt: &Stmt, expression: &Box<Expr>) -> String {
        self.parenthesize_exprs("print", &vec![expression])
    }

    fn visit_return_stmt(
        &mut self,
        _stmt: &Stmt,
        _keyword: &Token,
        value: &Option<Box<Expr>>,
    ) -> String {
        if let Some(expr) = value {
            self.parenthesize_exprs("return", &vec![expr])
        } else {
            self.parenthesize_exprs("return", &vec![])
        }
    }

    fn visit_var_stmt(
        &mut self,
        _stmt: &Stmt,
        name: &Token,
        initializer: &Option<Box<Expr>>,
    ) -> String {
        let name = format!("var_stmt \"{}\"", name.lexeme);
        if let Some(expr) = initializer {
            self.parenthesize_exprs(&name, &vec![expr])
        } else {
            self.parenthesize_exprs(&name, &vec![])
        }
    }

    fn visit_while_stmt(
        &mut self,
        _stmt: &Stmt,
        condition: &Box<Expr>,
        body: &Box<Stmt>,
    ) -> String {
        let name = self.parenthesize_exprs("while", &vec![condition]);
        self.parenthesize_stmts(&name, &vec![body.clone()])
    }
}
