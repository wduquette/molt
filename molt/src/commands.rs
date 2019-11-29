//! # Standard Molt Command Definitions
//!
//! This module defines the standard Molt commands.

use crate::interp::Interp;
use crate::types::*;
use crate::*;
use std::fs;
use std::time::Instant;

/// # append *varName* ?*value* ...?
///
/// Appends one or more strings to a variable.
/// See molt-book for full semantics.
pub fn cmd_append(interp: &mut Interp, argv: &[Value]) -> MoltResult {
    check_args(1, argv, 2, 0, "varName ?value value ...?")?;

    // FIRST, get the value of the variable.  If the variable is undefined,
    // start with the empty string.
    let mut new_string: String = interp
        .var(&argv[1])
        .and_then(|val| Ok(val.to_string()))
        .unwrap_or_else(|_| String::new());

    // NEXT, append the remaining values to the string.
    for item in &argv[2..] {
        new_string.push_str(item.as_str());
    }

    // NEXT, save and return the new value.
    interp.set_var_return(&argv[1], new_string.into())
}

/// assert_eq received, expected
///
/// Asserts that two values have identical string representations.
/// See molt-book for full semantics.
pub fn cmd_assert_eq(_interp: &mut Interp, argv: &[Value]) -> MoltResult {
    check_args(1, argv, 3, 3, "received expected")?;

    if argv[1] == argv[2] {
        molt_ok!()
    } else {
        molt_err!(
            "assertion failed: received \"{}\", expected \"{}\".",
            argv[1],
            argv[2]
        )
    }
}

/// # break
///
/// Breaks a loops.
/// See molt-book for full semantics.
pub fn cmd_break(_interp: &mut Interp, argv: &[Value]) -> MoltResult {
    check_args(1, argv, 1, 1, "")?;

    Err(ResultCode::Break)
}

/// catch script ?resultVarName?
///
/// Executes a script, returning the result code.  If the resultVarName is given, the result
/// of executing the script is returned in it.  The result code is returned as an integer,
/// 0=Ok, 1=Error, 2=Return, 3=Break, 4=Continue.
pub fn cmd_catch(interp: &mut Interp, argv: &[Value]) -> MoltResult {
    check_args(1, argv, 2, 3, "script ?resultVarName?")?;

    let result = interp.eval_body(&argv[1]);

    let (code, value) = match result {
        Ok(val) => (0, val),
        Err(ResultCode::Error(val)) => (1, val),
        Err(ResultCode::Return(val)) => (2, val),
        Err(ResultCode::Break) => (3, Value::empty()),
        Err(ResultCode::Continue) => (4, Value::empty()),
    };

    if argv.len() == 3 {
        interp.set_var(&argv[2], value)?;
    }

    Ok(Value::from(code))
}

/// # continue
///
/// Continues with the next iteration of the inmost loop.
pub fn cmd_continue(_interp: &mut Interp, argv: &[Value]) -> MoltResult {
    check_args(1, argv, 1, 1, "")?;

    Err(ResultCode::Continue)
}

/// error *message*
///
/// Returns an error with the given message.
///
/// ## TCL Liens
///
/// * In Standard TCL, `error` can optionally set the stack trace and an error code.
pub fn cmd_error(_interp: &mut Interp, argv: &[Value]) -> MoltResult {
    check_args(1, argv, 2, 2, "message")?;

    molt_err!(argv[1].clone())
}

