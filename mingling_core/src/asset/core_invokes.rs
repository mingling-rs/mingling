use std::marker::PhantomData;

use crate::{
    AnyOutput, ChainProcess, Grouped, NextProcess, ProgramCollect, RenderResult, ResourceMarker,
};

/// Type used to invoke Renderers in Mingling
///
/// It is marked as `#[non_exhaustive]` and all internal fields are private.
/// This type can only be implicitly created by the hidden external API
/// `__resource_marker_default` of `ResourceMarker`.
///
/// ```rust,ignore
/// //  You can inject other renderers as types into the current function
/// //                                     |
/// #[renderer(buffer)] //                 vvvvvvvvvvvvvvvvvvvv
/// fn render_foo(_: ResultFoo, renderer: &RendererInvoker<Bar>) {
///     let bar = Bar::default();
///     r_append!(renderer.invoke(bar));
/// }
/// ```
#[non_exhaustive]
pub struct RendererInvoker<T> {
    phantom: PhantomData<T>,
    create_by_res_injection: bool,
}

impl<T> RendererInvoker<T>
where
    T: Send,
{
    /// Invoke the renderer with the given value.
    ///
    /// This function triggers the rendering pipeline for the provided value `T`.
    /// It can only be executed when the `RendererInvoker` was created by the resource injection system
    /// (i.e., via `__resource_marker_default`). If the invoker was created manually or cloned outside
    /// the injection context, calling this method will panic.
    ///
    /// # Panics
    ///
    /// Panics if the `RendererInvoker` was not created by the resource injection system.
    ///
    /// # Hook
    ///
    /// It will not execute any program hooks, because this type is used for **bypassing** or **reusing**, not for flow control.
    pub fn invoke<C>(&self, value: T) -> RenderResult
    where
        C: ProgramCollect<Enum = C> + 'static,
        T: Grouped<C>,
    {
        assert!(
            self.create_by_res_injection,
            "The current RendererInvoker was not created by the resource injection system, so it cannot be executed!"
        );
        C::render(AnyOutput::new(value))
    }
}

impl<T> ResourceMarker for RendererInvoker<T> {
    fn __resource_marker_clone(&self) -> Self {
        Self {
            phantom: PhantomData,
            create_by_res_injection: self.create_by_res_injection,
        }
    }

    fn __resource_marker_default() -> Self {
        // Create RendererInvoker with `create_by_res_injection` marked as true
        //
        // Reason: RendererInvoker is designed to only be created through resource injection.
        // When the resource injection does not find a corresponding value,
        // this method will be used to generate a default value to pass in.
        Self {
            phantom: PhantomData,
            create_by_res_injection: true,
        }
    }

    fn __resource_marker_modify<C>(f: impl FnOnce(&mut Self))
    where
        C: ProgramCollect<Enum = C> + 'static,
    {
        // DO NOTHING
        //
        // When ResourceMarker is asked to modify, it should not execute anything.
        let _ = f;
    }
}

/// Type used to invoke Chain in Mingling
///
/// It is marked as `#[non_exhaustive]` and all internal fields are private.
/// This type can only be implicitly created by the hidden external API
/// `__resource_marker_default` of `ResourceMarker`.
///
/// ```rust,ignore
/// //  You can inject other chain as types into the current function
/// //                                 |
/// #[chain] //                        vvvvvvvvvvvvvvvvvvvvv
/// fn handle_foo(_: EntryFoo, chain: &ChainInvoker<StateBar>) {
///     let bar = Bar::default();
///     let next = chain.invoke_once(bar);
/// }
/// ```
#[non_exhaustive]
pub struct ChainInvoker<T> {
    phantom: PhantomData<T>,
    create_by_res_injection: bool,
}

