# info *subcommand* ?*arg* ...?

Returns information about the state of the Molt interpreter.

* [info commands](#info-commands)
* [info complete](#info-complete-command)
* [info vars](#info-vars)

## info commands

Returns an unsorted list of the commands defined in the interpreter,
including both binary commands and procs.

**TCL Liens**: does not support filtering the list using a `glob`
pattern.

## info complete *command*

Returns 1 if the command appears to be a complete Tcl command, i.e., it
has no unmatched quotes, braces, or brackets, and 0 otherwise.  REPLs can
use this to allow the user to build up a multi-line command.

## info vars

Returns an unsorted list of the names of all variables that are visible
in the current scope, whether global or local.

**TCL Liens**: does not support filtering the list using a `glob`
pattern.
