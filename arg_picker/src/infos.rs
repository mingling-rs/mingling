use crate::{Pickable, PickerArg, parselib::ParserStyle};

/// Represents the result of parsing or looking up a value.
///
/// This enum is generic over the type being parsed. It models three possible outcomes:
/// - [`Unparsed`](Self::Unparsed): The value has not yet been parsed (default).
/// - [`Parsed`](Self::Parsed): The value was successfully parsed into `Type`.
/// - [`NotFound`](Self::NotFound): The requested value could not be found.
#[derive(Default)]
pub enum PickerArgResult<Type> {
    /// The value has not yet been parsed (default).
    #[default]
    Unparsed,

    /// The value was successfully parsed into `Type`.
    Parsed(Type),

    /// The requested value could not be found.
    NotFound,
}

impl<Type, E> From<Result<Type, E>> for PickerArgResult<Type> {
    /// Converts a `Result<Type, E>` into a `PickerArgResult<Type>`.
    ///
    /// - `Ok(value)` maps to [`Parsed(value)`](Self::Parsed).
    /// - `Err(_)` maps to [`NotFound`](Self::NotFound).
    fn from(result: Result<Type, E>) -> Self {
        result.map_or_else(|_| Self::NotFound, |value| Self::Parsed(value))
    }
}

impl<Type> From<Option<Type>> for PickerArgResult<Type> {
    /// Converts an `Option<Type>` into a `PickerArgResult<Type>`.
    ///
    /// - `Some(value)` maps to [`Parsed(value)`](Self::Parsed).
    /// - `None` maps to [`NotFound`](Self::NotFound).
    fn from(option: Option<Type>) -> Self {
        option.map_or_else(|| Self::NotFound, |value| Self::Parsed(value))
    }
}

impl<Type> PickerArgResult<Type> {
    /// Returns `true` if the result is [`Parsed`](Self::Parsed).
    ///
    /// # Examples
    ///
    /// ```
    /// use arg_picker::PickerArgResult;
    ///
    /// let result: PickerArgResult<i32> = PickerArgResult::Parsed(42);
    /// assert!(result.is_parsed());
    ///
    /// let result: PickerArgResult<i32> = PickerArgResult::NotFound;
    /// assert!(!result.is_parsed());
    /// ```
    pub const fn is_parsed(&self) -> bool {
        matches!(self, Self::Parsed(_))
    }

    /// Returns `true` if the result is [`Parsed`](Self::Parsed) or [`NotFound`](Self::NotFound).
    /// i.e., the value exists (was either found or not yet parsed).
    /// Typically indicates the value was "found" in some sense.
    ///
    /// # Examples
    ///
    /// ```
    /// use arg_picker::PickerArgResult;
    ///
    /// let result: PickerArgResult<i32> = PickerArgResult::Parsed(42);
    /// assert!(result.is_found());
    ///
    /// let result: PickerArgResult<i32> = PickerArgResult::NotFound;
    /// assert!(result.is_found());
    /// ```
    pub const fn is_found(&self) -> bool {
        matches!(self, Self::Parsed(_) | Self::NotFound)
    }

    /// Returns `true` if the result is [`Unparsed`](Self::Unparsed) or [`NotFound`](Self::NotFound).
    ///
    /// # Examples
    ///
    /// ```
    /// use arg_picker::PickerArgResult;
    ///
    /// let result: PickerArgResult<i32> = PickerArgResult::Unparsed;
    /// assert!(result.is_err());
    ///
    /// let result: PickerArgResult<i32> = PickerArgResult::Parsed(10);
    /// assert!(!result.is_err());
    /// ```
    pub const fn is_err(&self) -> bool {
        !matches!(self, Self::Parsed(_))
    }

    /// Returns `Some(&Type)` if [`Parsed`](Self::Parsed), otherwise `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use arg_picker::PickerArgResult;
    ///
    /// let result: PickerArgResult<i32> = PickerArgResult::Parsed(42);
    /// assert_eq!(result.parsed(), Some(&42));
    ///
    /// let result: PickerArgResult<i32> = PickerArgResult::NotFound;
    /// assert_eq!(result.parsed(), None);
    /// ```
    pub const fn parsed(&self) -> Option<&Type> {
        if let Self::Parsed(value) = self {
            Some(value)
        } else {
            None
        }
    }

    /// Returns the contained [`Parsed`](Self::Parsed) value or panics with a given message.
    ///
    /// # Panics
    /// Panics if the value is not [`Parsed`](Self::Parsed), with a message including the provided `msg`.
    ///
    /// # Examples
    ///
    /// ```should_panic
    /// use arg_picker::PickerArgResult;
    ///
    /// let result: PickerArgResult<i32> = PickerArgResult::NotFound;
    /// result.expect("expected a parsed value");
    /// ```
    pub fn expect(self, msg: &str) -> Type {
        match self {
            Self::Parsed(value) => value,
            _ => panic!("{}", msg),
        }
    }

