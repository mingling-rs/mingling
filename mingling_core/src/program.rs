#[cfg(not(windows))]
use std::env;

#[cfg(feature = "structural_renderer")]
use crate::config::StructuralRendererSetting;
use crate::{
    AnyOutput, GlobalResContainer,
    asset::dispatcher::Dispatcher,
    config::{ProgramStdoutSetting, ProgramUserContext, QueryMode},
    error::ChainProcessError,
    hook::ProgramHook,
    renderer::stream::{RenderStreamState, RenderedRecord},
};

#[cfg(feature = "structural_renderer")]
use std::sync::atomic::{AtomicBool, Ordering};

pub mod config;

#[doc(hidden)]
pub mod error;

#[doc(hidden)]
pub mod exec;

pub mod hook;

#[cfg(feature = "repl")]
#[doc(hidden)]
pub mod repl_exec;

#[doc(hidden)]
pub mod setup;

mod collection;
pub use collection::*;

mod single_instance;
pub use single_instance::*;

mod flag;
pub use flag::*;

mod string_vec;
pub use string_vec::*;

mod once_exec;

/// Program, used to define the behavior of the entire command-line program
#[derive(Default)]
pub struct Program<C>
where
    C: ProgramCollect<Enum = C>,
{
    pub(crate) collect: std::marker::PhantomData<C>,

    pub(crate) args: Vec<String>,

    /// Program stdout settings.
    ///
    /// This struct controls the program's output behavior, including whether
    /// to output errors, render results, suppress panic messages, enable
    /// verbose/quiet/debug modes, colored output, and progress indicators.
    ///
    /// # Convention-only fields
    ///
    /// Fields marked with convention only are not enforced by the framework;
    /// they serve as a shared convention for applications to follow consistently.
    pub stdout_setting: ProgramStdoutSetting,

    /// User-defined context that can be accessed throughout the program.
    ///
    /// This field holds a user-defined context value that can be
    /// accessed and modified at any point during program execution. It is
    /// useful for passing shared state or configuration across different
    /// parts of the program.
    pub user_context: ProgramUserContext,

    #[cfg(feature = "structural_renderer")]
    /// Setting for the structural renderer.
    ///
    /// This field is only available when the `structural_renderer` feature is enabled.
    pub structural_renderer_name: StructuralRendererSetting,

    pub(crate) hooks: Vec<ProgramHook<C>>,

    pub(crate) resources: GlobalResContainer,

    /// Current inline render stream state.
    pub(crate) render_stream: std::sync::Mutex<RenderStreamState>,

    /// Whether any inline render record has been emitted during this run.
    #[cfg(feature = "structural_renderer")]
    pub(crate) render_stream_used: AtomicBool,
}

