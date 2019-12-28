# info *subcommand* ?*arg* ...?

Returns information about the state of the Molt interpreter.

| Subcommand                              | Description                                      |
| --------------------------------------- | ------------------------------------------------ |
| [info args](#info-args-procname)        | Names of procedure's arguments                   |
| [info body](#info-body-procname)        | Gets procedure body                              |
| [info commands](#info-commands)         | Names of all defined commands                    |
| [info complete](#info-complete-command) | Is this string a syntactically complete command? |
| [info procs](#info-procs)               | Names of all defined procedures                  |
| [info vars](#info-vars)                 | Names of all variables in the current scope      |

## info args *procname*

Retrieves a list of the names of the arguments of the named procedure.  Returns an error
if the command is undefined or is a binary command.

## info body *procname*

Retrieves the body of the named procedure.  Returns an error if the command is undefined or
is a binary command.

## info commands

Returns an unsorted list of the names of the commands defined in the interpreter,
including both binary commands and procedures.

**TCL Liens**: does not support filtering the list using a `glob`
pattern.

## info complete *command*

Returns 1 if the command appears to be a complete Tcl command, i.e., it
has no unmatched quotes, braces, or brackets, and 0 otherwise.  REPLs can
use this to allow the user to build up a multi-line command.

## info procs

Returns an unsorted list of the names of the procedures defined in the interpreter,
omitting binary commands.

**TCL Liens**: does not support filtering the list using a `glob`
pattern.

## info vars

Returns an unsorted list of the names of all variables that are visible
in the current scope, whether global or local.

**TCL Liens**: does not support filtering the list using a `glob`
pattern.
