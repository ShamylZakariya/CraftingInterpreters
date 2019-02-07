package com.craftinginterpreters.lox;

import java.util.List;

public class LoxLambda implements LoxCallable {
    private final Expr.Lambda lambda;
    private final Environment closure;

    public LoxLambda(Expr.Lambda lambda, Environment closure) {
        this.lambda = lambda;
        this.closure = closure;
    }

    @Override
    public int arity() {
        return lambda.params.size();
    }

    @Override
    public Object call(Interpreter interpreter, List<Object> arguments) {
        Environment environment = new Environment(closure);
        for (int i = 0; i < lambda.params.size(); i++) {
            environment.define(lambda.params.get(i).lexeme, arguments.get(i));
        }

        try {
            interpreter.executeBlock(lambda.body, environment);
        } catch(Return returnValue) {
            return returnValue.value;
        }
        return null;
    }

    @Override
    public String toString() {
        return "<lambda>";
    }
}
