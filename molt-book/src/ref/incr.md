# incr *varName* ?*increment*?

Increments integer-valued-variable *varName* by the given *increment*, which defaults to 1.
If the variable is unset, it is set to the *increment*.  The command returns the incremented
value.

## Examples

```tcl
unset a
incr a    ;# => 1
incr a    ;# => 2
incr a 3  ;# => 5

for {set a 1} {$a < 10} {incr a} {
    ...
}
```
