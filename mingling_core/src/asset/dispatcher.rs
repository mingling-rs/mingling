use std::fmt::Display;

use crate::{ChainProcess, Program, ProgramCollect, asset::node::Node};

/// Dispatches user input commands to specific [`ChainProcess`](./enum.ChainProcess.html)
///
/// Note: If you are using [mingling_macros](https://crates.io/crates/mingling_macros),
/// you can use the `dispatcher!("node.subnode", CommandType => Entry)` macro to declare a `Dispatcher`
pub trait Dispatcher<C> {
    /// Returns a command node for matching user input
    fn node(&self) -> Node;

    /// Returns a [`ChainProcess`](./enum.ChainProcess.html) based on user input arguments,
    /// to be sent to the specific invocation
    fn begin(&self, args: Vec<String>) -> ChainProcess<C>;

    /// Clones the current dispatcher for implementing the `Clone` trait
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

impl<C> Program<C>
where
    C: ProgramCollect<Enum = C>,
{
    /// Adds a dispatcher to the program.
    #[cfg_attr(
        feature = "dispatch_tree",
        deprecated(
            note = "When the `dispatch_tree` feature is enabled, the `dispatcher` field no longer exists inside Program. All types are collected at compile time by the `gen_program!()` macro, so the `with_dispatcher` function is no longer needed"
        )
    )]
    pub fn with_dispatcher<Disp>(&mut self, dispatcher: Disp) -> &mut Self
    where
        Disp: Dispatcher<C> + Send + Sync + 'static,
    {
        #[cfg(not(feature = "dispatch_tree"))]
        {
            self.dispatcher.push(Box::new(dispatcher));
        }
        #[cfg(feature = "dispatch_tree")]
        {
            let _ = dispatcher;
        }
        self
    }

    /// Add some dispatchers to the program.
    #[cfg_attr(
        feature = "dispatch_tree",
        deprecated(
            note = "When the `dispatch_tree` feature is enabled, the `dispatcher` field no longer exists inside Program. All types are collected at compile time by the `gen_program!()` macro, so the `with_dispatcher` function is no longer needed"
        )
    )]
    pub fn with_dispatchers<D>(&mut self, dispatchers: D) -> &mut Self
    where
        D: Into<Dispatchers<C>>,
    {
        #[cfg(not(feature = "dispatch_tree"))]
        {
            let dispatchers = dispatchers.into();
            self.dispatcher.extend(dispatchers.dispatcher);
        }
        #[cfg(feature = "dispatch_tree")]
        {
            let _ = dispatchers;
        }
        self
    }
}

/// A collection of dispatchers.
///
/// This struct holds a vector of boxed `Dispatcher` trait objects,
/// allowing multiple dispatchers to be grouped together and passed
/// to the program via `Program::with_dispatchers`.
/// A collection of dispatchers.
///
/// This struct holds a vector of boxed `Dispatcher` trait objects,
/// allowing multiple dispatchers to be grouped together and passed
/// to the program via `Program::with_dispatchers`.
pub struct Dispatchers<G> {
    dispatcher: Vec<Box<dyn Dispatcher<G> + Send + Sync + 'static>>,
}

impl<G> From<Vec<Box<dyn Dispatcher<G> + Send + Sync>>> for Dispatchers<G> {
    fn from(dispatcher: Vec<Box<dyn Dispatcher<G> + Send + Sync>>) -> Self {
        Self { dispatcher }
    }
}

impl<G> From<Box<dyn Dispatcher<G> + Send + Sync>> for Dispatchers<G> {
    fn from(dispatcher: Box<dyn Dispatcher<G> + Send + Sync>) -> Self {
        Self {
            dispatcher: vec![dispatcher],
        }
    }
}

impl<D, G> From<(D,)> for Dispatchers<G>
where
    D: Dispatcher<G> + Send + Sync + 'static,
    G: Display,
{
    fn from(dispatcher: (D,)) -> Self {
        Self {
            dispatcher: vec![Box::new(dispatcher.0)],
        }
    }
}

impl<D1, D2, G> From<(D1, D2)> for Dispatchers<G>
where
    D1: Dispatcher<G> + Send + Sync + 'static,
    D2: Dispatcher<G> + Send + Sync + 'static,
    G: Display,
{
    fn from(dispatchers: (D1, D2)) -> Self {
        Self {
            dispatcher: vec![Box::new(dispatchers.0), Box::new(dispatchers.1)],
        }
    }
}

impl<D1, D2, D3, G> From<(D1, D2, D3)> for Dispatchers<G>
where
    D1: Dispatcher<G> + Send + Sync + 'static,
    D2: Dispatcher<G> + Send + Sync + 'static,
    D3: Dispatcher<G> + Send + Sync + 'static,
    G: Display,
{
    fn from(dispatchers: (D1, D2, D3)) -> Self {
        Self {
            dispatcher: vec![
                Box::new(dispatchers.0),
                Box::new(dispatchers.1),
                Box::new(dispatchers.2),
            ],
        }
    }
}