impl<T> ChainInvoker<T>
where
    T: Send,
{
    /// Execute one step of the chain with the given value, returning the next state.
    ///
    /// This function performs **only a single step** of chain execution — it invokes the
    /// current handler for value `T` and returns the next [`ChainProcess`] state, without
    /// automatically continuing the chain. The caller is responsible for proceeding through
    /// subsequent steps based on the returned state.
    ///
    /// It can only be executed when the `ChainInvoker` was created by the resource injection system
    /// (i.e., via `__resource_marker_default`). If the invoker was created manually or cloned outside
    /// the injection context, calling this method will panic.
    ///
    /// # Special Behavior
    ///
    /// If no chain exists for this type, it will convert itself into a `ChainProcess` that routes to the chain and return it.
    ///
    /// # Panics
    ///
    /// Panics if the `ChainInvoker` was not created by the resource injection system.
    ///
    /// # Hooks
    ///
    /// It will not execute any program hooks, because this type is used for **bypassing** or **reusing**, not for flow control.
    #[might_be_async::func]
    #[allow(clippy::future_not_send)]
    // The generated future is only awaited within the single-threaded chain
    // execution, so it does not need to be `Send`.
    pub fn invoke_once<C>(&self, value: T) -> ChainProcess<C>
    where
        C: ProgramCollect<Enum = C> + 'static,
        T: Grouped<C>,
    {
        self.pre_check();

        let any = AnyOutput::new(value);

        if C::has_chain(&any) {
            might_be_async::invoke!(C::do_chain(any))
        } else {
            ChainProcess::Ok((any, NextProcess::Chain))
        }
    }

    /// Continuously execute the chain until it is routed to a renderer or can no longer continue
    ///
    /// This function can only be called when the `ChainInvoker` was created by the resource injection system
    /// (i.e., via `__resource_marker_default`). If the invoker was created manually or cloned outside
    /// the injection context, calling this method will panic.
    ///
    /// # Panics
    ///
    /// Panics if the `ChainInvoker` was not created by the resource injection system.
    ///
    /// # Hooks
    ///
    /// It will not execute any program hooks, because this type is used for **bypassing** or **reusing**, not for flow control.
    #[might_be_async::func]
    // The generated future is only awaited within the single-threaded chain
    // execution, so it does not need to be `Send`.
    #[allow(clippy::future_not_send)]
    pub fn invoke_to_last<C>(&self, value: T) -> ChainProcess<C>
    where
        C: ProgramCollect<Enum = C> + 'static,
        T: Grouped<C>,
    {
        self.pre_check();

        let mut current = might_be_async::invoke!(C::do_chain(AnyOutput::new(value)));

        loop {
            match current {
                ChainProcess::Ok((any, NextProcess::Chain)) => {
                    if C::has_chain(&any) {
                        current = might_be_async::invoke!(C::do_chain(any));
                    } else {
                        // If the next step of this type does not have a Chain, reconstruct it back
                        return ChainProcess::Ok((any, NextProcess::Chain));
                    }
                }
                _ => return current,
            }
        }
    }

    /// Continuously execute the chain until it is rendered into a `RenderResult`
    ///
    /// This function can only be called when the `ChainInvoker` was created by the resource injection system
    /// (i.e., via `__resource_marker_default`). If the invoker was created manually or cloned outside
    /// the injection context, calling this method will panic.
    ///
    /// # Special Behavior
    ///
    /// If an error occurs during rendering, or the result type does not contain a renderer, it will be rendered as an empty `RenderResult`
    ///
    /// # Panics
    ///
    /// Panics if the `ChainInvoker` was not created by the resource injection system.
    ///
    /// # Hooks
    ///
    /// It will not execute any program hooks, because this type is used for **bypassing** or **reusing**, not for flow control.
    #[might_be_async::func]
    #[allow(clippy::future_not_send)]
    // The generated future is only awaited within the single-threaded chain
    // execution, so it does not need to be `Send`.
    pub fn invoke_to_result<C>(&self, value: T) -> RenderResult
    where
        C: ProgramCollect<Enum = C> + 'static,
        T: Grouped<C>,
    {
        self.pre_check();

        let last = might_be_async::invoke!(self.invoke_to_last(value));

        match last {
            ChainProcess::Err(_) => RenderResult::new(),
            ChainProcess::Ok((any, _)) => {
                if C::has_renderer(&any) {
                    C::render(any)
                } else {
                    RenderResult::new()
                }
            }
        }
    }

    fn pre_check(&self) {
        assert!(
            self.create_by_res_injection,
            "The current ChainInvoker was not created by the resource injection system, so it cannot be executed!"
        );
    }
}

impl<T> ResourceMarker for ChainInvoker<T> {
    fn __resource_marker_clone(&self) -> Self {
        Self {
            phantom: PhantomData,
            create_by_res_injection: self.create_by_res_injection,
        }
    }

    fn __resource_marker_default() -> Self {
        // Create ChainInvoker with `create_by_res_injection` marked as true
        //
        // Reason: ChainInvoker is designed to only be created through resource injection.
        // When the resource injection does not find a corresponding value,
        // this method will be used to generate a default value to pass in.
        Self {
            phantom: PhantomData,
            create_by_res_injection: true,
        }
    }

    fn __resource_marker_modify<C>(f: impl FnOnce(&mut Self))
    where
        C: ProgramCollect<Enum = C> + 'static,
    {
        // DO NOTHING
        //
        // When ResourceMarker is asked to modify, it should not execute anything.
        let _ = f;
    }
}
