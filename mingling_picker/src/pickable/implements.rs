use crate::{Pickable, PickerFlagAttr};

impl<'a> Pickable<'a> for String {
    fn get_attr(flag: &'a crate::PickerFlag<'a, Self>) -> PickerFlagAttr {
        PickerFlagAttr::positional_or_single(flag)
    }

    fn tag(_ctx: super::TagPhaseContext) -> Vec<usize> {
        vec![]
    }

    fn pick(_raw_strs: &[&str]) -> crate::PickerArgResult<Self> {
        todo!()
    }
}

impl<'a> Pickable<'a> for Vec<String> {
    fn get_attr(flag: &'a crate::PickerFlag<'a, Self>) -> PickerFlagAttr {
        PickerFlagAttr::positional_or_multi(flag)
    }

    fn tag(_ctx: super::TagPhaseContext) -> Vec<usize> {
        vec![]
    }

    fn pick(_raw_strs: &[&str]) -> crate::PickerArgResult<Self> {
        todo!()
    }
}
