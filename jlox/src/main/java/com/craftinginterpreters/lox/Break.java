package com.craftinginterpreters.lox;

class Break extends RuntimeException {
    private static final long serialVersionUID = 1L;

    public Break() {
        super(null, null, false, false);
    }
}
