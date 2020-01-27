package com.craftinginterpreters.lox;

/**
 * Exception thrown to exit a loop. See Interpreter.visitBreakStmt
 */
class Break extends RuntimeException {
    private static final long serialVersionUID = 1L;

    public Break() {
        super(null, null, false, false);
    }
}
