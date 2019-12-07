# Embedding Molt

This chapter explains how to embed Molt in a Rust application.  There are several parts
to this:

*   Creating a Molt interpreter
*   [Defining application-specific Molt commands](./commands.md)
*   Invoking the interpreter to [evaluate Molt commands and scripts](./eval.md)

An application may execute scripts for its own purposes and arbitrary scripts defined by
the user.  One common pattern is to define a [shell application](./shell.md) the user
may use to execute their own scripts using the application-specific command set.  

It is also possible to define [Molt library crate](./library.md) that defines commands
for installation into an interpreter.

The initial step, creating a Molt interpreter, is trivially easy:

```rust
use molt::Interp;

let mut interp = Interp::new();

// Add application-specific commands
```

This creates an interpreter containing the standard set of Molt commands.  Alternatively,
you can create a completely empty interpreter and add just the commands you want:

```rust
use molt::Interp;

let mut interp = Interp::empty();

// Add application-specific commands
```

This is useful if you wish to use the Molt interpreter as a safe file parser.  

Eventually there will be an API for adding specific standard Molt commands back into an empty
interpreter so that the application can create a custom command set (e.g., including
variable access and control structures but excluding file I/O), but that hasn't yet
been implemented.

We'll cover the remaining topics in the following sections.
