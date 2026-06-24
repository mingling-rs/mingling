use std::marker::PhantomData;

use mingling_core::{
    ProgramCollect,
    hook::{ProgramControlUnit, ProgramHook},
    setup::ProgramSetup,
    this,
};

use crate::res::ResExitCode;

/// Provides the ability to control the program's exit code, which is returned when the program ends.
///
/// - Use `mingling::update_exit_code` to update the exit code.
/// - Use `mingling::current_exit_code` to query the current exit code.
pub struct ExitCodeSetup<C> {
    _collect: PhantomData<C>,
}

impl<C> Default for ExitCodeSetup<C>
where
    C: ProgramCollect<Enum = C> + 'static,
{
    fn default() -> Self {
        Self {
            _collect: PhantomData,
        }
    }
}

impl<C> ProgramSetup<C> for ExitCodeSetup<C>
where
    C: ProgramCollect<Enum = C> + 'static,
{
    fn setup(self, program: &mut crate::Program<C>) {
        // Insert resource
        program.with_resource(ResExitCode { exit_code: 0 });

        // Insert hook to override exit code before program ends
        program.with_hook(ProgramHook::empty().on_finish(|_| {
            let this = this::<C>().res_or_default::<ResExitCode>();
            ProgramControlUnit::OverrideExitCode(this.exit_code)
        }));
    }
}
