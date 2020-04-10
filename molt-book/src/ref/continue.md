# continue

Continues execution with the next iteration of the inmost loop containing
the `continue` command.

```Tcl
foreach item $list {
    ...
    if {[someCondition]} {
        continue
    }

    # Skips this code on [someCondition]
    ...
}
```

## `continue` and `return`

The `continue` command is semantically equivalent to `return -code continue -level 0`, as is
the following procedure:

```tcl
proc my_continue {} {
    return -code continue -level 1
}
```


See the [**return**](return.md) reference page for more information.
