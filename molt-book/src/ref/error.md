# error *message*

Returns an error with the given *message*.  The error may
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

## TCL Notes

In standard TCL, the `error` command may also return a stack trace and an
error code.  In time Molt will likely implement the full error return protocol.
