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
//! fn render_name(name: ResultName) -> RenderResult {
//!     let mut result = RenderResult::default();
//!     result.println(&format!("Hello, {}!", *name));
//!     result
//! }
//!
//! #[renderer]
//! fn render_dispatcher_not_found(err: ErrorDispatcherNotFound) -> RenderResult {
//!     let mut result = RenderResult::default();
//!     if err.len() > 0 {
//!         result.println(&format!("Command not found: [{}]", err.join(" ")));
//!     }
//!     result
//! }
//!
//! gen_program!();
//! ```
//!
//! Output:
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
//! See [Examples](_mingling_examples/index.html) or [Helpdoc](https://mingling-rs.github.io/mingling/docs/examples.html)
//!
//! # About Features
//! All features of `Mingling` are opt-in. To learn what each feature provides, see [Features](feature/index.html) or [Helpdoc](https://mingling-rs.github.io/mingling/docs/doc.html#/pages/other/features)

mod example_docs;

// Re-export Core lib
pub use mingling::*;
pub use mingling_core as mingling;

/// `Mingling` argument parser (Built-in)
#[cfg(feature = "parser")]
pub mod parser;

/// `Mingling` argument parser (Picker2)
#[cfg(feature = "picker")]
pub mod picker {
    pub use mingling_picker::*;
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
pub mod macros {
    /// `#[chain]` - Used to generate a struct implementing the `Chain` trait via a method
    pub use mingling_macros::chain;
    /// `#[completion(EntryType)]` - Used to generate completion entry
    #[cfg(feature = "comp")]
    pub use mingling_macros::completion;
    /// `dispatcher!("greet", CMDGreet => EntryGreet)` - Used to create a dispatcher that routes to a `Chain`
    pub use mingling_macros::dispatcher;
    /// `#[dispatcher_clap("greet", CMDGreet)]` - Used to create a dispatcher with clap argument parsing
    #[cfg(feature = "clap")]
    pub use mingling_macros::dispatcher_clap;
    /// `empty_result!()` - Used to create an empty result value for early return from a chain function
    #[cfg(feature = "extra_macros")]
    pub use mingling_macros::empty_result;
    /// `entry!["--greet", "Alice"]` - Creates a packed entry value from a list of string literals
    #[cfg(feature = "extra_macros")]
    pub use mingling_macros::entry;
    /// `gen_program!()` - Used to collect data and create a command-line context
    pub use mingling_macros::gen_program;
    /// `group!(ErrorIo = std::io::Error)` - Used to register an external type as a group member
    #[cfg(feature = "extra_macros")]
    pub use mingling_macros::group;
    /// `group_structural!(ErrorIo = std::io::Error)` - Like `group!` but also marks the type for structured output
    #[cfg(all(feature = "structural_renderer", feature = "extra_macros"))]
    pub use mingling_macros::group_structural;
    /// `#[help]` - Used to generate a struct implementing the `HelpRequest` trait via a method
    pub use mingling_macros::help;
    /// `node!("remote.rm")` - Used to create a `Node` struct via a literal
    pub use mingling_macros::node;
    /// `pack!(StateGreet = String)` - Used to create a wrapper type for use with `Chain` and `Renderer`
    pub use mingling_macros::pack;
    /// `pack_err!(ErrorUnknown)` - Used to create an error struct with automatic `name` field
    #[cfg(feature = "extra_macros")]
    pub use mingling_macros::pack_err;
    /// `pack_err_structural!(ErrorUnknown)` - Like `pack_err!` but also marks the type for structured output
    #[cfg(all(feature = "structural_renderer", feature = "extra_macros"))]
    pub use mingling_macros::pack_err_structural;
    /// `pack_structural!(StateGreet = String)` - Like `pack!` but also marks the type for structured output
    #[cfg(feature = "structural_renderer")]
    pub use mingling_macros::pack_structural;
    #[cfg(feature = "comp")]
    #[doc(hidden)]
    pub use mingling_macros::program_comp_gen;
    #[doc(hidden)]
    pub use mingling_macros::program_fallback_gen;
    #[doc(hidden)]
    pub use mingling_macros::program_final_gen;
    /// `#[program_setup]` - Used to generate program setup
    #[cfg(feature = "extra_macros")]
    pub use mingling_macros::program_setup;
    #[doc(hidden)]
    pub use mingling_macros::register_chain;
    #[doc(hidden)]
    pub use mingling_macros::register_dispatcher;
    #[doc(hidden)]
    pub use mingling_macros::register_help;
    #[doc(hidden)]
    pub use mingling_macros::register_renderer;
    #[doc(hidden)]
    pub use mingling_macros::register_type;
    /// `#[renderer]` - Used to generate a struct implementing the `Renderer` trait via a method
    pub use mingling_macros::renderer;
    /// `route! { /* ... */ }` - Used to generate a route that either returns a successful result or early returns an error.
    #[cfg(feature = "extra_macros")]
    pub use mingling_macros::route;
    /// `suggest! { "hello", "bye" }` - Used to generate suggestions
    #[cfg(feature = "comp")]
    pub use mingling_macros::suggest;
    /// `suggest_enum!(EnumNames)` - Used to generate enum suggestions
    #[cfg(feature = "comp")]
    pub use mingling_macros::suggest_enum;

    #[cfg(feature = "picker")]
    pub use mingling_macros::*;
}

/// derive macro `EnumTag`
pub use mingling_macros::EnumTag;

/// derive macro Groupped
pub use mingling_macros::Groupped;

/// derive macro `StructuralData` — marks a type as supporting structured output
#[cfg(feature = "structural_renderer")]
pub use mingling_macros::StructuralData;

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
    /// Re-export of the `renderer` macro for defining renderer functions.
    pub use crate::macros::renderer;
    /// Re-export of the `Groupped` derive macro for grouping types.
    pub use crate::Groupped;
    /// Re-export of the `RenderResult` struct for outputting rendering result
    pub use crate::RenderResult;
    /// Like `pack_err!` but also marks the type for structured output
    #[cfg(all(feature = "structural_renderer", feature = "extra_macros"))]
    pub use mingling_macros::pack_err_structural;
    /// Like `pack!` but also marks the type for structured output
    #[cfg(feature = "structural_renderer")]
    pub use mingling_macros::pack_structural;

    /// Re-export of the `completion` macro for generating completion entries.
    #[cfg(feature = "comp")]
    pub use crate::macros::completion;

    /// Re-export of the `AsPicker` trait for picker functionality.
    #[cfg(feature = "parser")]
    pub use crate::parser::AsPicker;

    #[cfg(feature = "picker")]
    pub use mingling_picker::prelude::*;
}
