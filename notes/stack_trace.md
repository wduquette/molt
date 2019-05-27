# Notes on TCL 7.6 Stack Traces

## Stack Trace construction

* A TCL C function can detect an error or propagate an error.  
* It knows the difference by the ERR_ALREADY_LOGGED interpreter flag.
* Either way, it calls `Tcl_AddErrorInfo()`` to save a stack trace entry.
* The entry can vary depending on whether this is a newly detected error or a propagated error.

In `Tcl_Eval()``, for example,  a detection entry looks like this:

```
    while executing
"{some TCL command} ..."
```

and a propagation entry looks like this:

```
    invoked from within
"{the calling TCL command} ..."
```

The command is elided and the ellipsis included only if the command is too long.

`The Tcl_AddErrorInfo` function does this:

*   On a newly detected error, it clears the TCL `errorInfo` variable and sets the `errorCode`
    to `NONE` if it isn't already set.
    *   Note: it's setting the actual TCL variables.
*   Then, either way it appends the message to the current value of `errorInfo`.

Thus, `errorInfo` gets built up with the error at the top and the ultimate caller at the
bottom.

## How Molt Should Do It

The natural thing is to augment the `MoltReturn data`, and particularly the `ResultCode::Error`
enum.  At present, the `Error` constant has one argument, a String.  In addition or instead it
should take a struct, ErrorInfo, which can accumulate stack trace info for later programmatic
query (unsupported in TCL 7.6).  We also need to update the `errorInfo` variable; and it may
be appropriate to do that as we go, as now; otherwise it needs to be done when the error is
finally caught.

## Speculation: What should Molt's error handling look like?

TCL's error handling evolved over many years and versions.  Should we use an errorInfo variable?
Should we provide a command to return the most recent error?  Should we require that you catch
it?  Perhaps errorInfo should be the property of the molt-shell, rather than molt itself?

## Contributors to the stack trace

Some notes:

* Many functions appear to simply AddErrorInfo and return TCL_ERROR or -1 without setting
  the actual interp result.  Molt would probably use the same mechanism for both kinds.

| Command/Function    | Module      | Line | What                                       |
| ------------------- | ----------- | ---: | ------------------------------------------ |
| AfterProc           | tclEvent.c  | 1870 | ("after" script)                           |
| InterpProc          | tclProc.c   | 510  | (procedure "{name}" line %d)               |
| SortCompareProc     | tclCmdIL.c  | 1428 | (converting list element...)               |
| SortCompareProc     | tclCmdIL.c  | 1443 | (converting list element...)               |
| SortCompareProc     | tclCmdIL.c  | 1468 | (user-defined comparison command)          |
| Tcl_BackgroundError | tclEvent.c  | 1255 | "", Setting up for background error        |
| Tcl_Eval            | tclBasic.c  | 1486 | "while executing", "invoked from within"   |
| Tcl_EvalFile        | tclIOUtil.c |  421 | (file "{name}" line %d)                    |
| Tcl_GetOpenMode     | tclIOUtil.c |  256 | "while processing open access modes"       |
| Tcl_Main            | tclMain.c   |  158 | Appends "" before reading errorInfo        |
| Tcl_ParseVar        | tclParse.c  | 1333 | (parsing index for array "...")            |
| Tcl_PkgRequire      | tclPkg.c    |  203 | ("package ifneeded" script)                |
| Tcl_PkgRequire      | tclPkg.c    |  203 | ("package unknown" script)                 |
| "case"              | tclCmdAH.c  |  n/a | "case" is obsolete.                        |
| "error"             | tclCmdAH.c  |  393 | The error message.                         |
| "eval"              | tclCmdAH.c  |  454 | "eval" body line %d                        |
| "for"               | tclCmdAH.c  | 1162 | "for" initial command                      |
| "for"               | tclCmdAH.c  | 1179 | "for" body line %d                         |
| "for"               | tclCmdAH.c  | 1188 | "for" loop-end command                     |
| "foreach"           | tclCmdAH.c  | 1330 | "foreach" body line %d                     |
| "incr"              | tclCmdIL.c  |  196 | (reading value of variable to increment)   |
| "incr"              | tclCmdIL.c  |  206 | (reading increment)                        |
| "interp", slaves    | tclInterp.c |  n/a | Won't be implementing this any time soon.  |
| "load"              | tclLoad.c   |  n/a | Dynamic loading isn't a thing in Rust.     |
| "switch"            | tclCmdMZ.c  | 1702 | (... arm line %d)                          |
| "time"              | tclCmdMZ.c  | 1768 | ("time" body line %d)                      |
| "uplevel"           | tclProc.c   |  313 | ("uplevel" body line %d)                   |
| "while"             | tclCmdMZ.c  | 2104 | ("while" body line %d)                     |
