// Doc Not Optimize
use mingling_core::{ProgramCollect, this};

/// Represents a program exit code.
///
/// This struct should be used together with `ExitCodeSetup`:
/// ```ignore
/// program.with_setup(ExitCodeSetup::default());
/// ```
/// The exit code is stored globally per `ProgramCollect` type and can be
/// retrieved via [`exit_code()`] or updated via [`update_exit_code()`].
#[derive(Debug, Default, Clone, Copy)]
pub struct ResExitCode {
    /// The numeric exit code value.
    pub exit_code: i32,
}

/// Updates the globally stored exit code for the given `ProgramCollect` type.
pub fn update_exit_code<C>(exit_code: i32)
where
    C: ProgramCollect<Enum = C> + 'static,
{
    this::<C>().modify_res(|e: &mut ResExitCode| e.exit_code = exit_code);
}

/// Retrieves the globally stored exit code for the given `ProgramCollect` type.
/// Returns `0` if no exit code has been set.
#[must_use]
pub fn exit_code<C>() -> i32
where
    C: ProgramCollect<Enum = C> + 'static,
{
    match this::<C>().res::<ResExitCode>() {
        Some(e) => e.exit_code,
        None => 0,
    }
}
