use crate::parselib::{ParserStyle, SingleMatcher};
use crate::pickable_needed::*;

impl<'a> Pickable<'a> for String {
    fn get_attr(flag: &'a PickerArg<'a, Self>) -> PickerArgAttr {
        PickerArgAttr::positional_or_single(flag)
    }

    fn tag(ctx: TagPhaseContext) -> Vec<usize> {
        SingleMatcher::tag(ctx)
    }

    fn pick(raw_strs: &[&str]) -> PickerArgResult<Self> {
        match raw_strs.len() {
            0 => PickerArgResult::NotFound,
            1 => {
                let s = raw_strs[0];
                let sep = ParserStyle::global_style().value_separator;
                if let Some(pos) = s.rfind(sep) {
                    return PickerArgResult::Parsed(s[pos + 1..].to_string());
                }
                PickerArgResult::Parsed(s.to_string())
            }
            _ => PickerArgResult::Parsed(raw_strs[1].to_string()),
        }
    }
}
