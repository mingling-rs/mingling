/// Constant defining the name of the completion subcommand.
///
/// When a user invokes this subcommand (e.g., `your_program __comp`), the
/// program enters completion mode and generates shell completions based on
/// the current shell context.
///
/// This value is used internally by the completion system to intercept the
/// command-line input and redirect to the completion handler.
///
/// ```
/// # #[cfg(feature = "comp")] {
/// # use mingling_core::constants::COMPLETION_SUBCOMMAND;
/// assert_eq!("__comp", COMPLETION_SUBCOMMAND);
/// # }
/// ```
#[cfg(feature = "comp")]
pub const COMPLETION_SUBCOMMAND: &str = "__comp";
