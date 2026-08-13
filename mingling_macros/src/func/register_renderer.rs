// Doc Not Optimize
use proc_macro::TokenStream;

use crate::attr::renderer;

pub(crate) fn register_renderer_impl(input: TokenStream) -> TokenStream {
    renderer::register_renderer(input)
}
