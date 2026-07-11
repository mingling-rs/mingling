use crate::RenderResult;

/// Handles help rendering for command-line arguments
pub trait HelpRequest {
    /// The entry type
    type Entry;

    /// Process the previous value and write the result into the provided [`RenderResult`](./struct.RenderResult.html)
    fn render_help(p: Self::Entry) -> RenderResult;
}
