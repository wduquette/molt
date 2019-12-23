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
//! # Interp is not Sync!
//!
//! The `Interp` class (and the rest of Molt) is intended for use in a single thread.  It is
//! safe to have multiple `Interps` in different threads; but use `String` (or another `Sync`)
//! when passing data between them.  In particular, `Value` is not `Sync`.
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
use crate::expr;
use crate::molt_err;
use crate::molt_ok;
use crate::parser;
use crate::parser::Script;
use crate::parser::Word;
use crate::scope::ScopeStack;
use crate::types::*;
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
pub struct Interp {
    // Command Table
    commands: HashMap<String, Rc<Command>>,

    // Variable Table
    scopes: ScopeStack,

    // Context ID Counter
    last_context_id: u64,

    // Context Map
    // TODO: Remove: context_map: HashMap<ContextID, Box<dyn Any>>,
    context_map: HashMap<ContextID, ContextBox>,

    // Defines the recursion limit for Interp::eval().
    recursion_limit: usize,

    // Current number of eval levels.
    num_levels: usize,

    // Profile Map
    profile_map: HashMap<String, ProfileRecord>,
}

/// A command defined in the interpreter.
enum Command {
    /// A binary command implemented as a Rust CommandFunc.
    Func(CommandFunc, ContextID),

    /// A Molt procedure
    Proc(Procedure),
}

impl Command {
    // Execute the command according to its kind.
    fn execute(&self, interp: &mut Interp, argv: &[Value]) -> MoltResult {
        match self {
            Command::Func(func, context_id) => {
                func(interp, *context_id, argv)
            }
            Command::Proc(proc) => {
                proc.execute(interp, argv)
            }
        }
    }
}

/// Sentinal value for command functions with no related context.
///
/// **NOTE**: it would make no sense to use `Option<ContextID>` instead of a sentinal
/// value.  Whether or not a command has related context is known at compile
/// time, and is an essential part of the command's definition; it never changes.
/// Commands with context will access the function's context_id argument; and
/// and commands without have no reason to do so.  Using a sentinel allows the same
/// function type to be used for all binary Molt commands with minimal hassle to the
/// client developer.
const NULL_CONTEXT: ContextID = ContextID(0);

/// A container for a command's context struct, containing the context in a box,
/// and a reference count.
///
/// The reference count is incremented when the context's ID is used with a command,
/// and decremented when the command is forgotten.  If the reference count is
/// decremented to zero, the context is removed.
struct ContextBox {
    data: Box<dyn Any>,
    ref_count: usize,
}

impl ContextBox {
    /// Creates a new context box for the given data, and sets its reference count to 0.
    fn new<T: 'static>(data: T) -> Self {
        Self {
            data: Box::new(data),
            ref_count: 0,
        }
    }

    /// Increments the context's reference count.
    fn increment(&mut self) {
        self.ref_count += 1;
    }

    /// Decrements the context's reference count.  Returns true if the count is now 0,
    /// and false otherwise.
    ///
    /// Panics if the count is already 0.
    #[allow(dead_code)] // TODO: Remove once the design is complete.
    fn decrement(&mut self) -> bool {
        assert!(self.ref_count != 0, "attempted to decrement context ref count below zero");
        self.ref_count -= 1;
        self.ref_count == 0
    }
}

struct ProfileRecord {
    count: u128,
    nanos: u128,
}

impl ProfileRecord {
    fn new() -> Self {
        Self { count: 0, nanos: 0 }
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
        interp.add_command("array", commands::cmd_array);
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
        interp.add_command("parse", parser::cmd_parse);
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
        let value = Value::from(script);
        self.eval_value(&value)
    }

