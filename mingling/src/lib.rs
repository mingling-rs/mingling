#![doc(html_logo_url = "https://github.com/mingling-rs/mingling/raw/main/docs/res/icon3.png")]
#![doc(
    html_favicon_url = "https://github.com/mingling-rs/mingling/raw/main/docs/res/favicon_small.png"
)]
#![deny(missing_docs)]
#![doc = include_str!("docs/lib.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(all(feature = "core", feature = "docs_rs"))]
mod gen_program;

#[cfg(all(feature = "core", feature = "docs_rs"))]
#[doc = include_str!("docs/gen_program.md")]
#[allow(nonstandard_style)]
pub mod CRATE_ROOT {
    pub use crate::gen_program::*;
}

#[cfg(feature = "core")]
mod example_docs;

// Re-export Core lib
#[cfg(feature = "core")]
pub use mingling::*;

#[cfg(feature = "core")]
pub use mingling_core as mingling;

/// `Mingling` argument parser (Built-in)
#[cfg(feature = "parser")]
pub mod parser;

/// `Mingling` argument parser (Picker2)
#[cfg(feature = "picker")]
pub mod picker;

mod constants;

/// Constants used throughout the Mingling framework.
pub mod consts {
    pub use crate::constants::*;
}

/// Re-export of all macros from `mingling_macros`.
///
/// This module re-exports all macros provided by the `mingling_macros` crate,
/// including `dispatcher!`, `chain!`, `renderer!`,
/// `gen_program!`, `pack!`, and many others. These macros form the core
/// building blocks of the Mingling framework.
///
/// For detailed documentation, usage examples, and the full list of available
/// macros, please refer to the `mingling_macros` crate [documentation](https://docs.rs/mingling_macros/latest/mingling_macros/):
///
/// <https://docs.rs/mingling_macros/latest/mingling_macros/>
#[allow(unused_imports)]
#[cfg(feature = "macros")]
pub mod macros {
    #[cfg(feature = "picker")]
    pub use arg_picker::macros::*;
    pub use mingling_macros::buffer;
    pub use mingling_macros::chain;
    #[cfg(feature = "extras")]
    pub use mingling_macros::command;
    #[cfg(feature = "comp")]
    pub use mingling_macros::completion;
    pub use mingling_macros::dispatcher;
    #[cfg(feature = "clap")]
    pub use mingling_macros::dispatcher_clap;
    #[cfg(feature = "extras")]
    pub use mingling_macros::empty_result;
    #[cfg(feature = "extras")]
    pub use mingling_macros::entry;
    pub use mingling_macros::gen_program;
    #[cfg(feature = "extras")]
    pub use mingling_macros::group;
    #[cfg(all(feature = "structural_renderer", feature = "extras"))]
    pub use mingling_macros::group_structural;
    pub use mingling_macros::help;
    pub use mingling_macros::metadata;
    pub use mingling_macros::mlint;
    pub use mingling_macros::node;
    pub use mingling_macros::pack;
    #[cfg(feature = "extras")]
    pub use mingling_macros::pack_err;
    #[cfg(all(feature = "structural_renderer", feature = "extras"))]
    pub use mingling_macros::pack_err_structural;
    #[cfg(feature = "structural_renderer")]
    pub use mingling_macros::pack_structural;
    #[cfg(feature = "comp")]
    #[doc(hidden)]
    pub use mingling_macros::program_comp_gen;
    #[doc(hidden)]
    pub use mingling_macros::program_fallback_gen;
    #[doc(hidden)]
    pub use mingling_macros::program_final_gen;
    #[cfg(feature = "extras")]
    pub use mingling_macros::program_setup;
    pub use mingling_macros::r_append;
    pub use mingling_macros::r_eprint;
    pub use mingling_macros::r_eprintln;
    pub use mingling_macros::r_print;
    pub use mingling_macros::r_println;
    #[doc(hidden)]
    pub use mingling_macros::register_chain;
    #[doc(hidden)]
    pub use mingling_macros::register_dispatcher;
    #[doc(hidden)]
    pub use mingling_macros::register_help;
    #[doc(hidden)]
    pub use mingling_macros::register_metadata;
    #[doc(hidden)]
    pub use mingling_macros::register_renderer;
    #[doc(hidden)]
    pub use mingling_macros::register_type;
    #[cfg(feature = "extras")]
    pub use mingling_macros::render_route;
    pub use mingling_macros::renderer;
    #[cfg(feature = "extras")]
    pub use mingling_macros::renderify;
    #[cfg(feature = "extras")]
    pub use mingling_macros::route;
    #[cfg(feature = "extras")]
    pub use mingling_macros::routeify;
    #[cfg(feature = "comp")]
    pub use mingling_macros::suggest;
    #[cfg(feature = "comp")]
    pub use mingling_macros::suggest_enum;
}

