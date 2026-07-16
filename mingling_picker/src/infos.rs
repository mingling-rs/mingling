use crate::{Pickable, PickerArg};

/// Represents the result of parsing or looking up a value.
///
/// This enum is generic over the type being parsed. It models three possible outcomes:
/// - [`Unparsed`](PickerArgResult::Unparsed): The value has not yet been parsed (default).
/// - [`Parsed`](PickerArgResult::Parsed): The value was successfully parsed into `Type`.
/// - [`NotFound`](PickerArgResult::NotFound): The requested value could not be found.
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
    /// - `Ok(value)` maps to [`Parsed(value)`](PickerArgResult::Parsed).
    /// - `Err(_)` maps to [`NotFound`](PickerArgResult::NotFound).
    fn from(result: Result<Type, E>) -> Self {
        match result {
            Ok(value) => PickerArgResult::Parsed(value),
            Err(_) => PickerArgResult::NotFound,
        }
    }
}

impl<Type> From<Option<Type>> for PickerArgResult<Type> {
    /// Converts an `Option<Type>` into a `PickerArgResult<Type>`.
    ///
    /// - `Some(value)` maps to [`Parsed(value)`](PickerArgResult::Parsed).
    /// - `None` maps to [`NotFound`](PickerArgResult::NotFound).
    fn from(option: Option<Type>) -> Self {
        match option {
            Some(value) => PickerArgResult::Parsed(value),
            None => PickerArgResult::NotFound,
        }
    }
}

impl<Type> PickerArgResult<Type> {
    /// Returns `true` if the result is [`Parsed`](PickerArgResult::Parsed).
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_picker::PickerArgResult;
    ///
    /// let result: PickerArgResult<i32> = PickerArgResult::Parsed(42);
    /// assert!(result.is_parsed());
    ///
    /// let result: PickerArgResult<i32> = PickerArgResult::NotFound;
    /// assert!(!result.is_parsed());
    /// ```
    pub fn is_parsed(&self) -> bool {
        matches!(self, PickerArgResult::Parsed(_))
    }

    /// Returns `true` if the result is [`Parsed`](PickerArgResult::Parsed) or [`NotFound`](PickerArgResult::NotFound).
    /// i.e., the value exists (was either found or not yet parsed).
    /// Typically indicates the value was "found" in some sense.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_picker::PickerArgResult;
    ///
    /// let result: PickerArgResult<i32> = PickerArgResult::Parsed(42);
    /// assert!(result.is_found());
    ///
    /// let result: PickerArgResult<i32> = PickerArgResult::NotFound;
    /// assert!(result.is_found());
    /// ```
    pub fn is_found(&self) -> bool {
        matches!(self, PickerArgResult::Parsed(_) | PickerArgResult::NotFound)
    }

    /// Returns `true` if the result is [`Unparsed`](PickerArgResult::Unparsed) or [`NotFound`](PickerArgResult::NotFound).
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_picker::PickerArgResult;
    ///
    /// let result: PickerArgResult<i32> = PickerArgResult::Unparsed;
    /// assert!(result.is_err());
    ///
    /// let result: PickerArgResult<i32> = PickerArgResult::Parsed(10);
    /// assert!(!result.is_err());
    /// ```
    pub fn is_err(&self) -> bool {
        !matches!(self, PickerArgResult::Parsed(_))
    }

    /// Returns `Some(&Type)` if [`Parsed`](PickerArgResult::Parsed), otherwise `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_picker::PickerArgResult;
    ///
    /// let result: PickerArgResult<i32> = PickerArgResult::Parsed(42);
    /// assert_eq!(result.parsed(), Some(&42));
    ///
    /// let result: PickerArgResult<i32> = PickerArgResult::NotFound;
    /// assert_eq!(result.parsed(), None);
    /// ```
    pub fn parsed(&self) -> Option<&Type> {
        if let PickerArgResult::Parsed(value) = self {
            Some(value)
        } else {
            None
        }
    }

    /// Returns the contained [`Parsed`](PickerArgResult::Parsed) value or panics with a given message.
    ///
    /// # Panics
    /// Panics if the value is not [`Parsed`](PickerArgResult::Parsed), with a message including the provided `msg`.
    ///
    /// # Examples
    ///
    /// ```should_panic
    /// use mingling_picker::PickerArgResult;
    ///
    /// let result: PickerArgResult<i32> = PickerArgResult::NotFound;
    /// result.expect("expected a parsed value");
    /// ```
    pub fn expect(self, msg: &str) -> Type {
        match self {
            PickerArgResult::Parsed(value) => value,
            _ => panic!("{}", msg),
        }
    }

    /// Returns the contained [`Parsed`](PickerArgResult::Parsed) value or panics.
    ///
    /// # Panics
    /// Panics if the value is not [`Parsed`](PickerArgResult::Parsed).
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_picker::PickerArgResult;
    ///
    /// let result: PickerArgResult<i32> = PickerArgResult::Parsed(42);
    /// assert_eq!(result.unwrap(), 42);
    /// ```
    ///
    /// ```should_panic
    /// use mingling_picker::PickerArgResult;
    ///
    /// let result: PickerArgResult<i32> = PickerArgResult::NotFound;
    /// result.unwrap();
    /// ```
    pub fn unwrap(self) -> Type {
        match self {
            PickerArgResult::Parsed(value) => value,
            PickerArgResult::Unparsed => {
                panic!("called `PickerArgResult::unwrap()` on an `Unparsed` value")
            }
            PickerArgResult::NotFound => {
                panic!("called `PickerArgResult::unwrap()` on a `NotFound` value")
            }
        }
    }

