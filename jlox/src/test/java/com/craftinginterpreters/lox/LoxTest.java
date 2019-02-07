package com.craftinginterpreters.lox;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;
import static org.junit.Assert.fail;

import java.io.File;
import java.io.IOException;
import java.util.ArrayList;
import java.util.List;

import org.junit.After;
import org.junit.Before;
import org.junit.Test;

public class LoxTest {

    /**
     * Adds assertTrue() and assertFalse() to Lox global namespace
     */
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
                boolean condition = Interpreter.isTruthy(arguments.get(1));
                assertTrue(message, condition);
                return null;
            }
        });

        Lox.interpreter.defineGlobal("assertFalse", new LoxCallable() {

            @Override
            public int arity() {
                return 2;
            }

            @Override
            public Object call(Interpreter interpreter, List<Object> arguments) {
                String message = (String) arguments.get(0);
                boolean condition = Interpreter.isTruthy(arguments.get(1));
                assertFalse(message, condition);
                return null;
            }
        });

        Lox.interpreter.defineGlobal("assertEquals", new LoxCallable() {

            @Override
            public int arity() {
                return 3;
            }

            @Override
            public Object call(Interpreter interpreter, List<Object> arguments) {
                String message = (String) arguments.get(0);
                Object expected = arguments.get(1);
                Object actual = arguments.get(2);
                assertEquals(message, expected, actual);
                return null;
            }
        });        
    }

    @After
    public void confirmErrorFreeExecution() {
        assertFalse("Expect no errors", Lox.hadError);
        assertFalse("Expect no runtime errors", Lox.hadRuntimeError);
    }

    @Test
    public void testInterpreterAssert() {
        // confirm that our assertTrue and assertFalse work with the language spec
        Lox.run("assertTrue(\"truth is truthy\", true);");
        Lox.run("assertTrue(\"one is truthy\", 1);");
        Lox.run("assertTrue(\"zero is truthy\", 0);"); // non-nil is true!
        Lox.run("assertFalse(\"false is falsy\", false);");
        Lox.run("assertFalse(\"nil is falsy\", nil);");
    }

    @Test
    public void testLoxTestSuite() {
        for (String loxTestFile : getLoxTestSuite()) {
            System.out.println("Running " + loxTestFile);
            runFile(loxTestFile);
        }
    }

    //
    //  Privates
    //

    private static String LOX_TEST_FILES_DIR = "src/test/lox/";

    private List<String> getLoxTestSuite() {
        String[] files = new File(LOX_TEST_FILES_DIR).list();
        List<String> loxFiles = new ArrayList<>();
        for (String file : files) {
            if (file.endsWith(".lox")) {
                loxFiles.add(LOX_TEST_FILES_DIR + file);
            }
        }
        return loxFiles;
    }

    private void runFile(String pathToLoxFile) {
        try {
            Lox.runFile(pathToLoxFile);
        } catch (IOException e) {
            fail(e.toString());
        }
    }
}
