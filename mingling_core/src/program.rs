// Doc Not Optimize
#[cfg(not(windows))]
use std::env;

#[cfg(feature = "structural_renderer")]
use crate::config::StructuralRendererSetting;
use crate::{
    AnyOutput, GlobalResContainer,
    asset::dispatcher::Dispatcher,
    config::{ProgramStdoutSetting, ProgramUserContext},
    error::ChainProcessError,
    hook::ProgramHook,
};

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

    #[cfg(not(feature = "dispatch_tree"))]
    pub(crate) dispatcher: Vec<Box<dyn Dispatcher<C> + Send + Sync>>,

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

            #[cfg(not(feature = "dispatch_tree"))]
            dispatcher: Vec::new(),

            stdout_setting: ProgramStdoutSetting::default(),
            user_context: ProgramUserContext::default(),

            #[cfg(feature = "structural_renderer")]
            structural_renderer_name: StructuralRendererSetting::Disable,

            hooks: Vec::new(),

            resources: GlobalResContainer::new(),
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

    /// Dynamically dispatch input arguments to registered entry types
    ///
    /// # Errors
    ///
    /// Returns `Err(ChainProcessError)` if the dispatch fails,
    /// e.g., if no dispatcher is found for the given arguments.
    pub fn dispatch_args_dynamic(
        &'static self,
        args: impl Into<StringVec>,
    ) -> Result<AnyOutput<C>, ChainProcessError> {
        let sv: Vec<String> = args.into().into();
        match exec::dispatch_args_dynamic(self, &sv) {
            Ok(ok) => Ok(ok),
            Err(e) => Err(e.into()),
        }
    }

    /// Use a prefix tree to quickly match arguments and dispatch to an Entry
    #[cfg(feature = "dispatch_tree")]
    pub fn dispatch_args_trie(
        &'static self,
        args: impl Into<StringVec>,
    ) -> Result<AnyOutput<C>, ChainProcessError> {
        let string_vec: Vec<String> = args.into().into();
        match C::dispatch_args_trie(&string_vec) {
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
    #[cfg(feature = "dispatch_tree")]
    let r = C::get_nodes();

    #[cfg(feature = "dispatch_tree")]
    {
        #[cfg(feature = "debug")]
        {
            let node_strs: Vec<String> = r.iter().map(|v| v.0.clone()).collect();
            crate::info!("All Nodes: [{}]", node_strs.join(", "));
        }
    }

    #[cfg(not(feature = "dispatch_tree"))]
    let r: Vec<_> = program
        .dispatcher
        .iter()
        .map(|disp| {
            let node_str = disp
                .node()
                .to_string()
                .split('.')
                .collect::<Vec<_>>()
                .join(" ");
            (node_str, &**disp)
        })
        .collect();

    #[cfg(not(feature = "dispatch_tree"))]
    {
        #[cfg(feature = "debug")]
        {
            let node_strs: Vec<String> = r.iter().map(|v| v.0.clone()).collect();
            crate::info!("All Nodes: [{}]", node_strs.join(", "));
        }
    }

    r
}
