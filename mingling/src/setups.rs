mod basic;
pub use basic::*;

mod dirs;
pub use dirs::*;

mod exit_code;
pub use exit_code::*;

/// Picker's `ProgramSetup` variant.
///
/// Internally does not use its own argument parsing,
/// but relies on `arg_picker`'s argument parsing capability.
#[cfg(feature = "picker")]
pub mod picker;

#[cfg(feature = "structural_renderer")]
mod structural_renderer;

#[cfg(feature = "structural_renderer")]
pub use structural_renderer::*;

#[cfg(feature = "repl")]
mod repl_basic;

#[cfg(feature = "repl")]
pub use repl_basic::*;
