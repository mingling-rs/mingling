/// Represents the result of parsing or looking up a value.
///
/// This enum is generic over the type being parsed. It models four possible outcomes:
/// - [`Unparsed`](PickerResult::Unparsed): The value has not yet been parsed (default).
/// - [`Parsed`](PickerResult::Parsed): The value was successfully parsed into `Type`.
/// - [`NotFound`](PickerResult::NotFound): The requested value could not be found.
/// - [`ParseError`](PickerResult::ParseError): The input could not be parsed due to a format error.
#[derive(Default)]
pub enum PickerResult<Type> {
    /// The value has not yet been parsed (default).
    #[default]
    Unparsed,

    /// The value was successfully parsed into `Type`.
    Parsed(Type),

    /// The requested value could not be found.
    NotFound,

    /// The input could not be parsed due to a format error.
    ParseError,
}

impl<Type, E> From<Result<Type, E>> for PickerResult<Type> {
    /// Converts a `Result<Type, E>` into a `PickerResult<Type>`.
    ///
    /// - `Ok(value)` maps to [`Parsed(value)`](PickerResult::Parsed).
    /// - `Err(_)` maps to [`ParseError`](PickerResult::ParseError).
    fn from(result: Result<Type, E>) -> Self {
        match result {
            Ok(value) => PickerResult::Parsed(value),
            Err(_) => PickerResult::ParseError,
        }
    }
}

impl<Type> From<Option<Type>> for PickerResult<Type> {
    /// Converts an `Option<Type>` into a `PickerResult<Type>`.
    ///
    /// - `Some(value)` maps to [`Parsed(value)`](PickerResult::Parsed).
    /// - `None` maps to [`NotFound`](PickerResult::NotFound).
    fn from(option: Option<Type>) -> Self {
        match option {
            Some(value) => PickerResult::Parsed(value),
            None => PickerResult::NotFound,
        }
    }
}

impl<Type> PickerResult<Type> {
    /// Returns `true` if the result is [`Parsed`](PickerResult::Parsed).
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_picker::PickerResult;
    ///
    /// let result: PickerResult<i32> = PickerResult::Parsed(42);
    /// assert!(result.is_parsed());
    ///
    /// let result: PickerResult<i32> = PickerResult::NotFound;
    /// assert!(!result.is_parsed());
    /// ```
    pub fn is_parsed(&self) -> bool {
        matches!(self, PickerResult::Parsed(_))
    }

    /// Returns `true` if the result is [`Parsed`](PickerResult::Parsed) or [`NotFound`](PickerResult::NotFound).
    /// i.e., the value exists (was either found or not yet parsed, but not a parse error).
    /// Typically indicates the value was "found" in some sense.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_picker::PickerResult;
    ///
    /// let result: PickerResult<i32> = PickerResult::Parsed(42);
    /// assert!(result.is_found());
    ///
    /// let result: PickerResult<i32> = PickerResult::NotFound;
    /// assert!(result.is_found());
    ///
    /// let result: PickerResult<i32> = PickerResult::ParseError;
    /// assert!(!result.is_found());
    /// ```
    pub fn is_found(&self) -> bool {
        matches!(self, PickerResult::Parsed(_) | PickerResult::NotFound)
    }

    /// Returns `true` if the result is [`ParseError`](PickerResult::ParseError).
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_picker::PickerResult;
    ///
    /// let result: PickerResult<i32> = PickerResult::ParseError;
    /// assert!(result.is_err());
    ///
    /// let result: PickerResult<i32> = PickerResult::Parsed(10);
    /// assert!(!result.is_err());
    /// ```
    pub fn is_err(&self) -> bool {
        matches!(self, PickerResult::ParseError)
    }

