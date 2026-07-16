use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use crate::{
    matcher_needed::Matcher,
    parselib::{MultiArgMatcher, ParserStyle},
    Pickable, PickerArg, PickerArgAttr, PickerArgResult,
    SinglePickable, TagPhaseContext,
};

/// Boundary check for multi-value positional parameters.
pub trait BoundaryCheck {
    /// Returns `true` if `raw` is **not** part of the current parameter group.
    fn check_boundary(raw: &str) -> bool;
}

// ============================================================
// MultiPickableWithBoundary — the single multi-value trait
// ============================================================

/// Trait for multi-value parameters.
///
/// - [`pick_multi`](MultiPickableWithBoundary::pick_multi): converts collected
///   strings into `Self`.
/// - [`Checker`](MultiPickableWithBoundary::Checker): a type implementing
///   [`BoundaryCheck`] used by the positional matcher to determine where one
///   group ends and another begins.
pub trait MultiPickableWithBoundary: Sized {
    /// The boundary checker type.
    type Checker: BoundaryCheck;

    /// Convert the collected raw strings into `Self`.
    fn pick_multi(raw: Vec<String>) -> PickerArgResult<Self>;
}

/// Marker: unit type that always accepts — no boundary.
pub struct NoBoundary;

impl BoundaryCheck for NoBoundary {
    #[inline(always)]
    fn check_boundary(_raw: &str) -> bool {
        false // never stop: greedy
    }
}

/// `Vec<T>` is greedy — it takes everything with `NoBoundary`.
impl<T: SinglePickable> MultiPickableWithBoundary for Vec<T> {
    type Checker = NoBoundary;

    fn pick_multi(raw: Vec<String>) -> PickerArgResult<Self> {
        let mut result = Vec::with_capacity(raw.len());
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

/// `VecUntil<T>` uses `T` itself as the boundary checker.
impl<T> MultiPickableWithBoundary for VecUntil<T>
where
    T: SinglePickable + BoundaryCheck,
{
    type Checker = T;

    fn pick_multi(raw: Vec<String>) -> PickerArgResult<Self> {
        let mut inner = Vec::with_capacity(raw.len());
        for s in &raw {
            match T::pick_single(Some(s)) {
                PickerArgResult::Parsed(v) => inner.push(v),
                PickerArgResult::NotFound => return PickerArgResult::NotFound,
                PickerArgResult::Unparsed => {}
            }
        }
        PickerArgResult::Parsed(VecUntil { inner, _marker: PhantomData })
    }
}

// ============================================================
// VecUntil<T> — Vec wrapper with boundary checking
// ============================================================

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VecUntil<T> {
    inner: Vec<T>,
    _marker: PhantomData<T>,
}

impl<T> VecUntil<T> {
    pub fn into_inner(self) -> Vec<T> {
        self.inner
    }
}

impl<T> From<Vec<T>> for VecUntil<T> {
    fn from(v: Vec<T>) -> Self {
        VecUntil { inner: v, _marker: PhantomData }
    }
}

impl<T> From<VecUntil<T>> for Vec<T> {
    fn from(v: VecUntil<T>) -> Self {
        v.inner
    }
}

impl<T> Deref for VecUntil<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Vec<T> {
        &self.inner
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

impl<T> DerefMut for VecUntil<T> {
    fn deref_mut(&mut self) -> &mut Vec<T> {
        &mut self.inner
    }
}

// Concrete Pickable impls

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
        <Vec<T> as MultiPickableWithBoundary>::pick_multi(owned)
    }
}

impl<'a, T> Pickable<'a> for VecUntil<T>
where
    T: SinglePickable + BoundaryCheck,
{
    fn get_attr(flag: &'a PickerArg<'a, Self>) -> PickerArgAttr {
        PickerArgAttr::positional_or_multi(flag)
    }

    fn tag(ctx: TagPhaseContext) -> Vec<usize> {
        let args = ctx.args;
        let is_positional = ctx.arg_info.positional;
        let positions = MultiArgMatcher::match_all(ctx.into());
        if positions.is_empty() {
            return positions;
        }

        // Apply BoundaryCheck to trim positions that don't belong.
        // For named: positions[0] is the flag, values start at [1..].
        // For positional: all positions are values.
        let start = if is_positional { 0 } else { 1 };
        if start >= positions.len() {
            return positions; // flag only, no values
        }

        let mut cut = start;
        for &idx in &positions[start..] {
            if let Some(raw) = args.get(idx) {
                if T::check_boundary(raw) {
                    break; // boundary hit — stop here
                }
            }
            cut += 1;
        }

        positions[..cut].to_vec()
    }

    fn pick(raw_strs: &[&str]) -> PickerArgResult<Self> {
        // Strip the flag position from named args.
        let strs = strip_flag(raw_strs);
        let owned: Vec<String> = strs.iter().map(|&s| s.to_string()).collect();
        <VecUntil<T> as MultiPickableWithBoundary>::pick_multi(owned)
    }
}
