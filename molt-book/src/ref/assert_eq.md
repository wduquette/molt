# assert_eq *received* *expected*

Asserts that the string *received* equals the string *expected*.  On success,
returns the empty string; on failure, returns an error.

This command is primarily intended for use in examples, to show the expected
result of a computation, rather than for use in test suites.  For testing,
see the [`test`](./test.md) command and the
[`molt test`](../cmdline/molt_test.md) tool.

## TCL Notes

This command is not part of Standard TCL; it is provided because of its
similarity to the Rust `assert_eq!` macro.
