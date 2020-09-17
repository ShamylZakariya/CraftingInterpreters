# README

This is an implementation of the [Lox](https://craftinginterpreters.com) language AST interpreter in [Rust](https://rust-lang.org).

It is largely similar in design to the one described in chapters 1-13 of Crafting Interpreters, but given Rust's approach to dynamic method dispatch and lack of exceptions, many details of the implementation differ. For example, the AST is implemented here as an enum, rather than through polymorphism. Also, this implementation implements many of the chapter challenges, such as the ternary operator, class methods, getter properties, etc.