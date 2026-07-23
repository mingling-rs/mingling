#![deny(missing_docs)]
//! Mingling Core
//!
//! # Intro
//! This crate is the core implementation of `mingling`, containing the complete logic for command dispatching, execution, and rendering.
//!
//! # Note
//! It is not recommended to use [mingling_core](https://crates.io/crates/mingling_core) directly, as this will lose the code generation functionality of [mingling_macros](https://crates.io/crates/mingling_macros).
//!
//! Recommended to import [mingling](https://crates.io/crates/mingling) to use its features.

// Private Modules

mod any;
mod asset;
mod program;
mod renderer;
mod tester;

/// Module for setting up a `Mingling` program.
///
/// This module provides the [`ProgramSetup`] type, which allows users to configure
/// and initialize the program's execution environment.
pub mod setup {
    pub use crate::program::setup::ProgramSetup;
}

/// This module provides result types for Mingling components.
///
/// These are re-exported at the top level via `mingling::res`.
#[doc(hidden)]
pub mod core_res {
    #[cfg(feature = "repl")]
    pub use crate::program::repl_exec::res::*;
}

/// Provides the runtime logic for Mingling's dynamic completion system.
///
/// This module contains the core functionality for the "comp" (completion) feature,
/// which enables dynamic tab-completion and input suggestion capabilities within
/// Mingling applications.
#[cfg(feature = "comp")]
pub(crate) mod comp;

/// Provides Mingling's build script module, used in `build.rs` to provide build-time behavior for certain features.
///
/// To use it, add the following to your `Cargo.toml` under `[build-dependencies]`, and enable the features
/// that require build-time behavior from the crate:
///
/// ```toml
/// [build-dependencies.mingling]
/// version = "0.3.0"
/// features = [
///     "build", // Enable it
///     "comp",  // If you need completion-related build-time behavior, enable this as well
/// ]
/// ```
#[cfg(feature = "build")]
#[doc(hidden)]
pub mod build;

// Public Modules

/// Provides a toolkit for `Mingling` testing capabilities.
pub mod test {
    pub use crate::tester::*;
}

/// Provided for framework developers
pub mod debug;

// NOT re-exported at top level: the `StructuralData` trait is sealed and only
// accessible through the derive macro. Users who need the trait can access it
// via `mingling::renderer::structural::StructuralData` (through the inner alias).
#[cfg(feature = "structural_renderer")]
pub use crate::renderer::structural::StructuralRenderer;

pub use crate::any::*;
pub use crate::asset::chain::*;
pub use crate::asset::core_invokes::*;
pub use crate::asset::dispatcher::*;
pub use crate::asset::enum_tag::*;
pub use crate::asset::global_resource::*;
pub use crate::asset::help::*;
pub use crate::asset::lazy_resource::*;
pub use crate::asset::node::*;
pub use crate::asset::renderer::*;
pub use crate::asset::routable::*;
#[cfg(feature = "comp")]
pub use crate::comp::*;
pub use crate::program::*;
pub use crate::renderer::render_result::*;

/// All error types of `Mingling`
pub mod error {
    pub use crate::asset::chain::error::*;
    pub use crate::exec::error::*;
    pub use crate::program::error::*;

    #[cfg(feature = "structural_renderer")]
    pub use crate::renderer::structural::error::*;

    #[cfg(feature = "pathf")]
    pub use mingling_pathf::error::*;
}

#[doc(hidden)]
mod private;

/// Internal API provided by Mingling Core
///
/// These are used by macros and are not exposed to users, but are still accessible externally.
#[doc(hidden)]
#[allow(unused_imports)]
pub mod __private {
    pub use crate::private::*;
}
