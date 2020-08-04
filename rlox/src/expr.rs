use crate::scanner::*;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Expr {
    Assign {
        name: Token,
        value: Box<Expr>,
    },
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
    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Ternary {
        condition: Box<Expr>,
        then_value: Box<Expr>,
        else_value: Box<Expr>,
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
            Expr::Assign { name, value } => visitor.visit_assign_expr(&name, &value),
            Expr::Binary {
                left,
                operator,
                right,
            } => visitor.visit_binary_expr(&left, &operator, &right),
            Expr::Grouping { expression } => visitor.visit_grouping_expr(&expression),
            Expr::Literal { value } => visitor.visit_literal_expr(&value),
            Expr::Logical {
                left,
                operator,
                right,
            } => visitor.visit_logical_expr(&left, &operator, &right),
            Expr::Ternary {
                condition,
                then_value,
                else_value,
            } => visitor.visit_ternary_expr(condition, then_value, else_value),
            Expr::Unary { operator, right } => visitor.visit_unary_expr(&operator, &right),
            Expr::Variable { name } => visitor.visit_variable_expr(&name),
        }
    }
}

// -----------------------------------------------------------------------

pub trait ExprVisitor<R> {
    fn visit_assign_expr(&mut self, name: &Token, value: &Box<Expr>) -> R;
    fn visit_binary_expr(&mut self, left: &Box<Expr>, operator: &Token, right: &Box<Expr>) -> R;
    fn visit_grouping_expr(&mut self, expr: &Box<Expr>) -> R;
    fn visit_literal_expr(&mut self, literal: &crate::scanner::Literal) -> R;
    fn visit_logical_expr(&mut self, left: &Box<Expr>, operator: &Token, right: &Box<Expr>) -> R;
    fn visit_ternary_expr(
        &mut self,
        condition: &Box<Expr>,
        then_value: &Box<Expr>,
        else_value: &Box<Expr>,
    ) -> R;
    fn visit_unary_expr(&mut self, operator: &Token, right: &Box<Expr>) -> R;
    fn visit_variable_expr(&mut self, name: &Token) -> R;
}
