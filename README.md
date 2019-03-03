# Molt -- More Or Less TCL

The general notion is to build a minimal version of TCL for embedding in Rust
apps.  See [The Molt Book](https://github.com/wduquette/molt-book) for details
and user documentation.

## Building

To build Molt:

*   Install the latest stable version of Rust (1.33.0 at time of writing)
*   Clone this repository
*   To build:

```
$ cd .../molt
$ cargo build
```

*   To run the interactive shell

```
$ cargo run shell
```

*   To run the language test suite

```
$ cargo run test test/all.tcl
```

## TODO Items

* Add `rename` command (and use it in -cleanup clauses in test suite)
* Add `error` command
* Add complete tests for the existing Tcl commands.
    * Test expression parser thoroughly
      * Add tests for "eq", "ne", "in", "ni"
      * Implement remaining math functions
* Implement interp evaluation depth checking.
* Use Interp::complete() in the shell, to build up multiline commands.
* Continue to add commands from the "next" list, below.
* Flesh out Rust tests and Rust API docs in the code base.
  * Design public API using `pub use` in `lib.rs`, so the examples read
    properly from the user's point of view.
* Define molt extension architecture
  * E.g., the ability to add extensions to Molt as Rust crates.
* Add costly features to core molt (e.g., regexp, glob) as Rust features.
* Make molt::get_int() parse the same varieties as Tcl_GetInt() does.
* On-going:
    * Document Molt's TCL dialect using mdbook, and publish to GitHub pages.
* Consider generalizing the Subcommand array mechanism; standard command sets
  can be defined the same way, and loaded into the interpreter on creation.
* Implement stack traces
  * Need not mimic TCL's output.

The following commands need to get implemented next.

* cd, pwd
* concat
* error
* eval
* info level
* info commands (without glob matching)
* list commands
* rename
* string 
* upvar
* while

The following commands are not implemented by Molt at the present time,
but most will probably be added eventually.

* array
* cd
* concat
* dict
* error
* eval
* format
* info * (most subcommands)
* lassign
* linsert
* lmap
* lrange
* lrepeat
* lreplace
* lreverse
* lsearch
* lset
* lsort
* pwd
* regexp
* regsub
* rename
* split
* string
* subst
* switch
* throw
* time
* try
* uplevel
* upvar
* while
