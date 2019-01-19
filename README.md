# Molt -- More Or Less TCL

The general notion is to build a minimal version of TCL for embedding in Rust
apps.  See "Plans", below.

## TODO Items

* Extend main app to execute a script if given.
* Flesh out Rust tests
  * Design public API using `pub use` in `lib.rs`, so the examples read
    properly from the user's point of view.
* Implement minimal tcltest equivalent for testing the commands.
  * Main app will need to accept a script
* Add full command tests.
* Make molt::get_integer() parse the same varieties as Tcl_GetInt() does.
* Flesh out documentation
  * Including examples  

## Plans

The goal is to produce a command language using basic TCL syntax
(e.g., commands, strings, and variable and command interpolation) that is
embeddable in Rust applications and extensible in Rust.  It will include
a "molt" application that provides script execution and an interactive
shell, but this is intended as an example and development aid, rather than
as a tool be used on its own. (Famous last words....)

### Initial Goals:

* Embedding, script execution, and an interactive shell.
* Basic parsing, as in TCL 7.6.
* Support for lists and dicts
* A smattering of basic commands
* Procs
* Expression parsing.
* A simple tcltest equivalent.
* The embedding API will allow the client to easily control the set of
  standard commands included in an interpreter.

### Excluded Features

The following features of modern TCL are currently off of the table:

* Namespaces
* Traces
* Slave interpreters
* The majority of standard TCL commands.
* Byte-compiling

Ultimately I'll want to add something like byte-compilation for speed; but
I want to have a test suite in place first.

### Specific Differences from TCL

The following are specific differences from TCL 8 not explicitly stated
above:

* Integer literals beginning with "0" are NOT assumed to be octal.
  Nor will they ever be.
* The encoding is currently always UTF-8.
* In `$name`, the name may include underscores and any character that
  Rust considers to be alphanumeric.
