use std::collections::HashMap;

use crate::error::RuntimeError;
use crate::interpreter::{LoxObject, Result};
use crate::scanner::Token;

pub struct Environment {
    values: HashMap<String, LoxObject>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: &str, value: LoxObject) {
        println!("Defining \"{}\" as \"{}\"", name, value);
        self.values.insert(name.to_owned(), value);
    }

    pub fn get(&self, name: &Token) -> Result<LoxObject> {
        if let Some(v) = self.values.get(&name.lexeme) {
            Ok(v.clone())
        } else {
            Err(RuntimeError::new(
                name,
                &format!("Undefined variable \"{}\".", name.lexeme),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::LoxObject;
    use crate::scanner::{Token, TokenType};

    #[test]
    fn define_works() {
        let mut env = Environment::new();
        let definitions = vec![
            (
                Token::new(TokenType::Identifier, String::from("a"), None, 1),
                LoxObject::Number(10.0),
            ),
            (
                Token::new(TokenType::Identifier, String::from("b"), None, 1),
                LoxObject::Str(String::from("Hello World")),
            ),
            (
                Token::new(TokenType::Identifier, String::from("c"), None, 1),
                LoxObject::Boolean(false),
            ),
        ];

        for (name, value) in definitions {
            env.define(&name.lexeme, value.clone());
            assert_eq!(env.get(&name).unwrap(), value);
        }
    }

    #[test]
    fn define_overwrites() {
        let mut env = Environment::new();
        let name = Token::new(TokenType::Identifier, String::from("a"), None, 1);
        env.define(&name.lexeme, LoxObject::Number(10.0));
        env.define(&name.lexeme, LoxObject::Str(String::from("Hello World")));
        env.define(&name.lexeme, LoxObject::Boolean(false));

        assert_eq!(env.get(&name).unwrap(), LoxObject::Boolean(false));
    }
}
