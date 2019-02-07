package com.craftinginterpreters.lox;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

public class Interpreter implements Expr.Visitor<Object>, Stmt.Visitor<Void> {

    protected final Environment globals = new Environment();
    private Environment environment = globals;
    private final Map<Expr, Integer> locals = new HashMap<>();

    public Interpreter() {
        this(null);
    }

    public Interpreter(Map<String, Object> globals) {
        defineGlobal("clock", new LoxCallable() {
            @Override
            public int arity() {
                return 0;
            }

            @Override
            public Object call(Interpreter interpreter, List<Object> arguments) {
                return (double) System.currentTimeMillis();
            }

            @Override
            public String toString() {
                return "<native fn clock()>";
            }
        });

        if (globals != null) {
            for (Map.Entry<String, Object> operation : globals.entrySet()) {
                defineGlobal(operation.getKey(), operation.getValue());
            }
        }
    }

    void defineGlobal(String name, Object value) {
        globals.define(name, value);
    }

    Object getValue(String name) {
        return environment.get(name);
    }

    void interpret(List<Stmt> statements) {
        try {
            for (Stmt statement : statements) {
                execute(statement);
            }
        } catch (RuntimeError error) {
            Lox.runtimeError(error);
        }
    }

    @Override
    public Void visitBlockStmt(Stmt.Block stmt) {
        executeBlock(stmt.statements, new Environment(environment));
        return null;
    }

    @Override
    public Void visitBreakStmt(Stmt.Break stmt) {
        throw new Break();
    }

    @Override
    public Void visitClassStmt(Stmt.Class stmt) {
        environment.define(stmt.name.lexeme, null);

        Map<String, LoxFunction> properties = new HashMap<>();
        for (Stmt.Function property : stmt.properties) {
            LoxFunction propertyFunction = new LoxFunction(property, environment, false);
            properties.put(property.name.lexeme, propertyFunction);
        }

        Map<String, LoxFunction> methods = new HashMap<>();
        for (Stmt.Function method : stmt.methods) {
            boolean isClassInitializer = method.name.lexeme.equals("init");
            LoxFunction function = new LoxFunction(method, environment, isClassInitializer);
            methods.put(method.name.lexeme, function);
        }

        Map<String, LoxFunction> classMethods = new HashMap<>();
        for (Stmt.Function classMethod : stmt.classMethods) {
            LoxFunction function = new LoxFunction(classMethod, environment, false);
            classMethods.put(classMethod.name.lexeme, function);
        }

        LoxClass klass = new LoxClass(stmt.name.lexeme, properties, methods, classMethods);
        environment.assign(stmt.name, klass);
        return null;
    }

    @Override
    public Void visitExpressionStmt(Stmt.Expression stmt) {
        Object result = evaluate(stmt.expression);
        if (Lox.isRepl) {
            System.out.println(stringify(result));
        }
        return null;
    }

    @Override
    public Void visitFunctionStmt(Stmt.Function stmt) {
        LoxFunction function = new LoxFunction(stmt, environment, false);
        environment.define(stmt.name.lexeme, function);
        return null;
    }

    @Override
    public Void visitIfStmt(Stmt.If stmt) {
        if (isTruthy(evaluate(stmt.condition))) {
            execute(stmt.thenBranch);
        } else if (stmt.elseBranch != null) {
            execute(stmt.elseBranch);
        }
        return null;
    }

    @Override
    public Void visitPrintStmt(Stmt.Print stmt) {
        Object value = evaluate(stmt.expression);
        System.out.println(stringify(value));
        return null;
    }

    @Override
    public Void visitReturnStmt(Stmt.Return stmt) {
        Object value = null;
        if (stmt.value != null) {
            value = evaluate(stmt.value);
        }
        throw new Return(value);
    }

    @Override
    public Void visitVarStmt(Stmt.Var stmt) {
        Object value = null;
        if (stmt.initializer != null) {
            value = evaluate(stmt.initializer);
        }

        environment.define(stmt.name.lexeme, value);
        return null;
    }

    @Override
    public Void visitWhileStmt(Stmt.While stmt) {
        while (isTruthy(evaluate(stmt.condition))) {
            try {
                execute(stmt.body);
            } catch (Break e) {
                break;
            }
        }
        return null;
    }

    @Override
    public Object visitAssignExpr(Expr.Assign expr) {
        Object value = evaluate(expr.value);

        Integer distance = locals.get(expr);
        if (distance != null) {
            environment.assignAt(distance, expr.name, value);
        } else {
            globals.assign(expr.name, value);
        }
        return value;
    }

    @Override
    public Object visitBinaryExpr(Expr.Binary expr) {
        Object left = evaluate(expr.left);
        Object right = evaluate(expr.right);

        switch (expr.operator.type) {
        case MINUS:
            checkNumberOperands(expr.operator, left, right);
            return (double) left - (double) right;
        case SLASH:
            checkNumberOperands(expr.operator, left, right);
            if ((double) right == 0) {
                throw new RuntimeError(expr.operator, "Attempted divide by zero");
            }
            return (double) left / (double) right;
        case STAR:
            checkNumberOperands(expr.operator, left, right);
            return (double) left * (double) right;
        case PLUS:
            if (left instanceof Double && right instanceof Double) {
                return (double) left + (double) right;
            }
            if (left instanceof String) {
                return (String) left + stringify(right);
            }
            throw new RuntimeError(expr.operator, "Operands must be two numbers or two strings");
        case GREATER:
            checkNumberOperands(expr.operator, left, right);
            return (double) left > (double) right;
        case GREATER_EQUAL:
            checkNumberOperands(expr.operator, left, right);
            return (double) left >= (double) right;
        case LESS:
            checkNumberOperands(expr.operator, left, right);
            return (double) left < (double) right;
        case LESS_EQUAL:
            checkNumberOperands(expr.operator, left, right);
            return (double) left <= (double) right;
        case BANG_EQUAL:
            return !isEqual(left, right);
        case EQUAL_EQUAL:
            return isEqual(left, right);
        default:
            throw new RuntimeError(expr.operator, "visitBinaryExpr on incorrect token.");
        }
    }

