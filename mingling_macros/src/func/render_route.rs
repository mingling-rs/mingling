// Doc Not Optimize
use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

/// Routes errors to the rendering pipeline instead of the chain pipeline.
pub(crate) fn render_route(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as syn::Expr);
    let expanded = quote! {
        match #expr {
            Ok(r) => r,
            Err(e) => return <crate::ThisProgram as ::mingling::ProgramCollect>::render(::mingling::AnyOutput::new(e)),
        }
    };
    TokenStream::from(expanded)
}
