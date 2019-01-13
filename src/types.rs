//! Public Type Declarations

use crate::interp::Interp;
use self::Status::*;

/// "Ok" result codes for GCL calls
#[derive(Clone,Debug,PartialEq,Eq)]
pub enum Status {
    /// A normal result, e.g., TCL_OK
    Okay(Value),

    /// An error result.
    Error(Value),

    /// A script called [return $value]
    Return(Value),

    /// A script called [break] in a loop
    Break,

    /// A script called [continue] in a loop
    Continue,
}

impl Status {
    pub fn error(str: &str) -> Self {
        Error(Value::from(str))
    }

    pub fn result(str: &str) -> Self {
        Okay(Value::from(str))
    }

    pub fn okay() -> Self {
        Okay(Value::empty())
    }

    pub fn is_okay(&self) -> bool {
        if let Okay(_)= self {
            true
        } else {
            false
        }
    }

    pub fn is_error(&self) -> bool {
        if let Error(_)= self {
            true
        } else {
            false
        }
    }
}

/// A simple command function, used to implement a command without any attached
/// context data.
pub type CommandFunc = fn(&mut Interp, &[&str]) -> Status;

/// A trait defining a command object: a struct that implements a command (and may also
/// have context data).
pub trait Command {
    fn execute(&self, interp: &mut Interp, argv: &[&str]) -> Status;
}

/// A GCL data value
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Value {
    str: String,
}

impl Default for Value {
    fn default() -> Self {
        Value::empty()
    }
}

impl Value {
    pub fn empty() -> Self {
        Self {
            str: String::new(),
        }
    }

    pub fn new(str: String) -> Self {
        Self {
            str
        }
    }

    pub fn from(str: &str) -> Self {
        Self {
            str: str.into(),
        }
    }

    pub fn set(&mut self, str: &str) {
        self.str = String::from(str);
    }

    pub fn concat(&mut self, text: &str) {
        self.str.push_str(text);
    }

    pub fn as_string(&self) -> String {
        self.str.clone()
    }

    pub fn as_str(&self) -> &str {
        &self.str
    }

    pub fn clear(&mut self) {
        self.str.clear();
    }
}
