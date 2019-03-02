# catch *script* ?*resultVarName*?

Executes the script, catching the result, including any errors.  The return value of `catch`
is an integer code that indicates why the script returned.  If given, the variable called
*resultVarName* in the caller's scope is set to the actual return value.

There are five return codes:

| Return Code  | Effect |
| ------------ | ------ |
| 0 (normal)   | Normal. The result variable is set to the script's result. |
| 1 (error)    | A command in the script threw an error. The result variable is set to the error message. |
| 2 (return)   | The script called [**return**](./return.md) The result variable is set to the returned value. |
| 3 (break)    | The script called [**break**](./break.md). |
| 4 (continue) | The script called [**continue**](./continue.md). |

## Example

`catch` is most often used to catch errors.  For example,

```tcl
if {[catch {do_something} result]} {
    puts "Error result: $result"
} else {
    puts "Good result: $result"
}
```

## TCL Liens

In Standard TCL, the `catch` command has an additional argument, a variable that receives a
dictionary with full details about the context of the returned value.  Molt doesn't yet
implement this mechanism.
