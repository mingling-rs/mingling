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

#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
// Using `"".to_string()` is clearer than `String::new()` for expressing "creating an empty value"
#![allow(clippy::manual_string_new)]

// Private Modules
mod any;
mod asset;
mod program;
mod renderer;
mod tester;

/// Module for setting up a `Mingling` program.
///
/// This module provides the [`ProgramSetup`](https://docs.rs/mingling/latest/mingling/setup/trait.ProgramSetup.html) type, which allows users to configure
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

// Public Modules

/// Provides a toolkit for `Mingling` testing capabilities.
pub mod test {
    pub use crate::tester::*;
}

/// Provided for framework developers
mod debug;

// The `StructuralData` trait is sealed and only implementable through the
// derive macro, but the trait itself is publicly accessible at the top level
// (`mingling::StructuralData`), where it coexists with the same-named derive
// macro (trait in the type namespace, derive in the macro namespace).
#[cfg(feature = "structural_renderer")]
pub use crate::renderer::structural::StructuralData;
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
pub use crate::asset::metadata::*;
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
}

/// Mingling's convention metadatas, which can be bound to types using `#[metadata]`, to provide identification for types
pub mod metadata;

/// Some common utilities in Mingling, providing a collection of functionality needed by many modules.
pub mod utils;
