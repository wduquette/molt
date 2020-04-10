# Defining Commands

At base, a Molt command is a Rust function that performs some kind of work and optionally
returns a value in the context of a specific Rust interpreter.  There are two ways an
application (or library crate) can define application-specific Rust commands:

* As a Rust `CommandFunc` function
* As a Molt procedure, or `proc`.

## `CommandFunc` Commands

A `CommandFunc` command is any Rust function that implements `CommandFunc`:

```rust
pub type CommandFunc = fn(&mut Interp, ContextID, &[Value]) -> MoltResult;
```

For example, here's a simple command that takes one argument and returns it
unchanged.

```rust
fn cmd_ident(_interp: &mut Interp, _context_id: ContextID, argv: &[Value]) -> MoltResult {
    check_args(1, argv, 2, 2, "value")?;

    molt_ok!(argv[1].clone())
}
```

The `argv` vector contains the arguments to the command, beginning with the
command's name.  The `check_args` method verifies that the command has the right
number of arguments, and returns the standard Tcl error message if not.  Finally,
it uses `molt_ok!` to return its first argument.

Install this command into the interpreter using the `Interp::add_command` method:

```rust
interp.add_command("ident", cmd_ident);
```

## `CommandFunc` Commands with Context

A normal `CommandFunc` is useful when extending the Molt language itself; but
application-specific commands need to manipulate the application and its data.  In this case,
add the required data to the interpreter's _context cache_.  The cached data can be retrieved,
used, and mutated by commands tagged with the relevant context ID.

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

    // Append the first argument's string rep to the
    // AppContext struct's text field.
    ctx.text.push_str(argv[1].as_str());

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

The saved `AppContext` will be dropped automatically if the `whatsit` command is
removed from the interpreter.

## Commands with Shared Context

Any number of Molt commands can share a single cached context struct:

```
    let interp = Interp::new();
    let id = interp.save_context(AppContext::new());

    interp.add_context_command("first", cmd_first, id);
    interp.add_context_command("second", cmd_second, id);
    interp.add_context_command("third", cmd_third, id);
    ...
```

The context struct will persist in the cache until the final command is removed (or, of
course, until the interpreter is dropped).

## Molt Objects

The standard way to represent an object in TCL is to define a command with attached
context data. The command's methods are implemented as subcommands.

The context cache supports this pattern trivially.  Define the object's instance variables
as a context struct, and define a command to create instances.

```
// Instance Data
struct InstanceContext { text: String }

// Command to make an instance
fn cmd_make(interp: &mut Interp, _: ContextID, argv: &[Value]) -> MoltResult {
    check_args(1, argv, 2, 2, "name")?;

    let id = interp.save_context(InstanceContext::new());

    interp.add_context_command(argv[1].as_str(), cmd_instance, id);

    molt_ok!()
}

// Instance Command
fn cmd_instance(interp: &mut Interp, _: ContextID, argv: &[Value]) -> MoltResult {
    check_args(1, argv, 2, 0, "subcommand ?args...?")?;

    // Get the context
    let ctx = interp.context::<AppContext>(context_id);

    // Do stuff based on argv[1], the subcommand.
    ...
}

// Registering the command
fn main() {
    let interp = Interp::new();

    interp.add_command("make", cmd_make);

    ...
}
```

Then, in Molt code you can create an object called `fred`, use its methods, and then
destroy it by renaming it to the empty string.

```tcl
% make fred
% fred do_something 1 2 3
...
% rename fred ""
```


## Molt Procedures

A Molt procedure is a routine coded in Tcl and defined using the `proc` command. A
crate can compile Tcl procedures into itself using the `include_str!` macro.  Start
by defining a script that defines the required procedure, say, `procs.tcl`, and put it
in the crate's `src/` folder adjacent to the Rust file that will load it.  The Rust
file can then do this:

```rust
let mut interp = Interp::new();

match interp.eval(include_str!("commands.tcl")) {
    Err(exception) => {
        if exception.is_error() {
            panic!("Couldn't load procs.tcl: {}", msg.value());
        }
    }
    _ => ()
}
```
