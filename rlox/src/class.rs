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
    class_fields: HashMap<String, LoxObject>,
    class_methods: HashMap<String, Rc<RefCell<LoxFunction>>>,
    super_class: Option<LoxClass>,
}

impl ClassData {
    pub fn find_method(&self, name: &str) -> Option<Rc<RefCell<LoxFunction>>> {
        if let Some(function) = self.methods.get(name) {
            Some(function.clone())
        } else if let Some(super_class) = &self.super_class {
            super_class.class_data.borrow().find_method(name)
        }
         else {
            None
        }
    }
    pub fn find_class_method(&self, name: &str) -> Option<Rc<RefCell<LoxFunction>>> {
        if let Some(function) = self.class_methods.get(name) {
            Some(function.clone())
        } else if let Some(super_class) = &self.super_class {
            super_class.class_data.borrow().find_class_method(name)
        } else {
            None
        }
    }
}

pub struct LoxClass {
    class_data: Rc<RefCell<ClassData>>,
}

impl LoxClass {
    pub fn new(
        name: &str,
        super_class: Option<LoxClass>,
        methods: HashMap<String, Rc<RefCell<LoxFunction>>>,
        class_methods: HashMap<String, Rc<RefCell<LoxFunction>>>,
    ) -> Self {
        LoxClass {
            class_data: Rc::new(RefCell::new(ClassData {
                name: String::from(name),
                methods,
                class_fields: HashMap::new(),
                class_methods,
                super_class: super_class
            }))
        }
    }

    pub fn get(&self, name: &Token) -> Result<LoxObject> {
        if let Some(obj) = self.class_data.borrow().class_fields.get(&name.lexeme) {
            Ok(obj.clone())
        } else if let Some(method) = self.class_data.borrow().find_class_method(&name.lexeme) {
            Ok(LoxObject::Callable(method))
        } else {
            Err(RuntimeError::new(
                name,
                &format!("Undefined variable \"{}\".", name.lexeme),
            ))
        }
    }

    pub fn set(&self, name: &Token, value: &LoxObject) {
        self.class_data
            .borrow_mut()
            .class_fields
            .insert(name.lexeme.to_owned(), value.clone());
    }
}

impl Clone for LoxClass {
    fn clone(&self) -> Self {
        LoxClass {
            class_data: self.class_data.clone()
        }
    }
}

impl fmt::Debug for LoxClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<class \"{}\">", self.class_data.borrow().name)
    }
}

impl fmt::Display for LoxClass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.class_data.borrow().name)
    }
}

impl PartialEq<LoxClass> for LoxClass {
    fn eq(&self, other: &Self) -> bool {
        &self == &other
    }
}

impl LoxCallable for LoxClass {
    fn arity(&self) -> usize {
        if let Some(initializer) = self.class_data.borrow().find_method("init") {
            initializer.borrow().arity()
        } else {
            0
        }
    }

    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: &Vec<LoxObject>,
    ) -> InterpretResult<Option<LoxObject>> {
        let instance = LoxInstance::new(self.class_data.clone());
        if let Some(initializer) = self.class_data.borrow().find_method("init") {
            let bound = initializer.borrow().bind(&instance);
            bound.call(interpreter, arguments)?;
        }
        Ok(Some(LoxObject::Instance(instance)))
    }

    fn is_property(&self) -> bool {
        false
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
                &format!("Undefined property \"{}\".", name.lexeme),
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
