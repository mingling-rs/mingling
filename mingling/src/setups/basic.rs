#![allow(deprecated)]

use mingling_core::{
    Flag, Program, ProgramCollect,
    config::{
        ConfirmationMode, ErrorOutput, InteractionMode, QueryMode, RenderOutput, Verbosity,
        YesAssumption,
    },
    setup::ProgramSetup,
};

/// Performs basic program initialization:
///
/// - Collects `--quiet` flag to control message rendering
/// - Collects `--help` flag to enable usage query mode
/// - Collects `--version` flag to enable version query mode
/// - Collects `--verbose` flag to enable verbose output
/// - Collects `--confirm` flag to skip user confirmation
#[cfg_attr(
    feature = "picker",
    deprecated(
        note = "When the `picker` feature is enabled, you can use `mingling::setup::picker::BasicProgramSetup` instead"
    )
)]
pub struct BasicProgramSetup;

impl<C> ProgramSetup<C> for BasicProgramSetup
where
    C: ProgramCollect<Enum = C>,
{
    fn setup(self, program: &mut Program<C>) {
        program.with_setup(HelpFlagSetup::new(["-h", "--help"]));
        program.with_setup(QuietFlagSetup::new(["-q", "--quiet"]));
        program.with_setup(VersionSetup::new(["-v", "--version"]));
        program.with_setup(VerboseSetup::new(["-V", "--verbose"]));
        program.with_setup(ConfirmFlagSetup::new(["-C", "--confirm"]));
    }
}

/// Provides setup for parsing the user help flag
///
/// The default value is `--help / -h`
#[cfg_attr(
    feature = "picker",
    deprecated(
        note = "When the `picker` feature is enabled, you can use `mingling::setup::picker::HelpFlagSetup` instead"
    )
)]
pub struct HelpFlagSetup {
    flag: Flag,
}

impl HelpFlagSetup {
    /// Creates a new `HelpFlagSetup` with the given flag aliases.
    pub fn new(flag: impl Into<Flag>) -> Self {
        Self { flag: flag.into() }
    }
}

impl<C> ProgramSetup<C> for HelpFlagSetup
where
    C: ProgramCollect<Enum = C>,
{
    fn setup(self, program: &mut Program<C>) {
        program.global_flag(self.flag, |p| {
            p.user_context.query = QueryMode::Usage;
        });
    }
}

impl Default for HelpFlagSetup {
    fn default() -> Self {
        Self {
            flag: ["-h", "--help"].into(),
        }
    }
}

/// Provides setup for parsing the version flag
///
/// The default value is `--version / -v`
#[cfg_attr(
    feature = "picker",
    deprecated(
        note = "When the `picker` feature is enabled, you can use `mingling::setup::picker::VersionSetup` instead"
    )
)]
pub struct VersionSetup {
    flag: Flag,
}

impl VersionSetup {
    /// Creates a new `VersionSetup` with the given flag aliases.
    pub fn new(flag: impl Into<Flag>) -> Self {
        Self { flag: flag.into() }
    }
}

impl<C> ProgramSetup<C> for VersionSetup
where
    C: ProgramCollect<Enum = C>,
{
    fn setup(self, program: &mut Program<C>) {
        program.global_flag(self.flag, |p| {
            p.user_context.query = QueryMode::Version;
        });
    }
}

impl Default for VersionSetup {
    fn default() -> Self {
        Self {
            flag: ["-v", "--version"].into(),
        }
    }
}

/// Provides setup for parsing the verbose flag
///
/// The default value is `--verbose / -V`
#[cfg_attr(
    feature = "picker",
    deprecated(
        note = "When the `picker` feature is enabled, you can use `mingling::setup::picker::VerboseSetup` instead"
    )
)]
pub struct VerboseSetup {
    flag: Flag,
}

impl VerboseSetup {
    /// Creates a new `VerboseSetup` with the given flag aliases.
    pub fn new(flag: impl Into<Flag>) -> Self {
        Self { flag: flag.into() }
    }
}

impl<C> ProgramSetup<C> for VerboseSetup
where
    C: ProgramCollect<Enum = C>,
{
    fn setup(self, program: &mut Program<C>) {
        program.global_flag(self.flag, |p| {
            p.stdout_setting.verbosity = Verbosity::Verbose;
        });
    }
}

impl Default for VerboseSetup {
    fn default() -> Self {
        Self {
            flag: ["-V", "--verbose"].into(),
        }
    }
}

/// Provides setup for parsing the quiet flag
///
/// The default value is `--quiet / -q`
#[cfg_attr(
    feature = "picker",
    deprecated(
        note = "When the `picker` feature is enabled, you can use `mingling::setup::picker::QuietFlagSetup` instead"
    )
)]
pub struct QuietFlagSetup {
    flag: Flag,
}

impl QuietFlagSetup {
    /// Creates a new `QuietFlagSetup` with the given flag aliases.
    pub fn new(flag: impl Into<Flag>) -> Self {
        Self { flag: flag.into() }
    }
}

impl<C> ProgramSetup<C> for QuietFlagSetup
where
    C: ProgramCollect<Enum = C>,
{
    fn setup(self, program: &mut Program<C>) {
        program.global_flag(self.flag, |p| {
            p.stdout_setting.render_output = RenderOutput::Hide;
            p.stdout_setting.error_output = ErrorOutput::Hide;
        });
    }
}

impl Default for QuietFlagSetup {
    fn default() -> Self {
        Self {
            flag: ["-q", "--quiet"].into(),
        }
    }
}

/// Provides setup for parsing the confirm flag
///
/// The default value is `--confirm / -C`
#[cfg_attr(
    feature = "picker",
    deprecated(
        note = "When the `picker` feature is enabled, you can use `mingling::setup::picker::ConfirmFlagSetup` instead"
    )
)]
pub struct ConfirmFlagSetup {
    flag: Flag,
}

impl ConfirmFlagSetup {
    /// Creates a new `ConfirmFlagSetup` with the given flag aliases.
    pub fn new(flag: impl Into<Flag>) -> Self {
        Self { flag: flag.into() }
    }
}

impl<C> ProgramSetup<C> for ConfirmFlagSetup
where
    C: ProgramCollect<Enum = C>,
{
    fn setup(self, program: &mut Program<C>) {
        program.global_flag(self.flag, |p| {
            p.user_context.confirmation = ConfirmationMode::Skip;
            p.user_context.interaction = InteractionMode::NonInteractive;
            p.user_context.yes_assumption = YesAssumption::AssumeYes;
        });
    }
}

impl Default for ConfirmFlagSetup {
    fn default() -> Self {
        Self {
            flag: ["-C", "--confirm"].into(),
        }
    }
}
