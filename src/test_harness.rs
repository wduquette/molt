//! Test Harness

use crate::types::*;
use crate::check_args;
use crate::interp::Interp;
use std::cell::RefCell;
use std::rc::Rc;
use std::fs;

/// Executes the Molt test harness, given the arguments.
///
/// The
pub fn test_harness(interp: &mut Interp, args: &[&str]) {
    // FIRST, announce who we are.
    println!("Molt {} -- Test Harness", env!("CARGO_PKG_VERSION"));

    // NEXT, get the script
    if args.is_empty() {
        eprintln!("missing test script");
        return;
    }

    let file_name = args[0];

    // NEXT, initialize the test result.
    let context = Rc::new(RefCell::new(TestContext::new()));

    // NEXT, install the test commands into the interpreter.
    interp.add_command_obj("test", Rc::new(TestCommand::new(&context)));

    // NEXT, execute the script.
    match fs::read_to_string(file_name) {
        Ok(script) => {
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
    let ctx = context.borrow();
    println!("\n{} tests, {} passed, {} failed, {} errors",
        ctx.num_tests, ctx.num_passed, ctx.num_failed, ctx.num_errors);
}

struct TestContext {
    num_tests: usize,
    num_passed: usize,
    num_failed: usize,
    num_errors: usize,
}

impl TestContext {
    fn new() -> Self {
        Self {
            num_tests: 0,
            num_passed: 0,
            num_failed: 0,
            num_errors: 0,
        }
    }
}

/// # test *name* *script* -ok|-error *result*
///
/// Executes the script expecting either a successful response or an error.
///
/// Note: This is an extremely minimal replacement for tcltest; at some
/// point I'll need something much more robust.
struct TestCommand {
    ctx: Rc<RefCell<TestContext>>,
}

impl TestCommand {
    fn new(ctx: &Rc<RefCell<TestContext>>) -> Self {
        Self {
            ctx: Rc::clone(ctx),
        }
    }
}

impl Command for TestCommand {
    fn execute(&self, interp: &mut Interp, argv: &[&str]) -> InterpResult {
        check_args(1, argv, 6, 6, "name description script -ok|-error result")?;

        // FIRST, get the arguments
        let name = argv[1];
        let description = argv[2];
        let script = argv[3];
        let code = argv[4];
        let output = argv[5];

        if code != "-ok" && code != "-error" {
            return molt_err!("unknown option: \"{}\"", code);
        }

        // NEXT, get the test context
        let mut ctx = self.ctx.borrow_mut();

        // NEXT, here's a test.
        ctx.num_tests += 1;

        match interp.eval(script) {
            Ok(out) => {
                if code == "-ok" && out == output {
                    // println!("*** test {} passed.", name);
                    ctx.num_passed += 1;
                } else {
                    ctx.num_failed += 1;
                    println!("\n*** FAILED {} {}", name, description);
                    println!("Expected <{}>", output);
                    println!("Received <{}>", out);
                }
            }
            Err(ResultCode::Error(out)) => {
                if code == "-error" && out == output {
                    // println!("*** test {} passed.", name);
                    ctx.num_passed += 1;
                } else {
                    ctx.num_failed += 1;
                    println!("\n*** FAILED {} {}", name, description);
                    println!("Expected <{}>", output);
                    println!("Received <{}>", out);
                }
            }
            Err(result) => {
                ctx.num_errors += 1;
                println!("test {} failed, unexpected result:\n{:?}", name, result);
            }
        }

        molt_ok!()
    }
}
