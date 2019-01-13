use gcl::interp::Interp;
use gcl::types::InterpResult;
use gcl::value::Value;
use gcl::types::ResultCode;

fn main() {
    let mut interp = Interp::new();
    interp.add_command("ident", cmd_ident);

    gcl::shell::shell(&mut interp, "% ");
}

fn cmd_ident(interp: &mut Interp, argv: &[&str]) -> InterpResult {
    gcl::utils::check_args(interp, argv, 2, 2, "value")?;

    interp.set_result(Value::from(argv[1]));

    Ok(ResultCode::Normal)
}
