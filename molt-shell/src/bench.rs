use molt::ContextID;
use molt::check_args;
use molt::Interp;
use molt::MoltResult;
use molt::ResultCode;
use molt::Command;
use molt::molt_ok;
use molt::Value;
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
    let context_id = interp.save_context(Context::new());

    // NEXT, install the test commands into the interpreter.
    interp.add_command("ident", cmd_ident);
    interp.add_command_object("measure", MeasureCommand::new(context_id));
    interp.add_command("ok", cmd_ok);

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
    let ctx = interp.context::<Context>(context_id);

    println!("{:>8} {:>8} -- Benchmark", "Micros", "Norm");

    let baseline = ctx.baseline.unwrap_or(1.0);

    for record in &ctx.measurements {
        println!("{:>8.2} {:>8.2} -- {} {}",
            record.micros,
            record.micros / baseline,
            record.name,
            record.description);
    }
}

struct Context {
    // The baseline, in microseconds
    baseline: Option<f64>,

    // The list of measurements.
    measurements: Vec<Measurement>,
}

impl Context {
    fn new() -> Self {
        Self {
            baseline: None,
            measurements: Vec::new()
        }
    }
}

struct Measurement {
    // The measurement's symbolic name
    name: String,

    // The measurement's human-readable description
    description: String,

    // The average number of microseconds per measured iteration
    micros: f64,
}

/// # measure *name* *description* *micros*
///
/// Records a benchmark measurement.
struct MeasureCommand {
    ctx_id: ContextID,
}

impl MeasureCommand {
    fn new(ctx_id: ContextID) -> Self {
        Self { ctx_id }
    }
}

impl Command for MeasureCommand {
    fn execute(&self, interp: &mut Interp, argv: &[Value]) -> MoltResult {
        molt::check_args(1, argv, 4, 4, "name description micros")?;

        // FIRST, get the arguments
        let name = argv[1].to_string();
        let description = argv[2].to_string();
        let micros = argv[3].as_float()?;

        // NEXT, get the test context
        let ctx = interp.context::<Context>(self.ctx_id);

        if ctx.baseline.is_none() {
            ctx.baseline = Some(micros);
        }

        let record = Measurement {
            name,
            description,
            micros,
        };

        ctx.measurements.push(record);

        molt_ok!()
    }
}

/// # ident value
///
/// Returns its argument.
fn cmd_ident(_interp: &mut Interp, argv: &[Value]) -> MoltResult {
    check_args(1, argv, 2, 2, "value")?;

    molt_ok!(argv[1].clone())
}


/// # ok ...
///
/// Takes any number of arguments, and returns "".
fn cmd_ok(_interp: &mut Interp, _argv: &[Value]) -> MoltResult {
    molt_ok!()
}
