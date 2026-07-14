use mingling_picker_macros::internal_repeat;

use crate::{Pickable, Picker, PickerArgs, PickerFlag, PickerResult};

internal_repeat!(1..=32 => {
    #[doc(hidden)]
    pub struct PickerPattern$<'a, (T$,+)>
    where (T$: Pickable<'a> + Default,+)
    {
        pub args: PickerArgs<'a>,
        (
            pub flag_$: &'a PickerFlag<'a, T$>,
            pub result_$: PickerResult<T$>,
            pub default_$: Option<Box<dyn FnOnce() -> T$>>,
            pub post_$: Option<Box<dyn FnOnce(T$) -> T$>>,
        +)
    }
});

internal_repeat!(1..=32 => {
    impl<'a, (T$,+)> PickerPattern$<'a, (T$,+)>
    where (T$: Pickable<'a> + Default,+)
    {
        /// Sets a default value provider for this flag.
        ///
        /// If the flag is not provided by the user at runtime, the given closure will be
        /// called to produce a default value. The closure is expected to return `T$`.
        ///
        /// # Example
        ///
        /// ```ignore
        /// let pattern = picker
        ///     .pick(&my_flag)
        ///     .or(|| 42);
        /// ```
        #[allow(clippy::type_complexity)]
        pub fn or<F>(mut self, func: F) -> Self
        where
            F: FnMut() -> T$,
            F: 'static,
        {
            self.default_$ = Some(Box::new(func));
            self
        }

        /// Attaches a post-processing function to this flag.
        ///
        /// After the flag's value is parsed (or defaulted), the given closure will be
        /// invoked with the parsed value and its return value will be used as the final
        /// result. This allows transforming or validating the parsed value.
        ///
        /// # Example
        ///
        /// ```ignore
        /// let pattern = picker
        ///     .pick(&my_flag)
        ///     .post(|val| val * 2);
        /// ```
        #[allow(clippy::type_complexity)]
        pub fn post<F>(mut self, func: F) -> Self
        where
            F: FnMut(T$) -> T$,
            F: 'static,
        {
            self.post_$ = Some(Box::new(func));
            self
        }
    }
});

internal_repeat!(1..32 => {
   impl<'a, (T$,+)> PickerPattern$<'a, (T$,+)>
   where (T$: Pickable<'a> + Default,+)
   {
       #[allow(clippy::type_complexity)]
       /// Adds a new flag to the picking chain, returning a new `PickerPattern` with one more type parameter.
       ///
       /// This method extends the current picking pattern by appending an additional flag.
       /// The previous flags and their results are preserved as part of the new pattern.
       /// The new flag's result is initially `Unparsed`.
       pub fn pick<N>(self, flag: &'a PickerFlag<'a, N>) -> PickerPattern$+<'a, (T$,+), N>
       where
           N: Pickable<'a> + Default,
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
        N: Pickable<'a> + Default,
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
