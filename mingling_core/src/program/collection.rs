#[cfg(feature = "async")]
use std::pin::Pin;

#[cfg(feature = "dispatch_tree")]
use crate::Dispatcher;

use crate::{AnyOutput, ChainProcess, Groupped, RenderResult};

#[cfg(feature = "general_renderer")]
use crate::{GeneralRendererSetting, error::GeneralRendererSerializeError};

#[cfg(feature = "comp")]
use crate::{ShellContext, Suggest};

mod mock;
pub use mock::*;

/// Collected program context
///
/// Note: It is recommended to use the `gen_program!()` macro from [mingling_macros](https://crates.io/crates/mingling_macros) to automatically create this type
pub trait ProgramCollect {
    /// Enum type representing internal IDs for the program
    type Enum;
    type ErrorDispatcherNotFound: Groupped<Self::Enum>;
    type ErrorRendererNotFound: Groupped<Self::Enum>;
    type ResultEmpty: Groupped<Self::Enum>;

    /// Use a prefix tree to quickly match arguments and dispatch to an Entry
    #[cfg(feature = "dispatch_tree")]
    fn dispatch_args_trie(
        raw: &[String],
    ) -> Result<AnyOutput<Self::Enum>, crate::error::ProgramInternalExecuteError>;

    /// Get all registered dispatcher names from the program
    #[cfg(feature = "dispatch_tree")]
    fn get_nodes() -> Vec<(String, &'static (dyn Dispatcher<Self::Enum> + Send + Sync))>;

    /// Build an [`AnyOutput`](./struct.AnyOutput.html) to indicate that a renderer was not found
    fn build_renderer_not_found(member_id: Self::Enum) -> AnyOutput<Self::Enum>;

    /// Build an [`AnyOutput`](./struct.AnyOutput.html) to indicate that a dispatcher was not found
    fn build_dispatcher_not_found(args: Vec<String>) -> AnyOutput<Self::Enum>;

    /// Build an [`AnyOutput`](./struct.AnyOutput.html) to indicate that the chain returned an empty result
    fn build_empty_result() -> AnyOutput<Self::Enum>;

    /// Render the input [`AnyOutput`](./struct.AnyOutput.html)
    fn render(any: AnyOutput<Self::Enum>, r: &mut RenderResult);

    /// Render help for Entry
    fn render_help(any: AnyOutput<Self::Enum>, r: &mut RenderResult);

    /// Find a matching chain to continue execution based on the input [AnyOutput](./struct.AnyOutput.html), returning a new [AnyOutput](./struct.AnyOutput.html)
    #[cfg(feature = "async")]
    fn do_chain(
        any: AnyOutput<Self::Enum>,
    ) -> Pin<Box<dyn Future<Output = ChainProcess<Self::Enum>> + Send>>;

    /// Find a matching chain to continue execution based on the input [`AnyOutput`](./struct.AnyOutput.html), returning a new [`AnyOutput`](./struct.AnyOutput.html)
    #[cfg(not(feature = "async"))]
    fn do_chain(any: AnyOutput<Self::Enum>) -> ChainProcess<Self::Enum>;

    /// Match and execute specific completion logic based on any Entry
    #[cfg(feature = "comp")]
    fn do_comp(any: &AnyOutput<Self::Enum>, ctx: &ShellContext) -> Suggest;

    /// Whether the program has a renderer that can handle the current [`AnyOutput`](./struct.AnyOutput.html)
    fn has_renderer(any: &AnyOutput<Self::Enum>) -> bool;

    /// Whether the program has a chain that can handle the current [`AnyOutput`](./struct.AnyOutput.html)
    fn has_chain(any: &AnyOutput<Self::Enum>) -> bool;

    /// Perform general rendering and presentation of any type
    ///
    /// # Errors
    ///
    /// Returns `Err(GeneralRendererSerializeError)` if serialization of the
    /// output value fails.
    #[cfg(feature = "general_renderer")]
    fn general_render(
        any: AnyOutput<Self::Enum>,
        setting: &GeneralRendererSetting,
    ) -> Result<RenderResult, GeneralRendererSerializeError>;
}
