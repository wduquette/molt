# append *varName* ?*value* ...?

Appends zero or more values to the value of variable *varName*.
If *varName* didn't previously exist, it is set to the concatenation
of the values.

## Examples

```Tcl
set x "this"
append x "that"
assert_eq $x "thisthat"

append y a b c
assert_eq $y abc
```
