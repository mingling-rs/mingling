// Doc Not Optimize
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

use crate::get_global_set;

/// Derive macro for `StructuralData`.
pub(crate) fn derive_structural_data(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let type_name = input.ident;

    // Register in STRUCTURED_TYPES
    let type_name_str = type_name.to_string();
    get_global_set(&crate::STRUCTURED_TYPES)
        .lock()
        .unwrap()
        .insert(type_name_str);

    // Generate BOTH the sealed impl AND the StructuralData impl.
    let expanded = quote! {
        impl ::mingling::__private::StructuralDataSealed<crate::ThisProgram> for #type_name {}
        impl ::mingling::__private::StructuralData<crate::ThisProgram> for #type_name {}
    };

    expanded.into()
}
