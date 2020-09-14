use std::fmt;

use crate::interpreter::{InterpretResult, Interpreter};
use crate::object::LoxObject;

pub trait LoxCallable {
    fn arity(&self) -> usize;
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: &Vec<LoxObject>,
    ) -> InterpretResult<Option<LoxObject>>;
    // returns true if this is a property field, which is invoked
    // simply by evaluating it, no need for a call expression.
    fn is_property(&self) -> bool;
}

impl fmt::Debug for dyn LoxCallable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<callable arity {}>", self.arity())
    }
}

impl fmt::Display for dyn LoxCallable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<callable arity {}>", self.arity())
    }
}

impl PartialEq<dyn LoxCallable> for dyn LoxCallable {
    fn eq(&self, other: &Self) -> bool {
        &self == &other
    }
}
