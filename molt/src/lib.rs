//! # Molt Client Library
//!
//! This module is the primary API for Molt users.

#![doc(html_root_url = "https://docs.rs/molt/0.1.0")]

pub use crate::types::*;
pub use crate::list::get_list;
pub use crate::list::list_to_string;
pub use crate::interp::Interp;

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
mod scope;
pub mod types;
mod util;

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

    // Helpers

    fn assert_err(result: &InterpResult, msg: &str) {
        assert_eq!(molt_err!(msg), *result);
    }

    fn assert_ok(result: &InterpResult) {
        assert!(result.is_ok(), "Result is not Ok");
    }
}
