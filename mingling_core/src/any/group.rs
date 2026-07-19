use crate::{AnyOutput, ChainProcess, ProgramCollect, Routable};

/// Used to mark a type with a unique enum ID, assisting dynamic dispatch
///
/// **Note:** Unlike earlier versions, `Grouped` no longer requires `Serialize`
/// even when the `structural_renderer` feature is enabled. Structured output is
/// controlled separately via the \[`StructuralData`\] trait.
pub trait Grouped<Group>
where
    Self: Sized + 'static,
{
    /// Returns the specific enum value representing its ID within that enum
    fn member_id() -> Group;
}

impl<T, C> Routable<C> for T
where
    C: ProgramCollect<Enum = C>,
    T: Grouped<C> + Send,
{
    fn to_chain(self) -> ChainProcess<C> {
        AnyOutput::new(self).route_chain()
    }

    fn to_render(self) -> ChainProcess<C> {
        AnyOutput::new(self).route_renderer()
    }
}
