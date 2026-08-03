#![allow(dead_code)]

//! Program lifecycle hook system.
//!
//! The hook system lets you observe and influence the program's execution
//! at well-defined lifecycle points — before dispatch, after chain processing,
//! on finish, etc.
//!
//! # Architecture
//!
//! Hooks are registered on [`ProgramHook`] via builder methods (`.on_begin()`,
//! `.on_pre_dispatch()`, `.on_finish()`, …) and attached to the program with
//! [`Program::with_hook`].
//!
//! Each hook callback receives a structured info type (e.g. [`HookPreDispatchInfo`])
//! and returns either `()` (no-op) or a [`ProgramControlUnit`] / [`ProgramControls`]
//! to influence the execution flow (override exit code, re-route to another
//! chain/renderer/help, etc.).
//!
//! # Hook Lifecycle Order
//!
//! ```text
//! begin
//!   │
//!   ├─ pre_dispatch    ← before an entry is dispatched
//!   ├─ post_dispatch   ← after dispatching, before chain
//!   ├─ pre_chain       ← before the type enters the chain pipeline
//!   ├─ post_chain      ← after chain processing completes
//!   ├─ pre_render      ← before the type enters the renderer
//!   ├─ post_render     ← after rendering completes
//!   │
//! finish              ← before program exits
//! ```
//!
//! When the `repl` feature is enabled, additional REPL-specific hooks are
//! available (e.g. `repl_post_readline`, `repl_on_receive_result`).

use crate::{Program, ProgramCollect};

mod hook_info;
pub use hook_info::*;

mod control_unit;
pub use control_unit::*;

/// The program's lifecycle hook registry.
///
/// `ProgramHook<C>` stores optional callbacks for each supported lifecycle
/// event — from program start (`begin`) through dispatch, chaining, rendering,
/// and finally program exit (`finish`).
///
/// Hooks are constructed via the builder pattern (see methods like
/// [`on_begin`](Self::on_begin), [`on_pre_dispatch`](Self::on_pre_dispatch),
/// etc.) and attached to a [`Program`] using
/// [`Program::with_hook`].
#[derive(Default)]
#[allow(clippy::type_complexity)] // Shutup!
pub struct ProgramHook<C>
where
    C: ProgramCollect<Enum = C>,
{
    /// Executes when the program starts running
    pub begin: Option<Box<dyn Fn(&HookBeginInfo) + Send + Sync>>,

    /// Executes before the program dispatches
    pub pre_dispatch:
        Option<Box<dyn for<'a> Fn(&HookPreDispatchInfo<'a>) -> ProgramControls<C> + Send + Sync>>,

    /// Executes after the program dispatches
    pub post_dispatch: Option<
        Box<dyn for<'a> Fn(&HookPostDispatchInfo<'a, C>) -> ProgramControls<C> + Send + Sync>,
    >,

    /// Executes before the type enters the chain
    pub pre_chain:
        Option<Box<dyn for<'a> Fn(&HookPreChainInfo<'a, C>) -> ProgramControls<C> + Send + Sync>>,

    /// Executes after the chain processing for the type ends
    pub post_chain:
        Option<Box<dyn for<'a> Fn(&HookPostChainInfo<'a, C>) -> ProgramControls<C> + Send + Sync>>,

    /// Executes before the type enters the renderer
    pub pre_render:
        Option<Box<dyn for<'a> Fn(&HookPreRenderInfo<'a, C>) -> ProgramControls<C> + Send + Sync>>,

    /// Executes after the type enters the renderer
    pub post_render:
        Option<Box<dyn for<'a> Fn(&HookPostRenderInfo<'a>) -> ProgramControls<C> + Send + Sync>>,

    /// Executes before the program ends
    pub finish: Option<Box<dyn Fn(&HookFinishInfo) -> ProgramControls<C> + Send + Sync>>,

    /// Executes when the program panics
    #[cfg(not(feature = "async"))]
    pub exec_panic: Option<Box<dyn for<'a> Fn(&HookPanicInfo<'a>) + Send + Sync>>,

    /// Executes when the REPL starts (only available with `repl` feature)
    #[cfg(feature = "repl")]
    pub repl_on_begin: Option<Box<dyn Fn(&HookREPLBeginInfo) + Send + Sync>>,

    /// Executes before reading the next REPL line (only available with `repl` feature)
    #[cfg(feature = "repl")]
    pub repl_pre_readline: Option<Box<dyn Fn(&HookREPLPreReadlineInfo) + Send + Sync>>,

    /// Custom REPL line reader (only available with `repl` feature)
    /// Returns `Some(line)` to provide a custom input line.
    #[cfg(feature = "repl")]
    pub repl_readline: Option<Box<dyn Fn(&HookREPLReadlineInfo) -> Option<String> + Send + Sync>>,

    /// Executes after reading a REPL line (only available with `repl` feature)
    #[cfg(feature = "repl")]
    pub repl_post_readline:
        Option<Box<dyn for<'a> Fn(&HookREPLPostReadlineInfo<'a>) + Send + Sync>>,

    /// Executes before executing a REPL command (only available with `repl` feature)
    #[cfg(feature = "repl")]
    pub repl_pre_exec: Option<Box<dyn for<'a> Fn(&HookREPLPreExecInfo<'a>) + Send + Sync>>,

    /// Executes after executing a REPL command (only available with `repl` feature)
    #[cfg(feature = "repl")]
    pub repl_post_exec: Option<Box<dyn Fn(&HookREPLPostExecInfo) + Send + Sync>>,

    /// Executes when the REPL receives a render result (only available with `repl` feature)
    #[cfg(feature = "repl")]
    pub repl_on_receive_result:
        Option<Box<dyn for<'a> Fn(&HookREPLOnReceiveResultInfo<'a>) + Send + Sync>>,

    /// Executes when the REPL panics (only available with `repl` feature)
    #[cfg(all(feature = "repl", not(feature = "async")))]
    pub repl_on_panic: Option<Box<dyn for<'a> Fn(&HookREPLOnPanicInfo<'a>) + Send + Sync>>,

    /// Executes when the REPL exits (only available with `repl` feature)
    #[cfg(feature = "repl")]
    pub repl_exit: Option<Box<dyn Fn(&HookREPLExitInfo) + Send + Sync>>,

    /// Executes after each REPL loop iteration (only available with `repl` feature)
    #[cfg(feature = "repl")]
    pub repl_loop_once: Option<Box<dyn Fn(&HookREPLLoopOnceInfo) + Send + Sync>>,
}

