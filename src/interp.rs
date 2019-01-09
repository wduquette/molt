//! The Interpreter
use crate::types::Command;
use crate::types::InterpResult;
use crate::types::CommandFunc;
use std::rc::Rc;
use std::collections::HashMap;



#[derive(Default)]
pub struct Interp {
    commands: HashMap<String,Rc<dyn Command>>,
}

impl Interp {
    pub fn new() -> Self {
        let mut interp = Self {
            commands: HashMap::new()
        };

        interp.add_command("ident", cmd_ident);
        interp.add_command("puts", cmd_puts);
        interp.add_command("exit", cmd_exit);
        interp
    }

    pub fn add_command(&mut self, name: &str, func: CommandFunc) {
        let command = Rc::new(CommandFuncWrapper::new(func));
        self.add_command_object(name, command);
    }

    pub fn add_command_object(&mut self, name: &str, command: Rc<dyn Command>) {
        self.commands.insert(name.into(), command);
    }

    /// Evaluates a script one command at a time.
    // TODO: I'll ultimately want a more complex Ok result.
    pub fn eval(&mut self, _script: &str) -> InterpResult {

        Ok("".into())
    }
}

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

fn cmd_exit(_interp: &mut Interp, argv: &[&str]) -> InterpResult {
    check_args(argv, 1, 1, "")?;

    // TODO: Allow an optional argument, and parse it to i32.
    std::process::exit(0);
}

fn cmd_ident(_interp: &mut Interp, argv: &[&str]) -> InterpResult {
    check_args(argv, 2, 2, "value")?;

    Ok(argv[1].into())
}

fn cmd_puts(_interp: &mut Interp, argv: &[&str]) -> InterpResult {
    check_args(argv, 2, 2, "text")?;

    println!("{}", argv[1]);

    Ok("".into())
}


/// Checks to see whether a command's argument list is of a reasonable size.
/// Returns an error if not.  The arglist must have at least min entries, and can have up
/// to max.  If max is 0, there is no maximum.  argv[0] is always the command name, and
/// is included in the count; thus, min should always be >= 1.
///
/// *Note:* Defined as a function because it doesn't need anything from the Interp.
pub fn check_args(argv: &[&str], min: usize, max: usize, argsig: &str) -> InterpResult {
    assert!(min >= 1);
    assert!(!argv.is_empty());

    if argv.len() < min || (max > 0 && argv.len() > max) {
        Err(format!("wrong # args: should be \"{} {}\"", argv[0], argsig))
    } else {
        Ok("".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_args() {
        assert_ok(&check_args(vec!["mycmd"].as_slice(), 1, 1, ""));
        assert_ok(&check_args(vec!["mycmd"].as_slice(), 1, 2, "arg1"));
        assert_ok(&check_args(vec!["mycmd","data"].as_slice(), 1, 2, "arg1"));
        assert_ok(&check_args(vec!["mycmd","data","data2"].as_slice(), 1, 0, "arg1"));

        assert_err(&check_args(vec!["mycmd"].as_slice(), 2, 2, "arg1"),
            "Wrong # args, should be: \"mycmd arg1\"");
        assert_err(&check_args(vec!["mycmd", "val1", "val2"].as_slice(), 2, 2, "arg1"),
            "Wrong # args, should be: \"mycmd arg1\"");
    }

    // Helpers

    fn assert_err(result: &InterpResult, msg: &str) {
        assert_eq!(Err(msg.into()), *result);
    }

    fn assert_ok(result: &InterpResult) {
        assert!(result.is_ok(), "Result is not Ok");
    }

    // fn assert_value(result: InterpResult, value: &str) {
    //     assert_eq!(Ok(value.into()), result);
    // }
}
