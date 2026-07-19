use crate::ChainProcess;

/// Provides routing capabilities for converting an item into a `ChainProcess`
/// directed to either the chain or render processing pipeline.
///
/// This trait enables items to be dispatched to different processing routes
/// (chain or render) by wrapping them into an `AnyOutput` and routing them
/// through the appropriate pipeline.
pub trait Routable<Group>
where
    Self: Sized + 'static,
{
    /// Converts the routable item into a `ChainProcess` directed to the chain route.
    ///
    /// This wraps the item into an `AnyOutput` and routes it to the chain processing pipeline.
    fn to_chain(self) -> ChainProcess<Group>;

    /// Converts the routable item into a `ChainProcess` directed to the render route.
    ///
    /// This wraps the item into an `AnyOutput` and routes it to the render processing pipeline.
    fn to_render(self) -> ChainProcess<Group>;
}
