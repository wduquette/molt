# return ?*options*? ?*value*?

Returns from a TCL procedure or script, optionally including a value.  By default, the
command simply returns the given *value*, or the empty string if *value* is omitted.  

```tcl
proc just_return {} {
    ...
    if {$a eq "all done"} {
        # Just return.  The return value will be the empty string, ""
        return
    }
    ...
}

proc identity {x} {
    # Return the argument
    return $x
}
```

The options allow the caller to return any TCL return code and to return through multiple
procedures at once.  The options are as follows:

| Option                 | Description                                                  |
| ---------------------- | ------------------------------------------------------------ |
| -code *code*           | The TCL result code; defaults to `ok`.                       |
| -level *level*         | Number of stack levels to return through; defaults to 1.     |
| -errorcode *errorCode* | The error code, when `-code` is `error`. Defaults to `NONE`. |
| -errorinfo *errorInfo* | The initial error stack trace. Defaults to the empty string. |

## The `-code` and `-level` Options

The `-code` and `-level` options work together.  The semantics are tricky to understand; a good
aid is to try things and use [**catch**](catch.md) to review the result value and options.

If `-code` is given, the *code* must be one of `ok` (the default), `error`, `return`, `break`,
`continue`, or an integer. Integer codes 0, 1, 2, 3, and 4 correspond to the symbolic constants
just given.  Other integers can be used to implement application-specific control structures.

If `-level` is given, the *level* must be an integer greater than or equal to zero; it represents
the number of stack levels to return through, and defaults to `1`.

Because of the defaults, a bare `return` is equivalent to `return -code ok -level 1`:

```tcl
# These are the same:
proc simple {}  { return "Hello world" }
proc complex {} { return -code ok -level 1 "Hello world" }
```

Both tell the interpreter to return "Hello world" to caller the caller of the current procedure
as a normal (`ok`) return value.  

By selecting a different `-code`, one can return some other error code.  For example,
`break` and `return -code break -level 0` are equivalent.  This can be useful in several ways. For
example, suppose you want to extend the language to support `break` and `continue` with labels,
to be used with some new control structure.  You could do the following; note the `-level 1`.  The
`return` command returns from your `labeled_break` procedure to its caller, where it is understood
as a `break` result.

```tcl
proc labeled_break {{label ""}} {
    return -code break -level 1 $label
}
```

Your new control structure would [**catch**] the result, see that it's a `break`, and jump to
the indicated label.

Similarly, suppose you want to write a command that works like `return` but does some additional
processing. You could do the following; note the `-level 2`.  The `2` is because the command needs
to return from your `list_return` method, and *then* from the calling procedure: two stack levels.

```tcl
# Return arguments as a list
proc list_return {a b c} {
    return -level 2 -code ok [list a b c]
}
```

## Returning Errors Cleanly

The normal way to throw an error in TCL is to use either the [**error**](error.md) or
[**throw**](throw.md) command; the latter is used in more modern code when there's an explicit
error code.  However, both of these commands will appear in the error stack trace.

Some TCL programmers consider it good style in library code to throw errors using `return`, as
follows (with or without the `-errorcode`):

```tcl
proc my_library_proc {} {
   ...
   return -code error -level 1 -errorcode {MYLIB MYERROR} "My Error Message"
}
```

The advantage of this approach is that the stack trace will show `my_library_proc` as the source
of the error, rather than `error` or `catch`.  

## The `-errorinfo` Option and Re-throwing Errors

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

## TCL Liens

The standard TCL `return` command is more complicated than shown here; however, the Molt
implementation provides all of the useful patterns the author has ever seen in use.  Some of the
specific differences are as follows:

* Molt rejects any options other than the ones listed above, and ignores `-errorcode` and
  `-errorinfo` if the `-code` is anything other than `error`.  Standard TCL's `return` retains all
  option/value pairs it is given, to be included in the `catch` options.

* Standard TCL's `return` takes an `-options` option; in Standard TCL, `return -options $opts`
  is equivalent to `return {*}$ops`.  Molt doesn't support `-options`, as it doesn't add any
  value and is confusing.

* Standard TCL provides two versions of the stack trace: the "error info", meant to be human
  readable, and the "error stack", for programmatic use.  The `-errorstack` is used to
  initialize the error stack when rethrowing errors, as `-errorinfo` is used to initialize the
  error info string.  Molt does not support the error stack at this time.

Some of these liens may be reconsidered over time.
