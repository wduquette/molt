# unset -- Clear a variable

**Syntax: unset ?-nocomplain? ?--? ?*name* *name* *name*...?**

Unsets one or more variables whose names are passed to the command.
It does not matter whether the variables actually exist or not.

The `-nocomplain` option is ignored.  The argument `--` indicates the
end of options; all arguments following `--` will be treated as variable
names whether they begin with a hyphen or not.

## TCL Differences

In standard TCL, it's an error to unset a variable that doesn't exist; the
command provides the `-nocomplain` option to cover this case. In Molt,
`unset` never complains; the `-nocomplain` option is provided only for
compatible with legacy TCL code.  (Per the TCL Core Team, the `-nocomplain`
option indicates, wherever it is found, that the original definition of the
command got the default behaviour wrong.)
