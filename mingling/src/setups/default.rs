use mingling_core::{Program, ProgramCollect, setup::ProgramSetup};

/// `DefaultSetup` — Setup for orchestrating a standard program configuration
///
/// This Setup composes a collection of commonly used setups into a single
/// convenience entry point. It handles basic flag parsing, environment
/// setup, and (optionally) interactive picker/renderer configuration based
/// on the enabled feature flags.
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
/// use mingling::setup::DefaultSetup;
///
/// let mut program = Program::<ThisProgram>::new();
/// program.with_setup(DefaultSetup);
/// ```
///
/// # Behavior
///
/// - When the `picker` feature is enabled, registers:
///   - The picker-specific [`crate::setup::picker::BasicProgramSetup`] for flag collection.
///   - The picker-specific [`crate::setup::picker::StructuralRendererSetup`] when the
///     `structural_renderer` feature is also enabled.
/// - When the `picker` feature is disabled, registers:
///   - The standard [`crate::setup::BasicProgramSetup`] for flag collection.
///   - The standard [`crate::setup::StructuralRendererSetup`] when the
///     `structural_renderer` feature is enabled.
/// - Always registers:
///   - An [`crate::setup::ExitCodeSetup`] to manage program exit codes.
///   - A [`crate::setup::DirectoryEnvironmentSetup`] to register common directory
///     resources (current dir, executable dir, home dir, temp dir).
///
/// # Notes
///
/// - Feature-dependent setups are conditionally included based on the
///   `picker` and `structural_renderer` feature flags.
/// - This struct is a zero-sized marker type used purely as a configuration
///   unit; it holds no state of its own.
pub struct DefaultSetup;

impl<C> ProgramSetup<C> for DefaultSetup
where
    C: ProgramCollect<Enum = C> + 'static,
{
    fn setup(self, program: &mut Program<C>) {
        #[cfg(feature = "picker")]
        {
            program.with_setup(crate::setup::picker::BasicProgramSetup);
            #[cfg(feature = "structural_renderer")]
            {
                program.with_setup(crate::setup::picker::StructuralRendererSetup);
            }
        }
        #[cfg(not(feature = "picker"))]
        {
            program.with_setup(crate::setup::BasicProgramSetup);
            #[cfg(feature = "structural_renderer")]
            {
                program.with_setup(crate::setup::StructuralRendererSetup);
            }
        }
        program.with_setup(crate::setup::ExitCodeSetup);
        program.with_setup(crate::setup::DirectoryEnvironmentSetup);
    }
}
