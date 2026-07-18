use arg_picker::{IntoPicker, PickerArg, value::Flag};
use mingling_core::{Program, ProgramCollect, setup::ProgramSetup};

use crate::{
    setup::picker::REMAINS,
    setups::picker::{CONFIRM_FLAG, HELP_FLAG, QUIET_FLAG},
};

/// Helper: picks a boolean flag from the program arguments, calls `f` with the
/// flag value, then replaces the program arguments with the remaining args.
fn pick_flag<'a, C>(
    program: &mut Program<C>,
    flag: &PickerArg<'a, Flag>,
    f: impl FnOnce(bool, &mut Program<C>),
) where
    C: ProgramCollect<Enum = C>,
{
    let args = program.take_args();
    let remains_arg = PickerArg::<PickerArgs<'a>>::new(&[], None, true);
    let (active, remains) = args.pick(flag).pick(&remains_arg).unwrap();
    f(*active, program);
    program.replace_args(remains.into());
}

/// Performs basic program initialization:
///
/// - Collects `--quiet` flag to control message rendering
/// - Collects `--help` flag to enable help mode
/// - Collects `--confirm` flag to skip user confirmation
pub struct BasicProgramSetup;

impl<C> ProgramSetup<C> for BasicProgramSetup
where
    C: ProgramCollect<Enum = C>,
{
    fn setup(self, program: &mut Program<C>) {
        program.with_setup(HelpFlagSetup::default());
        program.with_setup(QuietFlagSetup::default());
        program.with_setup(ConfirmFlagSetup::default());
    }
}

/// Provides setup for parsing the user help flag
///
/// The default value is `--help / -h`
pub struct HelpFlagSetup<'a> {
    flag: &'a PickerArg<'a, Flag>,
}

impl<'a> HelpFlagSetup<'a> {
    /// Creates a new `HelpFlagSetup` with the given flag aliases.
    pub fn new(flag: &'a PickerArg<Flag>) -> Self {
        Self { flag }
    }
}

impl<'a, C> ProgramSetup<C> for HelpFlagSetup<'a>
where
    C: ProgramCollect<Enum = C>,
{
    fn setup(self, program: &mut Program<C>) {
        pick_flag(program, self.flag, |active, ctx| {
            ctx.user_context.help = active;
        });
    }
}

impl<'a> Default for HelpFlagSetup<'a> {
    fn default() -> Self {
        Self { flag: &HELP_FLAG }
    }
}

/// Provides setup for parsing the quiet flag
///
/// The default value is `--quiet / -q`
pub struct QuietFlagSetup<'a> {
    flag: &'a PickerArg<'a, Flag>,
}

impl<'a> QuietFlagSetup<'a> {
    /// Creates a new `QuietFlagSetup` with the given flag aliases.
    pub fn new(flag: &'a PickerArg<Flag>) -> Self {
        Self { flag }
    }
}

impl<'a, C> ProgramSetup<C> for QuietFlagSetup<'a>
where
    C: ProgramCollect<Enum = C>,
{
    fn setup(self, program: &mut Program<C>) {
        pick_flag(program, self.flag, |active, ctx| {
            if active {
                ctx.stdout_setting.render_output = false;
                ctx.stdout_setting.error_output = false;
            }
        });
    }
}

impl<'a> Default for QuietFlagSetup<'a> {
    fn default() -> Self {
        Self { flag: &QUIET_FLAG }
    }
}

/// Provides setup for parsing the confirm flag
///
/// The default value is `--confirm / -C`
pub struct ConfirmFlagSetup<'a> {
    flag: &'a PickerArg<'a, Flag>,
}

impl<'a> ConfirmFlagSetup<'a> {
    /// Creates a new `ConfirmFlagSetup` with the given flag aliases.
    pub fn new(flag: &'a PickerArg<Flag>) -> Self {
        Self { flag }
    }
}

impl<'a, C> ProgramSetup<C> for ConfirmFlagSetup<'a>
where
    C: ProgramCollect<Enum = C>,
{
    fn setup(self, program: &mut Program<C>) {
        pick_flag(program, self.flag, |active, ctx| {
            if active {
                ctx.user_context.confirm = true;
            }
        });
    }
}

impl<'a> Default for ConfirmFlagSetup<'a> {
    fn default() -> Self {
        Self {
            flag: &CONFIRM_FLAG,
        }
    }
}
