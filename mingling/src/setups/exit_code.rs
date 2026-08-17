use mingling_core::{
    Program, ProgramCollect,
    hook::{ProgramControlUnit, ProgramControls, ProgramHook},
    setup::ProgramSetup,
    this,
};

use crate::res::ResExitCode;

/// `ExitCodeSetup` — Setup for controlling the program's exit code
///
/// This Setup registers an [`ResExitCode`] resource that tracks the desired exit
/// code for the program. When the program finishes, a hook reads this resource
/// and overrides the program's exit code if it has been modified from its
/// default value of `0`.
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
/// use mingling::setup::ExitCodeSetup;
///
/// let mut program = Program::<ThisProgram>::new();
/// program.with_setup(ExitCodeSetup);
/// ```
///
/// # Behavior
///
/// - Registers an [`ResExitCode`] resource initialised to `0`.
/// - Installs a program-finish hook that:
///   - Reads the current [`ResExitCode`] value.
///   - Overrides the program's exit code with that value if it is non-zero.
///   - Leaves the exit code untouched if the resource still holds its default
///     value of `0`.
///
/// # Notes
///
/// - Use [`update_exit_code`](crate::update_exit_code) to set a custom exit code
///   during program execution.
/// - Use [`current_exit_code`](crate::current_exit_code) to query the current value.
pub struct ExitCodeSetup;

impl<C> ProgramSetup<C> for ExitCodeSetup
where
    C: ProgramCollect<Enum = C> + 'static,
{
    fn setup(self, program: &mut Program<C>) {
        // Insert resource
        program.with_resource(ResExitCode { exit_code: 0 });

        // Insert hook to override exit code before program ends
        program.with_hook(ProgramHook::empty().on_finish(|_| {
            let this = this::<C>().res_or_default::<ResExitCode>();
            let ec = this.exit_code;

            // Only override when ResExitCode has been modified
            if ec != 0 {
                ProgramControlUnit::OverrideExitCode(this.exit_code).into()
            } else {
                ProgramControls::Empty
            }
        }));
    }
}