    /// Returns the contained [`Parsed`](PickerArgResult::Parsed) value or a provided `default`.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_picker::PickerArgResult;
    ///
    /// let result: PickerArgResult<i32> = PickerArgResult::Parsed(42);
    /// assert_eq!(result.unwrap_or(0), 42);
    ///
    /// let result: PickerArgResult<i32> = PickerArgResult::NotFound;
    /// assert_eq!(result.unwrap_or(0), 0);
    /// ```
    pub fn unwrap_or(self, default: Type) -> Type {
        match self {
            PickerArgResult::Parsed(value) => value,
            _ => default,
        }
    }

    /// Returns the contained [`Parsed`](PickerArgResult::Parsed) value or computes it from a closure.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_picker::PickerArgResult;
    ///
    /// let result: PickerArgResult<i32> = PickerArgResult::Parsed(42);
    /// assert_eq!(result.unwrap_or_else(|| 0), 42);
    ///
    /// let result: PickerArgResult<i32> = PickerArgResult::NotFound;
    /// assert_eq!(result.unwrap_or_else(|| 0), 0);
    /// ```
    pub fn unwrap_or_else<F: FnOnce() -> Type>(self, f: F) -> Type {
        match self {
            PickerArgResult::Parsed(value) => value,
            _ => f(),
        }
    }

    /// Returns the contained [`Parsed`](PickerArgResult::Parsed) value or the default value of `Type`.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_picker::PickerArgResult;
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
            PickerArgResult::Parsed(value) => value,
            _ => Type::default(),
        }
    }

    /// Converts `PickerArgResult<Type>` into `Option<Type>`.
    ///
    /// Returns `Some(Type)` if [`Parsed`](PickerArgResult::Parsed), otherwise `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_picker::PickerArgResult;
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
            PickerArgResult::Parsed(value) => Some(value),
            _ => None,
        }
    }
}

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
        let (long, alias) = match value.full.len() {
            0 => (None, None),
            _ => {
                let long = Some(value.full[0]);
                let alias = if value.full.len() > 1 {
                    Some(value.full[1..].to_vec())
                } else {
                    None
                };
                (long, alias)
            }
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

impl<'a, T: Pickable<'a>> From<&'a PickerArg<'a, T>> for PickerArgInfo<'a> {
    fn from(value: &'a PickerArg<'a, T>) -> Self {
        let (long, alias) = match value.full.len() {
            0 => (None, None),
            _ => {
                let long = Some(value.full[0]);
                let alias = if value.full.len() > 1 {
                    Some(value.full[1..].to_vec())
                } else {
                    None
                };
                (long, alias)
            }
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
    pub fn new() -> Self {
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
    pub fn with_short(mut self, short: char) -> Self {
        self.short = Some(short);
        self
    }

    /// Set the long flag (e.g., `"name"` for `--name`).
    pub fn with_long(mut self, long: &'a str) -> Self {
        self.long = Some(long);
        self
    }

    /// Set aliases for the tag.
    pub fn with_alias(mut self, alias: Vec<&'a str>) -> Self {
        self.alias = Some(alias);
        self
    }

    /// Mark the tag as positional.
    pub fn with_positional(mut self, positional: bool) -> Self {
        self.positional = positional;
        self
    }

    /// Mark the tag as optional.
    pub fn with_optional(mut self, optional: bool) -> Self {
        self.optional = optional;
        self
    }

    /// Mark the tag as multi-value.
    pub fn with_multi(mut self, multi: bool) -> Self {
        self.multi = multi;
        self
    }

    /// Mark the tag as a flag that participates in parsing after `--`.
    pub fn with_is_flag(mut self, is_flag: bool) -> Self {
        self.is_flag = is_flag;
        self
    }

    /// Set the short flag (e.g., `'n'` for `-n`).
    pub fn set_short(&mut self, short: char) -> &mut Self {
        self.short = Some(short);
        self
    }

    /// Set the long flag (e.g., `"name"` for `--name`).
    pub fn set_long(&mut self, long: &'a str) -> &mut Self {
        self.long = Some(long);
        self
    }

    /// Set aliases for the tag.
    pub fn set_alias(&mut self, alias: Vec<&'a str>) -> &mut Self {
        self.alias = Some(alias);
        self
    }

    /// Set whether this tag is positional.
    pub fn set_positional(&mut self, positional: bool) -> &mut Self {
        self.positional = positional;
        self
    }

    /// Set whether this tag is optional.
    pub fn set_optional(&mut self, optional: bool) -> &mut Self {
        self.optional = optional;
        self
    }

    /// Set whether this tag accepts multiple values.
    pub fn set_multi(&mut self, multi: bool) -> &mut Self {
        self.multi = multi;
        self
    }

    /// Set whether this tag participates in parsing after a `--` separator.
    pub fn set_is_flag(&mut self, is_flag: bool) -> &mut Self {
        self.is_flag = is_flag;
        self
    }
}

impl<'a> Default for PickerArgInfo<'a> {
    fn default() -> Self {
        Self::new()
    }
}
