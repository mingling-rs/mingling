use mingling_core::{Program, ProgramCollect, setup::ProgramSetup};

use crate::res::{ResCurrentDir, ResCurrentExe, ResHomeDir, ResTempDir};

/// `Directory Environment` Setup for managing common directory resources
///
/// This Setup registers commonly used directory resources into the program's
/// resource store. It provides the current working directory, the executable's
/// directory, the user's home directory, and the system's temporary directory,
/// so that these paths can be retrieved from the resource store without
/// recomputing them each time.
///
/// # Usage
///
/// This Setup can be registered using the
/// [`Program`](https://docs.rs/mingling/latest/mingling/struct.Program.html)
/// `with_setup` method, for example:
///
/// ```rust
/// # use mingling::MockProgramCollect as ThisProgram;
/// use mingling::Program;
/// use mingling::setup::DirectoryEnvironmentSetup;
///
/// let mut program = Program::<ThisProgram>::new();
/// program.with_setup(DirectoryEnvironmentSetup);
/// ```
///
/// # Behavior
///
/// - Registers an [`ResCurrentDir`] resource containing the current working
///   directory.
/// - Registers an [`ResCurrentExe`] resource containing the directory of the
///   currently running executable.
/// - Registers an [`ResHomeDir`] resource containing the user's home directory.
/// - Registers an [`ResTempDir`] resource containing the system's temporary
///   directory.
///
/// # Notes
///
/// - All directory values are resolved at setup time and stored in the
///   resource store.
/// - These resources can be retrieved later using the program's `resource`
///   accessor with the corresponding resource type.
pub struct DirectoryEnvironmentSetup;

impl<C> ProgramSetup<C> for DirectoryEnvironmentSetup
where
    C: ProgramCollect<Enum = C> + 'static,
{
    fn setup(self, program: &mut Program<C>) {
        program.with_resource(ResCurrentDir::default());
        program.with_resource(ResCurrentExe::default());
        program.with_resource(ResHomeDir::default());
        program.with_resource(ResTempDir::default());
    }
}
