use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

/// Defines the style of command-line argument parsing (prefixes, separators, etc.).
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ParserStyle<'a> {
    /// End-of-options marker (e.g., `--`)
    end_of_options: &'a str,

    /// Prefix for long options (e.g., `--` or `/`)
    long_prefix: &'a str,

    /// Prefix for short options (e.g., `-` or `/`)
    short_prefix: &'a str,

    /// Prefix for combined short flags (e.g., `-abc`)
    combine_prefix: &'a str,

    /// Separator between name and value (e.g., `=` or `:`)
    value_separator: char,

    /// Whether option names are case-sensitive
    case_sensitive: bool,

    /// Whether combining short flags is allowed (e.g., `-abc` for `-a -b -c`)
    allow_combine: bool,
}

/// Unix-like style (e.g., `--verbose`, `-v`, `--name=value`)
pub const UNIX_STYLE: ParserStyle = ParserStyle {
    end_of_options: "--",
    long_prefix: "--",
    short_prefix: "-",
    combine_prefix: "-",
    value_separator: '=',
    case_sensitive: true,
    allow_combine: true,
};

/// PowerShell style (e.g., `-Verbose`, `-Name:value`)
pub const POWERSHELL_STYLE: ParserStyle = ParserStyle {
    end_of_options: "--",
    long_prefix: "-",
    short_prefix: "-",
    combine_prefix: "-",
    value_separator: ':',
    case_sensitive: false,
    allow_combine: false,
};

/// Windows-style command-line (e.g., `/Verbose`, `/Name:value`)
pub const WINDOWS_STYLE: ParserStyle = ParserStyle {
    end_of_options: "--",
    long_prefix: "/",
    short_prefix: "/",
    combine_prefix: "/",
    value_separator: ':',
    case_sensitive: false,
    allow_combine: false,
};

static GLOBAL_STYLE: OnceLock<ParserStyle<'static>> = OnceLock::new();
static GLOBAL_STYLE_SET: AtomicBool = AtomicBool::new(false);

impl<'a> ParserStyle<'a> {
    /// Sets the global parser style.
    ///
    /// This function can only be called once. Subsequent calls will have no effect.
    /// The style is stored as a static reference; the provided style must be a static
    /// constant (e.g., `&'static ParserStyle`). Use the built-in constants like
    /// `UNIX_STYLE`, `POWERSHELL_STYLE`, or `WINDOWS_STYLE`.
    pub fn set_global_style(style: &'static ParserStyle<'static>) {
        if !GLOBAL_STYLE_SET.load(Ordering::Acquire) && GLOBAL_STYLE.set(*style).is_ok() {
            GLOBAL_STYLE_SET.store(true, Ordering::Release);
        }
    }

    /// Returns the global parser style, or `None` if not yet set.
    pub fn global_style() -> Option<&'static ParserStyle<'static>> {
        if GLOBAL_STYLE_SET.load(Ordering::Acquire) {
            GLOBAL_STYLE.get()
        } else {
            None
        }
    }
}
