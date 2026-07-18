use arg_picker::{IntoPicker, Pickable, PickerArg, value::Flag};
use mingling_core::{Program, ProgramCollect};

use crate::consts::REMAINS;

/// Picks a global flag from the program's arguments.
///
/// This function takes ownership of the program's current arguments, picks the specified `flag`
/// from them, and then returns the remaining arguments back to the program. It returns the
/// boolean value of the flag.
pub fn pick_global_flag<C>(program: &mut Program<C>, flag: &PickerArg<Flag>) -> bool
where
    C: ProgramCollect<Enum = C>,
{
    let args = program.take_args();
    let (flag, args) = args.pick(flag).pick(&REMAINS).unwrap();
    program.replace_args(args.into());
    *flag
}

/// Picks a global argument from the program's arguments.
///
/// This function takes ownership of the program's current arguments, picks the specified `arg`
/// from them, and then returns the remaining arguments back to the program. It returns the
/// picked argument value, or `None` if the argument was not present.
pub fn pick_global_argument<C, A>(program: &mut Program<C>, arg: &PickerArg<A>) -> Option<A>
where
    A: for<'a> Pickable<'a> + Default,
    C: ProgramCollect<Enum = C>,
{
    let args = program.take_args();
    let (arg, remains) = args.pick(arg).pick(&REMAINS).unpack();
    program.replace_args(remains.unwrap().into());
    arg
}
