# Evaluating Molt Code

An application can evaluate Molt code in several ways:

* Use the `molt::Interp::eval` or `molt::Interp::eval_body` method to evaluate an
  individual Molt command or script.

* Use the `molt::expr` function to evaluate a Molt expression, returning a result.

* Use the `molt::expr_test` method to evaluate a Molt boolean expression.

* Use the `molt_shell::repl` function to provide an interactive REPL to the user.

* Use the `molt_shell::script` function to evaluate a script file.

## Evaluating Scripts with `eval`

The `molt::Interp::eval` method evaluates a string as a Molt script and returns the result,
which will always be either `Ok(Value)` or `Err(ResultCode:Error(Value))`. (Other result
codes are translated into `Ok` or `Err` as appropriate.  See
[The `MoltResult` Type](./molt_result.md) for details.)

```rust
use molt::Interp;
use molt::types::*;

let mut interp = Interp::new();

...

let value: Value = interp.eval("...some Molt code...")?;
```

The explicit `Value` type declaration is included for clarity.

## Evaluating Scripts with `eval_body`

The `molt::Interp::eval_body` method is used when implementing control structures, as it
gives access to the entire set of `MoltResult` return codes.  

```rust
use molt::Interp;
use molt::types::*;

fn my_cmd(interp: &mut Interp, argv: &[Value]) -> MoltResult {
    // Get a Molt script to evaluate, e.g., a loop body:
    let body = argv[1];

    ...

    // Assume we're implementing some kind of loop.
    loop {
        ...

        // Evaluate the loop body
        let result = interp.eval_body(&*body.as_string());

        match result {
            Ok(value) => {
                // normal OK result.  What you do with it depends on the
                // control structure. There's probably nothing special to do
                // here; we'll just go on with the next iteration of the loop.
                continue;
            }
            Err(ResultCode::Err(msg)) => {
                // An error.  Control structures should usually let this
                // propagate.
                return result;
            }
            Err(ResultCode::Return(value)) => {
                // The code called the `return` command.  Let this propagate to
                // return from the enclosing `proc`.
                return result;
            }
            Err(ResultCode::Break) => {
                // The code called the `break` command.  If this function is
                // implementing a loop, it should return `Ok` to break out of
                // the loop; otherwise, let the `Break` propagate to break out
                // of the enclosing loop. Since this is a loop,  
                // break and return `Ok'
                break;
            }
            Err(ResultCode::Continue) => {
                // The code called the `continue` command.  If this function is
                // implementing a loop, this should continue on to the next
                // iteration of the loop after doing any necessary clean-up.  
                // Otherwise, let it propagate to continue in the enclosing loop.
                // Here, we're implementing a loop.
                continue;
            }
        }

        ...
    }

    ...
    // It's a loop, which normally returns an empty result.
    molt_ok!()
}
```

See [The `MoltResult` Type](./molt_result.md) for more information.

The explicit `Value` type declaration is included for clarity.

## Evaluating Expressions with `expr` and `expr_test`.

Evaluating Molt expressions is similar.  To get any expression result (usually a
numeric or boolean `Value`), use the `molt::expr` function, which return

```rust
use molt::Interp;
use molt::types::*;
use molt::expr;

let mut interp = Interp::new();

...

let value: Value = molt::expr(&mut interp, "1 + 1")?;
```

Use `molt::expr_test` when a specifically boolean result is wanted:

```rust
let flag: bool = molt::expr_test(&mut interp, "1 == 1")?;
```

(See the [`expr`](../ref/expr.md) command reference for more about Molt expressions.)

## Providing an interactive REPL

An interactive user shell or "REPL" (Read-Eval-Print-Loop) can be a great convenience
when developing and debugging application scripts; it can also be useful tool for
administering server processes.  To provide an interactive, use
the `molt_shell::repl` function.

```
use molt::Interp;

// FIRST, create and initialize the interpreter.
let mut interp = Interp::new();

// NOTE: commands can be added to the interpreter here.

// NEXT, invoke the REPL.
molt_shell::repl(&mut interp, "% ");
```

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
    eprintln!("Usage: myshell *filename.tcl");
}
```
