use std::str::Chars;

use std::iter::Peekable;

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

    pub fn skip_while<P>(&mut self, predicate: P)
        where P: Fn(&char) -> bool
    {
        while let Some(ch) = self.chars.peek() {
            if predicate(ch) {
                self.chars.next();
            } else {
                break;
            }
        }
    }

    pub fn is(&mut self, ch: char) -> bool {
        if let Some(c) = self.chars.peek() {
            *c == ch
        } else {
            false
        }
    }

    pub fn skip(&mut self) {
        self.chars.next();
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
}
