//! Public Type Declarations
//!
//! This module defines a number of types used throughout Molt's public API.
//!
//! The most important types are [`Value`], the type of data values in the Molt
//! language, and [`MoltResult`], Molt's standard `Result<T,E>` type.  `MoltResult`
//! is an alias for `Result<Value,Exception>`, where [`Exception`] contains the data
//! relating to an exceptional return from a script.  The heart of `Exception` is the
//! [`ResultCode`], which represents all of the ways a Molt script might return early:
//! errors, explicit returns, breaks, and continues.
//!
//! [`MoltInt`], [`MoltFloat`], [`MoltList`], and [`MoltDict`] a/Displayre simple type aliases
//! defining Molt's internal representation for integers, floats, and TCL lists and
//! dictionaries.
//!
//! [`MoltResult`]: type.MoltResult.html
//! [`Exception`]: type.Exception.html
//! [`MoltInt`]: type.MoltInt.html
//! [`MoltFloat`]: type.MoltFloat.html
//! [`MoltList`]: type.MoltList.html
//! [`MoltDict`]: type.MoltDict.html
//! [`ResultCode`]: enum.ResultCode.html
//! [`Value`]: ../value/index.html
//! [`interp`]: interp/index.html

use std::str::FromStr;
use crate::interp::Interp;
pub use crate::value::Value;
use indexmap::IndexMap;
use std::fmt;

// Molt Numeric Types

/// The standard integer type for Molt code.
///
/// The interpreter uses this type internally for all Molt integer values.
/// The primary reason for defining this as a type alias is future-proofing: at
/// some point we may wish to replace `MoltInt` with a more powerful type that
/// supports BigNums, or switch to `i128`.
pub type MoltInt = i64;

/// The standard floating point type for Molt code.
///
/// The interpreter uses this type internally for all Molt floating-point values.
/// The primary reason for defining this as a type alias is future-proofing: at
/// some point we may wish to replace `MoltFloat` with `f128`.
pub type MoltFloat = f64;

/// The standard list type for Molt code.
///
/// Lists are an important data structure, both in Molt code proper and in Rust code
/// that implements and works with Molt commands.  A list is a vector of `Value`s.
pub type MoltList = Vec<Value>;

/// The standard dictionary type for Molt code.
///
/// A dictionary is a mapping from `Value` to `Value` that preserves the key insertion
/// order.
pub type MoltDict = IndexMap<Value, Value>;

/// The standard `Result<T,E>` type for Molt code.
///
/// This is the return value of all Molt commands, and the most common return value
/// throughout the Molt code base.  Every Molt command returns a [`Value`] on success;
/// if the command has no explicit return value, it returns the empty `Value`, a `Value`
/// whose string representation is the empty string.
///
/// A Molt command returns an [`Exception`] whenever the calling Molt script should
/// return early: on error, when returning an explicit result via the `return` command,
/// or when breaking out of a loop via the `break` or `continue` commands.  The precise
/// nature of the return is indicated by the [`Exception`]'s [`ResultCode`].
///
/// Many of the functions in Molt's Rust API also return `MoltResult`, for easy use within
/// Molt command definitions.
///
/// [`Exception`]: struct.Exception.html
/// [`ResultCode`]: enum.ResultCode.html
/// [`Value`]: ../value/index.html
pub type MoltResult = Result<Value, Exception>;

