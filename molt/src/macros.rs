//! Convenience Macros
//!
//! This module contains macros for use by command authors.

/// Returns an `Ok` `MoltResult`.
///
/// If called with no arguments, returns an empty value as the `Ok` result.
/// If called with one argument, returns the argument as the `Ok` result.
/// If called with two or more arguments, computes the `Ok` result using
/// `format!()`; the first argument is naturally the format string.
#[macro_export]
macro_rules! molt_ok {
    () => (
        Ok(Value::empty())
    );
    ($arg:expr) => (
        Ok(Value::from($arg))
    );
    ($($arg:tt)*) => (
        Ok(Value::from(format!($($arg)*)))
    )
}

/// Returns an `Error` `MoltResult`.  The error message is formatted
/// as with `format!()`.
///
/// If called with one argument, returns the argument as the error message.
/// If called with two or more arguments, computes the error message using
/// `format!()`; the first argument is naturally the format string.
#[macro_export]
macro_rules! molt_err {
    ($arg:expr) => (
        Err(Exception::molt_err(Value::from($arg)))
    );
    ($($arg:tt)*) => (
        Err(Exception::molt_err(Value::from(format!($($arg)*))))
    )
}
