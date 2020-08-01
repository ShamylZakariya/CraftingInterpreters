use crate::scanner::*;

pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Literal {
        value: crate::scanner::Literal,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
}

impl Expr {
    pub fn accept<T, R>(&self, visitor: &T) -> R
    where
        T: Visitor<R>,
    {
        match self {
            Expr::Binary {
                left,
                operator,
                right,
            } => visitor.visit_binary_expr(&left, &operator, &right),
            Expr::Grouping { expression } => visitor.visit_grouping_expr(&expression),
            Expr::Literal { value } => visitor.visit_literal_expr(&value),
            Expr::Unary { operator, right } => visitor.visit_unary_expr(&operator, &right),
        }
    }
}

pub trait Visitor<R> {
    fn visit_binary_expr(&self, left: &Box<Expr>, operator: &Token, right: &Box<Expr>) -> R;
    fn visit_grouping_expr(&self, expr: &Box<Expr>) -> R;
    fn visit_literal_expr(&self, literal: &crate::scanner::Literal) -> R;
    fn visit_unary_expr(&self, operator: &Token, right: &Box<Expr>) -> R;
}