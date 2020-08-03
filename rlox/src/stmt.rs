use crate::expr::*;
use crate::scanner::Token;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Stmt {
    Block {
        statements: Vec<Box<Stmt>>,
    },
    Expression {
        expression: Box<Expr>,
    },
    If {
        condition: Box<Expr>,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    Print {
        expression: Box<Expr>,
    },
    Var {
        name: Token,
        initializer: Option<Box<Expr>>,
    },
    While {
        condition: Box<Expr>,
        body: Box<Stmt>,
    }
}

impl Stmt {
    pub fn accept<T, R>(&self, visitor: &mut T) -> R
    where
        T: StmtVisitor<R>,
    {
        match self {
            Stmt::Block { statements } => visitor.visit_block_stmt(&statements),
            Stmt::Expression { expression } => visitor.visit_expression_stmt(&expression),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => visitor.visit_if_stmt(condition, then_branch, else_branch),
            Stmt::Print { expression } => visitor.visit_print_stmt(&expression),
            Stmt::Var { name, initializer } => visitor.visit_var_stmt(&name, &initializer),
            Stmt::While { condition, body } => visitor.visit_while_stmt(&condition, &body),
        }
    }
}

// -----------------------------------------------------------------------

pub trait StmtVisitor<R> {
    fn visit_block_stmt(&mut self, statements: &Vec<Box<Stmt>>) -> R;
    fn visit_expression_stmt(&mut self, expression: &Box<Expr>) -> R;
    fn visit_if_stmt(
        &mut self,
        condition: &Box<Expr>,
        then_branch: &Box<Stmt>,
        else_branch: &Option<Box<Stmt>>,
    ) -> R;
    fn visit_print_stmt(&mut self, expression: &Box<Expr>) -> R;
    fn visit_var_stmt(&mut self, name: &Token, initializer: &Option<Box<Expr>>) -> R;
    fn visit_while_stmt(&mut self, condition: &Box<Expr>, body: &Box<Stmt>) -> R;
}
