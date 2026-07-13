use mingling_picker_macros::internal_repeat;

use crate::{Pickable, Picker, PickerArgs, PickerFlag, PickerResult};

internal_repeat!(1..=32 => {
    pub struct PickerPattern$<'a, (T$,+)>
    where (T$: Pickable + Default,+)
    {
        pub args: PickerArgs<'a>,
        (
            pub flag_$: &'a PickerFlag<'a, T$>,
            pub result_$: PickerResult<T$>,
        +)
    }
});

internal_repeat!(1..32 => {
   impl<'a, (T$,+)> PickerPattern$<'a, (T$,+)>
   where (T$: Pickable + Default,+)
   {
       #[allow(clippy::type_complexity)]
       pub fn pick<N>(self, flag: &'a PickerFlag<'a, N>) -> PickerPattern$+<'a, (T$,+), N>
       where
           N: Pickable + Default,
       {
           PickerPattern$+ {
               // Args
               args: self.args,

               // Current
               flag_$+: flag,
               result_$+: PickerResult::Unparsed,

               // Prev
               (
                   flag_$: self.flag_$,
                   result_$: self.result_$,
                +)
           }
       }
   }
});

impl<'a> Picker<'a> {
    pub fn pick<N>(self, flag: &'a PickerFlag<'a, N>) -> PickerPattern1<'a, N>
    where
        N: Pickable + Default,
    {
        PickerPattern1 {
            args: self.args,
            flag_1: flag,
            result_1: PickerResult::Unparsed,
        }
    }
}
