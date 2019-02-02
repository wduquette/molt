//! # Standard Molt Command Definitions
//!
//! This module defines the standard Molt commands.

use crate::interp::Interp;
use crate::types::*;
use crate::*;

/// # append *varName* ?*value* ...?
///
/// Appends any number of values to a variable's value, which need not
/// initially exist.
pub fn cmd_append(interp: &mut Interp, argv: &[&str]) -> InterpResult {
    check_args(1, argv, 2, 0, "varName ?value ...?")?;

    let var_result = interp.get_var(argv[1]);

    let mut new_value = if var_result.is_ok() {
        var_result.unwrap()
    } else {
        String::new()
    };

    for value in &argv[2..] {
        new_value.push_str(value);
    }

    interp.set_var(argv[1], &new_value);

    molt_ok!("{}", new_value)
}

/// assert_eq received, expected
///
/// Returns an error if received doesn't equal the expected value.
/// Primarily for use in examples.
pub fn cmd_assert_eq(_interp: &mut Interp, argv: &[&str]) -> InterpResult {
    check_args(1, argv, 3, 3, "received expected")?;

    if argv[1] == argv[2] {
        molt_ok!()
    } else {
        molt_err!("assertion failed: received \"{}\", expected \"{}\".", argv[1], argv[2])
    }
}

/// # exit ?*returnCode*?
///
/// Terminates the application by calling `std::process::exit()`.
/// If given, _returnCode_ must be an integer return code; if absent, it
/// defaults to 0.
pub fn cmd_exit(_interp: &mut Interp, argv: &[&str]) -> InterpResult {
    check_args(1, argv, 1, 2, "?returnCode?")?;

    let return_code: MoltInt = if argv.len() == 1 {
        0
    } else {
        get_int(argv[1])?
    };

    std::process::exit(return_code)
}

/// # foreach *varName* *list* *body*
///
/// Calls the *body* as a script once for each entry in the *list*,
/// with *varName* taking on successive list items
pub fn cmd_foreach(interp: &mut Interp, argv: &[&str]) -> InterpResult {
    check_args(1, argv, 4, 4, "varName list body")?;

    let var_name = argv[1];
    let list = get_list(argv[2])?;
    let body = argv[3];

    for item in list {
        interp.set_var(var_name, &item);
        interp.eval_body(body)?;
    }

    molt_ok!()
}


/// # global ?*varName* ...?
///
/// Appends any number of values to a variable's value, which need not
/// initially exist.
pub fn cmd_global(interp: &mut Interp, argv: &[&str]) -> InterpResult {
    // Accepts any number of arguments

    // FIRST, if we're at the global scope this is a no-op.
    if interp.scope_level() > 0 {
        for name in &argv[1..] {
            interp.upvar(0, name);
        }
    }
    molt_ok!()
}

/// # if *condition* ?then? *script* ?else? ?*script*?
#[derive(Eq, PartialEq, Debug)]
enum IfWants {
    Expr,
    ThenBody,
    SkipThenClause,
    ElseClause,
    ElseBody,
}

