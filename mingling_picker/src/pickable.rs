use crate::PickerResult;

mod implements;

/// A trait for types that can be constructed from a raw string representation.
///
/// Implementing this trait allows a type to be "picked" or parsed from a string,
/// enabling deserialization or configuration loading from textual input.
///
/// # Requirements
///
/// - The implementing type must be [`Sized`] and implement [`Default`].
/// - The [`pick`] method performs the actual parsing and may fail.
///
/// # Errors
///
/// Returns a [`PickerResult`] which encapsulates either a successful parse
/// or an error indicating why the input could not be parsed.
///
/// # Examples
///
/// ```
/// # use mingling_picker::{Pickable, PickerResult};
/// #[derive(Default)]
/// struct MyType(String);
///
/// impl Pickable for MyType {
///     fn pick(raw_str: &str) -> PickerResult<Self> {
///         PickerResult::Parsed(MyType(raw_str.to_string()))
///     }
/// }
/// ```
pub trait Pickable
where
    Self: Sized + Default,
{
    /// Parses a `Self` value from the given raw string input.
    fn pick(raw_str: &str) -> PickerResult<Self>;
}
