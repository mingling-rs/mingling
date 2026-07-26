use proc_macro::TokenStream;
use quote::quote;

/// Creates an empty result value wrapped in `ChainProcess` for early return
/// from a chain function.
pub(crate) fn empty_result(_input: TokenStream) -> TokenStream {
    let expanded = quote! {
        <crate::ResultEmpty as ::mingling::Routable::<crate::ThisProgram>>::to_chain(crate::ResultEmpty)
    };
    TokenStream::from(expanded)
}
