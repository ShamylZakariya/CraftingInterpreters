package com.craftinginterpreters.lox;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

public class Environment {
    private final Environment enclosing;
    private final Map<String, Object> valuesByName = new HashMap<>();
    private final List<Object> valuesList = new ArrayList<>();

    Environment() {
        enclosing = null;
    }

    Environment(Environment enclosing) {
        this.enclosing = enclosing;
    }

    Object get(Token name) {
        if (valuesByName.containsKey(name.lexeme)) {
            return valuesByName.get(name.lexeme);
        }

        if (enclosing != null) {
            return enclosing.get(name);
        }

        throw new RuntimeError(name, "Undefined variable '" + name.lexeme + "'.");
    }

    Object getAt(int distance, Token name) {
        System.out.println("Environment::getAt - using name-based variable lookup! OH NO!");
        Environment anc = ancestor(distance);
        assert anc.valuesByName.containsKey(name.lexeme) : "Interpreter and Resolver out of sync - interpreter's environment should have variable" + name;
        return anc.valuesByName.get(name.lexeme);
    }

    Object getAt(int distance, int offset) {
        Environment anc = ancestor(distance);
        return anc.valuesList.get(offset);
    }

    int define(String name, Object value) {
        valuesByName.put(name, value);
        valuesList.add(value );
        return valuesList.size() - 1;
    }

    Environment ancestor(int distance) {
        Environment env = this;
        for (int i = 0; i < distance; i++) {
            env = env.enclosing;
        }
        return env;
    }

    void assign(Token name, Object value) {
        if (valuesByName.containsKey(name.lexeme)) {
            valuesByName.put(name.lexeme, value);
            return;
        }

        if (enclosing != null) {
            enclosing.assign(name, value);
            return;
        }

        throw new RuntimeError(name, "Undefined variable '" + name.lexeme + "'.");
    }

    void assignAt(int distance, Token name, Object value) {
        System.out.println("Environment::assignAt - using name-based variable lookup! OH NO!");
        ancestor(distance).valuesByName.put(name.lexeme, value);
    }

    void assignAt(int distance, int offset, Object value) {
        ancestor(distance).valuesList.set(offset, value);
    }
}
