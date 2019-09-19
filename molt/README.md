# molt -- An Embeddable Tcl Interpreter for Rust

Molt is an embeddable TCL interpreter for Rust.  Applications can define new TCL
commands in Rust and execute TCL scripts and strings.  For example,

```rust
use molt::Interp;
let mut interp = Interp::new();

let four = interp.eval("expr {2 + 2}")?;
assert_eq!(four.as_int(), 4);
```

A new command is defined like so:

```rust
/// # square *x*
///
/// Computes the square of a value
pub fn cmd_square(_interp: &mut Interp, argv: &[Value]) -> MoltResult {
    // Correct number of arguments?
    check_args(1, argv, 2, 2, "x")?;

    // Get x, if it's an integer
    let x = argv[1].as_int()?;

    // Return the result.
    molt_ok!(x * x)
}
```

and installed like so:

```rust
use molt::Interp;
let mut interp = Interp::new();
interp.addCommand("square", cmd_square);

let num = interp.eval("square 5")?;
assert_eq!(num.as_int(), 25);
```

Values are represented by the `Value` type, which can be converted to and from any type consistent
with the value's string representation: integers, floats, lists, and any type that defines the
`MoltAny` trait.

Molt is still a work in progress.  The basic TCL language is in place, but many TCL commands
remain to be implemented.  See the Molt Book for details.

See my [blog](https://wduquette.github.io/) for news,
[The Molt Book](https://wduquette.github.io/molt/) for details, and
the [GitHub Repo](https://github.com/wjduquette/molt) for issue tracking, etc.
