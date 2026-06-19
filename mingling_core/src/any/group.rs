use crate::{AnyOutput, ChainProcess};

/// Used to mark a type with a unique enum ID, assisting dynamic dispatch
pub trait Groupped<Group> {
    /// Returns the specific enum value representing its ID within that enum
    fn member_id() -> Group;

    /// Converts the grouped item into a `ChainProcess` directed to the chain route.
    ///
    /// This wraps the item into an `AnyOutput` and routes it to the chain processing pipeline.
    fn to_chain(self) -> ChainProcess<Group> {
        AnyOutput::new(self).route_chain()
    }

    /// Converts the grouped item into a `ChainProcess` directed to the render route.
    ///
    /// This wraps the item into an `AnyOutput` and routes it to the render processing pipeline.
    fn to_render(self) -> ChainProcess<Group> {
        AnyOutput::new(self).route_renderer()
    }
}
