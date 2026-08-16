use std::fmt::Display;

use crate::{ChainProcess, asset::node::Node};

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
/// # use mingling_core::Node;
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
///     fn node(&self) -> Node {
///         Node::default().join("greet")
///     }
///
///     fn begin(&self, args: Vec<String>) -> ChainProcess<ThisProgram> {
///         Routable::to_chain(Foo { args })
///     }
///
///     fn clone_dispatcher(&self) -> Box<dyn Dispatcher<ThisProgram>> {
///         Box::new(CMDGreet)
///     }
/// }
/// ```
pub trait Dispatcher<C> {
    /// Get the node of this Dispatcher, used to tell the program loop which arguments should be handled by this Dispatcher
    ///
    /// Example:
    ///
    /// ```
    /// # use mingling_core::ChainProcess;
    /// # use mingling_core::Dispatcher;
    /// # use mingling_core::Grouped;
    /// # use mingling_core::Routable;
    /// # use mingling_core::Node;
    /// # use mingling_core::MockProgramCollect as ThisProgram;
    /// # unsafe impl Grouped<ThisProgram> for Foo {
    /// # fn member_id() -> ThisProgram { ThisProgram::Foo }
    /// # }
    /// # struct CMDGreet;
    /// # struct Foo {
    /// #     args: Vec<String>
    /// # }
    /// # impl Dispatcher<ThisProgram> for CMDGreet {
    /// fn node(&self) -> Node {
    ///     // Construct the node
    ///     Node::default().join("greet")
    /// }
    /// #     fn begin(&self, args: Vec<String>) -> ChainProcess<ThisProgram> {
    /// #         Routable::to_chain(Foo { args })
    /// #     }
    /// #     fn clone_dispatcher(&self) -> Box<dyn Dispatcher<ThisProgram>> {
    /// #         Box::new(CMDGreet)
    /// #     }
    /// # }
    /// ```
    fn node(&self) -> Node;

    /// Begin logic, receives the remaining arguments after the prefix has been stripped
    ///
    /// Example:
    ///
    /// ```
    /// # use mingling_core::ChainProcess;
    /// # use mingling_core::Dispatcher;
    /// # use mingling_core::Grouped;
    /// # use mingling_core::Routable;
    /// # use mingling_core::Node;
    /// # use mingling_core::MockProgramCollect as ThisProgram;
    /// # unsafe impl Grouped<ThisProgram> for Foo {
    /// # fn member_id() -> ThisProgram { ThisProgram::Foo }
    /// # }
    /// # struct CMDGreet;
    /// # struct Foo {
    /// #     args: Vec<String>
    /// # }
    /// # impl Dispatcher<ThisProgram> for CMDGreet {
    /// #     fn node(&self) -> Node {
    /// #         Node::default().join("greet")
    /// #     }
    /// fn begin(&self, args: Vec<String>) -> ChainProcess<ThisProgram> {
    ///     // Create Foo from args and route it to the next chain
    ///     Routable::to_chain(Foo { args })
    /// }
    /// #     fn clone_dispatcher(&self) -> Box<dyn Dispatcher<ThisProgram>> {
    /// #         Box::new(CMDGreet)
    /// #     }
    /// # }
    /// ```
    fn begin(&self, args: Vec<String>) -> ChainProcess<C>;

    /// Clone the dispatcher's Box for dynamic dispatch
    ///
    /// Example:
    ///
    /// ```
    /// # use mingling_core::ChainProcess;
    /// # use mingling_core::Dispatcher;
    /// # use mingling_core::Grouped;
    /// # use mingling_core::Routable;
    /// # use mingling_core::Node;
    /// # use mingling_core::MockProgramCollect as ThisProgram;
    /// # unsafe impl Grouped<ThisProgram> for Foo {
    /// # fn member_id() -> ThisProgram { ThisProgram::Foo }
    /// # }
    /// # struct CMDGreet;
    /// # struct Foo {
    /// #     args: Vec<String>
    /// # }
    /// # impl Dispatcher<ThisProgram> for CMDGreet {
    /// #     fn node(&self) -> Node {
    /// #         Node::default().join("greet")
    /// #     }
    /// #     fn begin(&self, args: Vec<String>) -> ChainProcess<ThisProgram> {
    /// #         Routable::to_chain(Foo { args })
    /// #     }
    /// fn clone_dispatcher(&self) -> Box<dyn Dispatcher<ThisProgram>> {
    ///     // Create a new Box
    ///     Box::new(CMDGreet)
    /// }
    /// # }
    /// ```
    fn clone_dispatcher(&self) -> Box<dyn Dispatcher<C>>;
}

impl<G> Clone for Box<dyn Dispatcher<G>>
where
    G: Display,
{
    fn clone(&self) -> Self {
        self.clone_dispatcher()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ChainProcess;
    use std::fmt::Display;
    #[derive(Clone)]
    struct MockDispatcher {
        name: &'static str,
    }

    impl<C: Display> Dispatcher<C> for MockDispatcher {
        fn node(&self) -> crate::asset::node::Node {
            self.name.into()
        }

        fn begin(&self, _args: Vec<String>) -> ChainProcess<C> {
            unimplemented!("not used in these tests")
        }

        fn clone_dispatcher(&self) -> Box<dyn Dispatcher<C>> {
            Box::new(self.clone())
        }
    }
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[allow(dead_code)]
    enum MockG {
        A,
    }

    impl Display for MockG {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "A")
        }
    }

    #[test]
    fn test_box_clone_dispatcher() {
        let disp: Box<dyn Dispatcher<MockG>> = Box::new(MockDispatcher { name: "clonable" });
        let cloned = disp.clone_dispatcher();
        assert_eq!(cloned.node().to_string(), "clonable");
    }
}
