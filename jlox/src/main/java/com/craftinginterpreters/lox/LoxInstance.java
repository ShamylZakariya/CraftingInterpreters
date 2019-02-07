package com.craftinginterpreters.lox;

import java.util.HashMap;
import java.util.Map;

public class LoxInstance {
    private LoxClass klass;
    private final Map<String, Object> fields = new HashMap<>();

    public LoxInstance(LoxClass klass) {
        this.klass = klass;
    }

    public LoxInstance() {
        klass = null;
    }

    Object get(Token name) {
        if (fields.containsKey(name.lexeme)) {
            return fields.get(name.lexeme);
        }

        LoxFunction method = klass.findMethod(this, name.lexeme);
        if (method != null) {
            return method;
        }

        throw new RuntimeError(name, "Undefined property '" + name.lexeme + "'.");
    }

    void set(Token name, Object value) {
        fields.put(name.lexeme, value);
    }

    LoxFunction getProperty(Token name) {
        if (klass == null) {
            return null;
        }

        LoxFunction property = klass.findProperty(this, name.lexeme);
        if (property != null) {
            return property;
        }

        return null;
    }

    @Override
    public String toString() {
        return "<" + klass.name + " instance>";
    }
}
