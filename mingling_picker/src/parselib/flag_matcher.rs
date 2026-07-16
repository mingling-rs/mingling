use crate::{
    matcher_needed::*,
    parselib::{build_possible_flags, get_seeked_first, multi_seek_eq},
};

/// `FlagMatcher` is used to match flags in command-line arguments.
///
/// Flags typically start with `-` or `--` (e.g., `-h`, `--help`),
/// and do not carry additional values. This matcher is responsible for finding
/// these flags in the argument list, taking into account that flags after `--`
/// (end-of-options marker) should not be matched.
pub struct FlagMatcher;

impl Matcher for FlagMatcher {
    fn on_match_one(
        args: &[MaskedArg],
        style: &ParserStyle,
        arg_info: &PickerArgInfo,
    ) -> Option<usize> {
        let possible_flags = build_possible_flags(style, arg_info);
        let flag_refs: Vec<&str> = possible_flags.iter().map(|s| s.as_str()).collect();
        let end_of_options = seek_end_of_options(args, style);

        let result = get_seeked_first(multi_seek_eq(args, &flag_refs, style.case_sensitive));

        match (end_of_options, result) {
            (Some(end), Some(current)) if current > end => None,
            _ => result,
        }
    }

    fn on_match_all(
        args: &[MaskedArg],
        style: &ParserStyle,
        arg_info: &PickerArgInfo,
    ) -> Vec<usize> {
        let possible_flags = build_possible_flags(style, arg_info);
        single_pass_match_all(args, style, &possible_flags)
    }
}

/// Single-pass match: finds the `--` marker and matching flags in one iteration.
fn single_pass_match_all(
    args: &[MaskedArg],
    style: &ParserStyle,
    possible_flags: &[String],
) -> Vec<usize> {
    let flag_refs: Vec<&str> = possible_flags.iter().map(|s| s.as_str()).collect();
    let eoo = style.end_of_options;
    let case_sensitive = style.case_sensitive;

    let mut end_pos: Option<usize> = None;
    let mut matches: Vec<usize> = Vec::new();

    for arg in args {
        if end_pos.is_none() {
            let is_eoo = if case_sensitive {
                arg.raw == eoo
            } else {
                arg.raw.eq_ignore_ascii_case(eoo)
            };
            if is_eoo {
                end_pos = Some(arg.raw_idx);
                continue;
            }
        }

        // Only match flags before the end-of-options marker.
        if end_pos.is_none() {
            let matched = if case_sensitive {
                flag_refs.contains(&arg.raw)
            } else {
                flag_refs.iter().any(|s| arg.raw.eq_ignore_ascii_case(s))
            };
            if matched {
                matches.push(arg.raw_idx);
            }
        }
    }

    matches
}

/// Locate the end-of-options marker (`--`) in the argument list.
fn seek_end_of_options(args: &[MaskedArg], style: &ParserStyle) -> Option<usize> {
    get_seeked_first(
        args.iter()
            .filter(|arg| {
                if style.case_sensitive {
                    arg.raw == style.end_of_options
                } else {
                    arg.raw.eq_ignore_ascii_case(style.end_of_options)
                }
            })
            .map(|arg| arg.raw_idx)
            .collect(),
    )
}