    @Override
    public Object visitCallExpr(Expr.Call expr) {
        Object callee = evaluate(expr.callee);

        List<Object> arguments = new ArrayList<>();
        for (Expr argument : expr.arguments) {
            arguments.add(evaluate(argument));
        }

        if (!(callee instanceof LoxCallable)) {
            throw new RuntimeError(expr.paren, "Can only call functions and classes");
        }

        LoxCallable function = (LoxCallable) callee;

        if (arguments.size() != function.arity()) {
            throw new RuntimeError(expr.paren,
                    "Expected " + function.arity() + " arguments but got " + arguments.size());
        }

        return function.call(this, arguments);
    }

    @Override
    public Object visitGetExpr(Expr.Get expr) {
        Object object = evaluate(expr.object);
        if (object instanceof LoxInstance) {

            LoxFunction property = ((LoxInstance) object).getProperty(expr.name);
            if (property != null) {
                return property.call(this, null); // properties have no arguments
            }

            return ((LoxInstance) object).get(expr.name);
        }

        throw new RuntimeError(expr.name, "Only classes and class instances have properties.");
    }

    @Override
    public Object visitGroupingExpr(Expr.Grouping expr) {
        return evaluate(expr.expression);
    }

    @Override
    public Object visitLambdaExpr(Expr.Lambda expr) {
        return new LoxLambda(expr, environment);
    }

    @Override
    public Object visitLiteralExpr(Expr.Literal expr) {
        return expr.value;
    }

    @Override
    public Object visitLogicalExpr(Expr.Logical expr) {
        Object left = evaluate(expr.left);
        if (expr.operator.type == TokenType.OR) {
            if (isTruthy(left))
                return left;
        } else {
            if (!isTruthy(left))
                return left;
        }

        return evaluate(expr.right);
    }

    @Override
    public Object visitSetExpr(Expr.Set expr) {
        Object object = evaluate(expr.object);

        if (!(object instanceof LoxInstance)) {
            throw new RuntimeError(expr.name, "Only instances have fields.");
        }

        Object value = evaluate(expr.value);
        ((LoxInstance) object).set(expr.name, value);
        return value;
    }

    @Override
    public Object visitThisExpr(Expr.This expr) {
        return lookUpVariable(expr.keyword, expr);
    }

    @Override
    public Object visitTernaryExpr(Expr.Ternary expr) {
        if (isTruthy(evaluate(expr.condition))) {
            return evaluate(expr.thenBranch);
        }
        return evaluate(expr.elseBranch);
    }

    @Override
    public Object visitUnaryExpr(Expr.Unary expr) {
        Object right = evaluate(expr.right);
        switch (expr.operator.type) {
        case BANG:
            return !isTruthy(right);
        case MINUS:
            checkNumberOperand(expr.operator, right);
            return -(double) right;
        default:
            break;
        }

        // unreachable
        return null;
    }

    @Override
    public Object visitVariableExpr(Expr.Variable expr) {
        return lookUpVariable(expr.name, expr);
    }

    private Object lookUpVariable(Token name, Expr expr) {
        Integer distance = locals.get(expr);
        if (distance != null) {
            return environment.getAt(distance, name);
        } else {
            return globals.get(name);
        }
    }

    private String stringify(Object object) {
        if (object == null) {
            return null;
        }

        if (object instanceof Double) {
            // workaround java adding .0 to integers stored in Doubles
            String text = object.toString();
            if (text.endsWith(".0")) {
                text = text.substring(0, text.length() - 2);
            }
            return text;
        }

        return object.toString();
    }

    private Object evaluate(Expr expr) {
        return expr.accept(this);
    }

    private void execute(Stmt statement) {
        statement.accept(this);
    }

    void resolve(Expr expr, int depth) {
        locals.put(expr, depth);
    }

    void executeBlock(List<Stmt> statements, Environment environment) {
        Environment previous = this.environment;
        try {
            this.environment = environment;
            for (Stmt statement : statements) {
                execute(statement);
            }
        } finally {
            this.environment = previous;
        }
    }

    static boolean isTruthy(Object object) {
        // false and nil are falsey, everything else is truthy
        if (object == null) {
            return false;
        }
        if (object instanceof Boolean) {
            return (boolean) object;
        }
        return true;
    }

    private boolean isEqual(Object a, Object b) {
        // nil is only == nil
        if (a == null && b == null) {
            return true;
        }
        if (a == null) {
            return false;
        }
        return a.equals(b);
    }

    private void checkNumberOperand(Token operator, Object operand) {
        if (operand instanceof Double)
            return;
        throw new RuntimeError(operator, "Operand must be a number.");
    }

    private void checkNumberOperands(Token operator, Object left, Object right) {
        if (left instanceof Double && right instanceof Double)
            return;
        throw new RuntimeError(operator, "Operands must be numbers.");
    }
}
