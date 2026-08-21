use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

/// Derive macro for `StructuralData`.
pub(crate) fn derive_structural_data(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let type_name = input.ident;

    // Delegate registration and impl generation to the `structural!` macro,
    // which is also the public entry point for external types.
    let expanded = quote! {
        ::mingling::macros::structural!(#type_name);
    };

    expanded.into()
}
