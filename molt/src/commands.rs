//! # Standard Molt Command Definitions
//!
//! This module defines the standard Molt commands.

use crate::expr::molt_expr_string;
use crate::expr::molt_expr_bool;
use crate::interp::Interp;
use crate::types::*;
use crate::*;
use std::fs;

/// # append *varName* ?*value* ...?
///
/// See molt-book for semantics.
pub fn cmd_append(interp: &mut Interp, argv: &[Value]) -> MoltResult {
    check_args(1, argv, 2, 0, "varName ?value value ...?")?;

    // FIRST, get the value of the variable.  If the variable is undefined,
    // start with the empty string.

    // TODO: Can we make this conversion simpler, but still as efficient?
    let var_name = &*argv[1].as_string();
    let var_result = interp.var(var_name);

    let mut new_string = if var_result.is_ok() {
        // Use to_string() because we need a mutable owned string.
        var_result.unwrap().to_string()
    } else {
        String::new()
    };

    // NEXT, append the remaining values to the string.
    for item in &argv[2..] {
        new_string.push_str(&*item.as_string());
    }

    // NEXT, save and return the new value.
    molt_ok!(interp.set_var2(var_name, new_string.into()))
}

/// assert_eq received, expected
///
/// Returns an error if received doesn't equal the expected value.
/// Primarily for use in examples.
pub fn cmd_assert_eq(_interp: &mut Interp, argv: &[&str]) -> MoltResult {
    check_str_args(1, argv, 3, 3, "received expected")?;

    if argv[1] == argv[2] {
        molt_ok!()
    } else {
        molt_err!("assertion failed: received \"{}\", expected \"{}\".", argv[1], argv[2])
    }
}

/// # break
///
/// Terminates the inmost loop.
pub fn cmd_break(_interp: &mut Interp, argv: &[&str]) -> MoltResult {
    check_str_args(1, argv, 1, 1, "")?;

    Err(ResultCode::Break)
}

/// catch script ?resultVarName?
///
/// Executes a script, returning the result code.  If the resultVarName is given, the result
/// of executing the script is returned in it.  The result code is returned as an integer,
/// 0=Ok, 1=Error, 2=Return, 3=Break, 4=Continue.
pub fn cmd_catch(interp: &mut Interp, argv: &[&str]) -> MoltResult {
    check_str_args(1, argv, 2, 3, "script ?resultVarName")?;

    let result = interp.eval_body(argv[1]);
    let code: MoltInt;
    let mut value = String::new();

    match result {
        Ok(val) => {
            code = 0;
            value = val.to_string();
        }
        Err(ResultCode::Error(val)) => {
            code = 1;
            value = val.to_string();
        }
        Err(ResultCode::Return(val)) => {
            code = 2;
            value = val.to_string();
        }
        Err(ResultCode::Break) => {
            code = 3;
        }
        Err(ResultCode::Continue) => {
            code = 4;
        }
    }

    if argv.len() == 3 {
        interp.set_var(argv[2], &value);
    }

    Ok(Value::from(code))
}

/// # continue
///
/// Continues with the next iteration of the inmost loop.
pub fn cmd_continue(_interp: &mut Interp, argv: &[&str]) -> MoltResult {
    check_str_args(1, argv, 1, 1, "")?;

    Err(ResultCode::Continue)
}

/// error *message*
///
/// Returns an error with the given message.
///
/// ## TCL Liens
///
/// * In Standard TCL, `error` can optionally set the stack trace and an error code.
pub fn cmd_error(_interp: &mut Interp, argv: &[&str]) -> MoltResult {
    check_str_args(1, argv, 2, 2, "message")?;

    molt_err!(argv[1])
}

/// # exit ?*returnCode*?
///
/// Terminates the application by calling `std::process::exit()`.
/// If given, _returnCode_ must be an integer return code; if absent, it
/// defaults to 0.
pub fn cmd_exit(interp: &mut Interp, argv: &[&str]) -> MoltResult {
    check_str_args(1, argv, 1, 2, "?returnCode?")?;

    let return_code: MoltInt = if argv.len() == 1 {
        0
    } else {
        interp.get_int(argv[1])?
    };

    std::process::exit(return_code as i32)
}

