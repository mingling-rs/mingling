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
            pub default_$: Option<Box<dyn FnMut() -> T$>>,
            pub post_$: Option<Box<dyn FnMut(T$) -> T$>>,
        +)
    }
});

internal_repeat!(1..32 => {
   impl<'a, (T$,+)> PickerPattern$<'a, (T$,+)>
   where (T$: Pickable + Default,+)
   {
       #[allow(clippy::type_complexity)]
       /// Adds a new flag to the picking chain, returning a new `PickerPattern` with one more type parameter.
       ///
       /// This method extends the current picking pattern by appending an additional flag.
       /// The previous flags and their results are preserved as part of the new pattern.
       /// The new flag's result is initially `Unparsed`.
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
               default_$+: None,
               post_$+: None,

               // Prev
               (
                   flag_$: self.flag_$,
                   result_$: self.result_$,
                   default_$: self.default_$,
                   post_$: self.post_$,
                +)
           }
       }
   }
});

impl<'a> Picker<'a> {
    /// Creates a `PickerPattern1` from the given flag to start a picking chain.
    ///
    /// This method initiates a parameter picking chain with one flag.
    /// The result is initially `Unparsed`.
    pub fn pick<N>(self, flag: &'a PickerFlag<'a, N>) -> PickerPattern1<'a, N>
    where
        N: Pickable + Default,
    {
        PickerPattern1 {
            args: self.args,
            flag_1: flag,
            result_1: PickerResult::Unparsed,
            default_1: None,
            post_1: None,
        }
    }
}
