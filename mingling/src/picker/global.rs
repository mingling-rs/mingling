// Doc Not Optimize
use arg_picker::{IntoPicker, Pickable, PickerArg, value::Flag};
use mingling_core::{Program, ProgramCollect};

use crate::consts::REMAINS;

/// Provides helper methods for picking arguments from a [`Program`]'s argument list.
///
/// This trait abstracts the functionality of extracting specific arguments or flags
/// from the program's current arguments, while restoring any remaining arguments
/// back into the program.
pub trait PickerHelper<C>
where
    C: ProgramCollect<Enum = C>,
{
    /// Takes ownership of the program's current arguments.
    ///
    /// Returns the program's argument list as a [`Vec<String>`], leaving the program
    /// with no arguments until [`replace_args`] is called.
    ///
    /// [`replace_args`]: PickerHelper::replace_args
    fn take_args(&mut self) -> Vec<String>;

    /// Replaces the program's current arguments with the provided list.
    ///
    /// Returns the previous argument list that was replaced.
    fn replace_args(&mut self, args: Vec<String>) -> Vec<String>;

    /// Picks a flag from the program's arguments.
    ///
    /// This function takes ownership of the program's current arguments, picks the specified `flag`
    /// from them, and then returns the remaining arguments back to the program. It returns the
    /// boolean value of the flag.
    fn pick_flag(&mut self, flag: &PickerArg<Flag>) -> bool {
        let args = self.take_args();
        let (flag, args) = args.pick(flag).pick(&REMAINS).unwrap();
        self.replace_args(args.into());
        *flag
    }

    /// Picks a argument from the program's arguments.
    ///
    /// This function takes ownership of the program's current arguments, picks the specified `arg`
    /// from them, and then returns the remaining arguments back to the program. It returns the
    /// picked argument value, or `None` if the argument was not present.
    fn pick_argument<A>(&mut self, arg: &PickerArg<A>) -> Option<A>
    where
        A: for<'a> Pickable<'a> + Default,
    {
        let args = self.take_args();
        let (arg, remains) = args.pick(arg).pick(&REMAINS).unpack();
        self.replace_args(remains.unwrap().into());
        arg
    }
}

impl<C> PickerHelper<C> for Program<C>
where
    C: ProgramCollect<Enum = C>,
{
    fn take_args(&mut self) -> Vec<String> {
        self.take_args()
    }

    fn replace_args(&mut self, args: Vec<String>) -> Vec<String> {
        self.replace_args(args)
    }
}