/// # expr expr
///
/// Evaluates an expression and returns its result.
///
/// ## TCL Liens
///
/// See the Molt Book.

pub fn cmd_expr(interp: &mut Interp, argv: &[&str]) -> MoltResult {
    check_str_args(1, argv, 2, 2, "expr")?;

    molt_expr_string(interp, argv[1])
}

/// # for *start* *test* *next* *command*
///
/// A standard "for" loop.  start, next, and command are scripts; test is an expression
///
pub fn cmd_for(interp: &mut Interp, argv: &[&str]) -> MoltResult {
    check_str_args(1, argv, 5, 5, "start test next command")?;

    // Start
    interp.eval(argv[1])?;

    while molt_expr_bool(interp, argv[2])? {
        let result = interp.eval_body(argv[4]);

        match result {
            Ok(_) => (),
            Err(ResultCode::Break) => break,
            Err(ResultCode::Continue) => (),
            _ => return result,
        }

        // Execute next script.  Break is allowed, but continue is not.
        let result = interp.eval_body(argv[3]);

        match result {
            Ok(_) => (),
            Err(ResultCode::Break) => break,
            Err(ResultCode::Continue) => {
                return molt_err!("invoked \"continue\" outside of a loop");
            },
            _ => return result,
        }
    }

    molt_ok!()
}

/// # foreach *varList* *list* *body*
///
/// Loops over the items the list, assigning successive items to the variables in the
/// *varList* and calling the *body* as a script once for each set of assignments.
/// On the last iteration, the second and subsequents variables in the *varList* will
/// be assigned the empty string if there are not enough list elements to fill them.
///
/// ## TCL Liens
///
/// * In Standard TCL, `foreach` can loop over several lists at the same time.
pub fn cmd_foreach(interp: &mut Interp, argv: &[&str]) -> MoltResult {
    check_str_args(1, argv, 4, 4, "varList list body")?;

    let var_list = interp.get_list(argv[1])?;
    let list = interp.get_list(argv[2])?;
    let body = argv[3];

    let mut i = 0;

    while i < list.len() {
        for var_name in &var_list {
            if i < list.len() {
                interp.set_var(&*var_name.as_string(), &list[i].as_string());
                i += 1;
            } else {
                interp.set_var(&*var_name.as_string(), "");
            }
        }

        let result = interp.eval_body(body);

        match result {
            Ok(_) => (),
            Err(ResultCode::Break) => break,
            Err(ResultCode::Continue) => (),
            _ => return result,
        }
    }

    molt_ok!()
}


/// # global ?*varName* ...?
///
/// Appends any number of values to a variable's value, which need not
/// initially exist.
pub fn cmd_global(interp: &mut Interp, argv: &[&str]) -> MoltResult {
    // Accepts any number of arguments

    // FIRST, if we're at the global scope this is a no-op.
    if interp.scope_level() > 0 {
        for name in &argv[1..] {
            interp.upvar(0, name);
        }
    }
    molt_ok!()
}

#[derive(Eq, PartialEq, Debug)]
enum IfWants {
    Expr,
    ThenBody,
    SkipThenClause,
    ElseClause,
    ElseBody,
}

