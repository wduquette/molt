//! # Standard Molt Command Definitions
//!
//! This module defines the standard Molt commands.

use crate::interp::Interp;
use crate::okay;
use crate::types::*;
use crate::*;

/// # exit ?*returnCode*?
///
/// Terminates the application by calling `std::process::exit()`.
/// If given, _returnCode_ must be an integer return code; if absent, it
/// defaults to 0.
pub fn cmd_exit(_interp: &mut Interp, argv: &[&str]) -> InterpResult {
    check_args(1, argv, 1, 2, "?returnCode?")?;

    let return_code: MoltInteger = if argv.len() == 1 {
        0
    } else {
        get_integer(argv[1])?
    };

    std::process::exit(return_code)
}

pub struct Subcommand(&'static str, CommandFunc);

pub fn unknown_subcommand_error(subs: &[Subcommand], sub: &str) -> InterpResult {
    let mut names = String::new();
    names.push_str(subs[0].0);
    let last = subs.len() - 1;

    if subs.len() > 1 {
        names.push_str(", ");
    }

    if subs.len() > 2 {
        let vec: Vec<&str> = subs[1..last].iter().map(|x| x.0).collect();
        names.push_str(&vec.join(", "));
    }

    if subs.len() > 1 {
        names.push_str(", or ");
        names.push_str(subs[last].0);
    }

    error(&format!("unknown or ambiguous subcommand \"{}\": must be {}", sub, &names))
}

const INFO_SUBCOMMANDS: [Subcommand; 3]  = [
    Subcommand("commands", cmd_info_commands),
    Subcommand("complete", cmd_info_complete),
    Subcommand("vars", cmd_info_vars),
];


/// # info *subcommand* ?*arg*...?
pub fn cmd_info(interp: &mut Interp, argv: &[&str]) -> InterpResult {
    check_args(1, argv, 2, 0, "subcommand ?arg ...?")?;

    for Subcommand(name, func) in &INFO_SUBCOMMANDS {
        if name == &argv[1] {
            return func(interp, argv);
        }
    }

    unknown_subcommand_error(&INFO_SUBCOMMANDS, &argv[1])
}

/// # info commands ?*pattern*?
pub fn cmd_info_commands(_interp: &mut Interp, _argv: &[&str]) -> InterpResult {
    error("TODO")
}

/// # info complete *command*
pub fn cmd_info_complete(interp: &mut Interp, argv: &[&str]) -> InterpResult {
    check_args(2, argv, 3, 3, "command")?;

    // TODO: Add way of returning a boolean result.
    if interp.complete(argv[2]) {
        Ok("1".into())
    } else {
        Ok("0".into())
    }
}

/// # info vars ?*pattern*?
pub fn cmd_info_vars(_interp: &mut Interp, _argv: &[&str]) -> InterpResult {
    error("TODO")
}

/// # puts *string*
///
/// Outputs the string to stdout.
///
/// ## TCL Liens
///
/// * Does not support `-nonewline`
/// * Does not support `channelId`
pub fn cmd_puts(_interp: &mut Interp, argv: &[&str]) -> InterpResult {
    check_args(1, argv, 2, 2, "string")?;

    println!("{}", argv[1]);
    okay()
}

/// # set *varName* ?*newValue*?
///
/// Sets variable *varName* to *newValue*, returning the value.
/// If *newValue* is omitted, returns the variable's current value,
/// returning an error if the variable is unknown.
///
/// ## TCL Liens
///
/// * Does not support arrays
pub fn cmd_set(interp: &mut Interp, argv: &[&str]) -> InterpResult {
    check_args(1, argv, 2, 3, "varName ?newValue?")?;

    let value;

    if argv.len() == 3 {
        value = argv[2].into();
        interp.set_var(argv[1], argv[2]);
    } else {
        value = interp.get_var(argv[1])?;
    }

    Ok(value)
}

/// # test *name* *script* -ok|-error *result*
///
/// Executes the script expecting either a successful response or an error.
///
/// Note: This is an extremely minimal replacement for tcltest; at some
/// point I'll need something much more robust.
pub fn cmd_test(interp: &mut Interp, argv: &[&str]) -> InterpResult {
    check_args(1, argv, 5, 5, "name script -ok|-error result")?;

    let name = argv[1];
    let script = argv[2];
    let code = argv[3];
    let output = argv[4];

    match interp.eval(script) {
        Ok(out) => {
            if code == "-ok" && out == output {
                println!("*** test {} passed.", name);
            } else {
                println!("*** test {} FAILED.", name);
                println!("Expected <{}>", output);
                println!("Received <{}>", out);
            }
        },
        Err(ResultCode::Error(out)) => {
            if code == "-error" && out == output {
                println!("*** test {} passed.", name);
            } else {
                println!("*** test {} FAILED.", name);
                println!("Expected <{}>", output);
                println!("Received <{}>", out);
            }
        }
        Err(result) => {
            println!("test {} failed, unexpected result:\n{:?}",
                name, result);
        }
    }

    okay()
}
