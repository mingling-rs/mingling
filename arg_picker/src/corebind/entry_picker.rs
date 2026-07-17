use std::marker::PhantomData;

use mingling_core::{ChainProcess, Groupped, ProgramCollect};

use crate::{Pickable, Picker, PickerArg, PickerArgs, PickerPattern1};

/// Trait for converting Mingling entry types (types that implement `Groupped<R>` and `Into<Vec<String>>`)
/// into [`Picker`] instances for a given route type `R`.
///
/// This trait provides a bridge between entry definitions created with the `mingling` framework
/// and the picker argument system used for CLI argument parsing and routing.
///
/// # Type Parameters
///
/// * `'a` — The lifetime of the picker and its references to argument definitions.
/// * `This` — The program type used for dispatching runtime routes; must implement [`ProgramCollect`].
/// * `Route` — The route type used for dispatching; must implement [`Groupped<This>`].
pub trait EntryPicker<'a, This> {
    /// Converts `self` into a [`Picker`] for the given route type `Route`.
    fn to_picker(self) -> Picker<'a, ChainProcess<This>>;

    /// Starts building a picker pattern with the first argument.
    ///
    /// Returns a [`PickerPattern1`] that can be further chained with additional
    /// arguments, defaults, routes, and post-processing.
    ///
    /// # Type Parameters
    ///
    /// * `Next` — The nominal type of the first argument; must implement [`Pickable`].
    ///
    /// # Parameters
    ///
    /// * `arg` — The argument definition, typically obtained from [`crate::macros::arg`].
    fn pick<Next>(
        self,
        arg: impl Into<&'a PickerArg<'a, Next>>,
    ) -> PickerPattern1<'a, Next, ChainProcess<This>>
    where
        Self: Sized,
        Next: Pickable<'a> + Default + Sized,
    {
        let picker = Self::to_picker(self);
        Picker::build_pattern1(picker.args, arg.into(), None)
    }

    /// Starts building a picker pattern with the first argument, using a default value provider.
    ///
    /// This is a shorthand for calling `.pick(arg).or(func)`.
    ///
    /// # Type Parameters
    ///
    /// * `Next` — The nominal type of the first argument; must implement [`Pickable`].
    ///
    /// # Parameters
    ///
    /// * `arg` — The argument definition, typically obtained from [`crate::macros::arg`].
    /// * `func` — A closure that provides a default value if the arg is not provided by the user.
    fn pick_or<Next, F>(
        self,
        arg: impl Into<&'a PickerArg<'a, Next>>,
        func: F,
    ) -> PickerPattern1<'a, Next, ChainProcess<This>>
    where
        Self: Sized,
        Next: Pickable<'a> + Default + Sized,
        F: FnMut() -> Next + 'static,
    {
        self.pick(arg).or(func)
    }

    /// Starts building a picker pattern with the first argument, using a default value.
    ///
    /// This is a shorthand for calling `.pick(arg).or_default()`.
    ///
    /// # Type Parameters
    ///
    /// * `Next` — The nominal type of the first argument; must implement [`Pickable`] and [`Default`].
    ///
    /// # Parameters
    ///
    /// * `arg` — The argument definition, typically obtained from [`crate::macros::arg`].
    fn pick_or_default<Next>(
        self,
        arg: impl Into<&'a PickerArg<'a, Next>>,
    ) -> PickerPattern1<'a, Next, ChainProcess<This>>
    where
        Self: Sized,
        Next: Pickable<'a> + Default + Sized,
    {
        self.pick(arg).or_default()
    }

    /// Starts building a picker pattern with the first argument, using a route if the arg is not provided.
    ///
    /// This is a shorthand for calling `.pick(arg).or_route(func)`.
    ///
    /// # Type Parameters
    ///
    /// * `Next` — The nominal type of the first argument; must implement [`Pickable`].
    ///
    /// # Parameters
    ///
    /// * `arg` — The argument definition, typically obtained from [`crate::macros::arg`].
    /// * `func` — A closure that produces a route value if the arg is not provided by the user.
    fn pick_or_route<Next, F>(
        self,
        arg: impl Into<&'a PickerArg<'a, Next>>,
        func: F,
    ) -> PickerPattern1<'a, Next, ChainProcess<This>>
    where
        Self: Sized,
        Next: Pickable<'a> + Default + Sized,
        F: FnMut() -> ChainProcess<This> + 'static,
    {
        self.pick(arg).or_route(func)
    }
}

impl<'a, This, Bind> EntryPicker<'a, This> for Bind
where
    This: ProgramCollect<Enum = This>,
    Bind: Groupped<This> + Into<Vec<String>>,
{
    fn to_picker(self) -> Picker<'a, ChainProcess<This>> {
        let args = self.into();
        Picker {
            route_phantom: PhantomData,
            args: PickerArgs::Owned(args),
        }
    }
}