/// This enum represents the different kinds of [`Exception`] that result from
/// evaluating a Molt script.
///
/// Client code will usually see only the `Error` code; the others will most often be caught
/// and handled within the interpreter.  However, client code may explicitly catch and handle
/// the `Return`, `Break`, and `Continue` codes at both the Rust and the TCL level
/// (see the `catch` command) in order to implement application-specific control structures.
///
/// # Future Work
///
/// * Standard TCL allows for an arbitrary number of result codes, which in turn allows the
///   application to define an arbitrary number of new kinds of control structures that are
///   distinct from the standard ones.  This type provides the `Other` code for this
///   purpose; however, there is as yet no support for it in either the Rust or TCL APIs.
///
/// [`Exception`]: struct.Exception.html

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ResultCode {
    /// Value for `return -code` to indicate returning `Ok(value)` higher up the stack.
    Okay,

    /// A Molt error.  The `Exception::value` is the error message for display to the user.
    Error,

    /// An explicit return from a Molt procedure.  The `Exception::value` is the returned
    /// value, or the empty value if `return` was called without a return value.  This result
    /// will bubble up until it reaches the top-level of the enclosing procedure, which will
    /// then return the value as a normal `Ok` result.  If it is received when evaluating an
    /// arbitrary script, i.e., if `return` is called outside of any procedure, the
    /// interpreter will convert it into a normal `Ok` result.
    Return,

    /// A `break` in a Molt loop.  It will break out of the inmost enclosing loop in the usual
    /// way.  If it is returned outside a loop (or some user-defined control structure that
    /// supports `break`), the interpreter will convert it into an error.
    Break,

    /// A `continue` in a Molt loop.  Execution will continue with the next iteration of
    /// the inmost enclosing loop in the usual way.  If it is returned outside a loop (or
    /// some user-defined control structure that supports `break`), the interpreter will
    /// convert it into an error.
    Continue,

    /// Experimental; not in use yet.
    Other(MoltInt),
}

impl fmt::Display for ResultCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ResultCode::Okay => write!(f, "ok"),
            ResultCode::Error => write!(f, "error"),
            ResultCode::Return => write!(f, "return"),
            ResultCode::Break => write!(f, "break"),
            ResultCode::Continue => write!(f, "continue"),
            ResultCode::Other(code) => write!(f, "{}", *code),
        }
    }
}

impl FromStr for ResultCode {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "ok" => return Ok(ResultCode::Okay),
            "error" => return Ok(ResultCode::Error),
            "return" => return Ok(ResultCode::Return),
            "break" => return Ok(ResultCode::Break),
            "continue" => return Ok(ResultCode::Continue),
            _ => (),
        }

        match Value::get_int(value) {
            Ok(num) => {
                match num {
                    0 => Ok(ResultCode::Okay),
                    1 => Ok(ResultCode::Error),
                    2 => Ok(ResultCode::Return),
                    3 => Ok(ResultCode::Break),
                    4 => Ok(ResultCode::Continue),
                    _ => Ok(ResultCode::Other(num)),
                }
            }
            Err(exception) => Err(exception.value().as_str().into()),
        }
    }
}

impl ResultCode {
    /// A convenience: retrieves the enumerated value, converting it from
    /// `Option<ResultCode>` into `Result<ResultCode,Exception>`.
    pub fn from_value(value: &Value) -> Result<Self, Exception> {
        if let Some(x) = value.as_copy::<ResultCode>() {
            Ok(x)
        } else {
            molt_err!("invalid result code: \"{}\"", value)
        }
    }

    /// Return the result code as an integer.
    pub fn as_int(&self) -> MoltInt {
        match self {
            ResultCode::Okay => 0,
            ResultCode::Error => 1,
            ResultCode::Return => 2,
            ResultCode::Break => 3,
            ResultCode::Continue => 4,
            ResultCode::Other(num) => *num,
        }
    }
}

/// This enum represents the exceptional results of evaluating a Molt script, as
/// used in [`MoltResult`].  It is often used as the `Err` type for other
/// functions in the Molt API, so that these functions can easily return errors when used
/// in the definition of Molt commands.
///
/// A Molt script can return a normal result, as indicated by [`MoltResult`]'s `Ok`
/// variant, or it can return one of a number of exceptional results, which
/// will bubble up the call stack in the usual way until caught.  The different kinds of
/// exceptional result are defined by the [`ResultCode`] enum.
///
/// # Future Work
///
/// * Accumulate the error stack trace (the `errorInfo`) as error exceptions bubble up.
/// * Accept and retain the error code (the `errorCode`)
/// * Support the `return` command's options
/// * Support the `catch` command's `optionVarName`.
///
/// [`ResultCode`]: enum.ResultCode.html
/// [`MoltResult`]: type.MoltResult.html

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Exception {
    /// The kind of exception
    code: ResultCode,

    /// The result value
    value: Value,

    /// The return -level value.  Should be non-zero only for `Return`.
    level: usize,

    /// The return -code value.  Should be equal to `code`, except for `code == Return`.
    next_code: ResultCode,

    /// The error info, if any.
    error_data: Option<ErrorData>,
}

