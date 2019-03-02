use molt::Interp;
use molt::ResultCode;

pub fn execute_script(interp: &mut Interp, script: String, args: &[&str]) {
    let arg0 = &args[2];
    let argv = if args.len() > 3 {
        molt::list_to_string(&args[3..])
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
