use crate::expr::*;
use crate::scanner::Token;

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum Stmt {
    Block {
        statements: Vec<Box<Stmt>>,
    },
    Break {
        keyword: Token,
    },
    Expression {
        expression: Box<Expr>,
    },
    Function {
        name: Token,
        parameters: Vec<Token>,
        body: Vec<Box<Stmt>>,
    },
    If {
        condition: Box<Expr>,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    Print {
        expression: Box<Expr>,
    },
    Return {
        keyword: Token,
        value: Option<Box<Expr>>,
    },
    Var {
        name: Token,
        initializer: Option<Box<Expr>>,
    },
    While {
        condition: Box<Expr>,
        body: Box<Stmt>,
    },
}

impl Stmt {
    pub fn accept<T, R>(&self, visitor: &mut T) -> R
    where
        T: StmtVisitor<R>,
    {
        match self {
            Stmt::Block { statements } => visitor.visit_block_stmt(&self, statements),
            Stmt::Break { keyword } => visitor.visit_break_stmt(&self, keyword),
            Stmt::Expression { expression } => visitor.visit_expression_stmt(&self, &expression),
            Stmt::Function {
                name,
                parameters,
                body,
            } => visitor.visit_function_stmt(&self, &name, &parameters, &body),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => visitor.visit_if_stmt(&self, condition, then_branch, else_branch),
            Stmt::Print { expression } => visitor.visit_print_stmt(&self, &expression),
            Stmt::Return { keyword, value } => visitor.visit_return_stmt(&self, &keyword, &value),
            Stmt::Var { name, initializer } => visitor.visit_var_stmt(&self, &name, &initializer),
            Stmt::While { condition, body } => visitor.visit_while_stmt(&self, &condition, &body),
        }
    }
}

// -----------------------------------------------------------------------

pub trait StmtVisitor<R> {
    fn visit_block_stmt(&mut self, stmt: &Stmt, statements: &Vec<Box<Stmt>>) -> R;
    fn visit_break_stmt(&mut self, stmt: &Stmt, keyword: &Token) -> R;
    fn visit_expression_stmt(&mut self, stmt: &Stmt, expression: &Box<Expr>) -> R;
    fn visit_function_stmt(
        &mut self,
        stmt: &Stmt,
        name: &Token,
        parameters: &Vec<Token>,
        body: &Vec<Box<Stmt>>,
    ) -> R;
    fn visit_if_stmt(
        &mut self,
        stmt: &Stmt,
        condition: &Box<Expr>,
        then_branch: &Box<Stmt>,
        else_branch: &Option<Box<Stmt>>,
    ) -> R;
    fn visit_print_stmt(&mut self, stmt: &Stmt, expression: &Box<Expr>) -> R;
    fn visit_return_stmt(&mut self, stmt: &Stmt, keyword: &Token, value: &Option<Box<Expr>>) -> R;
    fn visit_var_stmt(&mut self, stmt: &Stmt, name: &Token, initializer: &Option<Box<Expr>>) -> R;
    fn visit_while_stmt(&mut self, stmt: &Stmt, condition: &Box<Expr>, body: &Box<Stmt>) -> R;
}