impl Exception {
    /// Creates an `Error` exception with the given error message.
    pub fn molt_err(msg: Value) -> Self {
        let data = ErrorData::new(Value::from("NONE"), msg.as_str());

        Self {
            code: ResultCode::Error,
            value: msg,
            level: 0,
            next_code: ResultCode::Error,
            error_data: Some(data),
        }
    }

    /// Creates an `Error` exception with the given code and error message.
    pub fn molt_err2(error_code: Value, msg: Value) -> Self {
        let data = ErrorData::new(error_code, msg.as_str());

        Self {
            code: ResultCode::Error,
            value: msg,
            level: 0,
            next_code: ResultCode::Error,
            error_data: Some(data),
        }
    }

    /// Creates a `Return` exception, with the given return value.  Return `Value::empty()`
    /// if there is no specific result.
    pub fn molt_return(value: Value) -> Self {

        Self {
            code: ResultCode::Return,
            value,
            level: 1,
            next_code: ResultCode::Okay,
            error_data: None,
        }
    }

    /// Creates an extended `Return` exception with the given return value, `-level`,
    /// and `-code`. Return `Value::empty()` if there is no specific result.
    ///
    /// This function is primarily intended for use by the `return` command.
    ///
    /// It's an error if level == 0 and next_code == Okay; that's
    /// `Ok(value)` rather than an exception.
    pub(crate) fn molt_return_ext(value: Value, level: usize, next_code: ResultCode) -> Self {
        assert!(level > 0 || next_code != ResultCode::Okay);

        Self {
            code: ResultCode::Return,
            value,
            level,
            next_code,
            error_data: None,
        }
    }

    /// Creates a `Break` exception.
    pub fn molt_break() -> Self {
        Self {
            code: ResultCode::Break,
            value: Value::empty(),
            level: 0,
            next_code: ResultCode::Break,
            error_data: None,
        }
    }

    /// Creates a `Continue` exception.
    pub fn molt_continue() -> Self {
        Self {
            code: ResultCode::Continue,
            value: Value::empty(),
            level: 0,
            next_code: ResultCode::Continue,
            error_data: None,
        }
    }

    /// Only when the ResultCode is Return:
    ///
    /// * Decrements the -level.
    /// * If it's 0, sets code to -code.
    ///
    /// This is used in `Interp::eval_script` to implement the `return` command's
    /// `-code` and  `-level` protocol.
    pub(crate) fn decrement_level(&mut self) {
        assert!(self.code == ResultCode::Return && self.level > 0);
        self.level -= 1;
        if self.level == 0 {
            self.code = self.next_code;
        }
    }

    /// Returns true if the exception is an error exception, and false otherwise.
    pub fn is_error(&self) -> bool {
        self.code == ResultCode::Error
    }

    /// Gets the exception's result code.
    pub fn code(&self) -> ResultCode {
        self.code
    }

    /// Gets the exception's next code (when `code == ResultCode::Return` only)
    pub fn next_code(&self) -> ResultCode {
        self.next_code
    }

    /// Gets the exception's level.
    pub fn level(&self) -> usize {
        self.level
    }

    /// Gets the exception's value, i.e., the explicit return value or the error message.
    pub fn value(&self) -> Value {
        self.value.clone()
    }

    /// Gets the exception's error data, if any.
    pub fn error_data(&self) -> Option<&ErrorData> {
        self.error_data.as_ref()
    }

    pub fn is_new_error(&self) -> bool {
        if let Some(data) = &self.error_data {
            data.is_new()
        } else {
            false
        }
    }

