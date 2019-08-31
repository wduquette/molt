//! The Molt Interpreter
//!
//! TODO: This should be primary documentation on using the Molt Interp.
//!
//! The [`Interp`] struct is the primary API for embedding Molt into a Rust application.
//! Given an `Interp`, the application may:
//!
//! * Evaluate scripts
//! * Check scripts for completeness
//! * Extend the language by defining new Molt commands in Rust
//! * Set and get Molt variables in command bodies
//! * Interpret Molt values as a variety of data types.
//! * Access application data the context cache
//!
//! The following describes the features of the [`Interp`] in general; follow the links for
//! specifics of the various types and methods.
//!
//! # Creating an Interpreter
//!
//! There are two ways to create an interpreter.  The usual way is to call
//! [`Interp::new`](struct.Interp.html#method.new), which creates an interpreter and populates
//! it with all of the standard Molt commands.  Alternatively,
//! [`Interp::empty`](struct.Interp.html#method.empty) creates an interpreter with no commands,
//! allowing the application to define only those commands it needs.  This is useful when the goal
//! is to provide the application with a simple, non-scriptable console command set.
//!
//! TODO: Define a way to add various subsets of the standard commands to an initially
//! empty interpreter.
//!
//! ```
//! use molt::Interp;
//! let mut interp = Interp::new();
//! // ...
//! ```
//!
//! # Evaluating Scripts
//!
//! There are a number of ways to evaluate Molt scripts, all of which return [`MoltResult`]:
//!
//! ```ignore
//! pub type MoltResult = Result<Value, ResultCode>;
//! ```
//!
//! [`Value`] is the type of all Molt values (i.e., values that can be passed as parameters and
//! stored in variables).  [`ResultCode`] is an enum that encompasses all of the kinds of
//! exceptional return from Molt code, including errors, `return`, `break`, and `continue`.
//!
//! [`Interp::eval`](struct.Interp.html#method.eval) and
//! [`Interp::eval_value`](struct.Interp.html#method.eval_value) evaluate a string as a Molt
//! script, and return either a normal `Value` or a Molt error.  The script is evaluated in
//! the caller's context: if called at the application level, the script will be evaluated in
//! the interpreter's global scope; if called by a Molt command, it will be evaluated in the
//! scope in which that command is executing.
//!
//! [`Interp::eval_body`](struct.Interp.html#method.eval_body) is used to evaluate the body
//! of loops and other control structures.  Unlike `Interp::eval`, it passes the
//! `return`, `break`, and `continue` result codes back to the caller for handling.
//!
//! # Evaluating Expressions
//!
//! In Molt, as in Standard Tcl, algebraic expressions are evaluated by the `expr` command.  At
//! the Rust level this feature is provided by the
//! [`Interp::expr`](struct.Interp.html#method.expr) method, which takes the expression as a
//! [`Value`] and returns the computed `Value` or an error.
//!
//! There are three convenience methods,
//! [`Interp::expr_bool`](struct.Interp.html#method.expr_bool),
//! [`Interp::expr_int`](struct.Interp.html#method.expr_int), and
//! [`Interp::expr_float`](struct.Interp.html#method.expr_float), which streamline the computation
//! of a particular kind of value, and return an error if the computed result is not of that type.
//!
//! # Checking Scripts for Completeness
//!
//! The [`Interp::complete`](struct.Interp.html#method.complete) checks whether a Molt script is
//! complete: i.e., that it contains no unterminated quoted or braced strings that
//! would prevent it from being evaluated as Molt code.  This is primarily useful when
//! implementing a Read-Eval-Print-Loop, as it allows the REPL to easily determine whether it
//! should evaluate the input immediately or ask for an additional line of input.
//!
//! # Defining New Commands
//!
//! The usual reason for embedding Molt in an application is to extend it with
//! application-specific commands.  There are a number of ways to do this.
//!
//! The simplest method, and the one used by most of Molt's built-in commands, is to define a
//! [`CommandFunc`] and register it with the interpreter using the
//! [`Interp::add_command`](struct.Interp.html#method.add_command) method:
//!
//! ```pub type CommandFunc = fn(&mut Interp, &[Value]) -> MoltResult;```
//!
//! A `CommandFunc` is simply a Rust function that accepts an interpreter and a slice of Molt
//! [`Value`] objects and returns a [`MoltResult`].  The slice of [`Value`] objects represents
//! the name of the command and its arguments, which the function may interpret in any way it
//! desires.
//!
//! TODO: describe context commands and command objects.
//!
//! TODO: flesh out Molt's ensemble command API, and then describe how to define ensemble commands.
//!
//! [`MoltResult`]: ../types/struct.MoltResult.html
//! [`ResultCode`]: ../types/struct.ResultCode.html
//! [`CommandFunc`]: ../types/struct.CommandFunc.html
//! [`Value`]: ../Value/struct.Value.html
//! [`Interp`]: struct.Interp.html

use crate::commands;
use crate::eval_ptr::EvalPtr;
use crate::expr;
use crate::molt_err;
use crate::molt_ok;
use crate::scope::ScopeStack;
use crate::types::Command;
use crate::types::*;
use crate::util::is_varname_char;
use crate::value::Value;
use std::any::Any;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Instant;

/// The Molt Interpreter.
///
/// The `Interp` struct is the primary API for
/// embedding Molt into a Rust application.  The application creates an instance
/// of `Interp`, configures with it the required set of application-specific
/// and standard Molt commands, and then uses it to evaluate Molt scripts and
/// expressions.
///
/// # Example
///
/// By default, the `Interp` comes configured with the full set of standard
/// Molt commands.
///
/// ```
/// # use molt::types::*;
/// # use molt::Interp;
/// # use molt::molt_ok;
/// # fn dummy() -> MoltResult {
/// let mut interp = Interp::new();
/// let four = interp.eval("expr {2 + 2}")?;
/// assert_eq!(four, Value::from(4));
/// # molt_ok!()
/// # }
/// ```
#[derive(Default)]
#[allow(dead_code)] // TEMP
pub struct Interp {
    // Command Table
    commands: HashMap<String, Rc<dyn Command>>,

    // Variable Table
    scopes: ScopeStack,

    // Context ID Counter
    last_context_id: u64,

    // Context Map
    context_map: HashMap<ContextID, Box<dyn Any>>,

    // Defines the recursion limit for Interp::eval().
    recursion_limit: usize,

    // Current number of eval levels.
    num_levels: usize,

    // Profile Map
    profile_map: HashMap<String, ProfileRecord>,
}

struct ProfileRecord {
    count: u128,
    nanos: u128,
}

impl ProfileRecord {
    fn new() -> Self {
        Self {
            count: 0,
            nanos: 0,
        }
    }
}

// NOTE: The order of methods in the generated RustDoc depends on the order in this block.
// Consequently, methods are ordered pedagogically.
impl Interp {
    //--------------------------------------------------------------------------------------------
    // Constructors

