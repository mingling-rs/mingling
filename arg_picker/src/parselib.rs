mod flag_matcher;
pub use flag_matcher::*;

mod arg_matcher;
pub use arg_matcher::*;

mod multi_arg_matcher;
pub use multi_arg_matcher::*;

mod pos_matcher;
pub use pos_matcher::*;

mod single_matcher;
pub use single_matcher::*;

mod style;
pub use style::*;

mod utils;
pub use utils::*;

use crate::{PickerArgInfo, PickerArgs};

/// Represents a single argument with its original raw string and index.
///
/// This is used during pattern matching to provide context about
/// which argument is being processed.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MaskedArg<'a> {
    /// The raw string value of the argument.
    pub raw: &'a str,
    /// The original index of the argument in the full argument list.
    pub raw_idx: usize,
}

/// Trait for defining matching logic against masked arguments.
///
/// Implementors can define custom strategies for matching one or all
/// arguments that pass through a mask filter.
pub trait Matcher {
    /// Called when only one match is needed.
    ///
    /// Returns the index of the first matched argument, or `None` if no match.
    fn on_match_one(
        args: &[MaskedArg],
        style: &ParserStyle,
        arg_info: &PickerArgInfo,
    ) -> Option<usize>;

    /// Called when all matches are needed.
    ///
    /// Returns a vector of indices of all matched arguments.
    fn on_match_all(
        args: &[MaskedArg],
        style: &ParserStyle,
        arg_info: &PickerArgInfo,
    ) -> Vec<usize>;

    /// Convenience method that builds masked arguments from `PickerArgs` and a mask,
    /// then calls `on_match_one`.
    fn match_one<'a>(ctx: MatcherContext<'a>) -> Option<usize> {
        let masked_args = build_masked_args(ctx.args, ctx.mask);
        Self::on_match_one(masked_args.as_slice(), ctx.style, ctx.arg_info)
    }

    /// Convenience method that builds masked arguments from `PickerArgs` and a mask,
    /// then calls `on_match_all`.
    fn match_all<'a>(ctx: MatcherContext<'a>) -> Vec<usize> {
        let masked_args = build_masked_args(ctx.args, ctx.mask);
        Self::on_match_all(masked_args.as_slice(), ctx.style, ctx.arg_info)
    }
}

/// Context for matcher operations
///
/// This struct bundles together the key pieces of data needed during matching:
/// - `args`: The full set of parsed arguments.
/// - `mask`: A byte mask indicating which arguments are currently active (non-zero = active).
/// - `style`: The parsing style configuration.
pub struct MatcherContext<'a> {
    /// The full set of parsed arguments.
    pub args: &'a PickerArgs<'a>,

    /// A byte mask where non-zero values indicate the argument at that position is active/should be matched.
    pub mask: &'a [u8],

    /// The parsing style configuration.
    pub style: &'a ParserStyle<'a>,

    /// Metadata about the command-line argument/flag being processed.
    ///
    /// Contains information such as short form (`-n`), long form (`--name`),
    /// aliases, and parsing flags (positional, optional, multi, is_flag).
    /// Used by matchers to make decisions based on argument characteristics.
    pub arg_info: &'a PickerArgInfo<'a>,
}

impl<'a> From<&'a crate::TagPhaseContext<'a>> for MatcherContext<'a> {
    fn from(ctx: &'a crate::TagPhaseContext<'a>) -> Self {
        MatcherContext {
            args: ctx.args,
            mask: ctx.mask,
            style: ParserStyle::global_style(),
            arg_info: ctx.arg_info,
        }
    }
}

impl<'a> From<crate::TagPhaseContext<'a>> for MatcherContext<'a> {
    fn from(ctx: crate::TagPhaseContext<'a>) -> Self {
        MatcherContext {
            args: ctx.args,
            mask: ctx.mask,
            style: ParserStyle::global_style(),
            arg_info: ctx.arg_info,
        }
    }
}

/// Checks whether the argument at index `idx` is already claimed (masked).
///
/// Returns `true` if `idx` is within the mask bounds and the mask value is non-zero,
/// indicating the argument has been claimed by a previous matcher.
///
/// # Arguments
///
/// * `mask` - A byte slice where non-zero values indicate claimed arguments.
/// * `idx` - The index to check in the mask.
#[inline(always)]
pub fn is_masked(mask: &[u8], idx: usize) -> bool {
    idx < mask.len() && mask[idx] != 0
}

/// Builds a vector of [`MaskedArg`] from the given `PickerArgs` and mask.
///
/// Only arguments whose mask entry is `0` (i.e., available/not yet claimed) are included.
/// Each resulting [`MaskedArg`] retains its original raw string and its index in the full
/// argument list for later reference.
///
/// # Arguments
///
/// * `args` - The full set of parsed arguments.
/// * `mask` - A byte slice where `0` means available and non-zero means already claimed.
#[inline(always)]
pub fn build_masked_args<'a>(args: &'a PickerArgs, mask: &'a [u8]) -> Vec<MaskedArg<'a>> {
    let mut cidx = 0;
    args.iter()
        .filter_map(|r| {
            let idx = cidx;
            cidx += 1;
            // Include args where mask is 0 (available/not yet claimed).
            // mask[i] = 0 means available; mask[i] != 0 means already claimed.
            if !is_masked(mask, idx) {
                Some(MaskedArg {
                    raw: r,
                    raw_idx: idx,
                })
            } else {
                None
            }
        })
        .collect()
}
