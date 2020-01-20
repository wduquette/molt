# catch *script* ?*resultVarName*? ?*optionsVarName*?

Executes the script, catching the result, including any errors.  The return value of `catch`
is an integer code that indicates why the script returned.  If *resultVarName* is given, the
named variable is set to the actual return value in the caller's scope.  If *optionsVarName* is
given, the named variable is set to the [**return**](./return.md) options dictionary.

There are five return codes:

| Return Code  | Effect |
| ------------ | ------ |
| 0 (normal)   | Normal. The result variable is set to the script's result. |
| 1 (error)    | A command in the script threw an error. The result variable is set to the error message. |
| 2 (return)   | The script called [**return**](./return.md). The result variable is set to the returned value. |
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

The [**return**](./return.md) options can be used to rethrow the error:

```tcl
if {[catch {do_something} errMsg opts]} {
    puts "Error result: $errMsg"
    return -options $opts $errMsg
} else {
    puts "Good result: $result"
}
```
