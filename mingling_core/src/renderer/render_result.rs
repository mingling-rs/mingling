use std::{
    fmt::{Display, Formatter},
    io::Write,
    process::{ExitCode, exit},
};

use crate::RenderResultMode::{Stderr, Stdout};

/// Render result, containing the rendered text content.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct RenderResult {
    /// Whether the output should be written immediately.
    ///
    /// When set to `true`, rendered content will be flushed to stdout/stderr
    /// in real time while also being collected in the render buffer.
    immediate_output: bool,

    /// The buffered render output, stored as a list of (text, mode) pairs.
    ///
    /// Each entry contains a rendered string together with a `RenderResultMode`
    /// indicating whether it should be output to stdout or stderr.
    render_buffer: Vec<(String, RenderResultMode)>,

    /// The exit code to return from the rendering process.
    ///
    /// A value of `0` indicates success, while non-zero values indicate
    /// various error conditions.
    pub exit_code: i32,
}

/// Enum representing the output mode for render results.
///
/// This determines whether the rendered content should be directed to standard
/// output or standard error.
///
/// # Variants
///
/// * `Stdout` - Output will be written to standard output (stdout).
/// * `Stderr` - Output will be written to standard error (stderr).
#[repr(u8)]
#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum RenderResultMode {
    /// Standard output (stdout).
    #[default]
    Stdout = 0,

    /// Standard error (stderr).
    Stderr = 1,
}

impl<F> From<F> for RenderResult
where
    F: FnOnce() -> Self,
{
    fn from(value: F) -> Self {
        value()
    }
}

impl Write for RenderResult {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let s = std::str::from_utf8(buf).map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "not valid UTF-8")
        })?;
        self.append_to_buffer(s, Stdout);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Display for RenderResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", render_result_to_string(self).trim())
    }
}

impl From<()> for RenderResult {
    fn from(_value: ()) -> Self {
        Self::new()
    }
}

macro_rules! impl_from_int {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl From<$ty> for RenderResult {
                fn from(exit_code: $ty) -> Self {
                    Self {
                        exit_code: <i32>::try_from(exit_code).unwrap_or_default(),
                        ..Default::default()
                    }
                }
            }
        )+
    };
}

impl_from_int!(i32, i16, i8, u32, u16, u8, usize);

impl From<RenderResult> for ExitCode {
    fn from(value: RenderResult) -> Self {
        Self::from(u8::try_from(value.exit_code).unwrap_or_default())
    }
}

impl From<&RenderResult> for ExitCode {
    fn from(value: &RenderResult) -> Self {
        Self::from(u8::try_from(value.exit_code).unwrap_or_default())
    }
}

impl From<&String> for RenderResult {
    fn from(value: &String) -> Self {
        string_to_render_result(value, Stdout)
    }
}

impl From<String> for RenderResult {
    fn from(value: String) -> Self {
        string_to_render_result(value, Stdout)
    }
}

impl From<&str> for RenderResult {
    fn from(value: &str) -> Self {
        string_to_render_result(value, Stdout)
    }
}

impl From<RenderResult> for String {
    fn from(result: RenderResult) -> Self {
        render_result_to_string(&result)
    }
}

impl From<&RenderResult> for String {
    fn from(result: &RenderResult) -> Self {
        render_result_to_string(result)
    }
}

impl RenderResult {
    /// Creates a new `RenderResult` with default values (empty text and exit code 0).
    ///
    /// Equivalent to `RenderResult::default()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_core::RenderResult;
    ///
    /// let result = RenderResult::new();
    /// assert_eq!(result.exit_code, 0);
    /// assert!(result.is_empty());
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Marks the render result for immediate output, bypassing any buffering or
    /// deferred rendering.
    ///
    /// When set, the rendered content will be both collected in the result and
    /// immediately flushed to stdout/stderr in real time, rather than being
    /// deferred for later display.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_core::RenderResult;
    ///
    /// let mut result = RenderResult::default();
    /// result.immediate_output();
    /// ```
    pub const fn immediate_output(&mut self) -> &mut Self {
        self.immediate_output = true;
        self
    }

