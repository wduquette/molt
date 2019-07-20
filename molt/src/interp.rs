//! The Molt Interpreter
//!
//! The [`Interp`] struct is the primary API for embedding Molt into a Rust application.
//!
//! TODO: This should be primary documentation on using the Molt Interp.  Topics should
//! include:
//!
//! * Evaluating scripts
//! * Checking scripts for completeness
//! * Defining commands
//! * Using the context cache
//! * Setting and getting variables in command bodies
//!
//! [`Interp`]: struct.Interp.html

use crate::commands;
use crate::eval_ptr::EvalPtr;
use crate::expr;
use crate::molt_err;
use crate::molt_ok;
use crate::scope::ScopeStack;
use crate::types::Command;
use crate::types::*;
use crate::value::Value;
use std::any::Any;
use std::collections::HashMap;
use std::rc::Rc;

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
/// # use molt::Value;
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
    context_map: HashMap<ContextID, Box<Any>>,

    // Defines the recursion limit for Interp::eval().
    recursion_limit: usize,

    // Current number of eval levels.
    num_levels: usize,
}

// NOTE: The order of methods in the generated RustDoc depends on the order in this block.
// Consequently, methods are ordered pedagogically.
impl Interp {
    //--------------------------------------------------------------------------------------------
    // Constructors

    /// Creates a new Molt interpreter with no commands defined.  Use this when crafting
    /// command languages that shouldn't include the normal TCL commands, or as a base
    /// for adding specific command sets.
    pub fn empty() -> Self {
        Self {
            recursion_limit: 1000,
            commands: HashMap::new(),
            last_context_id: 0,
            context_map: HashMap::new(),
            scopes: ScopeStack::new(),
            num_levels: 0,
        }
    }

    /// Creates a new Molt interpreter, pre-populated with the standard Molt commands.
    /// Use `info commands` to retrieve the full list.
    /// TODO: Define command sets (sets of commands that go together, so that clients can
    /// add or remove them in groups).
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

