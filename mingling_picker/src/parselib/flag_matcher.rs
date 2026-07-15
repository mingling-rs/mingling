use crate::{
    matcher_needed::*,
    parselib::{build_possible_flags, get_seeked_first, multi_seek_eq, seek_end_of_options},
    vec_string_slice,
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
        let end_of_options = seek_end_of_options(args, style);
        let result = get_seeked_first(multi_seek_eq(
            args,
            vec_string_slice!(possible_flags),
            style.case_sensitive,
        ));

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
        let end_of_options = seek_end_of_options(args, style);
        let result = multi_seek_eq(
            args,
            vec_string_slice!(possible_flags),
            style.case_sensitive,
        );

        match end_of_options {
            Some(end) => result.into_iter().filter(|&idx| idx <= end).collect(),
            None => result,
        }
    }
}
