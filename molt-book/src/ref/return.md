# return ?-code code? ?-level level? ?*value*?

Returns from a TCL procedure or script, optionally including a value.  By default, the
command simply returns the given
*value*, or the empty string if *value* is omitted.  The options allow the caller to return
any TCL return code and to return through multiple procedures at once.

If given, the *code* must be one of `ok`, `error`, `return`, `break`, `continue`, or
an integer. Integer codes 0, 1, 2, 3, and 4 correspond to the
symbolic constants just given.  Other integers can be used to implement application-specific
control structures.

If given, the *level* must be an integer greater than or equal to zero; it represents the number
of stack levels to return through, and defaults to `1`.

The precise semantics are difficult to explain without reference to the internals; but see the
examples for the most useful cases.

## Examples

```tcl
proc simple_return {} {
    return "Hello world"    
}
```

Procedure `simple_return` returns the string `Hello world` in the usual way.

```tcl
proc complex_return {} {
    return -code ok -level 1 "Hello world"
}
```

Procedure `complex_return` is equivalent to `simple_return`: it returns to the calling procedure
or script, one level up the call stack, with the value "Hello world".  The code `ok` indicates
that this is a normal value, to be returned to the caller.


```tcl
proc returnOnX {value} {
    if {$value eq "X"} {
        return -code ok -level 2 "early"
    }
}

proc myproc {a b} {
    returnOnX $a
    returnOnX $b
    ...
}
```

Procedure `returnOnX` causes a return from its caller if it is passed the value `X`.  You can see
this in use in `myproc`, which uses it to verify that neither variable `a` nor `b` has the value
`X` before going on about its business.

```tcl
proc pointless_returns {} {
    return -code ok -level 0 "Hello world"
    set greeting [return -code ok -level 0 "Howdy"]
    return "Goodbye"
}
```

In procedure `pointless_returns`, the first `return` command returns `Hello world` to its
immediate caller without exiting the current procedure, and its return value is ignored.  The
second is similar, but its return value of `Howdy` is saved in the variable
`greeting`.  Finally, the procedure returns `Goodbye` to its caller.  In practice,
`return -level 0` isn't very useful.

```tcl
proc verbose_error {value} {
    if {$value eq "bad value"} {
        error "Got a bad value"
    }
}

proc immediate_error {value} {
    if {$value eq "bad value"} {
        return -code error "Got a bad value"
    }
}
```

Both `verbose_error` and `immediate_error` return an error to the caller.  The primary difference
between the two is that `verbose_error` includes the call to `error` in the `errorInfo` stack
trace.  Using `return -code error` is considered better style in library code, but both
patterns are commonly used in practice.

```tcl
proc complex_break {} {
    foreach value $list {
        if {$value eq "stop"} {
            # Just the same as an explicit break
            return -code break -level 0
        }
    }
}
```

Procedure `complex_break` uses `return` to reproduce the behavior of the normal `break`
command.

```tcl
proc tested_break {value} {
    if {$value eq "stop"} {
        return -code break -level 1    
    }
}

foreach value $list {
    # Break from the list if $value is "stop"
    tested_break $value
}
```

Procedure `tested_break` can be used like `break` but only breaks if it is passed the value
"stop".  Note that `tested_break` can't simply call `break`; the `break` command tries
to take effect *within* the calling procedure.

The `return -code continue` command emulates `continue` in just the same way as for break.

## TCL Liens

* In Standard TCL, `return` is still more complicated than shown here.  See the man page for
  TCL 8.6 for details.  
