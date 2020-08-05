# Challenges

## Implementing `continue`
- Make `RuntimeError` a member of a general `InterpreterExceptionalState` (or whatever) enumeration.
- Add `BreakEvent` or something to that enum.
- When a break statement is executed, return `Err(InterpreterExceptionalState::BreakEvent)`
- In visit_while_loop match the return value from `self.execute(body)` and if it's BreakEvent, exit the loop, if it's an error, pass it up.