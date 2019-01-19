//! The Interpreter
use crate::commands;
use crate::context::Context;
use crate::error;
use crate::parse_command;
use crate::types::Command;
use crate::types::CommandFunc;
use crate::types::*;
use std::collections::HashMap;
use std::collections::HashSet;
use std::rc::Rc;

/// A set of flags used during parsing.
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
#[allow(dead_code)] // TEMP
pub enum InterpFlags {
    /// TCL_BRACKET_TERM
    BracketTerm,

    /// interp->noEval
    NoEval,
}

/// The GCL Interpreter.
#[derive(Default)]
#[allow(dead_code)] // TEMP
pub struct Interp {
    // How many nested calls to Interp::eval() do we allow?
    max_nesting_depth: usize,

    // Command Table
    commands: HashMap<String, Rc<dyn Command>>,

    // Parsing flags: used to carry flag info through the parsing methods.
    flags: HashSet<InterpFlags>,

    // Current number of eval levels.
    num_levels: usize,
}

impl Interp {
    /// Create a new interpreter, pre-populated with the standard commands.
    /// TODO: Probably want to created it empty and provide command sets.
    pub fn new() -> Self {
        let mut interp = Self {
            max_nesting_depth: 255,
            commands: HashMap::new(),
            flags: HashSet::new(),
            num_levels: 0,
        };

        interp.add_command("exit", commands::cmd_exit);
        interp.add_command("puts", commands::cmd_puts);
        interp
    }

    pub fn add_command(&mut self, name: &str, func: CommandFunc) {
        let command = Rc::new(CommandFuncWrapper::new(func));
        self.add_command_obj(name, command);
    }

    pub fn add_command_obj(&mut self, name: &str, command: Rc<dyn Command>) {
        self.commands.insert(name.into(), command);
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
    ///
    /// TODO: Handle backslashes!
    fn parse_braced_word(&mut self, ctx: &mut Context) -> InterpResult {
        // FIRST, we have to count braces.  Skip the first one, and count it.
        ctx.next();
        let mut count = 1;
        let mut word = String::new();

        // NEXT, add characters to the word until we find the matching close-brace,
        // which is NOT added to the word.  It's an error if we reach the end before
        // finding the close-brace.
        while let Some(c) = ctx.next() {
            if c == '{' {
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
    ///
    /// TODO: Handle backslashes
    /// TODO: Handle interpolated commands.
    /// TODO: Handle interpolated variables.
    fn parse_quoted_word(&mut self, ctx: &mut Context) -> InterpResult {
        // FIRST, consume the the opening quote.
        ctx.next();

        // NEXT, add characters to the word until we reach the close quote
        let mut word = String::new();

        while let Some(c) = ctx.next() {
            if c != '"' {
                word.push(c);
            } else {
                return Ok(word);
            }
        }

        error("missing \"")
    }

    /// Parse a bare word.
    ///
    /// TODO: Handle backslashes
    /// TODO: Handle interpolated commands.
    /// TODO: Handle interpolated variables.
    fn parse_bare_word(&mut self, ctx: &mut Context) -> InterpResult {
        let mut word = String::new();

        while !ctx.at_end_of_command() && !ctx.next_is_line_white() {
            word.push(ctx.next().unwrap());
        }

        Ok(word)
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
