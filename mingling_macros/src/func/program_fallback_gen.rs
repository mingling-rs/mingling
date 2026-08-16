// Doc Not Optimize
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
        #[derive(::mingling::Grouped, ::mingling::Wrap, Default)]
        pub struct ErrorRendererNotFound(pub ::std::string::String);

        #[derive(::mingling::Grouped, ::mingling::Wrap, Default)]
        pub struct EntryFallback(pub ::std::vec::Vec<::std::string::String>);

        #pack_empty
    };
    TokenStream::from(expanded)
}
