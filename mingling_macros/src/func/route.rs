use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

/// Routes execution depending on a condition — early-returns the error from a `Result`,
/// converting the `Ok` branch to the next chain process value.
pub(crate) fn route(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as syn::Expr);
    let expanded = quote! {
        match #expr {
            Ok(r) => r,
            Err(e) => return ::mingling::Routable::to_chain(e),
        }
    };
    TokenStream::from(expanded)
}
