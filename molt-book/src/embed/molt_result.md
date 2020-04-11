# The MoltResult Type

`MoltResult` is Molt's standard `Result<T,E>` type; it is defined as

```rust
pub type MoltResult = Result<Value, Exception>;
```

The `Value` type is described in the [previous section](./molt_value.md); by default, many
Molt methods and functions return `Value` on success.

The `Exception` struct is used for all exceptional returns, including not only errors but also
procedure returns, loop breaks and continues, and application-specific result codes defined
as part of application-specific control structures.

The heart of the `Exception` struct is the `ResultCode`, which indicates the kind of
exception return. It is defined as follows:

```rust
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ResultCode {
    Okay,
    Error,
    Return,
    Break,
    Continue,
    Other(MoltInt),
}
```

* `ResultCode::Okay` is used internally.

* `ResultCode::Error` indicates that an error has been thrown; the exception's
  `value()` is the error message.  Use the exception's `error_code()` and
  `error_info()` methods to access the error code and stack trace.

* `ResultCode::Return`, which indicates that the Molt code has called the
  `return` command; the `value` is the returned value.  Molt procedures, defined using
  the `proc` command, will catch this and return `value` as the value of the procedure call.
  See the documentation for the [**return**](../ref/return.md) and
  [**catch**](../ref/catch.md) commands for information on a variety of advanced things
  that can be done using this result code.

* `ResultCode::Break` and `ResultCode::Continue` are returned by the `break` and
  `continue` commands and control loop execution in the usual way.

* `ResultCode::Other` can be returned by the [**return**](../ref/return.md) command, and is
  used when defining application-specific control structures in script code.

Of these, client Rust code will usually only deal with `ResultCode::Error` and
`ResultCode::Return`.  For example,

```rust
# use molt::types::*;
# use molt::Interp;

let mut interp = Interp::new();

let input = "set a 1";

match interp.eval(input) {
   Ok(val) => {
       // Computed a Value
       println!("Value: {}", val);
   }
   Err(exception) => {
       if exception.is_error() {
           // Got an error; print it out.
           println!("Error: {}", exception.value());
       } else {
           // It's a Return.
           println!("Value: {}", exception.value());
       }
   }
}
```

## Result Macros

Application-specific Rust code will usually only use `Ok(value)` and
`ResultCode::Error`. Since these two cases pop up so often,
Molt provides several macros to make them easier: `molt_ok!`, `molt_err!`,
and `molt_throw!`.

`molt_ok!` takes one or more arguments and converts them into an `Ok(Value)`.

```rust
// Returns the empty result.
return molt_ok!();

// Returns its argument as a Value (if Molt knows how to convert it)
return molt_ok!(5);

// A plain Value is OK to.
return molt_ok!(Value::from(5));

// Returns a formatted string as a Value using a Rust `format!` string.
return molt_ok!("The answer is {}.", x);
```

`molt_err!` works just the same way, but returns `Err(Exception)` with `ResultCode::Error`.

```
// Return a simple error message
return molt_err!("error message");

// Return a formatted error message
if x > 5 {
    return molt_err!("value is out of range: {}", x);
}
```

`molt_throw!` is like `molt_err!`, but allows the caller to set an explicit error code.  (By
default, Molt errors have an error code of `NONE`.) Error codes can be retrieved from the
`Exception` object in Rust code and via the [**catch**](../ref/catch.md) command in scripts.

```
// Throw a simple error
return molt_throw!("MYCODE", "error message");

// Throw a formatted error message
if x > 5 {
    return molt_throw!("MYCODE", "value is out of range: {}", x);
}
```
