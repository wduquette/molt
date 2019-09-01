//! Experimental code.

use crate::eval_ptr::EvalPtr;
use crate::types::ResultCode;
use crate::value::Value;
use crate::util::is_varname_char;

/// A single command in the script.
type Command = Vec<Word>;

/// A single word in a command
type Word = Vec<Token>;

/// A token within a word.
enum Token {
    String(String),
    VarRef(String),
    Script(Script),
}

/// A compiled script, which can be executed in the context of an interpreter.
pub struct Script {
    commands: Vec<Command>,
}

impl Script {
    fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }
}


pub fn parse(input: &str) -> Result<Script,ResultCode> {
    let mut ctx = EvalPtr::new(input);
    parse_script(&mut ctx)
}

fn parse_script(ctx: &mut EvalPtr) -> Result<Script, ResultCode> {
    let mut script = Script::new();

    while !ctx.at_end_of_script() {
        script.commands.push(parse_command(ctx)?);
    }

    Ok(script)
}

fn parse_command(ctx: &mut EvalPtr) -> Result<Command, ResultCode> {
    let mut cmd: Command = Vec::new();

    // FIRST, deal with whitespace and comments between "here" and the next command.
    while !ctx.at_end_of_script() {
        ctx.skip_block_white();

        // Either there's a comment, or we're at the beginning of the next command.
        // If the former, skip the comment; then check for more whitespace and comments.
        // Otherwise, go on to the command.
        if !ctx.skip_comment() {
            break;
        }
    }

    // Read words until we get to the end of the line or hit an error
    // NOTE: parse_word() can always assume that it's at the beginning of a word.
    while !ctx.at_end_of_command() {
        // FIRST, get the next word; there has to be one, or there's an input error.
        let word = if ctx.next_is('{') {
            parse_braced_word(ctx)?
        } else if ctx.next_is('"') {
            parse_quoted_word(ctx)?
        } else {
            parse_bare_word(ctx)?
        };

        // TODO: Adjacent Token::Value entries should be concatenated.

        cmd.push(word);

        // NEXT, skip any whitespace.
        ctx.skip_line_white();
    }

    // If we ended at a ";", consume the semi-colon.
    if ctx.next_is(';') {
        ctx.next();
    }

    Ok(cmd)
}

fn parse_braced_word(ctx: &mut EvalPtr) -> Result<Word, ResultCode> {
    // FIRST, skip the opening brace, and count it; non-escaped braces need to
    // balance.
    ctx.skip_char('{');
    let mut count = 1;

    // NEXT, add tokens to the word until we reach the close quote
    let mut word = String::new();
    let mut start = ctx.mark();

    while !ctx.at_end() {
        // Note: the while condition ensures that there's a character.
        if ctx.next_is('{') {
            count += 1;
            ctx.skip();
        } else if ctx.next_is('}') {
            count -= 1;

            if count > 0 {
                ctx.skip();
            } else {
                // We've found and consumed the closing brace.  We should either
                // see more more whitespace, or we should be at the end of the list
                // Otherwise, there are incorrect characters following the close-brace.
                word.push_str(ctx.token(start));
                let result = Ok(vec![Token::String(word)]);
                ctx.skip(); // Skip the closing brace

                if ctx.at_end_of_command() || ctx.next_is_line_white() {
                    return result;
                } else {
                    return molt_err!("extra characters after close-brace");
                }
            }
        } else if ctx.next_is('\\') {
            word.push_str(ctx.token(start));
            ctx.skip();

            // If there's no character it's because we're at the end; and there's
            // no close brace.
            if let Some(ch) = ctx.next() {
                if ch == '\n' {
                    word.push(' ');
                } else {
                    word.push('\\');
                    word.push(ch);
                }
            }
            start = ctx.mark();
        } else {
            ctx.skip();
        }
    }

    molt_err!("missing close-brace")
}

