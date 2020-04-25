# throw -- Throws an exception

**Syntax: throw *type* *message***

Throws an error with error code *type* and the given error *message*.  The error may
be caught using the [**catch**](./catch.md) command.

The error code is usually defined as a TCL list of symbols, e.g., `ARITH DIVZERO`.  Most standard
TCL error codes begin with `ARITH` (for arithmetic errors) or `TCL`.

## Example

```tcl
proc myproc {x} {
    if {$x < 0} {
        throw NEGNUM "input must be non-negative"
    }
    ...
}
```

Note that the [**error**](./error.md) command is equivalent to `throw NONE`; also, the `return`
command can also throw an error with an error code.  The three following
commands are semantically identical:

```tcl
error "My error message"

throw NONE "My error message"

return -code error -level 0 -errorcode NONE "My error message"
```
