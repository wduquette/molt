//! A character iterator that mimics a C (char*) pointer into a buffer.

use std::iter::Peekable;
use std::str::Chars;

#[derive(Clone)]
pub struct CharPtr<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> CharPtr<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().peekable(),
        }
    }

    /// Converts a Peekable<Chars> into a CharPtr.
    pub fn from_peekable(chars: Peekable<Chars<'a>>) -> Self {
        Self { chars }
    }

    /// Converts a CharPtr into a Peekable<Chars>.
    pub fn to_peekable(&self) -> Peekable<Chars<'a>> {
        self.chars.clone()
    }

    pub fn skip_while<P>(&mut self, predicate: P)
    where
        P: Fn(&char) -> bool,
    {
        while let Some(ch) = self.chars.peek() {
            if predicate(ch) {
                self.chars.next();
            } else {
                break;
            }
        }
    }

    pub fn skip(&mut self) {
        self.chars.next();
    }

    /// Skips the given number of characters.
    /// It is not an error if the iterator doesn't contain that many.
    pub fn skip_over(&mut self, num_chars: usize) {
        for _ in 0..num_chars {
            self.chars.next();
        }
    }

    pub fn next(&mut self) -> Option<char> {
        self.chars.next()
    }

    pub fn peek(&mut self) -> Option<char> {
        if let Some(pc) = self.chars.peek() {
            Some(*pc)
        } else {
            None
        }
    }

    pub fn is(&mut self, ch: char) -> bool {
        if let Some(c) = self.chars.peek() {
            *c == ch
        } else {
            false
        }
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn is_none(&mut self) -> bool {
        // &mut is needed because peek() can mutate the iterator
        self.chars.peek().is_none()
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn is_digit(&mut self, radix: u32) -> bool {
        // &mut is needed because peek() can mutate the iterator
        if let Some(pc) = self.chars.peek() {
            pc.is_digit(radix)
        } else {
            false
        }
    }

    pub fn has<P>(&mut self, predicate: P) -> bool
    where
        P: Fn(&char) -> bool,
    {
        if let Some(ch) = self.chars.peek() {
            predicate(ch)
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_ptr_peek_next() {
        let mut p = CharPtr::new("abc");

        assert_eq!(Some('a'), p.peek());
        assert_eq!(Some('a'), p.next());
        assert_eq!(Some('b'), p.peek());
        assert_eq!(Some('b'), p.next());
        assert_eq!(Some('c'), p.peek());
        assert_eq!(Some('c'), p.next());
        assert_eq!(None, p.peek());
        assert_eq!(None, p.next());
    }

    #[test]
    fn test_char_ptr_skip() {
        let mut p = CharPtr::new("abc");

        assert_eq!(Some('a'), p.peek());
        p.skip();
        assert_eq!(Some('b'), p.peek());
        p.skip();
        assert_eq!(Some('c'), p.peek());
        p.skip();
        assert_eq!(None, p.peek());
    }

    #[test]
    fn test_char_ptr_skip_over() {
        let mut p = CharPtr::new("abc");
        p.skip_over(2);
        assert_eq!(Some('c'), p.peek());

        let mut p = CharPtr::new("abc");
        p.skip_over(3);
        assert_eq!(None, p.peek());

        let mut p = CharPtr::new("abc");
        p.skip_over(6);
        assert_eq!(None, p.peek());
    }

    #[test]
    fn test_char_ptr_is() {
        let mut p = CharPtr::new("a");
        assert!(p.is('a'));
        assert!(!p.is('b'));
        p.skip();
        assert!(!p.is('a'));
    }

    #[test]
    fn test_char_ptr_skip_while() {
        let mut p = CharPtr::new("   abc");
        assert!(p.is(' '));
        p.skip_while(|c| c.is_whitespace());
        assert!(p.is('a'));
        p.skip_while(|c| c.is_whitespace());
        assert!(p.is('a'));
    }

    #[test]
    fn test_char_ptr_is_none() {
        let mut p = CharPtr::new("a");
        assert!(!p.is_none());
        p.skip();
        assert!(p.is_none());
    }

    #[test]
    fn test_char_ptr_is_digit() {
        let mut p = CharPtr::new("1a");
        assert!(p.is_digit(10));
        p.skip();
        assert!(!p.is_digit(10));
        p.skip();
        assert!(!p.is_digit(10));

        let mut p = CharPtr::new("1a");
        assert!(p.is_digit(16));
        p.skip();
        assert!(p.is_digit(16));
        p.skip();
        assert!(!p.is_digit(16));
    }

    #[test]
    fn test_char_ptr_has() {
        let mut p = CharPtr::new("a1");
        assert!(p.has(|c| c.is_alphabetic()));
        p.skip();
        assert!(!p.has(|c| c.is_alphabetic()));
        p.skip();
        assert!(!p.has(|c| c.is_alphabetic()));
    }
}
