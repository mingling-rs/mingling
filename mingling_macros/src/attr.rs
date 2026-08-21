// Doc Not Optimize
pub(crate) mod chain;
pub(crate) mod command;
#[cfg(feature = "comp")]
pub(crate) mod completion;
#[cfg(feature = "clap")]
pub(crate) mod dispatcher_clap;
pub(crate) mod help;
pub(crate) mod metadata;
pub(crate) mod mlint;
pub(crate) mod program_setup;
pub(crate) mod renderer;
