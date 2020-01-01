# info *subcommand* ?*arg* ...?

Returns information about the state of the Molt interpreter.

| Subcommand                              | Description                                       |
| --------------------------------------- | ------------------------------------------------- |
| [info args](#info-args-procname)        | Names of procedure's arguments                    |
| [info body](#info-body-procname)        | Gets procedure body                               |
| [info cmdtype](#info-cmdtype-command)   | Queries a command's type                          |
| [info commands](#info-commands)         | Names of all defined commands                     |
| [info complete](#info-complete-command) | Is this string a syntactically complete command?  |
| [info default](#info-default-procname-arg-varname) | A procedure argument's default value   |
| [info exists](#info-exists-varname)     | Is this a variable in the current scope?          |
| [info globals](#info-globals)           | Names of all variables in the global scope        |
| [info locals](#info-locals)             | Names of all local variables in the current scope |
| [info procs](#info-procs)               | Names of all defined procedures                   |
| [info vars](#info-vars)                 | Names of all variables in the current scope       |

## info args *procname*

Retrieves a list of the names of the arguments of the named procedure.  Returns an error
if the command is undefined or is a binary command.

For example,

```tcl
% proc myproc {a b c} { ... }
% info args myproc
a b c
%
```

## info body *procname*

Retrieves the body of the named procedure.  Returns an error if the command is undefined or
is a binary command.

For example,

```tcl
% proc myproc {name} { puts "Hello, $name" }
% info body myproc
puts "Hello, $name"
%
```

## info cmdtype *command*

Retrieves the named command's type, either `native` or `proc`.  The *command* is `native` if it's
implemented in Rust and `proc` if it's implemented as a TCL procedure.

```tcl
% proc myproc {} { ... }
% info cmdtype set
native
% info cmdtype myproc
proc
%
```

**TCL Liens**: Standard TCL defines a variety of other command types, e.g., slave interpreters,
interpreter aliases, objects, and so forth.  These will be added naturally if and when they are
added to Molt.

## info commands

Returns an unsorted list of the names of the commands defined in the interpreter,
including both binary commands and procedures.

**TCL Liens**: does not support filtering the list using a `glob`
pattern.

## info complete *command*

Returns 1 if the command appears to be a complete Tcl command, i.e., it
has no unmatched quotes, braces, or brackets, and 0 otherwise.  REPLs can
use this to allow the user to build up a multi-line command.

For example,

```tcl
% info complete { puts "Hello, world!" }
1
% info complete { puts "Hello, world! }
0
%
```

## info default *procname* *arg* *varname*

Retrieves the default value of procedure *procname*'s argument called *arg*.  If *arg* has
a default value, `info default` returns 1 and assigns the default value to the variable
called *varname*.  Otherwise, `info default` returns 0 and assigns the empty string to the
variable called *varname*.

The command throws an error if:

* *procname* doesn't name a procedure
* The procedure *procname* has no argument called *arg*
* The value can't be assigned to a variable called *varname*.

In the following example, `myproc` has two arguments, `a` and `b`.  `a` has no default value;
`b` has the default value `Howdy`.

```tcl
% proc myproc {a {b Howdy}} { ... }
% info default myproc a defvalue
0
% puts "<$defval>"
<>
% info default myproc b defvalue
1
% puts "<$defval>"
<Howdy>
%
```

## info exists *varname*

Returns 1 if *varname* is the name of a variable or array element in the current scope, and
0 otherwise.

```tcl
% set a 1
1
% set b(1) 1
1
% info exists a
1
% info exists b
1
% info exists c
0
% info exists b(1)
1
% info exists b(2)
0
```

## info globals

Returns an unsorted list of the names of all variables defined in the global scope.

**TCL Liens**: does not support filtering the list using a `glob`
pattern.

## info locals

Returns an unsorted list of the names of all local variables defined in the current scope, e.g.,
`proc` arguments and variables defined locally, but no variables brought in from other scopes
via `global` or `upvar`.

**TCL Liens**: does not support filtering the list using a `glob`
pattern.


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
