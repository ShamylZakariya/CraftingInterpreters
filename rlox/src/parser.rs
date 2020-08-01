use crate::error;
use crate::expr::*;
use crate::scanner::*;
use crate::stmt::*;

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

    pub fn parse(&mut self) -> Result<Vec<Box<Stmt>>> {
        let mut statements: Vec<Box<Stmt>> = vec![];
        while !self.is_at_end() {
            statements.push(self.statement()?);
        }
        Ok(statements)
    }

    /// Parse the tokens as an expression, returning the computed expression tree.
    /// This is for testing, not running whole programs.
    #[allow(dead_code)]
    pub fn parse_expression(&mut self) -> Result<Box<Expr>> {
        self.expression()
    }

    // Expressions

    fn primary(&mut self) -> Result<Box<Expr>> {
        if self.match_token(TokenType::False) {
            return Ok(Box::new(Expr::Literal {
                value: crate::scanner::Literal::False,
            }));
        }
        if self.match_token(TokenType::True) {
            return Ok(Box::new(Expr::Literal {
                value: crate::scanner::Literal::True,
            }));
        }
        if self.match_token(TokenType::Nil) {
            return Ok(Box::new(Expr::Literal {
                value: crate::scanner::Literal::Nil,
            }));
        }
        if self.match_tokens(&vec![TokenType::Number, TokenType::Str]) {
            if let Some(l) = self.previous().clone().literal {
                return Ok(Box::new(Expr::Literal { value: l }));
            }
        }
        if self.match_token(TokenType::LeftParen) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expect ')' after expression.")?;
            return Ok(Box::new(Expr::Grouping { expression: expr }));
        }

        Err(self.error(self.peek(), "Expect expression"))
    }

    fn unary(&mut self) -> Result<Box<Expr>> {
        if self.match_tokens(&vec![TokenType::Bang, TokenType::Minus]) {
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
        while self.match_tokens(&vec![TokenType::Slash, TokenType::Star]) {
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
        while self.match_tokens(&vec![TokenType::Minus, TokenType::Plus]) {
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
        while self.match_tokens(&vec![
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
        while self.match_tokens(&vec![TokenType::BangEqual, TokenType::EqualEqual]) {
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

    // Statements

    fn statement(&mut self) -> Result<Box<Stmt>> {
        if self.match_token(TokenType::Print) {
            self.print_statement()
        } else {
            self.expression_statement()
        }
    }

    fn print_statement(&mut self) -> Result<Box<Stmt>> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect \";\" after value.")?;
        Ok(Box::new(Stmt::Print { expression: value }))
    }

    fn expression_statement(&mut self) -> Result<Box<Stmt>> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect \";\" after expression.")?;
        Ok(Box::new(Stmt::Expression { expression: expr }))
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

    fn match_token(&mut self, t: TokenType) -> bool {
        if self.check(t) {
            self.advance();
            return true;
        }
        false
    }

    fn match_tokens(&mut self, types: &Vec<TokenType>) -> bool {
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
            (
                "1 + (5/2)",
                Box::new(Expr::Binary {
                    left: Box::new(Expr::Literal {
                        value: Literal::Number(1.0),
                    }),
                    operator: Token::new(TokenType::Plus, String::from("+"), None, 1),
                    right: Box::new(Expr::Grouping {
                        expression: Box::new(Expr::Binary {
                            left: Box::new(Expr::Literal {
                                value: Literal::Number(5.0),
                            }),
                            operator: Token::new(TokenType::Slash, String::from("/"), None, 1),
                            right: Box::new(Expr::Literal {
                                value: Literal::Number(2.0),
                            }),
                        }),
                    }),
                }),
            ),
            (
                "1 + 2 + 3 + 4",
                Box::new(Expr::Binary {
                    left: Box::new(Expr::Binary {
                        left: Box::new(Expr::Binary {
                            left: Box::new(Expr::Literal {
                                value: Literal::Number(1.0),
                            }),
                            operator: Token::new(TokenType::Plus, String::from("+"), None, 1),
                            right: Box::new(Expr::Literal {
                                value: Literal::Number(2.0),
                            })
                        }),
                        operator: Token::new(TokenType::Plus, String::from("+"), None, 1),
                        right: Box::new(Expr::Literal {
                            value: Literal::Number(3.0),
                        })
                    }),
                    operator: Token::new(TokenType::Plus, String::from("+"), None, 1),
                    right: Box::new(Expr::Literal {
                        value: Literal::Number(4.0),
                    })
                })
            ),
            (
                "5 == nil",
                Box::new(Expr::Binary {
                    left: Box::new(Expr::Literal {
                        value: Literal::Number(5.0),
                    }),
                    operator: Token::new(TokenType::EqualEqual, String::from("=="), None, 1),
                    right: Box::new(Expr::Literal {
                        value: Literal::Nil,
                    }),
                })
            ),
        ];

        for (expression, expected_ast) in expressions {
            let mut scanner = Scanner::new(expression);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);
            let parsed = parser.expression().unwrap();
            assert_eq!(parsed, expected_ast);
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
