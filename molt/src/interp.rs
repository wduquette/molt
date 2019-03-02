//! The Interpreter
use crate::list::list_to_string;
use crate::list::get_list;
use crate::commands;
use crate::context::Context;
use crate::molt_ok;
use crate::molt_err;
use crate::var_stack::VarStack;
use crate::types::Command;
use crate::types::CommandFunc;
use crate::types::*;
use std::collections::HashMap;
use std::rc::Rc;

/// The Molt Interpreter.
#[derive(Default)]
#[allow(dead_code)] // TEMP
pub struct Interp {
    // How many nested calls to Interp::eval() do we allow?
    max_nesting_depth: usize,

    // Command Table
    commands: HashMap<String, Rc<dyn Command>>,

    // Variable Table
    var_stack: VarStack,

    // Current number of eval levels.
    num_levels: usize,
}

impl Interp {
    /// Create a new interpreter, pre-populated with the standard commands.
    /// TODO: Probably want to create it empty and provide command sets.
    pub fn new() -> Self {
        let mut interp = Self {
            max_nesting_depth: 255,
            commands: HashMap::new(),
            var_stack: VarStack::new(),
            num_levels: 0,
        };

        interp.add_command("append", commands::cmd_append);
        interp.add_command("assert_eq", commands::cmd_assert_eq);
        interp.add_command("break", commands::cmd_break);
        interp.add_command("catch", commands::cmd_catch);
        interp.add_command("continue", commands::cmd_continue);
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
        interp.add_command("return", commands::cmd_return);
        interp.add_command("source", commands::cmd_source);
        interp.add_command("set", commands::cmd_set);
        interp.add_command("unset", commands::cmd_unset);
        interp
    }

    pub fn add_command(&mut self, name: &str, func: CommandFunc) {
        let command = Rc::new(CommandFuncWrapper::new(func));
        self.add_command_obj(name, command);
    }

    pub fn add_command_proc(&mut self, name: &str, args: Vec<String>, body: &str) {
        let command = Rc::new(CommandProc {
            args: args,
            body: body.to_string(),
        });

        self.add_command_obj(name, command);
    }

    pub fn add_command_obj(&mut self, name: &str, command: Rc<dyn Command>) {
        self.commands.insert(name.into(), command);
    }

    /// Gets a vector of the command names.
    pub fn get_command_names(&self) -> Vec<String> {
        let vec: Vec<String> = self.commands.keys().cloned().collect();

        vec
    }

    pub fn get_var(&self, name: &str) -> InterpResult {
        match self.var_stack.get(name) {
            Some(v) => molt_ok!(v.clone()),
            None => molt_err!("can't read \"{}\": no such variable", name),
        }
    }

    pub fn set_var(&mut self, name: &str, value: &str) {
        self.var_stack.set(name, value);
    }

    pub fn unset_var(&mut self, name: &str) {
        self.var_stack.unset(name);
    }

    /// Gets a vector of the visible var names.
    pub fn get_visible_var_names(&self) -> Vec<String> {
        self.var_stack.get_visible_names()
    }

    /// Pushes a variable scope on to the var_stack.
    /// Procs use this to define their local scope.
    pub fn push_scope(&mut self) {
        self.var_stack.push();
    }

    /// Pops a variable scope off of the var_stack.
    pub fn pop_scope(&mut self) {
        self.var_stack.pop();
    }

    /// Return the current scope level
    pub fn scope_level(&self) -> usize {
        self.var_stack.top()
    }

    /// Links the variable name in the current scope to the given scope.
    pub fn upvar(&mut self, level: usize, name: &str) {
        assert!(level <= self.var_stack.top(), "Invalid scope level");
        self.var_stack.upvar(level, name);
    }

    /// Evaluates a script one command at a time, returning the
    /// value of the last command in the script, the value of an explicit
    /// `return` command, or an error.
    ///
    /// `break` and `continue` results are converted to errors.
    ///
    /// This is the method to use when evaluating an entire script.
    pub fn eval(&mut self, script: &str) -> InterpResult {
        let mut ctx = Context::new(script);

        let result = self.eval_context(&mut ctx);

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
    /// InterpResult arises.
    ///
    /// This is the method to use when evaluating a control structure's
    /// script body; the control structure must handle the special
    /// result codes appropriately.
    pub fn eval_body(&mut self, script: &str) -> InterpResult {
        let mut ctx = Context::new(script);

        self.eval_context(&mut ctx)
    }

    /// Determines whether or not the script is syntactically complete,
    /// e.g., has no unmatched quotes, brackets, or braces.  Used by
    /// REPLs to determine whether or not to ask for another line of
    /// input.
    pub fn complete(&mut self, script: &str) -> bool {
        let mut ctx = Context::new(script);
        ctx.set_no_eval(true);

        self.eval_context(&mut ctx).is_ok()
    }

    /// Low-level script evaluator; evaluates the next script in the
    /// context.
    fn eval_context(&mut self, ctx: &mut Context) -> InterpResult {
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
    fn parse_word(&mut self, ctx: &mut Context) -> InterpResult {
        if ctx.next_is('{') {
            Ok(self.parse_braced_word(ctx)?)
        } else if ctx.next_is('"') {
            Ok(self.parse_quoted_word(ctx)?)
        } else {
            Ok(self.parse_bare_word(ctx)?)
        }
    }

    /// Parse a braced word.
    pub(crate) fn parse_braced_word(&mut self, ctx: &mut Context) -> InterpResult {
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
    pub(crate) fn parse_quoted_word(&mut self, ctx: &mut Context) -> InterpResult {
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
    fn parse_bare_word(&mut self, ctx: &mut Context) -> InterpResult {
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

    pub(crate) fn parse_script(&mut self, ctx: &mut Context) -> InterpResult {
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

    pub(crate) fn parse_variable(&mut self, ctx: &mut Context) -> InterpResult {
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

        Ok(self.get_var(&varname)?)
    }

    fn parse_braced_varname(&self, ctx: &mut Context) -> InterpResult {
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
    fn execute(&self, interp: &mut Interp, argv: &[&str]) -> InterpResult {
        (self.func)(interp, argv)
    }
}

// Context structure for a proc.
struct CommandProc {
    args: Vec<String>,
    body: String
}

impl Command for CommandProc {
    fn execute(&self, interp: &mut Interp, argv: &[&str]) -> InterpResult {
        // FIRST, push the proc's local scope onto the stack.
        interp.push_scope();

        // NEXT, process the proc's argument list.
        let mut argi = 1; // Skip the proc's name

        for (speci, spec) in self.args.iter().enumerate() {
            // FIRST, get the parameter as a vector.  It should be a list of
            // one or two elements.
            let vec = get_list(&spec)?;  // Should never fail
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
                return wrong_num_args_for_proc(argv[0], &self.args);
            }
        }

        // NEXT, do we have any arguments left over?
        if argi != argv.len() {
            return wrong_num_args_for_proc(argv[0], &self.args);
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

fn wrong_num_args_for_proc(name: &str, args: &[String]) -> InterpResult {
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

        let vec = get_list(arg).expect("error in proc arglist validation!");

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

/// Substitutes backslashes in the string, returning a new string.
pub fn subst_backslashes(str: &str) -> String {
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
