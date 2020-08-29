use std::collections::HashMap;
use std::{cell::RefCell, rc::Rc};

use crate::error::RuntimeError;
use crate::object::LoxObject;
use crate::interpreter::Result;
use crate::scanner::Token;

#[derive(Clone)]
pub struct Environment(Rc<RefCell<EnvironmentData>>);

struct EnvironmentData {
    enclosing: Option<Environment>,
    values: HashMap<String, LoxObject>,
}

impl Environment {
    pub fn new() -> Self {
        Environment(Rc::new(RefCell::new(EnvironmentData {
            enclosing: None,
            values: HashMap::new(),
        })))
    }

    pub fn as_child_of(parent: Environment) -> Self {
        Environment(Rc::new(RefCell::new(EnvironmentData {
            enclosing: Some(parent),
            values: HashMap::new(),
        })))
    }

    pub fn define(&mut self, name: &str, value: &LoxObject) {
        self.0
            .borrow_mut()
            .values
            .insert(name.to_owned(), value.clone());
    }

    pub fn assign(&mut self, name: &Token, value: &LoxObject) -> Result<()> {
        if self.0.borrow().values.contains_key(&name.lexeme) {
            self.0
                .borrow_mut()
                .values
                .insert(name.lexeme.to_owned(), value.clone());
            Ok(())
        } else if let Some(enclosing) = &mut self.0.borrow_mut().enclosing {
            enclosing.assign(name, value)
        } else {
            Err(RuntimeError::new(
                name,
                &format!("Undefined variable \"{}\".", name.lexeme),
            ))
        }
    }

    pub fn get(&self, name: &Token) -> Result<LoxObject> {
        if let Some(v) = self.0.borrow().values.get(&name.lexeme) {
            Ok(v.clone())
        } else if let Some(parent) = &self.0.borrow().enclosing {
            parent.get(name)
        } else {
            Err(RuntimeError::new(
                name,
                &format!("Undefined variable \"{}\".", name.lexeme),
            ))
        }
    }

    pub fn get_at(&self, distance: usize, name: &str) -> Result<LoxObject> {
        if let Some(ancestor) = self.ancestor(distance) {
            if let Some(value) = ancestor.0.borrow().values.get(name) {
                Ok(value.clone())
            } else {
                Err(RuntimeError::with_message(&format!(
                    "Environment::get_at - Undefined variable \"{}\" at distance: {}.",
                    name, distance
                )))
            }
        } else {
            Err(RuntimeError::with_message(&format!(
                "No ancestor at distance \"{}\".",
                distance
            )))
        }
    }

    pub fn assign_at(&mut self, distance: usize, name: &Token, value: &LoxObject) -> Result<()> {
        if let Some(ancestor) = self.ancestor(distance) {
            ancestor
                .0
                .borrow_mut()
                .values
                .insert(name.lexeme.to_owned(), value.clone());
            Ok(())
        } else {
            Err(RuntimeError::new(
                name,
                &format!("No ancestor environment at distance \"{}\".", distance),
            ))
        }
    }

    fn ancestor(&self, distance: usize) -> Option<Self> {
        if distance == 0 {
            Some(self.clone())
        } else {
            let data = self.0.borrow();
            if let Some(enclosing) = &data.enclosing {
                return enclosing.ancestor(distance - 1);
            }
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::LoxObject;
    use crate::scanner::{Token, TokenType};

    #[test]
    fn define_works() {
        let mut env = Environment::new();
        let definitions = vec![
            (
                Token::new(TokenType::Identifier, String::from("a"), None, 1, 0),
                LoxObject::Number(10.0),
            ),
            (
                Token::new(TokenType::Identifier, String::from("b"), None, 1, 1),
                LoxObject::Str(String::from("Hello World")),
            ),
            (
                Token::new(TokenType::Identifier, String::from("c"), None, 1, 2),
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
        let name = Token::new(TokenType::Identifier, String::from("a"), None, 1, 0);
        env.define(&name.lexeme, &LoxObject::Number(10.0));
        env.assign(&name, &LoxObject::Str(String::from("Hello World")))
            .unwrap();
        env.assign(&name, &LoxObject::Boolean(false)).unwrap();
        assert_eq!(env.get(&name).unwrap(), LoxObject::Boolean(false));
    }

    #[test]
    fn assign_cant_create_entry() {
        let mut env = Environment::new();
        let name = Token::new(TokenType::Identifier, String::from("a"), None, 1, 0);
        if let Ok(_) = env.assign(&name, &LoxObject::Nil) {
            panic!("Should not be able to assign to undefined variable.");
        }
    }

    #[test]
    fn define_overwrites() {
        let mut env = Environment::new();
        let name = Token::new(TokenType::Identifier, String::from("a"), None, 1, 0);
        env.define(&name.lexeme, &LoxObject::Number(10.0));
        env.define(&name.lexeme, &LoxObject::Str(String::from("Hello World")));
        env.define(&name.lexeme, &LoxObject::Boolean(false));

        assert_eq!(env.get(&name).unwrap(), LoxObject::Boolean(false));
    }
}
