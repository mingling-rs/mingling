// Doc Not Optimize
use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

pub(crate) fn suggest_enum(input: TokenStream) -> TokenStream {
    let enum_type = parse_macro_input!(input as syn::Type);

    let expanded = quote! {{
        let mut enum_suggest = ::mingling::Suggest::new();
        for (name, desc) in <#enum_type>::enums() {
            if desc.is_empty() {
                enum_suggest.insert(::mingling::SuggestItem::new(name.to_string()));
            } else {
                enum_suggest.insert(::mingling::SuggestItem::new_with_desc(
                    name.to_string(),
                    desc.to_string(),
                ));
            }
        }
        enum_suggest
    }};

    expanded.into()
}
