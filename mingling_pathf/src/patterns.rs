//! Mingling path matching patterns for command routing and field mapping.

pub use chain::*;
pub use command::*;
pub use completion::*;
pub use dispatcher::*;
pub use dispatcher_clap::*;
pub use grouped_derive::*;
pub use help::*;
pub use import_type::*;
pub use metadata::*;
pub use renderer::*;
pub use structural::*;

mod chain;
mod command;
mod completion;
mod dispatcher;
mod dispatcher_clap;
mod grouped_derive;
mod help;
mod import_type;
mod metadata;
mod renderer;
mod structural;
