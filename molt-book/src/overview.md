# Molt: More Or Less Tcl

**Molt 0.2.0** is a minimal implementation of the TCL language for embedding in Rust apps
and for scripting Rust libraries.  Molt is intended to be:

*   **Small in size.** Embedding Molt shouldn't greatly increase the size of the
    application.

*   **Small in language.** [Standard TCL](http://tcl-lang.org) has many features
    intended for building entire software systems.  Molt is intentionally
    limited to those needed for embedding.

*   **Small in dependencies.** Including the Molt interpreter in your project shouldn't
    drag in anything else--unless you ask for it.

*   **Easy to build.** Building Standard TCL is non-trivial.  Embedding
    Molt should be as simple as using any other crate.

*   **Easy to embed.** Extending Molt with TCL commands that wrap Rust APIs should
    be easy and simple.

Hence, perfect compatibility with Standard TCL is explicitly not a goal.  Many
features will not be implemented at all (e.g., octal literals); and others may
be implemented somewhat differently where a clearly better alternative exists
(e.g., `-nocomplain` will always be the normal behavior).  In addition, Molt will
prefer Rust standards where appropriate.

On the other hand, Molt is meant to be TCL (more or less), not simply a
"Tcl-like language", so gratuitous differences are to be avoided.  One of the
goals of this document is to carefully delineate:

*   The features that have not yet been implemented.
*   The features that likely will never be implemented.
*   Any small differences in behavior.
*   And especially, any features that have intentionally been implemented in
    a different way.

## What Molt Is For

Using Molt, you can:

*   Create a shell interpreter for scripting and interactive testing of your Rust crates.
*   Provide your Rust applications with an interactive REPL for debugging and
    administration.
*   Extend your Rust application with scripts provided at compile-time or at run-time.
*   Allow your users to script your applications and libraries.

See the [`molt-sample` repo](https://github.com/wduquette/molt-sample) for a sample Molt client
skeleton.

## New in Molt 0.2.2

### Dictionaries and the `dict` command

Molt now supports TCL dictionary values.  The [`dict`](ref/dict.md) command provides the
following subcommands:

*   dict create
*   dict exists
*   dict get
*   dict set
*   dict size

## New in Molt 0.2

### Associative Arrays

Molt now includes TCL's associative array variables:

```text
% set a(1) "Howdy"
Howdy
% set a(foo.bar) 5
5
% puts [array get a]
1 Howdy foo.bar 5
```

### The Expansion Operator

Molt now supports TCL's `{*}` operator, which expands a single
command argument into multiple arguments:

```text
% set a {a b c}
a b c
% list 1 2 $a 3 4
1 2 {a b c} 3 4
% list 1 2 {*}$a 3 4
1 2 a b c 3 4
```

### More `info` Subcommands

Molt now supports the following subcommands of the [`info`](ref/info.md) command:

* `info args`
* `info body`
* `info cmdtype`
* `info default`
* `info exists`
* `info globals` (no glob-filtering as yet)
* `info locals` (no glob-filtering as yet)
* `info procs`

### Rust API Change: Test Harness

The Molt test harness code has moved from `molt_shell:test_harness` to `molt::test_harness`,
so that it can be used in the `molt/tests/tcl_tests.rs` integration test.

### Rust API Change: Variable Access

The addition of array variables required changes to the `molt::Interp` struct's API for
setting and retrieving variables.  In particular, the `molt::Interp::var`,
`molt::Interp::set_var`, and `molt::Interp::set_and_return` methods now take the variable
name as a `&Value` rather than a `&str`; this simplifies client code, and means that most
commands implemented in Rust that work with variables don't need to care whether the
variable in question is a scalar or an array element.

### Rust API Change: Command Definition

Defining Molt commands in Rust has been simplified.  

First, the `Command` trait has been removed.  It was intended to provide a way to
attach context data to a command; but it was not very good for mutable data, and had
no way to share data among related commands (a common pattern).

Second, the interpreter's context cache has been improved.  Multiple commands can share a
context ID (and hence access to the shared context); and the cached data will be dropped
automatically when the last such command is removed from the interpreter.

Third, there is now only one command function signature:

```
fn my_command(interp: &mut Interp, context_id: ContextID, argv: &[Value]) -> MoltResult {
    ...
}
```

Commands that don't use a cached context should be defined as follows:

```
fn my_command(interp: &mut Interp, _: ContextID, argv: &[Value]) -> MoltResult {
    ...
}
```

## Coming Attractions

At this point Molt is capable and robust enough for real work, though the Rust-level API is
not yet completely stable.  Standard Rust `0.y.z` semantic versioning applies: ".y" changes
can break the Rust-level API, ".z" changes will not.

*   Feature: Regex and Glob pattern matching by Molt commands
*   Testing improvements
*   Documentation improvements
