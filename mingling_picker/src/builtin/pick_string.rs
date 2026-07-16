use crate::parselib::{ArgMatcher, Matcher, ParserStyle, PositionalMatcher};
use crate::pickable_needed::*;

impl<'a> Pickable<'a> for String {
    fn get_attr(flag: &'a PickerArg<'a, Self>) -> PickerArgAttr {
        PickerArgAttr::positional_or_single(flag)
    }

    fn tag(ctx: TagPhaseContext) -> Vec<usize> {
        if ctx.arg_info.positional {
            PositionalMatcher::match_one(ctx.into())
                .map(|i| vec![i])
                .unwrap_or_default()
        } else {
            ArgMatcher::match_all(ctx.into())
        }
    }

    fn pick(raw_strs: &[&str]) -> PickerArgResult<Self> {
        match raw_strs.len() {
            0 => PickerArgResult::NotFound,
            1 => {
                let s = raw_strs[0];
                let style = ParserStyle::global_style();

                // Inline value via style separator (e.g., --name=Alice, -Name:Alice).
                if let Some(pos) = s.rfind(style.value_separator) {
                    return PickerArgResult::Parsed(s[pos + 1..].to_string());
                }

                // If the single element looks like a named flag (starts with a prefix
                // such as `--`, `-`, or `/`), it's a flag with no value → NotFound.
                if s.starts_with(style.long_prefix) || s.starts_with(style.short_prefix) {
                    return PickerArgResult::NotFound;
                }

                // Positional value.
                PickerArgResult::Parsed(s.to_string())
            }
            _ => {
                // flag + value as separate args: take the second element.
                PickerArgResult::Parsed(raw_strs[1].to_string())
            }
        }
    }
}
