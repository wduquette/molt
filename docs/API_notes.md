# API Notes

Notes on Molt's Rust API, from reading the Rust API guidelines.

## Miscellaneous Things to change

### molt::types

* The public types should be `pub used` in the main package.
* Documentation needs to be fleshed out.
* Internal tests need to be fleshed out in general

## Interp Methods

Also, the module's `subst_backslashes` function should be an
`Interp` method, to be parallel with the other `subst_*` methods
we need.

## Expression Evaluation

The `Interp` should provide expression evaluation methods for use by client code.

## Other Notes

### Method Names

* `as`, `to`, and `into`, and extensions of these, are conversions of the
  struct into something else.
* Getters do not use `get_` as a prefix; prefer `name()` to `get_name()`.
  * If the struct has only one thing to get, `get()` is acceptable.
  * Note: this leaves `get_int()`, etc., available as Interp methods for
    converting Molt values to concrete data types.

### Standard Traits

* Where possible, implement Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash,
  Debug, Display, Default.