impl<D1, D2, D3, D4, G> From<(D1, D2, D3, D4)> for Dispatchers<G>
where
    D1: Dispatcher<G> + Send + Sync + 'static,
    D2: Dispatcher<G> + Send + Sync + 'static,
    D3: Dispatcher<G> + Send + Sync + 'static,
    D4: Dispatcher<G> + Send + Sync + 'static,
    G: Display,
{
    fn from(dispatchers: (D1, D2, D3, D4)) -> Self {
        Self {
            dispatcher: vec![
                Box::new(dispatchers.0),
                Box::new(dispatchers.1),
                Box::new(dispatchers.2),
                Box::new(dispatchers.3),
            ],
        }
    }
}

impl<D1, D2, D3, D4, D5, G> From<(D1, D2, D3, D4, D5)> for Dispatchers<G>
where
    D1: Dispatcher<G> + Send + Sync + 'static,
    D2: Dispatcher<G> + Send + Sync + 'static,
    D3: Dispatcher<G> + Send + Sync + 'static,
    D4: Dispatcher<G> + Send + Sync + 'static,
    D5: Dispatcher<G> + Send + Sync + 'static,
    G: Display,
{
    fn from(dispatchers: (D1, D2, D3, D4, D5)) -> Self {
        Self {
            dispatcher: vec![
                Box::new(dispatchers.0),
                Box::new(dispatchers.1),
                Box::new(dispatchers.2),
                Box::new(dispatchers.3),
                Box::new(dispatchers.4),
            ],
        }
    }
}

impl<D1, D2, D3, D4, D5, D6, G> From<(D1, D2, D3, D4, D5, D6)> for Dispatchers<G>
where
    D1: Dispatcher<G> + Send + Sync + 'static,
    D2: Dispatcher<G> + Send + Sync + 'static,
    D3: Dispatcher<G> + Send + Sync + 'static,
    D4: Dispatcher<G> + Send + Sync + 'static,
    D5: Dispatcher<G> + Send + Sync + 'static,
    D6: Dispatcher<G> + Send + Sync + 'static,
    G: Display,
{
    fn from(dispatchers: (D1, D2, D3, D4, D5, D6)) -> Self {
        Self {
            dispatcher: vec![
                Box::new(dispatchers.0),
                Box::new(dispatchers.1),
                Box::new(dispatchers.2),
                Box::new(dispatchers.3),
                Box::new(dispatchers.4),
                Box::new(dispatchers.5),
            ],
        }
    }
}

impl<D1, D2, D3, D4, D5, D6, D7, G> From<(D1, D2, D3, D4, D5, D6, D7)> for Dispatchers<G>
where
    D1: Dispatcher<G> + Send + Sync + 'static,
    D2: Dispatcher<G> + Send + Sync + 'static,
    D3: Dispatcher<G> + Send + Sync + 'static,
    D4: Dispatcher<G> + Send + Sync + 'static,
    D5: Dispatcher<G> + Send + Sync + 'static,
    D6: Dispatcher<G> + Send + Sync + 'static,
    D7: Dispatcher<G> + Send + Sync + 'static,
    G: Display,
{
    fn from(dispatchers: (D1, D2, D3, D4, D5, D6, D7)) -> Self {
        Self {
            dispatcher: vec![
                Box::new(dispatchers.0),
                Box::new(dispatchers.1),
                Box::new(dispatchers.2),
                Box::new(dispatchers.3),
                Box::new(dispatchers.4),
                Box::new(dispatchers.5),
                Box::new(dispatchers.6),
            ],
        }
    }
}

impl<G> std::ops::Deref for Dispatchers<G> {
    type Target = Vec<Box<dyn Dispatcher<G> + Send + Sync + 'static>>;

    fn deref(&self) -> &Self::Target {
        &self.dispatcher
    }
}

