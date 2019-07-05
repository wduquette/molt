# The MoltResult Type

`MoltResult` is Molt's standard `Result<T,E>` type; it is defined as

```rust
pub type MoltResult = Result<Value, ResultCode>;
```

The `Value` type is described in the [previous section](./molt_value.md); by default, many
Molt methods and functions return `Value` on success.

The `ResultCode` is more complicated, as it is used to pass not only errors but also
to manage control flow.  It is defined as follows:

```rust
#[derive(Eq, PartialEq, Clone, Debug)]
pub enum ResultCode {
    Error(Value),
    Return(Value),
    Break,
    Continue,
}
```

In addition to a normal `Ok` result, a Molt function, method, or command can return:

* `ResultCode::Error(msg)`, where `msg` is an error message; this indicates that
   something has thrown an error.

* `ResultCode::Return(value)`, which indicates that the Molt code has called the
  `return` command; the `value` is the returned value.  Molt procedures, defined using
  the `proc` command, will catch this and return `value` as the value of the procedure.

* `ResultCode::Break` and `ResultCode::Continue` are returned by the `break` and
  `continue` commands and control loop execution in the usual way.

## `molt_ok!` and `molt_err!`

Application-specific Rust code will usually only use `Ok(value)` and
`Err(ReturnCode::Error(value))`. Since these two cases pop up so often,
Molt provides a couple of macros to make them easier: `molt_ok!` and `molt_err!`.  

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

`molt_err!` works just the same way, but returns `Err(ReturnCode::Err(Value))`.

```
if x > 5 {
    return molt_err!("value is out of range: {}", x);
}
```
