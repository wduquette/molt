//! Convenience Macros
///
/// This module contains macros for use by command authors.

/// Returns an `Ok` `MoltResult`.
///
/// If called with no arguments, returns an empty string as the `Ok` result.
/// If called with one argument, returns the argument as the `Ok` result.
/// If called with two or more arguments, computes the `Ok` result using
/// `format!()`; the first argument is naturally the format string.
#[macro_export]
macro_rules! molt_ok {
    () => (
        Ok(Value::new(""))
    );
    ($arg:expr) => (
        Ok(Value::from_string($arg.to_string()))
    );
    ($($arg:tt)*) => (
        Ok(Value::from_string(format!($($arg)*)))
    )
}

/// Returns an `Error` `MoltResult`.  The error message is formatted
/// as with `format!()`.
#[macro_export]
macro_rules! molt_err {
    ($arg:expr) => (
        Err(ResultCode::Error(Value::from_string($arg.to_string())))
    );
    ($($arg:tt)*) => (
        Err(ResultCode::Error(Value::from_string(format!($($arg)*))))
    )
}
