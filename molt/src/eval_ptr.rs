//! The parsing context
//!
//! # TODO
//!
//! * Consider delegating skip_while() to iter::skip_while(), and replacing the
//!   "skip_sequence" methods with some useful predicate functions.

use crate::tokenizer::Tokenizer;

/// A struct that holds the parsing context: the iterator over the input string, and
/// any relevant flags.
pub struct EvalPtr<'a> {
    // The input iterator
    tok: Tokenizer<'a>,

    // Whether we're looking for a bracket or not.
    bracket_term: bool,

    // The term_char: the character that ends the script, None or Some<']'>
    term_char: Option<char>,

    // Whether we're evaluating commands or just checking for completeness.
    no_eval: bool,
}

impl<'a> EvalPtr<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            tok: Tokenizer::new(input),
            bracket_term: false,
            term_char: None,
            no_eval: false,
        }
    }

    /// Returns a mutable reference to the inner tokenizer.
    pub fn tok(&mut self) -> &mut Tokenizer<'a> {
        &mut self.tok
    }

    // TEMPORARY; used by `expr`. remove in favor of tok()
    pub fn from_tokenizer(ptr: &Tokenizer<'a>) -> Self {
        Self {
            tok: ptr.clone(),
            bracket_term: false,
            term_char: None,
            no_eval: false,
        }
    }

    // TEMPORARY; used by `expr`. remove in favor of tok()
    pub fn to_tokenizer(&self) -> Tokenizer<'a> {
        self.tok.clone()
    }

    //-----------------------------------------------------------------------
    // Configuration

    /// If true, the script ends with a right-bracket, ']'; otherwise it ends
    /// at the end of the input.
    pub fn set_bracket_term(&mut self, flag: bool) {
        self.bracket_term = flag;
        self.term_char = if flag { Some(']') } else { None };
    }

    // Returns whether or not the input ends with ']', i.e., at the end of the
    // an interpolated script.
    pub fn is_bracket_term(&self) -> bool {
        self.bracket_term
    }

    // Sets/clears "no eval" mode.  In "no eval" mode we scan the input for
    // validity, e.g., no unmatched braces, brackets, or quotes.
    pub fn set_no_eval(&mut self, flag: bool) {
        self.no_eval = flag;
    }

    // Returns whether or not we are scanning the input for completeness.
    pub fn is_no_eval(&self) -> bool {
        self.no_eval
    }

    //-----------------------------------------------------------------------
    // Tokenizer methods

    /// Get the next character.
    pub fn next(&mut self) -> Option<char> {
        self.tok.next()
    }

    /// Sees if the next character is the given character.
    // TODO: Change to "is".
    pub fn next_is(&mut self, ch: char) -> bool {
        self.tok.is(ch)
    }

    /// We are at the end of the input when there are no more characters left.
    pub fn at_end(&mut self) -> bool {
        self.tok.at_end()
    }

    /// Skip the next character
    pub fn skip(&mut self) {
        self.tok.skip();
    }

    /// Skip a specific character
    pub fn skip_char(&mut self, ch: char) {
        self.tok.skip_char(ch);
    }
    /// Skips over characters while the predicate is true.  Updates the index.
    pub fn skip_while<P>(&mut self, predicate: P)
    where
        P: Fn(&char) -> bool,
    {
        self.tok.skip_while(predicate);
    }

    // Returns the current index as a mark, for later use.
    pub fn mark(&mut self) -> usize {
        self.tok.mark()
    }

    /// Get the token between the mark and the index.  Returns "" if we're at the
    /// end or mark == index.
    pub fn token(&self, mark: usize) -> &str {
        self.tok.token(mark)
    }

    /// Parses a backslash-escape and returns its value. If the escape is valid,
    /// the value will be the substituted character.  If the escape is not valid,
    /// it will be the single character following the backslash.  Either way, the
    /// the index will point at what's next.  If there's nothing following the backslash,
    /// return the backslash.
    pub fn backslash_subst(&mut self) -> char {
        self.tok.backslash_subst()
    }

    //-----------------------------------------------------------------------
    // Parsing Helpers

    /// We are at the end of the script when we've reached the end-of-script marker
    /// or we are at the end of the input.
    pub fn at_end_of_script(&mut self) -> bool {
        self.tok.at_end() || self.tok.peek() == self.term_char
    }

    /// We are at the end of the command if we've reached a semi-colon or new-line, or
    /// we are at the end of the script.
    pub fn at_end_of_command(&mut self) -> bool {
        self.next_is('\n') || self.next_is(';') || self.at_end_of_script()
    }

    /// Is the current character is a valid whitespace character, including newlines?
    pub fn next_is_block_white(&mut self) -> bool {
        match self.tok.peek() {
            Some(c) => c.is_whitespace(),
            None => false,
        }
    }

    /// Is the current character is a valid whitespace character, excluding newlines?
    pub fn next_is_line_white(&mut self) -> bool {
        match self.tok.peek() {
            Some(c) => c.is_whitespace() && c != '\n',
            None => false,
        }
    }

    /// Is the current character a valid variable name character?
    pub fn next_is_varname_char(&mut self) -> bool {
        match self.tok.peek() {
            Some(c) => c.is_alphanumeric() || c == '_',
            None => false,
        }
    }

    /// Skips past any whitespace at the current point, *including* newlines.
    /// When this is complete we will be at the end of the script or on a non-white-space
    /// character.
    pub fn skip_block_white(&mut self) {
        while !self.at_end() && self.next_is_block_white() {
            self.tok.next();
        }
    }

    /// Skips past any whitespace on the current line, thus *excluding* newlines.
    /// When this is complete we will be at the end of the script, at the end of the
    /// current command, or on a non-white-space character.
    pub fn skip_line_white(&mut self) {
        while !self.at_end() && self.next_is_line_white() {
            self.tok.next();
        }
    }

    /// Skips past a comment if there is one, including any terminating newline.
    /// Returns true if it skipped a comment, and false otherwise.
    pub fn skip_comment(&mut self) -> bool {
        if self.next_is('#') {
            while !self.at_end() {
                let c = self.tok.next();
                if c == Some('\n') {
                    break;
                } else if c == Some('\\') {
                    // Skip the following character. The intent is to skip
                    // backslashed newlines, but in
                    // this context it doesn't matter.
                    self.tok.next();
                }
            }
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bracket_term() {
        let mut ctx = EvalPtr::new("123");

        assert!(!ctx.is_bracket_term());
        ctx.set_bracket_term(true);
        assert!(ctx.is_bracket_term());
    }

    #[test]
    fn test_next_is() {
        let mut ctx = EvalPtr::new("123");
        assert!(ctx.next_is('1'));
        assert!(!ctx.next_is('2'));

        let mut ctx = EvalPtr::new("");
        assert!(!ctx.next_is('1'));
    }

    #[test]
    fn test_at_end() {
        let mut ctx = EvalPtr::new("123");
        assert!(!ctx.at_end());

        let mut ctx = EvalPtr::new("");
        assert!(ctx.at_end());
    }

    #[test]
    fn test_at_end_of_script() {
        let mut ctx = EvalPtr::new("123");
        assert!(!ctx.at_end_of_script());

        let mut ctx = EvalPtr::new("");
        assert!(ctx.at_end_of_script());

        let mut ctx = EvalPtr::new("]");
        assert!(!ctx.at_end_of_script());
        ctx.set_bracket_term(true);
        assert!(ctx.at_end_of_script());
    }

    #[test]
    fn test_at_end_of_command() {
        let mut ctx = EvalPtr::new("123");
        assert!(!ctx.at_end_of_command());

        let mut ctx = EvalPtr::new(";123");
        assert!(ctx.at_end_of_command());

        let mut ctx = EvalPtr::new("\n123");
        assert!(ctx.at_end_of_command());

        let mut ctx = EvalPtr::new("]123");
        assert!(!ctx.at_end_of_command());

        let mut ctx = EvalPtr::new("]123");
        ctx.set_bracket_term(true);
        assert!(ctx.at_end_of_command());
    }

    #[test]
    fn test_next_is_block_white() {
        let mut ctx = EvalPtr::new("123");
        assert!(!ctx.next_is_block_white());

        let mut ctx = EvalPtr::new(" 123");
        assert!(ctx.next_is_block_white());

        let mut ctx = EvalPtr::new("\n123");
        assert!(ctx.next_is_block_white());
    }

    #[test]
    fn test_skip_block_white() {
        let mut ctx = EvalPtr::new("123");
        ctx.skip_block_white();
        assert!(ctx.next_is('1'));

        let mut ctx = EvalPtr::new("   123");
        ctx.skip_block_white();
        assert!(ctx.next_is('1'));

        let mut ctx = EvalPtr::new(" \n 123");
        ctx.skip_block_white();
        assert!(ctx.next_is('1'));
    }

    #[test]
    fn test_next_is_line_white() {
        let mut ctx = EvalPtr::new("123");
        assert!(!ctx.next_is_line_white());

        let mut ctx = EvalPtr::new(" 123");
        assert!(ctx.next_is_line_white());

        let mut ctx = EvalPtr::new("\n123");
        assert!(!ctx.next_is_line_white());
    }

    #[test]
    fn test_skip_line_white() {
        let mut ctx = EvalPtr::new("123");
        ctx.skip_line_white();
        assert!(ctx.next_is('1'));

        let mut ctx = EvalPtr::new("   123");
        ctx.skip_line_white();
        assert!(ctx.next_is('1'));

        let mut ctx = EvalPtr::new(" \n 123");
        ctx.skip_line_white();
        assert!(ctx.next_is('\n'));
    }

    #[test]
    fn test_skip_comment() {
        let mut ctx = EvalPtr::new("123");
        assert!(!ctx.skip_comment());
        assert!(ctx.next_is('1'));

        let mut ctx = EvalPtr::new(" #123");
        assert!(!ctx.skip_comment());
        assert!(ctx.next_is(' '));

        let mut ctx = EvalPtr::new("#123");
        assert!(ctx.skip_comment());
        assert!(ctx.at_end());

        let mut ctx = EvalPtr::new("#1 2 3 \na");
        assert!(ctx.skip_comment());
        assert!(ctx.next_is('a'));

        let mut ctx = EvalPtr::new("#1 \\na\nb");
        assert!(ctx.skip_comment());
        assert!(ctx.next_is('b'));

        let mut ctx = EvalPtr::new("#1 2] 3 \na");
        ctx.set_bracket_term(true);
        assert!(ctx.skip_comment());
        assert!(ctx.next_is('a'));
    }
}
