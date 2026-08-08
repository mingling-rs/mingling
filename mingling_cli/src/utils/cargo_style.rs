use colored::Colorize;

/// Formats a message in cargo-style format with a bold green prefix.
///
/// The message should be in the format `prefix: content`. The prefix will be
/// bold green and right-padded to 12 characters. If there is no colon in the
/// string, the entire string is printed as content with an empty prefix.
///
/// # Macros
///
/// - `format_cargo!("prefix: {}", arg)` — format-style invocation
/// - `format_cargo!(expr)` — direct expression invocation
///
/// # Panics
///
/// Panics if the prefix (text before the first `:`) exceeds 12 characters.
///
/// # Examples
///
/// ```ignore
/// format_cargo!("Compiling: hello.rs");
/// // Output: "   Compiling hello.rs" (green bold "Compiling" padded to 12)
/// ```
#[macro_export]
macro_rules! format_cargo {
    ($fmt:literal, $($arg:tt)*) => {
        $crate::utils::cargo_style::get_cargo_info_format(format!($fmt, $($arg)*))
    };
    ($cmd:expr) => {
        $crate::utils::cargo_style::get_cargo_info_format($cmd)
    };
}

/// Formats an error message in cargo-style format with a bold red "error" prefix.
///
/// # Macros
///
/// - `eformat_cargo!("prefix: {}", arg)` — format-style invocation
/// - `eformat_cargo!(expr)` — direct expression invocation
///
/// # Examples
///
/// ```ignore
/// eformat_cargo!("failed to parse input");
/// // Output: "error: failed to parse input" (red bold "error")
/// ```
#[macro_export]
macro_rules! eformat_cargo {
    ($fmt:literal, $($arg:tt)*) => {
        $crate::utils::cargo_style::get_cargo_error_format(format!($fmt, $($arg)*))
    };
    ($cmd:expr) => {
        $crate::utils::cargo_style::get_cargo_error_format($cmd)
    };
}

/// Formats a help message in cargo-style format with a bright white "help" prefix.
///
/// # Macros
///
/// - `hformat_cargo!("prefix: {}", arg)` — format-style invocation
/// - `hformat_cargo!(expr)` — direct expression invocation
///
/// # Examples
///
/// ```ignore
/// hformat_cargo!("use --verbose for more info");
/// // Output: "help: use --verbose for more info" (bright white "help")
/// ```
#[macro_export]
macro_rules! hformat_cargo {
    ($fmt:literal, $($arg:tt)*) => {
        $crate::utils::cargo_style::get_cargo_help_format(format!($fmt, $($arg)*))
    };
    ($cmd:expr) => {
        $crate::utils::cargo_style::get_cargo_help_format($cmd)
    };
}

/// Print a message in cargo-style format into a `RenderResult` buffer.
///
/// The first argument must be the renderer's buffer variable (e.g.
/// `__render_result_buffer` inside a `#[renderer(buffer)]` function).
///
/// # Examples
///
/// ```ignore
/// #[renderer(buffer)]
/// fn render_ok(_: ResOk) {
///     println_cargo!(__render_result_buffer, "Compiling: {}", name);
/// }
/// ```
#[macro_export]
macro_rules! println_cargo {
    ($buf:ident, $fmt:literal, $($arg:tt)*) => {
        ::mingling::macros::r_println!($buf, "{}", $crate::utils::cargo_style::get_cargo_info_format(format!($fmt, $($arg)*)))
    };
    ($buf:ident, $cmd:expr) => {
        ::mingling::macros::r_println!($buf, "{}", $crate::utils::cargo_style::get_cargo_info_format($cmd))
    };
}

/// Print an error message in cargo-style format into a `RenderResult` buffer.
///
/// # Examples
///
/// ```ignore
/// #[renderer(buffer)]
/// fn render_err(_: ResErr) {
///     eprintln_cargo!(__render_result_buffer, "failed to parse input");
/// }
/// ```
#[macro_export]
macro_rules! eprintln_cargo {
    ($buf:ident, $fmt:literal, $($arg:tt)*) => {
        ::mingling::macros::r_println!($buf, "{}", $crate::utils::cargo_style::get_cargo_error_format(format!($fmt, $($arg)*)))
    };
    ($buf:ident, $cmd:expr) => {
        ::mingling::macros::r_println!($buf, "{}", $crate::utils::cargo_style::get_cargo_error_format($cmd))
    };
}

/// Print a help message in cargo-style format into a `RenderResult` buffer.
///
/// # Examples
///
/// ```ignore
/// #[renderer(buffer)]
/// fn render_help(_: ResHelp) {
///     hprintln_cargo!(__render_result_buffer, "use --verbose for more info");
/// }
/// ```
#[macro_export]
macro_rules! hprintln_cargo {
    ($buf:ident, $fmt:literal, $($arg:tt)*) => {
        ::mingling::macros::r_println!($buf, "{}", $crate::utils::cargo_style::get_cargo_help_format(format!($fmt, $($arg)*)))
    };
    ($buf:ident, $cmd:expr) => {
        ::mingling::macros::r_println!($buf, "{}", $crate::utils::cargo_style::get_cargo_help_format($cmd))
    };
}

/// Format a message in cargo style format, with bold green prefix.
///
/// The input string is split at the first `:`. The part before the colon becomes
/// the prefix (bold green, right-padded to 12 characters), and the part after
/// becomes the content.
///
/// If no colon is found, the entire string is treated as content and no prefix
/// is shown.
///
/// # Panics
///
/// Panics if the prefix (text before the first `:`) exceeds 12 characters.
///
/// # Examples
///
/// ```ignore
/// get_cargo_info_format("Compiling: my_program.rs");
/// // returns "   Compiling my_program.rs"
/// ```
pub fn get_cargo_info_format(str: impl Into<String>) -> String {
    let s = str.into();
    let (prefix, content) = if let Some(pos) = s.find(':') {
        (
            s[..pos].trim().to_string(),
            s[pos + 1..].trim_start().to_string(),
        )
    } else {
        (String::new(), s.trim().to_string())
    };

    assert!(
        prefix.len() <= 12,
        "prefix length exceeds 12: '{}' has length {}",
        prefix,
        prefix.len()
    );

    let padding = " ".repeat(12 - prefix.len());

    format!(
        "{}{} {}",
        padding,
        prefix.bold().bright_green(),
        content.trim()
    )
}

/// Format an error message in cargo style format, with bold red "error" prefix.
///
/// The input string is printed as the error content, prefixed by a bold red
/// `error:` label.
///
/// # Examples
///
/// ```ignore
/// get_cargo_error_format("something went wrong");
/// // returns "error: something went wrong"
/// ```
pub fn get_cargo_error_format(str: impl Into<String>) -> String {
    format!("{}: {}", "error".bold().bright_red(), str.into())
}

/// Format a help message in cargo style format, with bright white "help" prefix.
///
/// The input string is printed as the help content, prefixed by a bright white
/// `help:` label (not bold).
///
/// # Examples
///
/// ```ignore
/// get_cargo_help_format("use --verbose for more info");
/// // returns "help: use --verbose for more info"
/// ```
pub fn get_cargo_help_format(str: impl Into<String>) -> String {
    format!("{}: {}", "help".bright_white(), str.into())
}
