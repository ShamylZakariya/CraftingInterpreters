use std::{fs, io, io::Write};

mod environment;
mod error;
mod expr;
mod interpreter;
mod natives;
mod parser;
mod resolver;
mod scanner;
mod stmt;

use crate::interpreter::Interpreter;
use crate::parser::Parser;
use crate::resolver::Resolver;
use crate::scanner::Scanner;
use crate::stmt::Stmt;


/*

Ok, what if resolver is created, used, and then destroyed. Not retained.
Perhaps that might lower the mutable ref count?

I don't see any reason why resolver needs to be an ivar

*/


pub struct Lox {
    had_error: bool,
    had_runtime_error: bool,
    interpreter: Interpreter
}

impl Lox {
    pub fn new() -> Lox {
        Lox {
            had_error: false,
            had_runtime_error: false,
            interpreter: Interpreter::new(),
        }
    }

    pub fn run_file(&mut self, file: &str) {
        let contents = fs::read_to_string(file).expect("Unable to open lox file");
        self.run(&contents);

        if self.had_error {
            std::process::exit(65);
        }
        if self.had_runtime_error {
            std::process::exit(70);
        }
    }

    pub fn run_prompt(&mut self) {
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
            self.run(&line);
            self.had_error = false;
        }
    }

    fn run(&mut self, source: &str) {
        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        match parser.parse() {
            Ok(statements) => {
                match self.resolve() {
                    Ok(()) => {
                        self.run_statements(&statements);
                    },
                    Err(_) => {
                        self.had_error = true;
                    }
                }
            }
            Err(_) => {
                self.had_error = true;
            }
        }
    }

    fn run_statements(&mut self, statements: &Vec<Box<Stmt>>) {
        let mut did_evaluate_single_expression = false;

        // if there's only 1 statement, and it is actually an expression,
        // evaluate it and print result to console. Otherwise execute as a program.
        if statements.len() == 1 {
            let first = statements[0].clone();
            match *first {
                Stmt::Expression { expression } => {
                    did_evaluate_single_expression = true;
                    match self.interpreter.evaluate(&expression) {
                        Ok(r) => println!("{}", r),
                        Err(_) => {
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
                Err(_) => {
                    self.had_runtime_error = true;
                }
            }
        }
    }

    fn resolve(&mut self) -> parser::Result<()> {
        //let mut _r = Resolver::new(&mut self.interpreter);
        Ok(())
    }
}