impl<C> Program<C>
where
    C: ProgramCollect<Enum = C>,
{
    /// Adds a typed hook to the program. The hook will be called at the appropriate
    /// lifecycle events.
    pub fn with_hook(&mut self, hook: ProgramHook<C>) -> &mut Self {
        self.hooks.push(hook);
        self
    }

    pub(crate) fn run_hook_on_begin(&self, info: HookBeginInfo) {
        if !self.user_context.run_hook {
            return;
        }

        for hook in &self.hooks {
            if let Some(ref begin) = hook.begin {
                begin(&info);
            }
        }
    }

    pub(crate) fn run_hook_pre_dispatch(&self, info: HookPreDispatchInfo) -> ProgramControls<C> {
        if !self.user_context.run_hook {
            return ProgramControls::Empty;
        }

        let mut controls = ProgramControls::Empty;
        for hook in &self.hooks {
            if let Some(ref pre_dispatch) = hook.pre_dispatch {
                controls = pre_dispatch(&info);
            }
        }
        controls
    }

    pub(crate) fn run_hook_post_dispatch(
        &self,
        info: HookPostDispatchInfo<C>,
    ) -> ProgramControls<C> {
        if !self.user_context.run_hook {
            return ProgramControls::Empty;
        }

        let mut controls = ProgramControls::Empty;
        for hook in &self.hooks {
            if let Some(ref post_dispatch) = hook.post_dispatch {
                controls = post_dispatch(&info);
            }
        }
        controls
    }

    pub(crate) fn run_hook_pre_chain(&self, info: HookPreChainInfo<C>) -> ProgramControls<C> {
        if !self.user_context.run_hook {
            return ProgramControls::Empty;
        }

        let mut controls = ProgramControls::Empty;
        for hook in &self.hooks {
            if let Some(ref pre_chain) = hook.pre_chain {
                controls = pre_chain(&info);
            }
        }
        controls
    }

    pub(crate) fn run_hook_post_chain(&self, info: HookPostChainInfo<C>) -> ProgramControls<C> {
        if !self.user_context.run_hook {
            return ProgramControls::Empty;
        }

        let mut controls = ProgramControls::Empty;
        for hook in &self.hooks {
            if let Some(ref post_chain) = hook.post_chain {
                controls = post_chain(&info);
            }
        }
        controls
    }

    pub(crate) fn run_hook_pre_render(&self, info: HookPreRenderInfo<C>) -> ProgramControls<C> {
        if !self.user_context.run_hook {
            return ProgramControls::Empty;
        }

        let mut controls = ProgramControls::Empty;
        for hook in &self.hooks {
            if let Some(ref pre_render) = hook.pre_render {
                controls = pre_render(&info);
            }
        }
        controls
    }

    pub(crate) fn run_hook_post_render(&self, info: HookPostRenderInfo) -> ProgramControls<C> {
        if !self.user_context.run_hook {
            return ProgramControls::Empty;
        }

        let mut controls = ProgramControls::Empty;
        for hook in &self.hooks {
            if let Some(ref post_render) = hook.post_render {
                controls = post_render(&info);
            }
        }
        controls
    }

    #[allow(dead_code)]
    #[cfg(not(feature = "async"))]
    pub(crate) fn run_hook_exec_panic(&self, info: HookPanicInfo) {
        if !self.user_context.run_hook {
            return;
        }

        for hook in &self.hooks {
            if let Some(ref exec_panic) = hook.exec_panic {
                exec_panic(&info);
            }
        }
    }

    pub(crate) fn run_hook_finish(&self, info: HookFinishInfo) -> ProgramControls<C> {
        if !self.user_context.run_hook {
            return ProgramControls::Empty;
        }

        let mut controls = ProgramControls::Empty;
        for hook in &self.hooks {
            if let Some(ref finish) = hook.finish {
                controls = finish(&info);
            }
        }
        controls
    }

    /// Runs the REPL begin hooks (only available with `repl` feature)
    #[cfg(feature = "repl")]
    pub(crate) fn run_hook_repl_on_begin(&self, info: HookREPLBeginInfo) {
        if !self.user_context.run_hook {
            return;
        }

        for hook in &self.hooks {
            if let Some(ref repl_on_begin) = hook.repl_on_begin {
                repl_on_begin(&info);
            }
        }
    }

    /// Runs the REPL pre-readline hooks (only available with `repl` feature)
    #[cfg(feature = "repl")]
    pub(crate) fn run_hook_repl_pre_readline(&self, info: HookREPLPreReadlineInfo) {
        if !self.user_context.run_hook {
            return;
        }

        for hook in &self.hooks {
            if let Some(ref repl_pre_readline) = hook.repl_pre_readline {
                repl_pre_readline(&info);
            }
        }
    }

    /// Runs the custom REPL readline hook (only available with `repl` feature)
    /// Returns `Some(line)` if a hook was set and returned Some, otherwise `None`.
    #[cfg(feature = "repl")]
    pub(crate) fn run_hook_repl_readline(&self, info: HookREPLReadlineInfo) -> Option<String> {
        if !self.user_context.run_hook {
            return None;
        }

        for hook in &self.hooks {
            if let Some(ref repl_readline) = hook.repl_readline {
                return repl_readline(&info);
            }
        }
        None
    }

    /// Runs the REPL post-readline hooks (only available with `repl` feature)
    #[cfg(feature = "repl")]
    pub(crate) fn run_hook_repl_post_readline(&self, info: HookREPLPostReadlineInfo) {
        if !self.user_context.run_hook {
            return;
        }

        for hook in &self.hooks {
            if let Some(ref repl_post_readline) = hook.repl_post_readline {
                repl_post_readline(&info);
            }
        }
    }

    /// Runs the REPL pre-exec hooks (only available with `repl` feature)
    #[cfg(feature = "repl")]
    pub(crate) fn run_hook_repl_pre_exec(&self, info: HookREPLPreExecInfo) {
        if !self.user_context.run_hook {
            return;
        }

        for hook in &self.hooks {
            if let Some(ref repl_pre_exec) = hook.repl_pre_exec {
                repl_pre_exec(&info);
            }
        }
    }

    /// Runs the REPL post-exec hooks (only available with `repl` feature)
    #[cfg(feature = "repl")]
    pub(crate) fn run_hook_repl_post_exec(&self, info: HookREPLPostExecInfo) {
        if !self.user_context.run_hook {
            return;
        }

        for hook in &self.hooks {
            if let Some(ref repl_post_exec) = hook.repl_post_exec {
                repl_post_exec(&info);
            }
        }
    }

    /// Runs the REPL receive result hooks (only available with `repl` feature)
    #[cfg(feature = "repl")]
    pub(crate) fn run_hook_repl_on_receive_result(&self, info: HookREPLOnReceiveResultInfo) {
        if !self.user_context.run_hook {
            return;
        }

        for hook in &self.hooks {
            if let Some(ref repl_on_receive_result) = hook.repl_on_receive_result {
                repl_on_receive_result(&info);
            }
        }
    }

    /// Runs the REPL panic hooks (only available with `repl` feature)
    #[cfg(all(feature = "repl", not(feature = "async")))]
    pub(crate) fn run_hook_repl_on_panic(&self, info: HookREPLOnPanicInfo) {
        if !self.user_context.run_hook {
            return;
        }

        for hook in &self.hooks {
            if let Some(ref repl_on_panic) = hook.repl_on_panic {
                repl_on_panic(&info);
            }
        }
    }

    /// Runs the REPL exit hooks (only available with `repl` feature)
    #[cfg(feature = "repl")]
    pub(crate) fn run_hook_repl_exit(&self, info: HookREPLExitInfo) {
        if !self.user_context.run_hook {
            return;
        }

        for hook in &self.hooks {
            if let Some(ref repl_exit) = hook.repl_exit {
                repl_exit(&info);
            }
        }
    }

    /// Runs the REPL loop_once hooks (only available with `repl` feature)
    #[cfg(feature = "repl")]
    pub(crate) fn run_hook_repl_loop_once(&self, info: HookREPLLoopOnceInfo) {
        if !self.user_context.run_hook {
            return;
        }

        for hook in &self.hooks {
            if let Some(ref repl_loop_once) = hook.repl_loop_once {
                repl_loop_once(&info);
            }
        }
    }
}

