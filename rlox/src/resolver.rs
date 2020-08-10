use std::{cell::RefCell, rc::Rc};

use crate::environment::Environment;
use crate::error;
use crate::expr::*;
use crate::interpreter::InterpretResult;
use crate::interpreter::Interpreter;
use crate::scanner::{Literal, Token, TokenType};
use crate::stmt::*;

pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Resolver {
        Resolver { interpreter }
    }
}

// impl ExprVisitor<InterpretResult<()>> for Resolver {

// }

// impl StmtVisitor<InterpretResult<()>> for Resolver {

// }
