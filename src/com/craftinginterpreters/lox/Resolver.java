package com.craftinginterpreters.lox;

import java.util.*;

public class Resolver implements Expr.Visitor<Void>, Stmt.Visitor<Void> {

    private enum FunctionType {
        NONE,
        FUNCTION,
        LAMBDA
    }

    private enum VariableState {
        DECLARED,
        DEFINED,
        ASSIGNED,
        ACCESSED
    }

    private class VariableInfo {
        private VariableState state;
        private final Token name;

        public VariableInfo(Token name, VariableState state) {
            this.name = name;
            this.state = state;
        }

        public Token getName() {
            return name;
        }

        public VariableState getState() {
            return state;
        }

        public void setState(VariableState state) {
            this.state = state;
        }
    }

    private final Interpreter interpreter;
    private final Stack<Map<String, VariableInfo>> scopes = new Stack();
    private FunctionType currentFunction = FunctionType.NONE;
    private boolean whileStatementPresent = false;

    public Resolver(Interpreter interpreter) {
        this.interpreter = interpreter;
    }

    void resolve(List<Stmt> statements) {
        for (Stmt statement : statements) {
            resolve(statement);
        }
    }

    private void resolve(Stmt stmt) {
        stmt.accept(this);
    }

    private void resolve(Expr expr) {
        expr.accept(this);
    }

    private void resolveFunction(Stmt.Function function, FunctionType type) {
        FunctionType enclosingFunction = currentFunction;
        currentFunction = type;

        // on entry into a function, reset the presence of a while statement to false
        boolean enclosingWhileStatementPresent = whileStatementPresent;
        whileStatementPresent = false;

        beginScope();
        for (Token param : function.params) {
            declare(param);
            define(param, true);
        }
        resolve(function.body);
        endScope();

        whileStatementPresent = enclosingWhileStatementPresent;
        currentFunction = enclosingFunction;
    }

    private void resolveLambda(Expr.Lambda lambda) {
        FunctionType enclosingFunction = currentFunction;
        currentFunction = FunctionType.LAMBDA;

        // on entry into a function, reset the presence of a while statement to false
        boolean enclosingWhileStatementPresent = whileStatementPresent;
        whileStatementPresent = false;

        beginScope();
        for (Token param : lambda.params) {
            declare(param);
            define(param, true);
        }
        resolve(lambda.body);
        endScope();

        whileStatementPresent = enclosingWhileStatementPresent;
        currentFunction = enclosingFunction;
    }

    private void beginScope() {
        scopes.push(new HashMap<>());
    }

    private void endScope() {
        Map<String, VariableInfo> info = scopes.pop();
        for(String name : info.keySet()) {
            VariableInfo vi = info.get(name);
            if (vi.getState() != VariableState.ACCESSED) {
                Lox.error(vi.getName(), "Variable \"" + vi.getName().lexeme + "\" was defined but never accessed.");
            }
        }
    }

    private void declare(Token name) {
        if (scopes.isEmpty()) {
            return;
        }

        // by declaring a var, we say "it exists, but hasn't been written to, yet"
        Map<String, VariableInfo> scope = scopes.peek();

        if (scope.containsKey(name.lexeme)) {
            Lox.error(name, "Variable with this name already declared in this scope.");
        }

        scope.put(name.lexeme, new VariableInfo(name, VariableState.DECLARED));
    }

    private void define(Token name, boolean assignedTo) {
        if (scopes.isEmpty()) {
            return;
        }
        // now we save that the var is defined and ready to go
        scopes.peek().get(name.lexeme).setState(assignedTo ? VariableState.ASSIGNED : VariableState.DEFINED);
    }

    private void resolveLocal(Expr expr, Token name) {
        for (int i = scopes.size() - 1; i >= 0; i--) {
            VariableInfo info = scopes.get(i).get(name.lexeme);
            if (info != null) {
                info.setState(VariableState.ACCESSED);
                interpreter.resolve(expr, scopes.size() - 1 - i);
                break;
            }
        }

        // not found, so assume var is a global
    }

