// Doc Not Optimize
use proc_macro::TokenStream;
use syn::parse_macro_input;

use crate::PACKED_TYPES;
use crate::get_global_set;

pub(crate) fn register_type_impl(input: TokenStream) -> TokenStream {
    let type_ident = parse_macro_input!(input as syn::Ident);
    let entry_str = type_ident.to_string();

    get_global_set(&PACKED_TYPES)
        .lock()
        .unwrap()
        .insert(entry_str);

    TokenStream::new()
}
