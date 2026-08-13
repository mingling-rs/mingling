// Doc Not Optimize
use arg_picker::{PickerArg, value::Flag};
use mingling_core::{
    ConfirmationMode, ErrorOutput, InteractionMode, Program, ProgramCollect, RenderOutput,
    YesAssumption, setup::ProgramSetup,
};

use crate::{
    consts::{CONFIRM_FLAG, HELP_FLAG, QUIET_FLAG},
    picker::PickerHelper,
};

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
    #[must_use]
    pub const fn new(flag: &'a PickerArg<Flag>) -> Self {
        Self { flag }
    }
}

impl<C> ProgramSetup<C> for HelpFlagSetup<'_>
where
    C: ProgramCollect<Enum = C>,
{
    fn setup(self, program: &mut Program<C>) {
        let help = program.pick_flag(self.flag);
        if help {
            program.user_context.help = true;
        }
    }
}

impl Default for HelpFlagSetup<'_> {
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
    #[must_use]
    pub const fn new(flag: &'a PickerArg<Flag>) -> Self {
        Self { flag }
    }
}

impl<C> ProgramSetup<C> for QuietFlagSetup<'_>
where
    C: ProgramCollect<Enum = C>,
{
    fn setup(self, program: &mut Program<C>) {
        let quiet = program.pick_flag(self.flag);
        if quiet {
            program.stdout_setting.render_output = RenderOutput::Hide;
            program.stdout_setting.error_output = ErrorOutput::Hide;
        }
    }
}

impl Default for QuietFlagSetup<'_> {
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
    #[must_use]
    pub const fn new(flag: &'a PickerArg<Flag>) -> Self {
        Self { flag }
    }
}

impl<C> ProgramSetup<C> for ConfirmFlagSetup<'_>
where
    C: ProgramCollect<Enum = C>,
{
    fn setup(self, program: &mut Program<C>) {
        let confirm = program.pick_flag(self.flag);
        if confirm {
            program.user_context.confirmation = ConfirmationMode::Skip;
            program.user_context.interaction = InteractionMode::NonInteractive;
            program.user_context.yes_assumption = YesAssumption::AssumeYes;
        }
    }
}

impl Default for ConfirmFlagSetup<'_> {
    fn default() -> Self {
        Self {
            flag: &CONFIRM_FLAG,
        }
    }
}
