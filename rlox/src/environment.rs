use std::{rc::Rc, cell::RefCell};
use std::collections::HashMap;

use crate::error::RuntimeError;
use crate::interpreter::{LoxObject, Result};
use crate::scanner::Token;

pub struct Environment {
    enclosing: Option<Rc<RefCell<Environment>>>,
    values: HashMap<String, LoxObject>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            enclosing: None,
            values: HashMap::new(),
        }
    }

    pub fn as_child_of(parent:Rc<RefCell<Environment>>) -> Self {
        Environment {
            enclosing: Some(parent),
            values: HashMap::new(),
        }
    }

    pub fn parent(&self) -> Option<Rc<RefCell<Environment>>> {
        self.enclosing.clone()
    }

    pub fn define(&mut self, name: &str, value: &LoxObject) {
        self.values.insert(name.to_owned(), value.clone());
    }

    pub fn assign(&mut self, name: &Token, value: &LoxObject) -> Result<()> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.to_owned(), value.clone());
            Ok(())
        } else if let Some(parent) = &self.enclosing {
            parent.borrow_mut().assign(name, value)?;
            Ok(())
        }
        else {
            Err(RuntimeError::new(
                name,
                &format!("Undefined variable \"{}\".", name.lexeme),
            ))
        }
    }

    pub fn get(&self, name: &Token) -> Result<LoxObject> {
        if let Some(v) = self.values.get(&name.lexeme) {
            Ok(v.clone())
        } else if let Some(parent) = &self.enclosing {
            parent.borrow().get(name)
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
            env.define(&name.lexeme, &value);
            assert_eq!(env.get(&name).unwrap(), value);
        }
    }

    #[test]
    fn assign_overwrites() {
        let mut env = Environment::new();
        let name = Token::new(TokenType::Identifier, String::from("a"), None, 1);
        env.define(&name.lexeme, &LoxObject::Number(10.0));
        env.assign(&name, &LoxObject::Str(String::from("Hello World"))).unwrap();
        env.assign(&name, &LoxObject::Boolean(false)).unwrap();
        assert_eq!(env.get(&name).unwrap(), LoxObject::Boolean(false));
    }

    #[test]
    fn assign_cant_create_entry() {
        let mut env = Environment::new();
        let name = Token::new(TokenType::Identifier, String::from("a"), None, 1);
        if let Ok(_) = env.assign(&name, &LoxObject::Nil) {
            panic!("Should not be able to assign to undefined variable.");
        }
    }

    #[test]
    fn define_overwrites() {
        let mut env = Environment::new();
        let name = Token::new(TokenType::Identifier, String::from("a"), None, 1);
        env.define(&name.lexeme, &LoxObject::Number(10.0));
        env.define(&name.lexeme, &LoxObject::Str(String::from("Hello World")));
        env.define(&name.lexeme, &LoxObject::Boolean(false));

        assert_eq!(env.get(&name).unwrap(), LoxObject::Boolean(false));
    }
}