    /// Creates a new Molt interpreter with no commands defined.  Use this when crafting
    /// command languages that shouldn't include the normal TCL commands, or as a base
    /// to which specific Molt command sets can be added.
    ///
    /// # Example
    ///
    /// ```
    /// # use molt::interp::Interp;
    /// let mut interp = Interp::empty();
    /// assert!(interp.command_names().is_empty());
    /// ```

    pub fn empty() -> Self {
        Self {
            recursion_limit: 1000,
            commands: HashMap::new(),
            last_context_id: 0,
            context_map: HashMap::new(),
            scopes: ScopeStack::new(),
            num_levels: 0,
            profile_map: HashMap::new(),
        }
    }

    /// Creates a new Molt interpreter, pre-populated with the standard Molt commands.
    /// Use [`command_names`](#method.command_names) (or the `info commands` Molt command)
    /// to retrieve the full list, and the [`add_command`](#method.add_command) family of
    /// methods to extend the interpreter with new commands.
    ///
    /// TODO: Define command sets (sets of commands that go together, so that clients can
    /// add or remove them in groups).
    ///
    /// ```
    /// # use molt::types::*;
    /// # use molt::Interp;
    /// # use molt::molt_ok;
    /// # fn dummy() -> MoltResult {
    /// let mut interp = Interp::new();
    /// let four = interp.eval("expr {2 + 2}")?;
    /// assert_eq!(four, Value::from(4));
    /// # molt_ok!()
    /// # }
    /// ```
    ///
    pub fn new() -> Self {
        let mut interp = Interp::empty();

        // TODO: These commands affect the interpreter only, not the external environment.
        // It might be desirable to subdivide them further, into those that can cause
        // denial-of-service kinds of problems, e.g., for, while, proc, rename, and those
        // that can't.
        interp.add_command("append", commands::cmd_append);
        interp.add_command("assert_eq", commands::cmd_assert_eq);
        interp.add_command("break", commands::cmd_break);
        interp.add_command("catch", commands::cmd_catch);
        interp.add_command("continue", commands::cmd_continue);
        interp.add_command("error", commands::cmd_error);
        interp.add_command("expr", commands::cmd_expr);
        interp.add_command("for", commands::cmd_for);
        interp.add_command("foreach", commands::cmd_foreach);
        interp.add_command("global", commands::cmd_global);
        interp.add_command("if", commands::cmd_if);
        interp.add_command("incr", commands::cmd_incr);
        interp.add_command("info", commands::cmd_info);
        interp.add_command("join", commands::cmd_join);
        interp.add_command("lappend", commands::cmd_lappend);
        interp.add_command("lindex", commands::cmd_lindex);
        interp.add_command("list", commands::cmd_list);
        interp.add_command("llength", commands::cmd_llength);
        interp.add_command("proc", commands::cmd_proc);
        interp.add_command("puts", commands::cmd_puts);
        interp.add_command("rename", commands::cmd_rename);
        interp.add_command("return", commands::cmd_return);
        interp.add_command("set", commands::cmd_set);
        interp.add_command("time", commands::cmd_time);
        interp.add_command("unset", commands::cmd_unset);
        interp.add_command("while", commands::cmd_while);

        // TODO: Requires file access.  Ultimately, might go in an extension crate if
        // the necessary operations aren't available in core::.
        interp.add_command("source", commands::cmd_source);

        // TODO: Useful for entire programs written in Molt; but not necessarily wanted in
        // extension scripts.
        interp.add_command("exit", commands::cmd_exit);

        // TODO: Developer Tools
        interp.add_command("pdump", commands::cmd_pdump);
        interp.add_command("pclear", commands::cmd_pclear);

        interp
    }

    //--------------------------------------------------------------------------------------------
    // Script and Expression Evaluation

    /// Evaluates a script one command at a time.  Returns the [`Value`](../value/struct.Value.html)
    /// of the last command in the script, or the value of any explicit `return` call in the
    /// script, or any error thrown by the script.  Other
    /// [`ResultCode`](../types/enum.ResultCode.html) values are converted to normal errors.
    ///
    /// Use this method (or [`eval_value`](#method.eval_value) to evaluate arbitrary scripts.
    /// Use [`eval_body`](#method.eval_body) to evaluate the body of control structures.
    ///
    /// # Example
    ///
    /// The following code shows how to evaluate a script and handle the result, whether
    /// it's a computed `Value` or an error message (which is also a `Value`).
    ///
    /// ```
    /// # use molt::types::*;
    /// # use molt::Interp;
    /// let mut interp = Interp::new();
    /// let input = "set a 1";
    /// match interp.eval(input) {
    ///    Ok(val) => {
    ///        // Computed a Value
    ///        println!("Value: {}", val);
    ///    }
    ///    Err(ResultCode::Error(msg)) => {
    ///        // Got an error; print it out.
    ///        println!("Error: {}", msg);
    ///    }
    ///    _ => {
    ///        // Won't ever happen, but the compiler doesn't know that.
    ///        // panic!() if you like.
    ///    }
    /// }
    /// ```
    pub fn eval(&mut self, script: &str) -> MoltResult {
        // FIRST, check the number of nesting levels
        self.num_levels += 1;

        if self.num_levels > self.recursion_limit {
            self.num_levels -= 1;
            return molt_err!("too many nested calls to Interp::eval (infinite loop?)");
        }

        // NEXT, evaluate the script and translate the result to Ok or Error
        let mut ctx = EvalPtr::new(script);

        let result = self.eval_context(&mut ctx);

        // NEXT, decrement the number of nesting levels.
        self.num_levels -= 1;

        // NEXT, translate and return the result.
        match result {
            Err(ResultCode::Return(value)) => Ok(value),
            Err(ResultCode::Break) => molt_err!("invoked \"break\" outside of a loop"),
            Err(ResultCode::Continue) => molt_err!("invoked \"continue\" outside of a loop"),
            _ => result,
        }
    }

    /// Evaluates a script one command at a time, where the script is passed as a
    /// `Value`.  Except for the signature, this command is semantically equivalent to
    /// [`eval`](#method.eval).
    pub fn eval_value(&mut self, script: &Value) -> MoltResult {
        // TODO: Could probably do better, here.  If the value is already a list, for
        // example, can maybe evaluate it as a command without parsing the string.
        // Tricky, though.  Don't want to have to parse it as a list.  Need a quick way
        // to determine if something is already a list.  What does Tcl 8 do?
        self.eval(script.as_str())
    }

