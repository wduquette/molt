//! Molt Test Harness
//!
//! A Molt test script is a Molt script containing tests of Molt code.  Each
//! test is a call of the Molt `test` command provided by the
//! `molt_shell::test_harness`.  The tests are executed in the context of the
//! the application's `molt::Interp` (and so can test application-specific commands).
//!
//! The test harness keeps track of the number of tests executed, and whether they
//! passed, failed, or returned an unexpected error.
//!
//! The `molt-app` tool provides access to the test harness for a standard Molt
//! interpreter:
//!
//! ```bash
//! $ molt test test/all.tcl
//! Molt 0.1.0 -- Test Harness
//!
//! 171 tests, 171 passed, 0 failed, 0 errors
//! ```
//!
//! If a test fails or returns an error, the test harness outputs the details.
//!
//! See the Molt Book (or the Molt test suite) for examples of test scripts.

use molt::molt_ok;
use molt::molt_err;
use molt::Interp;
use molt::InterpResult;
use molt::ResultCode;
use molt::Command;
use std::cell::RefCell;
use std::rc::Rc;
use std::fs;
use std::path::PathBuf;
use std::env;

/// Executes the Molt test harness, given the command-line arguments,
/// in the context of the given interpreter.
///
/// The first element of the `args` array must be the name of the test script
/// to execute.  The remaining elements are meant to be test harness options,
/// but are currently ignored.
pub fn test_harness(interp: &mut Interp, args: &[String]) {
    // FIRST, announce who we are.
    println!("Molt {} -- Test Harness", env!("CARGO_PKG_VERSION"));

    // NEXT, get the script file name
    if args.is_empty() {
        eprintln!("missing test script");
        return;
    }

    let path = PathBuf::from(&args[0]);
    let parent = path.parent();

    // NEXT, initialize the test result.
    let context = Rc::new(RefCell::new(TestContext::new()));

    // NEXT, install the test commands into the interpreter.
    interp.add_command_obj("test", Rc::new(TestCommand::new(&context)));

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
        molt::check_args(1, argv, 6, 6, "name description script -ok|-error result")?;

        // FIRST, get the arguments
        let name = argv[1];
        let description = argv[2];
        let script = argv[3];
        let code = argv[4];
        let output = argv[5];


        if !(code == "-ok" || code == "-error" || code == "-return" || code == "-break" ||
            code == "-continue")
        {
            return molt_err!("unknown option: \"{}\"", code);
        }

        if (code == "-break" || code == "-continue") && !output.is_empty() {
            return molt_err!("non-empty result with {}", code);
        }

        // NEXT, get the test context
        let mut ctx = self.ctx.borrow_mut();

        // NEXT, here's a test.
        ctx.num_tests += 1;

        interp.push_scope();
        let result = interp.eval_body(script);
        interp.pop_scope();

        match result {
            Ok(out) => {
                if code == "-ok" && out == output {
                    // println!("*** test {} passed.", name);
                    ctx.num_passed += 1;
                } else {
                    ctx.num_failed += 1;
                    println!("\n*** FAILED {} {}", name, description);
                    println!("Expected {} <{}>", code, output);
                    println!("Received -ok <{}>", out);
                }
            }
            Err(ResultCode::Error(out)) => {
                if code == "-error" && out == output {
                    // println!("*** test {} passed.", name);
                    ctx.num_passed += 1;
                } else {
                    ctx.num_failed += 1;
                    println!("\n*** FAILED {} {}", name, description);
                    println!("Expected {} <{}>", code, output);
                    println!("Received -error <{}>", out);
                }
            }
            Err(ResultCode::Return(out)) => {
                if code == "-return" && out == output {
                    // println!("*** test {} passed.", name);
                    ctx.num_passed += 1;
                } else {
                    ctx.num_failed += 1;
                    println!("\n*** FAILED {} {}", name, description);
                    println!("Expected {} <{}>", code, output);
                    println!("Received -return <{}>", out);
                }
            }
            Err(ResultCode::Break) => {
                if code == "-break" {
                    // println!("*** test {} passed.", name);
                    ctx.num_passed += 1;
                } else {
                    ctx.num_failed += 1;
                    println!("\n*** FAILED {} {}", name, description);
                    println!("Expected {} <{}>", code, output);
                    println!("Received -break <>");
                }
            }
            Err(ResultCode::Continue) => {
                if code == "-continue" {
                    // println!("*** test {} passed.", name);
                    ctx.num_passed += 1;
                } else {
                    ctx.num_failed += 1;
                    println!("\n*** FAILED {} {}", name, description);
                    println!("Expected {} <{}>", code, output);
                    println!("Received -continue <>");
                }
            }
        }

        molt_ok!()
    }
}
