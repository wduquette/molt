# error *message*

Returns an error with the given *message* and an error code of `NONE`.  The error may
be caught using the [**catch**](./catch.md) command.

## Example

```tcl
proc myproc {x} {
    if {$x < 0} {
        error "input must be non-negative"
    }
    ...
}
```

## TCL Liens

In standard TCL, the `error` also has optional `errorInfo` and `errorCode` arguments.  These
are used in older TCL code to rethrow errors without polluting the stack trace.  Modern TCL code
uses the [**throw**](./throw.md) command to throw an error with an error code and the
[**return**](./return.md) command to rethrow an error (see the reference page for an
example).  Consequently, Molt doesn't implement these arguments.
