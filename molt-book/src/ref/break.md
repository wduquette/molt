# break -- Break loop execution

**Syntax: break**

Breaks execution of the inmost loop containing the `break` command,
continuing execution after the loop.

```Tcl
foreach item $list {
    ...
    if {[someCondition]} {
        break
    }
    ...
}

# Execution continues here after the break
```

## `break` and `return`

The `break` command is semantically equivalent to `return -code break -level 0`, as is
the following procedure:

```tcl
proc my_break {} {
    return -code break -level 1
}
```

See the [**return**](return.md) reference page for more information.
