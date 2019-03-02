use molt::Interp;
use molt::ResultCode;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::fs;

/// Invokes an interactive REPL for the given `Interp`, using `rustlyline` line editing.
/// Enter `exit` or `^C` to exit.
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
                            if !value.is_empty() {
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

/// Executes a script from a set of command line arguments.  `args[0]` is presumed to be
/// the name of a Molt script file, with any subsequent arguments being arguments to
/// pass to the script.  The script will be be executed in the context of the
/// the given `Interp`.
///
/// # Argument Variables
///
/// The Molt variable `arg0` will be set to the name of the script file, `args[0]` value.
/// The Molt variable `argv` will be set to the remainder of the `args` array as a Molt list.
pub fn script(interp: &mut Interp, args: &[String]) {
    let arg0 = &args[0];
    let argv = &args[1..];
    match fs::read_to_string(&args[0]) {
        Ok(script) => execute_script(interp, script, arg0, argv),
        Err(e) => println!("{}", e),
    }
}

/// Executes a script read from a file, with any command-line arguments, in
/// the context of the given `Interp`.  The `script` is the text of the
/// script, and the `args` are the script arguments, with `args[0]` being
/// the name of the script.
///
/// # Argument Variables
///
/// The Molt variable `arg0` will be set to the `arg0` value.
/// The Molt variable `argv` will be set to the `argv` array as a Molt list.
fn execute_script(interp: &mut Interp, script: String, arg0: &str, argv: &[String]) {
    let argv = molt::list_to_string(argv);

    interp.set_var("arg0", arg0);
    interp.set_var("argv", &argv);

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
