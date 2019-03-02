# for *start* *test* *next* *command*

The `for` command provides a C-like "for" loop, where *start* is a script that initializes the
loop counter, *test* is a conditional expression, *next* is a script that updates the loop
counter, and *command* is the body script.

If the *command* script calls the [**break**](./break.md) command, the loop terminates
immediately; if the *command* script calls the [**continue**](./continue.md) command,
loop execution continues with the next iteration.

## Example

For example, the following loop counts from 0 to 9:

```tcl
for {set i 0} {$i < 10} {incr i} {
    puts "i=$i"
}
```

Note, though, that the *start* and *next* arguments are arbitrary scripts; for example, *start*
can initialize multiple variables, and *next* can update multiple variables.
