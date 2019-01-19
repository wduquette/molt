# Molt -- More Or Less TCL

The general notion is to build a minimal version of TCL for embedding in Rust
apps.  See "Plans", below.

## TODO Items

* Add list commands `list`, `lindex`, `llength`, with parsing and formatting
  aids.
  * Update main.rs to use the formatting aid to produce `argv`.
* Flesh out Rust tests
  * Design public API using `pub use` in `lib.rs`, so the examples read
    properly from the user's point of view.
* Implement minimal tcltest equivalent for testing the commands.
  * Main app will need to accept a script
* Add full command tests.
* Make molt::get_integer() parse the same varieties as Tcl_GetInt() does.
* Flesh out documentation
  * Including examples  

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
* The embedding API will allow the client to easily control the set of
  standard commands included in an interpreter.

### Excluded Features

The following features of modern TCL are currently off of the table:

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

* `exit`
* `puts` (partially; there's no support for output channels or -nonewline)
* `set`

The following commands need to get implemented next.

* append
* error
* expr
* for
* foreach
* if
* join
* lappend
* lindex
* list
* llength
* proc
* while
* unset

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
* append
* apply
* array
* auto_execok
* auto_import
* auto_load
* auto_load_index
* auto_qualify
* binary
* break
* case
* catch
* cd
* chan
* clock
* close
* concat
* continue
* coroutine
* dict
* encoding
* eof
* error
* eval
* exec
* expr
* fblocked
* fconfigure
* fcopy
* file
* fileevent
* flush
* for
* foreach
* format
* gets
* glob
* global
* history
* if
* incr
* info
* interp
* join
* lappend
* lassign
* lindex
* linsert
* list
* llength
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
* proc
* pwd
* read
* regexp
* regsub
* rename
* return
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
* unset
* update
* uplevel
* upvar
* variable
* vwait
* while
* yield
* yieldto
* zlib
