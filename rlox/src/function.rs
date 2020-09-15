use std::fmt;

use crate::ast::{CallableType, Stmt};
use crate::callable::LoxCallable;
use crate::class::LoxInstance;
use crate::environment::Environment;
use crate::interpreter::{InterpretResult, InterpretResultStatus, Interpreter};
use crate::object::LoxObject;
use crate::scanner::Token;

pub struct LoxFunction {
    name: Option<Token>,
    parameters: Vec<Token>,
    body: Vec<Box<Stmt>>,
    closure: Environment,
    is_initializer: bool,
    fn_type: CallableType,
}

impl LoxFunction {
    pub fn new_function(
        name: &Token,
        parameters: &Vec<Token>,
        body: &Vec<Box<Stmt>>,
        closure: Environment,
        is_initializer: bool,
        fn_type: CallableType,
    ) -> Self {
        LoxFunction {
            name: Some(name.clone()),
            parameters: parameters.clone(),
            body: body.clone(),
            closure,
            is_initializer,
            fn_type,
        }
    }

    pub fn new_lambda(
        parameters: &Vec<Token>,
        body: &Vec<Box<Stmt>>,
        closure: Environment,
    ) -> Self {
        LoxFunction {
            name: None,
            parameters: parameters.clone(),
            body: body.clone(),
            closure: closure,
            is_initializer: false,
            fn_type: CallableType::Lambda,
        }
    }

    pub fn bind(&self, instance: &LoxInstance) -> LoxFunction {
        let mut environment = Environment::as_child_of(self.closure.clone());
        environment.define("this", &LoxObject::Instance(instance.clone()));
        if let Some(name) = &self.name {
            LoxFunction::new_function(
                &name,
                &self.parameters,
                &self.body,
                environment,
                self.is_initializer,
                self.fn_type,
            )
        } else {
            panic!("Attempted to call bind() on a lambda.");
        }
    }
}

impl fmt::Display for LoxFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(name) = &self.name {
            write!(f, "{}", name)
        } else {
            write!(f, "lambda")
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
            // if function doesn't explicitly call return, we return None for it, unless
            // it's a class instance initializer in which case we return 'this'
            Ok(()) => {
                if self.is_initializer {
                    Ok(Some(self.closure.get_at(0, "this")?))
                } else {
                    Ok(None)
                }
            }
            Err(e) => match e {
                InterpretResultStatus::Return(v) => {
                    if self.is_initializer {
                        // Any call to return from an initializer will return 'this'
                        // Note: Resolver disallows explicit value return from initializers,
                        // so we know this will only occur for empty `return;` calls.
                        Ok(Some(self.closure.get_at(0, "this")?))
                    } else {
                        match v {
                            Some(v) => Ok(Some(v)),
                            None => Ok(None),
                        }
                    }
                }
                _ => Err(e),
            },
        }
    }

    fn is_property(&self) -> bool {
        match self.fn_type {
            CallableType::Property => true,
            _ => false,
        }
    }
}
