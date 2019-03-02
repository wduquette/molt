# molt test *filename* ?*args...*?

This command executes the test script called *filename* using the Molt
test harness, which is similar to Standard TCL's `tcltest` framework (though
much simpler, at present). Any arguments are passed to the test harness
(which ignores them, at present).

## Test Suites

`molt test` is often used to execute an entire test suite, spread over
multiple files.  To simplify writing such a suite, `molt test` assumes
that the folder contain the specified *filename* is the base folder for
the test suite, and sets the current working directory to that folder.
This allows the named test script to use [**source**](../ref/source.md) to
load other test scripts using paths relative to its own location.

## Writing Tests

Tests are written using the [**test**](../ref/test.md) command.  See
that man page for examples.

## Running Tests

For example,

```tcl
$ molt test good_tests.tcl
molt 0.1.0 -- Test Harness

5 tests, 5 passed, 0 failed, 0 errors
$ molt test bad_tests.tcl
molt 0.1.0 -- Test Harness

*** FAILED mytest-1.1 some proc
Expected <this result>
Received <that result>

2 tests, 1 passed, 1 failed, 0 errors
```
