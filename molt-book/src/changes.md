# Annotated Change Log

## New in Molt 0.3.2

Nothing yet!

## New in Molt 0.3.1

* Added the [**molt_throw!**](embed/molt_result.md#result-macros) macro.
* Improved the API documentation for `molt_ok!` and `molt_err!`.
* Added `Exception::error_code` and `Exception::error_info`, to streamline using exceptions.
* Added the `env()` array, which contains the current environment variable settings.
  * **Note**: as yet, changes to the `env()` array are not mirrored back to the process
    environment.
* Added the [**string**](ref/string.md) command
  * `string cat`
  * `string compare`
  * `string length`

## New in Molt 0.3.0

The changes in Molt 0.3.0 break the existing API in two ways:

* The syntax for `molt_shell::repl` has changed slightly.
* The `MoltResult` type has changed significantly.

Keep reading for the full details.

### Molt Shell: User-Definable Prompts

Thanks to Coleman McFarland, `molt_shell::repl` now supports programmable prompts via the
`tcl_prompt1` variable.  See the rustdocs and the [`molt_shell`](cmdline/molt_shell.md)
discussion in this book for more information.

### Error Stack Traces

Molt now provides error stack traces in more-or-less the same form as standard TCL.  Stack
traces are accessible to Rust clients, are printed by the Molt shell, and can be
accessed in scripts via the [`catch`](ref/catch.md) command and the `errorInfo` variable
in the usual TCL way.

### Error Codes

Molt scripts and Rust code can now throw errors with an explicit error code, as in Standard
TCL; see the [`throw`](ref/throw.md) and [`catch`](ref/catch.md) commands.

### Return Protocol

Molt now supports the full [`return`](ref/return.md)/[`catch`](ref/catch.md) protocol for
building application-specific control structures in script code.  The mechanism as implemented
is slightly simpler than in Standard TCL, but should be sufficient for all practical
purposes.  See the referenced commands for specifics.

### `MoltResult` and the `Exception` Struct

In order to support the above changes, the definition of the
`MoltResult` type has changed.  Instead of

```rust
pub type MoltResult = Result<Value, ResultCode>;
```

it is now

```rust
pub type MoltResult = Result<Value, Exception>;
```

where `Exception` is a struct containing the `ResultCode` and other necessary data.  The
`ResultCode` enum still exists, but has been simplified.  See the rust doc for details.

## New in Molt 0.2

### Dictionaries and the `dict` command

Molt now supports TCL dictionary values.  The [`dict`](ref/dict.md) command provides the
following subcommands:

*   dict create
*   dict exists
*   dict keys
*   dict get
*   dict remove
*   dict set
*   dict size
*   dict unset
*   dict values

Other `dict` subcommands will be added over time.


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
