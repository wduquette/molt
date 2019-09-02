# Molt: More Or Less Tcl

The goal of the Molt project is to create a "small TCL" interpreter for
embedding in Rust applications.  The word "small" is to be understood in
several senses:

*   **Small in size.** Embedding Molt shouldn't greatly increase the size of the
    application.
*   **Small in language.** [Standard TCL](http://tcl-lang.org) has many features
    intended for building entire software systems.  Molt is intentionally
    limited to those needed for embedding.
*   **Small in dependencies.** Building standard TCL is non-trivial.  Embedding
    Molt should be as simple as using any other crate.

Hence, perfect compatibility with standard TCL is explicitly not a goal.  Many
features will not be implemented at all (e.g., octal literals); and where a
clearly better alternative exists, others may be implemented somewhat
differently (e.g., `-nocomplain` will always be the normal behavior).

On the other hand, Molt is meant to be TCL (more or less), not simply a
"Tcl-like language", so gratuitous differences are to be avoided.  One of the
goals of this document is to carefully delineate:

*   The features that have not yet been implemented.
*   The features that likely will never be implemented.
*   Any small differences in behavior.
*   And especially, any features that have intentionally been implemented in
    a different way.

## Initial Goals

The initial goals are as follows; see [Tcl Compatibility](./tcl_comp.md) for
more details.

*   Embedding, script execution, and an interactive shell.
    *   DONE
*   Parse to internal form, rather than reparsing strings at execution a la TCL 7.6
    *   And eventually, some kind of byte-compilation.
    *   IN PROGRESS.  Scripts are parsed to internal form, which is cached for later
        evaluation; Expressions are not.
*   Support for lists and dicts
    *   IN PROGRESS.  Lists exist but not all list commands are implemented.  Dicts do not
        currently exist.
*   A minimal set of commands
*   Procs
    *   IN PROGRESS.  Optimization is needed.
*   Expressions
    *   IN PROGRESS.  Not all math functions are supported, and the parsing/evaluation needs
        optimization.
*   [**molt test**](./cmdline/molt_test.md), a simple test harness for Molt
    code.
    *   Along with a thorough test suite, using `cargo test` for the internals
        and `molt test` for the language.
    *   IN PROGRESS. The test tool exists; more tests are needed.
*   Modular: pay for what you need
    *   I.e. don't require a large regex library for all clients.
    *   But allow it to be added as needed.
*   Configurable command sets
    *   The embedding API should allow the client to easily control the set of
        commands included in an interpreter.  For example, a game engine might
        want to exclude `proc`, `source`, `eval`, etc.
*   Stack traces
    *   Standard TCL provides detailed error stack traces.  Molt should do the
        same, but probably not in the same fashion.
