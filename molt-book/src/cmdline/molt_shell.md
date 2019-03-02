# molt shell ?*script*? ?*args...*?

The `molt shell` command invokes the Molt interpreter.

## Interactive Use

When called without any arguments, the command invokes the interactive interpreter:

```tcl
$ molt shell
Molt 0.1.0
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
