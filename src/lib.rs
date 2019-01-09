//! Main GCL Library File

use std::str::Chars;

pub mod interp;
pub mod types;

pub fn parse_command(input: &mut Chars) -> Option<Vec<String>> {
    let mut cmd = Vec::new();
    let mut word = String::new();
    let mut in_word = false;

    while let Some(c) = input.next() {
        if c == '\n' {
            break;  // Found newline; TODO: Handle escaped newline
        }

        if in_word {
            if c.is_whitespace() {
                // Completed a word
                cmd.push(word.clone());
                word.clear();
                in_word = false;
            } else {
                // Add the character to the current word.
                word.push(c);
            }
        } else {
            // Looking for the next word
            if c.is_whitespace() {
                continue;
            } else {
                in_word = true;
                word.push(c);
            }
        }
    }

    if !word.is_empty() {
        cmd.push(word);
    }

    if cmd.is_empty() {
        None
    } else {
        Some(cmd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_command() {
        let input = String::from("  some words  \nanother");
        let chars = &mut input.chars();

        let cmd = parse_command(chars);
        assert!(cmd.is_some());

        let cmd = cmd.unwrap();
        assert!(cmd.len() == 2);
        assert_eq!(&cmd[0], "some");
        assert_eq!(&cmd[1], "words");

        let remainder: String = chars.collect();
        assert_eq!(&remainder, "another");
    }
}
