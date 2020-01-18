//! Molt Error Type
//!
//! Experimental; might eventually replace ResultCode.  The goal of this module is to work
//! out the ergonomics of the new solution.

use crate::types::MoltInt;
use crate::value::Value;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Code {
    /// Used only with `return -code`
    Okay,
    Error,
    Return,
    Break,
    Continue,
    Other(MoltInt),
}

#[derive(Debug)]
pub struct MoltErr {
    /// The desired result code.
    code: Code,

    /// The result value.
    result: Value,

    /// The error code
    error_code: Value,

    /// The error info
    error_info: Vec<String>,

    /// The stack level at which the `code` takes effect.  Each level lost decrements this,
    /// until it's 0; then it's handled normally.
    level: usize,
}

impl MoltErr {
    pub fn error(msg: Value) -> Self {
        Self {
            code: Code::Error,
            result: msg,
            error_code: Value::from("NONE"),
            error_info: Vec::new(),
            level: 0,
        }
    }
    pub fn code(&self) -> Code {
        self.code
    }

    pub fn result(&self) -> Value {
        self.result.clone()
    }
}

pub type NewResult = Result<Value, MoltErr>;

#[cfg(test)]
#[allow(dead_code)]
mod tests {
    // use super::*;
    // use crate::molt_err2;

    // #[test]
    // fn test_foo() {
    //     // Hah!  MoltErr doesn't need to be immutable; it's reasonable to allow callers to
    //     // add stack levels as it propagates.  Woohoo!
    //     if let Err(mut err) = throw("Foobar!") {
    //         err.error_info.push("Baz!".into());
    //     }
    // }

    // fn test_bar() {
    //     let res: NewResult = molt_err2!("Foobar!");
    //
    //     if let Err(err) = res {
    //         match err.code() {
    //             Code::Error => {
    //                 println!("Got an error: {}", err.result());
    //             },
    //             Code::Break => {
    //                 println!("Got a break!");
    //             }
    //             _ => ()
    //         }
    //     }
    // }
    //
    // fn throw(msg: &str) -> NewResult {
    //     let mut me = MoltErr {
    //         code: Code::Error,
    //         result: Value::empty(),
    //         error_code: Value::empty(),
    //         error_info: Vec::new(),
    //         level: 1,
    //     };
    //
    //     me.error_info.push(msg.into());
    //
    //     Err(me)
    // }
}
