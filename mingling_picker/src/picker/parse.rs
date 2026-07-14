use crate::{Pickable, PickerResult};
use mingling_picker_macros::internal_repeat;

internal_repeat!(1..=32 => {
    use crate::PickerPattern$;
});

internal_repeat!(1..=32 => {
    impl<'a, (T$,+)> PickerPattern$<'a, (T$,+)>
    where (T$: Pickable<'a> + Default,+)
    {
        #[allow(clippy::type_complexity)]
        pub fn parse(self) -> PickerResult<((T$,+))> {
            todo!()
        }
    }
});
