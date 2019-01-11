//! The Interpreter
use crate::parse_command;
use crate::Command;
use crate::InterpResult;
use crate::CommandFunc;
use crate::commands;
use std::rc::Rc;
use std::collections::HashMap;

/// The GCL Interpreter.
#[derive(Default)]
pub struct Interp {
    commands: HashMap<String,Rc<dyn Command>>,
}

impl Interp {
    /// Create a new interpreter, pre-populated with the standard commands.
    /// TODO: Probably want to created it empty and provide command sets.
    pub fn new() -> Self {
        let mut interp = Self {
            commands: HashMap::new()
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

        let mut result: String = String::new();

        while let Some(vec) = parse_command(chars) {
            // FIRST, convert to Vec<&str>
            let words: Vec<&str> = vec.iter().map(|s| &**s).collect();

            if let Some(cmd) = self.commands.get(words[0]) {
                let cmd = Rc::clone(cmd);
                result = cmd.execute(self, words.as_slice())?;
            } else {
                return Err(format!("invalid command name \"{}\"", words[0]));
            }
        }

        Ok(result)
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
