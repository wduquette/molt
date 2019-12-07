# test *name* *description* *args ...*

**Available in [**molt test**](../molt_test.md) scripts only!**

The `test` command executes its body as a Molt script and compares its result
to an expected value.  It may be used to test Molt commands, whether built-in or coded
in Molt.  The expected value may be an `-ok` result or an `-error` message.

The *name* and *description* are used to identify the test in the output.  The
*name* can be any string, but the convention is to use the format
"*baseName*-*x*.*y*", e.g., `mycommand-1.1`.  In the future, `molt test`
will allow the user to filter the set of tests on this name string.

The test is executed in its own local variable scope; variables used by the
test will be cleaned up automatically at the end of the test.  The
[**global**](../../ref/global.md) command may be used to reference global variables; however,
changes to these must be cleaned up explicitly.  Similarly, any
[**procs**](../../ref/proc.md) defined by the test must be cleaned up explicitly.

The `test` command has two forms, a brief form and an extended form with more options.

## test *name* *description* *body* -ok|-error *expectedValue*

In the brief form, the *body* is the test script itself; and it is expected to return
a normal result or an error message.  Either way, *expectedValue* is the expected value.

* The test **passes** if the *body* returns the right kind of result with the expected value.
* The test **fails** if the *body* returns the right kind of result (e.g., `-ok`) with
some other value.
* The test is in **error** if the *body* returns the wrong kind of result, (e.g., an
  error was returned when a normal result was expected).

## test *name* *description* *option value* ?*option value ...*?

In the extended form, the details of the test are specified using options:

* **-setup**: indicates a setup script, which will be executed before the body of the
  test.  The test is flagged as an **error** if the setup script returns anything
  but a normal result.

* **-body**: indicates the test's *body*, which is interpreted as described above.

* **-cleanup**: indicates a cleanup script, which will be executed after the body of the
  test.  The test is flagged as an **error** if the cleanup script returns anything but
  a normal result.

* **-ok | -error**: indicates the expected value, as described above.

## Examples

The following tests are for an imaginary `square` command that returns the square
of a number.  They use the brief form.

```Tcl
test square-1.1 {square errors} {
    square
} -error {wrong # args: should be "square number"}

test square-2.1 {square command} {
    square 3
} -ok {9}
```

The following test shows the extended form:

```Tcl
test newproc-1.1 {new proc} -setup {
    # Define a proc for use in the test
    proc myproc {} { return "called myproc" }
} -body {
    # Call the proc
    myproc
} -cleanup {
    # Clean up the proc
    rename myproc ""
} -error {called myproc}
```


## TCL Notes

This command is a simplified version of the `test` command defined by
Standard TCL's `tcltest(n)` framework.  The intention is to increase the
similarity over time.

This command has an enhancement over TCL's `test` command: the test has
its own local variable scope, just as a [**proc**](../../ref/proc.md) does.  The body
must use the [**global**](../../ref/global.md) command to access global variables.
