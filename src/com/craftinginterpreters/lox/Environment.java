package com.craftinginterpreters.lox;

import java.util.HashMap;
import java.util.Map;

public class Environment {

    private static final Object UNASSIGNED_SENTINEL = new Object();

    private final Environment enclosing;
    private final Map<String, Object> values = new HashMap<>();

    Environment() {
        enclosing = null;
    }

    Environment(Environment enclosing) {
        this.enclosing = enclosing;
    }

    Object get(Token name) {
        if (values.containsKey(name.lexeme)) {
            Object value = values.get(name.lexeme);
            if (value == UNASSIGNED_SENTINEL) {
                throw new RuntimeError(name, "Attempt to access uninitialized/unassigned variable.");
            }
            return value;
        }

        if (enclosing != null) {
            return enclosing.get(name);
        }

        throw new RuntimeError(name, "Undefined variable '" + name.lexeme + "'.");
    }

    Object getAt(int distance, Token name) {
        Environment anc = ancestor(distance);
        assert anc.values.containsKey(name.lexeme) : "Interpreter and Resolver out of sync - interpreter's environment should have variable" + name;
        Object value = anc.values.get(name.lexeme);

        if (value == UNASSIGNED_SENTINEL) {
            throw new RuntimeError(name, "Attempt to access uninitialized/unassigned variable.");
        }

        return value;
    }

    void define(String name, Object value) {
        values.put(name, value != null ? value : UNASSIGNED_SENTINEL);
    }

    Environment ancestor(int distance) {
        Environment env = this;
        for (int i = 0; i < distance; i++) {
            env = env.enclosing;
        }
        return env;
    }

    void assign(Token name, Object value) {
        if (values.containsKey(name.lexeme)) {
            values.put(name.lexeme, value);
            return;
        }

        if (enclosing != null) {
            enclosing.assign(name, value);
            return;
        }

        throw new RuntimeError(name, "Undefined variable '" + name.lexeme + "'.");
    }

    void assignAt(int distance, Token name, Object value) {
        ancestor(distance).values.put(name.lexeme, value);
    }


}