/// # exit ?*returnCode*?
///
/// Terminates the application by calling `std::process::exit()`.
/// If given, _returnCode_ must be an integer return code; if absent, it
/// defaults to 0.
pub fn cmd_exit(_interp: &mut Interp, argv: &[Value]) -> MoltResult {
    check_args(1, argv, 1, 2, "?returnCode?")?;

    let return_code: MoltInt = if argv.len() == 1 {
        0
    } else {
        argv[1].as_int()?
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

pub fn cmd_expr(interp: &mut Interp, argv: &[Value]) -> MoltResult {
    check_args(1, argv, 2, 2, "expr")?;

    interp.expr(&argv[1])
}

/// # for *start* *test* *next* *command*
///
/// A standard "for" loop.  start, next, and command are scripts; test is an expression
///
pub fn cmd_for(interp: &mut Interp, argv: &[Value]) -> MoltResult {
    check_args(1, argv, 5, 5, "start test next command")?;

    let start = &argv[1];
    let test = &argv[2];
    let next = &argv[3];
    let command = &argv[4];

    // Start
    interp.eval_value(start)?;

    while interp.expr_bool(test)? {
        let result = interp.eval_body(command);

        match result {
            Ok(_) => (),
            Err(ResultCode::Break) => break,
            Err(ResultCode::Continue) => (),
            _ => return result,
        }

        // Execute next script.  Break is allowed, but continue is not.
        let result = interp.eval_body(next);

        match result {
            Ok(_) => (),
            Err(ResultCode::Break) => break,
            Err(ResultCode::Continue) => {
                return molt_err!("invoked \"continue\" outside of a loop");
            }
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
pub fn cmd_foreach(interp: &mut Interp, argv: &[Value]) -> MoltResult {
    check_args(1, argv, 4, 4, "varList list body")?;

    let var_list = &*argv[1].as_list()?;
    let list = &*argv[2].as_list()?;
    let body = &argv[3];

    let mut i = 0;

    while i < list.len() {
        for var in var_list {
            if i < list.len() {
                interp.set_var(&var, list[i].clone())?;
                i += 1;
            } else {
                interp.set_var(&var, Value::empty())?;
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
pub fn cmd_global(interp: &mut Interp, argv: &[Value]) -> MoltResult {
    // Accepts any number of arguments

    // FIRST, if we're at the global scope this is a no-op.
    if interp.scope_level() > 0 {
        for name in &argv[1..] {
            // TODO: Should upvar take the name as a Value?
            interp.upvar(0, name.as_str());
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
pub fn cmd_if(interp: &mut Interp, argv: &[Value]) -> MoltResult {
    let mut argi = 1;
    let mut wants = IfWants::Expr;

    while argi < argv.len() {
        match wants {
            IfWants::Expr => {
                wants = if interp.expr_bool(&argv[argi])? {
                    IfWants::ThenBody
                } else {
                    IfWants::SkipThenClause
                };
            }
            IfWants::ThenBody => {
                if argv[argi].as_str() == "then" {
                    argi += 1;
                }

                if argi < argv.len() {
                    return interp.eval_body(&argv[argi]);
                } else {
                    break;
                }
            }
            IfWants::SkipThenClause => {
                if argv[argi].as_str() == "then" {
                    argi += 1;
                }

                if argi < argv.len() {
                    argi += 1;
                    wants = IfWants::ElseClause;
                }
                continue;
            }
            IfWants::ElseClause => {
                if argv[argi].as_str() == "elseif" {
                    wants = IfWants::Expr;
                } else {
                    wants = IfWants::ElseBody;
                    continue;
                }
            }
            IfWants::ElseBody => {
                if argv[argi].as_str() == "else" {
                    argi += 1;

                    // If "else" appears, then the else body is required.
                    if argi == argv.len() {
                        return molt_err!(
                            "wrong # args: no script following after \"{}\" argument",
                            argv[argi - 1]
                        );
                    }
                }

                if argi < argv.len() {
                    return interp.eval_body(&argv[argi]);
                } else {
                    break;
                }
            }
        }

        argi += 1;
    }

    if argi < argv.len() {
        return molt_err!("wrong # args: extra words after \"else\" clause in \"if\" command");
    } else if wants == IfWants::Expr {
        return molt_err!(
            "wrong # args: no expression after \"{}\" argument",
            argv[argi - 1]
        );
    } else if wants == IfWants::ThenBody || wants == IfWants::SkipThenClause {
        return molt_err!(
            "wrong # args: no script following after \"{}\" argument",
            argv[argi - 1]
        );
    } else {
        // Looking for ElseBody, but there doesn't need to be one.
        molt_ok!() // temp
    }
}

/// # incr *varName* ?*increment* ...?
///
/// Increments an integer variable by a value.
pub fn cmd_incr(interp: &mut Interp, argv: &[Value]) -> MoltResult {
    check_args(1, argv, 2, 3, "varName ?increment?")?;

    let increment: MoltInt = if argv.len() == 3 {
        argv[2].as_int()?
    } else {
        1
    };

    let new_value = increment
        + interp
            .var(&argv[1])
            .and_then(|val| Ok(val.as_int()?))
            .unwrap_or_else(|_| 0);

    interp.set_var_return(&argv[1], new_value.into())
}

/// # info *subcommand* ?*arg*...?
pub fn cmd_info(interp: &mut Interp, argv: &[Value]) -> MoltResult {
    check_args(1, argv, 2, 0, "subcommand ?arg ...?")?;
    let subc = Subcommand::find(&INFO_SUBCOMMANDS, argv[1].as_str())?;

    (subc.1)(interp, argv)
}

const INFO_SUBCOMMANDS: [Subcommand; 3] = [
    Subcommand("commands", cmd_info_commands),
    Subcommand("complete", cmd_info_complete),
    Subcommand("vars", cmd_info_vars),
];

/// # info commands ?*pattern*?
pub fn cmd_info_commands(interp: &mut Interp, _argv: &[Value]) -> MoltResult {
    molt_ok!(Value::from(interp.command_names()))
}

/// # info complete *command*
pub fn cmd_info_complete(interp: &mut Interp, argv: &[Value]) -> MoltResult {
    check_args(2, argv, 3, 3, "command")?;

    if interp.complete(argv[2].as_str()) {
        molt_ok!(true)
    } else {
        molt_ok!(false)
    }
}

/// # info vars ?*pattern*?
/// TODO: Add glob matching as a feature, and provide optional pattern argument.
pub fn cmd_info_vars(interp: &mut Interp, _argv: &[Value]) -> MoltResult {
    molt_ok!(Value::from(interp.vars_in_scope()))
}

/// # join *list* ?*joinString*?
///
/// Joins the elements of a list with a string.  The join string defaults to " ".
pub fn cmd_join(_interp: &mut Interp, argv: &[Value]) -> MoltResult {
    check_args(1, argv, 2, 3, "list ?joinString?")?;

    let list = &argv[1].as_list()?;

    let join_string = if argv.len() == 3 {
        argv[2].to_string()
    } else {
        " ".to_string()
    };

    // TODO: Need to implement a standard join() method for MoltLists.
    let list: Vec<String> = list.iter().map(|v| v.to_string()).collect();

    molt_ok!(list.join(&join_string))
}

/// # lappend *varName* ?*value* ...?
///
/// Appends any number of values to a variable's list value, which need not
/// initially exist.
pub fn cmd_lappend(interp: &mut Interp, argv: &[Value]) -> MoltResult {
    check_args(1, argv, 2, 0, "varName ?value ...?")?;

    let var_result = interp.var(&argv[1]);

    let mut list: MoltList = if var_result.is_ok() {
        var_result.expect("got value").to_list()?
    } else {
        Vec::new()
    };

    let mut values = argv[2..].to_owned();
    list.append(&mut values);
    interp.set_var_return(&argv[1], Value::from(list))
}

/// # lindex *list* ?*index* ...?
///
/// Returns an element from the list, indexing into nested lists.
pub fn cmd_lindex(_interp: &mut Interp, argv: &[Value]) -> MoltResult {
    check_args(1, argv, 2, 0, "list ?index ...?")?;

    if argv.len() != 3 {
        lindex_into(&argv[1], &argv[2..])
    } else {
        lindex_into(&argv[1], &*argv[2].as_list()?)
    }
}

pub fn lindex_into(list: &Value, indices: &[Value]) -> MoltResult {
    let mut value: Value = list.clone();

    for index_val in indices {
        let list = value.as_list()?;
        let index = index_val.as_int()?;

        value = if index < 0 || index as usize >= list.len() {
            Value::empty()
        } else {
            list[index as usize].clone()
        };
    }

    molt_ok!(value)
}

/// # list ?*arg*...?
///
/// Converts its arguments into a canonical list.
pub fn cmd_list(_interp: &mut Interp, argv: &[Value]) -> MoltResult {
    // No arg check needed; can take any number.
    molt_ok!(&argv[1..])
}

/// # llength *list*
///
/// Returns the length of the list.
pub fn cmd_llength(_interp: &mut Interp, argv: &[Value]) -> MoltResult {
    check_args(1, argv, 2, 2, "list")?;

    molt_ok!(argv[1].as_list()?.len() as MoltInt)
}

/// # pdump
///
/// Dumps profile data.  Developer use only.
pub fn cmd_pdump(interp: &mut Interp, argv: &[Value]) -> MoltResult {
    check_args(1, argv, 1, 1, "")?;

    interp.profile_dump();

    molt_ok!()
}

/// # pclear
///
/// Clears profile data.  Developer use only.
pub fn cmd_pclear(interp: &mut Interp, argv: &[Value]) -> MoltResult {
    check_args(1, argv, 1, 1, "")?;

    interp.profile_clear();

    molt_ok!()
}

/// # proc *name* *args* *body*
///
/// Defines a procedure.
pub fn cmd_proc(interp: &mut Interp, argv: &[Value]) -> MoltResult {
    check_args(1, argv, 4, 4, "name args body")?;

    // FIRST, get the arguments
    let name = argv[1].as_str();
    let args = &*argv[2].as_list()?;
    let body = argv[3].as_str();

    // NEXT, validate the argument specs
    for arg in args {
        let vec = arg.as_list()?;

        if vec.is_empty() {
            return molt_err!("argument with no name");
        } else if vec.len() > 2 {
            return molt_err!("too many fields in argument specifier \"{}\"", arg);
        }
    }

    // NEXT, add the command.
    interp.add_proc(name, args, body);

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
pub fn cmd_puts(_interp: &mut Interp, argv: &[Value]) -> MoltResult {
    check_args(1, argv, 2, 2, "string")?;

    println!("{}", argv[1]);
    molt_ok!()
}

/// # rename *oldName* *newName*
///
/// Renames the command called *oldName* to have the *newName*.  If the
/// *newName* is "", the command is destroyed.
pub fn cmd_rename(interp: &mut Interp, argv: &[Value]) -> MoltResult {
    check_args(1, argv, 3, 3, "oldName newName")?;

    // FIRST, get the arguments
    let old_name = argv[1].as_str();
    let new_name = argv[2].as_str();

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
/// * Doesn't support all of TCL's fancy return machinery. Someday it will.
pub fn cmd_return(_interp: &mut Interp, argv: &[Value]) -> MoltResult {
    check_args(1, argv, 1, 2, "?value?")?;

    let value = if argv.len() == 1 {
        Value::empty()
    } else {
        argv[1].clone()
    };

    Err(ResultCode::Return(value))
}

/// # set *varName* ?*newValue*?
///
/// Sets variable *varName* to *newValue*, returning the value.
/// If *newValue* is omitted, returns the variable's current value,
/// returning an error if the variable is unknown.
pub fn cmd_set(interp: &mut Interp, argv: &[Value]) -> MoltResult {
    check_args(1, argv, 2, 3, "varName ?newValue?")?;

    if argv.len() == 3 {
        interp.set_var_return(&argv[1], argv[2].clone())
    } else {
        molt_ok!(interp.var(&argv[1])?.clone())
    }
}

/// # source *filename*
///
/// Sources the file, returning the result.
pub fn cmd_source(interp: &mut Interp, argv: &[Value]) -> MoltResult {
    check_args(1, argv, 2, 2, "filename")?;

    let filename = argv[1].as_str();

    match fs::read_to_string(filename) {
        Ok(script) => interp.eval(&script),
        Err(e) => molt_err!("couldn't read file \"{}\": {}", filename, e),
    }
}

/// # time *command* ?*count*?
///
/// Executes the command the given number of times, and returns the average
/// number of microseconds per iteration.  The *count* defaults to 1.
pub fn cmd_time(interp: &mut Interp, argv: &[Value]) -> MoltResult {
    check_args(1, argv, 2, 3, "command ?count?")?;

    let command = &argv[1];

    let count = if argv.len() == 3 {
        argv[2].as_int()?
    } else {
        1
    };

    let start = Instant::now();

    for _i in 0..count {
        let result = interp.eval_value(command);
        if result.is_err() {
            return result;
        }
    }

    let span = start.elapsed();

    let avg = if count > 0 {
        span.as_nanos() / (count as u128)
    } else {
        0
    } as MoltInt;

    molt_ok!("{} nanoseconds per iteration", avg)
}

/// # unset ?-nocomplain? *varName*
///
/// Removes the variable from the interpreter.  This is a no op if
/// there is no such variable.  The -nocomplain option is accepted for
/// compatible with standard TCL, but is never required.
pub fn cmd_unset(interp: &mut Interp, argv: &[Value]) -> MoltResult {
    check_args(1, argv, 1, 0, "?-nocomplain? ?--? ?name name name...?")?;

    let mut options_ok = true;

    for arg in argv {
        let var = arg.as_str();

        if options_ok {
            if var == "--" {
                options_ok = false;
                continue;
            } else if var == "-nocomplain" {
                continue;
            }
        }

        interp.unset_var(var);
    }

    molt_ok!()
}

/// # while *test* *command*
///
/// A standard "while" loop.  *test* is a boolean expression; *command* is a script to
/// execute so long as the expression is true.
pub fn cmd_while(interp: &mut Interp, argv: &[Value]) -> MoltResult {
    check_args(1, argv, 3, 3, "test command")?;

    while interp.expr_bool(&argv[1])? {
        let result = interp.eval_body(&argv[2]);

        match result {
            Ok(_) => (),
            Err(ResultCode::Break) => break,
            Err(ResultCode::Continue) => (),
            _ => return result,
        }
    }

    molt_ok!()
}
