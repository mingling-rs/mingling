use crate::{PickerArgResult::Parsed, PickerArgs, parselib::build_masked_args, pickable_needed::*};

impl<'a> Pickable<'a> for PickerArgs<'a> {
    fn get_attr(_flag: &'a PickerArg<'a, Self>) -> PickerArgAttr {
        // Use the lowest priority attribute
        PickerArgAttr::Postprocess
    }

    fn tag(ctx: TagPhaseContext) -> Vec<usize> {
        // Collect all remaining raw index values
        build_masked_args(ctx.args, ctx.mask)
            .iter()
            .map(|m| m.raw_idx)
            .collect()
    }

    fn pick(raw_strs: &[&str]) -> PickerArgResult<Self> {
        let remains: Vec<String> = raw_strs.iter().map(|s| s.to_string()).collect();
        Parsed(PickerArgs::Owned(remains))
    }
}
