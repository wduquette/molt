//! Convenience Macros
//!
//! This module contains macros for use by command authors.

/// Returns an `Ok` `MoltResult`.
///
/// If called with no arguments, returns an empty value as the `Ok` result.
/// If called with one argument, returns the argument as the `Ok` result, converting it
/// to a value automatically.
/// If called with two or more arguments, computes the `Ok` result using
/// `format!()`; the first argument is naturally the format string.
///
/// # Examples
///
/// ```
/// use molt::*;
///
/// // Return the empty result
/// fn func1() -> MoltResult {
///     // ...
///     molt_ok!()
/// }
///
/// assert_eq!(func1(), Ok(Value::empty()));
///
/// // Return an arbitrary value
/// fn func2() -> MoltResult {
///     // ...
///     molt_ok!(17)
/// }
///
/// assert_eq!(func2(), Ok(17.into()));
///
/// // Return a formatted value
/// fn func3() -> MoltResult {
///     // ...
///     molt_ok!("The answer is {}", 17)
/// }
///
/// assert_eq!(func3(), Ok("The answer is 17".into()));
/// ```
#[macro_export]
macro_rules! molt_ok {
    () => (
        Ok($crate::Value::empty())
    );
    ($arg:expr) => (
        Ok($crate::Value::from($arg))
    );
    ($($arg:tt)*) => (
        Ok($crate::Value::from(format!($($arg)*)))
    )
}

/// Returns an `Error` `MoltResult`.  The error message is formatted
/// as with `format!()`.
///
/// If called with one argument, the single argument is used as the error message.
/// If called with more than one argument, the first is a `format!()` format string,
/// and the remainder are the values to format.
///
/// This macro wraps the [`Exception::molt_err`](types/struct.Exception.html#method.molt_err)
/// method.
///
/// # Examples
///
/// ```
/// use molt::*;
///
/// // Return a simple error message
/// fn err1() -> MoltResult {
///     // ...
///     molt_err!("error message")
/// }
///
/// let result = err1();
/// assert!(result.is_err());
///
/// let exception = result.err().unwrap();
/// assert!(exception.is_error());
/// assert_eq!(exception.value(), "error message".into());
///
/// // Return a formatted error
/// fn err2() -> MoltResult {
///    // ...
///    molt_err!("invalid value: {}", 17)
/// }
///
/// let result = err2();
/// assert!(result.is_err());
///
/// let exception = result.err().unwrap();
/// assert!(exception.is_error());
/// assert_eq!(exception.value(), "invalid value: 17".into());
/// ```
#[macro_export]
macro_rules! molt_err {
    ($arg:expr) => (
        Err($crate::Exception::molt_err($crate::Value::from($arg)))
    );
    ($($arg:tt)*) => (
        Err($crate::Exception::molt_err($crate::Value::from(format!($($arg)*))))
    )
}

/// Returns an `Error` `MoltResult` with a specific error code.  The error message is formatted
/// as with `format!()`.
///
/// The macro requires two or more arguments.  The first argument is always the error code.
/// If called with two arguments, the second is the error message.
/// If called with more than two arguments, the second is a `format!()` format string and
/// the remainder are the values to format.
///
/// This macro wraps
/// the [`Exception::molt_err2`](types/struct.Exception.html#method.molt_err2)
/// method.
///
/// # Examples
///
/// ```
/// use molt::*;
///
/// // Throw a simple error
/// fn throw1() -> MoltResult {
///     // ...
///     molt_throw!("MYCODE", "error message")
/// }
///
/// let result = throw1();
/// assert!(result.is_err());
///
/// let exception = result.err().unwrap();
/// assert!(exception.is_error());
/// assert_eq!(exception.value(), "error message".into());
/// assert_eq!(exception.error_code(), "MYCODE".into());
///
/// // Return a formatted error
/// fn throw2() -> MoltResult {
///    // ...
///    molt_throw!("MYCODE", "invalid value: {}", 17)
/// }
///
/// let result = throw2();
/// assert!(result.is_err());
///
/// let exception = result.err().unwrap();
/// assert!(exception.is_error());
/// assert_eq!(exception.value(), "invalid value: 17".into());
/// assert_eq!(exception.error_code(), "MYCODE".into());
/// ```
#[macro_export]
macro_rules! molt_throw {
    ($code:expr, $msg:expr) => (
        Err($crate::Exception::molt_err2($crate::Value::from($code), $crate::Value::from($msg)))
    );
    ($code:expr, $($arg:tt)*) => (
        Err($crate::Exception::molt_err2($crate::Value::from($code), $crate::Value::from(format!($($arg)*))))
    )
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_molt_ok() {
        let result: MoltResult = molt_ok!();
        assert_eq!(Ok(Value::empty()), result);

        let result: MoltResult = molt_ok!(5);
        assert_eq!(Ok(Value::from(5)), result);

        let result: MoltResult = molt_ok!("Five");
        assert_eq!(Ok(Value::from("Five")), result);

        let result: MoltResult = molt_ok!("The answer is {}.", 5);
        assert_eq!(Ok(Value::from("The answer is 5.")), result);
    }

    #[test]
    fn test_molt_err() {
        check_err(molt_err!("error message"), "error message");
        check_err(molt_err!("error {}", 5), "error 5");
    }

    #[test]
    fn test_molt_throw() {
        check_throw(
            molt_throw!("MYERR", "error message"),
            "MYERR",
            "error message",
        );
        check_throw(molt_throw!("MYERR", "error {}", 5), "MYERR", "error 5");
    }

    fn check_err(result: MoltResult, msg: &str) -> bool {
        match result {
            Err(exception) => exception.is_error() && exception.value() == msg.into(),
            _ => false,
        }
    }

    fn check_throw(result: MoltResult, code: &str, msg: &str) -> bool {
        match result {
            Err(exception) => {
                exception.is_error()
                    && exception.value() == msg.into()
                    && exception.error_code() == code.into()
            }
            _ => false,
        }
    }
}
