use crate::types::*;
use crate::interp::Interp;
use crate::utils;

pub fn cmd_exit(_interp: &mut Interp, argv: &[&str]) -> Status {
    let result = utils::check_args(argv, 1, 1, "");

    if !result.is_okay() {
        result
    } else {
        // TODO: Allow an optional argument, and parse it to i32.
        std::process::exit(0)
    }
}

pub fn cmd_puts(_interp: &mut Interp, argv: &[&str]) -> Status {
    let result = utils::check_args(argv, 2, 2, "text");

    if !result.is_okay() {
        return result;
    }

    println!("{}", argv[1]);
    Status::okay()
}
