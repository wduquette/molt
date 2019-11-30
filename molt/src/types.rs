//! Public Type Declarations

use crate::interp::Interp;
pub use crate::value::Value;

// Molt Numeric Types

/// The standard integer type for Molt code.
///
/// The interpreter uses this type internally for all Molt integer values.
/// The primary reason for defining this as a type alias is future-proofing: at
/// some point we may wish to replace `MoltInt` with a more powerful type that
/// supports BigNums, or switch to `i128`.
pub type MoltInt = i64;

/// The standard floating point type for Molt code.
///
/// The interpreter uses this type internally for all Molt floating-pojint values.
/// The primary reason for defining this as a type alias is future-proofing: at
/// some point we may wish to replace `MoltFloat` with a more powerful type that
/// supports BigNums, or switch to `f128`.
pub type MoltFloat = f64;

/// The standard list type for Molt code.
///
/// Lists are an important data structure, both in Molt code proper and in Rust code
/// that implements and works with Molt commands.  A list is a list of `Value`s.
pub type MoltList = Vec<Value>;

/// Molt's standard `Result<T,E>` type.
///
/// This is the most common result value returned by Molt code.  The
/// `Ok` type is `Value`, the standard Molt value type; the `Err` type is
/// [`ResultCode`], which encompasses the four exceptional Molt return values.
///
/// [`ResultCode`]: enum.ResultCode.html
pub type MoltResult = Result<Value, ResultCode>;

/// Exceptional results of evaluating a Molt script.
///
/// A Molt script can return a normal result, as indicated by the `Ok`
/// [`MoltResult`], or it can return one of a number of exceptional results, which
/// will bubble up the call stack in the usual way until caught.
///
/// * `Error(Value)`: This code indicates a Molt error; the `Value` is the error message
///   for display to the user.
///
/// * `Return(Value)`: This code indicates that a Molt procedure called the
///   `return` command.  The `Value` is the returned value, or the empty value if
///   no value was returned.  This result will bubble up until it reaches the top-level
///   of the procedure, which will then return the value as a normal `Ok` result.  If
///   it is received when evaluating an arbitrary script, i.e., if `return` is called outside
///   of any procedure, the interpreter will convert it into a normal `Ok` result.
///
/// * `Break`: This code indicates that the Molt `break` command was called.  It will
///   break out of the inmost enclosing loop in the usual way.  When returned outside a
///   loop (or some user-defined control structure that supports `break`), the interpreter
///   will convert it into an error.
///
/// * `Continue`: This code indicates that the Molt `continue` command was called.  It will
///   continue with the next iteration of the inmost enclosing loop in the usual way.
///   When returned outside a loop (or some user-defined control structure that supports
///   `continue`), the interpreter will convert it into an error.
///
/// Client code will usually see only the `Error` code; the others will most often be caught
/// and handled within the interpreter.
///
/// # Future Work
///
/// * Standard TCL includes more information with non-`Ok` results, especially for error cases.
///   Ultimately, this type will be need to be extended to support that.
///
/// * Standard TCL allows for an arbitrary number of result codes, which in turn allows the
///   application to define an arbitrary number of new kinds of control structures that are
///   distinct from the standard ones.  At some point we might wish to add one or more
///   generic result codes, parallel to `Break` and `Continue`, for this purpose.
///
/// [`MoltResult`]: type.MoltResult.html
#[derive(Eq, PartialEq, Clone, Debug)]
pub enum ResultCode {
    Error(Value),
    Return(Value),
    Break,
    Continue,
}

impl ResultCode {
    /// Indicates whether the result code is an `Error(String)`.
    pub fn is_error(&self) -> bool {
        match self {
            ResultCode::Error(_) => true,
            _ => false,
        }
    }
}

/// A unique identifier, used to identify cached context data.
#[derive(Eq, PartialEq, Debug, Hash, Copy, Clone)]
pub struct ContextID(pub u64);

/// A trait defining a Molt command object: a struct that implements a command (and may also
/// have context data).
///
/// A simple command should be defined as a [`CommandFunc`]; define a full-fledged `Command`
/// struct when the command needs access to context data other than that provided by the
/// the interpreter itself.  For example, application-specific commands will often need
/// access to application data, which can be provided as attributes of the `Command`
/// struct.
///
/// TODO: Revise this so that `argv: &[Value]`.
///
/// [`CommandFunc`]: type.CommandFunc.html
pub trait Command {
    /// The `Command`'s execution method: the Molt interpreter calls this method  to
    /// execute the command.  The method receives the object itself, the interpreter,
    /// and an array representing the command and its arguments.
    fn execute(&self, interp: &mut Interp, argv: &[Value]) -> MoltResult;
}

/// A simple command function, used to implement a command without any attached
/// context data (other than the [`Interp`] itself).
///
/// The command function receives the interpreter and an array representing the
/// command and its arguments.
///
/// [`Interp`]: ../interp/struct.Interp.html
pub type CommandFunc = fn(&mut Interp, &[Value]) -> MoltResult;

/// A simple command function, used to implement a command that retrieves
/// application context from the [`Interp`]'s context cache.
///
/// The command function receives the interpreter, the context ID, and an array
/// representing the command and its arguments.
///
/// [`Interp`]: ../interp/struct.Interp.html
pub type ContextCommandFunc = fn(&mut Interp, ContextID, &[Value]) -> MoltResult;

/// Used for defining subcommands of ensemble commands.
///
/// The tuple fields are the subcommand's name and [`CommandFunc`].
///
/// TODO: This interface isn't yet stable; we probably want to support [`Command`]
/// instead of [`CommandFunc`].
///
/// [`Command`]: trait.Command.html
/// [`CommandFunc`]: type.CommandFunc.html
pub struct Subcommand(pub &'static str, pub CommandFunc);

impl Subcommand {
    /// Looks up a subcommand of an ensemble command by name in a table,
    /// returning the usual error if it can't be found.  It is up to the
    /// ensemble command to call the returned subcommand with the
    /// appropriate arguments.  See the implementation of the `info`
    /// command for an example.
    ///
    /// # TCL Notes
    ///
    /// * In standard TCL, subcommand lookups accept any unambiguous prefix of the
    ///   subcommand name, as a convenience for interactive use.  Molt does not.
    pub fn find<'a>(subs: &'a [Subcommand], sub: &str) -> Result<&'a Subcommand, ResultCode> {
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

        molt_err!(
            "unknown or ambiguous subcommand \"{}\": must be {}",
            sub,
            &names
        )
    }
}

/// The name of a variable.  For scalar variables, `index` is `None`; for array variables,
/// `index` is `Some(String)`.  This value is returned by `Value::as_var_name`.
#[derive(Debug, Eq, PartialEq)]
pub struct VarName {
    name: String,
    index: Option<String>,
}

impl VarName {
    /// Creates a scalar variable name.
    pub fn scalar(name: String) -> Self {
        Self { name, index: None }
    }

    /// Creates an array element name given its variable name and index string.
    pub fn array(name: String, index: String) -> Self {
        Self {
            name,
            index: Some(index),
        }
    }

    /// Returns the parsed variable name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the parsed array index, if any.
    pub fn index(&self) -> Option<&str> {
        self.index.as_ref().map(|x| &**x)
    }
}
