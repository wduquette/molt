//! The Molt Interpreter
//!
//! The [`Interp`] struct is the primary API for embedding Molt into a Rust application.
//!
//! [`Interp`]: struct.Interp.html

use crate::list::list_to_string;
use crate::commands;
use crate::context::Context;
use crate::molt_ok;
use crate::molt_err;
use crate::scope::ScopeStack;
use crate::types::Command;
use crate::types::CommandFunc;
use crate::types::*;
use crate::value::MoltValue;
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
/// # use molt::interp::Interp;
/// # fn dummy() -> MoltResult {
/// let mut interp = Interp::new();
/// let four = interp.eval("expr {2 + 2}")?;
/// assert_eq!(four, "4");
/// # Ok("".to_string())
/// # }
/// ```
#[derive(Default)]
#[allow(dead_code)] // TEMP
pub struct Interp {
    // Command Table
    commands: HashMap<String, Rc<dyn Command>>,

    // Variable Table
    scopes: ScopeStack,

    // Defines the recursion limit for Interp::eval().
    recursion_limit: usize,

    // Current number of eval levels.
    num_levels: usize,
}

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
            scopes: ScopeStack::new(),
            num_levels: 0,
        }
    }

    /// Creates a new Molt interpreter, pre-populated with the standard Molt commands.
    /// Use `info commands` to retrieve the full list.
    /// TODO: Define command sets
    pub fn new() -> Self {
        let mut interp = Interp::empty();

        interp.add_command("append", commands::cmd_append);
        interp.add_command("assert_eq", commands::cmd_assert_eq);
        interp.add_command("break", commands::cmd_break);
        interp.add_command("catch", commands::cmd_catch);
        interp.add_command("continue", commands::cmd_continue);
        interp.add_command("error", commands::cmd_error);
        interp.add_command("exit", commands::cmd_exit);
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
        interp.add_command("source", commands::cmd_source);
        interp.add_command("set", commands::cmd_set);
        interp.add_command("unset", commands::cmd_unset);
        interp.add_command("while", commands::cmd_while);
        interp
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
    // Command Definition and Handling

    /// Adds a command defined by a `CommandFunc` to the interpreter.
    ///
    /// This is the normal way to add commands to
    /// the interpreter.  If the command requires context other than the interpreter itself,
    /// define a struct that implements `Command` and use `add_command_object`.
    pub fn add_command(&mut self, name: &str, func: CommandFunc) {
        let command = Rc::new(CommandFuncWrapper::new(func));
        self.add_command_object(name, command);
    }

    /// Adds a procedure to the interpreter.
    ///
    /// This is how to add a Molt `proc` to the interpreter.  The arguments are the same
    /// as for the `proc` command and the `commands::cmd_proc` function.
    pub fn add_proc(&mut self, name: &str, args: MoltList, body: &str) {
        let command = Rc::new(CommandProc {
            args,
            body: body.to_string(),
        });

        self.add_command_object(name, command);
    }

    /// Adds a command to the interpreter using a `Command` trait object.
    ///
    /// Use this when defining a command that requires application context.
    pub fn add_command_object(&mut self, name: &str, command: Rc<dyn Command>) {
        self.commands.insert(name.into(), command);
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
        let vec: MoltList = self.commands.keys().cloned().collect();

        vec
    }

    //--------------------------------------------------------------------------------------------
    // Argument Conversions
    //
    // These methods convert strings to and from Molt values in the context of the
    // `Interp`

    /// Converts a string argument into a boolean, returning an error on failure.
    /// A command function will call this to convert an argument,
    /// using "?" to propagate errors to the interpreter.
    ///
    /// Molt accepts the following strings as Boolean values:
    ///
    /// * **true**: `true`, `yes`, `on`, `1`
    /// * **false**: `false`, `no`, `off`, `0`
    ///
    /// This method does not evaluate expressions; use TODO to evaluate boolean
    /// expressions.
    ///
    /// # Example
    ///
    /// ```
    /// # use molt::types::*;
    /// # use molt::interp::Interp;
    /// # fn dummy() -> Result<bool,ResultCode> {
    /// # let interp = Interp::new();
    /// let arg = "yes";
    /// let flag = interp.get_bool(arg)?;
    /// assert!(flag);
    /// # Ok(flag)
    /// # }
    /// ```
    pub fn get_bool(&self, arg: &str) -> Result<bool, ResultCode> {
        let value: &str = &arg.to_lowercase();
        match value {
            "1" | "true" | "yes" | "on" => Ok(true),
            "0" | "false" | "no" | "off" => Ok(false),
            _ => molt_err!("expected boolean but got \"{}\"", arg),
        }
    }

    /// Converts an string argument into a `MoltFloat`, returning an error on failure.
    /// A command function will call this to convert an argument into a number,
    /// using "?" to propagate errors to the interpreter.
    ///
    /// Molt accepts any string acceptable to `str::parse<f64>` as a valid floating
    /// point string.
    ///
    /// # Example
    ///
    /// ```
    /// # use molt::Interp;
    /// # use molt::types::*;
    /// # fn dummy() -> Result<MoltFloat,ResultCode> {
    /// # let interp = Interp::new();
    /// let arg = "1e2";
    /// let val = interp.get_float(arg)?;
    /// # Ok(val)
    /// # }
    /// ```
    pub fn get_float(&self, arg: &str) -> Result<MoltFloat, ResultCode> {
        match arg.parse::<MoltFloat>() {
            Ok(val) => Ok(val),
            Err(_) => molt_err!("expected floating-point number but got \"{}\"", arg),
        }
    }

    /// Converts a string argument into a `MoltInt`, returning an error on failure.
    /// A command function will call this to convert an argument into an integer,
    /// using "?" to propagate errors to the interpreter.
    ///
    /// Molt accepts decimal integer strings, and hexadecimal integer strings
    /// with a `0x` prefix.  Strings may begin with a unary "+" or "-".
    ///
    /// # Example
    ///
    /// ```
    /// # use molt::Interp;
    /// # use molt::types::*;
    /// # fn dummy() -> Result<MoltInt,ResultCode> {
    /// # let interp = Interp::new();
    /// let arg = "1";
    /// let int = interp.get_int(arg)?;
    /// # Ok(int)
    /// # }
    /// ```
    pub fn get_int(&self, arg: &str) -> Result<MoltInt, ResultCode> {
        let mut arg = arg;
        let mut minus = 1;

        if arg.starts_with('+') {
            arg = &arg[1..];
        } else if arg.starts_with('-') {
            minus = -1;
            arg = &arg[1..];
        }

        let parse_result = if arg.starts_with("0x") {
            MoltInt::from_str_radix(&arg[2..], 16)
        } else {
            arg.parse::<MoltInt>()
        };

        match parse_result {
            Ok(int) => Ok(minus * int),
            Err(_) => molt_err!("expected integer but got \"{}\"", arg),
        }
    }

    /// Converts a string argument into a `MoltList`,
    /// returning an error on failure. A command function will call this to convert
    /// an argument into a list, using "?" to propagate errors to the interpreter.
    ///
    /// TCL list syntax is too complex to discuss here, but basically consists
    /// of whitespace-delimited items, with normal TCL quoting for items containing
    /// whitespace.
    ///
    /// # Example
    ///
    /// ```
    /// # use molt::Interp;
    /// # use molt::types::*;
    /// # fn dummy() -> Result<MoltList,ResultCode> {
    /// # let interp = Interp::new();
    /// let arg = "a {b c} d";
    /// let list = interp.get_list(arg)?;
    /// assert_eq!(list.len(), 3);
    /// assert_eq!(list[1], "b c".to_string());
    /// # Ok(list)
    /// # }
    /// ```
    pub fn get_list(&self, str: &str) -> Result<MoltList, ResultCode> {
        crate::list::get_list(str)
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
    /// if necessary.
    pub fn set_var(&mut self, name: &str, value: &str) {
        // TODO: Temporary fix while integrating MoltValue.
        self.scopes.set(name, MoltValue::new(value));
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
    // Script and Expression Evaluation

    /// Evaluates a script one command at a time, returning the
    /// value of the last command in the script, the value of an explicit
    /// `return` command, or an error.
    ///
    /// `break` and `continue` results are converted to errors.
    ///
    /// This is the method to use when evaluating an entire script.
    pub fn eval(&mut self, script: &str) -> MoltResult {
        // FIRST, check the number of nesting levels
        self.num_levels += 1;

        if self.num_levels > self.recursion_limit {
            self.num_levels -= 1;
            return molt_err!("too many nested calls to Interp::eval (infinite loop?)");
        }

        // NEXT, evaluate the script and translate the result to Ok or Error
        let mut ctx = Context::new(script);

        let result = self.eval_context(&mut ctx);

        // NEXT, decrement the number of nesting levels.
        self.num_levels -= 1;

        // NEXT, translate and return the result.
        match result {
            Err(ResultCode::Return(value)) => {
                molt_ok!(value)
            }
            Err(ResultCode::Break) => {
                molt_err!("invoked \"break\" outside of a loop")
            }
            Err(ResultCode::Continue) => {
                molt_err!("invoked \"continue\" outside of a loop")
            }
            _ => result
        }
    }

    /// Evaluates a script one command at a time, returning whatever
    /// MoltResult arises.
    ///
    /// This is the method to use when evaluating a control structure's
    /// script body; the control structure must handle the special
    /// result codes appropriately.
    pub fn eval_body(&mut self, script: &str) -> MoltResult {
        let mut ctx = Context::new(script);

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
        let mut ctx = Context::new(script);
        ctx.set_no_eval(true);

        self.eval_context(&mut ctx).is_ok()
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
    fn eval_context(&mut self, ctx: &mut Context) -> MoltResult {
        let mut result_value = String::new();

        while !ctx.at_end_of_script() {
            let vec = self.parse_command(ctx)?;

            if vec.is_empty() {
                break;
            }

            // When scanning for info
            if ctx.is_no_eval() {
                continue;
            }

            // FIRST, convert to Vec<&str>
            let words: Vec<&str> = vec.iter().map(|s| &**s).collect();

            if let Some(cmd) = self.commands.get(words[0]) {
                let cmd = Rc::clone(cmd);
                let result = cmd.execute(self, words.as_slice());
                match result {
                    Ok(v) => result_value = v,
                    _ => return result,
                }
            } else {
                return molt_err!("invalid command name \"{}\"", words[0]);
            }
        }

        Ok(result_value)
    }

    fn parse_command(&mut self, ctx: &mut Context) -> Result<Vec<String>, ResultCode> {
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
    fn parse_word(&mut self, ctx: &mut Context) -> MoltResult {
        if ctx.next_is('{') {
            Ok(self.parse_braced_word(ctx)?)
        } else if ctx.next_is('"') {
            Ok(self.parse_quoted_word(ctx)?)
        } else {
            Ok(self.parse_bare_word(ctx)?)
        }
    }

    /// Parse a braced word.
    pub(crate) fn parse_braced_word(&mut self, ctx: &mut Context) -> MoltResult {
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
    pub(crate) fn parse_quoted_word(&mut self, ctx: &mut Context) -> MoltResult {
        // FIRST, consume the the opening quote.
        ctx.next();

        // NEXT, add characters to the word until we reach the close quote
        let mut word = String::new();

        while !ctx.at_end() {
            // Note: the while condition ensures that there's a character.
            if ctx.next_is('[') {
                word.push_str(&self.parse_script(ctx)?);
            } else if ctx.next_is('$') {
                word.push_str(&self.parse_variable(ctx)?);
            } else if ctx.next_is('\\') {
                subst_backslash(ctx, &mut word);
            } else if !ctx.next_is('"') {
                word.push(ctx.next().unwrap());
            } else {
                ctx.skip_char('"');
                return Ok(word);
            }
        }

        molt_err!("missing \"")
    }

    /// Parse a bare word.
    fn parse_bare_word(&mut self, ctx: &mut Context) -> MoltResult {
        let mut word = String::new();

        while !ctx.at_end_of_command() && !ctx.next_is_line_white() {
            // Note: the while condition ensures that there's a character.
            if ctx.next_is('[') {
                word.push_str(&self.parse_script(ctx)?);
            } else if ctx.next_is('$') {
                word.push_str(&self.parse_variable(ctx)?);
            } else if ctx.next_is('\\') {
                subst_backslash(ctx, &mut word);
            } else {
                word.push(ctx.next().unwrap());
            }
        }

        Ok(word)
    }

    pub(crate) fn parse_script(&mut self, ctx: &mut Context) -> MoltResult {
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

    pub(crate) fn parse_variable(&mut self, ctx: &mut Context) -> MoltResult {
        // FIRST, skip the '$'
        ctx.skip_char('$');

        // NEXT, make sure this is really a variable reference.  If it isn't
        // just return a "$".
        if !ctx.next_is_varname_char() && !ctx.next_is('{') {
            return Ok("$".into());
        }

        // NEXT, get the variable name
        let mut varname = String::new();

        if ctx.next_is_varname_char() {
            while ctx.next_is_varname_char() {
                varname.push(ctx.next().unwrap());
            }
        } else if ctx.next_is('{') {
            ctx.skip_char('{');
            varname.push_str(&self.parse_braced_varname(ctx)?);
        }

        Ok(self.var(&varname)?)
    }

    fn parse_braced_varname(&self, ctx: &mut Context) -> MoltResult {
        let mut string = String::new();

        while !ctx.at_end() {
            let c = ctx.next().unwrap();

            if c == '}' {
                return Ok(string);
            } else {
                string.push(c);
            }
        }

        molt_err!("missing close-brace for variable name")
    }
}

/// A struct that wraps a command function and implements the Command trait.
struct CommandFuncWrapper {
    func: CommandFunc,
}

impl CommandFuncWrapper {
    fn new(func: CommandFunc) -> Self {
        Self { func }
    }
}

impl Command for CommandFuncWrapper {
    fn execute(&self, interp: &mut Interp, argv: &[&str]) -> MoltResult {
        (self.func)(interp, argv)
    }
}

// Context structure for a proc.
struct CommandProc {
    args: MoltList,
    body: String
}

impl Command for CommandProc {
    fn execute(&self, interp: &mut Interp, argv: &[&str]) -> MoltResult {
        // FIRST, push the proc's local scope onto the stack.
        interp.push_scope();

        // NEXT, process the proc's argument list.
        let mut argi = 1; // Skip the proc's name

        for (speci, spec) in self.args.iter().enumerate() {
            // FIRST, get the parameter as a vector.  It should be a list of
            // one or two elements.
            let vec = interp.get_list(&spec)?;  // Should never fail
            assert!(vec.len() == 1 || vec.len() == 2);

            // NEXT, if this is the args parameter, give the remaining args,
            // if any.  Note that "args" has special meaning only if it's the
            // final arg spec in the list.
            if vec[0] == "args" && speci == self.args.len() - 1 {
                let args = if argi < argv.len() {
                    list_to_string(&argv[argi..])
                } else {
                    "".into()
                };
                interp.set_var("args", &args);

                // We've processed all of the args
                argi = argv.len();
                break;
            }

            // NEXT, do we have a matching argument?
             if argi < argv.len() {
                // Pair them up
                interp.set_var(&vec[0], argv[argi]);
                argi += 1;
                continue;
            }

            // NEXT, do we have a default value?
            if vec.len() == 2 {
                interp.set_var(&vec[0], &vec[1]);
            } else {
                // We don't; we're missing a required argument.
                return wrong_num_args_for_proc(interp, argv[0], &self.args);
            }
        }

        // NEXT, do we have any arguments left over?
        if argi != argv.len() {
            return wrong_num_args_for_proc(interp, argv[0], &self.args);
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

fn wrong_num_args_for_proc(interp: &Interp, name: &str, args: &[String]) -> MoltResult {
    let mut msg = String::new();
    msg.push_str("wrong # args: should be \"");
    msg.push_str(name);

    for (i, arg) in args.iter().enumerate() {
        msg.push(' ');

        // "args" has special meaning only in the last place.
        if arg == "args" && i == args.len() - 1  {
            msg.push_str("?arg ...?");
            break;
        }

        let vec = interp.get_list(arg).expect("error in proc arglist validation!");

        if vec.len() == 1 {
            msg.push_str(&vec[0]);
        } else {
            msg.push('?');
            msg.push_str(&vec[0]);
            msg.push('?');
        }
    }
    msg.push_str("\"");

    molt_err!(&msg)
}

/// Performs standard TCL backslash substitution in the string, returning a new string.
pub(crate) fn subst_backslashes(str: &str) -> String {
    let mut item = String::new();
    let mut ctx = Context::new(str);

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
fn subst_backslash(ctx: &mut Context, word: &mut String) {
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
    fn test_recursion_limit() {
        let mut interp = Interp::new();

        assert_eq!(interp.recursion_limit(), 1000);
        interp.set_recursion_limit(100);
        assert_eq!(interp.recursion_limit(), 100);

        assert!(interp.eval("proc myproc {} { myproc }").is_ok());
        assert_eq!(interp.eval("myproc"),
            molt_err!("too many nested calls to Interp::eval (infinite loop?)"));
    }

    #[test]
    fn test_complete() {
        // This function tests the function by testing the Interp method
        let mut interp = Interp::new();

        assert!(interp.complete("abc"));
        assert!(interp.complete("a {bc} [def] \"ghi\" xyz"));

        assert!(!interp.complete("a {bc"));
        assert!(!interp.complete("a [bc"));
        assert!(!interp.complete("a \"bc"));
    }

    #[test]
    fn test_get_bool() {
        let interp = Interp::new();

        assert_eq!(Ok(true), interp.get_bool("1"));
        assert_eq!(Ok(true), interp.get_bool("true"));
        assert_eq!(Ok(true), interp.get_bool("yes"));
        assert_eq!(Ok(true), interp.get_bool("on"));
        assert_eq!(Ok(true), interp.get_bool("TRUE"));
        assert_eq!(Ok(true), interp.get_bool("YES"));
        assert_eq!(Ok(true), interp.get_bool("ON"));
        assert_eq!(Ok(false), interp.get_bool("0"));
        assert_eq!(Ok(false), interp.get_bool("false"));
        assert_eq!(Ok(false), interp.get_bool("no"));
        assert_eq!(Ok(false), interp.get_bool("off"));
        assert_eq!(Ok(false), interp.get_bool("FALSE"));
        assert_eq!(Ok(false), interp.get_bool("NO"));
        assert_eq!(Ok(false), interp.get_bool("OFF"));
        assert_eq!(interp.get_bool("nonesuch"),
            molt_err!("expected boolean but got \"nonesuch\""));
    }

    #[test]
    fn test_get_float() {
        let interp = Interp::new();

        assert_eq!(interp.get_float("1"), Ok(1.0));
        assert_eq!(interp.get_float("-1"), Ok(-1.0));
        assert_eq!(interp.get_float("+1"), Ok(1.0));
        assert_eq!(interp.get_float("1e3"), Ok(1000.0));
        assert_eq!(interp.get_float("a"),
            molt_err!("expected floating-point number but got \"a\""));
    }

    #[test]
    fn test_get_int() {
        let interp = Interp::new();

        assert_eq!(interp.get_int("1"), Ok(1));
        assert_eq!(interp.get_int("-1"), Ok(-1));
        assert_eq!(interp.get_int("+1"), Ok(1));
        assert_eq!(interp.get_int("0xFF"), Ok(255));
        assert_eq!(interp.get_int("+0xFF"), Ok(255));
        assert_eq!(interp.get_int("-0xFF"), Ok(-255));

        assert_eq!(interp.get_int(""), molt_err!("expected integer but got \"\""));
        assert_eq!(interp.get_int("a"), molt_err!("expected integer but got \"a\""));
        assert_eq!(interp.get_int("0x"), molt_err!("expected integer but got \"0x\""));
        assert_eq!(interp.get_int("0xABGG"),
            molt_err!("expected integer but got \"0xABGG\""));
    }

    #[test]
    fn test_get_list() {
        // NOTE: List syntax is tested in list.rs; this simply verifies that
        // Interp provides an interface to it.
        let interp = Interp::new();

        let vec = interp.get_list("a b c").unwrap();
        assert_eq!(vec.len(), 3);
        assert_eq!(vec[0], "a".to_string());
        assert_eq!(vec[1], "b".to_string());
        assert_eq!(vec[2], "c".to_string());

        let result = interp.get_list("a {b c");
        assert!(result.is_err());
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
}
