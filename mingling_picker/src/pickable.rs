use crate::{PickerFlag, PickerResult, PickerTag};

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
/// # use mingling_picker::{Pickable, PickerResult, PickerFlag, PickerTag};
/// #[derive(Default)]
/// struct MyType(String);
///
/// impl<'a> Pickable<'a> for MyType {
///     fn tag(flag: &'a PickerFlag<'a, Self>) -> PickerTag<'a> {
///         let tag = PickerTag::from(flag);
///     }
///
///     fn pick(raw_strs: &[&str]) -> PickerResult<Self> {
///         PickerResult::Parsed(MyType(raw_strs.join(" ")))
///     }
/// }
/// ```
pub trait Pickable<'a>
where
    Self: Sized + Default,
{
    /// Given a [`PickerFlag`], returns a [`PickerTag`] that tells the parser
    /// about the argument's characteristics (e.g., positional, optional, multi).
    fn tag(flag: &'a PickerFlag<'a, Self>) -> PickerTag<'a>;

    /// Parses a `Self` value from the given raw string input.
    fn pick(raw_strs: &[&str]) -> PickerResult<Self>;
}