    /// Returns the contained [`Parsed`](Self::Parsed) value or panics.
    ///
    /// # Panics
    /// Panics if the value is not [`Parsed`](Self::Parsed).
    ///
    /// # Examples
    ///
    /// ```
    /// use arg_picker::PickerArgResult;
    ///
    /// let result: PickerArgResult<i32> = PickerArgResult::Parsed(42);
    /// assert_eq!(result.unwrap(), 42);
    /// ```
    ///
    /// ```should_panic
    /// use arg_picker::PickerArgResult;
    ///
    /// let result: PickerArgResult<i32> = PickerArgResult::NotFound;
    /// result.unwrap();
    /// ```
    pub fn unwrap(self) -> Type {
        match self {
            Self::Parsed(value) => value,
            Self::Unparsed => {
                panic!("called `PickerArgResult::unwrap()` on an `Unparsed` value")
            }
            Self::NotFound => {
                panic!("called `PickerArgResult::unwrap()` on a `NotFound` value")
            }
        }
    }

    /// Returns the contained [`Parsed`](Self::Parsed) value or a provided `default`.
    ///
    /// # Examples
    ///
    /// ```
    /// use arg_picker::PickerArgResult;
    ///
    /// let result: PickerArgResult<i32> = PickerArgResult::Parsed(42);
    /// assert_eq!(result.unwrap_or(0), 42);
    ///
    /// let result: PickerArgResult<i32> = PickerArgResult::NotFound;
    /// assert_eq!(result.unwrap_or(0), 0);
    /// ```
    pub fn unwrap_or(self, default: Type) -> Type {
        match self {
            Self::Parsed(value) => value,
            _ => default,
        }
    }

    /// Returns the contained [`Parsed`](Self::Parsed) value or computes it from a closure.
    ///
    /// # Examples
    ///
    /// ```
    /// use arg_picker::PickerArgResult;
    ///
    /// let result: PickerArgResult<i32> = PickerArgResult::Parsed(42);
    /// assert_eq!(result.unwrap_or_else(|| 0), 42);
    ///
    /// let result: PickerArgResult<i32> = PickerArgResult::NotFound;
    /// assert_eq!(result.unwrap_or_else(|| 0), 0);
    /// ```
    pub fn unwrap_or_else<F: FnOnce() -> Type>(self, f: F) -> Type {
        match self {
            Self::Parsed(value) => value,
            _ => f(),
        }
    }

    /// Returns the contained [`Parsed`](Self::Parsed) value or the default value of `Type`.
    ///
    /// # Examples
    ///
    /// ```
    /// use arg_picker::PickerArgResult;
    ///
    /// let result: PickerArgResult<i32> = PickerArgResult::Parsed(42);
    /// assert_eq!(result.unwrap_or_default(), 42);
    ///
    /// let result: PickerArgResult<i32> = PickerArgResult::NotFound;
    /// assert_eq!(result.unwrap_or_default(), 0);
    /// ```
    pub fn unwrap_or_default(self) -> Type
    where
        Type: Default,
    {
        match self {
            Self::Parsed(value) => value,
            _ => Type::default(),
        }
    }

    /// Converts `PickerArgResult<Type>` into `Option<Type>`.
    ///
    /// Returns `Some(Type)` if [`Parsed`](Self::Parsed), otherwise `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use arg_picker::PickerArgResult;
    ///
    /// let result: PickerArgResult<i32> = PickerArgResult::Parsed(42);
    /// assert_eq!(result.to_option(), Some(42));
    ///
    /// let result: PickerArgResult<i32> = PickerArgResult::NotFound;
    /// assert_eq!(result.to_option(), None);
    ///
    /// let result: PickerArgResult<i32> = PickerArgResult::Unparsed;
    /// assert_eq!(result.to_option(), None);
    /// ```
    pub fn to_option(self) -> Option<Type> {
        match self {
            Self::Parsed(value) => Some(value),
            _ => None,
        }
    }
}

// In PickerArgInfo, positional, optional, multi, and is_flag may coexist.
#[allow(clippy::struct_excessive_bools)]
/// Represents metadata about a command-line argument or flag.
///
/// This struct stores all relevant information about a tag/argument that can be used
/// for parsing command-line inputs. It includes the short form (e.g., `-n`), long form
/// (e.g., `--name`), aliases, and various flags that control parsing behavior.
pub struct PickerArgInfo<'a> {
    /// The short form of the tag, e.g. `'n'` for `-n`.
    pub short: Option<char>,
    /// The long form of the tag, e.g. `"name"` for `--name`.
    pub long: Option<&'a str>,
    /// Alternative names for the tag, e.g. `["-N", "--nickname"]`.
    pub alias: Option<Vec<&'a str>>,
    /// Whether this tag is a positional argument (no `-` or `--` prefix).
    pub positional: bool,
    /// Whether this tag is optional or required.
    pub optional: bool,
    /// Whether this tag can accept multiple values.
    pub multi: bool,
    /// Whether this tag participates in parsing after a `--` separator.
    pub is_flag: bool,
}

impl<'a, T> From<PickerArg<'a, T>> for PickerArgInfo<'a>
where
    T: Pickable<'a>,
{
    fn from(value: PickerArg<'a, T>) -> Self {
        value.into_info()
    }
}

