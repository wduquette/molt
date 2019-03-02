# test *name* *description* *body* *option* *result*

**Available with `molt test` only!**

The `test` command is used to test Molt commands, whether built-in or coded
in molt.  It executes its *body* as a Molt script, and compares the text
of the result to the given *result* value.  The *option* indicates the
expected result code: `-ok`, `-error`, `-result`, `-break`, or `-continue`.
For `-break` and `-continue`, the *result* value must be the empty string.

The *body* has its own local variable scope; use
[**global**](./global.md) to reference global variables.

The *name* is used to identify the test in the output.  Any string can be
used, but the convention is to use the format "*baseName*-*x*.*y*", e.g.,
`mycommand-1.1`.  Standard TCL's `tcltest(n)` framework takes advantage of
this convention to allow the developer to run subsets of tests, using
"glob"-style matching, e.g., run all tests that match `mycommand-2.*`.  
Molt doesn't currently provide this feature, but likely will in the future.

See also [`molt test`](../cmdline/test.md).

## Examples

The following tests an imaginary `square` command that returns the square
of a number.

```Tcl
test square-1.1 {square errors} {
    square
} -error {wrong # args: should be "square number"}

test square-2.1 {square command} {
    square 3
} -ok {9}
```

## TCL Notes

This command is a simplified version of the `test` command defined by
Standard TCL's `tcltest(n)` framework.  The intention is to increase the
similarity over time.

This command has an enhancement over TCL's `test` command: the test body has
its own local variable scope, just as a [**proc**](./proc.md) does.  The body
must use the [**global**](./global.md) command to access global variables.
