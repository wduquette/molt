//! Internal Utilities
//!
//! This module contains function for use by molt only.

use crate::tokenizer::Tokenizer;
use crate::types::*;
use std::cmp::Ordering;

pub fn is_varname_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

/// Reads the integer string from the head of the input.  If the function returns `Some`,
/// the value is the integer string that was read, and the `ptr` points to the following
/// character. Otherwise the `ptr` will be unchanged.
///
/// The string may consist of:
///
/// * A unary plus or minus
/// * One or more decimal digits.
///
/// ## Notes
///
/// * The resulting string has the form of an integer, but might be out of the valid range.

pub fn read_int(ptr: &mut Tokenizer) -> Option<String> {
    let mut p = ptr.clone();
    let mut result = String::new();
    let mut missing_digits = true;

    // FIRST, skip a unary operator.
    if p.is('+') || p.is('-') {
        result.push(p.next().unwrap());
    }

    // NEXT, skip a "0x".
    let mut radix = 10;

    if p.is('0') {
        result.push(p.next().unwrap());

        if p.is('x') {
            result.push(p.next().unwrap());
            radix = 16;
        } else {
            missing_digits = false;
        }
    }

    // NEXT, read the digits
    while p.has(|ch| ch.is_digit(radix)) {
        missing_digits = false;
        result.push(p.next().unwrap());
    }

    if result.is_empty() || missing_digits {
        None
    } else {
        ptr.skip_over(result.len());
        Some(result)
    }
}

/// Reads the floating point string from the head of the input.  If the function returns `Some`,
/// the value is the string that was read, and the `ptr` points to the following character.
/// Otherwise the `ptr` will be unchanged.
///
/// The string will consist of:
///
/// * Possibly, a unary plus/minus
/// * "Inf" (case insensitive), -OR-
/// * A number:
///   * Some number of decimal digits, optionally containing a ".".
///   * An optional exponent beginning with "e" or "E"
///   * The exponent may contain a + or -, followed by some number of digits.
///
/// ## Notes
///
/// * The resulting string has the form of a floating point number but might be out of the
///   valid range.
pub fn read_float(ptr: &mut Tokenizer) -> Option<String> {
    let mut p = ptr.clone();
    let mut result = String::new();
    let mut missing_mantissa = true;
    let mut missing_exponent = false;

    // FIRST, skip a unary operator.
    if p.is('+') || p.is('-') {
        result.push(p.next().unwrap());
    }

    // NEXT, looking for Inf
    if p.is('I') || p.is('i') {
        result.push(p.next().unwrap());

        if p.is('N') || p.is('n') {
            result.push(p.next().unwrap());
        } else {
            return None;
        }

        if p.is('F') || p.is('f') {
            result.push(p.next().unwrap());
            // Update the pointer.
            ptr.skip_over(result.len());
            return Some(result);
        } else {
            return None;
        }
    }

    // NEXT, get any integer digits
    while p.has(|ch| ch.is_digit(10)) {
        missing_mantissa = false;
        result.push(p.next().unwrap());
    }

    // NEXT, get any fractional part.
    if p.is('.') {
        result.push(p.next().unwrap());

        while p.has(|ch| ch.is_digit(10)) {
            missing_mantissa = false;
            result.push(p.next().unwrap());
        }
    }

    // NEXT, get any exponent.
    if p.is('e') || p.is('E') {
        missing_exponent = true;
        result.push(p.next().unwrap());

        if p.is('+') || p.is('-') {
            result.push(p.next().unwrap());
        }

        while p.has(|ch| ch.is_digit(10)) {
            missing_exponent = false;
            result.push(p.next().unwrap());
        }
    }

    if result.is_empty() || missing_mantissa || missing_exponent {
        None
    } else {
        // Update the pointer.
        ptr.skip_over(result.len());
        Some(result)
    }
}

/// Compare two strings, up to an optional length, returning -1, 0, or 1 as a
/// molt result.
pub(crate) fn compare_len(
    str1: &str,
    str2: &str,
    length: Option<MoltInt>,
) -> Result<MoltInt, Exception> {
    let s1;
    let s2;

    if let Some(len) = length {
        s1 = str1.substring(0, len as usize);
        s2 = str2.substring(0, len as usize);
    } else {
        s1 = str1;
        s2 = str2;
    }

    match s1.cmp(s2) {
        Ordering::Less => Ok(-1),
        Ordering::Equal => Ok(0),
        Ordering::Greater => Ok(1),
    }
}

// From carlomilanesi, rust forums
// https://users.rust-lang.org/t/how-to-get-a-substring-of-a-string/1351/11
use std::ops::{Bound, RangeBounds};

