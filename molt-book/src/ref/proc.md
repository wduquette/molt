# proc -- Procedure definition

**Syntax: proc *name* *args* *body***

Defines a procedure with the given *name*, argument list *args*, and
script *body*.  The procedure may be called like any built-in command.

The argument list, *args*, is a list of argument specifiers, each of
which may be:

* A name, representing a required argument
* A list of two elements, a name and a default value, representing an
  optional argument
* The name `args`, representing any additional arguments.

Optional arguments must follow required arguments, and `args` must
appear last.  

When called, the procedure returns the result of the last command in the
body script, or the result of calling [`return`](./return.md), or an
error.

## TCL Liens

Molt does not support namespaces or namespace syntax in procedure names.
