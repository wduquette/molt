# catch *script* ?*resultVarName*? ?*optionsVarName*?

Executes the script, catching the result, including any errors.  The return value of `catch`
is an integer code that indicates why the script returned.  If *resultVarName* is given, the
named variable is set to the actual return value in the caller's scope.  If *optionsVarName* is
given, the named variable is set to the [**return**](./return.md) options dictionary.

There are five return codes:

| Return Code  | Effect |
| ------------ | ------ |
| 0 (ok)       | Normal. The result variable is set to the script's result. |
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

The `optionsVarName` can be used to capture the [**return**](./return.md) command's options:

```
% catch { return "Howdy" } result opts
2
% set result
Howdy
% set opts
-code 0 -level 1
```

## TCL Liens

This will be implemented soon:

The [**return**](./return.md) `-options` option can be used to rethrow the error:

```tcl
if {[catch {do_something} errMsg opts]} {
    puts "Error result: $errMsg"
    return -options $opts
} else {
    puts "Good result: $result"
}
```
