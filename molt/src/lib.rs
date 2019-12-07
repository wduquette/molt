//! # Molt Client Library
//!
//! This module is the primary API for Molt users.

#![doc(html_root_url = "https://docs.rs/molt/0.1.0")]

pub use crate::interp::Interp;
pub use crate::list::list_to_string;
pub use crate::types::*;
pub use crate::test_harness::test_harness;

mod commands;
mod eval_ptr;
mod expr;
pub mod interp;
mod list;
pub mod tokenizer;
#[macro_use]
mod macros;
mod parser;
mod scope;
mod test_harness;
pub mod types;
mod util;
pub mod value;

/// Checks to see whether a command's argument list is of a reasonable size.
/// Returns an error if not.  The arglist must have at least min entries, and can have up
/// to max.  If max is 0, there is no maximum.  argv[0] is always the command name, and
/// is included in the count; thus, min should always be >= 1.
///
/// *Note:* Defined as a function because it doesn't need anything from the Interp.
pub fn check_args(
    namec: usize,
    argv: &[Value],
    min: usize,
    max: usize,
    argsig: &str,
) -> MoltResult {
    assert!(namec >= 1);
    assert!(min >= 1);
    assert!(!argv.is_empty());

    if argv.len() < min || (max > 0 && argv.len() > max) {
        let cmd_tokens = Value::from(&argv[0..namec]);
        molt_err!(
            "wrong # args: should be \"{} {}\"",
            cmd_tokens.to_string(),
            argsig
        )
    } else {
        molt_ok!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_args() {
        assert_ok(&check_args(1, &mklist(vec!["mycmd"].as_slice()), 1, 1, ""));
        assert_ok(&check_args(
            1,
            &mklist(vec!["mycmd"].as_slice()),
            1,
            2,
            "arg1",
        ));
        assert_ok(&check_args(
            1,
            &mklist(vec!["mycmd", "data"].as_slice()),
            1,
            2,
            "arg1",
        ));
        assert_ok(&check_args(
            1,
            &mklist(vec!["mycmd", "data", "data2"].as_slice()),
            1,
            0,
            "arg1",
        ));

        assert_err(
            &check_args(1, &mklist(vec!["mycmd"].as_slice()), 2, 2, "arg1"),
            "wrong # args: should be \"mycmd arg1\"",
        );
        assert_err(
            &check_args(
                1,
                &mklist(vec!["mycmd", "val1", "val2"].as_slice()),
                2,
                2,
                "arg1",
            ),
            "wrong # args: should be \"mycmd arg1\"",
        );
    }

    // TODO: stopgap until we have finalized the MoltList API.
    fn mklist(argv: &[&str]) -> MoltList {
        argv.iter().map(|s| Value::from(*s)).collect()
    }

    // Helpers

    fn assert_err(result: &MoltResult, msg: &str) {
        assert_eq!(molt_err!(msg), *result);
    }

    fn assert_ok(result: &MoltResult) {
        assert!(result.is_ok(), "Result is not Ok");
    }
}
