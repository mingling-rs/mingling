//! Mingling Core
//!
//! # Intro
//! This crate is the core implementation of `mingling`, containing the complete logic for command dispatching, execution, and rendering.
//!
//! # Note
//! It is not recommended to use [mingling_core](https://crates.io/crates/mingling_core) directly, as this will lose the code generation functionality of [mingling_macros](https://crates.io/crates/mingling_macros).
//!
//! Recommended to import [mingling](https://crates.io/crates/mingling) to use its features.

mod any;
mod asset;
mod program;
mod renderer;
mod tester;

/// Provides a toolkit for `Mingling` testing capabilities.
pub mod test {
    pub use crate::tester::*;
}

#[cfg(feature = "structural_renderer")]
pub use crate::renderer::structural::StructuralRenderer;

// NOT re-exported at top level: the `StructuralData` trait is sealed and only
// accessible through the derive macro. Users who need the trait can access it
// via `mingling::renderer::structural::StructuralData` (through the inner alias).
pub use crate::any::group::*;
pub use crate::any::*;

pub use crate::asset::chain::*;
pub use crate::asset::dispatcher::*;
pub use crate::asset::enum_tag::*;
pub use crate::asset::global_resource::*;
pub use crate::asset::help::*;
pub use crate::asset::lazy_resource::*;
pub use crate::asset::node::*;
pub use crate::asset::renderer::*;

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

pub use crate::program::*;

pub use crate::renderer::render_result::*;

#[cfg(feature = "builds")]
#[doc(hidden)]
pub mod builds;

/// Provides build scripts for users
#[cfg(feature = "builds")]
pub mod build {
    #[cfg(feature = "comp")]
    pub use crate::builds::comp::*;
}

/// Provided for framework developers
pub mod debug;

#[cfg(feature = "comp")]
#[doc(hidden)]
pub mod comp;

#[cfg(feature = "comp")]
pub use crate::comp::*;

pub mod setup {
    pub use crate::program::setup::ProgramSetup;
}

/// Private API — not intended for direct use.
#[doc(hidden)]
pub mod __private {
    /// Sealed trait for `StructuralData` — only implementable via derive macro.
    pub trait StructuralDataSealed {}

    /// Re-export so the derive macro can reference the trait without
    /// conflicting with the derive macro name at `::mingling::StructuralData`.
    #[cfg(feature = "structural_renderer")]
    pub use crate::renderer::structural::structural_data::StructuralData;
}

#[doc(hidden)]
pub mod core_res {
    #[cfg(feature = "repl")]
    pub use crate::program::repl_exec::res::ResREPL;
}

#[cfg(feature = "pathf")]
pub mod pathf {
    pub use mingling_pathf::config::*;
    pub use mingling_pathf::module_pathf::*;
    pub use mingling_pathf::pattern_analyzer::*;
    pub use mingling_pathf::patterns::*;

    pub use mingling_pathf::analyze_and_build_type_mapping;

    use std::path::Path;

    /// Wraps `analyze_and_build_type_mapping_for` with config derived from
    /// the crate's feature flags (e.g., `dispatch_tree`).
    pub fn analyze_and_build_type_mapping_for(
        crate_dir: &Path,
        output_dir: &Path,
    ) -> Result<(), crate::error::MinglingPathfinderError> {
        let config = mingling_pathf::config::PathfinderConfig {
            use_dispatch_tree: cfg!(feature = "dispatch_tree"),
        };
        mingling_pathf::analyze_and_build_type_mapping_for(crate_dir, output_dir, &config)
    }
}
