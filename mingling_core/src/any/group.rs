use crate::{AnyOutput, ChainProcess, ProgramCollect, Routable};

/// Used to mark a type with a unique enum ID, assisting dynamic dispatch
///
/// # Safety
///
/// The returned `Group` value is an enum variant created by `register_type!` when
/// registering the type's ID. Whether the variant matches correctly is guaranteed
/// by `Grouped derive` or macros like `pack!`. If implemented manually, and the
/// type name written in `member_id()` does not match the actually registered type,
/// dispatching to that type will result in **100% undefined behavior**.
pub unsafe trait Grouped<Group>
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
