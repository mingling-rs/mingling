use std::{
    fmt::{self, Display, Formatter},
    io::Write,
    process::{ExitCode, exit},
};

use crate::RenderResultMode::{Stderr, Stdout};

/// A single emitted output item handed to a print hook.
///
/// `RenderResultPrint` bundles the text content and the output mode together
/// into one value, so a print hook can route the content to stdout/stderr — or
/// any custom sink — as a unit instead of juggling two separate arguments.
///
/// Values of this type are produced whenever a [`RenderResult`] with bound
/// print hooks (see [`RenderResult::bind_print_hook`] and
/// [`RenderResult::immediate_output`]) writes content through
/// `print`/`println`/`eprint`/`eprintln`, and are handed to every hook in
/// binding order. They are also used to flush another result's buffered content
/// via [`RenderResult::append_other`] when the destination has hooks bound.
///
/// # Fields
///
/// * `content` — The raw text that was written, including any trailing newline
///   added by `println`/`eprintln`.
/// * `mode` — The output mode (`Stdout` or `Stderr`) the content was written
///   with, which tells the hook where the content belongs.
///
/// # Examples
///
/// ```
/// use mingling_core::{RenderResult, RenderResultMode, RenderResultPrint};
///
/// // Build an output item manually
/// let print = RenderResultPrint {
///     content: "Hello, world!".to_string(),
///     mode: RenderResultMode::Stdout,
/// };
/// assert_eq!(print.content, "Hello, world!");
/// assert_eq!(print.mode, RenderResultMode::Stdout);
///
/// // Use it inside a print hook
/// let mut result = RenderResult::default();
/// result.bind_print_hook(|print| match print.mode {
///     RenderResultMode::Stdout => print!("{}", print.content),
///     RenderResultMode::Stderr => eprint!("{}", print.content),
/// });
/// result.eprintln("something went wrong"); // goes to stderr via the hook
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderResultPrint {
    /// The emitted text content.
    ///
    /// This is the raw text that was written to the render buffer when the
    /// hook fired. For `println`/`eprintln` it includes the trailing newline;
    /// for `print`/`eprint` it is exactly the given text.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_core::{RenderResultMode, RenderResultPrint};
    ///
    /// let print = RenderResultPrint {
    ///     content: "Hello".to_string(),
    ///     mode: RenderResultMode::Stdout,
    /// };
    /// assert_eq!(print.content, "Hello");
    /// ```
    pub content: String,

    /// The output mode the content was written with.
    ///
    /// Indicates whether the content was originally directed to stdout
    /// (`Stdout`) or stderr (`Stderr`), allowing a hook to route the content
    /// to the matching stream.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_core::{RenderResultMode, RenderResultPrint};
    ///
    /// let print = RenderResultPrint {
    ///     content: "error".to_string(),
    ///     mode: RenderResultMode::Stderr,
    /// };
    /// assert_eq!(print.mode, RenderResultMode::Stderr);
    /// ```
    pub mode: RenderResultMode,
}

/// Optional list of print hooks bound to a `RenderResult`.
///
/// Each hook receives the emitted [`RenderResultPrint`]. See
/// [`RenderResult::bind_print_hook`] and [`RenderResult::immediate_output`].
type PrintHook = Option<Vec<Box<dyn FnMut(RenderResultPrint)>>>;

/// Render result, containing the rendered text content.
///
/// `RenderResult` is the core data structure used throughout the rendering pipeline
/// to collect and output text content. It maintains a render buffer (`render_buffer`),
/// where each entry carries an output mode [`RenderResultMode`] that determines
/// whether the content is ultimately written to stdout or stderr. It also records
/// the process exit code (`exit_code`), which can be used to terminate the process
/// with the appropriate status after rendering completes.
///
/// # Features
///
/// - **Buffered output**: All rendered content is first collected into the buffer
///   and can be output uniformly at a convenient time.
/// - **Immediate output**: Can be enabled via [`immediate_output`](RenderResult::immediate_output),
///   which binds a print hook that flushes content to stdout/stderr in real time
///   while also being added to the buffer. Custom hooks can be bound with
///   [`bind_print_hook`](RenderResult::bind_print_hook).
/// - **Dual-channel output**: The `Stdout` and `Stderr` modes distinguish between
///   normal output and error output.
/// - **Exit code management**: Supports carrying an exit code to exit the process
///   with a specific status code after rendering.
///
/// # Conversions to and from Other Types
///
/// `RenderResult` implements conversions for various sources and targets:
///
/// - From types implementing `impl Into<String>` (such as `&str`, `String`, `&String`),
///   producing a result containing only that text directed to stdout.
/// - From integer types (`i32`, `u8`, etc.), producing an empty result with the
///   specified exit code.
/// - Into `String`, extracting the concatenated text of all buffered content.
/// - Into `ExitCode`, for use when the process exits.
///
/// # Examples
///
/// ```
/// use mingling_core::{RenderResult, RenderResultMode};
///
/// // Create an empty render result
/// let result = RenderResult::new();
/// assert!(result.is_empty());
///
/// // Create from a string
/// let result: RenderResult = "Hello, world!".into();
/// assert_eq!(result.to_string(), "Hello, world!");
///
/// // Specify an exit code via an integer
/// let result: RenderResult = 42.into();
/// assert_eq!(result.exit_code, 42);
///
/// // Create via a closure (implemented by `From<F>`)
/// let result: RenderResult = (|| RenderResult::from("closure result")).into();
/// assert_eq!(result.to_string(), "closure result");
/// ```
#[derive(Default)]
pub struct RenderResult {
    /// Print hooks invoked with the buffered content and its output mode.
    ///
    /// When hooks are bound (via [`immediate_output`](RenderResult::immediate_output)
    /// or [`bind_print_hook`](RenderResult::bind_print_hook)), every
    /// `print`/`println`/`eprint`/`eprintln` call additionally emits the content
    /// through each hook in binding order — typically flushing it to stdout/stderr
    /// in real time — while the content is still appended to the buffer.
    ///
    /// The default value is `None`, meaning content is only buffered and output
    /// uniformly at the end (e.g. via [`std_print`](RenderResult::std_print)).
    print_hook: PrintHook,