impl<G> From<Dispatchers<G>> for Vec<Box<dyn Dispatcher<G> + Send + Sync + 'static>> {
    fn from(val: Dispatchers<G>) -> Self {
        val.dispatcher
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ChainProcess;
    use std::fmt::Display;

    /// A minimal mock Dispatcher for testing Dispatchers conversions.
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

    /// Minimal mock group for Dispatchers tests
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
    fn test_dispatchers_from_single_tuple() {
        let disp = MockDispatcher { name: "foo" };
        let dispatchers: Dispatchers<MockG> = Dispatchers::from((disp,));
        assert_eq!(dispatchers.dispatcher.len(), 1);
    }

    #[test]
    fn test_dispatchers_from_two_tuple() {
        let d1 = MockDispatcher { name: "a" };
        let d2 = MockDispatcher { name: "b" };
        let dispatchers: Dispatchers<MockG> = Dispatchers::from((d1, d2));
        assert_eq!(dispatchers.dispatcher.len(), 2);
    }

    #[test]
    fn test_dispatchers_from_three_tuple() {
        let d1 = MockDispatcher { name: "x" };
        let d2 = MockDispatcher { name: "y" };
        let d3 = MockDispatcher { name: "z" };
        let dispatchers: Dispatchers<MockG> = Dispatchers::from((d1, d2, d3));
        assert_eq!(dispatchers.dispatcher.len(), 3);
    }

    #[test]
    fn test_dispatchers_from_four_tuple() {
        let d1 = MockDispatcher { name: "1" };
        let d2 = MockDispatcher { name: "2" };
        let d3 = MockDispatcher { name: "3" };
        let d4 = MockDispatcher { name: "4" };
        let dispatchers: Dispatchers<MockG> = Dispatchers::from((d1, d2, d3, d4));
        assert_eq!(dispatchers.dispatcher.len(), 4);
    }

    #[test]
    fn test_dispatchers_from_five_tuple() {
        let d1 = MockDispatcher { name: "a" };
        let d2 = MockDispatcher { name: "b" };
        let d3 = MockDispatcher { name: "c" };
        let d4 = MockDispatcher { name: "d" };
        let d5 = MockDispatcher { name: "e" };
        let dispatchers: Dispatchers<MockG> = Dispatchers::from((d1, d2, d3, d4, d5));
        assert_eq!(dispatchers.dispatcher.len(), 5);
    }

    #[test]
    fn test_dispatchers_from_six_tuple() {
        let d1 = MockDispatcher { name: "a" };
        let d2 = MockDispatcher { name: "b" };
        let d3 = MockDispatcher { name: "c" };
        let d4 = MockDispatcher { name: "d" };
        let d5 = MockDispatcher { name: "e" };
        let d6 = MockDispatcher { name: "f" };
        let dispatchers: Dispatchers<MockG> = Dispatchers::from((d1, d2, d3, d4, d5, d6));
        assert_eq!(dispatchers.dispatcher.len(), 6);
    }

    #[test]
    fn test_dispatchers_from_seven_tuple() {
        let d1 = MockDispatcher { name: "a" };
        let d2 = MockDispatcher { name: "b" };
        let d3 = MockDispatcher { name: "c" };
        let d4 = MockDispatcher { name: "d" };
        let d5 = MockDispatcher { name: "e" };
        let d6 = MockDispatcher { name: "f" };
        let d7 = MockDispatcher { name: "g" };
        let dispatchers: Dispatchers<MockG> = Dispatchers::from((d1, d2, d3, d4, d5, d6, d7));
        assert_eq!(dispatchers.dispatcher.len(), 7);
    }

    #[test]
    fn test_dispatchers_from_vec_of_boxed() {
        let d1: Box<dyn Dispatcher<MockG> + Send + Sync> = Box::new(MockDispatcher { name: "a" });
        let d2: Box<dyn Dispatcher<MockG> + Send + Sync> = Box::new(MockDispatcher { name: "b" });
        let dispatchers: Dispatchers<MockG> = vec![d1, d2].into();
        assert_eq!(dispatchers.dispatcher.len(), 2);
    }

    #[test]
    fn test_dispatchers_from_single_boxed() {
        let d: Box<dyn Dispatcher<MockG> + Send + Sync> = Box::new(MockDispatcher { name: "x" });
        let dispatchers: Dispatchers<MockG> = d.into();
        assert_eq!(dispatchers.dispatcher.len(), 1);
    }

    #[test]
    fn test_dispatchers_deref() {
        let disp = MockDispatcher { name: "test" };
        let dispatchers: Dispatchers<MockG> = Dispatchers::from((disp,));
        let inner: &Vec<Box<dyn Dispatcher<MockG> + Send + Sync + 'static>> = &dispatchers;
        assert_eq!(inner.len(), 1);
    }

    #[test]
    fn test_dispatchers_into_vec() {
        let disp = MockDispatcher { name: "foo" };
        let dispatchers: Dispatchers<MockG> = Dispatchers::from((disp,));
        let vec: Vec<Box<dyn Dispatcher<MockG> + Send + Sync + 'static>> = dispatchers.into();
        assert_eq!(vec.len(), 1);
    }

    #[test]
    fn test_box_clone_dispatcher() {
        let disp: Box<dyn Dispatcher<MockG>> = Box::new(MockDispatcher { name: "clonable" });
        let cloned = disp.clone_dispatcher();
        assert_eq!(cloned.node().to_string(), "clonable");
    }
}
