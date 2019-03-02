//! # Molt Client Library
//!
//! This module is the primary API for Molt users.

use crate::types::*;
pub use crate::list::get_list;
pub use crate::list::list_to_string;
pub use crate::test_harness::*;

#[allow(dead_code)] // Temporary
mod char_ptr;
mod commands;
mod context;
#[allow(dead_code)] // Temporary
mod expr;
pub mod interp;
mod list;
#[macro_use]
mod macros;
pub mod test_harness;
pub mod types;
pub mod util;
pub mod var_stack;

/// Checks to see whether a command's argument list is of a reasonable size.
/// Returns an error if not.  The arglist must have at least min entries, and can have up
/// to max.  If max is 0, there is no maximum.  argv[0] is always the command name, and
/// is included in the count; thus, min should always be >= 1.
///
/// *Note:* Defined as a function because it doesn't need anything from the Interp.
pub fn check_args(
    namec: usize,
    argv: &[&str],
    min: usize,
    max: usize,
    argsig: &str,
) -> InterpResult {
    assert!(namec >= 1);
    assert!(min >= 1);
    assert!(!argv.is_empty());

    if argv.len() < min || (max > 0 && argv.len() > max) {
        molt_err!("wrong # args: should be \"{} {}\"",
            argv[0..namec].join(" "),
            argsig
        )
    } else {
        molt_ok!()
    }
}

/// Converts an argument into a Molt integer, returning an error on failure.
/// A command function will call this to convert an argument into an integer,
/// using "?" to propagate errors to the interpreter.
///
/// TODO: support hex as well.  Util util::read_int at the same time.
///
/// # Example
///
/// ```
/// # use molt::types::*;
/// # fn dummy() -> Result<MoltInt,ResultCode> {
/// let arg = "1";
/// let int = molt::get_int(arg)?;
/// # Ok(int)
/// # }
/// ```
pub fn get_int(arg: &str) -> Result<MoltInt, ResultCode> {
    match arg.parse::<MoltInt>() {
        Ok(int) => Ok(int),
        Err(_) => molt_err!("expected integer but got \"{}\"", arg),
    }
}

/// Converts an argument into a Molt float, returning an error on failure.
/// A command function will call this to convert an argument into a number,
/// using "?" to propagate errors to the interpreter.
///
/// # Example
///
/// ```
/// # use molt::types::*;
/// # fn dummy() -> Result<MoltFloat,ResultCode> {
/// let arg = "1e2";
/// let val = molt::get_float(arg)?;
/// # Ok(val)
/// # }
/// ```
pub fn get_float(arg: &str) -> Result<MoltFloat, ResultCode> {
    match arg.parse::<MoltFloat>() {
        Ok(val) => Ok(val),
        Err(_) => molt_err!("expected floating-point number but got \"{}\"", arg),
    }
}

/// Converts an argument into a boolean, returning an error on failure.
/// A command function will call this to convert an argument,
/// using "?" to propagate errors to the interpreter.
///
/// Boolean values:
///
/// true:
/// # Example
///
/// ```
/// # use molt::types::*;
/// # fn dummy() -> Result<bool,ResultCode> {
/// let arg = "yes";
/// let flag = molt::get_boolean(arg)?;
/// # Ok(flag)
/// # }
/// ```
pub fn get_boolean(arg: &str) -> Result<bool, ResultCode> {
    let value: &str = &arg.to_lowercase();
    match value {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        _ => molt_err!("expected boolean but got \"{}\"", arg),
    }
}

/// Looks up a subcommand of an ensemble command by name in a table,
/// returning the usual error if it can't be found.
///
/// Note: doesn't attempt to match partial names.
pub fn get_subcommand<'a>(subs: &'a [Subcommand], sub: &str) -> Result<&'a Subcommand, ResultCode> {
    for subcmd in subs {
        if subcmd.0 == sub {
            return Ok(subcmd);
        }
    }

    let mut names = String::new();
    names.push_str(subs[0].0);
    let last = subs.len() - 1;

    if subs.len() > 1 {
        names.push_str(", ");
    }

    if subs.len() > 2 {
        let vec: Vec<&str> = subs[1..last].iter().map(|x| x.0).collect();
        names.push_str(&vec.join(", "));
    }

    if subs.len() > 1 {
        names.push_str(", or ");
        names.push_str(subs[last].0);
    }

    molt_err!("unknown or ambiguous subcommand \"{}\": must be {}", sub, &names)
}

/// Converts a `Vec<String>` to a `Vec<&str>`.
pub fn vec_string_to_str(slice: &[String]) -> Vec<&str> {
    let result: Vec<&str> = slice.iter().map(|x| &**x).collect();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_args() {
        assert_ok(&check_args(1, vec!["mycmd"].as_slice(), 1, 1, ""));
        assert_ok(&check_args(1, vec!["mycmd"].as_slice(), 1, 2, "arg1"));
        assert_ok(&check_args(
            1,
            vec!["mycmd", "data"].as_slice(),
            1,
            2,
            "arg1",
        ));
        assert_ok(&check_args(
            1,
            vec!["mycmd", "data", "data2"].as_slice(),
            1,
            0,
            "arg1",
        ));

        assert_err(
            &check_args(1, vec!["mycmd"].as_slice(), 2, 2, "arg1"),
            "wrong # args: should be \"mycmd arg1\"",
        );
        assert_err(
            &check_args(1, vec!["mycmd", "val1", "val2"].as_slice(), 2, 2, "arg1"),
            "wrong # args: should be \"mycmd arg1\"",
        );
    }

    #[test]
    fn test_get_boolean() {
        assert_eq!(Ok(true), get_boolean("1"));
        assert_eq!(Ok(true), get_boolean("true"));
        assert_eq!(Ok(true), get_boolean("yes"));
        assert_eq!(Ok(true), get_boolean("on"));
        assert_eq!(Ok(true), get_boolean("TRUE"));
        assert_eq!(Ok(true), get_boolean("YES"));
        assert_eq!(Ok(true), get_boolean("ON"));
        assert_eq!(Ok(false), get_boolean("0"));
        assert_eq!(Ok(false), get_boolean("false"));
        assert_eq!(Ok(false), get_boolean("no"));
        assert_eq!(Ok(false), get_boolean("off"));
        assert_eq!(Ok(false), get_boolean("FALSE"));
        assert_eq!(Ok(false), get_boolean("NO"));
        assert_eq!(Ok(false), get_boolean("OFF"));
        assert_eq!(get_boolean("nonesuch"), molt_err!("expected boolean but got \"nonesuch\""));
    }

    #[test]
    fn test_get_int() {
        assert_eq!(get_int("1"), Ok(1));
        assert_eq!(get_int("-1"), Ok(-1));
        assert_eq!(get_int("+1"), Ok(1));
        assert_eq!(get_int("a"), molt_err!("expected integer but got \"a\""));
    }

    #[test]
    fn test_get_float() {
        assert_eq!(get_float("1"), Ok(1.0));
        assert_eq!(get_float("-1"), Ok(-1.0));
        assert_eq!(get_float("+1"), Ok(1.0));
        assert_eq!(get_float("1e3"), Ok(1000.0));
        assert_eq!(get_float("a"), molt_err!("expected floating-point number but got \"a\""));
    }

    // Helpers

    fn assert_err(result: &InterpResult, msg: &str) {
        assert_eq!(molt_err!(msg), *result);
    }

    fn assert_ok(result: &InterpResult) {
        assert!(result.is_ok(), "Result is not Ok");
    }
}
