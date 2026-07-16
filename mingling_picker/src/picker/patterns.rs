use mingling_picker_macros::internal_repeat;

use crate::{Pickable, Picker, PickerArg, PickerArgResult, PickerArgs};

internal_repeat!(1..=32 => {
    #[doc(hidden)]
    pub struct PickerPattern$<'a, (T$,+), Route>
    where (T$: Pickable<'a>,+)
    {
        pub args: PickerArgs<'a>,
        pub error_route: Option<Route>,
        (
            pub arg_$: &'a PickerArg<'a, T$>,
            pub result_$: PickerArgResult<T$>,
            pub default_$: Option<Box<dyn FnOnce() -> T$>>,
            pub route_$: Option<Box<dyn FnOnce() -> Route>>,
            pub post_$: Option<Box<dyn FnOnce(T$) -> T$>>,
        +)
    }
});

internal_repeat!(1..=32 => {
    impl<'a, (T$,+), Route> PickerPattern$<'a, (T$,+), Route>
    where (T$: Pickable<'a>,+)
    {
        /// Sets a default value provider for this arg.
        ///
        /// If the arg is not provided by the user at runtime, the given closure will be
        /// called to produce a default value. The closure is expected to return `T$`.
        ///
        /// # Example
        ///
        /// ```ignore
        /// let pattern = picker
        ///     .pick(&my_arg)
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

        /// Uses the default value for this arg's type if the arg is not provided.
        ///
        /// If the arg is not provided by the user at runtime, the default value for `T$`
        /// (as defined by the `Default` trait) will be used.
        ///
        /// # Example
        ///
        /// ```ignore
        /// let pattern = picker
        ///     .pick(&my_arg)
        ///     .or_default();
        /// ```
        #[allow(clippy::type_complexity)]
        pub fn or_default(mut self) -> Self
        where
            T$: Default,
        {
            self.default_$ = Some(Box::new(|| T$::default()));
            self
        }

        /// Sets a route for when the arg is not provided.
        ///
        /// If the arg is not provided by the user at runtime, the given closure will be
        /// called to produce a route value that will be returned early.
        ///
        /// # Example
        ///
        /// ```ignore
        /// let pattern = picker
        ///     .pick(&my_arg)
        ///     .or_route(|| Redirect::home());
        /// ```
        pub fn or_route<F>(mut self, func: F) -> Self
        where
            F: FnMut() -> Route,
            F: 'static,
        {
            self.route_$ = Some(Box::new(func));
            self
        }


        /// Resets the route for this picker pattern, allowing a different route type.
        ///
        /// This method converts the current `PickerPattern` into a new one with a different
        /// route type `NewRoute`. All existing arg configurations, defaults, and post-
        /// processing functions are preserved, but the `error_route` and individual
        /// `route_$` fields are cleared (set to `None`).
        ///
        /// This is useful when you want to change the error/redirect route type mid-chain,
        /// for example when composing patterns from different contexts that use different
        /// route enums.
        #[allow(clippy::type_complexity)]
        pub fn with_route<NewRoute>(self) -> PickerPattern$<'a, (T$,+), NewRoute> {
            PickerPattern$ {
                args: self.args,
                error_route: None,
                (
                    arg_$: self.arg_$,
                    result_$: self.result_$,
                    default_$: self.default_$,
                    route_$: None,
                    post_$: self.post_$,
                +)
            }
        }

        /// Attaches a post-processing function to this arg.
        ///
        /// After the arg's value is parsed (or defaulted), the given closure will be
        /// invoked with the parsed value and its return value will be used as the final
        /// result. This allows transforming or validating the parsed value.
        ///
        /// # Example
        ///
        /// ```ignore
        /// let pattern = picker
        ///     .pick(&my_arg)
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
   impl<'a, (T$,+), Route> PickerPattern$<'a, (T$,+), Route>
   where (T$: Pickable<'a>,+)
   {
       #[allow(clippy::type_complexity)]
       /// Adds a new arg to the picking chain, returning a new `PickerPattern` with one more type parameter.
       ///
       /// This method extends the current picking pattern by appending an additional arg.
       /// The previous args and their results are preserved as part of the new pattern.
       /// The new arg's result is initially `Unparsed`.
       pub fn pick<N>(self, arg: impl Into<&'a PickerArg<'a, N>>) -> PickerPattern$+<'a, (T$,+), N, Route>
       where
           N: Pickable<'a>,
       {
           PickerPattern$+ {
               // Args
               args: self.args,
               error_route: self.error_route,

               // Current
               arg_$+: arg.into(),
               result_$+: PickerArgResult::Unparsed,
               default_$+: None,
               route_$+: None,
               post_$+: None,

               // Prev
               (
                   arg_$: self.arg_$,
                   result_$: self.result_$,
                   default_$: self.default_$,
                   route_$: self.route_$,
                   post_$: self.post_$,
                +)
           }
       }
   }
});

impl<'a, Route> Picker<'a, Route> {
    /// Creates a `PickerPattern1` from the given arg to start a picking chain.
    ///
    /// This method initiates a parameter picking chain with one arg.
    /// The result is initially `Unparsed`.
    pub fn pick<N>(self, arg: impl Into<&'a PickerArg<'a, N>>) -> PickerPattern1<'a, N, Route>
    where
        N: Pickable<'a>,
    {
        PickerPattern1 {
            args: self.args,
            error_route: None::<Route>,
            arg_1: arg.into(),
            result_1: PickerArgResult::Unparsed,
            route_1: None,
            default_1: None,
            post_1: None,
        }
    }
}