    /// Returns `Some(&Type)` if [`Parsed`](PickerResult::Parsed), otherwise `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_picker::PickerResult;
    ///
    /// let result: PickerResult<i32> = PickerResult::Parsed(42);
    /// assert_eq!(result.parsed(), Some(&42));
    ///
    /// let result: PickerResult<i32> = PickerResult::NotFound;
    /// assert_eq!(result.parsed(), None);
    /// ```
    pub fn parsed(&self) -> Option<&Type> {
        if let PickerResult::Parsed(value) = self {
            Some(value)
        } else {
            None
        }
    }

    /// Returns the contained [`Parsed`](PickerResult::Parsed) value or panics with a given message.
    ///
    /// # Panics
    /// Panics if the value is not [`Parsed`](PickerResult::Parsed), with a message including the provided `msg`.
    ///
    /// # Examples
    ///
    /// ```should_panic
    /// use mingling_picker::PickerResult;
    ///
    /// let result: PickerResult<i32> = PickerResult::NotFound;
    /// result.expect("expected a parsed value");
    /// ```
    pub fn expect(self, msg: &str) -> Type {
        match self {
            PickerResult::Parsed(value) => value,
            _ => panic!("{}", msg),
        }
    }

    /// Returns the contained [`Parsed`](PickerResult::Parsed) value or panics.
    ///
    /// # Panics
    /// Panics if the value is not [`Parsed`](PickerResult::Parsed).
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_picker::PickerResult;
    ///
    /// let result: PickerResult<i32> = PickerResult::Parsed(42);
    /// assert_eq!(result.unwrap(), 42);
    /// ```
    ///
    /// ```should_panic
    /// use mingling_picker::PickerResult;
    ///
    /// let result: PickerResult<i32> = PickerResult::NotFound;
    /// result.unwrap();
    /// ```
    pub fn unwrap(self) -> Type {
        match self {
            PickerResult::Parsed(value) => value,
            PickerResult::Unparsed => {
                panic!("called `PickerResult::unwrap()` on an `Unparsed` value")
            }
            PickerResult::NotFound => {
                panic!("called `PickerResult::unwrap()` on a `NotFound` value")
            }
            PickerResult::ParseError => {
                panic!("called `PickerResult::unwrap()` on a `ParseError` value")
            }
        }
    }

    /// Returns the contained [`Parsed`](PickerResult::Parsed) value or a provided `default`.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_picker::PickerResult;
    ///
    /// let result: PickerResult<i32> = PickerResult::Parsed(42);
    /// assert_eq!(result.unwrap_or(0), 42);
    ///
    /// let result: PickerResult<i32> = PickerResult::NotFound;
    /// assert_eq!(result.unwrap_or(0), 0);
    /// ```
    pub fn unwrap_or(self, default: Type) -> Type {
        match self {
            PickerResult::Parsed(value) => value,
            _ => default,
        }
    }

    /// Returns the contained [`Parsed`](PickerResult::Parsed) value or computes it from a closure.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_picker::PickerResult;
    ///
    /// let result: PickerResult<i32> = PickerResult::Parsed(42);
    /// assert_eq!(result.unwrap_or_else(|| 0), 42);
    ///
    /// let result: PickerResult<i32> = PickerResult::NotFound;
    /// assert_eq!(result.unwrap_or_else(|| 0), 0);
    /// ```
    pub fn unwrap_or_else<F: FnOnce() -> Type>(self, f: F) -> Type {
        match self {
            PickerResult::Parsed(value) => value,
            _ => f(),
        }
    }

    /// Returns the contained [`Parsed`](PickerResult::Parsed) value or the default value of `Type`.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_picker::PickerResult;
    ///
    /// let result: PickerResult<i32> = PickerResult::Parsed(42);
    /// assert_eq!(result.unwrap_or_default(), 42);
    ///
    /// let result: PickerResult<i32> = PickerResult::NotFound;
    /// assert_eq!(result.unwrap_or_default(), 0);
    /// ```
    pub fn unwrap_or_default(self) -> Type
    where
        Type: Default,
    {
        match self {
            PickerResult::Parsed(value) => value,
            _ => Type::default(),
        }
    }
}
