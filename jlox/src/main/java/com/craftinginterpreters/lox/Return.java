package com.craftinginterpreters.lox;

class Return extends RuntimeException{
    private static final long serialVersionUID = 1L;
    final Object value;

    Return(Object value) {
        super(null, null, false, false);
        this.value = value;
    }
}
