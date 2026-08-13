// Doc Not Optimize
#![allow(clippy::type_complexity)] // Aha, Type Gymnastics!

use arg_picker_macros::internal_repeat;

internal_repeat!(1..=32 => {
    #[doc(hidden)]
    pub struct PickerResult$<(T$,+), Route> {
        /// The route selected by the picker, if any.
        /// If this is `Some`, the picker chose to follow a route instead of selecting values,
        /// and all value fields (`v1`, `v2`, ...) will be `None`.
        ///
        /// Note: "route" here refers to an alternative path/choice, not a network route.
        pub route: Option<Route>,

        (
            #[doc = concat!("The optional value for the ", $, "th type parameter.")]
            pub v$: Option<T$>,
        +)
    }
});

internal_repeat!(1..=32 => {
    impl<(T$,+), Route> PickerResult$<(T$,+), Route> {
        /// Unwraps the result, panicking if a route was selected or a required
        /// value is missing.
        ///
        /// # Panics
        ///
        /// Panics if `self.route` is `Some(...)`, or if a required argument was
        /// not provided by the user.
        pub fn unwrap(self) -> ((T$,+)) {
            ((self.v$.expect(concat!("missing required argument at position ", $)),+))
        }

        /// Returns the parsed values, using the default values for any missing
        /// required arguments, or panicking if a route was selected.
        ///
        /// # Panics
        ///
        /// Panics if a route was selected.
        ///
        /// # Type Constraints
        ///
        /// All types in the tuple must implement [`Default`].
        pub fn unwrap_or_default(self) -> ((T$,+))
        where
            (
                T$: Default,
            +) {
            let p = self;
            ((p.v$.unwrap_or_default(),+))
        }

        /// Returns the parsed values, using the provided closure to generate
        /// default values for any missing required arguments, or panicking if
        /// a route was selected.
        ///
        /// # Panics
        ///
        /// Panics if a route was selected.
        pub fn unwrap_or_else<F>(self, op: F) -> ((T$,+))
        where
            F: FnOnce(Route) -> ((T$,+)),
        {
            let r = self.to_result();
            r.unwrap_or_else(op)
        }

        /// Returns the parsed values, or panics with the given message if
        /// a route was selected.
        ///
        /// # Panics
        ///
        /// Panics if a route was selected, with the provided message.
        ///
        /// # Type Constraints
        ///
        /// `Route` must implement [`std::fmt::Debug`] so that the error
        /// message can include the route value.
        pub fn expect(self, msg: &str) -> ((T$,+)) where Route: std::fmt::Debug {
            self.to_result().expect(msg)
        }

        /// Returns the individual option values without checking the route.
        pub fn unpack(self) -> ((Option<T$>,+)) {
            ((self.v$,+))
        }

        /// Converts to a `Result`, returning `Err(route)` if a route was selected,
        /// or `Ok(values)` otherwise.
        pub fn to_result(self) -> Result<((T$,+)), Route> {
            if let Some(r) = self.route {
                return Err(r);
            }
            Ok(self.unwrap())
        }

        /// Converts to an `Option`, returning `None` if a route was selected,
        /// or `Some(values)` otherwise.
        pub fn to_option(self) -> Option<((T$,+))> {
            if let Some(_) = self.route {
                return None;
            }
            Some(self.unwrap())
        }
    }
});
