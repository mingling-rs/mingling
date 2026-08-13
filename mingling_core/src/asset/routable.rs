use crate::{AnyOutput, ChainProcess, Grouped, ProgramCollect};

/// Represents a type that can be routed within a group.
///
/// Used to indicate that a group member can be routed into another [`ChainProcess`]
/// within the execution logic of a [`Chain`].
///
/// # Blanket impl
///
/// When a type implements [`Grouped`], it automatically gets a corresponding [`Routable`]
/// implementation, meaning all types deriving [`Grouped`] can flow through the program loop.
///
/// # Reference
///
/// You can use the [`routeify`](https://docs.rs/mingling/latest/mingling/macros/attr.routeify.html)
/// macro and the [`route!`](https://docs.rs/mingling/latest/mingling/macros/macro.route.html) macro
/// to build flexible program execution logic.
///
/// # Example
///
/// ```
/// # use mingling_core::Routable;
/// # use mingling_core::ChainProcess;
/// # use mingling_core::MockProgramCollect as ThisProgram;
/// # use mingling_core::Grouped;
/// # unsafe impl Grouped<ThisProgram> for Foo {
/// #     fn member_id() -> ThisProgram {
/// #         ThisProgram::Foo
/// #     }
/// # }
/// struct Foo;
///
/// // With `Grouped` implemented, the type automatically implements `Routable`
/// // and can be converted into a `ChainProcess` via `to_chain` / `to_render`:
/// fn takes_chain<T: Routable<ThisProgram>>(value: T) -> ChainProcess<ThisProgram> {
///     value.to_chain()
/// }
///
/// # fn main() {
/// #     takes_chain(Foo);
/// # }
/// ```
pub trait Routable<Group>
where
    Self: Sized + 'static,
{
    /// Converts the current type into a [`ChainProcess`] that can be used for execution in a program chain (`Chain`).
    ///
    /// # Return value
    ///
    /// Returns a [`ChainProcess`] wrapping the current value,
    /// which will be scheduled for execution in the program chain.
    ///
    /// # Example
    ///
    /// ```
    /// # use mingling_core::{Grouped, ChainProcess};
    /// # use mingling_core::MockProgramCollect as ThisProgram;
    /// # unsafe impl Grouped<ThisProgram> for StateMyType {
    /// #     fn member_id() -> ThisProgram {
    /// #         ThisProgram::Foo
    /// #     }
    /// # }
    /// use mingling_core::Routable;
    ///
    /// struct StateMyType;
    ///
    /// let my_type = StateMyType;
    /// let process: ChainProcess<ThisProgram> = my_type.to_chain();
    /// ```
    fn to_chain(self) -> ChainProcess<Group>;

    /// Converts the current type into a [`ChainProcess`] that can be used for the rendering pipeline.
    ///
    /// # Return value
    ///
    /// Returns a [`ChainProcess`] wrapping the current value,
    /// which will be scheduled for execution in the rendering pipeline.
    ///
    /// # Example
    ///
    /// ```
    /// # use mingling_core::{Grouped, ChainProcess};
    /// # use mingling_core::MockProgramCollect as ThisProgram;
    /// # unsafe impl Grouped<ThisProgram> for StateMyType {
    /// #     fn member_id() -> ThisProgram {
    /// #         ThisProgram::Foo
    /// #     }
    /// # }
    /// use mingling_core::Routable;
    ///
    /// struct StateMyType;
    ///
    /// let my_type = StateMyType;
    /// let process: ChainProcess<ThisProgram> = my_type.to_render();
    /// ```
    fn to_render(self) -> ChainProcess<Group>;
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
