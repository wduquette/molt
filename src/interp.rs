//! The Interpreter
use crate::error;
use crate::types::*;
use std::collections::HashSet;
use crate::parse_command;
use crate::types::Command;
use crate::types::CommandFunc;
use crate::commands;
use std::rc::Rc;
use std::collections::HashMap;

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
    commands: HashMap<String,Rc<dyn Command>>,

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
    // TODO: I'll ultimately want a more complex Ok result.
    pub fn eval(&mut self, script: &str) -> InterpResult {
        let chars = &mut script.chars();
        let mut result_value = String::new();

        while let Some(vec) = parse_command(chars) {
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
}

/// A struct that wraps a command function and implements the Command trait.
struct CommandFuncWrapper {
    func: CommandFunc,
}

impl CommandFuncWrapper {
    fn new(func: CommandFunc) -> Self {
        Self {
            func
        }
    }
}

impl Command for CommandFuncWrapper {
    fn execute(&self, interp: &mut Interp, argv: &[&str]) -> InterpResult {
        (self.func)(interp, argv)
    }
}
