# Molt Library Crates

A Molt library crate is simply a Rust crate that can install commands into a
Molt interpreter using any of the methods described in this chapter. For example,
a crate might provide an `install` function:

```rust
use molt::Interp

pub fn install(interp: &mut Interp) {
    interp.add_command("mycommand", mycommand);
    ...
}
```
