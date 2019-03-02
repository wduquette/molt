# unset *varName*

Unsets variable *varName*'s value.  The `unset` command has no effect if
no such variable is visible in the current scope.

## TCL Differences

In standard TCL, it's an error to unset a variable that doesn't exist; the
command provides the `-nocomplain` option to cover this case.  Per the TCL Core Team,
the `-nocomplain` option indicates, wherever it is found, that the original
definition of the command got it wrong.  In Molt, `unset` never complains.
