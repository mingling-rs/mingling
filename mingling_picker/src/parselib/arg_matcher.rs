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
    /// Check whether `raw` matches `flag_str` (exact or eq-separated).
    #[inline(always)]
    fn matches(raw: &str, flag_str: &str, case_sensitive: bool) -> bool {
        if case_sensitive {
            raw == flag_str
                || raw.starts_with(flag_str) && raw.as_bytes().get(flag_str.len()) == Some(&b'=')
        } else {
            raw.eq_ignore_ascii_case(flag_str)
                || (raw.len() > flag_str.len()
                    && raw[..flag_str.len()].eq_ignore_ascii_case(flag_str)
                    && raw.as_bytes()[flag_str.len()] == b'=')
        }
    }

    /// Check whether the argument at the given position (in the masked slice)
    /// contains its value inline (eq mode), so no extra slot is needed.
    #[inline(always)]
    fn is_eq_mode(raw: &str, flag_str: &str) -> bool {
        raw.len() > flag_str.len() && raw.as_bytes().get(flag_str.len()) == Some(&b'=')
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

        for arg in args {
            if end.is_some_and(|e| arg.raw_idx >= e) {
                break;
            }

            let matched = possible_flags
                .iter()
                .any(|f| Self::matches(arg.raw, f, style.case_sensitive));
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

        let mut result = Vec::new();
        let mut i = 0;
        while i < args.len() {
            if end.is_some_and(|e| args[i].raw_idx >= e) {
                break;
            }

            let matched = possible_flags
                .iter()
                .any(|f| Self::matches(args[i].raw, f, style.case_sensitive));

            if matched {
                let flag_str = possible_flags
                    .iter()
                    .find(|f| Self::matches(args[i].raw, f, style.case_sensitive))
                    .expect("already matched");

                result.push(args[i].raw_idx);

                if !Self::is_eq_mode(args[i].raw, flag_str) {
                    if i + 1 < args.len() {
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
