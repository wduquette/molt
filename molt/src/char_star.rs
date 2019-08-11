use std::str::Chars;

#[derive(Clone,Debug)]
pub struct CharStar<'a> {
    input: &'a str,
    chars: Chars<'a>,
    head_index: usize,
    mark_index: usize,
}

impl<'a> CharStar<'a> {
    // Create a new one for the given input.
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.chars(),
            head_index: 0,
            mark_index: 0,
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

    // Start parsing a new token at the current head
    pub fn mark_head(&mut self) {
        self.mark_index = self.head_index;
    }

    // Get the token between the mark and the head.
    pub fn token(&self) -> &str {
        &self.input[self.mark_index..self.head_index]
    }

    // Get the token between the mark and the head, and update the mark.
    pub fn next_token(&mut self) -> &str {
        let token = &self.input[self.mark_index..self.head_index];
        self.mark_index = self.head_index;
        token
    }

    // Resets the head to the mark.
    pub fn backup(&mut self) {
        self.head_index = self.mark_index;
        self.chars = self.input[self.head_index..].chars();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basics() {
        // Create the iterator
        let mut ptr = CharStar::new("abcdefghijklmnopqrstuvwxyz");
        assert_eq!(ptr.input(), "abcdefghijklmnopqrstuvwxyz");
        assert_eq!(ptr.mark(), "abcdefghijklmnopqrstuvwxyz");
        assert_eq!(ptr.head(), "abcdefghijklmnopqrstuvwxyz");

        // Skip three characters
        ptr.next();
        ptr.next();
        ptr.next();
        assert_eq!(ptr.input(), "abcdefghijklmnopqrstuvwxyz");
        assert_eq!(ptr.mark(), "abcdefghijklmnopqrstuvwxyz");
        assert_eq!(ptr.head(), "defghijklmnopqrstuvwxyz");

        // Mark the current spot
        ptr.mark_head();
        assert_eq!(ptr.input(), "abcdefghijklmnopqrstuvwxyz");
        assert_eq!(ptr.mark(), "defghijklmnopqrstuvwxyz");
        assert_eq!(ptr.head(), "defghijklmnopqrstuvwxyz");

        // Skip three more characters
        ptr.next();
        ptr.next();
        ptr.next();
        assert_eq!(ptr.input(), "abcdefghijklmnopqrstuvwxyz");
        assert_eq!(ptr.mark(), "defghijklmnopqrstuvwxyz");
        assert_eq!(ptr.head(), "ghijklmnopqrstuvwxyz");
        assert_eq!(ptr.token(), "def");

        // next_token
        assert_eq!(ptr.next_token(), "def");
        assert_eq!(ptr.input(), "abcdefghijklmnopqrstuvwxyz");
        assert_eq!(ptr.mark(), "ghijklmnopqrstuvwxyz");
        assert_eq!(ptr.head(), "ghijklmnopqrstuvwxyz");
        assert_eq!(ptr.token(), "");

        // backup
        ptr.next();
        ptr.next();
        ptr.next();
        assert_eq!(ptr.mark(), "ghijklmnopqrstuvwxyz");
        assert_eq!(ptr.head(), "jklmnopqrstuvwxyz");
        assert_eq!(ptr.token(), "ghi");

        ptr.backup();
        assert_eq!(ptr.token(), "");
        assert_eq!(ptr.head(), "ghijklmnopqrstuvwxyz");
    }
}
