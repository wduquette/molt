//! Experimental code.

use crate::check_args;
use crate::eval_ptr::EvalPtr;
use crate::interp::Interp;
use crate::types::MoltResult;
use crate::types::ResultCode;
use crate::util::is_varname_char;
use crate::value::Value;

/// A compiled script, which can be executed in the context of an interpreter.
#[derive(Debug, PartialEq)]
pub struct Script {
    // A script is a list of one or more commands to execute.
    commands: Vec<Command>,
}

impl Script {
    fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }
}

/// A command is a list of words.  This represents a single command in a Script.
type Command = Vec<Word>;

#[derive(Debug, PartialEq)]
enum Word {
    Value(Value),
    VarRef(String),
    Script(Script),
    Tokens(Vec<Word>),
    String(String), // Only used in Tokens
}

pub fn parse(input: &str) -> Result<Script, ResultCode> {
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
        let word: Word = if ctx.next_is('{') {
            parse_braced_word(ctx)?
        } else if ctx.next_is('"') {
            parse_quoted_word(ctx)?
        } else {
            parse_bare_word(ctx)?
        };

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
    let mut text = String::new();
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
                text.push_str(ctx.token(start));
                let result = Ok(Word::String(text));
                ctx.skip(); // Skip the closing brace

                if ctx.at_end_of_command() || ctx.next_is_line_white() {
                    return result;
                } else {
                    return molt_err!("extra characters after close-brace");
                }
            }
        } else if ctx.next_is('\\') {
            text.push_str(ctx.token(start));
            ctx.skip();

            // If there's no character it's because we're at the end; and there's
            // no close brace.
            if let Some(ch) = ctx.next() {
                if ch == '\n' {
                    text.push(' ');
                } else {
                    text.push('\\');
                    text.push(ch);
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
    let mut tokens = Tokens::new();
    let mut start = ctx.mark();

    while !ctx.at_end() {
        // Note: the while condition ensures that there's a character.
        if ctx.next_is('[') {
            if start != ctx.mark() {
                tokens.push_str(ctx.token(start));
            }
            tokens.push(Word::Script(parse_brackets(ctx)?));
            start = ctx.mark();
        } else if ctx.next_is('$') {
            if start != ctx.mark() {
                tokens.push_str(ctx.token(start));
            }
            parse_varname(ctx, &mut tokens)?;
            start = ctx.mark();
        } else if ctx.next_is('\\') {
            if start != ctx.mark() {
                tokens.push_str(ctx.token(start));
            }
            tokens.push_char(ctx.backslash_subst());
            start = ctx.mark();
        } else if ctx.next_is('"') {
            if start != ctx.mark() {
                tokens.push_str(ctx.token(start));
            }
            ctx.skip_char('"');
            if !ctx.at_end_of_command() && !ctx.next_is_line_white() {
                return molt_err!("extra characters after close-quote");
            } else {
                return Ok(tokens.take());
            }
        } else {
            ctx.skip();
        }
    }

    molt_err!("missing \"")
}

/// Parse a bare word.
fn parse_bare_word(ctx: &mut EvalPtr) -> Result<Word, ResultCode> {
    let mut tokens = Tokens::new();
    let mut start = ctx.mark();

    while !ctx.at_end_of_command() && !ctx.next_is_line_white() {
        // Note: the while condition ensures that there's a character.
        if ctx.next_is('[') {
            if start != ctx.mark() {
                tokens.push_str(ctx.token(start));
            }
            tokens.push(Word::Script(parse_brackets(ctx)?));
            start = ctx.mark();
        } else if ctx.next_is('$') {
            if start != ctx.mark() {
                tokens.push_str(ctx.token(start));
            }
            parse_varname(ctx, &mut tokens)?;
            start = ctx.mark();
        } else if ctx.next_is('\\') {
            if start != ctx.mark() {
                tokens.push_str(ctx.token(start));
            }
            tokens.push_char(ctx.backslash_subst());
            start = ctx.mark();
        } else {
            ctx.skip();
        }
    }

    if start != ctx.mark() {
        tokens.push_str(ctx.token(start));
    }

    Ok(tokens.take())
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

fn parse_varname(ctx: &mut EvalPtr, tokens: &mut Tokens) -> Result<(), ResultCode> {
    // FIRST, skip the '$'
    ctx.skip_char('$');

    // NEXT, make sure this is really a variable reference.  If it isn't
    // just return a "$".
    if !ctx.next_is_varname_char() && !ctx.next_is('{') {
        tokens.push_char('$');
        return Ok(());
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

    tokens.push(Word::VarRef(var_name));
    Ok(())
}

struct Tokens {
    list: Vec<Word>,
    got_string: bool,
    string: String,
}

impl Tokens {
    fn new() -> Self {
        Self {
            list: Vec::new(),
            got_string: false,
            string: String::new(),
        }
    }

    fn push(&mut self, word: Word) {
        if self.got_string {
            let string = std::mem::replace(&mut self.string, String::new());
            self.list.push(Word::String(string));
            self.got_string = false;
        }

        self.list.push(word);
    }

    fn push_str(&mut self, str: &str) {
        self.string.push_str(str);
        self.got_string = true;
    }

    fn push_char(&mut self, ch: char) {
        self.string.push(ch);
        self.got_string = true;
    }

    fn take(mut self) -> Word {
        if self.got_string {
            // If there's nothing but the string, turn it into a value.
            // Otherwise, just add it to the list of tokens.
            if self.list.is_empty() {
                return Word::Value(Value::from(self.string));
            } else {
                let string = std::mem::replace(&mut self.string, String::new());
                self.list.push(Word::String(string));
            }
        }

        if self.list.is_empty() {
            Word::Value(Value::empty())
        } else if self.list.len() == 1 {
            self.list.pop().unwrap()
        } else {
            Word::Tokens(self.list)
        }
    }
}

/// # parse *script*
pub fn cmd_parse(_interp: &mut Interp, argv: &[Value]) -> MoltResult {
    check_args(1, argv, 2, 2, "script")?;

    let script = &argv[1];

    molt_ok!(format!("{:?}", parse(script.as_str())?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokens() {
        // No tokens pushed; get empty string.
        let tokens = Tokens::new();
        assert_eq!(tokens.take(), Word::Value(Value::empty()));

        // Push normal Word only; get it back.
        let mut tokens = Tokens::new();
        tokens.push(Word::Value(Value::from("abc")));
        assert_eq!(tokens.take(), Word::Value(Value::from("abc")));

        // Push a single str.  Get Value.
        let mut tokens = Tokens::new();
        tokens.push_str("xyz");
        assert_eq!(tokens.take(), Word::Value(Value::from("xyz")));

        // Push two strs.  Get one value.
        let mut tokens = Tokens::new();
        tokens.push_str("abc");
        tokens.push_str("def");
        assert_eq!(tokens.take(), Word::Value(Value::from("abcdef")));

        // Push strs and chars.  Get one value.
        let mut tokens = Tokens::new();
        tokens.push_str("abc");
        tokens.push_char('/');
        tokens.push_str("def");
        assert_eq!(tokens.take(), Word::Value(Value::from("abc/def")));

        // Push multiple normal words
        let mut tokens = Tokens::new();
        tokens.push(Word::VarRef("a".into()));
        tokens.push(Word::String("xyz".into()));
        assert_eq!(
            tokens.take(),
            Word::Tokens(vec![Word::VarRef("a".into()), Word::String("xyz".into())])
        );

        // Push a string, a word, and another string
        let mut tokens = Tokens::new();
        tokens.push_str("a");
        tokens.push_str("b");
        tokens.push(Word::VarRef("xyz".into()));
        tokens.push_str("c");
        tokens.push_str("d");
        assert_eq!(
            tokens.take(),
            Word::Tokens(vec![
                Word::String("ab".into()),
                Word::VarRef("xyz".into()),
                Word::String("cd".into())
            ])
        );
    }
}
