use std::{
    fmt::{Display, Formatter},
    io::Write,
    ops::Deref,
};

/// Render result, containing the rendered text content.
#[derive(Default, Debug, PartialEq)]
pub struct RenderResult {
    render_text: String,

    /// The exit code to return from the rendering process.
    ///
    /// A value of `0` indicates success, while non-zero values indicate
    /// various error conditions.
    pub exit_code: i32,
}

impl Write for RenderResult {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let s = std::str::from_utf8(buf).map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "not valid UTF-8")
        })?;
        self.render_text.push_str(s);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Display for RenderResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.render_text.trim())
    }
}

impl Deref for RenderResult {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.render_text
    }
}

impl From<()> for RenderResult {
    fn from(_value: ()) -> Self {
        RenderResult::new()
    }
}

macro_rules! impl_from_int {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl From<$ty> for RenderResult {
                fn from(exit_code: $ty) -> Self {
                    RenderResult {
                        exit_code: exit_code as i32,
                        ..Default::default()
                    }
                }
            }
        )+
    };
}

impl_from_int!(i32, i16, i8, u32, u16, u8, usize);

impl From<&String> for RenderResult {
    fn from(value: &String) -> Self {
        RenderResult {
            render_text: value.clone(),
            exit_code: 0,
        }
    }
}

impl From<String> for RenderResult {
    fn from(value: String) -> Self {
        RenderResult {
            render_text: value,
            exit_code: 0,
        }
    }
}

impl From<&str> for RenderResult {
    fn from(value: &str) -> Self {
        RenderResult {
            render_text: value.to_string(),
            exit_code: 0,
        }
    }
}

impl From<RenderResult> for String {
    fn from(result: RenderResult) -> Self {
        result.render_text
    }
}

impl From<&RenderResult> for String {
    fn from(result: &RenderResult) -> Self {
        result.render_text.clone()
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
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends the given text to the rendered content.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_core::RenderResult;
    /// use std::ops::Deref;
    ///
    /// let mut result = RenderResult::default();
    /// result.print("Hello");
    /// result.print(", world!");
    /// assert_eq!(result.deref(), "Hello, world!");
    /// ```
    pub fn print(&mut self, text: impl AsRef<str>) {
        self.render_text.push_str(text.as_ref());
    }

    /// Appends the given text followed by a newline to the rendered content.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_core::RenderResult;
    /// use std::ops::Deref;
    ///
    /// let mut result = RenderResult::default();
    /// result.println("First line");
    /// result.println("Second line");
    /// assert_eq!(result.deref(), "First line\nSecond line\n");
    /// ```
    pub fn println(&mut self, text: impl AsRef<str>) {
        self.render_text.push_str(text.as_ref());
        self.render_text.push('\n');
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
        self.render_text.clear();
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
    fn print_appends_text() {
        let mut result = RenderResult::default();
        result.print("Hello");
        assert_eq!(result.deref(), "Hello");
    }

    #[test]
    fn println_appends_text_with_newline() {
        let mut result = RenderResult::default();
        result.println("Hello");
        assert_eq!(result.deref(), "Hello\n");
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
    fn write_appends_utf8_bytes() {
        let mut result = RenderResult::default();
        let n = IoWrite::write(&mut result, b"hello").unwrap();
        assert_eq!(n, 5);
        assert_eq!(result.deref(), "hello");
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
        let formatted = format!("{}", result);
        assert_eq!(formatted, "hello world\n");
    }

    #[test]
    fn deref_exposes_inner_text_as_str() {
        let mut result = RenderResult::default();
        result.print("test");

        let s: &str = &result;
        assert_eq!(s, "test");
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
}
