use crate::{
    Pickable, PickerArg, PickerArgAttr, PickerArgResult, SinglePickable, TagPhaseContext,
    matcher_needed::Matcher,
    parselib::{MultiArgMatcher, ParserStyle},
};

/// Boundary check for multi-value positional parameters.
///
/// Determines whether a raw string marks the end of a multi-value
/// parameter's input range.
pub trait BoundaryCheck {
    /// Returns `true` if `raw` indicates a boundary (i.e., the start of
    /// a new parameter), stopping greedy collection.
    fn check_boundary(raw: &str) -> bool;
}

/// Trait for multi-value parameters.
///
/// Implementors define how a sequence of raw strings is converted into
/// a single value, with an associated [`BoundaryCheck`] to control where
/// collection stops.
pub trait MultiPickableWithBoundary: Sized {
    /// The boundary checker type that determines when to stop consuming
    /// positional arguments.
    type Checker: BoundaryCheck;

    /// Parse and collect multiple raw string values into `Self`.
    ///
    /// The caller should stop passing additional items once the
    /// associated [`Checker`](Self::Checker) signals a boundary.
    fn pick_multi(raw: Vec<String>) -> PickerArgResult<Self>;
}

/// Marker: unit type that always accepts — no boundary.
pub struct NoBoundary;

impl BoundaryCheck for NoBoundary {
    #[inline]
    fn check_boundary(_raw: &str) -> bool {
        false
    }
}

/// `Vec<T>` is greedy — it takes everything with `NoBoundary`.
impl<T: SinglePickable> MultiPickableWithBoundary for Vec<T> {
    type Checker = NoBoundary;

    fn pick_multi(raw: Vec<String>) -> PickerArgResult<Self> {
        let mut result = Self::with_capacity(raw.len());
        for s in &raw {
            match T::pick_single(Some(s)) {
                PickerArgResult::Parsed(v) => result.push(v),
                PickerArgResult::NotFound => return PickerArgResult::NotFound,
                PickerArgResult::Unparsed => {}
            }
        }
        PickerArgResult::Parsed(result)
    }
}

/// If the first raw string looks like a named flag (starts with the
/// style's long or short prefix), strip it — it's the flag, not a value.
fn strip_flag<'a>(raw_strs: &'a [&'a str]) -> &'a [&'a str] {
    if let Some(first) = raw_strs.first() {
        let style = ParserStyle::global_style();
        if first.starts_with(style.long_prefix) || first.starts_with(style.short_prefix) {
            return &raw_strs[1..];
        }
    }
    raw_strs
}

// Pickable impl for Vec<T>

impl<'a, T> Pickable<'a> for Vec<T>
where
    T: SinglePickable,
{
    fn get_attr(flag: &'a PickerArg<'a, Self>) -> PickerArgAttr {
        PickerArgAttr::positional_or_multi(flag)
    }

    fn tag(ctx: TagPhaseContext) -> Vec<usize> {
        MultiArgMatcher::match_all(ctx.into())
    }

    fn pick(raw_strs: &[&str]) -> PickerArgResult<Self> {
        let strs = strip_flag(raw_strs);
        let owned: Vec<String> = strs.iter().map(|&s| s.to_string()).collect();
        <Self as MultiPickableWithBoundary>::pick_multi(owned)
    }
}
