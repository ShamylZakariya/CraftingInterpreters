use crate::parser::expr::*;
use crate::parser::scanner::*;

struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens: tokens,
            current: 0,
        }
    }

    // Recursive descent, ordered by associativity
    fn primary(&mut self) -> Box<Expr> {
        if self.match_token(&vec![TokenType::False]) {
            return Box::new(Expr::Literal {
                value: crate::parser::scanner::Literal::False,
            });
        }
        if self.match_token(&vec![TokenType::True]) {
            return Box::new(Expr::Literal {
                value: crate::parser::scanner::Literal::True,
            });
        }
        if self.match_token(&vec![TokenType::Nil]) {
            return Box::new(Expr::Literal {
                value: crate::parser::scanner::Literal::Nil,
            });
        }
        if self.match_token(&vec![TokenType::Number, TokenType::Str]) {
            if let Some(l) = self.previous().clone().literal {
                return Box::new(Expr::Literal { value: l });
            }
        }
        if self.match_token(&vec![TokenType::LeftParen]) {
            let expr = self.expression();
            self.consume(TokenType::RightParen, "Expect ')' after expression.");
            return Box::new(Expr::Grouping { expression: expr });
        }

        panic!("Shouldn't make it here in primary()");
        // make compiler happy
        return Box::new(Expr::Literal {
            value: crate::parser::scanner::Literal::False,
        });
    }

    fn unary(&mut self) -> Box<Expr> {
        if self.match_token(&vec![TokenType::Bang, TokenType::Minus]) {
            let op = self.previous().clone();
            let right = self.unary();
            return Box::new(Expr::Unary {
                operator: op,
                right: right,
            });
        }
        self.primary()
    }

    fn multiplication(&mut self) -> Box<Expr> {
        let mut expr = self.unary();
        while self.match_token(&vec![TokenType::Slash, TokenType::Star]) {
            let op = self.previous().clone();
            let right = self.unary();
            expr = Box::new(Expr::Binary {
                left: expr,
                operator: op,
                right: right,
            })
        }
        expr
    }

    fn addition(&mut self) -> Box<Expr> {
        let mut expr = self.multiplication();
        while self.match_token(&vec![TokenType::Minus, TokenType::Plus]) {
            let op = self.previous().clone();
            let right = self.multiplication();
            expr = Box::new(Expr::Binary {
                left: expr,
                operator: op,
                right: right,
            })
        }
        expr
    }

    fn comparison(&mut self) -> Box<Expr> {
        let mut expr = self.addition();
        while self.match_token(&vec![
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let op = self.previous().clone();
            let right = self.addition();
            expr = Box::new(Expr::Binary {
                left: expr,
                operator: op,
                right: right,
            })
        }
        expr
    }

    fn equality(&mut self) -> Box<Expr> {
        let mut expr = self.comparison();
        while self.match_token(&vec![TokenType::BangEqual, TokenType::EqualEqual]) {
            let op = self.previous().clone();
            let right = self.comparison();
            expr = Box::new(Expr::Binary {
                left: expr,
                operator: op,
                right: right,
            });
        }
        expr
    }

    fn expression(&mut self) -> Box<Expr> {
        self.equality()
    }

    // Helpers

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn check(&self, t: TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.peek().token_type == t
        }
    }

    fn consume(&mut self, token_type: TokenType, on_error_message: &str) -> &Token {
        if self.check(token_type) {
            return self.advance();
        }

        // can't throw, so we need to start propagating errors up
        todo!();
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    fn match_token(&mut self, types: &Vec<TokenType>) -> bool {
        for t in types {
            if self.check(*t) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_expressions() {
        let expressions = vec![
            "1 + (5/2)",
            "1 + 2 + 3 + 4",
            "5 == nil",
            "1 >= 3",
            "nil < 10",
            "false == true;",
        ];

        let expected_asts = vec![
            "(+ 1 (group (/ 5 2)))",
            "(+ (+ (+ 1 2) 3) 4)",
            "(== 5 Nil)",
            "(>= 1 3)",
            "(< Nil 10)",
            "(== False True)",
        ];

        for (expression, expected_ast) in expressions.iter().zip(expected_asts) {
            let mut scanner = Scanner::new(expression);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);
            let root = parser.expression();
            let ast = print_ast(&root);
            assert_eq!(ast, expected_ast);
        }
    }
}
