#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
// Some code requires wildcard imports to reduce boilerplate code
#![allow(clippy::wildcard_imports)]

mod builtin;

mod picker;
pub use picker::*;

mod pickable;
pub use pickable::*;

mod arg;
pub use arg::*;

mod infos;
pub use infos::*;

/// Provides the specific parsing logic for command-line arguments and common utilities,
/// as well as customization of command-line argument styles.
pub mod parselib;

/// Parser-provided parseable command-line types
pub mod value;

/// The prelude module, which re-exports the most commonly used traits and types.
///
/// This module is intended to be imported with a wildcard import:
///
/// ```
/// use arg_picker::prelude::*;
/// ```
pub mod prelude {
    pub use crate::IntoPicker;
    pub use crate::macros::arg;
}

/// Re-export of the `arg_picker_macros` crate
pub mod macros {
    pub use arg_picker_macros::arg;
}

/// Provides the types necessary for implementing the `Pickable` trait
pub mod pickable_needed {
    pub use crate::{Pickable, PickerArg, PickerArgAttr, PickerArgResult, TagPhaseContext};
}

/// Provides the types necessary for implementing the `Matcher` trait
pub mod matcher_needed {
    pub use crate::PickerArgInfo;
    pub use crate::parselib::{MaskedArg, Matcher, ParserStyle};
}

mod constants;

/// Re-export of constants used by `arg-picker`.
///
/// This module provides access to various constants defined internally, such as
/// default values, configuration limits, and other static parameters.
pub mod consts {
    pub use crate::constants::*;
}
