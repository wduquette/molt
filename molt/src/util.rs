//! Internal Utilities
//!
//! This module contains function for use by molt only.

use crate::char_ptr::CharPtr;

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

pub fn read_int(ptr: &mut CharPtr) -> Option<String> {
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
    while p.is_digit(radix) {
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
/// * Some number of decimal digits, optionally containing a ".".
/// * An optional exponent beginning with "e" or "E"
/// * The exponent may contain a + or -, followed by some number of digits.
///
/// ## Notes
///
/// * The resulting string has the form of a floating point number but might be out of the
///   valid range.
pub fn read_float(ptr: &mut CharPtr) -> Option<String> {
    let mut p = ptr.clone();
    let mut result = String::new();
    let mut missing_mantissa = true;
    let mut missing_exponent = false;

    // FIRST, skip a unary operator.
    if p.is('+') || p.is('-') {
        result.push(p.next().unwrap());
    }

    // NEXT, get any integer digits
    while p.is_digit(10) {
        missing_mantissa = false;
        result.push(p.next().unwrap());
    }

    // NEXT, get any fractional part.
    if p.is('.') {
        result.push(p.next().unwrap());

        while p.is_digit(10) {
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

        while p.is_digit(10) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_util_read_int() {
        let mut p = CharPtr::new("abc");
        assert_eq!(None, read_int(&mut p));
        assert_eq!(Some('a'), p.peek());

        let mut p = CharPtr::new("-abc");
        assert_eq!(None, read_int(&mut p));
        assert_eq!(Some('-'), p.peek());

        let mut p = CharPtr::new("+abc");
        assert_eq!(None, read_int(&mut p));
        assert_eq!(Some('+'), p.peek());

        let mut p = CharPtr::new("123");
        assert_eq!(Some("123".into()), read_int(&mut p));
        assert_eq!(None, p.peek());

        let mut p = CharPtr::new("123abc");
        assert_eq!(Some("123".into()), read_int(&mut p));
        assert_eq!(Some('a'), p.peek());

        let mut p = CharPtr::new("+123abc");
        assert_eq!(Some("+123".into()), read_int(&mut p));
        assert_eq!(Some('a'), p.peek());

        let mut p = CharPtr::new("-123abc");
        assert_eq!(Some("-123".into()), read_int(&mut p));
        assert_eq!(Some('a'), p.peek());
    }

    #[test]
    #[allow(clippy::cyclomatic_complexity)]
    fn test_util_read_float() {
        let mut p = CharPtr::new("abc");
        assert_eq!(None, read_float(&mut p));
        assert_eq!(Some('a'), p.peek());

        let mut p = CharPtr::new("-abc");
        assert_eq!(None, read_float(&mut p));
        assert_eq!(Some('-'), p.peek());

        let mut p = CharPtr::new("+abc");
        assert_eq!(None, read_float(&mut p));
        assert_eq!(Some('+'), p.peek());

        let mut p = CharPtr::new("123");
        assert_eq!(Some("123".into()), read_float(&mut p));
        assert_eq!(None, p.peek());

        let mut p = CharPtr::new("123abc");
        assert_eq!(Some("123".into()), read_float(&mut p));
        assert_eq!(Some('a'), p.peek());

        let mut p = CharPtr::new("123.");
        assert_eq!(Some("123.".into()), read_float(&mut p));
        assert_eq!(None, p.peek());

        let mut p = CharPtr::new(".123");
        assert_eq!(Some(".123".into()), read_float(&mut p));
        assert_eq!(None, p.peek());

        let mut p = CharPtr::new("123.123");
        assert_eq!(Some("123.123".into()), read_float(&mut p));
        assert_eq!(None, p.peek());

        let mut p = CharPtr::new("1e5");
        assert_eq!(Some("1e5".into()), read_float(&mut p));
        assert_eq!(None, p.peek());

        let mut p = CharPtr::new("1e+5");
        assert_eq!(Some("1e+5".into()), read_float(&mut p));
        assert_eq!(None, p.peek());

        let mut p = CharPtr::new("1e-5");
        assert_eq!(Some("1e-5".into()), read_float(&mut p));
        assert_eq!(None, p.peek());

        let mut p = CharPtr::new("1.1e1a");
        assert_eq!(Some("1.1e1".into()), read_float(&mut p));
        assert_eq!(Some('a'), p.peek());

        let mut p = CharPtr::new("+123abc");
        assert_eq!(Some("+123".into()), read_float(&mut p));
        assert_eq!(Some('a'), p.peek());

        let mut p = CharPtr::new("-123abc");
        assert_eq!(Some("-123".into()), read_float(&mut p));
        assert_eq!(Some('a'), p.peek());
    }
}
