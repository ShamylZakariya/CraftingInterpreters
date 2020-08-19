use crate::scanner::*;
use crate::stmt::Stmt;

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
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
    Call {
        callee: Box<Expr>,
        paren: Token, // this is to record line number
        arguments: Vec<Box<Expr>>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Lambda {
        parameters: Vec<Token>,
        body: Vec<Box<Stmt>>,
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
            Expr::Assign { name, value } => visitor.visit_assign_expr(&self, &name, &value),
            Expr::Binary {
                left,
                operator,
                right,
            } => visitor.visit_binary_expr(&self, &left, &operator, &right),
            Expr::Call {
                callee,
                paren,
                arguments,
            } => visitor.visit_call_expr(&self, &callee, &paren, &arguments),
            Expr::Grouping { expression } => visitor.visit_grouping_expr(&self, &expression),
            Expr::Lambda { parameters, body } => visitor.visit_lambda_expr(&self, &parameters, &body),
            Expr::Literal { value } => visitor.visit_literal_expr(&self, &value),
            Expr::Logical {
                left,
                operator,
                right,
            } => visitor.visit_logical_expr(&self, &left, &operator, &right),
            Expr::Ternary {
                condition,
                then_value,
                else_value,
            } => visitor.visit_ternary_expr(&self, condition, then_value, else_value),
            Expr::Unary { operator, right } => visitor.visit_unary_expr(&self, &operator, &right),
            Expr::Variable { name } => visitor.visit_variable_expr(&self, &name),
        }
    }
}

// -----------------------------------------------------------------------

pub trait ExprVisitor<R> {
    fn visit_assign_expr(&mut self, expr:&Expr, name: &Token, value: &Box<Expr>) -> R;
    fn visit_binary_expr(&mut self, expr:&Expr, left: &Box<Expr>, operator: &Token, right: &Box<Expr>) -> R;
    fn visit_grouping_expr(&mut self, expr:&Expr, content: &Box<Expr>) -> R;
    fn visit_call_expr(
        &mut self,
        expr: &Expr,
        callee: &Box<Expr>,
        paren: &Token,
        arguments: &Vec<Box<Expr>>,
    ) -> R;
    fn visit_lambda_expr(&mut self, expr:&Expr, parameters: &Vec<Token>, body: &Vec<Box<Stmt>>) -> R;
    fn visit_literal_expr(&mut self, expr: &Expr, literal: &crate::scanner::Literal) -> R;
    fn visit_logical_expr(&mut self, expr: &Expr, left: &Box<Expr>, operator: &Token, right: &Box<Expr>) -> R;
    fn visit_ternary_expr(
        &mut self,
        expr: &Expr,
        condition: &Box<Expr>,
        then_value: &Box<Expr>,
        else_value: &Box<Expr>,
    ) -> R;
    fn visit_unary_expr(&mut self, expr: &Expr, operator: &Token, right: &Box<Expr>) -> R;
    fn visit_variable_expr(&mut self, expr: &Expr, name: &Token) -> R;
}
