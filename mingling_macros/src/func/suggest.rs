use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Expr, LitStr, Token, parse_macro_input};

struct SuggestInput {
    items: Punctuated<SuggestItem, Token![,]>,
}

enum SuggestItem {
    WithDesc(Box<(LitStr, Expr)>), // "-i" = "Insert something"
    Simple(LitStr),                // "-I"
}

impl Parse for SuggestInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let items = Punctuated::parse_terminated(input)?;
        Ok(SuggestInput { items })
    }
}

impl Parse for SuggestItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key: LitStr = input.parse()?;

        if input.peek(Token![:]) {
            let _colon: Token![:] = input.parse()?;
            let value: Expr = input.parse()?;
            Ok(SuggestItem::WithDesc(Box::new((key, value))))
        } else {
            Ok(SuggestItem::Simple(key))
        }
    }
}

#[cfg(feature = "comp")]
pub(crate) fn suggest(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as SuggestInput);

    let mut items = Vec::new();

    for item in input.items {
        match item {
            SuggestItem::WithDesc(boxed) => {
                let (key, value) = *boxed;
                items.push(quote! {
                    ::mingling::SuggestItem::new_with_desc(#key.to_string(), #value.to_string())
                });
            }
            SuggestItem::Simple(key) => {
                items.push(quote! {
                    ::mingling::SuggestItem::new(#key.to_string())
                });
            }
        }
    }

    let expanded = if items.is_empty() {
        quote! {
            ::mingling::Suggest::new()
        }
    } else {
        quote! {{
            let mut suggest = ::mingling::Suggest::new();
            #(suggest.insert(#items);)*
            suggest
        }}
    };

    expanded.into()
}

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