    /// Evaluates a script one command at a time, returning whatever
    /// [`MoltResult`](../types/type.MoltResult.html) arises.
    ///
    /// This is the method to use when evaluating a control structure's
    /// script body; the control structure must handle the special
    /// result codes appropriately.
    ///
    /// # Example
    ///
    /// The following code could be used to process the body of one of the Molt looping
    /// commands, e.g., `while` or
    /// `foreach`.  [`ResultCode`](../types/enum.ResultCode.html)`::Return` and `ResultCode::Error`
    /// return out of the looping command altogether, returning control to the caller.
    /// `ResultCode::Break` breaks out of the loop.  `Ok` and `ResultCode::Continue`
    /// continue with the next iteration.
    ///
    /// ```ignore
    /// ...
    /// while (...) {
    ///     let result = interp.eval_body(&body);
    ///
    ///     match result {
    ///         Ok(_) => (),
    ///         Err(ResultCode::Return(_)) => return result,
    ///         Err(ResultCode::Error(_)) => return result,
    ///         Err(ResultCode::Break) => break,
    ///         Err(ResultCode::Continue) => (),
    ///     }
    /// }
    ///
    /// molt_ok!()
    /// ```
    pub fn eval_body(&mut self, script: &Value) -> MoltResult {
        let mut ctx = EvalPtr::new(script.as_str());

        self.eval_context(&mut ctx)
    }

    /// Determines whether or not the script is syntactically complete,
    /// e.g., has no unmatched quotes, brackets, or braces.
    ///
    /// REPLs use this to determine whether or not to ask for another line of
    /// input.
    ///
    /// # Example
    ///
    /// ```
    /// # use molt::types::*;
    /// # use molt::interp::Interp;
    /// let mut interp = Interp::new();
    /// assert!(interp.complete("set a [expr {1+1}]"));
    /// assert!(!interp.complete("set a [expr {1+1"));
    /// ```

    pub fn complete(&mut self, script: &str) -> bool {
        let mut ctx = EvalPtr::new(script);
        ctx.set_no_eval(true);

        self.eval_context(&mut ctx).is_ok()
    }

    /// Evaluates a [Molt expression](https://wduquette.github.io/molt/ref/expr.html) and
    /// returns its value.  The expression is passed a `Value` which is interpreted as a `String`.
    ///
    /// # Example
    /// ```
    /// use molt::Interp;
    /// use molt::types::*;
    /// # fn dummy() -> Result<String,ResultCode> {
    /// let mut interp = Interp::new();
    /// let expr = Value::from("2 + 2");
    /// let sum = interp.expr(&expr)?.as_int()?;
    ///
    /// assert_eq!(sum, 4);
    /// # Ok("dummy".to_string())
    /// # }
    /// ```
    pub fn expr(&mut self, expr: &Value) -> MoltResult {
        expr::expr(self, expr)
    }

    /// Evaluates a boolean [Molt expression](https://wduquette.github.io/molt/ref/expr.html)
    /// and returns its value, or an error if it couldn't be interpreted as a boolean.
    ///
    /// # Example
    ///
    /// ```
    /// use molt::Interp;
    /// use molt::types::*;
    /// # fn dummy() -> Result<String,ResultCode> {
    /// let mut interp = Interp::new();
    ///
    /// let expr = Value::from("1 < 2");
    /// let flag: bool = interp.expr_bool(&expr)?;
    ///
    /// assert!(flag);
    /// # Ok("dummy".to_string())
    /// # }
    /// ```
    pub fn expr_bool(&mut self, expr: &Value) -> Result<bool, ResultCode> {
        expr::expr(self, expr)?.as_bool()
    }

    /// Evaluates a [Molt expression](https://wduquette.github.io/molt/ref/expr.html)
    /// and returns its value as an integer, or an error if it couldn't be interpreted as an
    /// integer.
    ///
    /// # Example
    ///
    /// ```
    /// use molt::Interp;
    /// use molt::types::*;
    /// # fn dummy() -> Result<String,ResultCode> {
    /// let mut interp = Interp::new();
    ///
    /// let expr = Value::from("1 + 2");
    /// let val: MoltInt = interp.expr_int(&expr)?;
    ///
    /// assert_eq!(val, 3);
    /// # Ok("dummy".to_string())
    /// # }
    /// ```
    pub fn expr_int(&mut self, expr: &Value) -> Result<MoltInt, ResultCode> {
        expr::expr(self, expr)?.as_int()
    }

    /// Evaluates a [Molt expression](https://wduquette.github.io/molt/ref/expr.html)
    /// and returns its value as a float, or an error if it couldn't be interpreted as a
    /// float.
    ///
    /// # Example
    ///
    /// ```
    /// use molt::Interp;
    /// use molt::types::*;
    /// # fn dummy() -> Result<String,ResultCode> {
    /// let mut interp = Interp::new();
    ///
    /// let expr = Value::from("1.1 + 2.2");
    /// let val: MoltFloat = interp.expr_float(&expr)?;
    ///
    /// assert_eq!(val, 3.3);
    /// # Ok("dummy".to_string())
    /// # }
    /// ```
    pub fn expr_float(&mut self, expr: &Value) -> Result<MoltFloat, ResultCode> {
        expr::expr(self, expr)?.as_float()
    }

    //--------------------------------------------------------------------------------------------
    // Command Definition and Handling

    /// Adds a command defined by a `CommandFunc` to the interpreter. This is the normal way to
    /// add commands to the interpreter.
    ///
    /// # Accessing Application Data
    ///
    /// When embedding Molt in an application, it is common to define commands that require
    /// mutable or immutable access to application data.  If the command requires
    /// access to data other than that provided by the `Interp` itself, e.g., application data,
    /// consider adding the relevant data structure to the context cache and then use
    /// [`add_context_command`](#method.add_context_command).  Alternatively, define a struct that
    /// implements `Command` and use [`add_command_object`](#method.add_command_object).
    pub fn add_command(&mut self, name: &str, func: CommandFunc) {
        let command = CommandFuncWrapper::new(func);
        self.add_command_object(name, command);
    }

    /// Adds a command defined by a `ContextCommandFunc` to the interpreter.
    ///
    /// This is the normal way to add commands requiring application context to
    /// the interpreter.  It is up to the module creating the context to free it when it is
    /// no longer required.
    ///
    /// **Warning**: Do not use this method to define a TCL object, i.e., a command with
    /// its own data and lifetime.  Use a type that implements `Command` and `Drop`.
    pub fn add_context_command(
        &mut self,
        name: &str,
        func: ContextCommandFunc,
        context_id: ContextID,
    ) {
        let command = ContextCommandFuncWrapper::new(func, context_id);
        self.add_command_object(name, command);
    }

    /// Adds a procedure to the interpreter.
    ///
    /// This is how to add a Molt `proc` to the interpreter.  The arguments are the same
    /// as for the `proc` command and the `commands::cmd_proc` function.
    pub(crate) fn add_proc(&mut self, name: &str, args: &[Value], body: &str) {
        let command = CommandProc {
            args: args.to_owned(),
            body: body.to_string(),
        };

        self.add_command_object(name, command);
    }

