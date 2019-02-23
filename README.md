# Molt -- More Or Less TCL

The general notion is to build a minimal version of TCL for embedding in Rust
apps.  See "Plans", below.

## TODO Items

* Implement expression parser
  * CURRENTLY IN PROGRESS.  See docs/journal.md.
  * Add expr to the molt book.  
    *   Include precedence table.
  * Remove expr.md
  * Revise `if` command to use molt_expr_bool()
  * Revise `test` to take a description, like the tcltest command does.
  * Copy the Tcl 7.6 tests, and look for errors.
  * Add tests for "eq", "ne", "in", "ni"
  * Implement remaining math functions
* Implement interp evaluation depth checking.
* Use Interp::complete() in the shell, to build up multiline commands.
* Implement `error` command
* Continue to add commands from the "next" list, below.
* Flesh out Rust tests and Rust API docs in the code base.
  * Design public API using `pub use` in `lib.rs`, so the examples read
    properly from the user's point of view.
* Improve the `test` harness for the TCL command test suite.
  * Need ability to clean up.
  * Support test flags, so that tests can be excluded.
  * Support accumulating test results, so that the logs can be short.
  * Implementing "molt test" as part of the molt app,
    alongside "molt shell".
    *   Would provide the `test` command, as well as the overall harness.
* Turn molt into a multi-crate project.
  * The base language crate, `molt`
  * The application crate, `molt-app`
  * Extension crates, e.g., `molt-pattern` (provides regexp, glob)
  * This keeps the base language crate small, while allowing `molt test` to
    use patterns.
* Make molt::get_int() parse the same varieties as Tcl_GetInt() does.
* On-going:
    * Document Molt's TCL dialect using mdbook, and publish to GitHub pages.
* Consider adding an "object" command that defines a simple object
  containing a dictionary:
  * `$obj set var ?value?`
* Consider generalizing the Subcommand array mechanism; standard command sets
  can be defined the same way, and loaded into the interpreter on creation.
  * A binary extension is just a crate that can so initialize the interp.
* Implement stack traces
  * Need not mimic TCL's output.

## Plans

The goal is to produce a command language using basic TCL syntax
(e.g., commands, strings, and variable and command interpolation) that is
embeddable in Rust applications and extensible in Rust.  It will include
a "molt" application that provides script execution and an interactive
shell, but this is intended as an example and development aid, rather than
as a tool be used on its own. (Famous last words....)

### Initial Goals:

* Embedding, script execution, and an interactive shell.
* Basic parsing, as in TCL 7.6.
* Support for lists and dicts
* A smattering of basic commands
* Procs
* Expression parsing.
* A simple tcltest equivalent.
* Pay for what you need
  * I.e. don't require large regex, etc., libraries of all clients.
* Configurable command sets
  * The embedding API should allow the client to easily control the set of
    commands included in an interpreter.  For example, a game engine might
    want to exclude `proc`, `source`, `eval`, etc.
* Stack traces

### Excluded Features

The following features of modern TCL are currently off of the table:

* Abbreviated names
  * TCL will attempt to match partial names of commands and subcommands,
    as a convenience for interactive use.  Molt does not.
* Namespaces
* Traces
* Slave interpreters
* The majority of standard TCL commands.
* Byte-compiling

Ultimately I'll want to add something like byte-compilation for speed; but
I want to have a test suite in place first.

### Current Status

* The basic parser is in place, but has not been fully tested or
  optimized for speed.

The following commands have been implemented:

* `append`
* `break`
* `continue`
* `exit`
* `expr` (implementation in progress)
* `global`
* `if` (using scripts instead of expressions)
* `info commands` (without pattern matching)
* `info complete`
* `info vars` (without pattern matching)
* `join`  
* `lappend`
* `lindex`
* `list`
* `llength`
* `proc`
* `puts` (partially; there's no support for output channels or -nonewline)
* `return` (partially; supports only normal returns)
* `set`
* `unset`

The following commands need to get implemented next.

* error
* for
* info level
* info vars (without glob matching)
* info commands (without glob matching)
* source
* upvar
* while

### Specific Differences from TCL

The following are specific differences from TCL 8 not explicitly stated
above:

* Integer literals beginning with "0" are NOT assumed to be octal.
  Nor will they ever be.
* The encoding is currently always UTF-8.
* In `$name`, the name may include underscores and any character that
  Rust considers to be alphanumeric.

The following commands are not implemented by Molt at the present time:

* after
* apply
* array
* auto_execok
* auto_import
* auto_load
* auto_load_index
* auto_qualify
* binary
* case
* catch
* cd
* chan
* clock
* close
* concat
* coroutine
* dict
* encoding
* eof
* error
* eval
* exec
* fblocked
* fconfigure
* fcopy
* file
* fileevent
* flush
* for
* format
* gets
* glob
* history
* if
* incr
* info * (most subcommands)
* interp
* lassign
* linsert
* lmap
* load
* lrange
* lrepeat
* lreplace
* lreverse
* lsearch
* lset
* lsort
* namespace
* open
* package
* pid
* pwd
* read
* regexp
* regsub
* rename
* scan
* seek
* socket
* source
* split
* string
* subst
* switch
* tailcall
* tclLog
* tell
* throw
* time
* trace
* try
* unknown
* unload
* update
* uplevel
* upvar
* variable
* vwait
* while
* yield
* yieldto
* zlib