    /// Appends the given text and mode to the render buffer.
    ///
    /// Unlike `print` and `println` which only store plain text in a single string,
    /// this method stores the text along with a `RenderResultMode` that indicates
    /// whether the output should be directed to stdout or stderr. This allows for
    /// more fine-grained control over output routing when the buffer is later flushed.
    ///
    /// # Arguments
    ///
    /// * `text` - The text content to append to the buffer.
    /// * `mode` - The output mode (`Stdout` or `Stderr`) indicating where the text
    ///   should be directed.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_core::{RenderResult, RenderResultMode};
    ///
    /// let mut result = RenderResult::default();
    /// result.append_to_buffer("Hello", RenderResultMode::Stdout);
    /// result.append_to_buffer("Error message", RenderResultMode::Stderr);
    /// ```
    pub fn append_to_buffer(&mut self, text: impl Into<String>, mode: RenderResultMode) {
        self.render_buffer.push((text.into(), mode));
    }

    /// Appends the given text followed by a newline, along with the mode, to the render buffer.
    ///
    /// This is a convenience method that calls `append_to_buffer` for the text and then
    /// appends a newline with the same mode.
    ///
    /// # Arguments
    ///
    /// * `text` - The text content to append to the buffer.
    /// * `mode` - The output mode (`Stdout` or `Stderr`) indicating where the text
    ///   should be directed.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_core::{RenderResult, RenderResultMode};
    ///
    /// let mut result = RenderResult::default();
    /// result.append_line_to_buffer("Hello", RenderResultMode::Stdout);
    /// result.append_line_to_buffer("Warning", RenderResultMode::Stderr);
    /// ```
    pub fn append_line_to_buffer(&mut self, text: impl Into<String>, mode: RenderResultMode) {
        self.append_to_buffer(text, mode);
        self.append_to_buffer("\n", mode);
    }

    /// Appends the contents of another `RenderResult` to this one.
    ///
    /// If this `RenderResult` has `immediate_output` enabled but the other does not,
    /// the other's content will be immediately flushed to the appropriate output stream
    /// (stdout/stderr) while also being appended to the render buffer.
    ///
    /// The `exit_code` of the other result is **not** transferred — only the buffered
    /// content and the `immediate_output` flag of the other result are merged.
    ///
    /// # Arguments
    ///
    /// * `other` - The `RenderResult` whose contents should be appended.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_core::{RenderResult, RenderResultMode};
    ///
    /// let mut dest = RenderResult::default();
    /// let mut src = RenderResult::default();
    ///
    /// src.append_to_buffer("Hello", RenderResultMode::Stdout);
    /// src.append_to_buffer(" Error", RenderResultMode::Stderr);
    ///
    /// dest.append_other(src);
    /// assert_eq!(dest.to_string(), "Hello Error");
    /// ```
    pub fn append_other(&mut self, other: impl Into<Self>) {
        let other = other.into();

        // If self has immediate output enabled, but the input does not, the input needs immediate output.
        let immediate_output = !other.immediate_output && self.immediate_output;

        for i in other.render_buffer {
            if immediate_output {
                match &i.1 {
                    Stdout => print!("{}", i.0),
                    Stderr => eprint!("{}", i.0),
                }
            }
            self.render_buffer.push(i);
        }
    }

    /// Appends the given text to the rendered content.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_core::RenderResult;
    ///
    /// let mut result = RenderResult::default();
    /// result.print("Hello");
    /// result.print(", world!");
    /// assert_eq!(result.to_string(), "Hello, world!");
    /// ```
    pub fn print(&mut self, text: impl Into<String>) {
        let text = text.into();
        if self.immediate_output {
            print!("{text}");
        }
        self.append_to_buffer(text, Stdout);
    }

    /// Appends the given text followed by a newline to the rendered content.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_core::RenderResult;
    ///
    /// let mut result = RenderResult::default();
    /// result.println("First line");
    /// result.println("Second line");
    /// assert_eq!(result.to_string(), "First line\nSecond line");
    /// ```
    pub fn println(&mut self, text: impl Into<String>) {
        let text = text.into();
        if self.immediate_output {
            println!("{text}");
        }
        self.append_line_to_buffer(text, Stdout);
    }

    /// Appends the given text to the rendered content, marking it for stderr output.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_core::RenderResult;
    ///
    /// let mut result = RenderResult::default();
    /// result.eprint("Hello");
    /// result.eprint(", world!");
    /// assert_eq!(result.to_string(), "Hello, world!");
    /// ```
    pub fn eprint(&mut self, text: impl Into<String>) {
        let text = text.into();
        if self.immediate_output {
            eprint!("{text}");
        }
        self.append_to_buffer(text, Stderr);
    }

