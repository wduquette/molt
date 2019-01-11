use crate::Interp;
use crate::InterpResult;
use crate::utils;

pub fn cmd_exit(_interp: &mut Interp, argv: &[&str]) -> InterpResult {
    utils::check_args(argv, 1, 1, "")?;

    // TODO: Allow an optional argument, and parse it to i32.
    std::process::exit(0);
}

pub fn cmd_puts(_interp: &mut Interp, argv: &[&str]) -> InterpResult {
    utils::check_args(argv, 2, 2, "text")?;

    println!("{}", argv[1]);

    Ok("".into())
}
