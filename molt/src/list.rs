//! TCL List Parsing and Formatting

use crate::eval_ptr::EvalPtr;
use crate::interp::subst_backslashes;
use crate::molt_err;
use crate::types::*;
use crate::value::Value;

//--------------------------------------------------------------------------
// List Parsing

/// Parses a list-formatted string into a vector, throwing
/// a Molt error if the list cannot be parsed as a list.
pub(crate) fn get_list(str: &str) -> Result<MoltList, ResultCode> {
    let mut ctx = EvalPtr::new(str);

    parse_list(&mut ctx)
}

// Is the character is valid list whitespace character?
// fn is_list_white(ch: Option<char>) -> bool {
//     match ch {
//         Some(' ') => true,
//         Some('\n') => true,
//         Some('\r') => true,
//         Some('\t') => true,
//         Some('\x0B') => true, // Vertical Tab
//         Some('\x0C') => true, // Form Feed
//         _ => false,
//     }
// }

fn parse_list(ctx: &mut EvalPtr) -> Result<MoltList, ResultCode> {
    // FIRST, skip any list whitespace.
    ctx.skip_list_white();

    // Read words until we get to the end of the input or hit an error
    let mut items = Vec::new();
    while !ctx.at_end() {
        // FIRST, get the next item; there has to be one.
        // Throw an error if there's a formatting problem.
        items.push(parse_item(ctx)?);

        // NEXT, skip whitespace to the end or the next item.
        ctx.skip_list_white();
    }

    // NEXT, return the items.
    Ok(items)
}

/// We're at the beginning of an item in the list.
/// It's either a bare word, a braced string, or a quoted string--or there's
/// an error in the input.  Whichever it is, get it.
fn parse_item(ctx: &mut EvalPtr) -> MoltResult {
    if ctx.next_is('{') {
        Ok(parse_braced_item(ctx)?)
    } else if ctx.next_is('"') {
        Ok(parse_quoted_item(ctx)?)
    } else {
        Ok(parse_bare_item(ctx)?)
    }
}

/// Parse a braced item.
fn parse_braced_item(ctx: &mut EvalPtr) -> MoltResult {
    // FIRST, we have to count braces.  Skip the first one, and count it.
    ctx.next();
    let mut count = 1;
    let mut item = String::new();

    // NEXT, add characters to the item until we find the matching close-brace,
    // which is NOT added to the item.  It's an error if we reach the end before
    // finding the close-brace.
    while let Some(c) = ctx.next() {
        if c == '\\' {
            // Backslash handling. Just include it and the next character as is.
            // Note: this means that escaped '{' and '}' characters
            // don't affect the count.
            item.push('\\');
            if !ctx.at_end() {
                item.push(ctx.next().unwrap());
            }
            continue;
        } else if c == '{' {
            count += 1;
        } else if c == '}' {
            count -= 1;
        }

        if count > 0 {
            item.push(c)
        } else {
            // We've found and consumed the closing brace.  We should either
            // see more more whitespace, or we should be at the end of the list
            // Otherwise, there are incorrect characters following the close-brace.
            if ctx.at_end() || ctx.next_is_list_white() {
                return Ok(Value::from(item));
            } else {
                return molt_err!("extra characters after close-brace");
            }
        }
    }

    assert!(count > 0);
    molt_err!("unmatched open brace in list")
}

/// Parse a quoted item.  Does *not* do backslash substitution.
fn parse_quoted_item(ctx: &mut EvalPtr) -> MoltResult {
    // FIRST, consume the the opening quote.
    ctx.next();

    // NEXT, add characters to the item until we reach the close quote
    let mut item = String::new();

    while !ctx.at_end() {
        // Note: the while condition ensures that there's a character.
        if ctx.next_is('\\') {
            // Backslash; push this character and the next.
            item.push(ctx.next().unwrap());
            if !ctx.at_end() {
                item.push(ctx.next().unwrap());
            }
        } else if !ctx.next_is('"') {
            item.push(ctx.next().unwrap());
        } else {
            ctx.skip_char('"');
            return Ok(Value::from(subst_backslashes(&item)));
        }
    }

    molt_err!("unmatched open quote in list")
}

