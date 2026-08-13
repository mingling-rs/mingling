// Doc Not Optimize
use crate::{matcher_needed::*, parselib::seek_end_of_options};

/// `PositionalMatcher` matches positional arguments — values not associated
/// with any named flag.
///
/// # Rules
///
/// * Before `--`: skips any argument that starts with the style's long or short
///   prefix (those belong to named matchers).
/// * After `--`: takes **everything** — the `--` marker signals that all
///   remaining values are positional, even if they look like flags.
/// * Runs at the lowest priority (see [`PickerArgAttr::Positional`](crate::PickerArgAttr::Positional)).
pub struct PositionalMatcher;

impl PositionalMatcher {
    /// Check whether `raw` looks like a named flag (starts with a prefix).
    #[inline]
    fn is_flag_like(raw: &str, style: &ParserStyle) -> bool {
        raw.starts_with(style.long_prefix) || raw.starts_with(style.short_prefix)
    }
}

impl Matcher for PositionalMatcher {
    fn on_match_one(
        args: &[MaskedArg],
        style: &ParserStyle,
        _arg_info: &PickerArgInfo,
    ) -> Option<usize> {
        let end = seek_end_of_options(args, style);

        for arg in args {
            if end.is_some_and(|e| arg.raw_idx == e) {
                // Hit `--`: everything from here on is positional,
                // including the first arg after `--`.
                continue;
            }
            if end.is_some_and(|e| arg.raw_idx > e) {
                // After `--`: accept everything.
                return Some(arg.raw_idx);
            }
            // Before `--`: skip flag-like args.
            if !Self::is_flag_like(arg.raw, style) {
                return Some(arg.raw_idx);
            }
        }

        None
    }

    fn on_match_all(
        args: &[MaskedArg],
        style: &ParserStyle,
        _arg_info: &PickerArgInfo,
    ) -> Vec<usize> {
        let end = seek_end_of_options(args, style);
        let mut after_end = false;
        let mut result = Vec::new();

        for arg in args {
            if end.is_some_and(|e| arg.raw_idx == e) {
                after_end = true;
                continue;
            }
            if after_end || !Self::is_flag_like(arg.raw, style) {
                result.push(arg.raw_idx);
            }
        }

        result
    }
}