pub fn cmd_if(interp: &mut Interp, argv: &[&str]) -> InterpResult {
    let mut argi = 1;
    let mut wants = IfWants::Expr;

    while argi < argv.len() {
        match wants {
            IfWants::Expr => {
                let cond_result = interp.eval(argv[argi])?;
                wants = if get_boolean(&cond_result)? {
                    IfWants::ThenBody
                } else {
                    IfWants::SkipThenClause
                };
            },
            IfWants::ThenBody => {
                if argv[argi] == "then" {
                    argi += 1;
                }

                if argi < argv.len() {
                    return interp.eval_body(argv[argi]);
                } else {
                    break;
                }
            },
            IfWants::SkipThenClause => {
                if argv[argi] == "then" {
                    argi += 1;
                }

                if argi < argv.len() {
                    argi += 1;
                    wants = IfWants::ElseClause;
                }
                continue;
            }
            IfWants::ElseClause => {
                if argv[argi] == "elseif" {
                    wants = IfWants::Expr;
                } else {
                    wants = IfWants::ElseBody;
                    continue;
                }
            }
            IfWants::ElseBody => {
                if argv[argi] == "else" {
                    argi += 1;

                    // If "else" appears, then the else body is required.
                    if argi == argv.len() {
                        return molt_err!("wrong # args: no script following after \"{}\" argument",
                            argv[argi - 1]);
                    }
                }

                if argi < argv.len() {
                    return interp.eval_body(argv[argi]);
                } else {
                    break;
                }
            }
        }

        argi += 1;
    }

    if argi < argv.len() {
        return molt_err!(
            "wrong # args: extra words after \"else\" clause in \"if\" command");
    } else if wants == IfWants::Expr {
        return molt_err!("wrong # args: no expression after \"{}\" argument",
            argv[argi-1]);
    } else if wants == IfWants::ThenBody || wants == IfWants::SkipThenClause {
        return molt_err!("wrong # args: no script following after \"{}\" argument",
            argv[argi - 1]);
    } else {
        // Looking for ElseBody, but there doesn't need to be one.
        molt_ok!() // temp
    }
}

/// # info *subcommand* ?*arg*...?
pub fn cmd_info(interp: &mut Interp, argv: &[&str]) -> InterpResult {
    check_args(1, argv, 2, 0, "subcommand ?arg ...?")?;
    let subc = get_subcommand(&INFO_SUBCOMMANDS, argv[1])?;

    (subc.1)(interp, argv)
}

const INFO_SUBCOMMANDS: [Subcommand; 3] = [
    Subcommand("commands", cmd_info_commands),
    Subcommand("complete", cmd_info_complete),
    Subcommand("vars", cmd_info_vars),
];

/// # info commands ?*pattern*?
pub fn cmd_info_commands(interp: &mut Interp, _argv: &[&str]) -> InterpResult {
    molt_ok!("{}", list_to_string(&interp.get_command_names()))
}

/// # info complete *command*
pub fn cmd_info_complete(interp: &mut Interp, argv: &[&str]) -> InterpResult {
    check_args(2, argv, 3, 3, "command")?;

    // TODO: Add way of returning a boolean result.
    if interp.complete(argv[2]) {
        molt_ok!("1")
    } else {
        molt_ok!("0")
    }
}

/// # info vars ?*pattern*?
pub fn cmd_info_vars(interp: &mut Interp, _argv: &[&str]) -> InterpResult {
    molt_ok!(list_to_string(&interp.get_visible_var_names()))
}

/// # join *list* ?*joinString*?
///
/// Joins the elements of a list with a string.  The join string defaults to " ".
pub fn cmd_join(_interp: &mut Interp, argv: &[&str]) -> InterpResult {
    check_args(1, argv, 2, 3, "list ?joinString?")?;

    let list = get_list(argv[1])?;

    let join_string = if argv.len() == 3 {
        argv[2]
    } else {
        " "
    };

    molt_ok!(list.join(join_string))
}

/// # lindex *list* ?*index* ...?
///
/// Returns an element from the list, indexing into nested lists.
pub fn cmd_lindex(_interp: &mut Interp, argv: &[&str]) -> InterpResult {
    check_args(1, argv, 2, 0, "list ?index ...?")?;

    let mut value = argv[1].to_string();

    for index_string in &argv[2..] {
        let list = get_list(&value)?;
        let index = get_int(index_string)?;

        value = if index < 0 || index as usize >= list.len() {
            "".to_string()
        }  else {
            list[index as usize].to_string()
        };
    }

    molt_ok!(value)
}

/// # list ?*arg*...?
///
/// Converts its arguments into a canonical list.
pub fn cmd_list(_interp: &mut Interp, argv: &[&str]) -> InterpResult {
    // No arg check needed; can take any number.
    molt_ok!(list_to_string(&argv[1..]))
}

