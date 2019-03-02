# if *expr1* ?then? *body1* elseif *expr2* ?then? *body2* elseif ... ?else? ?*bodyN*?

Tests a chain of one or more expressions, and executes the matching *body*,
which must be a script.  Returns the result of the last command executed in
the selected *body*.

Both the `then` and `else` keywords are optional.  The standard TCL
convention is to always omit the `then` keywords and to always
include the `else` keyword when there's an `else` clause.

## Examples

```tcl
if {$x > 0} {
    puts "positive"
}

if {$x < 0} {
    puts "negative"
} else {
    puts "non-negative"
}

if {$x > 0} {
    puts "positive"
} elseif {$x < 0} {
    puts "negative"
} else {
    puts "zero"
}

set value [if {$x > 0} {
    expr {$x + $y}   
} else {
    expr {$x - $y}   
}]
```
