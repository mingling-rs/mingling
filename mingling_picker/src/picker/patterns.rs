use crate::{Pickable, PickerArguments, PickerRequirement};

mingling_picker_macros::internal_repeat! (1..=32 => {
    pub struct PickerPattern$<'a, (Type$,)+>
    where (Type$: Pickable + Default,)+
    {
        pub args: PickerArguments<'a>,
        (
            pub require_$: PickerRequirement<'a, Type$>,
        )+
    }
});