    @Override
    public Void visitBlockStmt(Stmt.Block stmt) {
        beginScope();
        resolve(stmt.statements);
        endScope();
        return null;
    }

    @Override
    public Void visitBreakStmt(Stmt.Break stmt) {

        if (!whileStatementPresent) {
            Lox.error(stmt.keyword, "Cannot break outside of a loop.");
        }

        return null;
    }

    @Override
    public Void visitExpressionStmt(Stmt.Expression stmt) {
        resolve(stmt.expression);
        return null;
    }

    @Override
    public Void visitFunctionStmt(Stmt.Function stmt) {
        declare(stmt.name);
        define(stmt.name, true);

        resolveFunction(stmt, FunctionType.FUNCTION);
        return null;
    }


    @Override
    public Void visitIfStmt(Stmt.If stmt) {
        resolve(stmt.condition);
        resolve(stmt.thenBranch);
        if (stmt.elseBranch != null) {
            resolve(stmt.elseBranch);
        }
        return null;
    }

    @Override
    public Void visitPrintStmt(Stmt.Print stmt) {
        resolve(stmt.expression);
        return null;
    }

    @Override
    public Void visitReturnStmt(Stmt.Return stmt) {

        if (currentFunction == FunctionType.NONE) {
            Lox.error(stmt.keyword, "Cannot return from top-level code.");
        }

        if (stmt.value != null) {
            resolve(stmt.value);
        }
        return null;
    }

    @Override
    public Void visitWhileStmt(Stmt.While stmt) {
        boolean enclosingWhileStatementPresent = whileStatementPresent;
        whileStatementPresent = true;

        resolve(stmt.condition);
        resolve(stmt.body);

        whileStatementPresent = enclosingWhileStatementPresent;

        return null;
    }

    @Override
    public Void visitVarStmt(Stmt.Var stmt) {
        declare(stmt.name);
        if (stmt.initializer != null) {
            resolve(stmt.initializer);
            define(stmt.name, true);
        } else {
            define(stmt.name, false);
        }
        return null;
    }

    @Override
    public Void visitAssignExpr(Expr.Assign expr) {
        resolve(expr.value);
        resolveLocal(expr, expr.name);
        return null;
    }

    @Override
    public Void visitBinaryExpr(Expr.Binary expr) {
        resolve(expr.left);
        resolve(expr.right);
        return null;
    }

    @Override
    public Void visitCallExpr(Expr.Call expr) {
        resolve(expr.callee);
        for (Expr argument : expr.arguments) {
            resolve(argument);
        }
        return null;
    }

    @Override
    public Void visitGroupingExpr(Expr.Grouping expr) {
        resolve(expr.expression);
        return null;
    }

    @Override
    public Void visitLambdaExpr(Expr.Lambda expr) {
        resolveLambda(expr);
        return null;
    }

    @Override
    public Void visitLiteralExpr(Expr.Literal expr) {
        return null;
    }

    @Override
    public Void visitLogicalExpr(Expr.Logical expr) {
        resolve(expr.left);
        resolve(expr.right);
        return null;
    }

    @Override
    public Void visitTernaryExpr(Expr.Ternary expr) {
        resolve(expr.condition);
        resolve(expr.thenBranch);
        resolve(expr.elseBranch);
        return null;
    }

    @Override
    public Void visitUnaryExpr(Expr.Unary expr) {
        resolve(expr.right);
        return null;
    }

    @Override
    public Void visitVariableExpr(Expr.Variable expr) {
        if (!scopes.isEmpty()) {
            VariableInfo info = scopes.peek().get(expr.name.lexeme);
            if (info != null) {
                switch(info.getState()) {
                    case DECLARED:
                        Lox.error(expr.name, "Cannot read local variable in its own initializer.");
                        break;
                    case DEFINED:
                        Lox.error(expr.name, "Cannot read from unassigned local variable.");
                        break;
                    case ASSIGNED:
                        break;
                    case ACCESSED:
                        break;
                }
            }
        }

        resolveLocal(expr, expr.name);
        return null;
    }

}
