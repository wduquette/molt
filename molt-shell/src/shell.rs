use molt::Interp;
use molt::MoltList;
use molt::ResultCode;
use molt::Value;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::fs;

/// Invokes an interactive REPL for the given interpreter, using `rustyline` line editing.
///
/// The REPL will display the given prompt to the user.  Press `^C` to terminate
/// the REPL, returning control to the caller.  Entering `exit` will also normallycause the
/// application to terminate (but the `exit` command can be removed or redefined by the
/// application).
///
/// See [`molt::interp`](../molt/interp/index.html) for details on how to configure and
/// add commands to a Molt interpreter.
///
/// # Example
///
/// ```
/// use molt::Interp;
///
/// // FIRST, create and initialize the interpreter.
/// let mut interp = Interp::new();
///
/// // NOTE: commands can be added to the interpreter here.
///
/// // NEXT, invoke the REPL.
/// molt_shell::repl(&mut interp, "% ");
/// ```
pub fn repl(interp: &mut Interp, prompt: &str) {
    let mut rl = Editor::<()>::new();

    loop {
        let readline = rl.readline(prompt);
        match readline {
            Ok(line) => {
                let line = line.trim();
                if !line.is_empty() {
                    match interp.eval(line) {
                        Ok(value) | Err(ResultCode::Return(value)) => {
                            rl.add_history_entry(line);

                            // Don't output empty values.
                            if !value.as_str().is_empty() {
                                println!("{}", value);
                            }
                        }
                        Err(ResultCode::Error(msg)) => {
                            println!("{}", msg);
                        }
                        result => {
                            // Must be Break or Continue, which should have been caught
                            // by eval(), so this should never happen.  But panicking would
                            // be rude.
                            println!("Unexpected eval return: {:?}", result);
                        }
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                break;
            }
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                println!("I/O Error: {:?}", err);
                break;
            }
        }
    }
}

/// Executes a script from a set of command line arguments.
///
/// `args[0]` is presumed to be the name of a Molt script file, with any subsequent
/// arguments being arguments to pass to the script.  The script will be be executed in
/// the context of the given interpreter.
///
/// # Molt Variables
///
/// The calling information will be passed to the interpreter in the form of Molt
/// variables:
///
/// * The Molt variable `arg0` will be set to the `arg0` value.
/// * The Molt variable `argv` will be set to a Molt list containing the remainder of the
///   `argv` array.
///
/// See [`molt::interp`](../molt/interp/index.html) for details on how to configure and
/// add commands to a Molt interpreter.
///
/// # Example
///
/// ```
/// use molt::Interp;
/// use std::env;
///
/// // FIRST, get the command line arguments.
/// let args: Vec<String> = env::args().collect();
///
/// // NEXT, create and initialize the interpreter.
/// let mut interp = Interp::new();
///
/// // NOTE: commands can be added to the interpreter here.
///
/// // NEXT, evaluate the file, if any.
/// if args.len() > 1 {
///     molt_shell::script(&mut interp, &args[1..]);
/// } else {
///     eprintln!("Usage: myshell *filename.tcl");
/// }
/// ```
pub fn script(interp: &mut Interp, args: &[String]) {
    let arg0 = &args[0];
    let argv = &args[1..];
    match fs::read_to_string(&args[0]) {
        Ok(script) => execute_script(interp, script, arg0, argv),
        Err(e) => println!("{}", e),
    }
}

/// Executes a script read from a file, with any command-line arguments, in
/// the context of the given interpreter.  The `script` is the text of the
/// script, `arg0` is the name of the script file, and `argv` contains the script
/// arguments.
///
/// # Molt Variables
///
/// The calling information will be passed to the interpreter in the form of Molt
/// variables:
///
/// * The Molt variable `arg0` will be set to the `arg0` value.
/// * The Molt variable `argv` will be set to the `argv` array as a Molt list.
fn execute_script(interp: &mut Interp, script: String, arg0: &str, argv: &[String]) {
    let argv: MoltList = argv.iter().map(Value::from).collect();
    interp.set_var("arg0", &Value::from(arg0));
    interp.set_var("argv", &Value::from(argv));

    match interp.eval(&script) {
        Ok(_) => (),
        Err(ResultCode::Return(_)) => (),
        Err(ResultCode::Error(msg)) => {
            eprintln!("{}", msg);
            std::process::exit(1);
        }
        result => {
            // Break or Continue; should never happen, since eval() is supposed to convert
            // these to errors.
            panic!("Unexpected eval return: {:?}", result)
        }
    }
}
