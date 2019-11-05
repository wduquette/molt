# Molt -- More Or Less TCL

[![Crates.io](https://img.shields.io/crates/v/molt.svg)](https://crates.io/crates/molt)

The goal of this project is to build a minimal version of TCL for embedding in Rust
apps.  See [The Molt Book](https://wduquette.github.io/molt) for details
and user documentation.

See the [`molt-sample` repo](https://github.com/wduquette/molt-sample) for a sample Molt client
skeleton.

**NOTE:** A big part of this effort is defining and refining the Rust API used to
interact with and extend the interpreter.  At this point in development the API
can change without notice!  (And if you have suggestions for improvement, feel
free to write an issue.)

## Building

To build Molt:

*   Install the latest stable version of Rust (1.36.0 at time of writing)
*   Clone this repository
*   To build:

```
$ cd .../molt
$ cargo build
```

* To run the interactive shell

```
$ cargo run shell
```

* To run the language test suite

```
$ cargo run test test/all.tcl
```

## TODO Items

*   Revise expr.rs to separate parsing from evaluation.
*   Cleanup
    *   Look at how to best store proc details for efficient execution.
    *   Ponder the MoltList API, and consider if we can make it cleaner?
        *   Can add methods to MoltList in value.rs: `impl Vec<Value>`.  At least, I think
            I can.
    *   Consider whether var names should be stored as Values.
    *   Consider whether molt::MoltFloat, molt::MoltInt, and molt::MoltList should be
        molt::Float, molt::Int, and molt::List.
*   Add Interp::eval_file(), and implement `source` in terms of it.
*   Investigate performance of basic benchmarks.
*   Issues from wduquette/molt.
*   Add complete tests for the existing Tcl commands.
    *   "foreach"
    *   "join"
    *   "lindex"
    *   Test expression parser thoroughly
        * Add tests for "eq", "ne", "in", "ni"
* Implement stack traces
  * Need not mimic TCL's output.
* Implement remaining math functions in `expr`
* Continue to add commands from the "next" list, below.
* Flesh out Rust tests and Rust API docs in the code base.
  * Follow API design guide from the RUST nursery.
  * Design public API using `pub use` in `lib.rs`, so the examples read
    properly from the user's point of view.
* Define molt extension architecture
  * E.g., the ability to add extensions to Molt as Rust crates.
* Add costly features to core molt (e.g., regexp, glob) as Rust features.
  * `molt test` needs to be able to filter tests based on the available
    features.
* On-going:
    * Document Molt's TCL dialect using mdbook, and publish to GitHub pages.
* Consider generalizing the Subcommand array mechanism; standard command sets
  can be defined the same way, and loaded into the interpreter on creation.

The following commands need to get implemented next.

* `cd`, `pwd`
* `concat`
* `eval`
* `info level`
* `info commands` with glob matching
* The remaining list commands
* `string` (relevant subcommands)
* `upvar`

The following commands are not implemented by Molt at the present time,
but most will probably be added eventually.

* `array`
* `cd`
* `concat`
* `dict`
* `eval`
* `format` (complex!)
  * Might want two versions, one with printf syntax and one that's rustier.
* `info *`` (most subcommands)
* `lassign`
* `linsert`
* `lmap`
* `lrange`
* `lrepeat`
* `lreplace`
* `lreverse`
* `lsearch`
* `lset`
* `lsort`
* `pwd`
* `regexp`
* `regsub`
* `split`
* `string *` (most subcommands)
* `subst`
* `switch`
* `throw`
* `try`
* `uplevel`
* `upvar`

## Acknowledgements

I've gotten help from many people in this endeavor; here's a (necessarily partial) list.

* Jonathan Castello, for general Rust info
* Kevin Kenny, for help with TCL numerics
* Don Porter, for help with TCL parsing
* rfdonnelly, for the crates.io badge, etc.
* Various folks from users.rust-lang.org who have answered my questions:
    * Krishna Sannasi, for help getting `Value` to work with arbitrary user data types
    * Yandros, for pointing me at `OnceCell` and `UnsafeCell`.
    * jethrogb, for help on how to use `Ref::map` to return a `Ref<T>` of a component deep within
      a `RefCell<S>` from a function.  (Mind you, once I got it working and gave it a try I
      tore it out again, because of `BorrowMutError` panics.  But I had to try it.)
