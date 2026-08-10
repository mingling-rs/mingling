use crate::{
    matcher_needed::*,
    parselib::{build_possible_flags, seek_end_of_options},
};

/// `ArgMatcher` is used for parameters that carry a single value.
///
/// It handles two scenarios:
///
/// **Named** — `--name Alice` or `--name=Alice`.
///   Each flag occurrence consumes **one** following argument as its value,
///   regardless of what it is (even if it looks like a flag).
///   This ensures the mask correctly claims the value slot; validation is
///   the `Pickable`'s responsibility.
///
/// **Positional** — no flag prefix, matched by position.
///
/// # Examples
///
/// | Input | `on_match_one` | `on_match_all` |
/// |-------|----------------|----------------|
/// | `--name Alice` | `[0, 1]` (via Pickable tag) | `[0, 1]` |
/// | `--name=Alice` | `[0]` | `[0]` |
/// | `--val a --val b` | `[0, 1]` | `[0, 1, 2, 3]` |
///
/// Args after `--` are ignored.
pub struct ArgMatcher;

impl ArgMatcher {
    /// Check whether `raw` matches `flag_str`, optionally with an inline value
    /// separated by the style's value separator (`=` for Unix, `:` for PowerShell).
    #[inline]
    fn matches(raw: &str, flag_str: &str, case_sensitive: bool, sep: char) -> bool {
        let eq_match =
            |r: &str, f: &str| r.len() > f.len() && r.as_bytes().get(f.len()) == Some(&(sep as u8));

        if case_sensitive {
            raw == flag_str || (raw.starts_with(flag_str) && eq_match(raw, flag_str))
        } else {
            raw.eq_ignore_ascii_case(flag_str)
                || (raw.len() > flag_str.len()
                    && raw[..flag_str.len()].eq_ignore_ascii_case(flag_str)
                    && raw.as_bytes()[flag_str.len()] == sep as u8)
        }
    }

    /// Check whether the argument contains its value inline via the style's
    /// value separator (eq mode), so no extra mask slot is needed.
    #[inline]
    fn is_inline_value(raw: &str, flag_str: &str, sep: char) -> bool {
        raw.len() > flag_str.len() && raw.as_bytes().get(flag_str.len()) == Some(&(sep as u8))
    }
}

impl Matcher for ArgMatcher {
    fn on_match_one(
        args: &[MaskedArg],
        style: &ParserStyle,
        arg_info: &PickerArgInfo,
    ) -> Option<usize> {
        if arg_info.positional {
            return args.first().map(|a| a.raw_idx);
        }

        let possible_flags = build_possible_flags(style, arg_info);
        let end = seek_end_of_options(args, style);
        let sep = style.value_separator;

        for arg in args {
            if end.is_some_and(|e| arg.raw_idx >= e) {
                break;
            }

            let matched = possible_flags
                .iter()
                .any(|f| Self::matches(arg.raw, f, style.case_sensitive, sep));
            if matched {
                return Some(arg.raw_idx);
            }
        }

        None
    }

    fn on_match_all(
        args: &[MaskedArg],
        style: &ParserStyle,
        arg_info: &PickerArgInfo,
    ) -> Vec<usize> {
        if arg_info.positional {
            let end = seek_end_of_options(args, style);
            return args
                .iter()
                .take_while(|a| end.is_none_or(|e| a.raw_idx < e))
                .map(|a| a.raw_idx)
                .collect();
        }

        let possible_flags = build_possible_flags(style, arg_info);
        let end = seek_end_of_options(args, style);
        let sep = style.value_separator;

        let mut result = Vec::new();
        let mut i = 0;
        while i < args.len() {
            if end.is_some_and(|e| args[i].raw_idx >= e) {
                break;
            }

            let matched = possible_flags
                .iter()
                .any(|f| Self::matches(args[i].raw, f, style.case_sensitive, sep));

            if matched {
                let flag_str = possible_flags
                    .iter()
                    .find(|f| Self::matches(args[i].raw, f, style.case_sensitive, sep))
                    .expect("already matched");

                result.push(args[i].raw_idx);

                if !Self::is_inline_value(args[i].raw, flag_str, sep) {
                    if i + 1 < args.len()
                        // Don't consume `--` (end-of-options marker) as a value.
                        && end.is_none_or(|e| args[i + 1].raw_idx < e)
                    {
                        result.push(args[i + 1].raw_idx);
                        i += 2;
                        continue;
                    }
                    i += 1;
                    continue;
                }
                i += 1;
                continue;
            }
            i += 1;
        }

        result
    }
}
