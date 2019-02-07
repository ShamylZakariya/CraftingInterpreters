package com.craftinginterpreters.lox;

import java.util.List;
import java.util.Map;

public class LoxClass extends LoxInstance implements LoxCallable {
    final String name;
    private final Map<String, LoxFunction> properties;
    private final Map<String, LoxFunction> methods;
    private final Map<String, LoxFunction> classMethods;

    public LoxClass(String name, Map<String, LoxFunction> properties, Map<String, LoxFunction> methods, Map<String,LoxFunction> classMethods) {
        super();
        this.name = name;
        this.properties = properties;
        this.methods = methods;
        this.classMethods = classMethods;
    }

    @Override
    Object get(Token name) {
        // attempt to find this class method - note we bind 'this' to the class, not an instance!
        if (classMethods.containsKey(name.lexeme)) {
            return classMethods.get(name.lexeme).bind(this);
        }

        throw new RuntimeError(name, "Undefined class method '" + name.lexeme + "' on class '" + this.name + "'.");
    }

    LoxFunction findProperty(LoxInstance instance, String name) {
        if (properties.containsKey(name)) {
            return properties.get(name).bind(instance);
        }

        return null;
    }

    LoxFunction findMethod(LoxInstance instance, String name) {
        if (methods.containsKey(name)) {
            return methods.get(name).bind(instance);
        }

        return null;
    }

    @Override
    public String toString() {
        return "<class " + name + ">";
    }

    @Override
    public int arity() {
        LoxFunction initializer = methods.get("init");
        if (initializer == null) {
            return 0;
        }
        return initializer.arity();
    }

    @Override
    public Object call(Interpreter interpreter, List<Object> arguments) {
        LoxInstance instance = new LoxInstance(this);

        // if the class defines an init() method, invoke it
        LoxFunction initializer = methods.get("init");
        if (initializer != null) {
            initializer.bind(instance).call(interpreter, arguments);
        }

        return instance;
    }
}
