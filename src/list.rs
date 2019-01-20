//! TCL List Parsing and Formatting

use crate::types::*;
use crate::error;
use crate::context::Context;
use crate::interp::subst_backslashes;

/// Parses a list-formatted string into a vector, throwing
/// a Molt error if the list cannot be parsed as a list.
pub fn get_list(str: &str) -> Result<Vec<String>, ResultCode> {
    let mut ctx = Context::new(str);

    parse_list(&mut ctx)
}

fn parse_list(ctx: &mut Context) -> Result<Vec<String>, ResultCode> {
    // FIRST, skip any list whitespace.
    ctx.skip_list_white();

    // Read words until we get to the end of the input or hit an error
    let mut items = Vec::new();
    while !ctx.at_end_of_command() {
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
fn parse_item(ctx: &mut Context) -> InterpResult {
    if ctx.next_is('{') {
        Ok(parse_braced_item(ctx)?)
    } else if ctx.next_is('"') {
        Ok(subst_backslashes(&parse_quoted_item(ctx)?))
    } else {
        Ok(subst_backslashes(&parse_bare_item(ctx)?))
    }
}

/// Parse a braced item.
fn parse_braced_item(ctx: &mut Context) -> InterpResult {
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
                return Ok(item);
            } else {
                return error("extra characters after close-brace");
            }
        }
    }

    assert!(count > 0);
    error("missing close-brace")
}

/// Parse a quoted item.  Does *not* do backslash substitution.
fn parse_quoted_item(ctx: &mut Context) -> InterpResult {
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
            return Ok(item);
        }
    }

    error("missing \"")
}

/// Parse a bare item.  Does *not* do backslash substitution.
fn parse_bare_item(ctx: &mut Context) -> InterpResult {
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

    Ok(item)
}
