use crate::parselib::{FlagMatcher, Matcher};
use crate::pickable_needed::*;

impl<'a> Pickable<'a> for bool {
    fn get_attr(_: &'a PickerArg<'a, Self>) -> PickerArgAttr {
        PickerArgAttr::Flag
    }

    fn tag(ctx: TagPhaseContext) -> Vec<usize> {
        FlagMatcher::match_all(ctx.into())
    }

    fn pick(raw_strs: &[&str]) -> PickerArgResult<Self> {
        if raw_strs.is_empty() {
            // No matching flag found — signal NotFound so the fallback chain
            // (default → route) gets a chance to run.
            PickerArgResult::NotFound
        } else {
            PickerArgResult::Parsed(true)
        }
    }
}