pub(crate) trait StringUtils {
    fn substring(&self, start: usize, len: usize) -> &str;
    fn slice(&self, range: impl RangeBounds<usize>) -> &str;
}

impl StringUtils for str {
    fn substring(&self, start: usize, len: usize) -> &str {
        let mut char_pos = 0;
        let mut byte_start = 0;
        let mut it = self.chars();
        loop {
            if char_pos == start {
                break;
            }
            if let Some(c) = it.next() {
                char_pos += 1;
                byte_start += c.len_utf8();
            } else {
                break;
            }
        }
        char_pos = 0;
        let mut byte_end = byte_start;
        loop {
            if char_pos == len {
                break;
            }
            if let Some(c) = it.next() {
                char_pos += 1;
                byte_end += c.len_utf8();
            } else {
                break;
            }
        }
        &self[byte_start..byte_end]
    }

    fn slice(&self, range: impl RangeBounds<usize>) -> &str {
        let start = match range.start_bound() {
            Bound::Included(bound) | Bound::Excluded(bound) => *bound,
            Bound::Unbounded => 0,
        };
        let len = match range.end_bound() {
            Bound::Included(bound) => *bound + 1,
            Bound::Excluded(bound) => *bound,
            Bound::Unbounded => self.len(),
        } - start;
        self.substring(start, len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_util_read_int() {
        let mut p = Tokenizer::new("abc");
        assert_eq!(None, read_int(&mut p));
        assert_eq!(Some('a'), p.peek());

        let mut p = Tokenizer::new("-abc");
        assert_eq!(None, read_int(&mut p));
        assert_eq!(Some('-'), p.peek());

        let mut p = Tokenizer::new("+abc");
        assert_eq!(None, read_int(&mut p));
        assert_eq!(Some('+'), p.peek());

        let mut p = Tokenizer::new("123");
        assert_eq!(Some("123".into()), read_int(&mut p));
        assert_eq!(None, p.peek());

        let mut p = Tokenizer::new("123abc");
        assert_eq!(Some("123".into()), read_int(&mut p));
        assert_eq!(Some('a'), p.peek());

        let mut p = Tokenizer::new("+123abc");
        assert_eq!(Some("+123".into()), read_int(&mut p));
        assert_eq!(Some('a'), p.peek());

        let mut p = Tokenizer::new("-123abc");
        assert_eq!(Some("-123".into()), read_int(&mut p));
        assert_eq!(Some('a'), p.peek());
    }

    #[test]
    #[allow(clippy::cognitive_complexity)]
    fn test_util_read_float() {
        let mut p = Tokenizer::new("abc");
        assert_eq!(None, read_float(&mut p));
        assert_eq!(Some('a'), p.peek());

        let mut p = Tokenizer::new("-abc");
        assert_eq!(None, read_float(&mut p));
        assert_eq!(Some('-'), p.peek());

        let mut p = Tokenizer::new("+abc");
        assert_eq!(None, read_float(&mut p));
        assert_eq!(Some('+'), p.peek());

        let mut p = Tokenizer::new("123");
        assert_eq!(Some("123".into()), read_float(&mut p));
        assert_eq!(None, p.peek());

        let mut p = Tokenizer::new("123abc");
        assert_eq!(Some("123".into()), read_float(&mut p));
        assert_eq!(Some('a'), p.peek());

        let mut p = Tokenizer::new("123.");
        assert_eq!(Some("123.".into()), read_float(&mut p));
        assert_eq!(None, p.peek());

        let mut p = Tokenizer::new(".123");
        assert_eq!(Some(".123".into()), read_float(&mut p));
        assert_eq!(None, p.peek());

        let mut p = Tokenizer::new("123.123");
        assert_eq!(Some("123.123".into()), read_float(&mut p));
        assert_eq!(None, p.peek());

        let mut p = Tokenizer::new("1e5");
        assert_eq!(Some("1e5".into()), read_float(&mut p));
        assert_eq!(None, p.peek());

        let mut p = Tokenizer::new("1e+5");
        assert_eq!(Some("1e+5".into()), read_float(&mut p));
        assert_eq!(None, p.peek());

        let mut p = Tokenizer::new("1e-5");
        assert_eq!(Some("1e-5".into()), read_float(&mut p));
        assert_eq!(None, p.peek());

        let mut p = Tokenizer::new("1.1e1a");
        assert_eq!(Some("1.1e1".into()), read_float(&mut p));
        assert_eq!(Some('a'), p.peek());

        let mut p = Tokenizer::new("+123abc");
        assert_eq!(Some("+123".into()), read_float(&mut p));
        assert_eq!(Some('a'), p.peek());

        let mut p = Tokenizer::new("-123abc");
        assert_eq!(Some("-123".into()), read_float(&mut p));
        assert_eq!(Some('a'), p.peek());
    }
}
