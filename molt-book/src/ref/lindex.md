# lindex *list* ?*index* ...?

Returns an element from the *list*, indexing into nested lists.  The indices
may be represented as individual indices on the command line, or as a list
of indices.  Indices are integers from 0 to length - 1.  If an index is less
than 0 or greater than or equal to the list length, `lindex` will return
the empty string.

## Examples

```tcl
lindex {a {b c d} e}        ;# "a {b c d} e"
lindex {a {b c d} e} 1      ;# "b c d"
lindex {a {b c d} e} 1 1    ;# "c"
lindex {a {b c d} e} {}     ;# "a {b c d} e"
lindex {a {b c d} e} {1 1}  ;# "c"
```

## TCL Liens

Indices in standard TCL may take several additional forms.  For example,
`end` indexes the last entry in the list; `end-1` indexes the next to last
entry, and so forth.  Molt doesn't yet support this. 
