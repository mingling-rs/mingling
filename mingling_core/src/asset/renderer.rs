use crate::RenderResult;

/// Takes over a type (`Self::Previous`) and converts it to a [`RenderResult`](./struct.RenderResult.html)
pub trait Renderer {
    /// The previous type in the chain
    type Previous;

    /// Process the previous value and write the result into the provided [`RenderResult`](./struct.RenderResult.html)
    fn render(p: Self::Previous) -> RenderResult;
}
