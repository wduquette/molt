# Tcl Compatibility

The initial version of Molt is aiming at compatibility with TCL 7.6, the
last version not to be byte-compiled (with a few TCL 8-based additions).
Once the basic interpreter and command set is in place, along with a thorough
test suite (based on TCL's own), the plan is to begin to optimize for speed.

Each [command's man page](./ref/reference.md) in this book documents any
temporary or permanent differences between it and the similarly named command
in [Standard TCL](http://tcl-lang.org).

The remainder of this section documents overall differences; see the
[Molt README](https://github.com/wduquette/molt) for details on current
development.

Note that if the modular design is done properly, some of the features
described as never to be implemented could be added as extension crates.

## Features that already exist

See the [command reference](./ref/reference.md) for the set of commands that
have already been implemented.  The current set of features includes:

*   Script execution
*   Procedure definition
*   Control flow
*   Local and global variables
*   Evaluation of expressions
*   A modicum of introspection
*   An interactive REPL
    *   Using the `rustyline` crate for line editing.
*   Execution of script files
*   A test harness
*   The beginnings of a detailed test suite

## Features to be added soon

*   [**molt test**](./cmdline/molt_test.md) Enhancements
*   An improved test suite
*   The remainder of the minimal TCL command set:
    *   catch
    *   error
    *   info level
    *   info commands (without glob matching)
    *   upvar
    *   while
    *   ...
*   A clean and modular embedding API
    *   Including interactive REPLs, and the ability to use
        `molt test` to test custom Molt extensions.

## Features to be added later

*   Array variables (associative arrays)
*   Dictionaries
*   Globs and Regexes
*   Byte Compilation

## Features that may never be added

*   Slave interpreters
*   Namespaces
*   Traces
*   File I/O
*   Event loop

## Features that will almost certainly never be added

*   The TCL autoloader
*   Packages/TCL Modules
    *   Though some form of module architecture might become valuable at
        some point.
*   Coroutines
*   Communication between Interps in different threads
*   Support for dynamic libraries
*   Support for binary extensions written in C (or anything but Rust)
*   Network I/O
*   OOP

## Miscellaneous Differences

See the man pages for specific commands for other differences.

*   Integer literals beginning with "0" are NOT assumed to be octal.
    Nor will they ever be.
*   The encoding is currently always UTF-8.
*   In `$name`, the name may include underscores and any character that
    Rust considers to be alphanumeric.
*   The notion of what constitutes whitespace is generally left up to Rust.
*   When using the TCL shell interactively, TCL will attempt to match
    partial names of commands and subcommands as a convenience.  Molt does not.
    *   In principle, some form of tab-completion could be added at some point.
