#[cfg(feature = "async")]
use std::pin::Pin;

#[cfg(feature = "dispatch_tree")]
use crate::Dispatcher;

use crate::{AnyOutput, ChainProcess, Groupped, ProgramCollect, RenderResult};

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

impl Groupped<MockProgramCollect> for MockProgramCollect {
    fn member_id() -> MockProgramCollect {
        MockProgramCollect::Foo
    }
}

impl ProgramCollect for MockProgramCollect {
    type Enum = MockProgramCollect;
    type ErrorDispatcherNotFound = MockProgramCollect;
    type ErrorRendererNotFound = MockProgramCollect;
    type ResultEmpty = MockProgramCollect;

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

    fn build_dispatcher_not_found(_args: Vec<String>) -> AnyOutput<Self::Enum> {
        unreachable!()
    }

    fn build_empty_result() -> AnyOutput<Self::Enum> {
        unreachable!()
    }

    fn render(_any: AnyOutput<Self::Enum>, _r: &mut RenderResult) {
        unreachable!()
    }

    fn render_help(_any: AnyOutput<Self::Enum>, _r: &mut RenderResult) {
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
