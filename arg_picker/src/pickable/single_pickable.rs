// Doc Not Optimize
use crate::{Pickable, PickerArg, PickerArgAttr, PickerArgResult, TagPhaseContext};

/// `SinglePickable` trait defines how to parse a type from a single command-line argument.
///
/// This trait provides a simplified interface for types that consume exactly one argument value.
/// It is automatically implemented by the blanket `impl` of [`Pickable`], so types implementing
/// `SinglePickable` will work with the full `Pickable` argument parsing system.
///
/// Additionally, `Option<S>` where `S: SinglePickable` also implements [`Pickable`], allowing
/// optional arguments to be parsed naturally.
///
/// # Type Parameters
///
/// * `Self` - The type to be parsed from a single argument string.
pub trait SinglePickable
where
    Self: Sized,
{
    /// Parse a single optional string value into an instance of `Self`.
    ///
    /// # Parameters
    ///
    /// * `str` - An `Option<&str>` representing the raw argument value. If `None`,
    ///   it indicates that no argument value was provided (e.g., for flag-like arguments).
    ///
    /// # Returns
    ///
    /// Returns [`PickerArgResult<Self>`], i.e., the parsed `Self` instance on success,
    /// or an appropriate error message on failure.
    fn pick_single(str: Option<&str>) -> PickerArgResult<Self>;
}

impl<'a, S> Pickable<'a> for S
where
    S: SinglePickable,
{
    fn get_attr(flag: &'a PickerArg<'a, Self>) -> PickerArgAttr {
        PickerArgAttr::positional_or_single(flag)
    }

    fn tag(ctx: TagPhaseContext) -> Vec<usize> {
        crate::parselib::SingleMatcher::tag(ctx)
    }

    fn pick(raw_strs: &[&str]) -> PickerArgResult<Self> {
        Self::pick_single(crate::parselib::seek_single(raw_strs))
    }
}

impl<'a, S> Pickable<'a> for Option<S>
where
    S: SinglePickable,
{
    fn get_attr(flag: &'a PickerArg<'a, Self>) -> PickerArgAttr {
        PickerArgAttr::positional_or_single(flag)
    }

    fn tag(ctx: TagPhaseContext) -> Vec<usize> {
        crate::parselib::SingleMatcher::tag(ctx)
    }

    fn pick(raw_strs: &[&str]) -> PickerArgResult<Self> {
        match S::pick(raw_strs) {
            PickerArgResult::Unparsed => PickerArgResult::Unparsed,
            PickerArgResult::Parsed(r) => PickerArgResult::Parsed(Some(r)),
            PickerArgResult::NotFound => PickerArgResult::Parsed(None),
        }
    }
}
