//! # Standard GCL Command Definitions
//!
//! This module defines the standard GCL commands.

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
    check_args(argv, 1, 2, "?returnCode?")?;

    let return_code: GclInteger = if argv.len() == 1 {
        0
    } else {
        get_integer(argv[1])?
    };

    std::process::exit(return_code)
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
    check_args(argv, 2, 2, "string")?;

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
    check_args(argv, 2, 3, "varName ?newValue?")?;

    let value;

    if argv.len() == 3 {
        value = argv[2].into();
        interp.set_var(argv[1], argv[2]);
    } else {
        value = interp.get_var(argv[1])?;
    }

    Ok(value)
}
