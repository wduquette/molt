# Molt -- More Or Less TCL

[![Crates.io](https://img.shields.io/crates/v/molt.svg)](https://crates.io/crates/molt)

Molt is a minimal implementation of the TCL language for embedding in Rust apps and for
scripting Rust libraries.  See [The Molt Book](https://wduquette.github.io/molt) for details
and user documentation.

## New in Molt 0.2.0

**Associative Arrays:** Molt now includes TCL's associative array variables:

```text
% set a(1) "Howdy"
Howdy
% set a(foo.bar) 5
5
% puts [array get a]
1 Howdy foo.bar 5
```

**The Expansion Operator:** Molt now supports the `{*}` operator, which expands a single
command argument into multiple arguments:

```text
% set a {a b c}
% list 1 2 $a 3 4
1 2 {a b c} 3 4
% list 1 2 {*}$a 3 4
1 2 a b c 3 4
```

**Rust API Changes:**

*   The addition of array variables required changes to the `molt::Interp` struct's API for
    setting and retrieving variables.  In particular, the `molt::Interp::var`,
    `molt::Interp::set_var`, and `molt::Interp::set_and_return` methods now take the variable
    name as a `&Value` rather than a `&str`; this simplifies client code, and means that most
    commands implemented in Rust that work with variables don't need to care whether the
    variable in question is a scalar or an array element.


## Coming Attractions

At this point Molt is capable and robust enough for real work, though the Rust-level API is
not yet completely stable.  Standard Rust `0.y.z` semantic versioning applies: ".y" changes
can break the Rust-level API, ".z" changes will not.

*   Feature: Regex and Glob pattern matching by Molt commands
*   Improved support for implementing Molt commands in Rust that require associated context
    data.
*   Testing improvements
*   Documentation improvements

## Why Molt Exists

Using Molt, you can:

*   Create a shell interpreter for scripting and interactive testing of your Rust crates.
*   Provide your Rust applications with an interactive REPL for debugging and
    administration.
*   Extend your Rust application with scripts provided at compile-time or at run-time.
*   Allow your users to script your applications and libraries.

See the [`molt-sample` repo](https://github.com/wduquette/molt-sample) for a sample Molt client
skeleton.

## Molt and Standard TCL

Molt is intended to be lightweight and require minimal dependencies, so that it can be added
to any project without greatly increasing its footprint.  (At present, the core
language is a single library create with no dependencies at all!)  As such, it does not provide
all of the features of Standard TCL (e.g., TCL 8.6).

At the same time, Molt's implementation of TCL should be consistent with TCL 8.6 so far as it
goes.  Some archaic commands and command features are omitted; some changes
are made so Molt works better in the Rust ecosystem. (E.g., Molt's notion of whitespace is
the same as Rust's.) All liens against Standard TCL are documented in
the [The Molt Book](https://wduquette.github.io/molt).

No effort has been made to make the Rust-level API for extending Molt in Rust look like
Standard TCL's C API; rather, the goal is to make the Rust-level API as simple and ergonomic
as possible. **Note**: A big part of this effort is defining and refining the Rust API used
to interact with and extend the interpreter. If you have comments or suggestions for
improvement, please contact me or write an issue!

## Building and Installation

The easiest approach is to get the latest Molt through `crates.io`.  Look for the
`molt`, `molt-shell`, and `molt-app` crates, or add them to your dependencies list
in `cargo.toml`.

To build Molt:

*   Install the latest stable version of Rust (1.38.0 at time of writing)
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

## Acknowledgements

I've gotten help from many people in this endeavor; here's a (necessarily partial) list.

* Jonathan Castello, for general Rust info
* Kevin Kenny, for help with TCL numerics and general encouragement
* Don Porter, for help with TCL parsing
* rfdonnelly, for the crates.io badge, etc.
* Various folks from users.rust-lang.org who have answered my questions:
    * Krishna Sannasi, for help getting `Value` to work with arbitrary user data types
    * Yandros, for pointing me at `OnceCell` and `UnsafeCell`.
    * jethrogb, for help on how to use `Ref::map` to return a `Ref<T>` of a component deep within
      a `RefCell<S>` from a function.  (Mind you, once I got it working and gave it a try I
      tore it out again, because of `BorrowMutError` panics.  But I had to try it.)
