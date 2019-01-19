//! The parsing context

use std::iter::Peekable;
use std::str::Chars;

/// A struct that holds the editing context: the iterator over the input string, and
/// any relevant flags.
pub struct Context<'a> {
    // The input iterator
    chars: Peekable<Chars<'a>>,

    // Whether we're looking for a bracket or not.
    bracket_term: bool,

    // The term_char: the character that ends the script, None or Some<']'>
    term_char: Option<&'a char>,
}

impl<'a> Context<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().peekable(),
            bracket_term: false,
            term_char: None,
        }
    }

    //-----------------------------------------------------------------------
    // Configuration

    /// If true, the script ends with a right-bracket, ']'; otherwise it ends
    /// at the end of the input.
    pub fn set_bracket_term(&mut self, flag: bool) {
        self.bracket_term = flag;
        self.term_char = if flag { Some(&']') } else { None };
    }

    pub fn is_bracket_term(&self) -> bool {
        self.bracket_term
    }

    //-----------------------------------------------------------------------
    // Helpers

    /// Sees if the next character is the given character.
    pub fn next_is(&mut self, char: char) -> bool {
        self.chars.peek() == Some(&char)
    }

    /// We are at the end of the input when there are no more characters left.
    pub fn at_end(&mut self) -> bool {
        self.chars.peek() == None
    }

    /// We are at the end of the script when we've reached the end-of-script marker
    /// or we are at the end of the input.
    pub fn at_end_of_script(&mut self) -> bool {
        self.chars.peek() == self.term_char || self.chars.peek() == None
    }

    /// We are at the end of the command if we've reached a semi-colon or new-line, or
    /// we are at the end of the script.
    pub fn at_end_of_command(&mut self) -> bool {
        self.next_is('\n') || self.next_is(';') || self.at_end_of_script()
    }

    /// Is the current character is a valid whitespace character, including newlines?
    pub fn next_is_block_white(&mut self) -> bool {
        match self.chars.peek() {
            Some(c) => c.is_whitespace(),
            None => false,
        }
    }

    /// Is the current character is a valid whitespace character, excluding newlines?
    pub fn next_is_line_white(&mut self) -> bool {
        match self.chars.peek() {
            Some(c) => c.is_whitespace() && *c != '\n',
            None => false,
        }
    }

    /// Skips past any whitespace at the current point, *including* newlines.
    /// When this is complete we will be at the end of the script or on a non-white-space
    /// character.
    pub fn skip_block_white(&mut self) {
        while !self.at_end() && self.next_is_block_white() {
            self.chars.next();
        }
    }

    /// Skips past any whitespace on the current line, thus *excluding* newlines.
    /// When this is complete we will be at the end of the script, at the end of the
    /// current command, or on a non-white-space character.
    pub fn skip_line_white(&mut self) {
        while !self.at_end() && self.next_is_line_white() {
            self.chars.next();
        }
    }

    /// Skips past a comment if there is one, including any terminating newline.
    /// Returns true if it skipped a comment, and false otherwise.
    ///
    /// TODO: Handle backslashes
    pub fn skip_comment(&mut self) -> bool {
        if self.next_is('#') {
            while !self.at_end() {
                if self.chars.next() == Some('\n') {
                    break;
                }
            }
            true
        } else {
            false
        }
    }

    /// Skip a specific character
    pub fn skip_char(&mut self, ch: char) {
        let c = self.chars.next();
        assert!(c == Some(ch), "expected '{}', got '{:?}' ", ch, c);
    }

    /// Get the next character.
    pub fn next(&mut self) -> Option<char> {
        self.chars.next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bracket_term() {
        let mut ctx = Context::new("123");

        assert!(!ctx.is_bracket_term());
        ctx.set_bracket_term(true);
        assert!(ctx.is_bracket_term());
    }

    #[test]
    fn test_next_is() {
        let mut ctx = Context::new("123");
        assert!(ctx.next_is('1'));
        assert!(!ctx.next_is('2'));

        let mut ctx = Context::new("");
        assert!(!ctx.next_is('1'));
    }

    #[test]
    fn test_at_end() {
        let mut ctx = Context::new("123");
        assert!(!ctx.at_end());

        let mut ctx = Context::new("");
        assert!(ctx.at_end());
    }

    #[test]
    fn test_at_end_of_script() {
        let mut ctx = Context::new("123");
        assert!(!ctx.at_end_of_script());

        let mut ctx = Context::new("");
        assert!(ctx.at_end_of_script());

        let mut ctx = Context::new("]");
        assert!(!ctx.at_end_of_script());
        ctx.set_bracket_term(true);
        assert!(ctx.at_end_of_script());
    }

    #[test]
    fn test_at_end_of_command() {
        let mut ctx = Context::new("123");
        assert!(!ctx.at_end_of_command());

        let mut ctx = Context::new(";123");
        assert!(ctx.at_end_of_command());

        let mut ctx = Context::new("\n123");
        assert!(ctx.at_end_of_command());

        let mut ctx = Context::new("]123");
        assert!(!ctx.at_end_of_command());

        let mut ctx = Context::new("]123");
        ctx.set_bracket_term(true);
        assert!(ctx.at_end_of_command());
    }

    #[test]
    fn test_next_is_block_white() {
        let mut ctx = Context::new("123");
        assert!(!ctx.next_is_block_white());

        let mut ctx = Context::new(" 123");
        assert!(ctx.next_is_block_white());

        let mut ctx = Context::new("\n123");
        assert!(ctx.next_is_block_white());
    }

    #[test]
    fn test_skip_block_white() {
        let mut ctx = Context::new("123");
        ctx.skip_block_white();
        assert!(ctx.next_is('1'));

        let mut ctx = Context::new("   123");
        ctx.skip_block_white();
        assert!(ctx.next_is('1'));

        let mut ctx = Context::new(" \n 123");
        ctx.skip_block_white();
        assert!(ctx.next_is('1'));
    }

    #[test]
    fn test_next_is_line_white() {
        let mut ctx = Context::new("123");
        assert!(!ctx.next_is_line_white());

        let mut ctx = Context::new(" 123");
        assert!(ctx.next_is_line_white());

        let mut ctx = Context::new("\n123");
        assert!(!ctx.next_is_line_white());
    }

    #[test]
    fn test_skip_line_white() {
        let mut ctx = Context::new("123");
        ctx.skip_line_white();
        assert!(ctx.next_is('1'));

        let mut ctx = Context::new("   123");
        ctx.skip_line_white();
        assert!(ctx.next_is('1'));

        let mut ctx = Context::new(" \n 123");
        ctx.skip_line_white();
        assert!(ctx.next_is('\n'));
    }

    #[test]
    fn test_skip_comment() {
        let mut ctx = Context::new("123");
        assert!(!ctx.skip_comment());
        assert!(ctx.next_is('1'));

        let mut ctx = Context::new(" #123");
        assert!(!ctx.skip_comment());
        assert!(ctx.next_is(' '));

        let mut ctx = Context::new("#123");
        assert!(ctx.skip_comment());
        assert!(ctx.at_end());

        let mut ctx = Context::new("#1 2 3 \na");
        assert!(ctx.skip_comment());
        assert!(ctx.next_is('a'));

        let mut ctx = Context::new("#1 2] 3 \na");
        ctx.set_bracket_term(true);
        assert!(ctx.skip_comment());
        assert!(ctx.next_is('a'));
    }

}
