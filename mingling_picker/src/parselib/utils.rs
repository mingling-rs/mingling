use crate::{
    PickerArgInfo,
    parselib::{MaskedArg, ParserStyle},
};

#[inline(always)]
pub fn build_possible_flags(style: &ParserStyle, arg_info: &PickerArgInfo) -> Vec<String> {
    let mut possible_flags = vec![];

    if let Some(short) = arg_info.short {
        possible_flags.push(style.flag_string(short));
    }

    if let Some(long) = arg_info.long {
        possible_flags.push(style.flag_string(long));
    }

    if let Some(aliases) = &arg_info.alias {
        for alias in aliases {
            possible_flags.push(style.flag_string(*alias));
        }
    }

    possible_flags
}

/// Seeks the index of the end-of-options marker (`--`) in the argument list.
///
/// This function searches for the standard end-of-options separator (`--`)
/// in the given argument list, respecting the parser's style settings
/// (e.g., case sensitivity). The end-of-options marker indicates that all
/// subsequent arguments should be treated as positional arguments, not flags.
#[must_use]
pub fn seek_end_of_options(args: &[MaskedArg], style: &ParserStyle) -> Option<usize> {
    get_seeked_first(seek_eq(args, style.end_of_options, style.case_sensitive))
}

/// Seeks arguments in `args` that are exactly equal to the given `string`.
///
/// Returns the indices of matching arguments.
#[must_use]
#[inline(always)]
pub fn seek_eq(args: &[MaskedArg], string: &str, case_sensitive: bool) -> Vec<usize> {
    args.iter()
        .filter(|arg| {
            if case_sensitive {
                arg.raw == string
            } else {
                arg.raw.eq_ignore_ascii_case(string)
            }
        })
        .map(|arg| arg.raw_idx)
        .collect()
}

/// Seeks arguments in `args` that contain the given `string` as a substring.
///
/// Returns the indices of matching arguments.
#[must_use]
#[inline(always)]
pub fn seek_contains(args: &[MaskedArg], string: &str, case_sensitive: bool) -> Vec<usize> {
    args.iter()
        .filter(|arg| {
            if case_sensitive {
                arg.raw.contains(string)
            } else {
                arg.raw.to_lowercase().contains(&string.to_lowercase())
            }
        })
        .map(|arg| arg.raw_idx)
        .collect()
}

/// Seeks arguments in `args` that start with the given `string`.
///
/// Returns the indices of matching arguments.
#[must_use]
#[inline(always)]
pub fn seek_start_with(args: &[MaskedArg], string: &str, case_sensitive: bool) -> Vec<usize> {
    args.iter()
        .filter(|arg| {
            if case_sensitive {
                arg.raw.starts_with(string)
            } else {
                arg.raw.to_lowercase().starts_with(&string.to_lowercase())
            }
        })
        .map(|arg| arg.raw_idx)
        .collect()
}

/// Seeks arguments in `args` that end with the given `string`.
///
/// Returns the indices of matching arguments.
#[must_use]
#[inline(always)]
pub fn seek_end_with(args: &[MaskedArg], string: &str, case_sensitive: bool) -> Vec<usize> {
    args.iter()
        .filter(|arg| {
            if case_sensitive {
                arg.raw.ends_with(string)
            } else {
                arg.raw.to_lowercase().ends_with(&string.to_lowercase())
            }
        })
        .map(|arg| arg.raw_idx)
        .collect()
}

/// Seeks arguments in `args` that are exactly equal to any of the given `strings`.
///
/// Returns the indices of matching arguments.
#[must_use]
#[inline(always)]
pub fn multi_seek_eq(args: &[MaskedArg], strings: &[&str], case_sensitive: bool) -> Vec<usize> {
    args.iter()
        .filter(|arg| {
            if case_sensitive {
                strings.contains(&arg.raw)
            } else {
                strings.iter().any(|s| arg.raw.eq_ignore_ascii_case(s))
            }
        })
        .map(|arg| arg.raw_idx)
        .collect()
}

/// Seeks arguments in `args` that contain any of the given `strings` as a substring.
///
/// Returns the indices of matching arguments.
#[must_use]
#[inline(always)]
pub fn multi_seek_contains(
    args: &[MaskedArg],
    strings: &[&str],
    case_sensitive: bool,
) -> Vec<usize> {
    args.iter()
        .filter(|arg| {
            if case_sensitive {
                strings.iter().any(|s| arg.raw.contains(s))
            } else {
                let lower_raw = arg.raw.to_lowercase();
                strings
                    .iter()
                    .any(|s| lower_raw.contains(&s.to_lowercase()))
            }
        })
        .map(|arg| arg.raw_idx)
        .collect()
}

/// Seeks arguments in `args` that start with any of the given `strings`.
///
/// Returns the indices of matching arguments.
#[must_use]
#[inline(always)]
pub fn multi_seek_start_with(
    args: &[MaskedArg],
    strings: &[&str],
    case_sensitive: bool,
) -> Vec<usize> {
    args.iter()
        .filter(|arg| {
            if case_sensitive {
                strings.iter().any(|s| arg.raw.starts_with(s))
            } else {
                let lower_raw = arg.raw.to_lowercase();
                strings
                    .iter()
                    .any(|s| lower_raw.starts_with(&s.to_lowercase()))
            }
        })
        .map(|arg| arg.raw_idx)
        .collect()
}

/// Seeks arguments in `args` that end with any of the given `strings`.
///
/// Returns the indices of matching arguments.
#[must_use]
#[inline(always)]
pub fn multi_seek_end_with(
    args: &[MaskedArg],
    strings: &[&str],
    case_sensitive: bool,
) -> Vec<usize> {
    args.iter()
        .filter(|arg| {
            if case_sensitive {
                strings.iter().any(|s| arg.raw.ends_with(s))
            } else {
                let lower_raw = arg.raw.to_lowercase();
                strings
                    .iter()
                    .any(|s| lower_raw.ends_with(&s.to_lowercase()))
            }
        })
        .map(|arg| arg.raw_idx)
        .collect()
}

/// Converts a `&Vec<String>` into a `Vec<&str>` by borrowing each string's slice.
///
/// This is useful for converting owned `String` vectors into borrowed `&str` slices
/// for functions that take `&[&str]` or similar parameters.
#[must_use]
#[inline(always)]
#[doc(hidden)]
pub fn vec_string_to_vec_str(input: &[String]) -> Vec<&str> {
    input.iter().map(|s| s.as_str()).collect()
}

/// Converts a `&Vec<String>` into a `Vec<&str>` by borrowing each string's slice.
///
/// This is useful for converting owned `String` vectors into borrowed `&str` slices
/// for functions that take `&[&str]` or similar parameters.
#[macro_export]
macro_rules! vec_string_slice {
    ($v:expr) => {
        $v.iter()
            .map(|s| s.as_str())
            .collect::<Vec<&str>>()
            .as_slice()
    };
}

/// Gets the first element from a vector of seek results, if any.
///
/// Returns `Some(index)` if the vector is non-empty, otherwise `None`.
#[must_use]
#[inline(always)]
pub fn get_seeked_first(seeked: Vec<usize>) -> Option<usize> {
    seeked.into_iter().next()
}
