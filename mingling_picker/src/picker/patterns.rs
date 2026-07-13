use crate::{Pickable, Picker, PickerArguments, PickerRequirement, PickerResult};

mingling_picker_macros::internal_repeat! (1..=32 => {
    pub struct PickerPattern$<'a, (T$,)+>
    where (T$: Pickable + Default,)+
    {
        pub args: PickerArguments<'a>,
        (
            pub require_$: &'a PickerRequirement<'a, T$>,
            pub result_$: PickerResult<T$>,
        )+
    }
});

impl<'a> Picker<'a> {
    pub fn pick<N>(self, req: &'a PickerRequirement<'a, N>) -> PickerPattern1<'a, N>
    where
        N: Pickable + Default,
    {
        PickerPattern1 {
            args: self.args,
            require_1: req,
            result_1: PickerResult::Unparsed,
        }
    }
}
