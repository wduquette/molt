# rename *oldName* *newName*

Renames the command called *oldName* to be *newName* instead.  

Any command may be renamed in this way; it is a common TCL approach to wrap a command by
renaming it and defining a new command with the *oldName* that calls the old command at
its *newName*.

If the *newName* is the empty string, the command will be removed from the interpreter.

## Examples

```tcl
proc myproc {} { ... }

# Rename the proc
rename myproc yourproc

# Remove the proc from the interpreter
rename yourproc ""
```
