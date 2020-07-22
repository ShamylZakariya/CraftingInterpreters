use crate::parser::scanner::*;

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
        value: crate::parser::scanner::Literal,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
}

pub fn accept<T>(expr: &Box<Expr>, visitor: &T) -> String
where
    T: Visitor,
{
    match &**expr {
        Expr::Binary {
            left,
            operator,
            right,
        } => {
            visitor.visit_binary_expr(&left, &operator, &right)
        }
        Expr::Grouping { expression } => {
            visitor.visit_grouping_expr(&expression)
        }
        Expr::Literal { value } => {
            visitor.visit_literal_expr(&value)
        }
        Expr::Unary { operator, right } => {
            visitor.visit_unary_expr(&operator, &right)
        }
    }
}

pub trait Visitor {
    fn visit_binary_expr(&self, left: &Box<Expr>, operator: &Token, right: &Box<Expr>) -> String;
    fn visit_grouping_expr(&self, expr: &Box<Expr>) -> String;
    fn visit_literal_expr(&self, literal: &crate::parser::scanner::Literal) -> String;
    fn visit_unary_expr(&self, operator: &Token, right: &Box<Expr>) -> String;
}

pub struct AstPrinter;
impl Visitor for AstPrinter {
    fn visit_binary_expr(&self, left: &Box<Expr>, operator: &Token, right: &Box<Expr>) -> String {
        parenthesize(&self, operator.lexeme.as_str(), &vec![left, right])
    }

    fn visit_grouping_expr(&self, expr: &Box<Expr>) -> String {
        parenthesize(&self, "group", &vec![expr])
    }

    fn visit_literal_expr(&self, literal: &crate::parser::scanner::Literal) -> String {
        literal.to_string()
    }

    fn visit_unary_expr(&self, operator: &Token, right: &Box<Expr>) -> String {
        parenthesize(&self, operator.lexeme.as_str(), &vec![right])
    }
}

fn parenthesize(ast_printer: &AstPrinter, name: &str, expressions: &Vec<&Box<Expr>>) -> String {
    let mut sequence = String::from("(");
    sequence.push_str(name);

    for expr in expressions {
        sequence.push_str(" ");
        sequence.push_str(accept(expr, ast_printer).as_str());
    }

    sequence.push_str(")");
    return sequence;
}

pub fn print_ast(expr: &Box<Expr>) -> String {
    let printer = Box::new(AstPrinter);
    accept(expr, &*printer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_ast_works() {
        let root = Box::new(Expr::Binary {
            left: Box::new(Expr::Unary {
                operator: Token::new(TokenType::Minus, String::from("-"), None, 1),
                right: Box::new(Expr::Literal {
                    value: crate::parser::scanner::Literal::Number(123 as f64),
                }),
            }),
            operator: Token::new(TokenType::Star, String::from("*"), None, 1),
            right: Box::new(Expr::Grouping {
                expression: Box::new(Expr::Literal {
                    value: crate::parser::scanner::Literal::Number(45.67),
                }),
            }),
        });
        println!("print_ast: {}", print_ast(&root));
    }
}