    /// Adds a command to the interpreter using a `Command` object.
    ///
    /// Use this when defining a command that requires application context.
    pub fn add_command_object<T: 'static + Command>(&mut self, name: &str, command: T) {
        self.commands.insert(name.into(), Rc::new(command));
    }

    /// Determines whether the interpreter contains a command with the given
    /// name.
    pub fn has_command(&self, name: &str) -> bool {
        self.commands.contains_key(name)
    }

    /// Renames the command.
    ///
    /// **Note:** This does not update procedures that reference the command under the old
    /// name.  This is intentional: it is a common TCL programming technique to wrap an
    /// existing command by renaming it and defining a new command with the old name that
    /// calls the original command at its new name.
    pub fn rename_command(&mut self, old_name: &str, new_name: &str) {
        if let Some(cmd) = self.commands.get(old_name) {
            let cmd = Rc::clone(cmd);
            self.commands.remove(old_name);
            self.commands.insert(new_name.into(), cmd);
        }
    }

    /// Removes the command with the given name.
    pub fn remove_command(&mut self, name: &str) {
        self.commands.remove(name);
    }

    /// Gets a vector of the names of the existing commands.
    ///
    pub fn command_names(&self) -> MoltList {
        let vec: MoltList = self
            .commands
            .keys()
            .cloned()
            .map(|x| Value::from(&x))
            .collect();

        vec
    }

    //--------------------------------------------------------------------------------------------
    // Interpreter Configuration

    /// Gets the interpreter's recursion limit.
    ///
    /// # Example
    /// ```
    /// # use molt::types::*;
    /// # use molt::interp::Interp;
    /// let mut interp = Interp::new();
    /// assert_eq!(interp.recursion_limit(), 1000);
    /// ```
    pub fn recursion_limit(&self) -> usize {
        self.recursion_limit
    }

    /// Sets the interpreter's recursion limit.  The default is 1000.
    ///
    /// # Example
    /// ```
    /// # use molt::types::*;
    /// # use molt::interp::Interp;
    /// let mut interp = Interp::new();
    /// interp.set_recursion_limit(100);
    /// assert_eq!(interp.recursion_limit(), 100);
    /// ```
    pub fn set_recursion_limit(&mut self, limit: usize) {
        self.recursion_limit = limit;
    }

    //--------------------------------------------------------------------------------------------
    // Context Cache

    /// Saves the client context data in the interpreter's context cache,
    /// returning a generated context ID.  Client commands can retrieve the data
    /// given the ID.
    ///
    ///
    /// # Example
    ///
    /// ```
    /// use molt::types::*;
    /// use molt::interp::Interp;
    ///
    /// let mut interp = Interp::new();
    /// let data: Vec<String> = Vec::new();
    /// let id = interp.save_context(data);
    /// ```
    pub fn save_context<T: 'static>(&mut self, data: T) -> ContextID {
        let id = self.context_id();
        self.context_map.insert(id, Box::new(data));
        id
    }

    /// Retrieves mutable client context given the context ID.
    ///
    /// # Example
    ///
    /// ```
    /// use molt::types::*;
    /// use molt::interp::Interp;
    ///
    /// let mut interp = Interp::new();
    /// let data: Vec<String> = Vec::new();
    /// let id = interp.save_context(data);
    ///
    /// // Later...
    /// let data: &mut Vec<String> = interp.context(id);
    /// data.push("New Value".into());
    ///
    /// // Or
    /// let data = interp.context::<Vec<String>>(id);
    /// data.push("New Value".into());
    /// ```
    ///
    /// # Panics
    ///
    /// This call panics if the context ID is unknown, or if the retrieved data
    /// has an unexpected type.
    pub fn context<T: 'static>(&mut self, id: ContextID) -> &mut T {
        self.context_map
            .get_mut(&id)
            .expect("unknown context ID")
            .downcast_mut::<T>()
            .expect("context type mismatch")
    }

    /// Removes a context record from the context cache.  Clears the data from
    /// the cache when it is no longer needed.
    ///
    /// # Example
    ///
    /// ```
    /// use molt::types::*;
    /// use molt::interp::Interp;
    ///
    /// let mut interp = Interp::new();
    /// let data: Vec<String> = Vec::new();
    /// let id = interp.save_context(data);
    ///
    /// // Later...
    /// interp.forget_context(id);
    /// ```
    ///
    pub fn forget_context(&mut self, id: ContextID) {
        self.context_map.remove(&id);
    }

    /// Generates a unique context ID for command context data.
    ///
    /// Normally the client will use [`save_context`](#method.save_context) to
    /// save the context data and generate the client ID in one operation, rather than
    /// call this explicitly.
    ////
    /// # Example
    ///
    /// ```
    /// use molt::types::*;
    /// use molt::interp::Interp;
    ///
    /// let mut interp = Interp::new();
    /// let id1 = interp.context_id();
    /// let id2 = interp.context_id();
    /// assert_ne!(id1, id2);
    /// ```
    pub fn context_id(&mut self) -> ContextID {
        // TODO: Practically speaking we won't overflow u64; but practically speaking
        // we should check any.
        self.last_context_id += 1;
        ContextID(self.last_context_id)
    }

    /// Saves a client context value in the interpreter for the given
    /// context ID.  Client commands can retrieve the data given the context ID.
    ///
    /// Normally the client will use [`save_context`](#method.save_context) to
    /// save the context data and generate the client ID in one operation, rather than
    /// call this explicitly.
    ///
    /// TODO: This method allows the user to generate a context ID and
    /// put data into the context cache as two separate steps; and to update the
    /// the data in the context cache for a given ID.  I'm not at all sure that
    /// either of those things is a good idea.  Waiting to see.
    ///
    /// # Example
    ///
    /// ```
    /// use molt::types::*;
    /// use molt::interp::Interp;
    ///
    /// let mut interp = Interp::new();
    /// let id = interp.context_id();
    /// let data: Vec<String> = Vec::new();
    /// interp.set_context(id, data);
    /// ```
    pub fn set_context<T: 'static>(&mut self, id: ContextID, data: T) {
        self.context_map.insert(id, Box::new(data));
    }

    //--------------------------------------------------------------------------------------------
    // Variable Handling

    /// Retrieves the value of the named variable in the current scope, if any.
    pub fn var(&self, name: &str) -> MoltResult {
        match self.scopes.get(name) {
            Some(v) => molt_ok!(v.clone()),
            None => molt_err!("can't read \"{}\": no such variable", name),
        }
    }

    /// Sets the value of the named variable in the current scope, creating the variable
    /// if necessary, and returning the value.
    pub fn set_and_return(&mut self, name: &str, value: Value) -> Value {
        self.scopes.set(name, value.clone());

        value
    }

    /// Sets the value of the named variable in the current scope, creating the variable
    /// if necessary.
    ///
    /// Ultimately, this should be set_var.
    pub fn set_var(&mut self, name: &str, value: &Value) {
        self.scopes.set(name, value.clone());
    }

    /// Unsets the value of the named variable in the current scope
    pub fn unset_var(&mut self, name: &str) {
        self.scopes.unset(name);
    }

    /// Gets a vector of the visible var names.
    pub fn vars_in_scope(&self) -> MoltList {
        self.scopes.vars_in_scope()
    }

    /// Pushes a variable scope on to the scope stack.
    /// Procs use this to define their local scope.
    pub fn push_scope(&mut self) {
        self.scopes.push();
    }

    /// Pops a variable scope off of the scope stack.
    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    /// Return the current scope level
    pub fn scope_level(&self) -> usize {
        self.scopes.current()
    }

    /// Links the variable name in the current scope to the given scope.
    pub fn upvar(&mut self, level: usize, name: &str) {
        assert!(level <= self.scopes.current(), "Invalid scope level");
        self.scopes.upvar(level, name);
    }

    //--------------------------------------------------------------------------------------------
    // The Molt Parser
    //
    // TODO: Can this be easily moved to another module?  It needs access to the
    // Interp struct's fields.

    /// Low-level script evaluator; evaluates the next script in the
    /// context.
    fn eval_context(&mut self, ctx: &mut EvalPtr) -> MoltResult {
        let mut result_value = Value::empty();

        while !ctx.at_end_of_script() {
            // let start = Instant::now();
            let words = self.parse_command(ctx)?;

            if words.is_empty() {
                break;
            }

            // self.profile_save(&format!("parse_command({})", words[0].as_str()), start);

            // When scanning for info
            if ctx.is_no_eval() {
                continue;
            }

            // FIRST, convert to Vec<&str>
            let name = words[0].as_str();
            if let Some(cmd) = self.commands.get(name) {
                // let start = Instant::now();
                let cmd = Rc::clone(cmd);
                let result = cmd.execute(self, words.as_slice());
                // self.profile_save(&format!("cmd.execute({})", name), start);
                match result {
                    Ok(v) => result_value = v,
                    _ => return result,
                }
            } else {
                return molt_err!("invalid command name \"{}\"", name);
            }
        }

        Ok(result_value)
    }

    fn parse_command(&mut self, ctx: &mut EvalPtr) -> Result<MoltList, ResultCode> {
        // FIRST, deal with whitespace and comments between "here" and the next command.
        while !ctx.at_end_of_script() {
            ctx.skip_block_white();

            // Either there's a comment, or we're at the beginning of the next command.
            // If the former, skip the comment; then check for more whitespace and comments.
            // Otherwise, go on to the command.
            if !ctx.skip_comment() {
                break;
            }
        }

        let mut words = Vec::new();

        // Read words until we get to the end of the line or hit an error
        // NOTE: parse_word() can always assume that it's at the beginning of a word.
        while !ctx.at_end_of_command() {
            // FIRST, get the next word; there has to be one, or there's an input error.
            let word = self.parse_word(ctx)?;

            // NEXT, save the word we found.
            words.push(word);

            // NEXT, skip any whitespace.
            ctx.skip_line_white();
        }

        // If we ended at a ";", consume the semi-colon.
        if ctx.next_is(';') {
            ctx.next();
        }

        Ok(words)
    }

    /// We're at the beginning of a word belonging to the current command.
    /// It's either a bare word, a braced string, or a quoted string--or there's
    /// an error in the input.  Whichever it is, get it.
    fn parse_word(&mut self, ctx: &mut EvalPtr) -> MoltResult {
        if ctx.next_is('{') {
            Ok(self.parse_braced_word(ctx)?)
        } else if ctx.next_is('"') {
            Ok(self.parse_quoted_word(ctx)?)
        } else {
            Ok(self.parse_bare_word(ctx)?)
        }
    }

    pub(crate) fn parse_braced_word(&mut self, ctx: &mut EvalPtr) -> MoltResult {
        // FIRST, skip the opening brace, and count it; non-escaped braces need to
        // balance.
        ctx.skip_char('{');
        let mut count = 1;

        // NEXT, add tokens to the word until we reach the close quote
        let mut word = String::new();
        let mut start = ctx.mark();

        while !ctx.at_end() {
            // Note: the while condition ensures that there's a character.
            if ctx.next_is('{') {
                count += 1;
                ctx.skip();
            } else if ctx.next_is('}') {
                count -= 1;

                if count > 0 {
                    ctx.skip();
                } else {
                    // We've found and consumed the closing brace.  We should either
                    // see more more whitespace, or we should be at the end of the list
                    // Otherwise, there are incorrect characters following the close-brace.
                    word.push_str(ctx.token(start));
                    let result = Ok(Value::from(word));
                    ctx.skip(); // Skip the closing brace

                    if ctx.at_end_of_command() || ctx.next_is_line_white() {
                        return result;
                    } else {
                        return molt_err!("extra characters after close-brace");
                    }
                }
            } else if ctx.next_is('\\') {
                word.push_str(ctx.token(start));
                ctx.skip();

                // If there's no character it's because we're at the end; and there's
                // no close brace.
                if let Some(ch) = ctx.next() {
                    if ch == '\n' {
                        word.push(' ');
                    } else {
                        word.push('\\');
                        word.push(ch);
                    }
                }
                start = ctx.mark();
            } else {
                ctx.skip();
            }
        }

        molt_err!("missing close-brace")
    }

    /// Parse a quoted word.
    pub(crate) fn parse_quoted_word(&mut self, ctx: &mut EvalPtr) -> MoltResult {
        // FIRST, consume the the opening quote.
        ctx.next();

        // NEXT, add tokens to the word until we reach the close quote
        let mut word = String::new();
        let mut start = ctx.mark();

        while !ctx.at_end() {
            // Note: the while condition ensures that there's a character.
            if ctx.next_is('[') {
                word.push_str(ctx.token(start));
                word.push_str(self.parse_script(ctx)?.as_str());
                start = ctx.mark();
            } else if ctx.next_is('$') {
                word.push_str(ctx.token(start));
                word.push_str(self.parse_variable(ctx)?.as_str());
                start = ctx.mark();
            } else if ctx.next_is('\\') {
                word.push_str(ctx.token(start));
                word.push(ctx.backslash_subst());
                start = ctx.mark();
            } else if ctx.next_is('"') {
                word.push_str(ctx.token(start));
                ctx.skip_char('"');
                if !ctx.at_end_of_command() && !ctx.next_is_line_white() {
                    return molt_err!("extra characters after close-quote");
                } else {
                    return Ok(Value::from(word));
                }
            } else {
                ctx.skip();
            }
        }

        molt_err!("missing \"")
    }

    /// Parse a bare word.
    fn parse_bare_word(&mut self, ctx: &mut EvalPtr) -> MoltResult {
        let mut word = String::new();
        let mut start = ctx.mark();

        while !ctx.at_end_of_command() && !ctx.next_is_line_white() {
            // Note: the while condition ensures that there's a character.
            if ctx.next_is('[') {
                word.push_str(ctx.token(start));
                word.push_str(self.parse_script(ctx)?.as_str());
                start = ctx.mark();
            } else if ctx.next_is('$') {
                word.push_str(ctx.token(start));
                word.push_str(self.parse_variable(ctx)?.as_str());
                start = ctx.mark();
            } else if ctx.next_is('\\') {
                word.push_str(ctx.token(start));
                word.push(ctx.backslash_subst());
                start = ctx.mark();
            } else {
                ctx.skip();
            }
        }

        word.push_str(ctx.token(start));

        Ok(Value::from(word))
    }

    pub(crate) fn parse_script(&mut self, ctx: &mut EvalPtr) -> MoltResult {
        // FIRST, skip the '['
        ctx.skip_char('[');

        // NEXT, parse the script up to the matching ']'
        let old_flag = ctx.is_bracket_term();
        ctx.set_bracket_term(true);
        let result = self.eval_context(ctx);
        ctx.set_bracket_term(old_flag);

        // NEXT, make sure there's a closing bracket
        if result.is_ok() {
            if ctx.next_is(']') {
                ctx.next();
            } else {
                return molt_err!("missing close-bracket");
            }
        }

        result
    }

    pub(crate) fn parse_variable(&mut self, ctx: &mut EvalPtr) -> MoltResult {
        // FIRST, skip the '$'
        ctx.skip_char('$');

        // NEXT, make sure this is really a variable reference.  If it isn't
        // just return a "$".
        if !ctx.next_is_varname_char() && !ctx.next_is('{') {
            return Ok(Value::from("$"));
        }

        // NEXT, is this a braced variable name?
        let var_value;

        if ctx.next_is('{') {
            ctx.skip_char('{');
            let start = ctx.mark();
            ctx.skip_while(|ch| *ch != '}');

            if ctx.at_end() {
                return molt_err!("missing close-brace for variable name");
            }

            var_value = self.var(ctx.token(start))?;
            ctx.skip_char('}');
        } else {
            let start = ctx.mark();
            ctx.skip_while(|ch| is_varname_char(*ch));
            var_value = self.var(ctx.token(start))?;
        }

        Ok(var_value)
    }

    //--------------------------------------------------------------------------------------------
    // Profiling

    pub fn profile_save(&mut self, name: &str, start: std::time::Instant) {
        let dur = Instant::now().duration_since(start).as_nanos();
        let rec = self.profile_map.entry(name.into()).or_insert_with(ProfileRecord::new);

        rec.count += 1;
        rec.nanos += dur;
    }

    pub fn profile_clear(&mut self) {
        self.profile_map.clear();
    }

    pub fn profile_dump(&self) {
        if self.profile_map.is_empty() {
            println!("no profile data");
        } else {
            for (name, rec) in &self.profile_map {
                let avg = rec.nanos / rec.count;
                println!("{} nanos {}, count={}", avg, name, rec.count);
            }
        }
    }
}

