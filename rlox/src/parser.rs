use crate::error;
use crate::expr::*;
use crate::scanner::*;

pub type Result<T> = std::result::Result<T, error::ParseError>;


pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens: tokens,
            current: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Box<Expr>> {
        self.expression()
    }

    // Recursive descent, ordered by associativity
    fn primary(&mut self) -> Result<Box<Expr>> {
        if self.match_token(&vec![TokenType::False]) {
            return Ok(Box::new(Expr::Literal {
                value: crate::scanner::Literal::False,
            }));
        }
        if self.match_token(&vec![TokenType::True]) {
            return Ok(Box::new(Expr::Literal {
                value: crate::scanner::Literal::True,
            }));
        }
        if self.match_token(&vec![TokenType::Nil]) {
            return Ok(Box::new(Expr::Literal {
                value: crate::scanner::Literal::Nil,
            }));
        }
        if self.match_token(&vec![TokenType::Number, TokenType::Str]) {
            if let Some(l) = self.previous().clone().literal {
                return Ok(Box::new(Expr::Literal { value: l }));
            }
        }
        if self.match_token(&vec![TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expect ')' after expression.")?;
            return Ok(Box::new(Expr::Grouping { expression: expr }));
        }

        Err(self.error(self.peek(), "Expect expression"))
    }

    fn unary(&mut self) -> Result<Box<Expr>> {
        if self.match_token(&vec![TokenType::Bang, TokenType::Minus]) {
            let op = self.previous().clone();
            let right = self.unary()?;
            return Ok(Box::new(Expr::Unary {
                operator: op,
                right: right,
            }));
        }
        self.primary()
    }

    fn multiplication(&mut self) -> Result<Box<Expr>> {
        let mut expr = self.unary()?;
        while self.match_token(&vec![TokenType::Slash, TokenType::Star]) {
            let op = self.previous().clone();
            let right = self.unary()?;
            expr = Box::new(Expr::Binary {
                left: expr,
                operator: op,
                right: right,
            });
        }
        Ok(expr)
    }

    fn addition(&mut self) -> Result<Box<Expr>> {
        let mut expr = self.multiplication()?;
        while self.match_token(&vec![TokenType::Minus, TokenType::Plus]) {
            let op = self.previous().clone();
            let right = self.multiplication()?;
            expr = Box::new(Expr::Binary {
                left: expr,
                operator: op,
                right: right,
            })
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Box<Expr>> {
        let mut expr = self.addition()?;
        while self.match_token(&vec![
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let op = self.previous().clone();
            let right = self.addition()?;
            expr = Box::new(Expr::Binary {
                left: expr,
                operator: op,
                right: right,
            })
        }
        Ok(expr)
    }

    fn equality(&mut self) -> Result<Box<Expr>> {
        let mut expr = self.comparison()?;
        while self.match_token(&vec![TokenType::BangEqual, TokenType::EqualEqual]) {
            let op = self.previous().clone();
            let right = self.comparison()?;
            expr = Box::new(Expr::Binary {
                left: expr,
                operator: op,
                right: right,
            });
        }
        Ok(expr)
    }

    fn expression(&mut self) -> Result<Box<Expr>> {
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

    fn consume(&mut self, token_type: TokenType, on_error_message: &str) -> Result<&Token> {
        if self.check(token_type) {
            return Ok(self.advance());
        }

        Err(self.error(self.peek(), on_error_message))
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

    // Error reporting, handling

    fn error(&self, token: &Token, message: &str) -> error::ParseError {
        error::report::parse_error_at_token(token, message);
        error::ParseError::new(token.clone(), message)
    }

    fn synchronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            if self.previous().token_type == TokenType::Semicolon {
                return;
            }
            match self.peek().token_type {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => {
                    return;
                }
                _ => (), // keep seeking
            }
            self.advance();
        }
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

        for expression in expressions {
            let mut scanner = Scanner::new(expression);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);
            parser.expression().unwrap();
        }
    }

    #[test]
    fn fails_to_parse_bad_expressions() {
        let expressions = vec!["1 + (5/2", "a = foo"];

        for expression in expressions {
            let mut scanner = Scanner::new(expression);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);
            if let Ok(_) = parser.expression() {
                panic!("Expression should not have parsed");
            }
        }
    }
}
