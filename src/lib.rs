//! Main GCL Library File

use std::str::Chars;

#[allow(clippy::new_without_default_derive)]
pub mod interp;

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

pub fn parse_word(input: &mut Chars) -> Option<String> {
    let mut word = String::new();
    let mut in_word = false;

    while let Some(c) = input.next() {
        if in_word {
            if c.is_whitespace() {
                break;  // Word is complete
            } else {
                word.push(c);
            }
        } else { // LookingForWord
            if c.is_whitespace() {
                continue;
            } else {
                in_word = true;
                word.push(c);
            }
        }
    }

    if in_word {
        Some(word)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_word() {
        let input = String::from("some words");
        let chars = &mut input.chars();

        assert_eq!(parse_word(chars), Some("some".into()));
        assert_eq!(parse_word(chars), Some("words".into()));
        assert_eq!(parse_word(chars), None);
    }

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

    #[test]
    fn test_parse_command2() {
        let input = String::from("  some words  \nanother");
        let chars = &mut input.chars();

        while let Some(cmd) = parse_command(chars) {
            println!("Got: {:?}", cmd);
        }
        assert!(true);
    }

}
