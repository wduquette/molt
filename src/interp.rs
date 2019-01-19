//! The Interpreter
use crate::okay;
use crate::commands;
use crate::context::Context;
use crate::error;
use crate::types::Command;
use crate::types::CommandFunc;
use crate::types::*;
use std::collections::HashMap;
use std::rc::Rc;

/// The GCL Interpreter.
#[derive(Default)]
#[allow(dead_code)] // TEMP
pub struct Interp {
    // How many nested calls to Interp::eval() do we allow?
    max_nesting_depth: usize,

    // Command Table
    commands: HashMap<String, Rc<dyn Command>>,

    // Variable Table
    vars: HashMap<String, String>,

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
            vars: HashMap::new(),
            num_levels: 0,
        };

        interp.add_command("exit", commands::cmd_exit);
        interp.add_command("puts", commands::cmd_puts);
        interp.add_command("set", commands::cmd_set);
        interp
    }

    pub fn add_command(&mut self, name: &str, func: CommandFunc) {
        let command = Rc::new(CommandFuncWrapper::new(func));
        self.add_command_obj(name, command);
    }

    pub fn add_command_obj(&mut self, name: &str, command: Rc<dyn Command>) {
        self.commands.insert(name.into(), command);
    }

    pub fn get_var(&self, name: &str) -> InterpResult {
        match self.vars.get(name) {
            Some(v) => Ok(v.clone()),
            None => error(&format!("can't read \"{}\": no such variable", name)),
        }
    }

    pub fn set_var(&mut self, name: &str, value: &str) {
        self.vars.insert(name.into(), value.into());
    }

    /// Evaluates a script one command at a time, and returns either an error or
    /// the result of the last command in the script.
    pub fn eval(&mut self, script: &str) -> InterpResult {
        let mut ctx = Context::new(script);

        self.eval_script(&mut ctx)
    }

    /// Low-level script evaluator; used to implement eval(), complete(), etc.
    fn eval_script(&mut self, ctx: &mut Context) -> InterpResult {
        let mut result_value = String::new();

        while !ctx.at_end_of_script() {
            let vec = self.parse_command(ctx)?;

            if vec.is_empty() {
                return okay();
            }

            // FIRST, convert to Vec<&str>
            let words: Vec<&str> = vec.iter().map(|s| &**s).collect();

            if let Some(cmd) = self.commands.get(words[0]) {
                let cmd = Rc::clone(cmd);
                let result = cmd.execute(self, words.as_slice());
                match result {
                    Ok(v) => {
                        result_value = v;
                    }
                    Err(ResultCode::Return(_)) => {
                        return error("return not yet implemented");
                    }
                    Err(ResultCode::Break) => {
                        return error("break not yet implemented");
                    }
                    Err(ResultCode::Continue) => {
                        return error("break not yet implemented");
                    }
                    Err(ResultCode::Error(_)) => {
                        return result;
                    }
                }
            } else {
                return error(&format!("invalid command name \"{}\"", words[0]));
            }
        }

        Ok(result_value)
    }

    fn parse_command(&mut self, ctx: &mut Context) -> Result<Vec<String>,ResultCode> {
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
    fn parse_braced_word(&mut self, ctx: &mut Context) -> InterpResult {
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
                    return Ok(word);
                } else {
                    return error("extra characters after close-brace");
                }
            }
        }

        assert!(count > 0);
        error("missing close-brace")
    }

    /// Parse a quoted word.
    fn parse_quoted_word(&mut self, ctx: &mut Context) -> InterpResult {
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
                self.parse_backslash(ctx, &mut word);
            } else if !ctx.next_is('"') {
                word.push(ctx.next().unwrap());
            } else {
                ctx.skip_char('"');
                return Ok(word);
            }
        }

        error("missing \"")
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
                self.parse_backslash(ctx, &mut word);
            } else {
                word.push(ctx.next().unwrap());
            }
        }

        Ok(word)
    }

    fn parse_script(&mut self, ctx: &mut Context) -> InterpResult {
        // FIRST, skip the '['
        ctx.skip_char('[');

        // NEXT, parse the script up to the matching ']'
        let old_flag = ctx.is_bracket_term();
        ctx.set_bracket_term(true);
        let result = self.eval_script(ctx);
        ctx.set_bracket_term(old_flag);

        // NEXT, make sure there's a closing bracket
        if result.is_ok() {
            if ctx.next_is(']') {
                ctx.next();
            } else {
                return error("missing close-bracket");
            }
        }

        result
    }

    fn parse_variable(&mut self, ctx: &mut Context) -> InterpResult {
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

        error("missing close-brace for variable name")
    }

    // Converts a backslash escape into the equivalent character.
    fn parse_backslash(&self, ctx: &mut Context, word: &mut String) {
        // FIRST, skip the first backslash.
        ctx.skip_char('\\');

        // NEXT, get the next character.
        if let Some(c) = ctx.next() {
            match c {
                'a' => word.push('\x07'),  // Audible Alarm
                'b' => word.push('\x08'),  // Backspace
                'f' => word.push('\x0c'),  // Form Feed
                'n' => word.push('\n'),    // New Line
                'r' => word.push('\r'),    // Carriage Return
                't' => word.push('\t'),    // Tab
                'v' => word.push('\x0b'),  // Vertical Tab
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
