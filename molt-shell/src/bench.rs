use molt::Interp;
use molt::MoltResult;
use molt::ResultCode;
use molt::Command;
use molt::Value;
use std::cell::RefCell;
use std::rc::Rc;
use std::fs;
use std::path::PathBuf;
use std::env;

/// Executes the Molt benchmark harness, given the command-line arguments,
/// in the context of the given interpreter.
///
/// The first element of the `args` array must be the name of the benchmark script
/// to execute.  The remaining elements are meant to be harness options,
/// but are currently ignored.
pub fn benchmark(interp: &mut Interp, args: &[String]) {
    // FIRST, announce who we are.
    println!("Molt {} -- Benchmark", env!("CARGO_PKG_VERSION"));

    // NEXT, get the script file name
    if args.is_empty() {
        eprintln!("missing benchmark script");
        return;
    }

    let path = PathBuf::from(&args[0]);
    let parent = path.parent();

    // NEXT, initialize the benchmark context.
    // let context = Rc::new(RefCell::new(BenchContext::new()));

    // NEXT, install the test commands into the interpreter.
    // interp.add_command_object("test", Rc::new(TestCommand::new(&context)));

    // NEXT, load the benchmark Tcl library
    if let Err(ResultCode::Error(value)) = interp.eval(include_str!("bench.tcl")) {
        panic!("Error in benchmark Tcl library: {}", &*value.as_string());
    }

    // NEXT, execute the script.
    match fs::read_to_string(&args[0]) {
        Ok(script) => {
            if parent.is_some() {
                let _ = env::set_current_dir(parent.unwrap());
            }
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
        Err(e) => println!("{}", e),
    }

    // NEXT, output the test results:
    // let ctx = context.borrow();
    // println!("\n{} tests, {} passed, {} failed, {} errors",
    //     ctx.num_tests, ctx.num_passed, ctx.num_failed, ctx.num_errors);
}