/// # if *expr* ?then? *script* elseif *expr* ?then? *script* ... ?else? ?*script*?
///
/// Standard conditional.  Returns the value of the selected script (or
/// "" if there is no else body and the none of the previous branches were selected).
///
/// # TCL Liens
///
/// * Because we don't yet have an expression parser, the *expr* arguments are evaluated as
///   scripts that must return a boolean value.
pub fn cmd_if(interp: &mut Interp, argv: &[&str]) -> MoltResult {
    let mut argi = 1;
    let mut wants = IfWants::Expr;

    while argi < argv.len() {
        match wants {
            IfWants::Expr => {
                wants = if molt_expr_bool(interp, argv[argi])? {
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

/// # incr *varName* ?*increment* ...?
///
/// Increments an integer variable by a value.
pub fn cmd_incr(interp: &mut Interp, argv: &[&str]) -> MoltResult {
    check_str_args(1, argv, 2, 3, "varName ?increment?")?;

    let increment: MoltInt = if argv.len() == 3 {
        interp.get_int(argv[2])?
    } else {
        1
    };

    let var_value = interp.var(argv[1]);

    let new_value = (if var_value.is_ok() {
        var_value.unwrap().as_int()? + increment
    } else {
        increment
    }).to_string();

    interp.set_var(argv[1], &new_value);
    molt_ok!("{}", new_value)
}


/// # info *subcommand* ?*arg*...?
pub fn cmd_info(interp: &mut Interp, argv: &[&str]) -> MoltResult {
    check_str_args(1, argv, 2, 0, "subcommand ?arg ...?")?;
    let subc = Subcommand::find(&INFO_SUBCOMMANDS, argv[1])?;

    (subc.1)(interp, argv)
}

const INFO_SUBCOMMANDS: [Subcommand; 3] = [
    Subcommand("commands", cmd_info_commands),
    Subcommand("complete", cmd_info_complete),
    Subcommand("vars", cmd_info_vars),
];

/// # info commands ?*pattern*?
pub fn cmd_info_commands(interp: &mut Interp, _argv: &[&str]) -> MoltResult {
    molt_ok!("{}", list_to_string(&interp.command_names()))
}

/// # info complete *command*
pub fn cmd_info_complete(interp: &mut Interp, argv: &[&str]) -> MoltResult {
    check_str_args(2, argv, 3, 3, "command")?;

    // TODO: Add way of returning a boolean result.
    if interp.complete(argv[2]) {
        molt_ok!("1")
    } else {
        molt_ok!("0")
    }
}

/// # info vars ?*pattern*?
/// TODO: Add glob matching as a feature, and provide optional pattern argument.
pub fn cmd_info_vars(interp: &mut Interp, _argv: &[&str]) -> MoltResult {
    molt_ok!(list_to_string(&interp.vars_in_scope()))
}

/// # join *list* ?*joinString*?
///
/// Joins the elements of a list with a string.  The join string defaults to " ".
pub fn cmd_join(interp: &mut Interp, argv: &[&str]) -> MoltResult {
    check_str_args(1, argv, 2, 3, "list ?joinString?")?;

    molt_err!("FUBAR")
    //
    // let list = interp.get_list(argv[1])?;
    //
    // let join_string = if argv.len() == 3 {
    //     argv[2]
    // } else {
    //     " "
    // };
    //
    // molt_ok!(list.join(join_string))
}

/// # lappend *varName* ?*value* ...?
///
/// Appends any number of values to a variable's list value, which need not
/// initially exist.
pub fn cmd_lappend(interp: &mut Interp, argv: &[&str]) -> MoltResult {
    check_str_args(1, argv, 2, 0, "varName ?value ...?")?;
    molt_err!("FUBAR")

    // let var_result = interp.var(argv[1]);
    //
    // let mut list: Vec<String> = if var_result.is_ok() {
    //     interp.get_list(&var_result.unwrap())?
    // } else {
    //     Vec::new()
    // };
    //
    // for value in &argv[2..] {
    //     list.push(value.to_string());
    // }
    //
    // let new_value = list_to_string(&list);
    //
    // interp.set_var(argv[1], &new_value);
    //
    // molt_ok!("{}", new_value)
}

/// # lindex *list* ?*index* ...?
///
/// Returns an element from the list, indexing into nested lists.
pub fn cmd_lindex(interp: &mut Interp, argv: &[&str]) -> MoltResult {
    check_str_args(1, argv, 2, 0, "list ?index ...?")?;
    molt_err!("FUBAR")

    // let mut value = argv[1].to_string();
    //
    // for index_string in &argv[2..] {
    //     let list = interp.get_list(&value)?;
    //     let index = interp.get_int(index_string)?;
    //
    //     value = if index < 0 || index as usize >= list.len() {
    //         "".to_string()
    //     }  else {
    //         list[index as usize].to_string()
    //     };
    // }
    //
    // molt_ok!(value)
}

/// # list ?*arg*...?
///
/// Converts its arguments into a canonical list.
pub fn cmd_list(_interp: &mut Interp, argv: &[Value]) -> MoltResult {
    // // No arg check needed; can take any number.
    molt_ok!(list_to_string(&argv[1..]))
}

/// # llength *list*
///
/// Returns the length of the list.
pub fn cmd_llength(interp: &mut Interp, argv: &[&str]) -> MoltResult {
    check_str_args(1, argv, 2, 2, "list")?;
    molt_err!("FUBAR")
    //
    // let list = interp.get_list(argv[1])?;
    //
    // molt_ok!(list.len().to_string())
}

pub fn cmd_proc(interp: &mut Interp, argv: &[&str]) -> MoltResult {
    check_str_args(1, argv, 4, 4, "name args body")?;
    molt_err!("FUBAR")
    //
    // // FIRST, get the arguments
    // let name = argv[1];
    // let args = interp.get_list(argv[2])?;
    // let body = argv[3];
    //
    // // NEXT, validate the argument specs
    // for arg in &args {
    //     let vec = interp.get_list(&arg)?;
    //
    //     if vec.is_empty() {
    //         return molt_err!("argument with no name");
    //     } else if vec.len() > 2 {
    //         return molt_err!("too many fields in argument specifier \"{}\"", arg);
    //     }
    // }
    //
    // // NEXT, add the command.
    // interp.add_proc(name, args, body);
    //
    // molt_ok!()
}

/// # puts *string*
///
/// Outputs the string to stdout.
///
/// ## TCL Liens
///
/// * Does not support `-nonewline`
/// * Does not support `channelId`
pub fn cmd_puts(_interp: &mut Interp, argv: &[Value]) -> MoltResult {
    check_args(1, argv, 2, 2, "string")?;

    println!("{}", argv[1]);
    molt_ok!()
}

/// # rename *oldName* *newName*
///
/// Renames the command called *oldName* to have the *newName*.  If the
/// *newName* is "", the command is destroyed.
pub fn cmd_rename(interp: &mut Interp, argv: &[&str]) -> MoltResult {
    check_str_args(1, argv, 3, 3, "oldName newName")?;

    // FIRST, get the arguments
    let old_name = argv[1];
    let new_name = argv[2];

    if !interp.has_command(old_name) {
        return molt_err!("can't rename \"{}\": command doesn't exist", old_name);
    }

    // NEXT, rename or remove the command.
    if new_name.is_empty() {
        interp.remove_command(old_name);
    } else {
        interp.rename_command(old_name, new_name);
    }

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
pub fn cmd_return(_interp: &mut Interp, argv: &[&str]) -> MoltResult {
    check_str_args(1, argv, 1, 2, "?value?")?;

    let value = if argv.len() == 1 {
        ""
    } else {
        argv[1]
    };

    Err(ResultCode::Return(Value::from(value)))
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
pub fn cmd_set(interp: &mut Interp, argv: &[&str]) -> MoltResult {
    check_str_args(1, argv, 2, 3, "varName ?newValue?")?;

    let value;

    if argv.len() == 3 {
        value = argv[2].into();
        interp.set_var(argv[1], argv[2]);
    } else {
        value = interp.var(argv[1])?.to_string();
    }

    molt_ok!(value)
}

/// # source *filename*
///
/// Sources the file, returning the result.
pub fn cmd_source(interp: &mut Interp, argv: &[&str]) -> MoltResult {
    check_str_args(1, argv, 2, 2, "filename")?;

    match fs::read_to_string(argv[1]) {
        Ok(script) => interp.eval(&script),
        Err(e) => molt_err!("couldn't read file \"{}\": {}", argv[1], e),
    }
}

/// # unset *varName*
///
/// Removes the variable from the interpreter.  This is a no op if
/// there is no such variable.
pub fn cmd_unset(interp: &mut Interp, argv: &[&str]) -> MoltResult {
    check_str_args(1, argv, 2, 2, "varName")?;

    interp.unset_var(argv[1]);

    molt_ok!()
}

/// # while *test* *command*
///
/// A standard "while" loop.  *test* is a boolean expression; *command* is a script to
/// execute so long as the expression is true.
pub fn cmd_while(interp: &mut Interp, argv: &[&str]) -> MoltResult {
    check_str_args(1, argv, 3, 3, "test command")?;

    while molt_expr_bool(interp, argv[1])? {
        let result = interp.eval_body(argv[2]);

        match result {
            Ok(_) => (),
            Err(ResultCode::Break) => break,
            Err(ResultCode::Continue) => (),
            _ => return result,
        }
    }

    molt_ok!()
}
