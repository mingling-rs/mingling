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
            // Check for flag-without-value before consuming ctx.
            let args = ctx.args;
            let positions = ArgMatcher::match_all(ctx.into());
            if positions.len() == 1 {
                let sep = ParserStyle::global_style().value_separator;
                if let Some(raw) = args.get(positions[0])
                    && !raw.contains(sep)
                {
                    return vec![];
                }
            }
            positions
        }
    }

    fn pick(raw_strs: &[&str]) -> PickerArgResult<Self> {
        match raw_strs.len() {
            0 => PickerArgResult::NotFound,
            1 => {
                let s = raw_strs[0];
                // Inline value via style separator (e.g., --name=Alice, -Name:Alice).
                let sep = ParserStyle::global_style().value_separator;
                if let Some(pos) = s.rfind(sep) {
                    return PickerArgResult::Parsed(s[pos + 1..].to_string());
                }
                // Positional value or single raw token — return as-is.
                // (Named flags without a value are already filtered out by tag.)
                PickerArgResult::Parsed(s.to_string())
            }
            _ => {
                // flag + value as separate args: take the second element.
                PickerArgResult::Parsed(raw_strs[1].to_string())
            }
        }
    }
}