    /// Evaluates a script one command at a time.  Returns the [`Value`](../value/struct.Value.html)
    /// of the last command in the script, or the value of any explicit `return` call in the
    /// script, or any error thrown by the script.  Other
    /// [`ResultCode`](../types/enum.ResultCode.html) values are converted to normal errors.
    ///
    /// Use this method (or [`eval`](#method.eval) to evaluate arbitrary scripts.
    /// Use [`eval_body`](#method.eval_body) to evaluate the body of control structures.
    ///
    pub fn eval_value(&mut self, value: &Value) -> MoltResult {
        // TODO: Could probably do better, here.  If the value is already a list, for
        // example, can maybe evaluate it as a command without using as_script().
        // Tricky, though.  Don't want to have to parse it as a list.  Need a quick way
        // to determine if something is already a list.

        // FIRST, check the number of nesting levels
        self.num_levels += 1;

        if self.num_levels > self.recursion_limit {
            self.num_levels -= 1;
            return molt_err!("too many nested calls to Interp::eval (infinite loop?)");
        }

        // NEXT, evaluate the script and translate the result to Ok or Error
        let result = self.eval_body(value);

        // NEXT, decrement the number of nesting levels.
        self.num_levels -= 1;

        // NEXT, translate and return the result.
        match result {
            Err(ResultCode::Return(val)) => Ok(val),
            Err(ResultCode::Break) => molt_err!("invoked \"break\" outside of a loop"),
            Err(ResultCode::Continue) => molt_err!("invoked \"continue\" outside of a loop"),
            _ => result,
        }
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
    pub fn eval_body(&mut self, body: &Value) -> MoltResult {
        self.eval_script(&*body.as_script()?)
    }

    /// Evaluates a parsed Script, producing a normal MoltResult.
    /// Also used by expr.rs.
    pub(crate) fn eval_script(&mut self, script: &Script) -> MoltResult {
        let mut result_value = Value::empty();

        for word_vec in script.commands() {
            let words = self.eval_word_vec(word_vec.words())?;

            if words.is_empty() {
                break;
            }

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

    /// Evaluates a WordVec, producing a list of Values.  The expansion operator is handled
    /// as a special case.
    fn eval_word_vec(&mut self, words: &[Word]) -> Result<MoltList, ResultCode> {
        let mut list: MoltList = Vec::new();

        for word in words {
            if let Word::Expand(word_to_expand) = word {
                let value = self.eval_word(word_to_expand)?;
                for val in &*value.as_list()? {
                    list.push(val.clone());
                }
            } else {
                list.push(self.eval_word(word)?);
            }
        }

        Ok(list)
    }

    /// Evaluates a single word, producing a value.  This is also used by expr.rs.
    pub(crate) fn eval_word(&mut self, word: &Word) -> MoltResult {
        match word {
            Word::Value(val) => Ok(val.clone()),
            Word::VarRef(name) => self.scalar(name),
            Word::ArrayRef(name, index_word) => {
                let index = self.eval_word(index_word)?;
                self.element(name, index.as_str())
            }
            Word::Script(script) => self.eval_script(script),
            Word::Tokens(tokens) => {
                let tlist = self.eval_word_vec(tokens)?;
                let string: String = tlist.iter().map(|i| i.as_str()).collect();
                Ok(Value::from(string))
            }
            Word::Expand(_) => panic!("recursive Expand!"),
            Word::String(str) => Ok(Value::from(str)),
        }
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
        parser::parse(script).is_ok()
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

    /// Adds a binary command with no related context to the interpreter.  This is the normal
    /// way to add most commands.
    ///
    /// # Accessing Application Data
    ///
    /// When embedding Molt in an application, it is common to define commands that require
    /// mutable or immutable access to application data.  If the command requires
    /// access to data other than that provided by the `Interp` itself,
    /// add the relevant data structure to the context cache, receiving a `ContextID`, and
    /// then use [`add_context_command`](#method.add_context_command).
    pub fn add_command(&mut self, name: &str, func: CommandFunc) {
        self.add_context_command(name, func, NULL_CONTEXT);
    }

    /// Adds a binary command with related context data to the interpreter.
    ///
    /// This is the normal way to add commands requiring application context to
    /// the interpreter.  The context data will be forgotten when the last command to
    /// reference it is discarded.
    pub fn add_context_command(
        &mut self,
        name: &str,
        func: CommandFunc,
        context_id: ContextID,
    ) {
        // TODO: Issue: currently, no way to decrement it when the command is removed!
        if context_id != NULL_CONTEXT {
            self.context_map.get_mut(&context_id).expect("unknown context ID").increment();
        }

        self.commands.insert(name.into(), Rc::new(Command::Func(func, context_id)));
    }

    /// Adds a procedure to the interpreter.
    ///
    /// This is how to add a Molt `proc` to the interpreter.  The arguments are the same
    /// as for the `proc` command and the `commands::cmd_proc` function.
    pub(crate) fn add_proc(&mut self, name: &str, args: &[Value], body: &str) {
        let proc = Procedure {
            args: args.to_owned(),
            body: body.to_string(),
        };

        self.commands.insert(name.into(), Rc::new(Command::Proc(proc)));
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
        self.context_map.insert(id, ContextBox::new(data));
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
            .data
            .downcast_mut::<T>()
            .expect("context type mismatch")
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
        self.context_map.insert(id, ContextBox::new(data));
    }

    //--------------------------------------------------------------------------------------------
    // Variable Handling

    /// Retrieves the value of the named scalar variable in the current scope.
    ///
    /// Returns an error if the variable is not found, or if the variable is an array variable.
    pub fn scalar(&self, name: &str) -> MoltResult {
        self.scopes.get(name)
    }

    /// Retrieves the value of the named array element in the current scope.
    ///
    /// Returns an error if the element is not found, or the variable is not an
    /// array variable.
    pub fn element(&self, name: &str, index: &str) -> MoltResult {
        self.scopes.get_elem(name, index)
    }

    /// Retrieves the value of the variable in the current scope.
    ///
    /// Returns an error if the variable is a scalar and the name names an array element,
    /// and vice versa.
    pub fn var(&self, var_name: &Value) -> MoltResult {
        let var_name = &*var_name.as_var_name();
        match var_name.index() {
            Some(index) => self.element(var_name.name(), index),
            None => self.scalar(var_name.name()),
        }
    }

    /// Sets the value of the named scalar variable in the current scope, creating the variable
    /// if necessary.
    ///
    /// Returns an error if the variable exists and is an array variable.
    pub fn set_scalar(&mut self, name: &str, value: Value) -> Result<(), ResultCode> {
        self.scopes.set(name, value)
    }

    /// Sets the value of the named scalar variable in the current scope, creating the variable
    /// if necessary, and returning the value.
    ///
    /// Returns an error if the variable exists and is an array variable.
    pub fn set_scalar_return(&mut self, name: &str, value: Value) -> MoltResult {
        // Clone the value, since we'll be returning it out again.
        self.scopes.set(name, value.clone())?;
        Ok(value)
    }

    /// Sets the value of an array element in the current scope, creating the variable
    /// if necessary.
    ///
    /// Returns an error if the variable exists and is not an array variable.
    ///
    /// TODO: test needed
    pub fn set_element(&mut self, name: &str, index: &str, value: Value) -> Result<(), ResultCode> {
        self.scopes.set_elem(name, index, value)
    }

    /// Sets the value of an array element in the current scope, creating the variable
    /// if necessary, and returning the value.
    ///
    /// Returns an error if the variable exists and is not an array variable.
    ///
    /// TODO: test needed
    pub fn set_element_return(&mut self, name: &str, index: &str, value: Value) -> MoltResult {
        // Clone the value, since we'll be returning it out again.
        self.scopes.set_elem(name, index, value.clone())?;
        Ok(value)
    }

    /// Sets the value of the variable in the current scope, if any.
    ///
    /// Returns an error if the variable is scalar and the name names an array element,
    /// and vice-versa.
    ///
    /// TODO: test needed
    pub fn set_var(&mut self, var_name: &Value, value: Value) -> Result<(), ResultCode> {
        let var_name = &*var_name.as_var_name();
        match var_name.index() {
            Some(index) => self.set_element(var_name.name(), index, value),
            None => self.set_scalar(var_name.name(), value),
        }
    }

    /// Sets the value of the variable in the current scope, if any, and returns its value.
    ///
    /// Returns an error if the variable is scalar and the name names an array element,
    /// and vice-versa.
    ///
    /// TODO: test needed
    pub fn set_var_return(&mut self, var_name: &Value, value: Value) -> MoltResult {
        let var_name = &*var_name.as_var_name();
        match var_name.index() {
            Some(index) => self.set_element_return(var_name.name(), index, value),
            None => self.set_scalar_return(var_name.name(), value),
        }
    }

    /// Unsets the value of the named variable or array element in the current scope
    pub fn unset_var(&mut self, name: &Value) {
        let var_name = name.as_var_name();

        if let Some(index) = var_name.index() {
            self.unset_element(var_name.name(), index);
        } else {
            self.unset(var_name.name());
        }
    }

    /// Unsets a variable given its name.
    pub fn unset(&mut self, name: &str) {
        self.scopes.unset(name);
    }

    /// Unsets a single element in an array.  Nothing happens if the index doesn't
    /// exist, or if the variable is not an array variable.
    pub fn unset_element(&mut self, array_name: &str, index: &str) {
        self.scopes.unset_element(array_name, index);
    }

    /// Unsets an array variable givne its name.  Nothing happens if the variable doesn't
    /// exist, or if the variable is not an array variable.
    pub fn array_unset(&mut self, array_name: &str) {
        self.scopes.array_unset(array_name);
    }

    /// Gets a vector of the visible var names.
    pub fn vars_in_scope(&self) -> MoltList {
        self.scopes.vars_in_scope()
    }

    /// Determines whether or not the name is the name of an array variable.
    pub fn array_exists(&self, array_name: &str) -> bool {
        self.scopes.array_exists(array_name)
    }

    /// Gets a flat vector of the keys and values from the given array
    pub fn array_get(&self, array_name: &str) -> MoltList {
        self.scopes.array_get(array_name)
    }

    /// Merges a flat vector of keys and values into the given array
    /// It's an error if the vector has an odd number of elements.
    pub fn array_set(&mut self, array_name: &str, kvlist: &[Value]) -> MoltResult {
        if kvlist.len() % 2 == 0 {
            self.scopes.array_set(array_name, kvlist)?;
            molt_ok!()
        } else {
            molt_err!("list must have an even number of elements")
        }
    }

    /// Gets a vector of the indices of the given array
    pub fn array_names(&self, array_name: &str) -> MoltList {
        self.scopes.array_indices(array_name)
    }

    /// Gets a vector of the indices of the given array
    pub fn array_size(&self, array_name: &str) -> usize {
        self.scopes.array_size(array_name)
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
    /// Note: the level is the absolute level, not the level relative to the
    /// current stack level, i.e., level=0 is the global scope.
    pub fn upvar(&mut self, level: usize, name: &str) {
        assert!(level <= self.scopes.current(), "Invalid scope level");
        self.scopes.upvar(level, name);
    }

    //--------------------------------------------------------------------------------------------
    // Profiling

    pub fn profile_save(&mut self, name: &str, start: std::time::Instant) {
        let dur = Instant::now().duration_since(start).as_nanos();
        let rec = self
            .profile_map
            .entry(name.into())
            .or_insert_with(ProfileRecord::new);

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

// Procedure Definition: much to do here!
struct Procedure {
    args: MoltList,
    body: String,
}

// TODO: Need to work out how we're going to store the Procedure details for
// best efficiency.
impl Procedure {
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
                interp.set_scalar("args", Value::from(&argv[argi..]))?;

                // We've processed all of the args
                argi = argv.len();
                break;
            }

            // NEXT, do we have a matching argument?
            if argi < argv.len() {
                // Pair them up
                interp.set_scalar(vec[0].as_str(), argv[argi].clone())?;
                argi += 1;
                continue;
            }

            // NEXT, do we have a default value?
            if vec.len() == 2 {
                interp.set_scalar(vec[0].as_str(), vec[1].clone())?;
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
            interp.eval_body(&Value::from("set a 1; set b 2")),
            Ok(Value::from("2"))
        );
        assert_eq!(
            interp.eval_body(&Value::from("error 2; set a whoops")),
            Err(ResultCode::Error(Value::from("2")))
        );
        assert_eq!(
            interp.eval_body(&Value::from("return 3; set a whoops")),
            Err(ResultCode::Return(Value::from("3")))
        );
        assert_eq!(
            interp.eval_body(&Value::from("break; set a whoops")),
            Err(ResultCode::Break)
        );
        assert_eq!(
            interp.eval_body(&Value::from("continue; set a whoops")),
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
}
