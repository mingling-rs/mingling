use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Expr, Token, parse_macro_input};

struct SuggestInput {
    items: Punctuated<SuggestItem, Token![,]>,
}

enum SuggestItem {
    WithDesc(Box<(Expr, Expr)>), // "-i" = "Insert something"
    Simple(Expr),                // "-I"
}

impl Parse for SuggestInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let items = Punctuated::parse_terminated(input)?;
        Ok(SuggestInput { items })
    }
}

impl Parse for SuggestItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key: Expr = input.parse()?;

        if input.peek(Token![:]) {
            let _colon: Token![:] = input.parse()?;
            let value: Expr = input.parse()?;
            Ok(SuggestItem::WithDesc(Box::new((key, value))))
        } else {
            Ok(SuggestItem::Simple(key))
        }
    }
}

/// 判断表达式是否是一个纯字符串字面量（仅由一对引号包裹）
fn is_pure_lit_str(expr: &Expr) -> bool {
    matches!(expr, Expr::Lit(lit) if matches!(lit.lit, syn::Lit::Str(_)))
}

#[cfg(feature = "comp")]
pub(crate) fn suggest(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as SuggestInput);

    let mut items = Vec::new();
    let mut simple_items = Vec::new();

    for item in input.items {
        match item {
            SuggestItem::WithDesc(boxed) => {
                let (key, value) = *boxed;
                if is_pure_lit_str(&key) {
                    items.push(quote! {
                        vec![#key .to_string()], #value
                    });
                } else {
                    items.push(quote! {
                        #key, #value
                    });
                }
            }
            SuggestItem::Simple(key) => {
                if is_pure_lit_str(&key) {
                    simple_items.push(quote! {
                        vec![#key .to_string()]
                    });
                } else {
                    simple_items.push(quote! {
                        #key
                    });
                }
            }
        }
    }

    let expanded = if items.is_empty() && simple_items.is_empty() {
        quote! {
            ::mingling::Suggest::new()
        }
    } else {
        quote! {{
            let mut suggest = ::mingling::Suggest::new();
            #(suggest.add_suggest_with_description(#items);)*
            #(suggest.add_suggest(#simple_items);)*
            suggest
        }}
    };

    expanded.into()
}