    /// Adds a line to the exception's error info.
    ///
    /// # Panics
    ///
    /// Panics if the exception is not an error exception.
    pub fn add_error_info(&mut self, line: &str) {
        if let Some(data) = &mut self.error_data {
            data.add_info(line);
        } else {
            panic!("add_error_info called for non-Error Exception");
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ErrorData {
    /// The error code; defaults to "NONE"
    error_code: Value,

    /// The TCL stack trace.
    stack_trace: Vec<String>,
}

impl ErrorData {
    // Creates a new ErrorData given the error code and error message.
    fn new(error_code: Value, error_msg: &str) -> Self {
        Self {
            error_code,
            stack_trace: vec![error_msg.into()],
        }
    }

    /// Returns the error code.
    pub fn error_code(&self) -> Value {
        self.error_code.clone()
    }

    pub fn is_new(&self) -> bool {
        self.stack_trace.len() == 1
    }

    /// Returns the stack trace.
    pub fn error_info(&self) -> Value {
        Value::from(self.stack_trace.join("\n"))
    }

    /// Adds to the stack trace.
    pub(crate) fn add_info(&mut self, info: &str) {
        self.stack_trace.push(info.into());
    }
}

/// A unique identifier, used to identify cached context data within a given
/// interpreter.  For more information see the discussion of command definition
/// and the context cache in [The Molt Book] and the [`interp`] module.
///
/// [The Molt Book]: https://wduquette.github.io/molt/
/// [`interp`]: ../interp/index.html

#[derive(Eq, PartialEq, Debug, Hash, Copy, Clone)]
pub struct ContextID(pub(crate) u64);

/// A function used to implement a binary Molt command. For more information see the
/// discussion of command definition in [The Molt Book] and the [`interp`] module.
///
/// The command may retrieve its application context from the [`interp`]'s context cache
/// if it was defined with a [`ContextID`].
///
/// The command function receives the interpreter, the context ID, and a slice
/// representing the command and its arguments.
///
/// [The Molt Book]: https://wduquette.github.io/molt/
/// [`interp`]: ../interp/index.html
/// [`ContextID`]: struct.ContextID.html
pub type CommandFunc = fn(&mut Interp, ContextID, &[Value]) -> MoltResult;

/// A Molt command that has subcommands is called an _ensemble_ command.  In Rust code,
/// the ensemble is defined as an array of `Subcommand` structs, each one mapping from
/// a subcommand name to the implementing [`CommandFunc`].  For more information,
/// see the discussion of command definition in [The Molt Book] and the [`interp`] module.
///
/// The tuple fields are the subcommand's name and implementing [`CommandFunc`].
///
/// [The Molt Book]: https://wduquette.github.io/molt/
/// [`interp`]: ../interp/index.html
/// [`CommandFunc`]: type.CommandFunc.html
pub struct Subcommand(pub &'static str, pub CommandFunc);

impl Subcommand {
    /// Looks up a subcommand of an ensemble command by name in a table,
    /// returning the usual error if it can't be found.  It is up to the
    /// ensemble command to call the returned subcommand with the
    /// appropriate arguments.  See the implementation of the `info`
    /// command for an example.
    ///
    /// # TCL Notes
    ///
    /// * In standard TCL, subcommand lookups accept any unambiguous prefix of the
    ///   subcommand name, as a convenience for interactive use.  Molt does not, as it
    ///   is confusing when used in scripts.
    pub fn find<'a>(
        ensemble: &'a [Subcommand],
        sub_name: &str,
    ) -> Result<&'a Subcommand, Exception> {
        for subcmd in ensemble {
            if subcmd.0 == sub_name {
                return Ok(subcmd);
            }
        }

        let mut names = String::new();
        names.push_str(ensemble[0].0);
        let last = ensemble.len() - 1;

        if ensemble.len() > 1 {
            names.push_str(", ");
        }

        if ensemble.len() > 2 {
            let vec: Vec<&str> = ensemble[1..last].iter().map(|x| x.0).collect();
            names.push_str(&vec.join(", "));
        }

        if ensemble.len() > 1 {
            names.push_str(", or ");
            names.push_str(ensemble[last].0);
        }

        molt_err!(
            "unknown or ambiguous subcommand \"{}\": must be {}",
            sub_name,
            &names
        )
    }
}

/// In TCL, variable references have two forms.  A string like "_some_var_(_some_index_)" is
/// the name of an array element; any other string is the name of a scalar variable.  This
/// struct is used when parsing variable references.  The `name` is the variable name proper;
/// the `index` is either `None` for scalar variables or `Some(String)` for array elements.
///
/// The Molt [`interp`]'s variable access API usually handles this automatically.  Should a
/// command need to distinguish between the two cases it can do so by using the
/// the [`Value`] struct's `Value::as_var_name` method.
///
/// [`Value`]: ../value/index.html
/// [`interp`]: ../interp/index.html
#[derive(Debug, Eq, PartialEq)]
pub struct VarName {
    name: String,
    index: Option<String>,
}

impl VarName {
    /// Creates a scalar `VarName` given the variable's name.
    pub fn scalar(name: String) -> Self {
        Self { name, index: None }
    }

    /// Creates an array element `VarName` given the element's variable name and index string.
    pub fn array(name: String, index: String) -> Self {
        Self {
            name,
            index: Some(index),
        }
    }

    /// Returns the parsed variable name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the parsed array index, if any.
    pub fn index(&self) -> Option<&str> {
        self.index.as_ref().map(|x| &**x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_result_code_as_string() {
        assert_eq!(Value::from_other(ResultCode::Okay).as_str(), "ok");
        assert_eq!(Value::from_other(ResultCode::Error).as_str(), "error");
        assert_eq!(Value::from_other(ResultCode::Return).as_str(), "return");
        assert_eq!(Value::from_other(ResultCode::Break).as_str(), "break");
        assert_eq!(Value::from_other(ResultCode::Continue).as_str(), "continue");
        assert_eq!(Value::from_other(ResultCode::Other(5)).as_str(), "5");

    }

    #[test]
    fn test_result_code_from_value() {
        assert_eq!(ResultCode::from_value(&"ok".into()), Ok(ResultCode::Okay));
        assert_eq!(ResultCode::from_value(&"error".into()), Ok(ResultCode::Error));
        assert_eq!(ResultCode::from_value(&"return".into()), Ok(ResultCode::Return));
        assert_eq!(ResultCode::from_value(&"break".into()), Ok(ResultCode::Break));
        assert_eq!(ResultCode::from_value(&"continue".into()), Ok(ResultCode::Continue));
        assert_eq!(ResultCode::from_value(&"5".into()), Ok(ResultCode::Other(5)));
        assert!(ResultCode::from_value(&"nonesuch".into()).is_err());
    }

    #[test]
    fn test_error_data() {
        let mut data = ErrorData::new("CODE".into(), "error message");

        assert_eq!(data.error_code(), "CODE".into());
        assert_eq!(data.error_info(), "error message".into());

        data.add_info("from unit test");
        assert_eq!(data.error_info(), "error message\nfrom unit test".into());
    }

    #[test]
    fn test_exception_molt_err() {
        let mut exception = Exception::molt_err("error message".into());

        assert_eq!(exception.code(), ResultCode::Error);
        assert_eq!(exception.value(), "error message".into());
        assert!(exception.is_error());
        assert!(exception.error_data().is_some());

        if let Some(data) = exception.error_data() {
            assert_eq!(data.error_code(), "NONE".into());
            assert_eq!(data.error_info(), "error message".into());
        }

        exception.add_error_info("from unit test");

        if let Some(data) = exception.error_data() {
            assert_eq!(data.error_info(), "error message\nfrom unit test".into());
        }
    }

    #[test]
    fn test_exception_molt_err2() {
        let exception = Exception::molt_err2("CODE".into(), "error message".into());

        assert_eq!(exception.code(), ResultCode::Error);
        assert_eq!(exception.value(), "error message".into());
        assert!(exception.is_error());
        assert!(exception.error_data().is_some());

        if let Some(data) = exception.error_data() {
            assert_eq!(data.error_code(), "CODE".into());
            assert_eq!(data.error_info(), "error message".into());
        }
    }

    #[test]
    #[should_panic]
    fn text_exception_add_error_info() {
        let mut exception = Exception::molt_break();

        exception.add_error_info("should panic; not an error exception");
    }

    #[test]
    fn test_exception_molt_return() {
        let exception = Exception::molt_return("result".into());

        assert_eq!(exception.code(), ResultCode::Return);
        assert_eq!(exception.value(), "result".into());
        assert!(!exception.is_error());
        assert!(!exception.error_data().is_some());
    }

    #[test]
    fn test_exception_molt_break() {
        let exception = Exception::molt_break();

        assert_eq!(exception.code(), ResultCode::Break);
        assert_eq!(exception.value(), "".into());
        assert!(!exception.is_error());
        assert!(!exception.error_data().is_some());
    }

    #[test]
    fn test_exception_molt_continue() {
        let exception = Exception::molt_continue();

        assert_eq!(exception.code(), ResultCode::Continue);
        assert_eq!(exception.value(), "".into());
        assert!(!exception.is_error());
        assert!(!exception.error_data().is_some());
    }
}