impl<'a, T: Pickable<'a>> From<&'a PickerArg<'a, T>> for PickerArgInfo<'a> {
    fn from(value: &'a PickerArg<'a, T>) -> Self {
        let (long, alias) = if value.full.is_empty() {
            (None, None)
        } else {
            let long = Some(value.full[0]);
            let alias = if value.full.len() > 1 {
                Some(value.full[1..].to_vec())
            } else {
                None
            };
            (long, alias)
        };

        Self {
            short: value.short,
            long,
            alias,
            positional: value.positional,
            optional: false,
            multi: false,
            is_flag: false,
        }
    }
}

impl<'a> PickerArgInfo<'a> {
    /// Create a new `PickerTag` with default values.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            short: None,
            long: None,
            alias: None,
            positional: false,
            optional: false,
            multi: false,
            is_flag: false,
        }
    }

    /// Set the short flag (e.g., `'n'` for `-n`).
    #[must_use]
    pub const fn with_short(mut self, short: char) -> Self {
        self.short = Some(short);
        self
    }

    /// Set the long flag (e.g., `"name"` for `--name`).
    #[must_use]
    pub const fn with_long(mut self, long: &'a str) -> Self {
        self.long = Some(long);
        self
    }

    /// Set aliases for the tag.
    #[must_use]
    pub fn with_alias(mut self, alias: Vec<&'a str>) -> Self {
        self.alias = Some(alias);
        self
    }

    /// Mark the tag as positional.
    #[must_use]
    pub const fn with_positional(mut self, positional: bool) -> Self {
        self.positional = positional;
        self
    }

    /// Mark the tag as optional.
    #[must_use]
    pub const fn with_optional(mut self, optional: bool) -> Self {
        self.optional = optional;
        self
    }

    /// Mark the tag as multi-value.
    #[must_use]
    pub const fn with_multi(mut self, multi: bool) -> Self {
        self.multi = multi;
        self
    }

    /// Mark the tag as a flag that participates in parsing after `--`.
    #[must_use]
    pub const fn with_is_flag(mut self, is_flag: bool) -> Self {
        self.is_flag = is_flag;
        self
    }

    /// Set the short flag (e.g., `'n'` for `-n`).
    pub const fn set_short(&mut self, short: char) -> &mut Self {
        self.short = Some(short);
        self
    }

    /// Set the long flag (e.g., `"name"` for `--name`).
    pub const fn set_long(&mut self, long: &'a str) -> &mut Self {
        self.long = Some(long);
        self
    }

    /// Set aliases for the tag.
    pub fn set_alias(&mut self, alias: Vec<&'a str>) -> &mut Self {
        self.alias = Some(alias);
        self
    }

    /// Set whether this tag is positional.
    pub const fn set_positional(&mut self, positional: bool) -> &mut Self {
        self.positional = positional;
        self
    }

    /// Set whether this tag is optional.
    pub const fn set_optional(&mut self, optional: bool) -> &mut Self {
        self.optional = optional;
        self
    }

    /// Set whether this tag accepts multiple values.
    pub const fn set_multi(&mut self, multi: bool) -> &mut Self {
        self.multi = multi;
        self
    }

    /// Set whether this tag participates in parsing after a `--` separator.
    pub const fn set_is_flag(&mut self, is_flag: bool) -> &mut Self {
        self.is_flag = is_flag;
        self
    }

    /// Returns the short flag string, e.g. `-n` for short `n`.
    ///
    /// Uses [`ParserStyle::global_style()`] to format the flag.
    ///
    /// # Returns
    ///
    /// - `Some(String)` if `self.short` is set.
    /// - `None` if `self.short` is `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use arg_picker::PickerArgInfo;
    ///
    /// let info = PickerArgInfo::new().with_short('n');
    /// assert_eq!(info.short_flag(), Some("-n".to_string()));
    ///
    /// let info = PickerArgInfo::new();
    /// assert_eq!(info.short_flag(), None);
    /// ```
    #[must_use]
    pub fn short_flag(&self) -> Option<String> {
        let short = self.short?;
        Some(ParserStyle::global_style().flag_string(short))
    }

    /// Returns the long flag string, e.g. `--name` for long `"name"`.
    ///
    /// Uses [`ParserStyle::global_style()`] to format the flag.
    ///
    /// # Returns
    ///
    /// - `Some(String)` if `self.long` is set.
    /// - `None` if `self.long` is `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use arg_picker::PickerArgInfo;
    ///
    /// let info = PickerArgInfo::new().with_long("name");
    /// assert_eq!(info.long_flag(), Some("--name".to_string()));
    ///
    /// let info = PickerArgInfo::new();
    /// assert_eq!(info.long_flag(), None);
    /// ```
    #[must_use]
    pub fn long_flag(&self) -> Option<String> {
        let long = self.long?;
        Some(ParserStyle::global_style().flag_string(long))
    }
}

impl Default for PickerArgInfo<'_> {
    fn default() -> Self {
        Self::new()
    }
}
