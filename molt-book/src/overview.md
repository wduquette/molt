# Molt: More Or Less Tcl

**Molt 0.3.2** is a minimal implementation of the TCL language for embedding in Rust apps
and for scripting Rust libraries.  Molt is intended to be:

*   **Small in size.** Embedding Molt shouldn't greatly increase the size of the
    application.

*   **Small in language.** [Standard TCL](http://tcl-lang.org) has many features
    intended for building entire software systems.  Molt is intentionally
    limited to those needed for embedding.

*   **Small in dependencies.** Including the Molt interpreter in your project shouldn't
    drag in anything else--unless you ask for it.

*   **Easy to build.** Building Standard TCL is non-trivial.  Embedding
    Molt should be as simple as using any other crate.

*   **Easy to embed.** Extending Molt with TCL commands that wrap Rust APIs should
    be easy and simple.

Hence, perfect compatibility with Standard TCL is explicitly not a goal.  Many
features will not be implemented at all (e.g., octal literals); and others may
be implemented somewhat differently where a clearly better alternative exists
(e.g., `-nocomplain` will always be the normal behavior).  In addition, Molt will
prefer Rust standards where appropriate.

On the other hand, Molt is meant to be TCL (more or less), not simply a
"Tcl-like language", so gratuitous differences are to be avoided.  One of the
goals of this document is to carefully delineate:

*   The features that have not yet been implemented.
*   The features that likely will never be implemented.
*   Any small differences in behavior.
*   And especially, any features that have intentionally been implemented in
    a different way.

## What Molt Is For

Using Molt, you can:

*   Create a shell interpreter for scripting and interactive testing of your Rust crates.
*   Provide your Rust applications with an interactive REPL for debugging and
    administration.
*   Extend your Rust application with scripts provided at compile-time or at run-time.
*   Allow your users to script your applications and libraries.

See the [`molt-sample` repo](https://github.com/wduquette/molt-sample) for a sample Molt client
skeleton.

## New in Molt 0.3.2

Nothing, yet!  See the [Annotated Change Log](changes.md) for the new features by version.

## Coming Attractions

At this point Molt is capable and robust enough for real work, though the Rust-level API is
not yet completely stable.  Standard Rust `0.y.z` semantic versioning applies: ".y" changes
can break the Rust-level API, ".z" changes will not.

*   More TCL Commands
*   Testing improvements
*   Documentation improvements
*   Feature: Regex and Glob pattern matching by Molt commands