/// Parse a quoted word.
fn parse_quoted_word(ctx: &mut EvalPtr) -> Result<Word, ResultCode> {
    // FIRST, consume the the opening quote.
    ctx.next();

    // NEXT, add tokens to the word until we reach the close quote
    let mut word: Word = Vec::new();
    let mut start = ctx.mark();

    while !ctx.at_end() {
        // Note: the while condition ensures that there's a character.
        if ctx.next_is('[') {
            word.push(Token::String(ctx.token(start).to_string()));
            word.push(Token::Script(parse_brackets(ctx)?));
            start = ctx.mark();
        } else if ctx.next_is('$') {
            word.push(Token::String(ctx.token(start).to_string()));
            word.push(parse_varname(ctx)?);
            start = ctx.mark();
        } else if ctx.next_is('\\') {
            word.push(Token::String(ctx.token(start).to_string()));
            word.push(Token::String(ctx.backslash_subst().to_string()));
            start = ctx.mark();
        } else if ctx.next_is('"') {
            word.push(Token::String(ctx.token(start).to_string()));
            ctx.skip_char('"');
            if !ctx.at_end_of_command() && !ctx.next_is_line_white() {
                return molt_err!("extra characters after close-quote");
            } else {
                return Ok(word);
            }
        } else {
            ctx.skip();
        }
    }

    molt_err!("missing \"")
}

/// Parse a bare word.
fn parse_bare_word(ctx: &mut EvalPtr) -> Result<Word, ResultCode> {
    let mut word: Word = Vec::new();
    let mut start = ctx.mark();

    while !ctx.at_end_of_command() && !ctx.next_is_line_white() {
        // Note: the while condition ensures that there's a character.
        if ctx.next_is('[') {
            word.push(Token::String(ctx.token(start).to_string()));
            word.push(Token::Script(parse_brackets(ctx)?));
            start = ctx.mark();
        } else if ctx.next_is('$') {
            word.push(Token::String(ctx.token(start).to_string()));
            word.push(parse_varname(ctx)?);
            start = ctx.mark();
        } else if ctx.next_is('\\') {
            word.push(Token::String(ctx.token(start).to_string()));
            word.push(Token::String(ctx.backslash_subst().to_string()));
            start = ctx.mark();
        } else {
            ctx.skip();
        }
    }

    word.push(Token::String(ctx.token(start).to_string()));

    Ok(word)
}

fn parse_brackets(ctx: &mut EvalPtr) -> Result<Script, ResultCode> {
    // FIRST, skip the '['
    ctx.skip_char('[');

    // NEXT, parse the script up to the matching ']'
    let old_flag = ctx.is_bracket_term();
    ctx.set_bracket_term(true);
    let result = parse_script(ctx);
    ctx.set_bracket_term(old_flag);

    // NEXT, make sure there's a closing bracket
    if result.is_ok() {
        if ctx.next_is(']') {
            ctx.next();
        } else {
            return molt_err!("missing close-bracket");
        }
    }

    result
}

fn parse_varname(ctx: &mut EvalPtr) -> Result<Token, ResultCode> {
    // FIRST, skip the '$'
    ctx.skip_char('$');

    // NEXT, make sure this is really a variable reference.  If it isn't
    // just return a "$".
    if !ctx.next_is_varname_char() && !ctx.next_is('{') {
        return Ok(Token::String("$".into()));
    }

    // NEXT, is this a braced variable name?
    let var_name;

    if ctx.next_is('{') {
        ctx.skip_char('{');
        let start = ctx.mark();
        ctx.skip_while(|ch| *ch != '}');

        if ctx.at_end() {
            return molt_err!("missing close-brace for variable name");
        }

        var_name = ctx.token(start).to_string();
        ctx.skip_char('}');
    } else {
        let start = ctx.mark();
        ctx.skip_while(|ch| is_varname_char(*ch));
        var_name = ctx.token(start).to_string();
    }

    Ok(Token::VarRef(var_name))
}
