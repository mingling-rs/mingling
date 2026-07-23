use crate::ChainProcess;

#[doc(hidden)]
pub mod error;

/// Takes over a type (G: Previous) and converts it to another [`AnyOutput`](./struct.AnyOutput.html)
pub trait Chain<G> {
    /// The previous type in the chain
    type Previous;

    /// Process the previous value and return a future that resolves to a [`ChainProcess<G>`](./enum.ChainProcess.html)
    #[cfg(feature = "async")]
    fn proc(p: Self::Previous) -> impl Future<Output = ChainProcess<G>> + Send;

    /// Process the previous value and return a future that resolves to a [`ChainProcess<G>`](./enum.ChainProcess.html)
    #[cfg(not(feature = "async"))]
    fn proc(p: Self::Previous) -> ChainProcess<G>;
}