/// Parse a bare item.  Does *not* do backslash substitution.
fn parse_bare_item(ctx: &mut EvalPtr) -> MoltResult {
    let mut item = String::new();

    while !ctx.at_end() && !ctx.next_is_list_white() {
        // Note: the while condition ensures that there's a character.
        if ctx.next_is('\\') {
            // Backslash; push this character and the next.
            item.push(ctx.next().unwrap());
            if !ctx.at_end() {
                item.push(ctx.next().unwrap());
            }
        } else {
            item.push(ctx.next().unwrap());
        }
    }

    Ok(Value::from(subst_backslashes(&item)))
}

//--------------------------------------------------------------------------
// List Formatting

/// Converts a list, represented as a vector of `Value`s, into a string, doing
/// all necessary quoting and escaping.
pub fn list_to_string(list: &[Value]) -> String {
    let mut vec: Vec<String> = Vec::new();

    let mut hash = !list.is_empty() && list[0].as_str().starts_with('#');

    for item in list {
        let item = item.to_string();
        match get_mode(&item) {
            Mode::AsIs => {
                if hash {
                    vec.push(brace_item(&item));
                    hash = false;
                } else {
                    vec.push(item)
                }
            }
            Mode::Brace => {
                vec.push(brace_item(&item));
            }
            Mode::Escape => {
                vec.push(escape_item(hash, &item));
                hash = false;
            }
        }
    }

    vec.join(" ")
}

fn brace_item(item: &str) -> String {
    let mut word = String::new();
    word.push('{');
    word.push_str(item);
    word.push('}');
    word
}

fn escape_item(hash: bool, item: &str) -> String {
    let mut word = String::new();

    // If hash, the first character is a "#" that must be escaped.
    // Just push the backslash on the front.
    if hash {
        word.push('\\');
    }

    for ch in item.chars() {
        if ch.is_whitespace() {
            word.push('\\');
            word.push(ch);
            continue;
        }

        match ch {
            '{' | ';' | '$' | '[' | ']' | '\\' => {
                word.push('\\');
                word.push(ch);
            }
            _ => word.push(ch),
        }
    }

    word
}

#[derive(Eq, PartialEq, Debug)]
enum Mode {
    AsIs,
    Brace,
    Escape,
}

fn get_mode(word: &str) -> Mode {
    // FIRST, if it's the empty string, just brace it.
    if word.is_empty() {
        return Mode::Brace;
    }

    // NEXT, inspect the content.
    let mut mode = Mode::AsIs;
    let mut brace_count = 0;

    let mut iter = word.chars().peekable();

    while let Some(ch) = iter.next() {
        if ch.is_whitespace() {
            mode = Mode::Brace;
            continue;
        }
        match ch {
            ';' | '$' | '[' | ']' => {
                mode = Mode::Brace;
            }
            '{' => brace_count += 1,
            '}' => brace_count -= 1,
            '\\' => {
                if iter.peek() == Some(&'\n') {
                    return Mode::Escape;
                } else {
                    mode = Mode::Brace;
                }
            }
            _ => (),
        }
    }

    if brace_count != 0 {
        Mode::Escape
    } else {
        mode
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_to_string() {
        assert_eq!(list_to_string(&[Value::from("a")]), "a");
        assert_eq!(list_to_string(&[Value::from("a"), Value::from("b")]), "a b");
        assert_eq!(
            list_to_string(&[Value::from("a"), Value::from("b"), Value::from("c")]),
            "a b c"
        );
        assert_eq!(
            list_to_string(&[Value::from("a"), Value::from(" "), Value::from("c")]),
            "a { } c"
        );
        assert_eq!(
            list_to_string(&[Value::from("a"), Value::from(""), Value::from("c")]),
            "a {} c"
        );
        assert_eq!(list_to_string(&[Value::from("a;b")]), "{a;b}");
        assert_eq!(list_to_string(&[Value::from("a$b")]), "{a$b}");
        assert_eq!(list_to_string(&[Value::from("a[b")]), "{a[b}");
        assert_eq!(list_to_string(&[Value::from("a]b")]), "{a]b}");
        assert_eq!(list_to_string(&[Value::from("a\\nb")]), "{a\\nb}");
        assert_eq!(
            list_to_string(&[Value::from("{ "), Value::from("abc")]),
            r#"\{\  abc"#
        );
    }

    // Most list parsing is tested in the Molt test suite.

    #[test]
    fn test_issue_43() {
        let list = get_list("a ;b c").unwrap();

        // If the list breaks on the semi-colon, the bug still exists.
        assert_eq!(list.len(), 3);
    }
}
