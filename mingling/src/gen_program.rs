#![allow(unused)]

use mingling_core::ChainProcess;
use mingling_core::Dispatcher;
use mingling_core::Grouped;
use mingling_core::Node;
use mingling_core::ProgramCollect;

/// The next processing step for the current chain.
pub type Next = ChainProcess<ThisProgram>;

/// An enum used to record the set of program type IDs.
///
/// It contains the IDs of all types for this program, registered by the `register_type!` macro.
pub enum ThisProgram {
    /// Indicates that no matching renderer was found for the given output.
    ErrorRendererNotFound,
    /// Indicates that no matching dispatcher was found for the given arguments.
    ErrorDispatcherNotFound,
    /// Indicates that the result is empty.
    ResultEmpty,
    /// Indicates the completion suggestions computed by the program for rendering.
    #[cfg(feature = "comp")]
    CompletionSuggest,
    /// Represents the original context dispatched by the `__comp` command.
    #[cfg(feature = "comp")]
    CompletionContext,
}

/// A struct representing a "renderer not found" error.
///
/// This type is created by the `pack!` macro as a variant of the
/// program's output type set (`ThisProgram`).
pub struct ErrorRendererNotFound {
    /// The name of the renderer that was not found
    inner: String,
}

/// A struct representing a "dispatcher not found" error.
///
/// This type is created by the `pack!` macro as a variant of the
/// program's output type set (`ThisProgram`).
pub struct ErrorDispatcherNotFound {
    /// The arguments provided by the user
    inner: Vec<String>,
}

/// A struct representing an empty result.
pub struct ResultEmpty;

/// A dispatcher representing the subcommand `__comp` itself.
///
/// **You can register it using `with_dispatcher`.**
#[cfg(feature = "comp")]
pub struct CMDCompletion;

/// Represents the completion context, containing the arguments provided by the user.
///
/// This struct holds the raw command-line arguments that were passed to the `__comp`
/// subcommand, which will be used to compute completion suggestions.
///
/// This type is created by the `pack!` macro as a variant of the
/// program's output type set (`ThisProgram`).
#[cfg(feature = "comp")]
pub struct CompletionContext {
    /// The arguments provided by the user
    inner: Vec<String>,
}

/// Represents a completion suggestion result.
///
/// This struct holds a pair of the shell context and the computed suggestion data,
/// which together provide the information needed to render completion candidates
/// to the user's shell.
///
/// This type is created by the `pack!` macro as a variant of the
/// program's output type set (`ThisProgram`).
#[cfg(feature = "comp")]
pub struct CompletionSuggest {
    /// The shell context and the computed suggestion data
    inner: (mingling_core::ShellContext, mingling_core::Suggest),
}

#[cfg(feature = "comp")]
impl Dispatcher<ThisProgram> for CMDCompletion {
    fn node(&self) -> mingling_core::Node {
        Node::default().join(mingling_core::COMPLETION_SUBCOMMAND)
    }

    fn begin(&self, args: Vec<String>) -> ChainProcess<ThisProgram> {
        use mingling_core::AnyOutput;
        AnyOutput::new(CompletionContext { inner: args }).route_chain()
    }

    fn clone_dispatcher(&self) -> Box<dyn Dispatcher<ThisProgram>> {
        todo!()
    }
}

// SAFETY: These implementations are provided for demonstration purposes only.
// The `member_id()` implementations map each type to its corresponding variant
// in the `ThisProgram` enum, and the IDs correctly correspond to the actual types.
// However, these are marked `unsafe` because the `Grouped` trait requires the
// implementor to guarantee that the type is the only one associated with the
// given enum variant — a guarantee that should be carefully verified in production code.
unsafe impl Grouped<ThisProgram> for ErrorRendererNotFound {
    fn member_id() -> ThisProgram {
        ThisProgram::ErrorRendererNotFound
    }
}

// SAFETY: These implementations are provided for demonstration purposes only.
// The `member_id()` implementations map each type to its corresponding variant
// in the `ThisProgram` enum, and the IDs correctly correspond to the actual types.
// However, these are marked `unsafe` because the `Grouped` trait requires the
// implementor to guarantee that the type is the only one associated with the
// given enum variant — a guarantee that should be carefully verified in production code.
unsafe impl Grouped<ThisProgram> for ErrorDispatcherNotFound {
    fn member_id() -> ThisProgram {
        ThisProgram::ErrorDispatcherNotFound
    }
}

