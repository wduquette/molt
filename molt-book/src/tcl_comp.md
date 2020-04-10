# Tcl Compatibility

Molt is aiming at limited compatibility with TCL 8.x, the current stable version
of Standard TCL, as described [above](overview.md).  The development plan is as follows:

*   Implement the complete Molt semantics
    *   Core interpreter
    *   Essential TCL commands
    *   Robust and ergonomic Rust-level API for extending TCL in Rust
    *   Related tools (e.g., TCL-level test harness)
    *   Thorough and complete test suite (at both Rust and TCL levels)
    *   Thorough documentation
*   Optimize for speed
    *   Ideally including byte-compilation
*   Extend with new features as requested.

Each TCL command provided standard by the Molt interpreter is documented in this
book with a complete [man page](./ref/reference.md).  A command's man page documents
the semantics of the command, and any temporary or permanent differences between it and the
similarly named command in [Standard TCL](http://tcl-lang.org).

The remainder of this section documents overall differences; see the
[Molt README](https://github.com/wduquette/molt) for details on current
development.

Note that some of the features described as never to be implemented
could conceivably be added as extension crates.

## Features that already exist

See the [command reference](./ref/reference.md) for the set of commands that
have already been implemented.  The current set of features includes:

At the TCL Level:

*   Script execution
*   Procedure definition
*   Standard control structures (except the `switch` command)
*   Local and global variables, including associative arrays
*   Boolean and numeric expressions
*   Dictionaries
*   Many standard TCL commands
*   A modicum of introspection

At the Rust Level:

*   A clean and modular embedding API
*   The `Interp` struct (e.g., Standard TCL's Interp)
    *   API for defining new commands in Rust, setting and querying variables, and
        evaluating TCL code.
*   The `Value` type (e.g., Tcl_Obj)
    *   TCL values are strings; `Value` shares them efficiently by reference counting, and
        caches binary data representations for run-time efficiency.

Related Tools:

*   An interactive REPL
    *   Using the `rustyline` crate for line editing.
*   A shell, for execution of script files
*   A test harness

## Features to be added soon

See the [overview](overview.md) and the Molt README.

## Features to be added eventually

*   Globs and Regexes
*   Some way to create ensemble commands and simple objects

## Features that might someday be added (depending on demand)

*   Namespaces
*   Slave interpreters
*   File I/O
*   Event loop
*   Byte Compilation
*   Communication between `Interps` in different threads
*   Traces
*   Some kind of TCL-level module architecture

## Features that will almost certainly never be added

*   The TCL autoloader
*   Packages/TCL Modules (as represented in Standard TCL)
*   Coroutines
*   Support for dynamically loading Molt extensions written in Rust
*   Support for Molt extensions written in C (or anything but Rust)
    *   But note that a Molt extension written in Rust can certainly call into
        C libraries in the usual way.
*   Network I/O
*   OOP (in the form of TclOO)

## Miscellaneous Differences

See the man pages for specific commands for other differences.

*   Integer literals beginning with "0" are NOT assumed to be octal,
    Nor will they ever be.
*   The encoding is currently always UTF-8.
*   In variable names, e.g. `$name`, the name may include underscores and any character that
    Rust considers to be alphanumeric.
*   The notion of what constitutes whitespace is generally left up to Rust.
*   When using the TCL shell interactively, TCL will attempt to match
    partial names of commands and subcommands as a convenience.  Molt does not.
    *   In principle, some form of tab-completion could be added at some point.
