use crate::types::*;

/// Checks to see whether a command's argument list is of a reasonable size.
/// Returns an error if not.  The arglist must have at least min entries, and can have up
/// to max.  If max is 0, there is no maximum.  argv[0] is always the command name, and
/// is included in the count; thus, min should always be >= 1.
///
/// *Note:* Defined as a function because it doesn't need anything from the Interp.
pub fn check_args(argv: &[&str], min: usize, max: usize, argsig: &str) -> Status {
    assert!(min >= 1);
    assert!(!argv.is_empty());

    if argv.len() < min || (max > 0 && argv.len() > max) {
        Status::error(&format!("wrong # args: should be \"{} {}\"", argv[0], argsig))
    } else {
        Status::okay()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_args() {
        assert_ok(&check_args(vec!["mycmd"].as_slice(), 1, 1, ""));
        assert_ok(&check_args(vec!["mycmd"].as_slice(), 1, 2, "arg1"));
        assert_ok(&check_args(vec!["mycmd","data"].as_slice(), 1, 2, "arg1"));
        assert_ok(&check_args(vec!["mycmd","data","data2"].as_slice(), 1, 0, "arg1"));

        assert_err(&check_args(vec!["mycmd"].as_slice(), 2, 2, "arg1"),
            "Wrong # args, should be: \"mycmd arg1\"");
        assert_err(&check_args(vec!["mycmd", "val1", "val2"].as_slice(), 2, 2, "arg1"),
            "Wrong # args, should be: \"mycmd arg1\"");
    }

    // Helpers

    fn assert_err(result: &Status, msg: &str) {
        assert_eq!(Status::error(msg), *result);
    }

    fn assert_ok(result: &Status) {
        assert!(result.is_okay(), "Result is not Ok");
    }

    // fn assert_value(result: Status, value: &str) {
    //     assert_eq!(Ok(value.into()), result);
    // }
}