#[cfg(feature = "macros")]
pub use mingling_macros::EnumTag;

#[cfg(feature = "macros")]
pub use mingling_macros::Grouped;

#[cfg(feature = "structural_renderer")]
pub use mingling_macros::StructuralData;

#[doc = include_str!("docs/docsrs_examples.md")]
#[cfg(all(feature = "core", feature = "docs_rs"))]
#[allow(nonstandard_style)]
pub mod EXAMPLES {
    pub use crate::example_docs::*;
}

#[cfg(feature = "core")]
mod features;

/// Module for checking which features are enabled at compile time.
///
/// Each constant re-exported from this module corresponds to a Cargo feature flag.
/// They can be used for conditional compilation or runtime branching based on
/// feature availability.
#[cfg(feature = "core")]
pub mod feature {
    include!("./features.rs");
}

#[cfg(feature = "core")]
mod setups;

/// Setups provided by Mingling, which can extend command-line programs.
#[cfg(feature = "core")]
pub mod setup {
    pub use crate::setups::*;
    pub use mingling_core::setup::*;
}

/// Mutable global resources provided within Mingling
#[cfg(feature = "core")]
pub mod res;

/// The prelude module provides convenient re-exports of commonly used macros and traits.
///
/// Importing this module brings the essential components of Mingling into scope,
/// reducing boilerplate when defining commands, dispatchers, renderers, and the
/// program entry point.
///
/// # Examples
///
/// ```rust
/// use mingling::prelude::*;
/// ```
pub mod prelude {
    #[cfg(feature = "core")]
    pub use crate::Grouped;
    #[cfg(feature = "core")]
    pub use crate::RenderResult;
    #[cfg(feature = "core")]
    pub use crate::Routable;
    #[cfg(feature = "macros")]
    pub use crate::macros::chain;
    #[cfg(all(feature = "extras", feature = "macros"))]
    pub use crate::macros::command;
    #[cfg(feature = "macros")]
    pub use crate::macros::dispatcher;
    #[cfg(all(feature = "extras", feature = "macros"))]
    pub use crate::macros::empty_result;
    #[cfg(feature = "macros")]
    pub use crate::macros::gen_program;
    #[cfg(feature = "macros")]
    pub use crate::macros::pack;
    #[cfg(all(feature = "extras", feature = "macros"))]
    pub use crate::macros::pack_err;
    #[cfg(feature = "macros")]
    pub use crate::macros::renderer;
    #[cfg(all(
        feature = "macros",
        feature = "structural_renderer",
        feature = "extras"
    ))]
    pub use mingling_macros::pack_err_structural;
    #[cfg(all(feature = "macros", feature = "structural_renderer"))]
    pub use mingling_macros::pack_structural;
    pub use mingling_macros::r_append;
    pub use mingling_macros::r_eprint;
    pub use mingling_macros::r_eprintln;
    pub use mingling_macros::r_print;
    pub use mingling_macros::r_println;

    #[cfg(all(feature = "macros", feature = "comp"))]
    pub use crate::macros::completion;

    #[cfg(feature = "parser")]
    pub use crate::parser::AsPicker;

    #[cfg(feature = "picker")]
    pub use arg_picker::prelude::arg;

    #[cfg(feature = "picker")]
    pub use crate::picker::EntryPicker;
}
