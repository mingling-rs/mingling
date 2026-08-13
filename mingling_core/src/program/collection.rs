// Doc Not Optimize
#[cfg(feature = "async")]
use std::pin::Pin;

#[cfg(feature = "dispatch_tree")]
use crate::Dispatcher;

use crate::{AnyOutput, ChainProcess, Grouped, RenderResult};

#[cfg(feature = "structural_renderer")]
use crate::{StructuralRendererSetting, error::StructuralRendererSerializeError};

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
    /// Error type when a dispatcher is not found for the given member
    type EntryFallback: Grouped<Self::Enum>;

    /// Error type when a renderer is not found for the given member
    type ErrorRendererNotFound: Grouped<Self::Enum>;

    /// Result type for an empty chain result
    ///
    /// When the `extras` feature is enabled,
    /// you can use the `empty_result!()` macro to create this
    type ResultEmpty: Grouped<Self::Enum>;

    /// Use a prefix tree to quickly match arguments and dispatch to an Entry
    #[cfg(feature = "dispatch_tree")]
    fn dispatch_args_trie(
        raw: &[String],
    ) -> Result<AnyOutput<Self::Enum>, crate::error::ProgramInternalExecuteError>;

    #[cfg(not(feature = "dispatch_tree"))]
    /// Use a prefix tree to quickly match arguments and dispatch to an Entry
    ///
    /// # Errors
    ///
    /// Returns an error if the program fails to execute the given arguments.
    fn dispatch_args_trie(
        _raw: &[String],
    ) -> Result<AnyOutput<Self::Enum>, crate::error::ProgramInternalExecuteError> {
        unreachable!()
    }

    /// Get all registered dispatcher names from the program
    #[cfg(feature = "dispatch_tree")]
    fn get_nodes() -> Vec<(String, &'static (dyn Dispatcher<Self::Enum> + Send + Sync))>;

    /// Build an [`AnyOutput`](./struct.AnyOutput.html) to indicate that a renderer was not found
    fn build_renderer_not_found(member_id: Self::Enum) -> AnyOutput<Self::Enum>;

    /// Build an [`AnyOutput`](./struct.AnyOutput.html) to indicate that a dispatcher was not found
    fn build_entry_fallback(args: Vec<String>) -> AnyOutput<Self::Enum>;

    /// Build an [`AnyOutput`](./struct.AnyOutput.html) to indicate that the chain returned an empty result
    fn build_empty_result() -> AnyOutput<Self::Enum>;

    /// Render the input [`AnyOutput`](./struct.AnyOutput.html)
    fn render(any: AnyOutput<Self::Enum>) -> RenderResult;

    /// Render help for Entry
    fn render_help(any: AnyOutput<Self::Enum>) -> RenderResult;

    /// Retrieves compile-time registered metadata of type `T` for the given
    /// enum member, if any was registered via `#[metadata(Entry)]`.
    ///
    /// Returns `None` when no metadata of type `T` has been registered for the
    /// provided member, or when the requested `T` does not match the registered
    /// metadata type. The concrete implementation of this method is generated
    /// by the `gen_program!` macro.
    fn get_metadata<T: 'static>(member_id: Self::Enum) -> Option<T> {
        let _ = member_id;
        None
    }

    /// Find a matching chain to continue execution based on the input [`AnyOutput`](./struct.AnyOutput.html), returning a new [`AnyOutput`](./struct.AnyOutput.html)
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

    /// Perform structural rendering and presentation of any type
    ///
    /// # Errors
    ///
    /// Returns `Err(StructuralRendererSerializeError)` if serialization of the
    /// output value fails.
    #[cfg(feature = "structural_renderer")]
    fn structural_render(
        any: AnyOutput<Self::Enum>,
        setting: &StructuralRendererSetting,
    ) -> Result<RenderResult, StructuralRendererSerializeError>;
}
