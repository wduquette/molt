# while -- "While" loop

**Syntax: while *test* *command***

The `while` command is a standard "while" loop, executing the *command* script just so
long as the *test* expression evaluates to true.

## Example

The following code will output the numbers from 1 to 10.

```tcl
set i 0
while {$i < 10} {
    puts "i=[incr i]"
}
```
