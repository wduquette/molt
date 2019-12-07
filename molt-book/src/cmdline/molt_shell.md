# molt shell ?*script*? ?*args...*?

The `molt shell` command invokes the Molt interpreter.

## Interactive Use

When called without any arguments, the command invokes the interactive interpreter:

```tcl
$ molt shell
Molt 0.2.0
%
```

Molt commands may be entered at the `%` prompt.  Enter `exit` to leave the interpreter.

## Script Execution

When called with arguments, the first argument is presumed to be the name of a Molt script;
any subsequent arguments are passed to the script.

```tcl
$ molt shell my_script.tcl arg1 arg2 arg3
...
$
```

When called in this way, the variable **arg0** contains the name of the script, and the
variable **argv** contains a list of the additional arguments (if any).

For example, consider the following script, `args.tcl`:

```tcl
puts "arg0 = $arg0"
puts "argv = $argv"
```

This script may be run as follows

```tcl
$ molt shell args.tcl a b c
arg0 = args.tcl
argv = a b c
$

## TCL Liens

The Standard TCL shell, `tclsh`, provides a number of features that Molt currently does not.

*   A `.tclshrc` file for initializing interactive shells.
    *   This can be added if there is demand; however, Molt is not primarily intended
        for general purpose scripting.

*   An option to execute a script and then start the interactive shell.
    *   This can be added if there is demand.

*   Environment variables for locating the interpreter's library of TCL code, locally
    installed TCL packages, etc.
    *   Molt's library of TCL code is compiled into the interpreter, rather than being
        loaded from disk at run-time.
    *   At present, Molt has no support for externally-defined TCL packages
        (other than the `source` command).
