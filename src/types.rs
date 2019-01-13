//! Public Type Declarations

use crate::interp::Interp;

/// "Ok" result codes for GCL calls
pub enum ResultCode {
    /// A normal result, e.g., TCL_OK
    Normal,

    /// A script called [return $value]
    Return,

    /// A script called [break] in a loop
    Break,

    /// A script called [continue] in a loop
    Continue,
}

/// The interpreter's result.
/// Err(()) corresponds to TCL_ERROR; the other ResultCodes corresponds to TCL_OK, etc.
pub type InterpResult = Result<ResultCode,()>;

/// A simple command function, used to implement a command without any attached
/// context data.
pub type CommandFunc = fn(&mut Interp, &[&str]) -> InterpResult;

/// A trait defining a command object: a struct that implements a command (and may also
/// have context data).
pub trait Command {
    fn execute(&self, interp: &mut Interp, argv: &[&str]) -> InterpResult;
}
