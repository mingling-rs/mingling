use mingling_core::{
    Program, ProgramCollect, config, hook::ProgramHook, setup::ProgramSetup, this,
};

use crate::res::ResConfirm;

/// Confirm setup for managing confirmation state
///
/// This Setup manages the confirmation flag within the program's resource
/// store. It registers a [`ResConfirm`] resource and sets up a hook that
/// checks the user's confirmation mode during program execution.
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
/// use mingling::setup::ConfirmSetup;
///
/// let mut program = Program::<ThisProgram>::new();
/// program.with_setup(ConfirmSetup);
/// ```
///
/// # Behavior
///
/// - Registers a [`ResConfirm`] resource that tracks confirmation state.
/// - At the beginning of command execution, checks whether the user's
///   confirmation mode is set to `Skip`.
/// - If confirmation is skipped, the [`Confirm`] resource is updated
///   to record the confirmed state.
///
/// # Notes
///
/// - This Setup applies uniformly to all subcommands of the entire program.
/// - The confirmation state is determined by the global `config` setting;
///   it does not support per-command overrides.
pub struct ConfirmSetup;

impl<C> ProgramSetup<C> for ConfirmSetup
where
    C: ProgramCollect<Enum = C> + 'static,
{
    fn setup(self, program: &mut Program<C>) {
        program.with_resource(ResConfirm::new());

        program.with_hook(ProgramHook::empty().on_pre_dispatch::<_, ()>(|_| {
            let p = this::<C>();
            let confirmed = p.user_context.confirmation == config::ConfirmationMode::Skip;
            if confirmed {
                p.modify_res(|c: &mut ResConfirm| {
                    c.set_confirmed();
                });
            }
        }));
    }
}
