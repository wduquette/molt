use molt::interp::Interp;
use molt::types::InterpResult;
use molt::types::ResultCode;
use std::env;
use std::fs;

fn main() {
    // FIRST, get the command line arguments.
    let args: Vec<String> = env::args().collect();
    let args: Vec<&str> = args.iter().map(|x| &**x).collect();
    
    // NEXT, create and initialize the interpreter.
    let mut interp = Interp::new();
    interp.add_command("ident", cmd_ident);
    interp.add_command("dump", cmd_dump);

    // NEXT, if there's at least one (other than the binary name), then it's a script.
    // TODO: capture the remaining arguments and make 'arg0' and 'argv' available.
    if args.len() > 1 {
        let filename = &args[1];

        match fs::read_to_string(filename) {
            Ok(script) => execute_script(&mut interp, script, &args),
            Err(e) => println!("{}", e),
        }
    } else {
        // Just run the interactive shell.
        // TODO: should be `molt::shell()`
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

/// Command used for dev testing.  It's temporary.
fn cmd_ident(_interp: &mut Interp, argv: &[&str]) -> InterpResult {
    molt::check_args(1, argv, 2, 2, "value")?;

    Ok(argv[1].into())
}

/// Command used for dev testing.  It's temporary.
fn cmd_dump(_interp: &mut Interp, argv: &[&str]) -> InterpResult {
    molt::check_args(1, argv, 2, 2, "list")?;

    let vec = molt::get_list(&argv[1])?;

    println!("dump list:");
    for item in vec {
        println!("item <{}>", item);
    }

    molt::okay()
}
