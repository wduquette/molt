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
    interp.add_command_object("test", TestCommand::new(&context));

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

#[derive(Eq,PartialEq,Debug)]
enum Code {
    Ok,
    Error
}

impl Code {
    fn to_string(&self) -> String {
        match self {
            Code::Ok => "-ok".into(),
            Code::Error => "-error".into(),
        }
    }
}

#[derive(Debug)]
struct TestInfo {
    name: String,
    description: String,
    setup: String,
    body: String,
    cleanup: String,
    code: Code,
    expect: String,
}

impl TestInfo {
    fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            setup: String::new(),
            body: String::new(),
            cleanup: String::new(),
            code: Code::Ok,
            expect: String::new(),
        }
    }

    fn print_failure(&self, got_code: &str, received: &str) {
        println!("\n*** FAILED {} {}", self.name, self.description);
        println!("Expected {} <{}>", self.code.to_string(), self.expect);
        println!("Received {} <{}>", got_code, received);
    }

    fn print_error(&self, result: &MoltResult) {
        println!("\n*** ERROR {} {}", self.name, self.description);
        println!("Expected {} <{}>", self.code.to_string(), self.expect);
        match result {
            Ok(val) => println!("Received -ok <{}>", val),
            Err(ResultCode::Error(msg)) => println!("Received -error <{}>", msg),
            Err(ResultCode::Return(val)) => println!("Received -return <{}>", val),
            Err(ResultCode::Break) => println!("Received -break <>"),
            Err(ResultCode::Continue) => println!("Received -continue <>"),
        }
    }

    fn print_helper_error(&self, part: &str, msg: &str) {
        println!("\n*** ERROR (in {}) {} {}", part, self.name, self.description);
        println!("    {}", msg);
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

    fn fancy_test(&self, interp: &mut Interp, argv: &[Value]) -> MoltResult {
        molt::check_args(1, argv, 4, 0, "name description option value ?option value...?")?;

        // FIRST, get the test context
        let mut ctx = self.ctx.borrow_mut();

        // NEXT, get the tes tinfo
        let mut info = TestInfo::new(&*argv[1].as_string(), &*argv[2].as_string());
        let mut iter = argv[3..].iter();
        loop {
            let opt = iter.next();
            if opt.is_none() {
                break;
            }
            let opt = &*opt.unwrap().as_string();

            let val = iter.next();
            if val.is_none() {
                ctx.num_errors += 1;
                info.print_helper_error("test command",
                    &format!("missing value for {}", opt));
                return molt_ok!();
            }
            let val = &*val.unwrap().as_string();

            match opt.as_ref() {
                "-setup" => info.setup = val.to_string(),
                "-body" => info.body = val.to_string(),
                "-cleanup" => info.cleanup = val.to_string(),
                "-ok" => {
                    info.code = Code::Ok;
                    info.expect = val.to_string();
                }
                "-error" => {
                    info.code = Code::Error;
                    info.expect = val.to_string();
                }
                _ => {
                    ctx.num_errors += 1;
                    info.print_helper_error("test command",
                        &format!("invalid option: \"{}\"", val));
                    return molt_ok!();
                }
            }
        }

        self.run_test(interp, &mut ctx, &info);

        molt_ok!()
    }

    fn simple_test(&self, interp: &mut Interp, argv: &[Value]) -> MoltResult {
        molt::check_args(1, argv, 6, 6, "name description script -ok|-error result")?;

        // FIRST, get the test context
        let mut ctx = self.ctx.borrow_mut();

        // NEXT, get the test info
        let mut info = TestInfo::new(&*argv[1].as_string(), &*argv[2].as_string());
        info.body = argv[3].to_string();
        info.expect = argv[5].to_string();

        let code = &*argv[4].as_string();

        info.code = if code == "-ok" {
            Code::Ok
        } else if code == "-error" {
            Code::Error
        } else {
            ctx.num_errors += 1;
            info.print_helper_error("test command",
                &format!("invalid option: \"{}\"", code));

            return molt_ok!();
        };

        self.run_test(interp, &mut ctx, &info);
        molt_ok!()
    }

    fn run_test(&self, interp: &mut Interp, ctx: &mut TestContext, info: &TestInfo) {
        // NEXT, here's a test.
        ctx.num_tests += 1;

        // NEXT, push a variable scope; -setup, -body, and -cleanup will share it.
        interp.push_scope();

        // Setup
        if let Err(ResultCode::Error(msg)) = interp.eval(&info.setup) {
            info.print_helper_error("-setup", &msg.to_string());
        }

        // Body
        let result = interp.eval_body(&info.body);

        // Cleanup
        if let Err(ResultCode::Error(msg)) = interp.eval(&info.cleanup) {
            info.print_helper_error("-cleanup", &msg.to_string());
        }

        // NEXT, pop the scope.
        interp.pop_scope();

        match &result {
            Ok(out) => {
                if info.code == Code::Ok {
                    if *out == Value::from(&info.expect) {
                        ctx.num_passed += 1;
                    } else {
                        ctx.num_failed += 1;
                        info.print_failure("-ok", &out.to_string());
                    }
                    return;
                }
            }
            Err(ResultCode::Error(out)) => {
                if info.code == Code::Error {
                    if *out == Value::from(&info.expect) {
                        ctx.num_passed += 1;
                    } else {
                        ctx.num_failed += 1;
                        info.print_failure("-error", &out.to_string());
                    }
                    return;
                }
            }
            _ => ()
        }
        ctx.num_errors += 1;
        info.print_error(&result);
    }
}

impl Command for TestCommand {
    fn execute(&self, interp: &mut Interp, argv: &[Value]) -> MoltResult {
        // FIRST, check the minimum command line.
        molt::check_args(1, argv, 4, 0, "name description args...")?;

        // NEXT, see which kind of command it is.
        let arg = &*argv[3].as_string();
        if arg.starts_with('-') {
            self.fancy_test(interp, argv)
        } else {
            self.simple_test(interp, argv)
        }
    }
}
