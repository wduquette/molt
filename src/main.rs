use molt::interp::Interp;
use molt::types::InterpResult;

fn main() {
    let mut interp = Interp::new();
    interp.add_command("ident", cmd_ident);

    molt::shell::shell(&mut interp, "% ");
}

fn cmd_ident(_interp: &mut Interp, argv: &[&str]) -> InterpResult {
    molt::check_args(argv, 2, 2, "value")?;

    Ok(argv[1].into())
}
