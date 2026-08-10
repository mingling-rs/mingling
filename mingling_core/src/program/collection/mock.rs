#[cfg(feature = "async")]
use std::pin::Pin;

#[cfg(feature = "dispatch_tree")]
use crate::Dispatcher;

use crate::{AnyOutput, ChainProcess, Grouped, ProgramCollect, RenderResult};

#[cfg(feature = "structural_renderer")]
use crate::{StructuralRendererSetting, error::StructuralRendererSerializeError};

#[cfg(feature = "comp")]
use crate::{ShellContext, Suggest};

#[cfg(feature = "structural_renderer")]
use serde::Serialize;

#[cfg_attr(feature = "structural_renderer", derive(Serialize))]
#[allow(unused)]
#[doc(hidden)]
pub enum MockProgramCollect {
    Foo,
    Bar,
}

/// SAFETY: This is a mock type used only for temporary testing.
/// It will never actually enter the macro system.
/// The internal `panic!` ensures that `member_id` will never be executed.
unsafe impl Grouped<Self> for MockProgramCollect {
    fn member_id() -> Self {
        panic!("Attempting to read an unsafe enum type");
    }
}

impl ProgramCollect for MockProgramCollect {
    type Enum = Self;
    type EntryFallback = Self;
    type ErrorRendererNotFound = Self;
    type ResultEmpty = Self;

    #[cfg(feature = "dispatch_tree")]
    fn dispatch_args_trie(
        _raw: &[String],
    ) -> Result<AnyOutput<Self::Enum>, crate::error::ProgramInternalExecuteError> {
        unreachable!()
    }

    #[cfg(feature = "dispatch_tree")]
    fn get_nodes() -> Vec<(String, &'static (dyn Dispatcher<Self::Enum> + Send + Sync))> {
        unreachable!()
    }

    fn build_renderer_not_found(_member_id: Self::Enum) -> AnyOutput<Self::Enum> {
        unreachable!()
    }

    fn build_entry_fallback(_args: Vec<String>) -> AnyOutput<Self::Enum> {
        unreachable!()
    }

    fn build_empty_result() -> AnyOutput<Self::Enum> {
        unreachable!()
    }

    fn render(_any: AnyOutput<Self::Enum>) -> RenderResult {
        unreachable!()
    }

    fn render_help(_any: AnyOutput<Self::Enum>) -> RenderResult {
        unreachable!()
    }

    #[cfg(feature = "async")]
    fn do_chain(
        _any: AnyOutput<Self::Enum>,
    ) -> Pin<Box<dyn Future<Output = ChainProcess<Self::Enum>> + Send>> {
        unreachable!()
    }

    #[cfg(not(feature = "async"))]
    fn do_chain(_any: AnyOutput<Self::Enum>) -> ChainProcess<Self::Enum> {
        unreachable!()
    }

    #[cfg(feature = "comp")]
    fn do_comp(_any: &AnyOutput<Self::Enum>, _ctx: &ShellContext) -> Suggest {
        unreachable!()
    }

    fn has_renderer(_any: &AnyOutput<Self::Enum>) -> bool {
        unreachable!()
    }

    fn has_chain(_any: &AnyOutput<Self::Enum>) -> bool {
        unreachable!()
    }

    #[cfg(feature = "structural_renderer")]
    fn structural_render(
        _any: AnyOutput<Self::Enum>,
        _setting: &StructuralRendererSetting,
    ) -> Result<RenderResult, StructuralRendererSerializeError> {
        unreachable!()
    }
}
