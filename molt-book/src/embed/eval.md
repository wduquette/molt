# Evaluating Molt Code

An application can evaluate Molt code in several ways:

* Use one of the `molt::Interp::eval` or `molt::Interp::eval_value` to evaluate an
  individual Molt command or script.

* Use the `molt::expr` function to evaluate a Molt expression, returning a Molt `Value`,
  or `molt::expr_bool`, `molt::expr_int`, and `molt::expr_float` for results of specific
  types.

* Use the `molt_shell::repl` function to provide an interactive REPL to the user.

* Use the `molt_shell::script` function to evaluate a script file (or just load the script's
  content and pass it to `molt::Interp::eval`).

## Evaluating Scripts with `eval`

The `molt::Interp::eval` method evaluates a string as a Molt script and returns the
result.  When executed at the top level, `ResultCode::Break`, `ResultCode::Continue`,
and `ResultCode::Other` are converted to errors, just as they are in `proc` bodies. See
[The `MoltResult` Type](./molt_result.md) for details.)

Thus, the following code will execute a script, returning its value and propagating
any exceptions to the caller.

```rust
use molt::Interp;
use molt::types::*;

let mut interp = Interp::new();

...

let value: Value = interp.eval("...some Molt code...")?;
```

The `molt::Interp::eval_value` method has identical semantics, but evaluates the string
representation of a molt `Value`. In this case, the `Value` will cache the parsed internal
form of the script to speed up subsequent evaluations.

## Evaluating Control Structure Bodies

The `molt::Interp::eval_value` method is used when implementing control structures.  For
example, this is an annotated version of of Molt's [**while**](./ref/while.md) command.

```rust
pub fn cmd_while(interp: &mut Interp, _: ContextID, argv: &[Value]) -> MoltResult {
    check_args(1, argv, 3, 3, "test command")?;

    // Here we evaluate the test expression as a boolean.  Any errors are propagated.
    while interp.expr_bool(&argv[1])? {
        // Here we evaluate the loop's body.
        let result = interp.eval_value(&argv[2]);

        if let Err(exception) = result {
            match exception.code() {
                // They want to break; so break out of the rust loop.
                ResultCode::Break => break,

                // They want to continue; so continue with the next iteration.
                ResultCode::Continue => (),

                // It's some other exception; just propagate it.
                _ => return Err(exception),
            }
        }
    }

    // All is good, so return Ok!
    molt_ok!()
}
```

See [The `MoltResult` Type](./molt_result.md) for more information.

## Evaluating Expressions with `expr` and `expr_bool`.

Evaluating Molt expressions is similar.  To get any expression result (usually a
numeric or boolean `Value`), use the `Interp::expr` method.

```rust
use molt::Interp;
use molt::types::*;
use molt::expr;

let mut interp = Interp::new();

...

let value: Value = interp.expr("1 + 1")?;
```

Use `Interp::expr_bool` when a specifically boolean result is wanted:

```rust
let flag: bool = interp.expr_bool("1 == 1")?;
```

(See the [`expr`](../ref/expr.md) command reference for more about Molt expressions.)

## Providing an interactive REPL

An interactive user shell or "REPL" (Read-Eval-Print-Loop) can be a great convenience
when developing and debugging application scripts; it can also be useful tool for
administering server processes.  To provide an interactive shell, use
the `molt_shell::repl` function.

```
use molt::Interp;

// FIRST, create and initialize the interpreter.
let mut interp = Interp::new();

// NOTE: commands can be added to the interpreter here.

// NEXT, invoke the REPL.
molt_shell::repl(&mut interp);
```

The REPL's prompt can be set using the `tcl_prompt1` variable to a script; see the
[**molt shell**](../cmdline/molt_shell.md) documentation for an example.

## Evaluating Script Files

To execute a user script file, one can load the file contents and use `Interp::eval` in
the normal way, or use the `molt_shell::script` function.  A shell application might
execute a user script as follows.  Any errors are output to the console.

```
use molt::Interp;
use std::env;

// FIRST, get the command line arguments.
let args: Vec<String> = env::args().collect();

// NEXT, create and initialize the interpreter.
let mut interp = Interp::new();

// NOTE: commands can be added to the interpreter here.

// NEXT, evaluate the file, if any.
if args.len() > 1 {
    molt_shell::script(&mut interp, &args[1..]);
} else {
    eprintln!("Usage: myshell filename.tcl");
}
```
