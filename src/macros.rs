//! Convenience Macros
///
/// This module contains macros for use by command authors.

/// Returns an `Ok` `InterpResult`.
///
/// If called with no arguments, returns an empty string as the `Ok` result.
/// If called with one argument, returns the argument as the `Ok` result.
/// If called with two or more arguments, computes the `Ok` result using
/// `format!()`; the first argument is naturally the format string.
#[macro_export]
macro_rules! molt_ok {
    () => (
        Ok("".to_string())
    );
    ($arg:expr) => (
        Ok($arg.to_string())
    );
    ($($arg:tt)*) => (
        Ok(format!($($arg)*))
    )
}

/// Returns an `Error` `InterpResult`.  The error message is formatted
/// as with `format!()`.
#[macro_export]
macro_rules! molt_err {
    ($($arg:tt)*) => (
        Err(ResultCode::Error(format!($($arg)*)))
    )
}
