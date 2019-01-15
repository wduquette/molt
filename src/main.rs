use gcl::interp::Interp;
use gcl::types::InterpResult;

fn main() {
    let mut interp = Interp::new();
    interp.add_command("ident", cmd_ident);

    gcl::shell::shell(&mut interp, "% ");
}

fn cmd_ident(_interp: &mut Interp, argv: &[&str]) -> InterpResult {
    gcl::utils::check_args(argv, 2, 2, "value")?;

    Ok(argv[1].into())
}
