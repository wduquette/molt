# continue

Continues execution with the next iteration of the inmost loop containing
the `continue` command.

## Example

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
