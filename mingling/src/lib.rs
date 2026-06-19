//! Mingling
//!
//! # Intro
//! [`Mingling`](https://github.com/mingling-rs/mingling) is a **proc-macro and type system-based** Rust CLI framework, suitable for developing complex command-line programs with numerous subcommands.
//!
//! # Use
//! Here is a basic project written using **Mingling**:
//! - When the user types `greet`, the program outputs `Hello, World!`
//! - When the user types `greet Alice`, the program outputs `Hello, Alice!`
//!
//! ```rust
//! use mingling::prelude::*;
//!
//! dispatcher!("greet", CMDGreet => EntryGreet);
//!
//! fn main() {
//!     let mut program = ThisProgram::new();
//!     program.with_dispatcher(CMDGreet);
//!     program.exec_and_exit();
//! }
//!
//! pack!(ResultName = String);
//!
//! #[chain]
//! fn handle_greet(args: EntryGreet) -> Next {
//!     let name: ResultName = args
//!         .inner
//!         .first()
//!         .cloned()
//!         .unwrap_or_else(|| "World".to_string())
//!         .into();
//!     name
//! }
//!
//! #[renderer]
//! fn render_name(name: ResultName) {
//!     r_println!("Hello, {}!", *name);
//! }
//!
//! #[renderer]
//! fn render_dispatcher_not_found(err: ErrorDispatcherNotFound) {
//!     if err.len() > 0 {
//!         r_println!("Command not found: [{}]", err.join(" "));
//!     }
//! }
//!
//! gen_program!();
//! ```
//!
// Output:
//!
//! ```text
//! > mycmd greet
//! Hello, World!
//! > mycmd greet Alice
//! Hello, Alice!
//! > mycmd great
//! Command not found: [great]
//! ```
//!
//! # Examples
//! `Mingling` provides detailed usage examples for your reference.
//! See [Examples](_mingling_examples/index.html)

mod example_docs;

// Re-export Core lib
pub use mingling::*;
pub use mingling_core as mingling;

/// `Mingling` argument parser
#[cfg(feature = "parser")]
pub mod parser;

/// Re-export from `mingling_macros`
#[allow(unused_imports)]
pub mod macros {
    /// Used to generate a struct implementing the `Chain` trait via a method
    pub use mingling_macros::chain;
    /// Used to generate completion entry
    #[cfg(feature = "comp")]
    pub use mingling_macros::completion;
    /// Used to create a dispatcher that routes to a `Chain`
    pub use mingling_macros::dispatcher;
    /// Used to create a dispatcher with clap argument parsing
    #[cfg(feature = "clap")]
    pub use mingling_macros::dispatcher_clap;
    /// Used to create an empty result value for early return from a chain function
    #[cfg(feature = "extra_macros")]
    pub use mingling_macros::empty_result;
    /// Creates a packed entry value from a list of string literals
    #[cfg(feature = "extra_macros")]
    pub use mingling_macros::entry;
    /// Used to collect data and create a command-line context
    pub use mingling_macros::gen_program;
    /// Used to register an external type as a group member
    #[cfg(feature = "extra_macros")]
    pub use mingling_macros::group;
    /// Used to generate a struct implementing the `HelpRequest` trait via a method
    pub use mingling_macros::help;
    /// Used to create a `Node` struct via a literal
    pub use mingling_macros::node;
    /// Used to create a wrapper type for use with `Chain` and `Renderer`
    pub use mingling_macros::pack;
    /// Used to create an error struct with automatic `name` field
    #[cfg(feature = "extra_macros")]
    pub use mingling_macros::pack_err;
    #[cfg(feature = "comp")]
    /// Internal macro for '`gen_program`' used to finally generate the completion structure
    pub use mingling_macros::program_comp_gen;
    /// Internal macro for '`gen_program`' used to finally generate the fallback
    pub use mingling_macros::program_fallback_gen;
    /// Internal macro for '`gen_program`' used to finally generate the program
    pub use mingling_macros::program_final_gen;
    /// Used to generate program setup
    #[cfg(feature = "extra_macros")]
    pub use mingling_macros::program_setup;
    /// Used to print content within a `Renderer` context
    pub use mingling_macros::r_print;
    /// Used to print content with a newline within a `Renderer` context
    pub use mingling_macros::r_println;
    /// Used to register a chain
    pub use mingling_macros::register_chain;
    /// Used to register a dispatcher for `dispatch_tree` feature
    pub use mingling_macros::register_dispatcher;
    /// Used to register a help
    pub use mingling_macros::register_help;
    /// Used to register a renderer
    pub use mingling_macros::register_renderer;
    /// Used to register a type into the context
    pub use mingling_macros::register_type;
    /// Used to generate a struct implementing the `Renderer` trait via a method
    pub use mingling_macros::renderer;
    /// Used to generate a route that either returns a successful result or early returns an error.
    #[cfg(feature = "extra_macros")]
    pub use mingling_macros::route;
    #[cfg(feature = "comp")]
    /// Used to generate suggestions
    pub use mingling_macros::suggest;
    #[cfg(feature = "comp")]
    /// Used to generate enum suggestions
    pub use mingling_macros::suggest_enum;
}

/// derive macro `EnumTag`
pub use mingling_macros::EnumTag;

/// derive macro Groupped
pub use mingling_macros::Groupped;

/// Example projects for `Mingling`, for learning how to use `Mingling`
pub mod _mingling_examples {
    pub use crate::example_docs::*;
}

mod features;

/// Module for checking which features are enabled at compile time.
///
/// Each constant re-exported from this module corresponds to a Cargo feature flag.
/// They can be used for conditional compilation or runtime branching based on
/// feature availability.
pub mod feature {
    include!("./features.rs");
}

mod setups;

/// Setups provided by Mingling, which can extend command-line programs.
pub mod setup {
    pub use crate::setups::*;
    pub use mingling_core::setup::*;
}

/// Mutable global resources provided within Mingling
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
    /// Re-export of the `Groupped` derive macro for grouping types.
    pub use crate::Groupped;
    /// Re-export of the `chain` macro for defining a chain of commands.
    pub use crate::macros::chain;
    /// Re-export of the `dispatcher` macro for routing commands.
    pub use crate::macros::dispatcher;
    /// Re-export of the `empty_result` macro for creating an empty result value for early return.
    #[cfg(feature = "extra_macros")]
    pub use crate::macros::empty_result;
    /// Re-export of the `gen_program` macro for generating the program entry point.
    pub use crate::macros::gen_program;
    /// Re-export of the `pack` macro for creating wrapper types.
    pub use crate::macros::pack;
    /// Re-export of the `pack_err` macro for creating error types.
    #[cfg(feature = "extra_macros")]
    pub use crate::macros::pack_err;
    /// Re-export of the `r_print` macro for printing within a renderer context.
    pub use crate::macros::r_print;
    /// Re-export of the `r_println` macro for printing with a newline within a renderer
    /// context.
    pub use crate::macros::r_println;
    /// Re-export of the `renderer` macro for defining renderer functions.
    pub use crate::macros::renderer;

    /// Re-export of the `completion` macro for generating completion entries.
    #[cfg(feature = "comp")]
    pub use crate::macros::completion;

    /// Re-export of the `AsPicker` trait for picker functionality.
    #[cfg(feature = "parser")]
    pub use crate::parser::AsPicker;
}
