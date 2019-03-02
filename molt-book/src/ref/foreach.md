# foreach *varList* *list* *body*

Loops over the elements in the *list*, assigning them to the variables
in the *varList* and executing the *body* for each set of assignments.

The [break](./break.md) and [continue](./continue.md) commands can be
used to control loop execution; see their reference pages for details.

## Examples

Prints out the values "1", "2", and "3" on successive lines.

```Tcl
foreach a {1 2 3} {
    puts $a
}
```

Prints out pairs of values from the list. In the final iteration there
is only value left, so `b` is assigned the empty string.

```Tcl
foreach {a b} {1 2 3 4 5} {
    puts "$a,$b"
}
# Outputs:
#
#  1,2
#  3,4
#  5,
```

## TCL Liens

In standard TCL, `foreach` can iterate over multiple lists at the
same time, e.g., the following script will output the pairs "a,1",
"b,2", and "c,3".  Molt doesn't currently support this extended syntax.

```Tcl
foreach x {a b c} y {1 2 3} {
    puts "$x,$y"
}
```
