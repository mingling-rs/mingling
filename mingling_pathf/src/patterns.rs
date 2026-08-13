// Doc Not Optimize
//! Mingling path matching patterns for command routing and field mapping.

pub use chain::*;
pub use command::*;
pub use completion::*;
pub use dispatcher::*;
pub use dispatcher_clap::*;
pub use group::*;
pub use grouped_derive::*;
pub use help::*;
pub use metadata::*;
pub use pack::*;
pub use renderer::*;

mod chain;
mod command;
mod completion;
mod dispatcher;
mod dispatcher_clap;
mod group;
mod grouped_derive;
mod help;
mod metadata;
mod pack;
mod renderer;
