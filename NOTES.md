# NOTES

## Chapter 10

### Lambda challenge notes
Look at `Object visitVariableExpr(Expr.Variable expr)` - it returns the defined object. We don't need to assign a name, just treat a lambda like a variable and return a new LambdaCallable

So, we do the following:
- Make a `LoxLambda` modeling the `LoxFunction`
- `Interpreter::visitLambdaExpr` and return a new LoxLambda
- `Parser` need to catch lambdas as `assignment` so we can say `var f = fun(){}` and we need to catch lambdas as `primary` so they can be caught in a function `call()` expression.