/// A struct that wraps a CommandFunc and implements the Command trait.
struct CommandFuncWrapper {
    func: CommandFunc,
}

impl CommandFuncWrapper {
    fn new(func: CommandFunc) -> Self {
        Self { func }
    }
}

impl Command for CommandFuncWrapper {
    fn execute(&self, interp: &mut Interp, argv: &[Value]) -> MoltResult {
        (self.func)(interp, argv)
    }
}

/// A struct that wraps a ContextCommandFunc and implements the Command trait.
struct ContextCommandFuncWrapper {
    func: ContextCommandFunc,
    context_id: ContextID,
}

impl ContextCommandFuncWrapper {
    fn new(func: ContextCommandFunc, context_id: ContextID) -> Self {
        Self { func, context_id }
    }
}

impl Command for ContextCommandFuncWrapper {
    fn execute(&self, interp: &mut Interp, argv: &[Value]) -> MoltResult {
        (self.func)(interp, self.context_id, argv)
    }
}

// EvalPtr structure for a proc.
struct CommandProc {
    args: MoltList,
    body: String,
}

// TODO: Need to work out how we're going to store the CommandProc details for
// best efficiency.
impl Command for CommandProc {
    fn execute(&self, interp: &mut Interp, argv: &[Value]) -> MoltResult {
        let name = argv[0].as_str();

        // FIRST, push the proc's local scope onto the stack.
        interp.push_scope();

        // NEXT, process the proc's argument list.
        let mut argi = 1; // Skip the proc's name

        for (speci, spec) in self.args.iter().enumerate() {
            // FIRST, get the parameter as a vector.  It should be a list of
            // one or two elements.
            let vec = &*spec.as_list()?; // Should never fail
            assert!(vec.len() == 1 || vec.len() == 2);

            // NEXT, if this is the args parameter, give the remaining args,
            // if any.  Note that "args" has special meaning only if it's the
            // final arg spec in the list.
            if vec[0].as_str() == "args" && speci == self.args.len() - 1 {
                interp.set_and_return("args", Value::from(&argv[argi..]));

                // We've processed all of the args
                argi = argv.len();
                break;
            }

            // NEXT, do we have a matching argument?
            if argi < argv.len() {
                // Pair them up
                interp.set_var(vec[0].as_str(), &argv[argi]);
                argi += 1;
                continue;
            }

            // NEXT, do we have a default value?
            if vec.len() == 2 {
                interp.set_var(vec[0].as_str(), &vec[1]);
            } else {
                // We don't; we're missing a required argument.
                return self.wrong_num_args(name);
            }
        }

        // NEXT, do we have any arguments left over?

        if argi != argv.len() {
            return self.wrong_num_args(name);
        }

        // NEXT, evaluate the proc's body, getting the result.
        let result = interp.eval(&self.body);

        // NEXT, pop the scope off of the stack; we're done with it.
        interp.pop_scope();

        // NEXT, return the computed result.
        // Note: no need for special handling for return, break, continue;
        // interp.eval() returns only Ok or a real error.
        result
    }
}

