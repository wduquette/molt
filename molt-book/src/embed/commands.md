# Defining Commands

At base, a Molt command is a Rust function that performs some kind of work and optionally
returns a value in the context of a specific Rust interpreter.  There are four ways an
application (or library crate) can define application-specific Rust commands:

* As a simple Rust `CommandFunc` function
* As a Rust `ContextCommandFunc` function that can access application-specific context
* As a Rust `Command` object
* As a Molt procedure, or `proc`.

## `CommandFunc` Commands

A `CommandFunc` command is any Rust function that implements `CommandFunc`:

```rust
pub type CommandFunc = fn(&mut Interp, &[Value]) -> MoltResult;
```

For example, here's a simple command that takes one argument and returns it
unchanged.

```rust
fn cmd_ident(_interp: &mut Interp, argv: &[Value]) -> MoltResult {
    check_args(1, argv, 2, 2, "value")?;

    molt_ok!(argv[1].clone())
}
```

The `argv` vector contains the arguments to the command, beginning with the
command's name.  The `check_args` method verifies that the command has a write
number of arguments, and returns the standard Tcl error message if not.  Finally,
it uses `molt_ok!` to return its first argument.

Install this command into the interpreter using the `Interp::add_command` method:

```rust
interp.add_command("ident", cmd_ident);
```

## `ContextCommandFunc` Commands

Normal `CommandFunc` commands are useful when extending the Molt language itself, but don't
help much when adding commands to manipulate the application and its data.  In this case,
it's often best to use a `ContextCommandFunc` in conjunction with the interpreter's
_context cache_.

The context cache is a hash map that allows the interpreter to keep arbitrary data and make
it available to commands. The usual pattern is like this:

* The application defines a type containing the data the command (or commands) requires.
  We'll call it `AppContext` for the purpose of this example.

* The application saves an instance of `AppContext` into the context cache, retrieving a
  `ContextID`.

* The application includes the `ContextID` when adding the command to the interpreter.

* The command retrieves the `AppContext` as a mutable borrow.

```rust
// The AppContext
struct AppContext { text: String }

// The Command
fn cmd_whatsit(interp: &mut Interp, context_id: ContextID, argv: &[Value]) -> MoltResult {
    check_args(1, argv, 2, 2, "value")?;

    let ctx = interp.context::<AppContext>(context_id);

    // Save the first argument to the AppContext's text field.
    ctx.text.push_str(&*argv[1].as_string());

    molt_ok!()
}

// Registering the command
fn main() {
    let interp = Interp::new();
    let id = interp.save_context(AppContext::new());

    interp.add_context_command("whatsit", cmd_whatsit, id);

    ...
}
```

## `Command` Objects

Sometimes it's desirable to include more data as part of the command itself.  In this
case one can define a type that implements the `Command` trait, and register an instance
with the interp.

```rust
struct MyCommand {
    b: RefCell<String>,
}

impl Command for MyCommand {
    fn execute(&self, interp: &mut Interp, argv: &[Value]) -> MoltResult {
        ...
    }
}
```

The instance is added to the interpreter using the `add_command_object` method:

```rust
interp.add_command_object("my_command", MyCommand::new());
```

This allows many additional patterns of use.  For example, objects in Tcl are often
represented as commands with associated data.  This can be handled using `Command` objects,
optionally in conjunction with the context cache:

*   Command `myclass` creates a new instance, a Tcl command with its own context record
    in the context cache.
*   The new instance can read and write from its instance data as needed.
*   The code of the "class" is defined by a struct that implements the `Command` trait.
*   The struct should also implement the `Drop` trait to release the context record
    when the object goes away.

TODO: We really need a bigger discussion of how to define Molt objects in Rust.

## Molt Procedures

A Molt procedure is a routine coded in Tcl and defined using the `proc` command. A
crate can compile Tcl procedures into itself using the `include_str!` macro.  Start
by defining a script that defines the required procedure, say, `procs.tcl`, and put it
in the crate's `src/` folder adjacent to the Rust file that will load it.  The Rust
file can then do this:

```rust
let mut interp = Interp::new();

match interp.eval(include_str!("commands.tcl")) {
    Err(ResultCode::Error(msg)) => {
        panic!("Couldn't load procs.tcl: {}", msg);
    }
    _ => ()
}
```
