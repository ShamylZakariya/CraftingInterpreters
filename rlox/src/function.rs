use crate::callable::LoxCallable;
use crate::environment::Environment;
use crate::interpreter::{Interpreter, InterpretResult, InterpretResultStatus};
use crate::object::LoxObject;
use crate::scanner::Token;
use crate::ast::Stmt;

pub struct LoxFunction {
    _name: Option<Token>,
    parameters: Vec<Token>,
    body: Vec<Box<Stmt>>,
    closure: Environment,
}

impl LoxFunction {
    pub fn new_function(
        name: &Token,
        parameters: &Vec<Token>,
        body: &Vec<Box<Stmt>>,
        closure: Environment,
    ) -> Self {
        LoxFunction {
            _name: Some(name.clone()),
            parameters: parameters.clone(),
            body: body.clone(),
            closure: closure,
        }
    }

    pub fn new_lambda(parameters: &Vec<Token>, body: &Vec<Box<Stmt>>, closure: Environment) -> Self {
        LoxFunction {
            _name: None,
            parameters: parameters.clone(),
            body: body.clone(),
            closure: closure,
        }
    }
}

impl LoxCallable for LoxFunction {
    fn arity(&self) -> usize {
        return self.parameters.len();
    }

    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: &Vec<LoxObject>,
    ) -> InterpretResult<Option<LoxObject>> {
        let mut env = Environment::as_child_of(self.closure.clone());
        for i in 0..self.parameters.len() {
            env.define(&self.parameters[i].lexeme, &arguments[i]);
        }

        let ret = interpreter.execute_block(&self.body, env);
        match ret {
            // if function doesn't explicitly call return, we return None for it
            Ok(()) => Ok(None),
            Err(e) => match e {
                InterpretResultStatus::Return(v) => match v {
                    Some(v) => Ok(Some(v)),
                    None => Ok(None),
                },
                _ => Err(e),
            },
        }
    }
}