# API Notes

Notes on Molt's Rust API, from reading the Rust API guidelines.

## Miscellaneous Things to change

### molt::types

* The public types should be `pub used` in the main package.
* Documentation needs to be fleshed out.
* Internal tests need to be fleshed out in general

### Define `MoltList` type using the "newtype" pattern.

* This allows adding methods.  E.g., vec_string_to_str can be
  MoltList::as_vec_str(&self).

### molt::util should be pub(for crate).

## Argument Parsing Functions

Functions used by command definitions to check and convert arguments should
generally be Interp methods.  This gives them a standard place in the API,
and provides an opportunity to tweak the results according to the Interp
configuration should that be necessary.

Use `get_<type>` naming, for consistency with Standard TCL style.

| Function      | Method       |
| ------------- | ------------ |
| `check_args`  | `check_args` |
| `get_boolean` | `get_bool`   |
| `get_float`   | `get_float`  |
| `get_int`     | `get_int`    |
| `get_list`    | `get_list`   |

## String Representation Functions

We need a standard naming scheme for methods to convert Rust values into TCL
results (e.g., for return from a function or assignment to a variable).
This is different than in Standard TCL, since there's no "interp.result"
field.

Suggestion: `<type>_result()`, e.g, `list_result(list: Vec<String>) -> String`.

So:

| Function          | Method        |
| ----------------- | ------------- |
| `list_to_string`  | `list_result` |

## Interp Methods

Also, the module's `subst_backslashes` function should be an
`Interp` method, to be parallel with the other `subst_*` methods
we need.

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
