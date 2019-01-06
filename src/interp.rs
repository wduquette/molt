//! The Interpreter
use std::collections::HashMap;


/// The interpreter's result.
pub type InterpResult = Result<String,String>;
pub type Command<T> = fn(&mut Interp<T>, &mut T, &[&str]) -> InterpResult;

#[derive(Default)]
pub struct Interp<T> {
    commands: HashMap<String,Command<T>>,
}

impl<T> Interp<T> {
    pub fn new() -> Self {
        let mut interp = Self {
            commands: HashMap::new()
        };

        interp.define("ident", cmd_ident);
        interp.define("puts", cmd_puts);
        interp
    }

    pub fn define(&mut self, name: &str, command: Command<T>) {
        self.commands.insert(name.into(), command);
    }

    /// Evaluates a script one command at a time.
    // TODO: I'll ultimately want a more complex Ok result.
    pub fn eval(&mut self, _context: &mut T, _script: &str) -> InterpResult {

        Ok("".into())
    }
}

fn cmd_puts<T>(_interp: &mut Interp<T>, _context: &mut T, argv: &[&str]) -> InterpResult {
    gcl_arg_check(argv, 2, 2, "text")?;

    println!("{}", argv[1]);

    Ok("".into())
}

fn cmd_ident<T>(_interp: &mut Interp<T>, _context: &mut T, argv: &[&str]) -> InterpResult {
    gcl_arg_check(argv, 2, 2, "value")?;

    Ok(argv[1].into())
}

/// Checks to see whether a command's argument list is of a reasonable size.
/// Returns an error if not.  The arglist must have at least min entries, and can have up
/// to max.  If max is 0, there is no maximum.  argv[0] is always the command name, and
/// is included in the count; thus, min should always be >= 1.
///
/// *Note:* Defined as a function because it doesn't need anything from the Interp.
pub fn gcl_arg_check(argv: &[&str], min: usize, max: usize, argsig: &str) -> InterpResult {
    assert!(min >= 1);
    assert!(!argv.is_empty());

    if argv.len() < min || (max > 0 && argv.len() > max) {
        Err(format!("Wrong # args, should be: \"{} {}\"", argv[0], argsig))
    } else {
        Ok("".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Context {
        info: String,
    }

    struct MyInterp {
        context: Context,
        interp: Interp<Context>,
    }

    impl Context {
        fn new() -> Self {
            Self {
                info: String::new(),
            }
        }
    }

    impl MyInterp {
        fn new() -> Self {
            Self {
                context: Context::new(),
                interp: Interp::new(),
            }
        }

        fn eval(&mut self, script: &str) -> InterpResult {
            self.interp.eval(&mut self.context, script)
        }
    }

    #[test]
    fn test_gcl_arg_check1() {
        assert_ok(&gcl_arg_check(vec!["mycmd"].as_slice(), 1, 1, ""));
        assert_ok(&gcl_arg_check(vec!["mycmd"].as_slice(), 1, 2, "arg1"));
        assert_ok(&gcl_arg_check(vec!["mycmd","data"].as_slice(), 1, 2, "arg1"));
        assert_ok(&gcl_arg_check(vec!["mycmd","data","data2"].as_slice(), 1, 0, "arg1"));

        assert_err(&gcl_arg_check(vec!["mycmd"].as_slice(), 2, 2, "arg1"),
            "Wrong # args, should be: \"mycmd arg1\"");
        assert_err(&gcl_arg_check(vec!["mycmd", "val1", "val2"].as_slice(), 2, 2, "arg1"),
            "Wrong # args, should be: \"mycmd arg1\"");
    }

    #[test]
    fn test_interp_new() {
        let mut my_interp =  MyInterp::new();

        assert_ok(&my_interp.eval("foo"));
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
