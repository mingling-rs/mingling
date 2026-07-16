//! Mingling path matching patterns for command routing and field mapping.

pub use basic_struct::*;
pub use chain::*;
pub use completion::*;
pub use dispatcher::*;
pub use dispatcher_clap::*;
pub use group::*;
pub use groupped_derive::*;
pub use help::*;
pub use pack::*;
pub use renderer::*;

mod basic_struct;
mod chain;
mod completion;
mod dispatcher;
mod dispatcher_clap;
mod group;
mod groupped_derive;
mod help;
mod pack;
mod renderer;
