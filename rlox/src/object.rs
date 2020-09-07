use std::fmt;
use std::{cell::RefCell, rc::Rc};

use crate::callable::LoxCallable;
use crate::class::{LoxClass, LoxInstance};
use crate::scanner::Literal;

#[derive(Debug, Clone)]
pub enum LoxObject {
    Boolean(bool),
    Callable(Rc<RefCell<dyn LoxCallable>>),
    Class(LoxClass),
    Instance(LoxInstance),
    Nil,
    Number(f64),
    Str(String),
    Undefined,
}

impl LoxObject {
    pub fn from_literal(literal: &crate::scanner::Literal) -> Self {
        match literal {
            Literal::Number(n) => LoxObject::Number(*n),
            Literal::Str(s) => LoxObject::Str(s.clone()),
            Literal::False => LoxObject::Boolean(false),
            Literal::True => LoxObject::Boolean(true),
            Literal::Nil => LoxObject::Nil,
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            LoxObject::Nil => false,     // nil is falsey
            LoxObject::Boolean(b) => *b, // booleans are what they are
            _ => true,                   // everything else is *something*, which is truthy
        }
    }
}

impl PartialEq<LoxObject> for LoxObject {
    fn eq(&self, _other: &Self) -> bool {
        use LoxObject::*;
        match (self, _other) {
            (Boolean(b1), Boolean(b2)) => b1 == b2,
            (Callable(c1), Callable(c2)) => c1 == c2,
            (Class(c1), Class(c2)) => c1 == c2,
            (Instance(i1), Instance(i2)) => i1 == i2,
            (Nil, Nil) => true,
            (Number(n1), Number(n2)) => n1 == n2,
            (Str(s1), Str(s2)) => s1 == s2,
            (Undefined, Undefined) => true,
            _ => false,
        }
    }
}

impl Eq for LoxObject {}

impl fmt::Display for LoxObject {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LoxObject::Boolean(v) => {
                if *v {
                    write!(f, "true")
                } else {
                    write!(f, "false")
                }
            }
            LoxObject::Callable(c) => write!(f, "{}", c.borrow()),
            LoxObject::Class(c) => write!(f, "{}", c),
            LoxObject::Instance(i) => write!(f, "{}", i),
            LoxObject::Nil => write!(f, "nil"),
            LoxObject::Number(n) => write!(f, "{}", n),
            LoxObject::Str(s) => write!(f, "{}", s),
            LoxObject::Undefined => write!(f, "<undefined>"),
        }
    }
}
