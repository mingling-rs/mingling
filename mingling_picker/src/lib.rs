mod builtin;

mod picker;
pub use picker::*;

mod pickable;
pub use pickable::*;

mod arg;
pub use arg::*;

mod infos;
pub use infos::*;

pub mod parselib;

pub mod value;

pub mod prelude {
    pub use crate::IntoPicker;
}

pub mod macros {
    pub use mingling_picker_macros::*;
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

#[cfg(feature = "mingling_support")]
mod corebind;

#[allow(unused_imports)]
#[cfg(feature = "mingling_support")]
pub use corebind::*;
