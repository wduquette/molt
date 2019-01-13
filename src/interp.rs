//! The Interpreter
use std::collections::HashSet;
use crate::parse_command;
use crate::Command;
use crate::InterpResult;
use crate::CommandFunc;
use crate::commands;
use std::rc::Rc;
use std::collections::HashMap;
use self::InterpFlags::*;

/// A set of flags used during parsing.
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum InterpFlags {
    /// TCL_BRACKET_TERM
    BracketTerm,

    /// interp->noEval
    NoEval,
}

/// The GCL Interpreter.
#[derive(Default)]
pub struct Interp {
    // Configuration parameters
    max_nesting_depth: usize,

    // Command storage
    commands: HashMap<String,Rc<dyn Command>>,

    // Parsing state
    flags: HashSet<InterpFlags>,
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

/// This is a Tcl script interpreter written to mimic Tcl_Eval from the final
/// TCL 7.6 distribution.  Line numbers references are to that version.
///
/// Naturally, the code differs in a number of ways.  Most particularly,
/// Rust can use "?" syntax to return a Result from the middle of
/// a routine; interp->setResult generally isn't needed.
///
/// ## TCL 7.6 line references
///
/// For convenience, many places in the code are marked with the matching line numbers
/// from the TCL 7.6 code base.  The references look like this:
///
/// * `tclBasic:1234` - This code is equivalent to that.
/// * `FLAG tclBasic:1234` - There's something in the TCL source that's been skipped for now,
///   but might important to complete the initial implementation.
/// * `SKIP tclBasic:1234 - feature...` - There's code here to support a feature we aren't
///   including for now, but might want later.
impl Interp {
    pub fn evalx(&mut self, script: &str) -> InterpResult {
        // FIRST, initialize parsing state.
        // tclBasic.c:1153
        let src = &mut script.chars().peekable();
        let flags = self.flags.clone();
        self.flags.clear();
        let mut result = String::new();

        // NEXT, determine the script terminator.  In TCL 7.6 this is a char, either
        // ']' or NUL.  Here, it's a result from peek().
        let terminator = if self.flags.contains(&BracketTerm) {
            Some(&']')
        } else {
            None
        };

        // NEXT, check depth of nested calls to eval().  If this gets too large,
        // it's probably because of an infinite loop somewhere.
        self.num_levels += 1;

        if self.num_levels > self.max_nesting_depth {
            self.num_levels -= 1;
            // FLAG tclBasic.c:1193: save termPtr
            return Err("too many nested calls to gcl::eval (infinite loop?)".into());
        }

        // NEXT, there can be many sub-commands (separated by semi-colons or
        // newlines) in one command string.  This outer loop iterates over
        // individual commands.
        // tclBasic.c:1203
        while src.peek() != terminator {
            // SKIP tclBasic:1210 - handle deleted interpreter.

            // FLAG tclBasic:1219 - prepare to handle error results and gather stack trace

            // NEXT, skim off leading white space and semi-colons, and skip comments.
            // tclBasic.c:1226
            while let Some(c) = src.peek() {
                if !is_tcl_space(*c) && *c != ';' && *c != '\n' {
                    break;
                } else {
                    src.next();
                }
            }

            if src.peek() == Some(&'#') {
                while let Some(c) = src.peek() {
                    if *c == '\\' {
                        // TODO tclBasic.c:1238 Handle escapes in comment
                    } else if *c == '\n' {
                        src.next();
                        // FLAG tclBasic.c:1242 - Update termPtr
                        break;
                    } else {
                        src.next();
                    }
                }

                continue;
            }

            // NEXT, we now have the start of an actual command.
            // FLAG: tclBasic.c:1250 - save cmdStart

            // NEXT, parse the words of the command, generating the argv for the
            // command procedure.  This code is extremely complicated in the C source;
            // it's using a C array and extended it dynamically as needed until there's
            // room for all of the words.  We'll just use a Vec.
            //
            // tclBasic.c: 1259

            let mut argv: Vec<String> = Vec::new();
            // NOTE: In the Tcl code, on non-OK result calls "goto done".
            // Not sure what args are needed
            if let Err(msg) = self.parse_words(src, &mut argv) {
                self.num_levels -= 1;
                return Err(msg);
            }

            // NEXT, if this is an empty command (or if we're just parsing commands
            // without evaluating them), then just skip to the next command.
            // WHD: I imagine NoEval is used for "info complete".
            // tclBasic.c:1325
            if argv.is_empty() || self.flags.contains(&NoEval) {
                continue;
            }

            // NEXT, save information for the history module, if needed.
            // SKIP tclBasic.c:1334 - TCL_RECORD_BOUNDS, don't think we need this.

            // NEXT, look up the command in the command table
            // SKIP tclBasic.c:1363 - command traces

            // NEXT At long last, invoke the command procedure.
            // tclBasic.c:1389
            // SKIP support for async handlers
            let words: Vec<&str> = argv.iter().map(|s| &**s).collect();

            if let Some(cmd) = self.commands.get(words[0]) {
                let cmd = Rc::clone(cmd);
                match cmd.execute(self, words.as_slice()) {
                    Ok(val) => {
                        result = val;
                    }
                    Err(msg) => {
                        self.num_levels -= 1;
                        return Err(msg);
                    }
                }
            } else {
                self.num_levels -= 1;
                return Err(format!("invalid command name \"{}\"", words[0]));
            }
        }

        // tclBasic.c:1407
        //
        // Does a variety of things, including decrementing the call level,
        // cleaning up memory, handling the special Tcl result codes, and
        // saving TCL stack trace info.
        // As I haven't yet implemented the "return", "continue", or "break"
        // commands we have no other return codes; and errors have already
        // been handled as much as they are going to be.
        self.num_levels -= 1;
        Ok(result)
    }

    /// This method parses one or more words from a command string, filling in a
    /// vector with fully-substituted copies of those words.
    /// tclParse.c:607
    fn parse_words(&mut self, src: &mut std::iter::Peekable<std::str::Chars<'_>>,
        argv: &mut Vec<String>)
    -> InterpResult
    {
        // TODO
        Ok("".into())
    }
}

/// Is the character a TCL whitespace character?  (Effectively,
/// any whitespace but CR and LF).
///
/// `CHAR_TYPE(c) == TCL_SPACE`
fn is_tcl_space(c: char) -> bool {
    c.is_whitespace() && c != '\n' && c != '\r'
}
