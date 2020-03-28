# catch *script* ?*resultVarName*? ?*optionsVarName*?

Executes the script, catching the script's result.  The `catch` command returns an integer
result code, indicating why the script returned.  If *resultVarName* is given, the
named variable in the caller's scope is set to the script's actual return value.  If
*optionsVarName* is given, the named variable is set to the [**return**](./return.md) options
dictionary in the caller's scope.

`catch` is most often used to catch errors.  For example,

```tcl
if {[catch {do_something} result]} {
    puts "Error message: $result"
} else {
    puts "Good result: $result"
}
```

## Return Codes

The return value of `catch` is an integer code that indicates why the script returned.  There are
five standard return codes:

| Return Code  | Effect |
| ------------ | ------ |
| 0 (ok)       | Normal. The result variable is set to the script's result. |
| 1 (error)    | A command in the script threw an error. The result variable is set to the error message. |
| 2 (return)   | The script called [**return**](./return.md). The result variable is set to the returned value. |
| 3 (break)    | The script called [**break**](./break.md). |
| 4 (continue) | The script called [**continue**](./continue.md). |

In addition, the `return` command allows any integer to be used as a return code; together with
`catch`, this can be used to implement new control structures.

## The `errorCode` and `errorInfo` Variables

When `catch` catches an error (or when an error message is output in the Molt REPL), the
global variable `errorCode` will be set to the specific error code (see [**throw**](throw.md))
and the global variable `errorInfo` will be set to a human-readable stack trace.

## The Options Dictionary

The options dictionary saved to the *optionsVarName* contains complete information about the
return options.  See [**return**](return.md) for a complete discussion of what the return
options are and how they are used.

## Rethrowing an Error

Sometimes it's desirable to catch an error, take some action (e.g., log it), and then rethrow
it.  The `return` command is used to do this:

```
set code [catch {
    # Some command or script that can throw an error
} result opts]

if {$code == 1} {
    # Log the error message
    puts "Got an error: $result"

    # Rethrow the error by returning with exactly the options and return
    # result that we received.
    return {*}$opts $result
}
```

## Visualizing the Return Protocol

The semantics of the `return`/`catch` protocol are tricky.  When implementing a new control
structure, or a modified or extended version of `return`, `break`, `continue`, etc., it is
often useful to execute short scripts and examine the options dictionary in the REPL:

```tcl
% catch { break } result opts
3
% set result
% set opts
-code 3 -level 0
% catch { return "Foo" } result opts
2
% set result
Foo
% set opts
-code 0 -level 1
%
```

This REPL dialog shows that `break` yields result code 3 immediately, to be handled by the
calling command (usually a loop), while `return` returns from the calling procedure (`-level 1`)
and then yields an `ok` (i.e., normal) result to *its* caller.

## TCL Liens

Molt's `catch` command differs from Standard TCL's in the following ways:

* The options dictionary, as returned, lacks the `-errorline` and `-errorstack` options.  These
  might be added over time.

* All options passed to `return`, whether understood by Standard TCL or not, are passed through
  and included in the `catch` options dictionary.  Molt does not currently support this.

All of the common patterns of use are supported.
