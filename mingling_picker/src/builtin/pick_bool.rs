use crate::{parselib::Matcher, pickable_needed::*};

impl<'a> Pickable<'a> for bool {
    fn get_attr(_: &'a PickerFlag<'a, Self>) -> PickerFlagAttr {
        PickerFlagAttr::Flag
    }

    fn tag(ctx: TagPhaseContext) -> Vec<usize> {
        <bool as Matcher>::match_all(ctx.into())
    }

    fn pick(raw_strs: &[&str]) -> PickerArgResult<Self> {
        PickerArgResult::Parsed(!raw_strs.is_empty())
    }
}