        interp
    }

    //--------------------------------------------------------------------------------------------
    // Script and Expression Evaluation

    /// Evaluates a script one command at a time.  Returns the `Value` of the last command
    /// in the script, or the value of any explicit `return` call in the script, or an
    /// error thrown by the script.
    ///
    /// This method returns only `Ok` or `ResultCode::Error`.  `ResultCode::Return` is converted
    /// to `Ok`, and `ResultCode::Break` and `ResultCode::Continue` are converted to errors.
    ///
    /// Use this method (or [`eval_value`](#method.eval_value) to evaluate arbitrary scripts.
    /// Use [`eval_body`](#method.eval_body) to evaluate the body of control structures.
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
            Err(ResultCode::Return(value)) => molt_ok!(value),
            Err(ResultCode::Break) => molt_err!("invoked \"break\" outside of a loop"),
            Err(ResultCode::Continue) => molt_err!("invoked \"continue\" outside of a loop"),
            _ => result,
        }
    }

    /// Evaluates a script one command at a time.  Returns the `Value` of the last command
    /// in the script, or the value of any explicit `return` call in the script, or an
    /// error thrown by the script.
    ///
    /// This method returns only `Ok` or `ResultCode::Error`.  `ResultCode::Return` is converted
    /// to `Ok`, and `ResultCode::Break` and `ResultCode::Continue` are converted to errors.
    ///
    /// Use this method (or [`eval`](#method.eval) to evaluate arbitrary scripts.
    /// Use [`eval_body`](#method.eval_body) to evaluate the body of control structures.
    pub fn eval_value(&mut self, script: &Value) -> MoltResult {
        // TODO: Could probably do better, here.  If the value is already a list, for
        // example, can maybe evaluate it as a command without parsing the string.
        self.eval(&*script.as_string())
    }

    /// Evaluates a script one command at a time, returning whatever
    /// MoltResult arises.
    ///
    /// This is the method to use when evaluating a control structure's
    /// script body; the control structure must handle the special
    /// result codes appropriately.
    pub fn eval_body(&mut self, script: &Value) -> MoltResult {
        let script = script.as_string();
        let mut ctx = EvalPtr::new(&*script);

        self.eval_context(&mut ctx)
    }

    /// Determines whether or not the script is syntactically complete,
    /// e.g., has no unmatched quotes, brackets, or braces.
    ///
    /// REPLs use this to determine whether or not to ask for another line of
    /// input.
    ///
    /// # Example
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

    // Evaluates an expression and returns its value.
    pub fn expr(&mut self, expr: &Value) -> MoltResult {
        expr::expr(self, expr)
    }

    // Evaluates a boolean expression and returns its value, or an error if it couldn't
    // be interpreted as a boolean.
    pub fn bool_expr(&mut self, expr: &Value) -> Result<bool, ResultCode> {
        expr::bool_expr(self, expr)
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
    // Command Definition and Handling

    /// Adds a command defined by a `CommandFunc` to the interpreter.
    ///
    /// This is the normal way to add commands to
    /// the interpreter.  If the command requires context other than the interpreter itself,
    /// add the context data to the context cache and use
    /// [`add_context_command`](#method.add_context_command), or define a struct that
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
    // Explicit Substitutions
    //
    // These methods substitute backslashes, variables, and commands into a string.

    /// Performs standard TCL backslash substitution in the string, returning a new string.
    ///
    /// The following substitions are performed:
    ///
    /// | Sequence   | Substitution                                                 |
    /// | ---------- | ------------------------------------------------------------ |
    /// | \a         | ASCII 7: Audible Alarm                                       |
    /// | \b         | ASCII 8: Backspace                                           |
    /// | \f         | ASCII 12: Form Feed                                          |
    /// | \n         | New Line                                                     |
    /// | \r         | Carriage Return                                              |
    /// | \t         | Tab                                                          |
    /// | \v         | ASCII 11: Vertical Tab                                       |
    /// | \ooo       | Character _ooo_, where _o_ is an octal digit.                |
    /// | \xhh       | Character _hh_, where _h_ is a hex digit.                    |
    /// | \uhhhh     | Character _hhhh_, where _hhhh_ is 1 to 4 hex digits.         |
    /// | \Uhhhhhhhh | Character _hhhhhhhh_, where _hhhhhhhh_ is 1 to 8 hex digits. |
    ///
    /// Any other character preceded by a backslash is replaced with itself.
    ///
    /// # Example
    ///
    /// ```
    /// # use molt::types::*;
    /// # use molt::interp::Interp;
    /// let interp = Interp::new();
    /// assert_eq!("+\x07-\n-\r-p+", interp.subst_backslashes("+\\a-\\n-\\r-\\x70+"));
    /// ```

    pub fn subst_backslashes(&self, str: &str) -> String {
        subst_backslashes(str)
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
            let words = self.parse_command(ctx)?;

            if words.is_empty() {
                break;
            }

            // When scanning for info
            if ctx.is_no_eval() {
                continue;
            }

            // FIRST, convert to Vec<&str>
            let name = &*words[0].as_string();
            if let Some(cmd) = self.commands.get(name) {
                let cmd = Rc::clone(cmd);
                let result = cmd.execute(self, words.as_slice());
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

    /// Parse a braced word.
    pub(crate) fn parse_braced_word(&mut self, ctx: &mut EvalPtr) -> MoltResult {
        // FIRST, we have to count braces.  Skip the first one, and count it.
        ctx.next();
        let mut count = 1;
        let mut word = String::new();

        // NEXT, add characters to the word until we find the matching close-brace,
        // which is NOT added to the word.  It's an error if we reach the end before
        // finding the close-brace.
        while let Some(c) = ctx.next() {
            if c == '\\' {
                // Backslash substitution.  If next character is a
                // newline, replace it with a space.  Otherwise, this
                // character and the next go into the word as is.
                // Note: this means that escaped '{' and '}' characters
                // don't affect the count.
                if ctx.next_is('\n') {
                    word.push(' ');
                } else {
                    word.push('\\');
                    if !ctx.at_end() {
                        word.push(ctx.next().unwrap());
                    }
                }
                continue;
            } else if c == '{' {
                count += 1;
            } else if c == '}' {
                count -= 1;
            }

            if count > 0 {
                word.push(c)
            } else {
                // We've found and consumed the closing brace.  We should either
                // see more more whitespace, or we should be at the end of the command.
                // Otherwise, there are incorrect characters following the close-brace.
                if ctx.at_end_of_command() || ctx.next_is_line_white() {
                    return molt_ok!(word);
                } else {
                    return molt_err!("extra characters after close-brace");
                }
            }
        }

        assert!(count > 0);
        molt_err!("missing close-brace")
    }

    /// Parse a quoted word.
    pub(crate) fn parse_quoted_word(&mut self, ctx: &mut EvalPtr) -> MoltResult {
        // FIRST, consume the the opening quote.
        ctx.next();

        // NEXT, add characters to the word until we reach the close quote
        let mut word = String::new();

        while !ctx.at_end() {
            // Note: the while condition ensures that there's a character.
            if ctx.next_is('[') {
                word.push_str(&*self.parse_script(ctx)?.as_string());
            } else if ctx.next_is('$') {
                word.push_str(&*self.parse_variable(ctx)?.as_string());
            } else if ctx.next_is('\\') {
                subst_backslash(ctx, &mut word);
            } else if !ctx.next_is('"') {
                word.push(ctx.next().unwrap());
            } else {
                ctx.skip_char('"');
                return Ok(Value::from(word));
            }
        }

        molt_err!("missing \"")
    }

    /// Parse a bare word.
    fn parse_bare_word(&mut self, ctx: &mut EvalPtr) -> MoltResult {
        let mut word = String::new();

        while !ctx.at_end_of_command() && !ctx.next_is_line_white() {
            // Note: the while condition ensures that there's a character.
            if ctx.next_is('[') {
                word.push_str(&*self.parse_script(ctx)?.as_string());
            } else if ctx.next_is('$') {
                word.push_str(&*self.parse_variable(ctx)?.as_string());
            } else if ctx.next_is('\\') {
                subst_backslash(ctx, &mut word);
            } else {
                word.push(ctx.next().unwrap());
            }
        }

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

        // NEXT, get the variable name
        let mut varname = String::new();

        if ctx.next_is_varname_char() {
            while ctx.next_is_varname_char() {
                varname.push(ctx.next().unwrap());
            }
        } else if ctx.next_is('{') {
            ctx.skip_char('{');
            varname.push_str(&*self.parse_braced_varname(ctx)?.as_string());
        }

        Ok(self.var(&varname)?)
    }

    fn parse_braced_varname(&self, ctx: &mut EvalPtr) -> MoltResult {
        let mut string = String::new();

        while !ctx.at_end() {
            let c = ctx.next().unwrap();

            if c == '}' {
                return Ok(Value::from(string));
            } else {
                string.push(c);
            }
        }

        molt_err!("missing close-brace for variable name")
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
        let name = &*argv[0].as_string();

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
            if &*vec[0].as_string() == "args" && speci == self.args.len() - 1 {
                interp.set_and_return("args", Value::from(&argv[argi..]));

                // We've processed all of the args
                argi = argv.len();
                break;
            }

            // NEXT, do we have a matching argument?
            if argi < argv.len() {
                // Pair them up
                interp.set_var(&*vec[0].as_string(), &argv[argi]);
                argi += 1;
                continue;
            }

            // NEXT, do we have a default value?
            if vec.len() == 2 {
                interp.set_var(&*vec[0].as_string(), &vec[1]);
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
            if *arg.as_string() == "args" && i == self.args.len() - 1 {
                msg.push_str("?arg ...?");
                break;
            }

            let vec = arg.as_list().expect("error in proc arglist validation!");

            if vec.len() == 1 {
                msg.push_str(&*vec[0].as_string());
            } else {
                msg.push('?');
                msg.push_str(&*vec[0].as_string());
                msg.push('?');
            }
        }
        msg.push_str("\"");

        molt_err!(&msg)
    }
}

/// Performs standard TCL backslash substitution in the string, returning a new string.
pub(crate) fn subst_backslashes(str: &str) -> String {
    let mut item = String::new();
    let mut ctx = EvalPtr::new(str);

    while !ctx.at_end() {
        if ctx.next_is('\\') {
            subst_backslash(&mut ctx, &mut item);
        } else {
            item.push(ctx.next().unwrap());
        }
    }

    item
}

// Converts a backslash escape into the equivalent character.
fn subst_backslash(ctx: &mut EvalPtr, word: &mut String) {
    // FIRST, skip the first backslash.
    ctx.skip_char('\\');

    // NEXT, get the next character.
    if let Some(c) = ctx.next() {
        match c {
            'a' => word.push('\x07'), // Audible Alarm
            'b' => word.push('\x08'), // Backspace
            'f' => word.push('\x0c'), // Form Feed
            'n' => word.push('\n'),   // New Line
            'r' => word.push('\r'),   // Carriage Return
            't' => word.push('\t'),   // Tab
            'v' => word.push('\x0b'), // Vertical Tab
            '0'...'7' => {
                let mut octal = String::new();
                octal.push(c);

                if ctx.next_is_octal_digit() {
                    octal.push(ctx.next().unwrap());
                }

                if ctx.next_is_octal_digit() {
                    octal.push(ctx.next().unwrap());
                }

                let val = u8::from_str_radix(&octal, 8).unwrap();
                word.push(val as char);
            }
            // \xhh -- 2 hex digits
            // \Uhhhhhhhh -- 1 to 8 hex digits
            'x' => {
                if !ctx.next_is_hex_digit() {
                    word.push(c);
                    return;
                }

                let mut hex = String::new();
                hex.push(ctx.next().unwrap());

                if ctx.next_is_hex_digit() {
                    hex.push(ctx.next().unwrap());
                } else {
                    word.push(c);
                    word.push_str(&hex);
                    return;
                }

                let val = u32::from_str_radix(&hex, 16).unwrap();
                if let Some(ch) = std::char::from_u32(val) {
                    word.push(ch);
                } else {
                    word.push('x');
                    word.push_str(&hex);
                }
            }
            // \uhhhh -- 1 to 4 hex digits
            // \Uhhhhhhhh -- 1 to 8 hex digits
            'u' | 'U' => {
                if !ctx.next_is_hex_digit() {
                    word.push(c);
                    return;
                }

                let mut hex = String::new();
                let max = if c == 'u' { 4 } else { 8 };

                while ctx.next_is_hex_digit() && hex.len() < max {
                    hex.push(ctx.next().unwrap());
                }

                let val = u32::from_str_radix(&hex, 16).unwrap();
                if let Some(ch) = std::char::from_u32(val) {
                    word.push(ch);
                } else {
                    word.push('u');
                    word.push_str(&hex);
                }
            }
            _ => word.push(c),
        }
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
        assert_eq!(interp.bool_expr(&Value::from("1")), Ok(true));
        assert_eq!(interp.bool_expr(&Value::from("0")), Ok(false));
        assert_eq!(
            interp.bool_expr(&Value::from("a")),
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

    #[test]
    fn test_subst_backslashes() {
        // This function tests the function by testing the Interp method
        let interp = Interp::new();

        assert_eq!("abc", interp.subst_backslashes("abc"));
        assert_eq!("1\x072", interp.subst_backslashes("1\\a2"));
        assert_eq!("1\x082", interp.subst_backslashes("1\\b2"));
        assert_eq!("1\x0c2", interp.subst_backslashes("1\\f2"));
        assert_eq!("1\n2", interp.subst_backslashes("1\\n2"));
        assert_eq!("1\r2", interp.subst_backslashes("1\\r2"));
        assert_eq!("1\t2", interp.subst_backslashes("1\\t2"));
        assert_eq!("1\x0b2", interp.subst_backslashes("1\\v2"));
        assert_eq!("1\x072", interp.subst_backslashes("1\\0072"));
        assert_eq!("XpY", interp.subst_backslashes("X\x70Y"));
        assert_eq!("X\x07Y", interp.subst_backslashes("X\\u7Y"));
        assert_eq!("XwY", interp.subst_backslashes("X\\u77Y"));
        assert_eq!("XwY", interp.subst_backslashes("X\\u077Y"));
        assert_eq!("XwY", interp.subst_backslashes("X\\u0077Y"));
        assert_eq!("X\x07Y", interp.subst_backslashes("X\\U7Y"));
        assert_eq!("XwY", interp.subst_backslashes("X\\U77Y"));
        assert_eq!("XwY", interp.subst_backslashes("X\\U077Y"));
        assert_eq!("XwY", interp.subst_backslashes("X\\U0077Y"));
        assert_eq!("XwY", interp.subst_backslashes("X\\U00077Y"));
        assert_eq!("XwY", interp.subst_backslashes("X\\U000077Y"));
        assert_eq!("XwY", interp.subst_backslashes("X\\U0000077Y"));
        assert_eq!("XwY", interp.subst_backslashes("X\\U00000077Y"));
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
