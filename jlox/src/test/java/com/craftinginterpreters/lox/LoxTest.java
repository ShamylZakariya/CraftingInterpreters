package com.craftinginterpreters.lox;

import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;

import java.util.List;

import org.junit.Before;
import org.junit.Test;

public class LoxTest {

    @Before
    public void setUpLoxEnvironment() {
        Lox.reset();
        Lox.interpreter.defineGlobal("assertTrue", new LoxCallable() {

            @Override
            public int arity() {
                return 2;
            }

            @Override
            public Object call(Interpreter interpreter, List<Object> arguments) {
                String message = (String) arguments.get(0);
                boolean condition = (Boolean) arguments.get(1);
                assertTrue(message, condition);
                return null;
            }
        });
    }

    @Test
    public void testInterpreterAssert() {
        Lox.run("assertTrue(\"truth is truthy\", true);");
        assertFalse("Expect no errors", Lox.hadError);
        assertFalse("Expect no runtime errors", Lox.hadRuntimeError);
    }
}
