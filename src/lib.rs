//! Crate `shellwords` provides utilities for parsing strings as they would be interpreted by the
//! UNIX Bourne shell.

#[deny(missing_debug_implementations, missing_docs, warnings)]
use lazy_static::lazy_static;
use regex::Regex;

/// Escapes a string so it will be interpreted as a single word by the UNIX Bourne shell.
///
/// If the input string is empty, this function returns an empty quoted string.
///
/// # Examples
///
/// ```
/// # extern crate shellwords;
/// # use shellwords::escape;
/// # fn main() {
/// assert_eq!(escape("special's.txt"), "special\\'s.txt".to_string());
/// # }
/// ```
pub fn escape(input: &str) -> String {
    lazy_static! {
        static ref ESCAPE_PATTERN: Regex = Regex::new(r"([^\p{L}0-9_\-.,:/@\n])").unwrap();
        static ref LINE_FEED: Regex = Regex::new(r"\n").unwrap();
    }

    if input.len() == 0 {
        return "''".to_owned();
    }

    let output = &ESCAPE_PATTERN.replace_all(input, "\\$1");

    LINE_FEED.replace_all(output, "'\n'").to_string()
}

/// Builds a command line string from a list of arguments.
///
/// The arguments are combined into a single string with each word separated by a space. Each
/// individual word is escaped as necessary via `escape`.
///
/// # Examples
///
/// ```
/// # extern crate shellwords;
/// # use shellwords::join;
/// # fn main() {
/// let args = ["There's", "a", "time", "and", "place", "for", "everything"];
/// assert_eq!(join(&args), "There\\'s a time and place for everything");
/// # }
/// ```
pub fn join(args: &[&str]) -> String {
    let escaped: Vec<String> = args.iter().map(|arg| escape(arg)).collect();

    escaped.join(" ")
}

/// Splits a string into a vector of words in the same way the UNIX Bourne shell does.
///
/// This function does not behave like a full command line parser. Only single quotes, double
/// quotes, and backslashes are treated as metacharacters. Within double quoted strings,
/// backslashes are only treated as metacharacters when followed by one of the following
/// characters:
///
/// * $
/// * `
/// * "
/// * \
/// * newline
///
/// # Errors
///
/// If the input contains mismatched quotes (a quoted string missing a matching ending quote),
/// a `MismatchedQuotes` error is returned.
///
/// # Examples
///
/// Quoted strings are intepreted as one word:
///
/// ```
/// # extern crate shellwords;
/// # use shellwords::split;
/// # fn main() {
/// assert_eq!(split("here are \"two words\"").unwrap(), ["here", "are", "two words"]);
/// # }
/// ```
///
/// The pipe character has no special meaning:
///
/// ```
/// # extern crate shellwords;
/// # use shellwords::split;
/// # fn main() {
/// assert_eq!(split("cat file.txt | less").unwrap(), ["cat", "file.txt", "|", "less"]);
/// # }
/// ```
///
pub fn split(input: &str) -> Result<Vec<String>, MismatchedQuotes> {
    lazy_static! {
        static ref MAIN_PATTERN: Regex = Regex::new(
            r#"(?m:\s*(?:([^\s\\'"]+)|'([^']*)'|"((?:[^"\\]|\\.)*)"|(\\.?)|(\S))(\s|\z)?)"#
        )
        .unwrap();
        static ref ESCAPE_PATTERN: Regex = Regex::new(r#"\\(.)"#).unwrap();
        static ref METACHAR_PATTERN: Regex = Regex::new(r#"\\([$`"\\\n])"#).unwrap();
    }

    let mut words = Vec::new();
    let mut field = String::new();

    for capture in MAIN_PATTERN.captures_iter(input) {
        if let Some(word) = capture.get(1) {
            field.push_str(word.as_str());
        } else if let Some(single_quoted_word) = capture.get(2) {
            field.push_str(single_quoted_word.as_str());
        } else if let Some(double_quoted_word) = capture.get(3) {
            field.push_str(&METACHAR_PATTERN.replace_all(double_quoted_word.as_str(), "$1"));
        } else if let Some(escape) = capture.get(4) {
            field.push_str(&ESCAPE_PATTERN.replace_all(escape.as_str(), "$1"));
        } else if capture.get(5).is_some() {
            return Err(MismatchedQuotes);
        }

        if capture.get(6).is_some() {
            words.push(field);
            field = String::new();
        }
    }

    Ok(words)
}

/// An error when splitting a string with mismatched quotes.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct MismatchedQuotes;

impl std::fmt::Display for MismatchedQuotes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Mismatched quotes")
    }
}

impl std::error::Error for MismatchedQuotes {}

#[cfg(test)]
mod tests {
    use super::{escape, join, split, MismatchedQuotes};

    #[test]
    fn nothing_special() {
        assert_eq!(split("a b c d").unwrap(), ["a", "b", "c", "d"]);
    }

    #[test]
    fn quoted_strings() {
        assert_eq!(split("a \"b b\" a").unwrap(), ["a", "b b", "a"]);
    }

    #[test]
    fn escaped_double_quotes() {
        assert_eq!(split("a \"\\\"b\\\" c\" d").unwrap(), ["a", "\"b\" c", "d"]);
    }

    #[test]
    fn escaped_single_quotes() {
        assert_eq!(split("a \"'b' c\" d").unwrap(), ["a", "'b' c", "d"]);
    }

    #[test]
    fn escaped_spaces() {
        assert_eq!(split("a b\\ c d").unwrap(), ["a", "b c", "d"]);
    }

    #[test]
    fn bad_double_quotes() {
        assert_eq!(split("a \"b c d e").unwrap_err(), MismatchedQuotes);
    }

    #[test]
    fn bad_single_quotes() {
        assert_eq!(split("a 'b c d e").unwrap_err(), MismatchedQuotes);
    }

    #[test]
    fn bad_quotes() {
        assert_eq!(split("one '\"\"\"").unwrap_err(), MismatchedQuotes);
    }

    #[test]
    fn trailing_whitespace() {
        assert_eq!(split("a b c d ").unwrap(), ["a", "b", "c", "d"]);
    }

    #[test]
    fn empty_escape() {
        assert_eq!(escape(""), "''");
    }

    #[test]
    fn full_escape() {
        assert_eq!(escape("foo '\"' bar"), "foo\\ \\'\\\"\\'\\ bar");
    }

    #[test]
    fn escape_and_join_whitespace() {
        let empty = "".to_owned();
        let space = " ".to_owned();
        let newline = "\n".to_owned();
        let tab = "\t".to_owned();

        let tokens = vec![
            empty.clone(),
            space.clone(),
            space.clone() + &space,
            newline.clone(),
            newline.clone() + &newline,
            tab.clone(),
            tab.clone() + &tab,
            empty.clone(),
            space + &newline + &tab,
            empty,
        ];

        for token in tokens.iter() {
            assert_eq!(
                vec![token.as_str()],
                split(escape(token.as_str()).as_str()).unwrap()
            );
        }

        let borrowed_tokens: Vec<&str> = tokens.iter().map(|token| &token[..]).collect();
        assert_eq!(
            tokens,
            split(join(borrowed_tokens.as_slice()).as_str()).unwrap()
        );
    }

    #[test]
    fn escape_multibyte() {
        assert_eq!(escape("あい"), "あい");
    }

    #[test]
    fn percent_signs() {
        assert_eq!(split("abc '%foo bar%'").unwrap(), ["abc", "%foo bar%"]);
    }
}
