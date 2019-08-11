//! Tokenizer is a type used for parsing a `&str` into slices in a way not easily
//! supported by the `Peekable<Chars>` iterator.  The basic procedure is as follows:
//!
//! * Use `next` and `peek` to query the iterator in the usual way.
//! * Detect the beginning of a token and mark it using `mark_head`.
//! * Skip just past the end of the token using `next`, `skip`, etc.
//! * Use `token` to retrieve a slice from the mark to the head.
//!
//! The `next_token` method retrieve the token and sets the mark to the head.

use std::iter::Peekable;
use std::str::Chars;


/// The Tokenizer type.  See the module-level documentation.
#[derive(Clone,Debug)]
pub struct Tokenizer<'a> {
    // The string being parsed.
    input: &'a str,

    // The starting index of the next character.
    head_index: usize,

    // The starting index of the marked character
    mark_index: usize,

    // The iterator used to extract characters from the input
    chars: Peekable<Chars<'a>>,
}

impl<'a> Tokenizer<'a> {
    /// Creates a new tokenizer for the given input.
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            head_index: 0,
            mark_index: 0,
            chars: input.chars().peekable(),
        }
    }

    /// Returns the entire input.
    pub fn input(&self) -> &str {
        self.input
    }

    /// Returns the remainder of the input starting at the mark.
    pub fn mark(&self) -> &str {
        &self.input[self.mark_index..]
    }

    // Returns the remainder of the input starting at the head.
    pub fn head(&self) -> &str {
        // self.chars.as_str()
        &self.input[self.head_index..]
    }

    /// Returns the next character and updates the head.
    pub fn next(&mut self) -> Option<char> {
        let ch = self.chars.next();

        if let Some(c) = ch {
            self.head_index += c.len_utf8();
        }

        ch
    }

    /// Returns the next character without updating the head.
    pub fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    /// Marks the current head as the start of the next token.
    pub fn mark_head(&mut self) {
        self.mark_index = self.head_index;
    }

    /// Get the token between the mark and the head.  Returns None if we're at the
    /// end or mark == head.
    pub fn token(&self) -> Option<&str> {
        if self.mark_index != self.head_index {
            Some(&self.input[self.mark_index..self.head_index])
        } else {
            None
        }
    }

    /// Gets the token between the mark and the head, and marks the head.
    /// Returns None if we're at the end, or mark == head.
    pub fn next_token(&mut self) -> Option<&str> {
        if self.mark_index != self.head_index {
            let token = &self.input[self.mark_index..self.head_index];
            self.mark_index = self.head_index;
            Some(token)
        } else {
            None
        }
    }

    /// Resets the head to the mark.  Use this when it's necessary to look ahead more
    /// than one character.
    pub fn backup(&mut self) {
        self.head_index = self.mark_index;
        self.chars = self.input[self.head_index..].chars().peekable();
    }

    /// Is the next character the given character?  Does not update the head.
    pub fn is(&mut self, ch: char) -> bool {
        if let Some(c) = self.chars.peek() {
            *c == ch
        } else {
            false
        }
    }

    /// Is the predicate true for the next character? Does not update the head.
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

    /// Is there anything left in the input?
    #[allow(clippy::wrong_self_convention)]
    pub fn at_end(&mut self) -> bool {
        // &mut is needed because peek() can mutate the iterator
        self.chars.peek().is_none()
    }

    /// Skip over the next character, updating the head.  This is equivalent to
    /// `next`, but communicates better.
    pub fn skip(&mut self) {
        self.chars.next();
    }

    /// Skips the given number of characters, updating the head.
    /// It is not an error if the iterator doesn't contain that many.
    pub fn skip_over(&mut self, num_chars: usize) {
        for _ in 0..num_chars {
            self.chars.next();
        }
    }

    /// Skips over characters while the predicate is true.  Updates the head.
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basics() {
        // Create the iterator
        let mut ptr = Tokenizer::new("abc");

        // Initial state
        assert_eq!(ptr.input(), "abc");
        assert_eq!(ptr.mark(), "abc");
        assert_eq!(ptr.head(), "abc");
        assert_eq!(ptr.peek(), Some('a'));
        assert_eq!(ptr.token(), None);
    }

    #[test]
    fn test_next() {
        // Create the iterator
        let mut ptr = Tokenizer::new("abc");

        assert_eq!(ptr.next(), Some('a'));
        assert_eq!(ptr.mark(), "abc");
        assert_eq!(ptr.head(), "bc");

        assert_eq!(ptr.next(), Some('b'));
        assert_eq!(ptr.mark(), "abc");
        assert_eq!(ptr.head(), "c");

        assert_eq!(ptr.next(), Some('c'));
        assert_eq!(ptr.mark(), "abc");
        assert_eq!(ptr.head(), "");

        assert_eq!(ptr.next(), None);
    }

    #[test]
    fn test_mark_head() {
        // Create the iterator
        let mut ptr = Tokenizer::new("abcdef");

        ptr.next();
        ptr.next();
        ptr.mark_head();

        assert_eq!(ptr.mark(), "cdef");
        assert_eq!(ptr.head(), "cdef");

        ptr.next();
        ptr.next();
        assert_eq!(ptr.mark(), "cdef");
        assert_eq!(ptr.head(), "ef");
    }

    #[test]
    fn test_token() {
        // Create the iterator
        let mut ptr = Tokenizer::new("abcdef");

        ptr.next();
        ptr.next();
        assert_eq!(ptr.token(), Some("ab"));
        assert_eq!(ptr.head(), "cdef");

        ptr.mark_head();
        ptr.next();
        ptr.next();

        assert_eq!(ptr.token(), Some("cd"));
        assert_eq!(ptr.head(), "ef");
    }

    #[test]
    fn test_next_token() {
        // Create the iterator
        let mut ptr = Tokenizer::new("abcdef");
        assert_eq!(ptr.next_token(), None);

        ptr.next();
        ptr.next();
        assert_eq!(ptr.next_token(), Some("ab"));
        assert_eq!(ptr.mark(), "cdef");

        ptr.next();
        ptr.next();
        assert_eq!(ptr.next_token(), Some("cd"));
        assert_eq!(ptr.mark(), "ef");
    }

    #[test]
    fn test_peek() {
        let mut ptr = Tokenizer::new("abcdef");

        assert_eq!(ptr.peek(), Some('a'));
        assert_eq!(ptr.head(), "abcdef");

        ptr.next();
        ptr.next();

        assert_eq!(ptr.peek(), Some('c'));
        assert_eq!(ptr.head(), "cdef");
    }

    #[test]
    fn test_backup() {
        let mut ptr = Tokenizer::new("abcdef");

        ptr.next();
        ptr.next();
        ptr.backup();

        assert_eq!(ptr.mark(), "abcdef");
        assert_eq!(ptr.head(), "abcdef");
        assert_eq!(ptr.peek(), Some('a'));

        ptr.next();
        ptr.next();
        ptr.mark_head();
        ptr.next();
        ptr.next();
        ptr.backup();

        assert_eq!(ptr.mark(), "cdef");
        assert_eq!(ptr.head(), "cdef");
        assert_eq!(ptr.peek(), Some('c'));
    }

    #[test]
    fn test_is() {
        let mut ptr = Tokenizer::new("a");
        assert!(ptr.is('a'));
        assert!(!ptr.is('b'));
        ptr.next();
        assert!(!ptr.is('a'));
    }

    #[test]
    fn test_has() {
        let mut ptr = Tokenizer::new("a1");
        assert!(ptr.has(|c| c.is_alphabetic()));
        ptr.skip();
        assert!(!ptr.has(|c| c.is_alphabetic()));
        ptr.skip();
        assert!(!ptr.has(|c| c.is_alphabetic()));
    }

    #[test]
    fn test_skip() {
        let mut ptr = Tokenizer::new("abc");

        assert_eq!(Some('a'), ptr.peek());
        ptr.skip();
        assert_eq!(Some('b'), ptr.peek());
        ptr.skip();
        assert_eq!(Some('c'), ptr.peek());
        ptr.skip();
        assert_eq!(None, ptr.peek());
    }

    #[test]
    fn test_skip_over() {
        let mut ptr = Tokenizer::new("abc");
        ptr.skip_over(2);
        assert_eq!(Some('c'), ptr.peek());

        let mut ptr = Tokenizer::new("abc");
        ptr.skip_over(3);
        assert_eq!(None, ptr.peek());

        let mut ptr = Tokenizer::new("abc");
        ptr.skip_over(6);
        assert_eq!(None, ptr.peek());
    }


}
