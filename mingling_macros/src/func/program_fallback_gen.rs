use proc_macro::TokenStream;
use quote::quote;

pub(crate) fn program_fallback_gen_impl(_input: TokenStream) -> TokenStream {
    #[cfg(feature = "structural_renderer")]
    let pack_empty = quote! {
        #[derive(::serde::Serialize, ::mingling::StructuralData, ::mingling::Grouped, Default)]
        pub struct ResultEmpty;
    };

    #[cfg(not(feature = "structural_renderer"))]
    let pack_empty = quote! {
        #[derive(::mingling::Grouped, Default)]
        pub struct ResultEmpty;
    };

    let expanded = quote! {
        ::mingling::macros::pack!(ErrorRendererNotFound = String);
        ::mingling::macros::pack!(ErrorDispatcherNotFound = Vec<String>);
        #pack_empty
    };
    TokenStream::from(expanded)
}