impl<C> Program<C>
where
    C: ProgramCollect<Enum = C>,
{
    /// Creates a new Program instance, initializing command-line arguments from the environment.
    #[must_use]
    pub fn new() -> Self {
        #[cfg(not(windows))]
        return Self::new_with_args(env::args().collect::<Vec<String>>());

        #[cfg(windows)]
        return Self::new_with_args({
            std::env::args_os()
                .map(|arg| {
                    use std::os::windows::ffi::OsStrExt;

                    let wide: Vec<u16> = arg.encode_wide().collect();
                    String::from_utf16_lossy(&wide)
                })
                .collect::<Vec<_>>()
        });
    }

    /// Creates a new Program instance with the provided command-line arguments.
    pub fn new_with_args(args: impl Into<StringVec>) -> Self {
        Self {
            collect: std::marker::PhantomData,
            args: args.into().into(),

            stdout_setting: ProgramStdoutSetting::default(),
            user_context: ProgramUserContext::default(),

            #[cfg(feature = "structural_renderer")]
            structural_renderer_name: StructuralRendererSetting::Disable,

            hooks: Vec::new(),

            resources: GlobalResContainer::new(),

            render_stream: std::sync::Mutex::new(RenderStreamState::default()),
            #[cfg(feature = "structural_renderer")]
            render_stream_used: AtomicBool::new(false),
        }
    }

    /// Returns the effective query mode, honoring the `print_help` and
    /// `print_version` output settings.
    ///
    /// A usage query is downgraded to [`QueryMode::None`] when
    /// [`ProgramStdoutSetting::print_help`](crate::config::ProgramStdoutSetting::print_help)
    /// is `false`, and a version query likewise when
    /// [`ProgramStdoutSetting::print_version`](crate::config::ProgramStdoutSetting::print_version)
    /// is `false`. This lets a program disable the built-in hijack (for example
    /// to let a `clap` dispatcher handle `--help`/`--version` itself).
    #[must_use]
    pub const fn effective_query(&self) -> QueryMode {
        match self.user_context.query {
            QueryMode::Usage if !self.stdout_setting.print_help => QueryMode::None,
            QueryMode::Version if !self.stdout_setting.print_version => QueryMode::None,
            query => query,
        }
    }

    /// Emits one inline render record according to the current stream state.
    ///
    /// In `Direct` mode the record is written and flushed immediately. In
    /// `Capture` mode it is stored for [`take_captured_records`](Self::take_captured_records).
    pub(crate) fn emit_rendered_record(&self, record: RenderedRecord) {
        #[cfg(feature = "structural_renderer")]
        self.render_stream_used.store(true, Ordering::Relaxed);

        let mut state = self
            .render_stream
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        match &mut *state {
            RenderStreamState::Direct { enabled: true } => {
                crate::renderer::stream::write_rendered_record(&record);
            }
            RenderStreamState::Direct { enabled: false } => {}
            RenderStreamState::Capture(records) => records.push(record),
        }
    }

    /// Returns whether any inline render record was emitted during this run.
    #[cfg(feature = "structural_renderer")]
    pub(crate) fn has_inline_render_output(&self) -> bool {
        self.render_stream_used.load(Ordering::Relaxed)
    }

    /// Marks this run as rendering inline, even before any record is emitted.
    ///
    /// The final empty result is suppressed for a run that streams, so a listing with no
    /// records writes nothing rather than one `null`.
    #[cfg(feature = "structural_renderer")]
    pub(crate) fn mark_render_stream_used(&self) {
        self.render_stream_used.store(true, Ordering::Relaxed);
    }

    /// Takes all records captured in `Capture` mode, leaving the capture list empty.
    pub(crate) fn take_captured_records(&self) -> Vec<RenderedRecord> {
        let mut state = self
            .render_stream
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        match &mut *state {
            RenderStreamState::Capture(records) => std::mem::take(records),
            RenderStreamState::Direct { .. } => Vec::new(),
        }
    }

    /// Returns a reference to the current program instance, if set.
    ///
    /// # Panics
    ///
    /// Panics if the program has not been initialized yet.
    pub fn this_program() -> &'static Self
    where
        C: 'static,
    {
        THIS_PROGRAM
            .get_raw()
            .unwrap()
            .downcast_ref::<Self>()
            .unwrap()
    }

    /// Returns a reference to the program's command-line arguments.
    #[must_use]
    pub fn get_args(&self) -> &[String] {
        &self.args
    }

    /// Returns a mutable reference to the program's command-line arguments.
    #[must_use]
    pub fn get_args_mut(&mut self) -> &mut [String] {
        &mut self.args
    }

    /// Takes ownership of the program's command-line arguments, replacing them with an empty Vec.
    /// This is useful when you need to transfer the arguments to another context or process them
    /// and then replace them later.
    #[must_use]
    pub fn take_args(&mut self) -> Vec<String> {
        std::mem::take(&mut self.args)
    }

    /// Replaces the program's command-line arguments with a new set and returns the old ones.
    ///
    /// # Arguments
    ///
    /// * `args` - The new command-line arguments to set.
    ///
    /// # Returns
    ///
    /// The previous command-line arguments.
    pub const fn replace_args(&mut self, args: Vec<String>) -> Vec<String> {
        std::mem::replace(&mut self.args, args)
    }

    /// Get all registered dispatcher names from the program
    #[must_use]
    pub fn get_nodes(
        &'static self,
    ) -> Vec<(String, &'static (dyn Dispatcher<C> + Send + Sync + 'static))> {
        get_nodes(self)
    }

    /// Dispatch input arguments to an entry
    ///
    /// # Errors
    ///
    /// Returns `Err(ChainProcessError)` if the dispatch fails,
    /// e.g., if no dispatcher is found for the given arguments.
    pub fn dispatch_args(
        &'static self,
        args: impl Into<StringVec>,
    ) -> Result<AnyOutput<C>, ChainProcessError> {
        let string_vec: Vec<String> = args.into().into();
        match C::dispatch_args(&string_vec) {
            Ok(ok) => Ok(ok),
            Err(e) => Err(e.into()),
        }
    }
}

/// Get all registered dispatcher names from the program
#[allow(unused_variables)]
#[must_use]
pub fn get_nodes<C: ProgramCollect<Enum = C>>(
    program: &'static Program<C>,
) -> Vec<(String, &'static (dyn Dispatcher<C> + Send + Sync + 'static))> {
    let r = C::get_nodes();

    #[cfg(feature = "debug")]
    {
        let node_strs: Vec<String> = r.iter().map(|v| v.0.clone()).collect();
        crate::info!("All Nodes: [{}]", node_strs.join(", "));
    }

    r
}
