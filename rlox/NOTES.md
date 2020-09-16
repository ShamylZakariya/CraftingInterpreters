# TODO

LoxObject::Callable has visible Rc<RefCell<dyn LoxCallable>>) which is ugly and punts a lot of responsibility to users of it; is there a way to hide the Rc<RefCell<>> with trickery like we've done for Environment?
    - Could be done by making a thin struct which has Rc<RefCell<dyn LoxCallable>> as its only field, and which implements the LoxCallable trait, as well as Copy/clone/etc?
    - Problems:
        - What to name it? Trait already owns LoxCallable
        - Should it implement the trait? Will that work without dyn for dynamic dispatch? SHouldn't be an issue since it will be monomorphically represented, and will own the dyn instance.

Does LoxClass need to wrap ClassData? Can it just own that data?

# Thoughts

Environment::get and LoxInstance::Get return clones of LoxObject; this should be fine since Callable, Class and Instance are all ref-counted types, and it's fine to copy Boolean, Nil, Number, Str and Undefined...

