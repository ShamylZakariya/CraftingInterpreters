use std::collections::HashMap;
use std::fmt;
use std::{cell::RefCell, rc::Rc};

use crate::callable::LoxCallable;
use crate::error::RuntimeError;
use crate::function::LoxFunction;
use crate::interpreter::{InterpretResult, Interpreter, Result};
use crate::object::LoxObject;
use crate::scanner::Token;

pub struct ClassData {
    name: String,
    methods: HashMap<String, Rc<RefCell<LoxFunction>>>,
}

impl ClassData {
    pub fn find_method(&self, name: &str) -> Option<Rc<RefCell<LoxFunction>>> {
        if let Some(function) = self.methods.get(name) {
            Some(function.clone())
        } else {
            None
        }
    }
}

pub struct LoxClass(Rc<RefCell<ClassData>>);

impl LoxClass {
    pub fn new(name: &str, methods: HashMap<String, Rc<RefCell<LoxFunction>>>) -> Self {
        LoxClass(Rc::new(RefCell::new(ClassData {
            name: String::from(name),
            methods: methods,
        })))
    }
}

impl Clone for LoxClass {
    fn clone(&self) -> Self {
        LoxClass(self.0.clone())
    }
}

impl fmt::Debug for LoxClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<class \"{}\">", self.0.borrow().name)
    }
}

impl fmt::Display for LoxClass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.borrow().name)
    }
}

impl PartialEq<LoxClass> for LoxClass {
    fn eq(&self, other: &Self) -> bool {
        &self == &other
    }
}

impl LoxCallable for LoxClass {
    fn arity(&self) -> usize {
        return 0;
    }

    fn call(
        &self,
        _interpreter: &mut Interpreter,
        _arguments: &Vec<LoxObject>,
    ) -> InterpretResult<Option<LoxObject>> {
        Ok(Some(LoxObject::Instance(LoxInstance::new(self.0.clone()))))
    }
}

// --------------------------------------------------------------------------------------------------------------------

pub struct LoxInstance {
    class_data: Rc<RefCell<ClassData>>,
    fields: Rc<RefCell<HashMap<String, LoxObject>>>,
}

impl LoxInstance {
    pub fn new(class_data: Rc<RefCell<ClassData>>) -> Self {
        LoxInstance {
            class_data,
            fields: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn get(&self, name: &Token) -> Result<LoxObject> {
        if let Some(obj) = self.fields.borrow().get(&name.lexeme) {
            Ok(obj.clone())
        } else if let Some(method) = self.class_data.borrow().find_method(&name.lexeme) {
            Ok(LoxObject::Callable(Rc::new(RefCell::new(
                method.borrow().bind(self),
            ))))
        } else {
            Err(RuntimeError::new(
                name,
                &format!("Undefined variable \"{}\".", name.lexeme),
            ))
        }
    }

    pub fn set(&self, name: &Token, value: &LoxObject) {
        self.fields
            .borrow_mut()
            .insert(name.lexeme.to_owned(), value.clone());
    }
}

impl Clone for LoxInstance {
    fn clone(&self) -> Self {
        LoxInstance {
            class_data: self.class_data.clone(),
            fields: self.fields.clone(),
        }
    }
}

impl fmt::Debug for LoxInstance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<instance of \"{}\">", self.class_data.borrow().name)
    }
}

impl fmt::Display for LoxInstance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} instance", self.class_data.borrow().name)
    }
}

impl PartialEq<LoxInstance> for LoxInstance {
    fn eq(&self, other: &Self) -> bool {
        &self == &other
    }
}