/// # llength *list*
///
/// Returns the length of the list.
pub fn cmd_llength(_interp: &mut Interp, argv: &[&str]) -> InterpResult {
    check_args(1, argv, 2, 2, "list")?;

    let list = get_list(argv[1])?;

    molt_ok!(list.len().to_string())
}

pub fn cmd_proc(interp: &mut Interp, argv: &[&str]) -> InterpResult {
    check_args(1, argv, 4, 4, "name args body")?;

    // FIRST, get the arguments
    let name = argv[1];
    let args = get_list(argv[2])?;
    let body = argv[3];

    // NEXT, validate the argument specs
    for arg in &args {
        let vec = get_list(&arg)?;

        if vec.is_empty() {
            return molt_err!("argument with no name");
        } else if vec.len() > 2 {
            return molt_err!("too many fields in argument specifier \"{}\"", arg);
        }
    }

    // NEXT, add the command.
    interp.add_command_proc(name, args, body);

    molt_ok!()
}

/// # puts *string*
///
/// Outputs the string to stdout.
///
/// ## TCL Liens
///
/// * Does not support `-nonewline`
/// * Does not support `channelId`
pub fn cmd_puts(_interp: &mut Interp, argv: &[&str]) -> InterpResult {
    check_args(1, argv, 2, 2, "string")?;

    println!("{}", argv[1]);
    molt_ok!()
}

/// # return ?value?
///
/// Returns from a proc.  The proc will return the given value, or ""
/// if no value is specified.
///
/// ## TCL Liens
///
/// * Doesn't support all of TCL's fancy return machinery.
pub fn cmd_return(_interp: &mut Interp, argv: &[&str]) -> InterpResult {
    check_args(1, argv, 1, 2, "?value?")?;

    let value = if argv.len() == 1 {
        ""
    } else {
        argv[1]
    };

    Err(ResultCode::Return(value.into()))
}

/// # set *varName* ?*newValue*?
///
/// Sets variable *varName* to *newValue*, returning the value.
/// If *newValue* is omitted, returns the variable's current value,
/// returning an error if the variable is unknown.
///
/// ## TCL Liens
///
/// * Does not support arrays
pub fn cmd_set(interp: &mut Interp, argv: &[&str]) -> InterpResult {
    check_args(1, argv, 2, 3, "varName ?newValue?")?;

    let value;

    if argv.len() == 3 {
        value = argv[2].into();
        interp.set_var(argv[1], argv[2]);
    } else {
        value = interp.get_var(argv[1])?;
    }

    molt_ok!(value)
}

/// # unset *varName*
///
/// Removes the variable from the interpreter.  This is a no op if
/// there is no such variable.
pub fn cmd_unset(interp: &mut Interp, argv: &[&str]) -> InterpResult {
    check_args(1, argv, 2, 2, "varName")?;

    interp.unset_var(argv[1]);

    molt_ok!()
}

/// # test *name* *script* -ok|-error *result*
///
/// Executes the script expecting either a successful response or an error.
///
/// Note: This is an extremely minimal replacement for tcltest; at some
/// point I'll need something much more robust.
pub fn cmd_test(interp: &mut Interp, argv: &[&str]) -> InterpResult {
    check_args(1, argv, 5, 5, "name script -ok|-error result")?;

    let name = argv[1];
    let script = argv[2];
    let code = argv[3];
    let output = argv[4];

    if code != "-ok" && code != "-error" {
        return molt_err!("unknown option: \"{}\"", code);
    }

    match interp.eval(script) {
        Ok(out) => {
            if code == "-ok" && out == output {
                println!("*** test {} passed.", name);
            } else {
                println!("*** test {} FAILED.", name);
                println!("Expected <{}>", output);
                println!("Received <{}>", out);
            }
        }
        Err(ResultCode::Error(out)) => {
            if code == "-error" && out == output {
                println!("*** test {} passed.", name);
            } else {
                println!("*** test {} FAILED.", name);
                println!("Expected <{}>", output);
                println!("Received <{}>", out);
            }
        }
        Err(result) => {
            println!("test {} failed, unexpected result:\n{:?}", name, result);
        }
    }

    molt_ok!()
}
