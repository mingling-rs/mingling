use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::parselib::ParserStyleNamingCase::{Kebab, Pascal};

/// Defines the style of command-line argument parsing (prefixes, separators, etc.).
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ParserStyle<'a> {
    /// End-of-options marker (e.g., `--`)
    pub end_of_options: &'a str,

    /// Prefix for long options (e.g., `--` or `/`)
    pub long_prefix: &'a str,

    /// Prefix for short options (e.g., `-` or `/`)
    pub short_prefix: &'a str,

    /// Prefix for combined short flags (e.g., `-abc`)
    pub combine_prefix: &'a str,

    /// Separator between name and value (e.g., `=` or `:`)
    pub value_separator: char,

    /// Whether option names are case-sensitive
    pub case_sensitive: bool,

    /// Whether combining short flags is allowed (e.g., `-abc` for `-a -b -c`)
    pub allow_combine: bool,

    /// Naming case
    pub naming_case: ParserStyleNamingCase,
}

impl<'a> ParserStyle<'a> {
    /// Formats a flag (short or long) into a full command-line option string.
    ///
    /// This method takes any type that can be converted into a `FlagStr` and produces
    /// a complete option string by prepending the appropriate prefix.
    ///
    /// # Examples
    ///
    /// ```
    /// # use arg_picker::parselib::{ParserStyle, FlagStr, UNIX_STYLE};
    /// let style = &UNIX_STYLE;
    ///
    /// assert_eq!(style.flag_string('v'), "-v");
    /// assert_eq!(style.flag_string("verbose"), "--verbose");
    /// ```
    ///
    /// # Parameters
    ///
    /// * `flag` - A value that can be converted to `FlagStr`, either a `char` for short flags
    ///   or a `&str` for long flags.
    ///
    /// # Returns
    ///
    /// A `String` with the prefix and the flag name combined.
    #[must_use]
    #[inline]
    pub fn flag_string<F>(&self, flag: F) -> String
    where
        F: Into<FlagStr<'a>>,
    {
        match flag.into() {
            FlagStr::Short(short) => format!("{}{}", self.short_prefix, short),
            FlagStr::Long(long) => format!("{}{}", self.long_prefix, long),
        }
    }
}

/// Represents a flag name for command-line argument parsing.
///
/// This enum can hold either a short flag (a single character, e.g., `'v'` for `-v`)
/// or a long flag (a string, e.g., `"verbose"` for `--verbose`).
///
/// # Examples
///
/// ```
/// use arg_picker::parselib::FlagStr;
///
/// let short: FlagStr = 'v'.into();
/// let long: FlagStr = "verbose".into();
/// ```
pub enum FlagStr<'a> {
    /// A short flag represented by a single character.
    Short(char),
    /// A long flag represented by a string slice.
    Long(&'a str),
}

impl From<char> for FlagStr<'_> {
    /// Converts a single character into a `FlagStr::Short`.
    fn from(c: char) -> Self {
        FlagStr::Short(c)
    }
}

impl<'a> From<&'a str> for FlagStr<'a> {
    /// Converts a string slice into a `FlagStr::Long`.
    fn from(s: &'a str) -> Self {
        FlagStr::Long(s)
    }
}

impl<'a> From<&'a String> for FlagStr<'a> {
    /// Converts a reference to a `String` into a `FlagStr::Long`.
    fn from(s: &'a String) -> Self {
        FlagStr::Long(s.as_str())
    }
}

/// Defines the naming convention for command-line option names.
///
/// Each variant represents a different case format that can be applied
/// to option names (e.g., long option names) during parsing or generation.
///
/// # Examples
///
/// ```
/// # use arg_picker::IntoPicker;
/// use arg_picker::parselib::ParserStyleNamingCase;
///
/// let case = ParserStyleNamingCase::Kebab;
/// assert_eq!(
///     case.convert("brew_coffee".to_string()),
///     "brew-coffee".to_string()
/// );
/// ```
#[repr(u8)]
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum ParserStyleNamingCase {
    /// `snake_case` format: words are separated by underscores, all lowercase.
    ///
    /// Example: `brew_coffee`
    #[default]
    Snake,
    /// `camelCase` format: first word is lowercase, subsequent words are capitalized.
    ///
    /// Example: `brewCoffee`
    Camel,
    /// `PascalCase` format: every word starts with an uppercase letter.
    ///
    /// Example: `BrewCoffee`
    Pascal,
    /// `kebab-case` format: words are separated by hyphens, all lowercase.
    ///
    /// Example: `brew-coffee`
    Kebab,
    /// `dot.case` format: words are separated by dots, all lowercase.
    ///
    /// Example: `brew.coffee`
    Dot,
    /// `Title Case` format: words are separated by spaces, each word capitalized.
    ///
    /// Example: `Brew Coffee`
    Title,
    /// `lower case` format: words are separated by spaces, all lowercase.
    ///
    /// Example: `brew coffee`
    Lower,
    /// `UPPER CASE` format: words are separated by spaces, all uppercase.
    ///
    /// Example: `BREW COFFEE`
    Upper,
}

impl ParserStyleNamingCase {
    /// Converts the input string `s` to the naming case represented by this variant.
    ///
    /// This method takes any type `S` that can be converted into a `String` and
    /// produced from a `String`, applies the corresponding case transformation,
    /// and returns the result.
    ///
    /// # Examples
    ///
    /// ```
    /// use arg_picker::parselib::ParserStyleNamingCase;
    ///
    /// let camel = ParserStyleNamingCase::Camel;
    /// assert_eq!(camel.convert("brew_coffee".to_string()), "brewCoffee");
    ///
    /// let kebab = ParserStyleNamingCase::Kebab;
    /// assert_eq!(kebab.convert("BrewCoffee".to_string()), "brew-coffee");
    /// ```
    pub fn convert<S>(&self, s: S) -> S
    where
        S: Into<String> + From<String>,
    {
        match self {
            Self::Camel => just_fmt::camel_case!(s.into()).into(),
            Self::Pascal => just_fmt::pascal_case!(s.into()).into(),
            Self::Kebab => just_fmt::kebab_case!(s.into()).into(),
            Self::Snake => just_fmt::snake_case!(s.into()).into(),
            Self::Dot => just_fmt::dot_case!(s.into()).into(),
            Self::Title => just_fmt::title_case!(s.into()).into(),
            Self::Lower => just_fmt::lower_case!(s.into()).into(),
            Self::Upper => just_fmt::upper_case!(s.into()).into(),
        }
    }
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
    naming_case: Kebab,
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
    naming_case: Pascal,
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
    naming_case: Pascal,
};

static GLOBAL_STYLE: OnceLock<ParserStyle<'static>> = OnceLock::new();
static GLOBAL_STYLE_SET: AtomicBool = AtomicBool::new(false);

impl ParserStyle<'_> {
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

    /// Returns the global parser style, falling back to `UNIX_STYLE` if not set.
    pub fn global_style() -> &'static ParserStyle<'static> {
        GLOBAL_STYLE.get().unwrap_or(&UNIX_STYLE)
    }
}