impl CommandProc {
    // Outputs the wrong # args message for the proc.  The name is passed in
    // because it can be changed via the `rename` command.
    fn wrong_num_args(&self, name: &str) -> MoltResult {
        let mut msg = String::new();
        msg.push_str("wrong # args: should be \"");
        msg.push_str(name);

        for (i, arg) in self.args.iter().enumerate() {
            msg.push(' ');

            // "args" has special meaning only in the last place.
            if arg.as_str() == "args" && i == self.args.len() - 1 {
                msg.push_str("?arg ...?");
                break;
            }

            let vec = arg.as_list().expect("error in proc arglist validation!");

            if vec.len() == 1 {
                msg.push_str(vec[0].as_str());
            } else {
                msg.push('?');
                msg.push_str(vec[0].as_str());
                msg.push('?');
            }
        }
        msg.push_str("\"");

        molt_err!(&msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let interp = Interp::empty();
        // Interpreter is empty
        assert!(interp.command_names().is_empty());
    }

    #[test]
    fn test_new() {
        let interp = Interp::new();

        // Interpreter is not empty
        assert!(!interp.command_names().is_empty());

        // Note: in theory, we should test here that the normal set of commands is present.
        // In fact, that should be tested by the `molt test` suite.
    }

    #[test]
    fn test_eval() {
        let mut interp = Interp::new();

        assert_eq!(interp.eval("set a 1"), Ok(Value::from("1")));
        assert_eq!(
            interp.eval("error 2"),
            Err(ResultCode::Error(Value::from("2")))
        );
        assert_eq!(interp.eval("return 3"), Ok(Value::from("3")));
        assert_eq!(
            interp.eval("break"),
            Err(ResultCode::Error(Value::from(
                "invoked \"break\" outside of a loop"
            )))
        );
        assert_eq!(
            interp.eval("continue"),
            Err(ResultCode::Error(Value::from(
                "invoked \"continue\" outside of a loop"
            )))
        );
    }

    #[test]
    fn test_eval_value() {
        let mut interp = Interp::new();

        assert_eq!(
            interp.eval_value(&Value::from("set a 1")),
            Ok(Value::from("1"))
        );
        assert_eq!(
            interp.eval_value(&Value::from("error 2")),
            Err(ResultCode::Error(Value::from("2")))
        );
        assert_eq!(
            interp.eval_value(&Value::from("return 3")),
            Ok(Value::from("3"))
        );
        assert_eq!(
            interp.eval_value(&Value::from("break")),
            Err(ResultCode::Error(Value::from(
                "invoked \"break\" outside of a loop"
            )))
        );
        assert_eq!(
            interp.eval_value(&Value::from("continue")),
            Err(ResultCode::Error(Value::from(
                "invoked \"continue\" outside of a loop"
            )))
        );
    }

    #[test]
    fn test_eval_body() {
        let mut interp = Interp::new();

        assert_eq!(
            interp.eval_body(&Value::from("set a 1")),
            Ok(Value::from("1"))
        );
        assert_eq!(
            interp.eval_body(&Value::from("error 2")),
            Err(ResultCode::Error(Value::from("2")))
        );
        assert_eq!(
            interp.eval_body(&Value::from("return 3")),
            Err(ResultCode::Return(Value::from("3")))
        );
        assert_eq!(
            interp.eval_body(&Value::from("break")),
            Err(ResultCode::Break)
        );
        assert_eq!(
            interp.eval_body(&Value::from("continue")),
            Err(ResultCode::Continue)
        );
    }

    #[test]
    fn test_complete() {
        let mut interp = Interp::new();

        assert!(interp.complete("abc"));
        assert!(interp.complete("a {bc} [def] \"ghi\" xyz"));

        assert!(!interp.complete("a {bc"));
        assert!(!interp.complete("a [bc"));
        assert!(!interp.complete("a \"bc"));
    }

    #[test]
    fn test_expr() {
        let mut interp = Interp::new();
        assert_eq!(interp.expr(&Value::from("1 + 2")), Ok(Value::from(3)));
        assert_eq!(
            interp.expr(&Value::from("a + b")),
            Err(ResultCode::Error(Value::from(
                "unknown math function \"a\""
            )))
        );
    }

    #[test]
    fn test_expr_bool() {
        let mut interp = Interp::new();
        assert_eq!(interp.expr_bool(&Value::from("1")), Ok(true));
        assert_eq!(interp.expr_bool(&Value::from("0")), Ok(false));
        assert_eq!(
            interp.expr_bool(&Value::from("a")),
            Err(ResultCode::Error(Value::from(
                "unknown math function \"a\""
            )))
        );
    }

    #[test]
    fn test_expr_int() {
        let mut interp = Interp::new();
        assert_eq!(interp.expr_int(&Value::from("1 + 2")), Ok(3));
        assert_eq!(
            interp.expr_int(&Value::from("a")),
            Err(ResultCode::Error(Value::from(
                "unknown math function \"a\""
            )))
        );
    }

    #[test]
    fn test_expr_float() {
        let mut interp = Interp::new();
        let val = interp
            .expr_float(&Value::from("1.1 + 2.2"))
            .expect("floating point value");

        assert!((val - 3.3).abs() < 0.001);

        assert_eq!(
            interp.expr_float(&Value::from("a")),
            Err(ResultCode::Error(Value::from(
                "unknown math function \"a\""
            )))
        );
    }

    #[test]
    fn test_parse_braced_word() {
        let mut interp = Interp::new();

        // Simple string
        assert_eq!(pbrace(&mut interp, "{abc}"), "abc|".to_string());

        // Simple string with following space
        assert_eq!(pbrace(&mut interp, "{abc} "), "abc| ".to_string());

        // String with white space
        assert_eq!(pbrace(&mut interp, "{a b c} "), "a b c| ".to_string());

        // String with $ and []space
        assert_eq!(pbrace(&mut interp, "{a $b [c]} "), "a $b [c]| ".to_string());

        // String with escaped braces
        assert_eq!(pbrace(&mut interp, "{a\\{bc} "), "a\\{bc| ".to_string());
        assert_eq!(pbrace(&mut interp, "{ab\\}c} "), "ab\\}c| ".to_string());

        // String with escaped newline (a real newline with a \ in front)
        assert_eq!(pbrace(&mut interp, "{ab\\\nc}"), "ab c|".to_string());
    }

    fn pbrace(interp: &mut Interp, input: &str) -> String {
        let mut ctx = EvalPtr::new(input);

        match interp.parse_braced_word(&mut ctx) {
            Ok(val) => format!("{}|{}", val.as_str(), ctx.tok().as_str()),
            Err(ResultCode::Error(value)) => format!("{}", value),
            Err(code) => format!("{:?}", code),
        }
    }

    #[test]
    fn test_parse_quoted_word() {
        let mut interp = Interp::new();

        // Simple string
        assert_eq!(pqw(&mut interp, "\"abc\""), "abc|".to_string());

        // Simple string with text following
        assert_eq!(pqw(&mut interp, "\"abc\"  "), "abc|  ".to_string());

        // Backslash substitution at beginning, middle, and end
        assert_eq!(pqw(&mut interp, "\"\\x77-\""), "w-|".to_string());
        assert_eq!(pqw(&mut interp, "\"a\\x77-\""), "aw-|".to_string());
        assert_eq!(pqw(&mut interp, "\"a\\x77\""), "aw|".to_string());

        // Variable substitution
        interp.set_var("x", &Value::from("5"));
        assert_eq!(pqw(&mut interp, "\"a$x.b\" "), "a5.b| ".to_string());

        interp.set_var("xyz1", &Value::from("10"));
        assert_eq!(pqw(&mut interp, "\"a$xyz1.b\" "), "a10.b| ".to_string());

        assert_eq!(pqw(&mut interp, "\"a$.b\" "), "a$.b| ".to_string());

        assert_eq!(pqw(&mut interp, "\"a${x}.b\" "), "a5.b| ".to_string());

        // Command substitution
        assert_eq!(pqw(&mut interp, "\"a[list b]c\""), "abc|".to_string());
        assert_eq!(pqw(&mut interp, "\"a[list b c]d\""), "ab cd|".to_string());

        // Extra characters after close-quote
        assert_eq!(pqw(&mut interp, "\"abc\"x  "), "extra characters after close-quote");
    }

    fn pqw(interp: &mut Interp, input: &str) -> String {
        let mut ctx = EvalPtr::new(input);

        match interp.parse_quoted_word(&mut ctx) {
            Ok(val) => format!("{}|{}", val.as_str(), ctx.tok().as_str()),
            Err(ResultCode::Error(value)) => format!("{}", value),
            Err(code) => format!("{:?}", code),
        }
    }

    #[test]
    fn test_parse_bare() {
        let mut interp = Interp::new();

        // Single word
        assert_eq!(pbare(&mut interp, "abc"), "abc|".to_string());

        // Single word with whitespace following
        assert_eq!(pbare(&mut interp, "abc "), "abc| ".to_string());
        assert_eq!(pbare(&mut interp, "abc\t"), "abc|\t".to_string());

        // Backslash substitution at beginning, middle, and end
        assert_eq!(pbare(&mut interp, "\\x77-"), "w-|".to_string());
        assert_eq!(pbare(&mut interp, "a\\x77-"), "aw-|".to_string());
        assert_eq!(pbare(&mut interp, "a\\x77"), "aw|".to_string());

        // Variable substitution
        interp.set_var("x", &Value::from("5"));
        assert_eq!(pbare(&mut interp, "a$x.b "), "a5.b| ".to_string());

        interp.set_var("xyz1", &Value::from("10"));
        assert_eq!(pbare(&mut interp, "a$xyz1.b "), "a10.b| ".to_string());

        assert_eq!(pbare(&mut interp, "a$.b "), "a$.b| ".to_string());

        assert_eq!(pbare(&mut interp, "a${x}.b "), "a5.b| ".to_string());

        // Command substitution
        assert_eq!(pbare(&mut interp, "a[list b]c"), "abc|".to_string());
        assert_eq!(pbare(&mut interp, "a[list b c]d"), "ab cd|".to_string());
    }

    fn pbare(interp: &mut Interp, input: &str) -> String {
        let mut ctx = EvalPtr::new(input);

        match interp.parse_bare_word(&mut ctx) {
            Ok(val) => format!("{}|{}", val.as_str(), ctx.tok().as_str()),
            Err(ResultCode::Error(value)) => format!("{}", value),
            Err(code) => format!("{:?}", code),
        }
    }

    #[test]
    fn test_parse_variable() {
        let mut interp = Interp::new();

        assert_eq!(pvar(&mut interp, "a", "$a"), "OK|".to_string());
        assert_eq!(pvar(&mut interp, "abc", "$abc"), "OK|".to_string());
        assert_eq!(pvar(&mut interp, "abc", "$abc."), "OK|.".to_string());
        assert_eq!(pvar(&mut interp, "a", "$a.bc"), "OK|.bc".to_string());
        assert_eq!(pvar(&mut interp, "a1_", "$a1_.bc"), "OK|.bc".to_string());
        assert_eq!(pvar(&mut interp, "a", "${a}b"), "OK|b".to_string());
        assert_eq!(pvar(&mut interp, "a", "$"), "$|".to_string());

        assert_eq!(pvar(&mut interp, "a", "$1"), "can't read \"1\": no such variable".to_string());

    }

    fn pvar(interp: &mut Interp, var: &str, input: &str) -> String {
        let mut ctx = EvalPtr::new(input);
        interp.set_var(var, &Value::from("OK"));

        match interp.parse_variable(&mut ctx) {
            Ok(val) => format!("{}|{}", val, ctx.tok().as_str()),
            Err(ResultCode::Error(value)) => format!("{}", value),
            Err(code) => format!("{:?}", code),
        }
    }

    #[test]
    fn test_recursion_limit() {
        let mut interp = Interp::new();

        assert_eq!(interp.recursion_limit(), 1000);
        interp.set_recursion_limit(100);
        assert_eq!(interp.recursion_limit(), 100);

        assert!(dbg!(interp.eval("proc myproc {} { myproc }")).is_ok());
        assert_eq!(
            interp.eval("myproc"),
            molt_err!("too many nested calls to Interp::eval (infinite loop?)")
        );
    }

    //-----------------------------------------------------------------------
    // Context Cache tests

    #[test]
    fn context_basic_use() {
        let mut interp = Interp::new();

        // Save a context object.
        let id = interp.save_context(String::from("ABC"));

        // Retrieve it.
        let ctx = interp.context::<String>(id);
        assert_eq!(*ctx, "ABC");
        ctx.push_str("DEF");

        let ctx = interp.context::<String>(id);
        assert_eq!(*ctx, "ABCDEF");
    }

    #[test]
    fn context_advanced_use() {
        let mut interp = Interp::new();

        // Save a context object.
        let id = interp.context_id();
        interp.set_context(id, String::from("ABC"));

        // Retrieve it.
        let ctx = interp.context::<String>(id);
        assert_eq!(*ctx, "ABC");
    }

    #[test]
    #[should_panic]
    fn context_unknown() {
        let mut interp = Interp::new();

        // Valid ID Generated, but no context saved.
        let id = interp.context_id();

        // Try to retrieve it.
        let _ctx = interp.context::<String>(id);

        // Should panic!
    }

    #[test]
    #[should_panic]
    fn context_wrong_type() {
        let mut interp = Interp::new();

        // Save a context object.
        let id = interp.save_context(String::from("ABC"));

        // Try to retrieve it as something else.
        let _ctx = interp.context::<Vec<String>>(id);

        // Should panic!
    }

    #[test]
    #[should_panic]
    fn context_forget() {
        let mut interp = Interp::new();

        // Save a context object.
        let id = interp.save_context(String::from("ABC"));

        // Retrieve it.
        let ctx = interp.context::<String>(id);
        assert_eq!(*ctx, "ABC");

        // Forget it
        interp.forget_context(id);

        // Retrieve it; should panic.
        let _ctx = interp.context::<String>(id);
    }
}