    /// Appends the given text followed by a newline to the rendered content, marking it for stderr output.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_core::RenderResult;
    ///
    /// let mut result = RenderResult::default();
    /// result.eprintln("First line");
    /// result.eprintln("Second line");
    /// assert_eq!(result.to_string(), "First line\nSecond line");
    /// ```
    pub fn eprintln(&mut self, text: impl Into<String>) {
        let text = text.into();
        if self.immediate_output {
            eprintln!("{text}");
        }
        self.append_line_to_buffer(text, Stderr);
    }

    /// Clears all rendered content.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_core::RenderResult;
    /// use std::ops::Deref;
    ///
    /// let mut result = RenderResult::default();
    /// result.print("Some content");
    /// assert!(!result.is_empty());
    /// result.clear();
    /// assert!(result.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.render_buffer.clear();
    }

    /// Outputs all buffered content to stdout and stderr according to their respective modes.
    ///
    /// Iterates through the render buffer and prints each buffered string to the appropriate
    /// output stream — stdout for `Stdout` entries and stderr for `Stderr` entries.
    ///
    /// This method is typically used to flush the buffered output at the end of rendering,
    /// ensuring that all output is displayed in the correct order and to the correct stream.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_core::{RenderResult, RenderResultMode};
    ///
    /// let mut result = RenderResult::default();
    /// result.append_to_buffer("Hello", RenderResultMode::Stdout);
    /// result.append_to_buffer("Error", RenderResultMode::Stderr);
    /// result.std_print(); // prints "Hello" to stdout and "Error" to stderr
    /// ```
    pub fn std_print(&self) {
        for (content, mode) in &self.render_buffer {
            match mode {
                Stdout => print!("{content}"),
                Stderr => eprint!("{content}"),
            }
        }
    }

    /// Returns the total number of characters (in terms of `char` count) in the buffered render output.
    ///
    /// This counts the length across all buffered entries, regardless of whether they are
    /// destined for stdout or stderr.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_core::RenderResult;
    ///
    /// let mut result = RenderResult::default();
    /// result.print("Hello");
    /// result.print(", 世界");
    /// assert_eq!(result.len(), 9); // "Hello, 世界" has 9 chars
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.render_buffer
            .iter()
            .map(|(s, _)| s.chars().count())
            .sum()
    }

    /// Returns `true` if the buffered render output contains no characters.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_core::RenderResult;
    ///
    /// let mut result = RenderResult::default();
    /// assert!(result.is_empty());
    /// result.print("Hello");
    /// assert!(!result.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Trims leading and trailing whitespace from the buffered render output.
    ///
    /// This method processes the render buffer as follows:
    /// - If the buffer is empty, it returns `self` unchanged.
    /// - If there is only one entry, whitespace is trimmed from both the start and end of that
    ///   single entry.
    /// - If there are multiple entries, whitespace is trimmed from the start of the first entry
    ///   and the end of the last entry.
    ///
    /// Whitespace in the middle entries is preserved. This is useful for cleaning up output
    /// without removing intentional spacing between separately buffered segments.
    ///
    /// # Returns
    ///
    /// A new `RenderResult` with the same `immediate_output` flag and `exit_code`, but with
    /// trimmed text content.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_core::RenderResult;
    ///
    /// let mut result = RenderResult::default();
    /// result.print("  Hello, world!  ");
    /// let trimmed = result.trim_buffer();
    /// assert_eq!(trimmed.to_string().trim(), "Hello, world!");
    /// ```
    #[must_use]
    pub fn trim_buffer(self) -> Self {
        if self.render_buffer.is_empty() {
            return self;
        }

        let mut buffer = self.render_buffer;
        if buffer.len() == 1 {
            // Only one entry: trim both start and end of this single entry
            let (text, mode) = buffer.remove(0);
            buffer.push((text.trim().to_string(), mode));
        } else {
            // Multiple entries: trim start of first, trim end of last
            let first_len = buffer.len();

            // Trim start of first entry
            let (first_text, first_mode) = buffer.remove(0);
            let trimmed_first = first_text.trim_start().to_string();
            buffer.insert(0, (trimmed_first, first_mode));

            // Trim end of last entry
            let (last_text, last_mode) = buffer.remove(first_len - 1);
            let trimmed_last = last_text.trim_end().to_string();
            buffer.push((trimmed_last, last_mode));
        }

        Self {
            render_buffer: buffer,
            immediate_output: self.immediate_output,
            exit_code: self.exit_code,
        }
    }

    /// Exits the process with the exit code stored in this `RenderResult`.
    ///
    /// This method calls `std::process::exit()` with the `exit_code` value,
    /// terminating the current process immediately.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_core::RenderResult;
    ///
    /// let mut result = RenderResult::new();
    /// result.exit_code = 42;
    /// // result.exit_process(); // would exit with code 42
    /// ```
    pub fn exit_process(&self) {
        exit(self.exit_code)
    }
}

