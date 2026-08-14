// Doc Not Optimize
use crate::{AnyOutput, ProgramCollect, RenderResult};

/// Represents the data passed to `begin` hook.
pub struct HookBeginInfo {}

/// Represents the data passed to `pre_dispatch` hook.
pub struct HookPreDispatchInfo<'a> {
    /// Arguments entered by the user before dispatching
    ///
    /// The reference is mutable so the hook can rewrite the arguments before
    /// they are matched against the registered dispatchers.
    pub arguments: &'a mut Vec<String>,
}

/// Represents the data passed to `post_dispatch` hook.
pub struct HookPostDispatchInfo<'a, C>
where
    C: ProgramCollect<Enum = C>,
{
    /// The entry point of dispatching
    pub entry: &'a C,
}

/// Represents the data passed to `pre_chain` hook.
pub struct HookPreChainInfo<'a, C>
where
    C: ProgramCollect<Enum = C>,
{
    /// Input to the chain
    pub input: &'a C,

    /// Raw data
    pub raw: &'a dyn std::any::Any,
}

/// Represents the data passed to `post_chain` hook.
pub struct HookPostChainInfo<'a, C>
where
    C: ProgramCollect<Enum = C>,
{
    /// Output of the chain
    pub output: &'a AnyOutput<C>,
}

/// Represents the data passed to `pre_render` hook.
pub struct HookPreRenderInfo<'a, C>
where
    C: ProgramCollect<Enum = C>,
{
    /// Render input
    pub input: &'a C,

    /// The raw data to be rendered
    pub raw: &'a dyn std::any::Any,
}

/// Represents the data passed to `post_render` hook.
pub struct HookPostRenderInfo<'a> {
    /// The rendering result
    pub result: &'a RenderResult,
}

/// Represents the data passed to `finish` hook.
pub struct HookFinishInfo {}

/// Represents the data passed to `exec_panic` hook.
#[cfg(not(feature = "async"))]
pub struct HookPanicInfo<'a> {
    /// Raw data of the panic
    pub panic: &'a crate::error::ProgramPanic,
}

#[cfg(feature = "repl")]
mod repl_hook {
    use crate::RenderResult;

    /// Represents the data passed to `repl_on_begin` hook.
    pub struct HookREPLBeginInfo {}

    /// Represents the data passed to `repl_pre_readline` hook.
    pub struct HookREPLPreReadlineInfo {}

    /// Represents the data passed to `repl_readline` hook.
    pub struct HookREPLReadlineInfo {}

    /// Represents the data passed to `repl_post_readline` hook.
    pub struct HookREPLPostReadlineInfo<'a> {
        /// The read line (mutable for editing)
        pub line: &'a mut String,
    }

    /// Represents the data passed to `repl_pre_exec` hook.
    pub struct HookREPLPreExecInfo<'a> {
        /// Arguments for the command
        pub args: &'a [String],
    }

    /// Represents the data passed to `repl_post_exec` hook.
    pub struct HookREPLPostExecInfo {}

    /// Represents the data passed to `repl_on_receive_result` hook.
    pub struct HookREPLOnReceiveResultInfo<'a> {
        /// The rendering result
        pub result: &'a RenderResult,
    }

    /// Represents the data passed to `repl_exit` hook.
    pub struct HookREPLExitInfo {}

    /// Represents the data passed to `repl_loop_once` hook.
    pub struct HookREPLLoopOnceInfo {}

    /// Represents the data passed to `repl_on_panic` hook.
    #[cfg(not(feature = "async"))]
    pub struct HookREPLOnPanicInfo<'a> {
        /// Raw data of the panic
        pub panic: &'a crate::error::ProgramPanic,
    }
}

#[cfg(feature = "repl")]
pub use repl_hook::*;
