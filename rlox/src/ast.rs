use crate::scanner::*;
use std::fmt;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum CallableType {
    ClassMethod,
    Function,
    Lambda,
    Method,
    Property,
}

impl fmt::Display for CallableType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CallableType::ClassMethod => write!(f, "class_method"),
            CallableType::Function => write!(f, "function"),
            CallableType::Lambda => write!(f, "lambda"),
            CallableType::Method => write!(f, "method"),
            CallableType::Property => write!(f, "property"),
        }
    }
}

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
    Get {
        object: Box<Expr>,
        name: Token,
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
    Set {
        object: Box<Expr>,
        name: Token,
        value: Box<Expr>,
    },
    Ternary {
        condition: Box<Expr>,
        then_value: Box<Expr>,
        else_value: Box<Expr>,
    },
    This {
        keyword: Token,
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
            Expr::Get { object, name } => visitor.visit_get_expr(&self, object, name),
            Expr::Grouping { expression } => visitor.visit_grouping_expr(&self, &expression),
            Expr::Lambda { parameters, body } => {
                visitor.visit_lambda_expr(&self, &parameters, &body)
            }
            Expr::Literal { value } => visitor.visit_literal_expr(&self, &value),
            Expr::Logical {
                left,
                operator,
                right,
            } => visitor.visit_logical_expr(&self, &left, &operator, &right),
            Expr::Set {
                object,
                name,
                value,
            } => visitor.visit_set_expr(&self, object, name, value),
            Expr::This { keyword } => visitor.visit_this_expr(&self, &keyword),
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

// -----------------------------------------------------------------------------

pub trait ExprVisitor<R> {
    fn visit_assign_expr(&mut self, expr: &Expr, name: &Token, value: &Box<Expr>) -> R;
    fn visit_binary_expr(
        &mut self,
        expr: &Expr,
        left: &Box<Expr>,
        operator: &Token,
        right: &Box<Expr>,
    ) -> R;
    fn visit_call_expr(
        &mut self,
        expr: &Expr,
        callee: &Box<Expr>,
        paren: &Token,
        arguments: &Vec<Box<Expr>>,
    ) -> R;
    fn visit_get_expr(&mut self, expr: &Expr, object: &Box<Expr>, name: &Token) -> R;
    fn visit_grouping_expr(&mut self, expr: &Expr, content: &Box<Expr>) -> R;
    fn visit_lambda_expr(
        &mut self,
        expr: &Expr,
        parameters: &Vec<Token>,
        body: &Vec<Box<Stmt>>,
    ) -> R;
    fn visit_literal_expr(&mut self, expr: &Expr, literal: &crate::scanner::Literal) -> R;
    fn visit_logical_expr(
        &mut self,
        expr: &Expr,
        left: &Box<Expr>,
        operator: &Token,
        right: &Box<Expr>,
    ) -> R;
    fn visit_set_expr(
        &mut self,
        expr: &Expr,
        object: &Box<Expr>,
        name: &Token,
        value: &Box<Expr>,
    ) -> R;
    fn visit_this_expr(&mut self, expr: &Expr, keyword: &Token) -> R;
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

// -----------------------------------------------------------------------------

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum Stmt {
    Block {
        statements: Vec<Box<Stmt>>,
    },
    Break {
        keyword: Token,
    },
    Class {
        name: Token,
        methods: Vec<Box<Stmt>>,
        class_methods: Vec<Box<Stmt>>,
    },
    Expression {
        expression: Box<Expr>,
    },
    Function {
        name: Token,
        parameters: Vec<Token>,
        body: Vec<Box<Stmt>>,
        fn_type: CallableType,
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
            Stmt::Class {
                name,
                methods,
                class_methods,
            } => visitor.visit_class_stmt(&self, name, methods, class_methods),
            Stmt::Expression { expression } => visitor.visit_expression_stmt(&self, &expression),
            Stmt::Function {
                name,
                parameters,
                body,
                fn_type,
            } => visitor.visit_function_stmt(&self, &name, &parameters, &body, *fn_type),
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
    fn visit_class_stmt(
        &mut self,
        stmt: &Stmt,
        name: &Token,
        methods: &Vec<Box<Stmt>>,
        class_methods: &Vec<Box<Stmt>>,
    ) -> R;
    fn visit_expression_stmt(&mut self, stmt: &Stmt, expression: &Box<Expr>) -> R;
    fn visit_function_stmt(
        &mut self,
        stmt: &Stmt,
        name: &Token,
        parameters: &Vec<Token>,
        body: &Vec<Box<Stmt>>,
        fn_type: CallableType,
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
