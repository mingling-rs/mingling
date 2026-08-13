use crate::ChainProcess;

#[doc(hidden)]
pub mod error;

/// Mingling's program logic execution unit
///
/// Binds a chain to a type. When the program is scheduled to that type, the
/// `proc` function in the chain will be executed to convert it to the next
/// type and send it to the scheduler.
///
/// # Async
///
/// When the `async` feature is enabled, the `proc` function of this trait no
/// longer requires returning a [`ChainProcess`], but rather a Future whose
/// output is a [`ChainProcess`].
///
/// # Manual impl
///
/// If you need to implement it manually, please do so as follows:
///
/// ```
/// # use mingling_core::Chain;
/// # use mingling_core::ChainProcess;
/// # enum ThisProgram {}
/// struct MyChain;
/// struct StateMyType;
///
/// impl Chain<ThisProgram> for MyChain {
///     type Previous = StateMyType;
///
///     fn proc(prev: Self::Previous) -> ChainProcess<ThisProgram> {
///         // Specific type conversion logic
/// # return mingling_core::ChainProcess::<ThisProgram>::Err(mingling_core::error::ChainProcessError::Other("test".to_string()));
///     }
/// }
/// ```
pub trait Chain<G> {
    /// The previous type bound to the chain, used to convert to the next arbitrary type in this chain
    type Previous;

    #[cfg(feature = "async")]
    /// The execution logic of the chain, converting the type `Previous` into the next type and returning an asynchronous [`ChainProcess`].
    ///
    /// Called when the `async` feature is enabled, this method returns a result that implements `Future`,
    /// whose output is a [`ChainProcess<G>`].
    fn proc(p: Self::Previous) -> impl Future<Output = ChainProcess<G>> + Send;

    #[cfg(not(feature = "async"))]
    /// The execution logic of the chain, converting the type `Previous` into the next type and returning a [`ChainProcess`].
    ///
    /// Called when the `async` feature is disabled, this method synchronously returns a [`ChainProcess<G>`].
    fn proc(p: Self::Previous) -> ChainProcess<G>;
}
