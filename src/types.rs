//! Public Type Declarations

use crate::interp::Interp;

/// The interpreter's result.
/// TODO: This will need to be more general in the long run.
pub type InterpResult = Result<String,String>;

/// A simple command function, used to implement a command without any attached
/// context data.
pub type CommandFunc = fn(&mut Interp, &[&str]) -> InterpResult;

/// A trait defining a command object: a struct that implements a command (and may also
/// have context data).
pub trait Command {
    fn execute(&self, interp: &mut Interp, argv: &[&str]) -> InterpResult;
}
