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
The Resolver looks like the right place to throw an error if a break statement occurs without an enclosing while loop. My current implementation will break out of nested function calls to the closest while loop, which is neat, but bad. I should copy Resolver::resolveFunction's approach to keep track of whether the current function context has a while loop in it.  

Q3: TO make resolver report an error if a local variable is never used. `scopes` is turned into a map of VariableInfo which has the `Token` and also a `VariableState` enum which records variable lifetime - `DECLARED, DEFINED, ASSIGNED, ACCESSED`. We update this state in the resolver and when a scope is ended, we report any variables who's state is not `ACCESSED`.
    - For consideration - a class object constructor may be run and assigned to a variable and never accessed, since it may be desirable for the class's constructor to perform some kind of event with side effects (global state, mutex lock, etc). Perhaps this should be changed to report error for variables created but not assigned-to. 
    - Note: I've made the decision that it is an error for a function to have arguments which are unused. 

Q4: Where to store the index when keeping variables in an array instead of a hashmap? Looks like `Interpreter::locals` `Map<Expr,Integer>` would be a good fit. But we should store `class VarInfo { int depth; int offset; }`
    - `Interpreter::resolve` - I need to perform a defineAt() type call on the appropriate Environment to get an offset.