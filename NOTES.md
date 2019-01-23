# NOTES

## Chapter 10

### Lambda challenge notes
Look at `Object visitVariableExpr(Expr.Variable expr)` - it returns the defined object. We don't need to assign a name, just treat a lambda like a variable and return a new LambdaCallable

So, we do the following:
- Make a `LoxLambda` modeling the `LoxFunction`
- `Interpreter::visitLambdaExpr` and return a new LoxLambda
- `Parser` need to catch lambdas as `assignment` so we can say `var f = fun(){}` and we need to catch lambdas as `primary` so they can be caught in a function `call()` expression.

Per the challenge notes, how to handle a `fun(){};` in the context of a statement? 
- I'm happy to treat it as an error! A lambda which isn't called, nor stored in a var would simply be created and discarded. **So it's an error**.

What about `(fun(){...})();` - this should be allowed to work.

## Chapter 11
Q3: The Resolver looks like the right place to throw an error if a break statement occurs without an enclosing while loop. My current implementation will break out of nested function calls to the closest while loop, which is neat, but bad. I should copy Resolver::resolveFunction's approach to keep track of whether the current function context has a while loop in it.  

Q4: Where to store the index when keeping variables in an array instead of a hashmap? Looks like `Interpreter::locals` `Map<Expr,Integer>` would be a good fit. But we should store `class VarInfo { int depth; int offset; }`