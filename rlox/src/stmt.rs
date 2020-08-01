use crate::expr::*;
use crate::scanner::Token;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Stmt {
    Expression {
        expression: Box<Expr>,
    },
    Print {
        expression: Box<Expr>,
    },
    Var {
        name: Token,
        initializer: Option<Box<Expr>>,
    },
}

impl Stmt {
    pub fn accept<T, R>(&self, visitor: &mut T) -> R
    where
        T: StmtVisitor<R>,
    {
        match self {
            Stmt::Expression { expression } => visitor.visit_expression_stmt(&expression),
            Stmt::Print { expression } => visitor.visit_print_stmt(&expression),
            Stmt::Var { name, initializer } => visitor.visit_var_stmt(&name, &initializer),
        }
    }
}

// -----------------------------------------------------------------------

pub trait StmtVisitor<R> {
    fn visit_expression_stmt(&mut self, expression: &Box<Expr>) -> R;
    fn visit_print_stmt(&mut self, expression: &Box<Expr>) -> R;
    fn visit_var_stmt(&mut self, name: &Token, initializer: &Option<Box<Expr>>) -> R;
}
