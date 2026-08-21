use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

use crate::get_global_set;

/// Registers the type in `STRUCTURED_TYPES` and generates the
/// `StructuralData` impl.
///
/// Called internally by `#[derive(StructuralData)]` and usable directly for
/// external types (which cannot use the derive).
#[cfg(feature = "structural_renderer")]
pub(crate) fn structural_impl(input: TokenStream) -> TokenStream {
    let type_ident = parse_macro_input!(input as syn::Ident);
    let entry_str = type_ident.to_string();

    get_global_set(&crate::STRUCTURED_TYPES)
        .lock()
        .unwrap()
        .insert(entry_str);

    quote! {
        impl ::mingling::StructuralData<crate::ThisProgram> for #type_ident {}
    }
    .into()
}
