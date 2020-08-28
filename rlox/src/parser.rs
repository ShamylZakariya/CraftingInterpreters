use std::fmt;

use crate::error;
use crate::expr::*;
use crate::scanner::*;
use crate::stmt::*;

pub type Result<T> = std::result::Result<T, error::ParseError>;

enum CallableType {
    Function,
}

impl fmt::Display for CallableType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CallableType::Function => write!(f, "function"),
        }
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    loop_depth: usize,
    callable_depth: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens: tokens,
            current: 0,
            loop_depth: 0,
            callable_depth: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Box<Stmt>>> {
        let mut statements: Vec<Box<Stmt>> = vec![];
        while !self.is_at_end() {
            statements.push(self.declaration_stmt()?);
        }
        Ok(statements)
    }

    /// Parse the tokens as an expression, returning the computed expression tree.
    /// This is for testing, not running whole programs.
    #[allow(dead_code)]
    pub fn parse_expression(&mut self) -> Result<Box<Expr>> {
        self.expression_expr()
    }

    // Expressions

    fn primary_expr(&mut self) -> Result<Box<Expr>> {
        if self.match_token(TokenType::Fun) {
            return self.lambda_expr();
        }

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

        if self.match_token(TokenType::Identifier) {
            return Ok(Box::new(Expr::Variable {
                name: self.previous().clone(),
            }));
        }

        if self.match_token(TokenType::LeftParen) {
            let expr = self.expression_expr()?;
            self.consume(TokenType::RightParen, "Expect ')' after expression.")?;
            return Ok(Box::new(Expr::Grouping { expression: expr }));
        }

        Err(error::ParseError::new(self.peek().clone(), "Expect expression"))
    }

    fn unary_expr(&mut self) -> Result<Box<Expr>> {
        if self.match_tokens(&vec![TokenType::Bang, TokenType::Minus]) {
            let op = self.previous().clone();
            let right = self.unary_expr()?;
            return Ok(Box::new(Expr::Unary {
                operator: op,
                right: right,
            }));
        }
        self.call_expr()
    }

    fn call_expr(&mut self) -> Result<Box<Expr>> {
        let mut expr = self.primary_expr()?;
        loop {
            if self.match_token(TokenType::LeftParen) {
                // parse argument list
                expr = self.finish_call_expr(expr)?;
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn finish_call_expr(&mut self, callee: Box<Expr>) -> Result<Box<Expr>> {
        let mut arguments = vec![];
        if !self.check(TokenType::RightParen) {
            loop {
                if arguments.len() >= 255 {
                    // just report error, don't bail.
                    error::report::parse_error_at_token(self.peek(), "Cannot have more than 255 arguments.");
                }
                arguments.push(self.expression_expr()?);
                if !self.match_token(TokenType::Comma) {
                    break;
                }
            }
        }
        let paren = self
            .consume(TokenType::RightParen, "Expect \")\" after arguments.")?
            .clone();
        Ok(Box::new(Expr::Call {
            callee,
            paren,
            arguments,
        }))
    }

    fn multiplication_expr(&mut self) -> Result<Box<Expr>> {
        let mut expr = self.unary_expr()?;
        while self.match_tokens(&vec![TokenType::Slash, TokenType::Star]) {
            let op = self.previous().clone();
            let right = self.unary_expr()?;
            expr = Box::new(Expr::Binary {
                left: expr,
                operator: op,
                right: right,
            });
        }
        Ok(expr)
    }

    fn addition_expr(&mut self) -> Result<Box<Expr>> {
        let mut expr = self.multiplication_expr()?;
        while self.match_tokens(&vec![TokenType::Minus, TokenType::Plus]) {
            let op = self.previous().clone();
            let right = self.multiplication_expr()?;
            expr = Box::new(Expr::Binary {
                left: expr,
                operator: op,
                right: right,
            })
        }
        Ok(expr)
    }

    fn comparison_expr(&mut self) -> Result<Box<Expr>> {
        let mut expr = self.addition_expr()?;
        while self.match_tokens(&vec![
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let op = self.previous().clone();
            let right = self.addition_expr()?;
            expr = Box::new(Expr::Binary {
                left: expr,
                operator: op,
                right: right,
            })
        }
        Ok(expr)
    }

    fn equality_expr(&mut self) -> Result<Box<Expr>> {
        let mut expr = self.comparison_expr()?;
        while self.match_tokens(&vec![TokenType::BangEqual, TokenType::EqualEqual]) {
            let op = self.previous().clone();
            let right = self.comparison_expr()?;
            expr = Box::new(Expr::Binary {
                left: expr,
                operator: op,
                right: right,
            });
        }
        Ok(expr)
    }

    fn expression_expr(&mut self) -> Result<Box<Expr>> {
        self.assignment_expr()
    }

    // Statements

    fn _declaration_stmt(&mut self) -> Result<Box<Stmt>> {
        if self.match_token(TokenType::Fun) {
            self.callable_depth += 1;
            let ret = self.function_stmt(CallableType::Function);
            self.callable_depth -= 1;
            ret
        } else if self.match_token(TokenType::Var) {
            self.var_declaration_stmt()
        } else {
            self.statement_stmt()
        }
    }

    fn declaration_stmt(&mut self) -> Result<Box<Stmt>> {
        match self._declaration_stmt() {
            Ok(r) => Ok(r),
            Err(e) => {
                self.synchronize();
                Err(e)
            }
        }
    }

    fn statement_stmt(&mut self) -> Result<Box<Stmt>> {
        if self.match_token(TokenType::For) {
            self.loop_depth += 1;
            let result = self.for_stmt();
            self.loop_depth -= 1;
            result
        } else if self.match_token(TokenType::If) {
            self.if_stmt()
        } else if self.match_token(TokenType::Print) {
            self.print_stmt()
        } else if self.match_token(TokenType::Return) {
            self.return_stmt()
        } else if self.match_token(TokenType::While) {
            self.loop_depth += 1;
            let result = self.while_stmt();
            self.loop_depth -= 1;
            result
        } else if self.match_token(TokenType::Break) {
            self.break_stmt()
        } else if self.match_token(TokenType::LeftBrace) {
            Ok(Box::new(Stmt::Block {
                statements: self.block_stmt()?,
            }))
        } else {
            self.expression_stmt()
        }
    }

    fn break_stmt(&mut self) -> Result<Box<Stmt>> {
        if self.loop_depth > 0 {
            self.consume(
                TokenType::Semicolon,
                "Expect \";\" after \"break\" statement.",
            )?;
            Ok(Box::new(Stmt::Break))
        } else {
            Err(error::ParseError::new(self.peek().clone(), "Break statement only allowed inside loops."))
        }
    }

    fn for_stmt(&mut self) -> Result<Box<Stmt>> {
        self.consume(TokenType::LeftParen, "Expect \"(\" after \"for\".")?;

        let mut initializer = None;
        if self.match_token(TokenType::Semicolon) {
        } else if self.match_token(TokenType::Var) {
            initializer = Some(self.var_declaration_stmt()?);
        } else {
            initializer = Some(self.expression_stmt()?);
        }

        let mut condition = None;
        if !self.check(TokenType::Semicolon) {
            condition = Some(self.expression_expr()?);
        }

        self.consume(
            TokenType::Semicolon,
            "Expect \";\" after for loop condition.",
        )?;

        let mut increment = None;
        if !self.check(TokenType::RightParen) {
            increment = Some(self.expression_expr()?);
        }

        self.consume(TokenType::RightParen, "Expect \")\" after for clauses.")?;

        let mut body = self.statement_stmt()?;

        //
        //   we're taking a for loop and desugaring it to a while loop
        //   for (var i = 0; i < 10; i = i + 1) { print i; }
        //
        //   becomes:
        //
        //   { var i = 0; while(i < 10) { { print i; } i = i + 1; } }
        //

        if let Some(increment) = increment {
            body = Box::new(Stmt::Block {
                statements: vec![
                    body,
                    Box::new(Stmt::Expression {
                        expression: increment,
                    }),
                ],
            });
        }

        if let Some(condition) = condition {
            body = Box::new(Stmt::While { condition, body });
        } else {
            body = Box::new(Stmt::While {
                condition: Box::new(Expr::Literal {
                    value: crate::scanner::Literal::True,
                }),
                body: body,
            });
        }

        if let Some(initializer) = initializer {
            body = Box::new(Stmt::Block {
                statements: vec![initializer, body],
            });
        }

        Ok(body)
    }

    fn if_stmt(&mut self) -> Result<Box<Stmt>> {
        self.consume(TokenType::LeftParen, "Expect \"(\" after \"if\".")?;
        let condition = self.expression_expr()?;
        self.consume(TokenType::RightParen, "Expect \")\" after if condition.")?;

        let then_branch = self.statement_stmt()?;
        let mut else_branch = Option::None;
        if self.match_token(TokenType::Else) {
            else_branch = Some(self.statement_stmt()?);
        }
        Ok(Box::new(Stmt::If {
            condition,
            then_branch,
            else_branch,
        }))
    }

    fn print_stmt(&mut self) -> Result<Box<Stmt>> {
        let value = self.expression_expr()?;
        self.consume(TokenType::Semicolon, "Expect \";\" after value.")?;
        Ok(Box::new(Stmt::Print { expression: value }))
    }

    fn return_stmt(&mut self) -> Result<Box<Stmt>> {
        if self.callable_depth == 0 {
            return Err(error::ParseError::new(
                self.peek().clone(),
                "Illegal use of return statement outside of a function.",
            ));
        }
        let keyword = self.previous().clone();
        let mut value = None;
        if !self.check(TokenType::Semicolon) {
            value = Some(self.expression_expr()?);
        }
        self.consume(TokenType::Semicolon, "Expect \";\" after return value.")?;
        Ok(Box::new(Stmt::Return { keyword, value }))
    }

    fn var_declaration_stmt(&mut self) -> Result<Box<Stmt>> {
        let name = self
            .consume(TokenType::Identifier, "Expect variable name.")?
            .clone();

        let mut initializer: Option<Box<Expr>> = None;
        if self.match_token(TokenType::Equal) {
            initializer = Some(self.expression_expr()?);
        }
        self.consume(
            TokenType::Semicolon,
            "Expect \";\" after variable declaration.",
        )?;
        Ok(Box::new(Stmt::Var {
            name: name,
            initializer: initializer,
        }))
    }

    fn while_stmt(&mut self) -> Result<Box<Stmt>> {
        self.consume(
            TokenType::LeftParen,
            "Expect \"(\" after variable \"while\".",
        )?;
        let condition = self.expression_expr()?;
        self.consume(
            TokenType::RightParen,
            "Expect \")\" after variable \"while\" condition.",
        )?;
        let body = self.statement_stmt()?;
        Ok(Box::new(Stmt::While { condition, body }))
    }

    fn expression_stmt(&mut self) -> Result<Box<Stmt>> {
        let expr = self.expression_expr()?;
        self.consume(TokenType::Semicolon, "Expect \";\" after expression.")?;
        Ok(Box::new(Stmt::Expression { expression: expr }))
    }

    fn function_stmt(&mut self, function_type: CallableType) -> Result<Box<Stmt>> {
        let name = self
            .consume(
                TokenType::Identifier,
                &format!("Expect \"{}\" name.", function_type),
            )?
            .clone();
        self.consume(
            TokenType::LeftParen,
            &format!("Expect \"(\" after {} name.", function_type),
        )?;
        let mut parameters = vec![];
        if !self.check(TokenType::RightParen) {
            loop {
                if parameters.len() >= 255 {
                    error::report::parse_error_at_token(
                        self.peek(),
                        "Cannot have more than 255 parameters in function declaration.",
                    );
                }

                parameters.push(
                    self.consume(TokenType::Identifier, "Expect parameter name.")?
                        .clone(),
                );
                if !self.match_token(TokenType::Comma) {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen, "Expect \")\" after parameter list.")?;
        self.consume(
            TokenType::LeftBrace,
            &format!("Expect \"{{\" before {} body.", function_type),
        )?;
        let body = self.block_stmt()?;
        Ok(Box::new(Stmt::Function {
            name,
            parameters,
            body,
        }))
    }

    fn block_stmt(&mut self) -> Result<Vec<Box<Stmt>>> {
        let mut statements = vec![];
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration_stmt()?);
        }
        self.consume(TokenType::RightBrace, "Expect \"}\" after block.")?;
        Ok(statements)
    }

    fn assignment_expr(&mut self) -> Result<Box<Expr>> {
        let mut expr = self.ternary_expr()?;
        if self.match_token(TokenType::Fun) {
            expr = self.lambda_expr()?;
        }
        if self.match_token(TokenType::Equal) {
            let equals = self.previous().clone();
            let value = self.assignment_expr()?;

            match *expr {
                Expr::Variable { name } => {
                    return Ok(Box::new(Expr::Assign {
                        name: name,
                        value: value,
                    }));
                }
                _ => {
                    return Err(error::ParseError::new(equals, "Invalid assignment target."));
                }
            }
        }
        Ok(expr)
    }

    fn lambda_expr(&mut self) -> Result<Box<Expr>> {
        self.consume(
            TokenType::LeftParen,
            "Expect \"(\" after lambda expression declaration",
        )?;
        let mut parameters = vec![];
        if !self.check(TokenType::RightParen) {
            loop {
                if parameters.len() >= 255 {
                    error::report::parse_error_at_token(
                        self.peek(),
                        "Cannot have more than 255 parameters in function declaration.",
                    );
                }

                parameters.push(
                    self.consume(TokenType::Identifier, "Expect parameter name.")?
                        .clone(),
                );
                if !self.match_token(TokenType::Comma) {
                    break;
                }
            }
        }
        self.consume(
            TokenType::RightParen,
            "Expect \")\" after lambda parameter list.",
        )?;
        self.consume(
            TokenType::LeftBrace,
            &format!("Expect \"{{\" before lambda body."),
        )?;
        let body = self.block_stmt()?;
        Ok(Box::new(Expr::Lambda { parameters, body }))
    }

    fn ternary_expr(&mut self) -> Result<Box<Expr>> {
        let expr = self.or_expr()?;
        if self.match_token(TokenType::QuestionMark) {
            let then_value = self.expression_expr()?;
            self.consume(
                TokenType::Colon,
                "Expect \":\" separating then and else clauses of ternary expression.",
            )?;
            let else_value = self.expression_expr()?;
            return Ok(Box::new(Expr::Ternary {
                condition: expr,
                then_value,
                else_value,
            }));
        }
        Ok(expr)
    }

    fn or_expr(&mut self) -> Result<Box<Expr>> {
        let mut expr = self.and_expr()?;
        while self.match_token(TokenType::Or) {
            let op = self.previous().clone();
            let right = self.and_expr()?;
            expr = Box::new(Expr::Logical {
                left: expr,
                operator: op,
                right: right,
            });
        }
        Ok(expr)
    }

    fn and_expr(&mut self) -> Result<Box<Expr>> {
        let mut expr = self.equality_expr()?;
        while self.match_token(TokenType::And) {
            let op = self.previous().clone();
            let right = self.equality_expr()?;
            expr = Box::new(Expr::Logical {
                left: expr,
                operator: op,
                right: right,
            });
        }
        Ok(expr)
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

        Err(error::ParseError::new(self.peek().clone(), on_error_message))
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

    // Error handling

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
                            }),
                        }),
                        operator: Token::new(TokenType::Plus, String::from("+"), None, 1),
                        right: Box::new(Expr::Literal {
                            value: Literal::Number(3.0),
                        }),
                    }),
                    operator: Token::new(TokenType::Plus, String::from("+"), None, 1),
                    right: Box::new(Expr::Literal {
                        value: Literal::Number(4.0),
                    }),
                }),
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
                }),
            ),
        ];

        for (expression, expected_ast) in expressions {
            let mut scanner = Scanner::new(expression);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);
            let parsed = parser.expression_expr().unwrap();
            assert_eq!(parsed, expected_ast);
        }
    }

    #[test]
    fn fails_to_parse_bad_expressions() {
        let expressions = vec!["1 + (5/2"];

        for expression in expressions {
            let mut scanner = Scanner::new(expression);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);
            assert!(
                parser.parse_expression().is_err(),
                "Program should not have parsed."
            );
        }
    }

    fn parse(code: &str) -> Result<Vec<Box<Stmt>>> {
        let mut scanner = Scanner::new(code);
        let tokens = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    #[test]
    fn desugars_for_loop_as_expected() {
        let baseline = r#"
{
    var i = 0;
    while (i < 3) {
        {
            print i;
        }
        i = i + 1;
    }
}
        "#;

        let for_loop = r#"
for (var i = 0; i < 3; i = i + 1) {
    print i;
}
        "#;


        let baseline_stmts = parse(baseline).expect("Baseline code should parse");
        println!("baseline: {:?}", baseline_stmts);
        let for_loop_stmts = parse(for_loop).expect("For-loop code should parse");
        println!("for_loop: {:?}", for_loop_stmts);
        assert_eq!(baseline_stmts, for_loop_stmts);
    }

    #[test]
    fn fails_to_parse_bad_programs() {
        let programs = vec![
            r#"
            var a = 0;
            var b = 1 // missing semicolon
            var c = 2;
            "#,
            r#"
            var a = 0;
            var b = 1;

            // break disallowed outside a while loop
            break;

            while (a < 10000) {
              print a;
              var temp = a;
              a = b;
              b = temp + b;
              if (a == 377) {
                  break;
              }
            }
            "#,
            r#"
            // use of a return statement outside of a function
            fun foo() {
                print "hello";
            }
            return;
            "#,
        ];

        for program in programs {
            let mut scanner = Scanner::new(program);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);
            assert!(parser.parse().is_err(), "Program should not have parsed.");
        }
    }
}
