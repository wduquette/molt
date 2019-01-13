use crate::types::ResultCode::Normal;
use crate::interp::Interp;
use crate::types::InterpResult;
use crate::utils;

pub fn cmd_exit(interp: &mut Interp, argv: &[&str]) -> InterpResult {
    utils::check_args(interp, argv, 1, 1, "")?;

    // TODO: Allow an optional argument, and parse it to i32.
    std::process::exit(0);
}

pub fn cmd_puts(interp: &mut Interp, argv: &[&str]) -> InterpResult {
    utils::check_args(interp, argv, 2, 2, "text")?;

    println!("{}", argv[1]);

    Ok(Normal)
}
