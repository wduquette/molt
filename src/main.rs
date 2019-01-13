use gcl::interp::Interp;
use gcl::types::Status;

fn main() {
    let mut interp = Interp::new();
    interp.add_command("ident", cmd_ident);

    gcl::shell::shell(&mut interp, "% ");
}

fn cmd_ident(_interp: &mut Interp, argv: &[&str]) -> Status {
    let result = gcl::utils::check_args(argv, 2, 2, "value");

    if !result.is_okay() {
        return result;
    }

    Status::result(argv[1])
}