    /// Render buffer, stored as a list of (text, output mode) pairs.
    ///
    /// Each entry contains:
    /// * The rendered string content;
    /// * A [`RenderResultMode`] enum value indicating whether the content should
    ///   be output to standard output (`Stdout`) or standard error (`Stderr`).
    ///
    /// The buffer preserves insertion order, guaranteeing the correct ordering of
    /// rendered content. The [`std_print`](RenderResult::std_print) method outputs
    /// the content to the corresponding streams according to their modes.
    ///
    /// This design allows mixing stdout and stderr content within a single result
    /// while precisely controlling the output direction of each, which is especially
    /// important in scenarios such as command-line tools and logging systems.
    render_buffer: Vec<(String, RenderResultMode)>,

    /// The exit code for the rendering process.
    ///
    /// The default value is `0` (indicating success); non-zero values indicate
    /// various error conditions. After rendering completes, the
    /// [`exit_process`](RenderResult::exit_process) method can use this exit code
    /// to terminate the current process, or the `From<RenderResult> for ExitCode`
    /// conversion can be used with `std::process::ExitCode`.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_core::RenderResult;
    ///
    /// let mut result = RenderResult::new();
    /// assert_eq!(result.exit_code, 0); // default success
    ///
    /// result.exit_code = 1; // mark an error
    /// assert_eq!(result.exit_code, 1);
    /// ```
    pub exit_code: i32,
}

impl fmt::Debug for RenderResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("RenderResult")
            .field("render_buffer", &self.render_buffer)
            .field("exit_code", &self.exit_code)
            .field("print_hooks", &self.print_hook.as_ref().map(Vec::len))
            .finish()
    }
}

impl Clone for RenderResult {
    /// The bound print hooks are opaque closures and cannot be cloned, so the
    /// cloned result is created without any hooks.
    fn clone(&self) -> Self {
        Self {
            print_hook: None,
            render_buffer: self.render_buffer.clone(),
            exit_code: self.exit_code,
        }
    }
}

impl PartialEq for RenderResult {
    fn eq(&self, other: &Self) -> bool {
        self.render_buffer == other.render_buffer && self.exit_code == other.exit_code
    }
}