// SAFETY: These implementations are provided for demonstration purposes only.
// The `member_id()` implementations map each type to its corresponding variant
// in the `ThisProgram` enum, and the IDs correctly correspond to the actual types.
// However, these are marked `unsafe` because the `Grouped` trait requires the
// implementor to guarantee that the type is the only one associated with the
// given enum variant — a guarantee that should be carefully verified in production code.
unsafe impl Grouped<ThisProgram> for ResultEmpty {
    fn member_id() -> ThisProgram {
        ThisProgram::ResultEmpty
    }
}

// SAFETY: These implementations are provided for demonstration purposes only.
// The `member_id()` implementations map each type to its corresponding variant
// in the `ThisProgram` enum, and the IDs correctly correspond to the actual types.
// However, these are marked `unsafe` because the `Grouped` trait requires the
// implementor to guarantee that the type is the only one associated with the
// given enum variant — a guarantee that should be carefully verified in production code.
#[cfg(feature = "comp")]
unsafe impl Grouped<ThisProgram> for CompletionContext {
    fn member_id() -> ThisProgram {
        ThisProgram::CompletionContext
    }
}

// SAFETY: These implementations are provided for demonstration purposes only.
// The `member_id()` implementations map each type to its corresponding variant
// in the `ThisProgram` enum, and the IDs correctly correspond to the actual types.
// However, these are marked `unsafe` because the `Grouped` trait requires the
// implementor to guarantee that the type is the only one associated with the
// given enum variant — a guarantee that should be carefully verified in production code.
#[cfg(feature = "comp")]
unsafe impl Grouped<ThisProgram> for CompletionSuggest {
    fn member_id() -> ThisProgram {
        ThisProgram::CompletionSuggest
    }
}

impl ProgramCollect for ThisProgram {
    type Enum = ThisProgram;

    type ErrorDispatcherNotFound = ErrorDispatcherNotFound;

    type ErrorRendererNotFound = ErrorRendererNotFound;

    type ResultEmpty = ResultEmpty;

    fn build_renderer_not_found(_member_id: Self::Enum) -> mingling_core::AnyOutput<Self::Enum> {
        todo!()
    }

    fn build_dispatcher_not_found(_args: Vec<String>) -> mingling_core::AnyOutput<Self::Enum> {
        todo!()
    }

    fn build_empty_result() -> mingling_core::AnyOutput<Self::Enum> {
        todo!()
    }

    fn render(_any: mingling_core::AnyOutput<Self::Enum>) -> mingling_core::RenderResult {
        todo!()
    }

    fn render_help(_any: mingling_core::AnyOutput<Self::Enum>) -> mingling_core::RenderResult {
        todo!()
    }

    #[cfg(feature = "async")]
    fn do_chain(
        _any: mingling_core::AnyOutput<Self::Enum>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = mingling_core::ChainProcess<Self::Enum>> + Send>,
    > {
        todo!()
    }

    #[cfg(not(feature = "async"))]
    fn do_chain(
        _any: mingling_core::AnyOutput<Self::Enum>,
    ) -> mingling_core::ChainProcess<Self::Enum> {
        todo!()
    }

    #[cfg(feature = "comp")]
    fn do_comp(
        _any: &mingling_core::AnyOutput<Self::Enum>,
        _ctx: &mingling_core::ShellContext,
    ) -> mingling_core::Suggest {
        todo!()
    }

    fn has_renderer(_any: &mingling_core::AnyOutput<Self::Enum>) -> bool {
        todo!()
    }

    fn has_chain(_any: &mingling_core::AnyOutput<Self::Enum>) -> bool {
        todo!()
    }

    #[cfg(feature = "structural_renderer")]
    fn structural_render(
        _any: mingling_core::AnyOutput<Self::Enum>,
        _setting: &mingling_core::StructuralRendererSetting,
    ) -> Result<mingling_core::RenderResult, mingling_core::error::StructuralRendererSerializeError>
    {
        todo!()
    }
}
