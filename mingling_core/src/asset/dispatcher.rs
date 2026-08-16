use crate::ChainProcess;

/// The entry logic of the Mingling program
///
/// Dispatcher is the first stop for args after they enter the program:
/// it is used to wrap the user's raw args into an initial [`ChainProcess`] and feed them into the program loop
///
/// # Manual impl
///
/// ```
/// # use mingling_core::ChainProcess;
/// # use mingling_core::Dispatcher;
/// # use mingling_core::Grouped;
/// # use mingling_core::Routable;
/// # use mingling_core::MockProgramCollect as ThisProgram;
/// # unsafe impl Grouped<ThisProgram> for Foo {
/// # fn member_id() -> ThisProgram { ThisProgram::Foo }
/// # }
/// struct CMDGreet;
/// struct Foo {
///     args: Vec<String>
/// }
///
/// impl Dispatcher<ThisProgram> for CMDGreet {
///     fn begin(&self, args: Vec<String>) -> ChainProcess<ThisProgram> {
///         Routable::to_chain(Foo { args })
///     }
/// }
/// ```
pub trait Dispatcher<C> {
    /// Begin logic, receives the remaining arguments after the command prefix
    /// has been stripped
    ///
    /// Example:
    ///
    /// ```
    /// # use mingling_core::ChainProcess;
    /// # use mingling_core::Dispatcher;
    /// # use mingling_core::Grouped;
    /// # use mingling_core::Routable;
    /// # use mingling_core::MockProgramCollect as ThisProgram;
    /// # unsafe impl Grouped<ThisProgram> for Foo {
    /// # fn member_id() -> ThisProgram { ThisProgram::Foo }
    /// # }
    /// # struct CMDGreet;
    /// # struct Foo {
    /// #     args: Vec<String>
    /// # }
    /// # impl Dispatcher<ThisProgram> for CMDGreet {
    /// fn begin(&self, args: Vec<String>) -> ChainProcess<ThisProgram> {
    ///     // Create Foo from args and route it to the next chain
    ///     Routable::to_chain(Foo { args })
    /// }
    /// # }
    /// ```
    fn begin(&self, args: Vec<String>) -> ChainProcess<C>;
}
