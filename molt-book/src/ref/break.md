# break

Breaks execution of the inmost loop containing the `break` command,
continuing execution after the loop.

## Example

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
