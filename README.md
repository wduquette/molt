# Molt -- More Or Less TCL

The general notion is to build a minimal version of TCL for embedding in Rust
apps.  See [The Molt Book](https://github.com/wduquette/molt-book) for details
and user documentation.

## TODO Items

* Add "molt test" test harness.
  * Fix `source` so that the CWD is set to the source'd file's folder during evaluation, and reset
    after.
  * Improve the `test` harness
    * Need ability to set up, clean up.
  * Add `error`
  * Copy the Tcl 7.6 tests, and look for errors.
* Test expression parser thoroughly
  * Add tests for "eq", "ne", "in", "ni"
  * Implement remaining math functions
* Implement interp evaluation depth checking.
* Use Interp::complete() in the shell, to build up multiline commands.
* Continue to add commands from the "next" list, below.
* Flesh out Rust tests and Rust API docs in the code base.
  * Design public API using `pub use` in `lib.rs`, so the examples read
    properly from the user's point of view.
* Turn molt into a multi-crate project.
  * The base language crate, `molt`
  * The application crate, `molt-app`
  * Extension crates, e.g., `molt-pattern` (provides regexp, glob)
  * This keeps the base language crate small, while allowing `molt test` to
    use patterns.
* Make molt::get_int() parse the same varieties as Tcl_GetInt() does.
* On-going:
    * Document Molt's TCL dialect using mdbook, and publish to GitHub pages.
* Consider generalizing the Subcommand array mechanism; standard command sets
  can be defined the same way, and loaded into the interpreter on creation.
  * A binary extension is just a crate that can so initialize the interp.
* Implement stack traces
  * Need not mimic TCL's output.

The following commands need to get implemented next.

* error
* info level
* info commands (without glob matching)
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
