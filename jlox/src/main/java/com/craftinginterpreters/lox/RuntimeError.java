package com.craftinginterpreters.lox;

public class RuntimeError extends RuntimeException {
    private static final long serialVersionUID = 1L;
    final Token token;

    public RuntimeError(Token token, String message) {
        super(message);
        this.token = token;
    }
}
