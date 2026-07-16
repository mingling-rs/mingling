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
        let converted = style.naming_case.convert(long.to_string());
        possible_flags.push(style.flag_string(&converted));
    }

    if let Some(aliases) = &arg_info.alias {
        for alias in aliases {
            let converted = style.naming_case.convert(alias.to_string());
            possible_flags.push(style.flag_string(&converted));
        }
    }

    possible_flags
}

/// Extract a single value from the raw strings tagged by [`SingleMatcher`].
///
/// Returns `None` if no value is available (empty slice),
/// the inline value after the style separator if present (eq mode),
/// or the value directly (positional or flag-following).
///
/// This is the standard `pick` helper for all `Single`-type
/// [`Pickable`](crate::Pickable) implementations.
#[must_use]
pub fn seek_single<'a>(raw_strs: &'a [&'a str]) -> Option<&'a str> {
    match raw_strs.len() {
        0 => None,
        1 => {
            let s = raw_strs[0];
            let sep = ParserStyle::global_style().value_separator;
            if let Some(pos) = s.rfind(sep) {
                Some(&s[pos + 1..])
            } else {
                Some(s)
            }
        }
        _ => Some(raw_strs[1]),
    }
}

/// Seeks the index of the end-of-options marker (`--`) in the argument list.
///
/// This function searches for the standard end-of-options separator (`--`)
/// in the given argument list, respecting the parser's style settings
/// (e.g., case sensitivity). The end-of-options marker indicates that all
/// subsequent arguments should be treated as positional arguments, not flags.
#[must_use]
pub fn seek_end_of_options(args: &[MaskedArg], style: &ParserStyle) -> Option<usize> {
    args.iter()
        .find(|arg| {
            if style.case_sensitive {
                arg.raw == style.end_of_options
            } else {
                arg.raw.eq_ignore_ascii_case(style.end_of_options)
            }
        })
        .map(|arg| arg.raw_idx)
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
