use std::io::Write;
use std::{fs, io};

mod error;
mod parser;

pub struct Lox {
    had_error: bool,
}

impl Lox {
    pub fn new() -> Lox {
        Lox { had_error: false }
    }

    pub fn run_file(&mut self, file: &str) {
        let contents = fs::read_to_string(file).expect("Unable to open lox file");
        self.run(&contents);

        if self.had_error {
            std::process::exit(65);
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
        let mut scanner = parser::scanner::Scanner::new(source);
        let tokens = scanner.scan_tokens();
        let mut parser = parser::parser::Parser::new(tokens);
        match parser.parse() {
            Ok(expr) => {
                let ast = parser::expr::print_ast(&expr);
                println!("{}", ast);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                self.had_error = true;
            }
        }
    }
}
