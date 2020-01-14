//! Molt Error Type
//!
//! Experimental; might eventually replace ResultCode.

use crate::value::Value;
use crate::types::MoltInt;

#[derive(Debug)]
enum Code {
    // Used only with `return -code`
    Okay,
    Error,
    Return,
    Break,
    Continue,
    Other(MoltInt)
}

#[derive(Debug)]
struct MoltErr {
    /// The current result code.  If `level` > 0, `next_code` will eventually replace it.
    code: Code,

    /// The result value.
    result: Value,

    /// The error code
    error_code: Value,

    /// The error info
    error_info: Vec<String>,

    /// The stack level at which the `next_code` replaces the `code`.
    level: usize,

    /// A code that will replace the current code after `level` stack levels.
    /// Or something like that.
    next_code: Option<Code>,
}

type NewResult = Result<Value, MoltErr>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_args() {
        // Hah!  MoltErr doesn't need to be immutable; it's reasonable to allow callers to
        // add stack levels as it propagates.  Woohoo!
        if let Err(mut me) = throw("Foobar!") {
            me.error_info.push("Baz!".into());
        }
    }

    fn throw(msg: &str) -> NewResult {
        let mut me = MoltErr {
            code: Code::Error,
            result: Value::empty(),
            error_code: Value::empty(),
            error_info: Vec::new(),
            level: 1,
            next_code: None,
        };

        me.error_info.push(msg.into());

        Err(me)
    }
}
