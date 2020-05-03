# Custom Shells

A custom Molt shell is simply an application that:

* Creates a Molt `interp`
* Adds any desired commands by the methods described in the previous section
* Passes the `interp` to `molt_shell::repl` (for an interactive shell)
* Passes the `interp` and a file to `molt_shell::script`

The [sample Molt application](https://github.com/wduquette/molt-sample) provides a full
example; here's a sketch:

```
fn main() {
    use std::env;

    // FIRST, get the command line arguments.
    let args: Vec<String> = env::args().collect();

    // NEXT, create and initialize the interpreter.
    let mut interp = Interp::new();

    // NOTE: commands can be added to the interpreter here, e.g.,

    // Add a single module
    interp.add_command("hello", cmd_hello);

    // Install a Molt extension crate
    molt_sample::install(&mut interp).expect("Could not install.");

    // NEXT, evaluate the file, if any.
    if args.len() > 1 {
        molt_shell::script(&mut interp, &args[1..]);
    } else {
        molt_shell::repl(&mut interp);
    }
}

pub fn cmd_hello(_interp: &mut Interp,  _: ContextID, argv: &[Value]) -> MoltResult {
    // Correct number of arguments?
    check_args(1, argv, 2, 2, "name")?;

    println!("Hello, {}", argv[1].as_str());
    molt_ok!()
}
```
