use crate::Pickable;

/// Represents the result of parsing or looking up a value.
///
/// This enum is generic over the type being parsed. It models four possible outcomes:
/// - [`Unparsed`](PickerResult::Unparsed): The value has not yet been parsed (default).
/// - [`Parsed`](PickerResult::Parsed): The value was successfully parsed into `Type`.
/// - [`NotFound`](PickerResult::NotFound): The requested value could not be found.
/// - [`FormatError`](PickerResult::FormatError): The input could not be parsed due to a format error.
#[derive(Default)]
pub enum PickerResult<Type>
where
    Type: Default + Pickable,
{
    /// The value has not yet been parsed (default).
    #[default]
    Unparsed,

    /// The value was successfully parsed into `Type`.
    Parsed(Type),

    /// The requested value could not be found.
    NotFound,

    /// The input could not be parsed due to a format error.
    FormatError,
}

impl<Type, E> From<Result<Type, E>> for PickerResult<Type>
where
    Type: Default + Pickable,
{
    /// Converts a `Result<Type, E>` into a `PickerResult<Type>`.
    ///
    /// - `Ok(value)` maps to [`Parsed(value)`](PickerResult::Parsed).
    /// - `Err(_)` maps to [`FormatError`](PickerResult::FormatError).
    fn from(result: Result<Type, E>) -> Self {
        match result {
            Ok(value) => PickerResult::Parsed(value),
            Err(_) => PickerResult::FormatError,
        }
    }
}

impl<Type> From<Option<Type>> for PickerResult<Type>
where
    Type: Default + Pickable,
{
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