impl Eq for RenderResult {}

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

    /// Enables immediate output by binding a print hook that flushes content to
    /// stdout/stderr in real time.
    ///
    /// After this is called, every `print`/`println`/`eprint`/`eprintln` call
    /// writes its content to the corresponding output stream immediately, while
    /// also keeping it in the buffer for later use (e.g. [`std_print`](RenderResult::std_print)
    /// or `to_string()`).
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_core::RenderResult;
    ///
    /// let mut result = RenderResult::default();
    /// result.immediate_output();
    /// result.print("Hello, ");
    /// result.print("world!"); // flushed to stdout right away
    /// assert_eq!(result.to_string(), "Hello, world!");
    /// ```
    pub fn immediate_output(&mut self) -> &mut Self {
        self.bind_print_hook(|RenderResultPrint { content, mode }| match mode {
            Stdout => print!("{content}"),
            Stderr => eprint!("{content}"),
        })
    }

    /// Binds a custom print hook invoked with the content and output mode of
    /// every `print`/`println`/`eprint`/`eprintln` call.
    ///
    /// Multiple hooks can be bound; they are invoked in binding order. This is
    /// the building block behind [`immediate_output`](RenderResult::immediate_output)
    /// and can be used to route output to a custom sink (e.g. for testing).
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_core::{RenderResult, RenderResultMode};
    ///
    /// let mut result = RenderResult::default();
    /// result.bind_print_hook(|print| {
    ///     println!(
    ///         "[{}] {}",
    ///         if print.mode == RenderResultMode::Stdout {
    ///             "out"
    ///         } else {
    ///             "err"
    ///         },
    ///         print.content
    ///     );
    /// });
    /// result.print("Hello");
    /// ```
    pub fn bind_print_hook(&mut self, hook: impl FnMut(RenderResultPrint) + 'static) -> &mut Self {
        self.print_hook
            .get_or_insert_with(Vec::new)
            .push(Box::new(hook));
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
    /// If this `RenderResult` has print hooks bound but the other does not, the
    /// other's content is emitted through this result's hooks (e.g. flushed to
    /// stdout/stderr) while also being appended to the render buffer.
    ///
    /// The `exit_code` and the print hooks of the other result are **not**
    /// transferred — only its buffered content is merged.
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

        // If self has hooks but the other does not, the other's buffered content
        // was never emitted — flush it through self's hooks while appending.
        let should_emit = self.print_hook.is_some() && other.print_hook.is_none();

        for (content, mode) in other.render_buffer {
            if should_emit {
                self.emit(&content, mode);
            }
            self.render_buffer.push((content, mode));
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
        self.emit(&text, Stdout);
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
        self.emit(&format!("{text}\n"), Stdout);
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
        self.emit(&text, Stderr);
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
        self.emit(&format!("{text}\n"), Stderr);
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
    /// A new `RenderResult` with the same print hooks and `exit_code`, but with
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
            print_hook: self.print_hook,
            exit_code: self.exit_code,
        }
    }

    /// Emits `content` to every bound print hook, if any.
    fn emit(&mut self, content: &str, mode: RenderResultMode) {
        if let Some(hooks) = &mut self.print_hook {
            for hook in hooks {
                hook(RenderResultPrint {
                    content: content.to_string(),
                    mode,
                });
            }
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
    use std::cell::RefCell;
    use std::io::Write as IoWrite;
    use std::rc::Rc;

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

    #[test]
    fn print_hooks_receive_content_and_mode() {
        let mut result = RenderResult::default();
        let captured: Rc<RefCell<Vec<RenderResultPrint>>> = Rc::default();
        let hook_captured = Rc::clone(&captured);
        result.bind_print_hook(move |print| hook_captured.borrow_mut().push(print));

        result.print("Hello");
        result.eprintln("World");

        assert_eq!(
            captured.borrow()[0],
            RenderResultPrint {
                content: "Hello".to_string(),
                mode: RenderResultMode::Stdout
            }
        );
        assert_eq!(
            captured.borrow()[1],
            RenderResultPrint {
                content: "World\n".to_string(),
                mode: RenderResultMode::Stderr
            }
        );
        assert_eq!(result.to_string(), "HelloWorld");
    }

    #[test]
    fn immediate_output_binds_stdout_hook() {
        let mut result = RenderResult::default();
        assert!(result.print_hook.is_none());
        result.immediate_output();
        assert!(result.print_hook.is_some());
    }

    #[test]
    fn append_other_emits_through_hooks_when_self_has_them() {
        let mut dest = RenderResult::default();
        let emitted: Rc<RefCell<Vec<String>>> = Rc::default();
        let hook_emitted = Rc::clone(&emitted);
        dest.bind_print_hook(move |print| hook_emitted.borrow_mut().push(print.content));

        let mut src = RenderResult::default();
        src.append_to_buffer("Hello", RenderResultMode::Stdout);
        dest.append_other(src);

        assert_eq!(emitted.borrow().as_slice(), ["Hello"]);
        assert_eq!(dest.to_string(), "Hello");
    }

    #[test]
    fn append_other_does_not_reemit_when_other_has_hooks() {
        let mut dest = RenderResult::default();
        let emitted: Rc<RefCell<Vec<String>>> = Rc::default();
        let hook_emitted = Rc::clone(&emitted);
        dest.bind_print_hook(move |print| hook_emitted.borrow_mut().push(print.content));

        let mut src = RenderResult::default();
        src.bind_print_hook(|_| {});
        src.append_to_buffer("Hello", RenderResultMode::Stdout);
        dest.append_other(src);

        assert!(emitted.borrow().is_empty());
        assert_eq!(dest.to_string(), "Hello");
    }

    #[test]
    fn write_does_not_emit_through_hooks() {
        let mut result = RenderResult::default();
        result.bind_print_hook(|_| panic!("append_to_buffer must not emit"));

        IoWrite::write(&mut result, b"Hello").unwrap();
        assert_eq!(result.to_string(), "Hello");
    }
}
