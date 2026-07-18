/// Provides the specific parsing logic for command-line arguments and common utilities,
/// as well as customization of command-line argument styles.
pub mod parselib {
    pub use arg_picker::parselib::*;
}

pub use arg_picker::*;

mod entry_picker;
pub use entry_picker::*;

mod global;
pub use global::*;
