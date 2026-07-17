use std::marker::PhantomData;

use mingling_core::{ChainProcess, Groupped, ProgramCollect};

use crate::{Pickable, Picker, PickerArg, PickerArgResult, PickerArgs, PickerPattern1};

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
    /// * `arg` — The argument definition, typically obtained from [`crate::arg`].
    fn pick<Next>(
        self,
        arg: impl Into<&'a PickerArg<'a, Next>>,
    ) -> PickerPattern1<'a, Next, ChainProcess<This>>
    where
        Self: Sized,
        Next: Pickable<'a> + Default + Sized,
    {
        let picker = Self::to_picker(self);
        PickerPattern1 {
            args: picker.args,
            arg_1: arg.into(),
            result_1: PickerArgResult::Unparsed,
            default_1: None,
            route_1: None,
            post_1: None,
            error_route: None,
        }
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
