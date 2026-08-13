use crate::RenderResult;

/// Rendering logic for Mingling programs
///
/// Add a rendering type for registered types in the Mingling program. When they are routed to `to_render()`, this renderer will be invoked to render them into the result output.
///
/// # Manual impl
///
/// Generally speaking, it is recommended to use the [`#[renderer]`](https://docs.rs/mingling/latest/mingling/macros/attr.renderer.html) macro instead.
/// If you need to implement this manually, please refer to the following example:
///
/// ```
/// # use mingling_core::Renderer;
/// # use mingling_core::RenderResult;
/// # struct MyRenderer;
/// # struct StateMyType;
///
/// impl Renderer for MyRenderer {
///     type Previous = StateMyType;
///
///     fn render(prev: Self::Previous) -> RenderResult {
///         // The specific rendering logic
///         # return mingling_core::RenderResult::default();
///     }
/// }
/// ```
pub trait Renderer {
    /// The previous type handled by the renderer, used to convert it into a render result
    type Previous;

    /// The rendering logic, which converts the `Previous` type into the corresponding [`RenderResult`] output
    ///
    /// When a program is routed to the type registered for this renderer, this method will be called to convert and render the previous type into the final result.
    fn render(p: Self::Previous) -> RenderResult;
}
