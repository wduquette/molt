# GCL -- Game Control Language

The general notion is to build a minimal version of TCL for embedding in Rust
apps.  We'll see how far I get.

Note: I'd call it "RCL", but someone has already taken that package name.

## TODO Items

* Implement full parser
  * Handle variable interpolation in `Interp::parse_*_word`.
  * Handle backslashes in `Interp::parse_*`.
  * Handle backslashes in Context::skip_comment()
* Flesh out Rust tests
  * Design public API using `pub use` in `lib.rs`, so the examples read
    properly from the user's point of view.
* Implement minimal tcltest equivalent for testing the commands.
  * Main app will need to accept a script
* Add full command tests.
* Flesh out documentation
  * Including examples  

## Plans

The goal is to produce a command language using basic TCL syntax
(e.g., commands, strings, and variable and command interpolation) that is
embeddable in Rust applications and extensible in Rust. The following
features of modern TCL are currently off of the table:

* Namespaces
* Traces
* Slave interpreters
* The majority of standard TCL commands.
* Byte-compiling

The API will allow the client to easily control the set of standard
commands included in an interpreter.

Initial tasks:

* Basic parsing, as in TCL 7.6.
* Support for lists and dicts
* A smattering of basic commands
* Procs
* Expression parsing.

Ultimately I'll want to add something like byte-compilation for speed; but
I want to have a test suite in place first.
