use std::marker::PhantomData;

use mingling_core::{ProgramCollect, setup::ProgramSetup};

use crate::res::{ResCurrentDir, ResCurrentExe, ResHomeDir, ResTempDir};

/// Provides the ability to set up commonly used directory resources for the program.
///
/// This setup item registers the following directory resources in the program:
/// - `ResCurrentDir`: Current working directory
/// - `ResCurrentExe`: Directory containing the executable
/// - `ResHomeDir`: User's home directory
/// - `ResTempDir`: Temporary directory
pub struct DirectoryEnvironmentSetup<C> {
    _collect: PhantomData<C>,
}

impl<C> Default for DirectoryEnvironmentSetup<C>
where
    C: ProgramCollect<Enum = C> + 'static,
{
    fn default() -> Self {
        Self {
            _collect: PhantomData,
        }
    }
}

impl<C> ProgramSetup<C> for DirectoryEnvironmentSetup<C>
where
    C: ProgramCollect<Enum = C> + 'static,
{
    fn setup(self, program: &mut crate::Program<C>) {
        program.with_resource(ResCurrentDir::default());
        program.with_resource(ResCurrentExe::default());
        program.with_resource(ResHomeDir::default());
        program.with_resource(ResTempDir::default());
    }
}
