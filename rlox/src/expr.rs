use crate::scanner::*;

#[derive(Clone, PartialEq, Eq, Debug)]
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
    Variable {
        name: Token,
    },
}

impl Expr {
    pub fn accept<T, R>(&self, visitor: &mut T) -> R
    where
        T: ExprVisitor<R>,
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
            Expr::Variable { name } => visitor.visit_variable_expr(&name),
        }
    }
}

// -----------------------------------------------------------------------

pub trait ExprVisitor<R> {
    fn visit_binary_expr(&mut self, left: &Box<Expr>, operator: &Token, right: &Box<Expr>) -> R;
    fn visit_grouping_expr(&mut self, expr: &Box<Expr>) -> R;
    fn visit_literal_expr(&mut self, literal: &crate::scanner::Literal) -> R;
    fn visit_unary_expr(&mut self, operator: &Token, right: &Box<Expr>) -> R;
    fn visit_variable_expr(&mut self, name: &Token) -> R;
}
