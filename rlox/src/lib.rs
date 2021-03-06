use std::io::Write;
use std::{fs, io};

mod ast;
mod ast_printer;
mod callable;
mod class;
mod environment;
mod error;
mod function;
mod interpreter;
mod natives;
mod object;
mod parser;
mod resolver;
mod scanner;

use crate::ast::Stmt;
use crate::ast_printer::AstPrinter;
use crate::interpreter::Interpreter;
use crate::parser::Parser;
use crate::resolver::Resolver;
use crate::scanner::Scanner;

pub struct Lox {
    had_error: bool,
    had_runtime_error: bool,
    interpreter: Interpreter,
}

impl Lox {
    pub fn new() -> Lox {
        Lox {
            had_error: false,
            had_runtime_error: false,
            interpreter: Interpreter::new(),
        }
    }

    pub fn run_file(&mut self, file: &str, display_ast: bool) {
        let contents = fs::read_to_string(file).expect("Unable to open lox file");
        self.run(&contents, display_ast);

        if self.had_error {
            std::process::exit(65);
        }
        if self.had_runtime_error {
            std::process::exit(70);
        }
    }

    pub fn run_prompt(&mut self, display_ast: bool) {
        loop {
            print!("> ");
            io::stdout().flush().unwrap();

            let mut line = String::new();
            io::stdin()
                .read_line(&mut line)
                .expect("Unable to read line from stdin");
            let line = line.trim();
            if line.len() == 0 {
                break;
            }
            self.run(&line, display_ast);
            self.had_error = false;
        }
    }

    fn run(&mut self, source: &str, display_ast: bool) {
        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        match parser.parse() {
            Ok(statements) => {
                if display_ast {
                    self.display_ast(&statements);
                } else {
                    match self.resolve(&statements) {
                        Ok(()) => {
                            self.run_statements(&statements);
                        }
                        Err(e) => {
                            error::report::resolver_error(&e);
                            self.had_error = true;
                        }
                    }
                }
            }
            Err(e) => {
                error::report::parse_error_at_token(&e.token, &e.message);
                self.had_error = true;
            }
        }
    }

    fn display_ast(&mut self, statements: &Vec<Box<Stmt>>) {
        let mut ast_printer = AstPrinter::new();

        let result = ast_printer.generate(statements);
        let sep = "-".repeat(72);
        println!("{}\n{}\n{}", sep, result, sep);
    }

    fn run_statements(&mut self, statements: &Vec<Box<Stmt>>) {
        let mut did_evaluate_single_expression = false;

        // if there's only 1 statement, and it is actually an expression,
        // evaluate it and print result to console. Otherwise execute as a program.
        if statements.len() == 1 {
            match &*statements[0] {
                Stmt::Expression { expression } => {
                    did_evaluate_single_expression = true;
                    match self.interpreter.evaluate(&expression) {
                        Ok(r) => println!("{}", r),
                        Err(e) => {
                            error::report::runtime_error(&e);
                            self.had_runtime_error = true;
                        }
                    }
                }
                _ => (),
            }
        }

        if !did_evaluate_single_expression {
            match self.interpreter.interpret(statements) {
                Ok(()) => (),
                Err(e) => {
                    error::report::runtime_error(&e);
                    self.had_runtime_error = true;
                }
            }
        }
    }

    fn resolve(&mut self, statements: &Vec<Box<Stmt>>) -> resolver::Result<()> {
        let mut r = Resolver::new(&mut self.interpreter);
        r.resolve(statements)
    }
}
