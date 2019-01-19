use crate::interp::Interp;
use crate::okay;
use crate::types::*;
use crate::utils;

pub fn cmd_exit(_interp: &mut Interp, argv: &[&str]) -> InterpResult {
    utils::check_args(argv, 1, 1, "")?;

    // TODO: Allow an optional argument, and parse it to i32.
    std::process::exit(0)
}

pub fn cmd_puts(_interp: &mut Interp, argv: &[&str]) -> InterpResult {
    utils::check_args(argv, 2, 2, "text")?;

    println!("{}", argv[1]);
    okay()
}

pub fn cmd_set(interp: &mut Interp, argv: &[&str]) -> InterpResult {
    utils::check_args(argv, 2, 3, "varName ?newValue?")?;

    let value;

    if argv.len() == 3 {
        value = argv[2].into();
        interp.set_var(argv[1], argv[2]);
    } else {
        value = interp.get_var(argv[1])?;
    }

    Ok(value)
}
