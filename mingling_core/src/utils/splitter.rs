/// A trait for splitting strings into arguments, respecting quotes and escapes.
///
/// # Examples
///
/// ```
/// use mingling_core::utils::ArgumentSplitter;
///
/// let args = "echo \"hello world\"".split_args();
/// assert_eq!(args, vec!["echo", "hello world"]);
/// ```
pub trait ArgumentSplitter {
    /// Splits the input string into a vector of argument strings.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_core::utils::ArgumentSplitter;
    ///
    /// let args = "a 'b c' d".split_args();
    /// assert_eq!(args, vec!["a", "b c", "d"]);
    /// ```
    fn split_args(self) -> Vec<String>;
}

impl<S: AsRef<str>> ArgumentSplitter for S {
    /// Splits the string into arguments, respecting single quotes, double
    /// quotes, and backslash escaping.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_core::utils::ArgumentSplitter;
    ///
    /// let args = r#"cmd --flag "value with spaces""#.split_args();
    /// assert_eq!(args, vec!["cmd", "--flag", "value with spaces"]);
    /// ```
    ///
    /// Escaped characters are unescaped:
    ///
    /// ```
    /// use mingling_core::utils::ArgumentSplitter;
    ///
    /// let args = r#"echo a\ b"#.split_args();
    /// assert_eq!(args, vec!["echo", "a b"]);
    /// ```
    fn split_args(self) -> Vec<String> {
        split_args(self.as_ref())
    }
}

/// Splits a string input into arguments, respecting single quotes, double quotes,
/// and backslash escaping.
fn split_args(input: &str) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars();

    while let Some(ch) = chars.next() {
        match ch {
            '\\' => {
                // Take the next character literally (if any) and add it to current.
                if let Some(next) = chars.next() {
                    current.push(next);
                }
                // If there's no next character, the backslash is just ignored/lost.
            }
            '"' | '\'' => {
                // Start of a quoted segment.
                let quote_char = ch;
                let mut escaped = false;
                loop {
                    match chars.next() {
                        None => break,
                        Some(c) => {
                            if escaped {
                                current.push(c);
                                escaped = false;
                            } else if c == '\\' {
                                escaped = true;
                            } else if c == quote_char {
                                break;
                            } else {
                                current.push(c);
                            }
                        }
                    }
                }
            }
            ' ' => {
                if !current.is_empty() {
                    result.push(current.clone());
                    current.clear();
                }
            }
            _ => {
                current.push(ch);
            }
        }
    }

    if !current.is_empty() {
        result.push(current);
    }

    result
}

#[cfg(test)]
mod splitter_tests {
    use crate::utils::splitter::split_args;

    #[test]
    fn test_split_with_double_quotes() {
        let input = r#"a "b c" d"#;
        let result = split_args(input);
        assert_eq!(result, vec!["a", "b c", "d"]);
    }

    #[test]
    fn test_split_with_single_quotes() {
        let input = "a 'b c' d";
        let result = split_args(input);
        assert_eq!(result, vec!["a", "b c", "d"]);
    }

    #[test]
    fn test_empty_input() {
        assert!(split_args("").is_empty());
    }

    #[test]
    fn test_no_quotes() {
        let result = split_args("hello world");
        assert_eq!(result, vec!["hello", "world"]);
    }

    #[test]
    fn test_double_quotes_at_edges() {
        let result = split_args(r#""hello world" foo"#);
        assert_eq!(result, vec!["hello world", "foo"]);
    }

    #[test]
    fn test_single_quotes_at_edges() {
        let result = split_args("'hello world' foo");
        assert_eq!(result, vec!["hello world", "foo"]);
    }

    #[test]
    fn test_multiple_double_quoted_parts() {
        let result = split_args(r#"a "b c" d "e f g""#);
        assert_eq!(result, vec!["a", "b c", "d", "e f g"]);
    }

    #[test]
    fn test_multiple_single_quoted_parts() {
        let result = split_args("a 'b c' d 'e f g'");
        assert_eq!(result, vec!["a", "b c", "d", "e f g"]);
    }

    #[test]
    fn test_backslash_escaped_space() {
        let result = split_args("a b\\ c d");
        assert_eq!(result, vec!["a", "b c", "d"]);
    }

    #[test]
    fn test_backslash_escaped_double_quote() {
        let result = split_args(r#"a b\"c d"#);
        assert_eq!(result, vec!["a", r#"b"c"#, "d"]);
    }

    #[test]
    fn test_backslash_escaped_single_quote() {
        let result = split_args("a b\\'c d");
        assert_eq!(result, vec!["a", "b'c", "d"]);
    }
}
