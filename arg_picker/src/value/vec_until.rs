use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use crate::{
    BoundaryCheck, MultiPickableWithBoundary, Pickable, PickerArg, PickerArgAttr, PickerArgResult,
    SinglePickable, TagPhaseContext,
    matcher_needed::Matcher,
    parselib::{MultiArgMatcher, ParserStyle},
};

/// A `Vec`-like container that stops collecting when [`BoundaryCheck`]
/// returns `true`.
///
/// This type exists to signal "I know what I'm doing with boundaries"
/// at the type level (as opposed to `Vec<T>` which greedily takes
/// everything).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VecUntil<T> {
    pub(crate) inner: Vec<T>,
    _marker: PhantomData<T>,
}

impl<T> VecUntil<T> {
    /// Consumes `self` and returns the underlying [`Vec<T>`].
    #[must_use]
    pub fn into_inner(self) -> Vec<T> {
        self.inner
    }
}

impl<T> From<Vec<T>> for VecUntil<T> {
    fn from(v: Vec<T>) -> Self {
        Self {
            inner: v,
            _marker: PhantomData,
        }
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

impl<T> DerefMut for VecUntil<T> {
    fn deref_mut(&mut self) -> &mut Vec<T> {
        &mut self.inner
    }
}

// MultiPickableWithBoundary impl

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
        PickerArgResult::Parsed(Self {
            inner,
            _marker: PhantomData,
        })
    }
}

// Pickable impl

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

        let start = usize::from(!is_positional);
        if start >= positions.len() {
            return positions;
        }

        let mut cut = start;
        for &idx in &positions[start..] {
            if let Some(raw) = args.get(idx)
                && T::check_boundary(raw)
            {
                break;
            }
            cut += 1;
        }

        positions[..cut].to_vec()
    }

    fn pick(raw_strs: &[&str]) -> PickerArgResult<Self> {
        let strs = strip_flag(raw_strs);
        let owned: Vec<String> = strs.iter().map(|&s| s.to_string()).collect();
        <Self as MultiPickableWithBoundary>::pick_multi(owned)
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
