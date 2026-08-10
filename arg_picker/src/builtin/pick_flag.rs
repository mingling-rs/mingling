use crate::parselib::{FlagMatcher, Matcher};
use crate::pickable_needed::*;
use crate::value::Flag;

impl<'a> Pickable<'a> for Flag {
    fn get_attr(_: &'a PickerArg<'a, Self>) -> PickerArgAttr {
        PickerArgAttr::Flag
    }

    fn tag(ctx: TagPhaseContext) -> Vec<usize> {
        FlagMatcher::match_all(ctx.into())
    }

    fn pick(raw_strs: &[&str]) -> PickerArgResult<Self> {
        if raw_strs.is_empty() {
            PickerArgResult::Parsed(Self::Inactive)
        } else {
            PickerArgResult::Parsed(Self::Active)
        }
    }
}
