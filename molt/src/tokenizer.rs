use std::iter::Peekable;
use std::str::Chars;

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
    // Create a new struct for the given input.
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            head_index: 0,
            mark_index: 0,
            chars: input.chars().peekable(),
        }
    }

    // Return the input.
    pub fn input(&self) -> &str {
        self.input
    }

    // Return from the mark to the end.
    pub fn mark(&self) -> &str {
        &self.input[self.mark_index..]
    }

    // Return the remainder as a &str
    pub fn head(&self) -> &str {
        // self.chars.as_str()
        &self.input[self.head_index..]
    }

    // Return the next character. If we've peeked, return the peeked character.
    // Otherwise just get the next one.
    pub fn next(&mut self) -> Option<char> {
        let ch = self.chars.next();

        if let Some(c) = ch {
            self.head_index += c.len_utf8();
        }

        ch
    }

    pub fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    // Start parsing a new token at the current head
    pub fn mark_head(&mut self) {
        self.mark_index = self.head_index;
    }

    // Get the token between the mark and the head.
    pub fn token(&self) -> Option<&str> {
        if self.mark_index != self.head_index {
            Some(&self.input[self.mark_index..self.head_index])
        } else {
            None
        }
    }

    // Get the token between the mark and the head, and update the mark.
    pub fn next_token(&mut self) -> Option<&str> {
        if self.mark_index != self.head_index {
            let token = &self.input[self.mark_index..self.head_index];
            self.mark_index = self.head_index;
            Some(token)
        } else {
            None
        }
    }

    // Resets the head to the mark.
    pub fn backup(&mut self) {
        self.head_index = self.mark_index;
        self.chars = self.input[self.head_index..].chars().peekable();
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
}
