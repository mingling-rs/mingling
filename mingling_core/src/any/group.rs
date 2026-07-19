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

    /// Converts the grouped item into a `ChainProcess` directed to the chain route.
    ///
    /// This wraps the item into an `AnyOutput` and routes it to the chain processing pipeline.
    fn to_chain(self) -> ChainProcess<Group>
    where
        Self: Send,
    {
        AnyOutput::new(self).route_chain()
    }

    /// Converts the grouped item into a `ChainProcess` directed to the render route.
    ///
    /// This wraps the item into an `AnyOutput` and routes it to the render processing pipeline.
    fn to_render(self) -> ChainProcess<Group>
    where
        Self: Send,
    {
        AnyOutput::new(self).route_renderer()
    }
}

impl<T, C> Routable<C> for T
where
    C: ProgramCollect<Enum = C>,
    T: Grouped<C> + Send,
{
    fn to_chain(self) -> ChainProcess<C> {
        T::to_chain(self)
    }

    fn to_render(self) -> ChainProcess<C> {
        T::to_render(self)
    }
}
