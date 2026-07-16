use crate::parselib::{SingleMatcher, seek_single};
use crate::pickable_needed::*;

impl<'a> Pickable<'a> for String {
    fn get_attr(flag: &'a PickerArg<'a, Self>) -> PickerArgAttr {
        PickerArgAttr::positional_or_single(flag)
    }

    fn tag(ctx: TagPhaseContext) -> Vec<usize> {
        SingleMatcher::tag(ctx)
    }

    fn pick(raw_strs: &[&str]) -> PickerArgResult<Self> {
        match seek_single(raw_strs) {
            Some(v) => PickerArgResult::Parsed(v.to_string()),
            None => PickerArgResult::NotFound,
        }
    }
}
