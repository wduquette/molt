use molt::interp::Interp;
use molt::types::ResultCode;
use std::env;
use std::fs;

fn main() {
    // FIRST, get the command line arguments.
    let args: Vec<String> = env::args().collect();
    let args: Vec<&str> = molt::vec_string_to_str(&args);

    // NEXT, create and initialize the interpreter.
    let mut interp = Interp::new();

    // NEXT, if there's at least one then it's a subcommand.
    if args.len() > 1 {
        let subcmd: &str = &args[1];

        match subcmd {
            "shell" => {
                if args.len() == 2 {
                    // TODO: should be `molt::shell()`
                    molt::shell::shell(&mut interp, "% ");
                } else {
                    // TODO: capture the remaining arguments and make 'arg0' and 'argv' available.
                    match fs::read_to_string(&args[2]) {
                        Ok(script) => execute_script(&mut interp, script, &args),
                        Err(e) => println!("{}", e),
                    }
                }
            }
            "test" => {
                eprintln!("molt test: not yet implemented");
            }
            "help" => {
                eprintln!("molt help: not yet implemented");
            }
            _ => {
                eprintln!("unknown subcommand: \"{}\"", subcmd);
            }
        }
    } else {
        // Just run the interactive shell.
        // But really, should run "molt help"
        molt::shell::shell(&mut interp, "% ");
    }
}

fn execute_script(interp: &mut Interp, script: String, args: &[&str]) {
    let arg0 = &args[1];
    let argv = if args.len() > 2 {
        molt::list_to_string(&args[2..])
    } else {
        String::new()
    };

    interp.set_var("arg0", arg0);
    interp.set_var("argv", &argv);

    match interp.eval(&script) {
        Ok(_) => (),
        Err(ResultCode::Error(msg)) => {
            eprintln!("{}", msg);
            std::process::exit(1);
        }
        Err(result) => {
            eprintln!("Unexpected eval return: {:?}", result);
            std::process::exit(1);
        }
    }
}
