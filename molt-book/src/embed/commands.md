# Defining Commands

At base, a Molt command is a Rust function that performs some kind of work and optionally
returns a value in the context of a specific Rust interpreter.  There are four ways an
application (or library crate) can define application-specific Rust commands:

* As a simple Rust `CommandFunc` function
* As a Rust `ContextCommandFunc` function that can access application-specific context
* As a Rust `Command` object
* As a Molt procedure, or `proc`.