#[inline]
fn render_result_to_string(result: &RenderResult) -> String {
    let mut buffer = String::new();
    for item in &result.render_buffer {
        buffer += &item.0;
    }
    buffer
}

#[inline]
fn string_to_render_result(string: impl Into<String>, mode: RenderResultMode) -> RenderResult {
    RenderResult {
        render_buffer: vec![(string.into(), mode)],
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write as IoWrite;

    #[test]
    fn default_creates_empty_text_with_exit_code_zero() {
        let result = RenderResult::default();
        assert!(result.is_empty());
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn clear_empties_content() {
        let mut result = RenderResult::default();
        result.print("something");
        assert!(!result.is_empty());
        result.clear();
        assert!(result.is_empty());
    }

    #[test]
    fn is_empty_returns_true_for_new_false_after_print() {
        let mut result = RenderResult::default();
        assert!(result.is_empty());
        result.print("x");
        assert!(!result.is_empty());
    }

    #[test]
    fn write_with_invalid_utf8_returns_error() {
        let mut result = RenderResult::default();
        let err = IoWrite::write(&mut result, &[0xff, 0xfe]).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    }

    #[test]
    fn display_trims_trailing_whitespace() {
        let mut result = RenderResult::default();
        result.print("  hello world  \n");
        let formatted = format!("{result}");
        assert_eq!(formatted, "hello world");
    }

    #[test]
    fn from_render_result_into_string_consumes() {
        let mut result = RenderResult::default();
        result.print("content");
        let s: String = result.into();
        assert_eq!(s, "content");
    }

    #[test]
    fn from_ref_render_result_into_string_clones() {
        let mut result = RenderResult::default();
        result.print("content");
        let s: String = String::from(&result);
        assert_eq!(s, "content");
        // original is still usable
        assert!(!result.is_empty());
    }

    #[test]
    fn trim_empty_buffer_returns_self() {
        let result = RenderResult::default();
        let trimmed = result.trim_buffer();
        assert!(trimmed.is_empty());
        assert_eq!(trimmed.exit_code, 0);
    }

    #[test]
    fn trim_single_entry_trims_both_ends() {
        let mut result = RenderResult::default();
        result.print("  Hello, world!  ");
        let trimmed = result.trim_buffer();
        assert_eq!(trimmed.to_string(), "Hello, world!");
    }

    #[test]
    fn trim_single_entry_nothing_to_trim() {
        let mut result = RenderResult::default();
        result.print("Hello");
        let trimmed = result.trim_buffer();
        assert_eq!(trimmed.to_string(), "Hello");
    }

    #[test]
    fn trim_multiple_entries_trims_first_start_and_last_end() {
        let mut result = RenderResult::default();
        result.print("  Hello");
        result.print(" World ");
        result.print("!  ");
        let trimmed = result.trim_buffer();
        // first entry trim_start: "Hello"
        // middle entry unchanged: " World "
        // last entry trim_end: "!"
        assert_eq!(trimmed.to_string(), "Hello World !");
    }

    #[test]
    fn trim_multiple_entries_only_whitespace_first_entry() {
        let mut result = RenderResult::default();
        result.print("   ");
        result.print("Hello");
        result.print("   ");
        let trimmed = result.trim_buffer();
        // first entry trim_start: ""
        // middle entry unchanged: "Hello"
        // last entry trim_end: ""
        assert_eq!(trimmed.to_string(), "Hello");
    }

    #[test]
    fn trim_preserves_exit_code() {
        let mut result = RenderResult::new();
        result.exit_code = 42;
        result.print("  test  ");
        let trimmed = result.trim_buffer();
        assert_eq!(trimmed.exit_code, 42);
    }

    #[test]
    fn trim_preserves_stderr_mode() {
        let mut result = RenderResult::default();
        result.eprint("  error  ");
        let trimmed = result.trim_buffer();
        assert_eq!(trimmed.render_buffer[0].1, RenderResultMode::Stderr);
        assert_eq!(trimmed.to_string(), "error");
    }
}