impl<C> ProgramHook<C>
where
    C: ProgramCollect<Enum = C>,
{
    /// Creates a new empty hook set with no handlers.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            begin: None,
            pre_dispatch: None,
            post_dispatch: None,
            pre_chain: None,
            post_chain: None,
            pre_render: None,
            post_render: None,
            finish: None,
            #[cfg(not(feature = "async"))]
            exec_panic: None,
            #[cfg(feature = "repl")]
            repl_on_begin: None,
            #[cfg(feature = "repl")]
            repl_pre_readline: None,
            #[cfg(feature = "repl")]
            repl_readline: None,
            #[cfg(feature = "repl")]
            repl_post_readline: None,
            #[cfg(feature = "repl")]
            repl_pre_exec: None,
            #[cfg(feature = "repl")]
            repl_post_exec: None,
            #[cfg(feature = "repl")]
            repl_on_receive_result: None,
            #[cfg(all(feature = "repl", not(feature = "async")))]
            repl_on_panic: None,
            #[cfg(feature = "repl")]
            repl_exit: None,
            #[cfg(feature = "repl")]
            repl_loop_once: None,
        }
    }

    /// Sets the handler for the `begin` event.
    #[must_use]
    pub fn on_begin<F, R>(mut self, handler: F) -> Self
    where
        F: Fn(&HookBeginInfo) + 'static + Send + Sync,
    {
        self.begin = Some(Box::new(move |info| handler(info)));
        self
    }

    /// Sets the handler for the `pre_dispatch` event.
    #[must_use]
    pub fn on_pre_dispatch<F, R>(mut self, handler: F) -> Self
    where
        F: for<'a> Fn(&HookPreDispatchInfo<'a>) -> R + 'static + Send + Sync,
        R: Into<ProgramControls<C>>,
    {
        self.pre_dispatch = Some(Box::new(move |info| handler(info).into()));
        self
    }

    /// Sets the handler for the `post_dispatch` event.
    #[must_use]
    pub fn on_post_dispatch<F, R>(mut self, handler: F) -> Self
    where
        F: for<'a> Fn(&HookPostDispatchInfo<'a, C>) -> R + 'static + Send + Sync,
        R: Into<ProgramControls<C>>,
    {
        self.post_dispatch = Some(Box::new(move |info| handler(info).into()));
        self
    }

    /// Sets the handler for the `pre_chain` event.
    #[must_use]
    pub fn on_pre_chain<F, R>(mut self, handler: F) -> Self
    where
        F: for<'a> Fn(&HookPreChainInfo<'a, C>) -> R + 'static + Send + Sync,
        R: Into<ProgramControls<C>>,
    {
        self.pre_chain = Some(Box::new(move |info| handler(info).into()));
        self
    }

    /// Sets the handler for the `post_chain` event.
    #[must_use]
    pub fn on_post_chain<F, R>(mut self, handler: F) -> Self
    where
        F: for<'a> Fn(&HookPostChainInfo<'a, C>) -> R + 'static + Send + Sync,
        R: Into<ProgramControls<C>>,
    {
        self.post_chain = Some(Box::new(move |info| handler(info).into()));
        self
    }

    /// Sets the handler for the `pre_render` event.
    #[must_use]
    pub fn on_pre_render<F, R>(mut self, handler: F) -> Self
    where
        F: for<'a> Fn(&HookPreRenderInfo<'a, C>) -> R + 'static + Send + Sync,
        R: Into<ProgramControls<C>>,
    {
        self.pre_render = Some(Box::new(move |info| handler(info).into()));
        self
    }

    /// Sets the handler for the `post_render` event.
    #[must_use]
    pub fn on_post_render<F, R>(mut self, handler: F) -> Self
    where
        F: for<'a> Fn(&HookPostRenderInfo<'a>) -> R + 'static + Send + Sync,
        R: Into<ProgramControls<C>>,
    {
        self.post_render = Some(Box::new(move |info| handler(info).into()));
        self
    }

    /// Sets the handler for the `finish` event.
    #[must_use]
    pub fn on_finish<F, R>(mut self, handler: F) -> Self
    where
        F: Fn(&HookFinishInfo) -> R + 'static + Send + Sync,
        R: Into<ProgramControls<C>>,
    {
        self.finish = Some(Box::new(move |info| handler(info).into()));
        self
    }

    /// Sets the handler for the `exec_panic` event.
    #[cfg(not(feature = "async"))]
    #[must_use]
    pub fn on_exec_panic<F, R>(mut self, handler: F) -> Self
    where
        F: for<'a> Fn(&HookPanicInfo<'a>) + 'static + Send + Sync,
    {
        self.exec_panic = Some(Box::new(move |info| handler(info)));
        self
    }

    /// Sets the handler for the REPL begin event (only available with `repl` feature).
    #[cfg(feature = "repl")]
    #[must_use]
    pub fn on_repl_begin<F>(mut self, handler: F) -> Self
    where
        F: Fn(&HookREPLBeginInfo) + 'static + Send + Sync,
    {
        self.repl_on_begin = Some(Box::new(move |info| handler(info)));
        self
    }

    /// Sets the handler for the REPL pre-readline event (only available with `repl` feature).
    /// This hook runs after `on_repl_begin` but before reading the next input line.
    #[cfg(feature = "repl")]
    #[must_use]
    pub fn on_repl_pre_readline<F>(mut self, handler: F) -> Self
    where
        F: Fn(&HookREPLPreReadlineInfo) + 'static + Send + Sync,
    {
        self.repl_pre_readline = Some(Box::new(move |info| handler(info)));
        self
    }

    /// Sets the custom REPL line reader (only available with `repl` feature).
    #[cfg(feature = "repl")]
    #[must_use]
    pub fn on_repl_readline<F>(mut self, handler: F) -> Self
    where
        F: Fn(&HookREPLReadlineInfo) -> Option<String> + 'static + Send + Sync,
    {
        self.repl_readline = Some(Box::new(move |info| handler(info)));
        self
    }

    /// Sets the handler for the REPL post-readline event (only available with `repl` feature).
    /// This hook runs after reading a line of input and receives the read line info.
    #[cfg(feature = "repl")]
    #[must_use]
    pub fn on_repl_post_readline<F>(mut self, handler: F) -> Self
    where
        F: for<'a> Fn(&HookREPLPostReadlineInfo<'a>) + 'static + Send + Sync,
    {
        self.repl_post_readline = Some(Box::new(move |info| handler(info)));
        self
    }

    /// Sets the handler for the REPL pre-exec event (only available with `repl` feature).
    /// This hook runs before executing a REPL command, receiving the parsed arguments.
    #[cfg(feature = "repl")]
    #[must_use]
    pub fn on_repl_pre_exec<F>(mut self, handler: F) -> Self
    where
        F: for<'a> Fn(&HookREPLPreExecInfo<'a>) + 'static + Send + Sync,
    {
        self.repl_pre_exec = Some(Box::new(move |info| handler(info)));
        self
    }

    /// Sets the handler for the REPL post-exec event (only available with `repl` feature).
    /// This hook runs after executing a REPL command.
    #[cfg(feature = "repl")]
    #[must_use]
    pub fn on_repl_post_exec<F>(mut self, handler: F) -> Self
    where
        F: Fn(&HookREPLPostExecInfo) + 'static + Send + Sync,
    {
        self.repl_post_exec = Some(Box::new(move |info| handler(info)));
        self
    }

    /// Sets the handler for the REPL receive result event (only available with `repl` feature).
    /// This hook runs after a command is executed, receiving the render result on success.
    #[cfg(feature = "repl")]
    #[must_use]
    pub fn on_repl_receive_result<F>(mut self, handler: F) -> Self
    where
        F: for<'a> Fn(&HookREPLOnReceiveResultInfo<'a>) + 'static + Send + Sync,
    {
        self.repl_on_receive_result = Some(Box::new(move |info| handler(info)));
        self
    }

    /// Sets the handler for the REPL panic event (only available with `repl` feature).
    #[cfg(all(feature = "repl", not(feature = "async")))]
    #[must_use]
    pub fn on_repl_panic<F>(mut self, handler: F) -> Self
    where
        F: for<'a> Fn(&HookREPLOnPanicInfo<'a>) + 'static + Send + Sync,
    {
        self.repl_on_panic = Some(Box::new(move |info| handler(info)));
        self
    }

    /// Sets the handler for the REPL exit event (only available with `repl` feature).
    /// This hook runs when the REPL is about to exit.
    #[cfg(feature = "repl")]
    #[must_use]
    pub fn on_repl_exit<F>(mut self, handler: F) -> Self
    where
        F: Fn(&HookREPLExitInfo) + 'static + Send + Sync,
    {
        self.repl_exit = Some(Box::new(move |info| handler(info)));
        self
    }

    /// Sets the handler for the REPL loop_once event (only available with `repl` feature).
    /// This hook runs after each REPL loop iteration.
    #[cfg(feature = "repl")]
    #[must_use]
    pub fn on_repl_loop_once<F>(mut self, handler: F) -> Self
    where
        F: Fn(&HookREPLLoopOnceInfo) + 'static + Send + Sync,
    {
        self.repl_loop_once = Some(Box::new(move |info| handler(info)));
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Grouped;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[cfg_attr(feature = "structural_renderer", derive(serde::Serialize))]
    enum MockHookEnum {
        A,
        B,
    }

    impl std::fmt::Display for MockHookEnum {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self)
        }
    }

    /// SAFETY:
    ///
    /// This implementation is only for testing purposes to satisfy trait bounds.
    /// Since this code only constructs `AnyOutput` and calls methods like
    /// `downcast`, `is`, `restore`, `route_chain`, and `route_renderer` —
    /// none of which involve `ProgramCollect::do_chain` or
    /// `ProgramCollect::render` — the type/member_id correspondence is
    /// never exploited in an unsafe way here.
    /// The caller must ensure that the associated `member_id` correctly
    /// corresponds to the type's role in the group.
    unsafe impl Grouped<MockHookEnum> for MockHookEnum {
        fn member_id() -> MockHookEnum {
            MockHookEnum::A
        }
    }

    impl ProgramCollect for MockHookEnum {
        type Enum = MockHookEnum;
        type EntryFallback = MockHookEnum;
        type ErrorRendererNotFound = MockHookEnum;
        type ResultEmpty = MockHookEnum;

        fn build_renderer_not_found(_member_id: MockHookEnum) -> crate::AnyOutput<MockHookEnum> {
            unreachable!()
        }

        fn build_entry_fallback(_args: Vec<String>) -> crate::AnyOutput<MockHookEnum> {
            unreachable!()
        }

        fn build_empty_result() -> crate::AnyOutput<MockHookEnum> {
            unreachable!()
        }

        fn render(_any: crate::AnyOutput<MockHookEnum>) -> crate::RenderResult {
            unreachable!()
        }

        fn render_help(_any: crate::AnyOutput<MockHookEnum>) -> crate::RenderResult {
            unreachable!()
        }

        fn do_chain(_any: crate::AnyOutput<MockHookEnum>) -> crate::ChainProcess<MockHookEnum> {
            unreachable!()
        }

        fn has_renderer(_any: &crate::AnyOutput<MockHookEnum>) -> bool {
            unreachable!()
        }

        fn has_chain(_any: &crate::AnyOutput<MockHookEnum>) -> bool {
            unreachable!()
        }

        #[cfg(feature = "comp")]
        fn do_comp(
            _any: &crate::AnyOutput<MockHookEnum>,
            _ctx: &crate::ShellContext,
        ) -> crate::Suggest {
            unreachable!()
        }

        #[cfg(feature = "structural_renderer")]
        fn structural_render(
            _any: crate::AnyOutput<MockHookEnum>,
            _setting: &crate::StructuralRendererSetting,
        ) -> Result<crate::RenderResult, crate::error::StructuralRendererSerializeError> {
            unreachable!()
        }
    }

    #[test]
    fn test_hook_empty() {
        let hook = ProgramHook::<MockHookEnum>::empty();
        assert!(hook.begin.is_none());
        assert!(hook.pre_dispatch.is_none());
        assert!(hook.post_dispatch.is_none());
        assert!(hook.pre_chain.is_none());
        assert!(hook.post_chain.is_none());
        assert!(hook.pre_render.is_none());
        assert!(hook.post_render.is_none());
        assert!(hook.finish.is_none());
    }

    #[test]
    fn test_hook_on_begin() {
        static CALLED: AtomicBool = AtomicBool::new(false);
        let hook = ProgramHook::<MockHookEnum>::empty().on_begin::<_, ()>(|_: &HookBeginInfo| {
            CALLED.store(true, Ordering::SeqCst);
        });
        assert!(hook.begin.is_some());
        (hook.begin.as_ref().unwrap())(&HookBeginInfo {});
        assert!(CALLED.load(Ordering::SeqCst));
    }

    #[test]
    fn test_hook_on_pre_dispatch() {
        static CALLED: AtomicBool = AtomicBool::new(false);
        let hook =
            ProgramHook::<MockHookEnum>::empty().on_pre_dispatch(|info: &HookPreDispatchInfo| {
                assert_eq!(info.arguments, &["a", "b"]);
                CALLED.store(true, Ordering::SeqCst);
            });
        assert!(hook.pre_dispatch.is_some());
        (hook.pre_dispatch.as_ref().unwrap())(&HookPreDispatchInfo {
            arguments: &["a".to_string(), "b".to_string()],
        });
        assert!(CALLED.load(Ordering::SeqCst));
    }

    #[test]
    fn test_hook_on_post_dispatch() {
        static CALLED: AtomicBool = AtomicBool::new(false);
        let hook = ProgramHook::<MockHookEnum>::empty().on_post_dispatch(
            |info: &HookPostDispatchInfo<MockHookEnum>| {
                assert_eq!(*info.entry, MockHookEnum::A);
                CALLED.store(true, Ordering::SeqCst);
            },
        );
        assert!(hook.post_dispatch.is_some());
        (hook.post_dispatch.as_ref().unwrap())(&HookPostDispatchInfo {
            entry: &MockHookEnum::A,
        });
        assert!(CALLED.load(Ordering::SeqCst));
    }

    #[test]
    fn test_hook_on_pre_chain() {
        static CALLED: AtomicBool = AtomicBool::new(false);
        let hook = ProgramHook::<MockHookEnum>::empty().on_pre_chain(
            |info: &HookPreChainInfo<MockHookEnum>| {
                assert_eq!(*info.input, MockHookEnum::A);
                CALLED.store(true, Ordering::SeqCst);
            },
        );
        assert!(hook.pre_chain.is_some());
        (hook.pre_chain.as_ref().unwrap())(&HookPreChainInfo {
            input: &MockHookEnum::A,
            raw: &42,
        });
        assert!(CALLED.load(Ordering::SeqCst));
    }

    #[test]
    fn test_hook_on_post_chain() {
        static CALLED: AtomicBool = AtomicBool::new(false);
        let hook = ProgramHook::<MockHookEnum>::empty().on_post_chain(
            |_info: &HookPostChainInfo<MockHookEnum>| {
                CALLED.store(true, Ordering::SeqCst);
            },
        );
        assert!(hook.post_chain.is_some());
        let output = crate::AnyOutput::new(MockHookEnum::A);
        (hook.post_chain.as_ref().unwrap())(&HookPostChainInfo { output: &output });
        assert!(CALLED.load(Ordering::SeqCst));
    }

    #[test]
    fn test_hook_on_pre_render() {
        static CALLED: AtomicBool = AtomicBool::new(false);
        let hook = ProgramHook::<MockHookEnum>::empty().on_pre_render(
            |info: &HookPreRenderInfo<MockHookEnum>| {
                assert_eq!(*info.input, MockHookEnum::A);
                CALLED.store(true, Ordering::SeqCst);
            },
        );
        assert!(hook.pre_render.is_some());
        (hook.pre_render.as_ref().unwrap())(&HookPreRenderInfo {
            input: &MockHookEnum::A,
            raw: &42,
        });
        assert!(CALLED.load(Ordering::SeqCst));
    }

    #[test]
    fn test_hook_on_post_render() {
        static CALLED: AtomicBool = AtomicBool::new(false);
        let hook =
            ProgramHook::<MockHookEnum>::empty().on_post_render(|_info: &HookPostRenderInfo| {
                CALLED.store(true, Ordering::SeqCst);
            });
        assert!(hook.post_render.is_some());
        let result = crate::RenderResult::default();
        (hook.post_render.as_ref().unwrap())(&HookPostRenderInfo { result: &result });
        assert!(CALLED.load(Ordering::SeqCst));
    }

    #[test]
    fn test_hook_on_finish() {
        let hook = ProgramHook::<MockHookEnum>::empty()
            .on_finish(|_: &HookFinishInfo| ProgramControlUnit::OverrideExitCode(42));
        assert!(hook.finish.is_some());
        let controls: Vec<ProgramControlUnit<MockHookEnum>> =
            (hook.finish.as_ref().unwrap())(&HookFinishInfo {})
                .into_iter()
                .collect();
        assert_eq!(controls.len(), 1);
        match &controls[0] {
            ProgramControlUnit::OverrideExitCode(code) => assert_eq!(*code, 42),
            _ => panic!("Unexpected control unit"),
        }
    }

    #[test]
    fn test_hook_builder_chaining() {
        let hook = ProgramHook::<MockHookEnum>::empty()
            .on_begin::<_, ()>(|_: &HookBeginInfo| ())
            .on_pre_dispatch(|_: &HookPreDispatchInfo| ())
            .on_post_dispatch(|_: &HookPostDispatchInfo<MockHookEnum>| ())
            .on_pre_chain(|_: &HookPreChainInfo<MockHookEnum>| ())
            .on_post_chain(|_: &HookPostChainInfo<MockHookEnum>| ())
            .on_pre_render(|_: &HookPreRenderInfo<MockHookEnum>| ())
            .on_post_render(|_: &HookPostRenderInfo| ())
            .on_finish(|_: &HookFinishInfo| ProgramControlUnit::OverrideExitCode(0));
        assert!(hook.begin.is_some());
        assert!(hook.pre_dispatch.is_some());
        assert!(hook.post_dispatch.is_some());
        assert!(hook.pre_chain.is_some());
        assert!(hook.post_chain.is_some());
        assert!(hook.pre_render.is_some());
        assert!(hook.post_render.is_some());
        assert!(hook.finish.is_some());
    }
}
