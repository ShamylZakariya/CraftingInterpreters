package com.craftinginterpreters.lox;

class Break extends RuntimeException {
    public Break() {
        super(null, null, false, false);
    }
}
