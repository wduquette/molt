//! Public Type Declarations

use crate::interp::Interp;

/// Molt Integer Type
pub type MoltInteger = i32;

/// The result of calling a function during Molt script evaluation, other than
/// `Ok(str)`.
#[derive(Eq, PartialEq, Clone, Debug)]
pub enum ResultCode {
    Error(String),
    Return(String),
    Break,
    Continue,
}

impl ResultCode {
    pub fn is_error(&self) -> bool {
        match self {
            ResultCode::Error(_) => true,
            _ => false,
        }
    }
}

pub type InterpResult = Result<String, ResultCode>;

/// A simple command function, used to implement a command without any attached
/// context data.
pub type CommandFunc = fn(&mut Interp, &[&str]) -> InterpResult;

/// A trait defining a command object: a struct that implements a command (and may also
/// have context data).
pub trait Command {
    fn execute(&self, interp: &mut Interp, argv: &[&str]) -> InterpResult;
}

/// Used for defining subcommands of ensemble commands.
/// Full description TODO, because it might change.
pub struct Subcommand(pub &'static str, pub CommandFunc);
