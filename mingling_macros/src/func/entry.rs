use proc_macro::TokenStream;
use quote::quote;
use syn::{Ident, LitStr, parse_macro_input};

enum EntryInput {
    Typed { ident: Ident, strings: Vec<String> },
    Untyped { strings: Vec<String> },
}

impl syn::parse::Parse for EntryInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // entry!(EntryType, ["a", "b", "c"])
        // entry!["a", "b", "c"]  — comes in as just ["a", "b", "c"]
        if input.peek(Ident) && input.peek2(syn::Token![,]) {
            // entry!(EntryType, ["a", "b", "c"])
            let ident: Ident = input.parse()?;
            let _comma: syn::Token![,] = input.parse()?;
            let content;
            syn::bracketed!(content in input);
            let strings = parse_strings(&content)?;
            Ok(Self::Typed { ident, strings })
        } else {
            // entry!["a", "b", "c"]  — bare bracket content
            let strings = parse_strings(input)?;
            Ok(Self::Untyped { strings })
        }
    }
}

fn parse_strings(input: &syn::parse::ParseBuffer) -> syn::Result<Vec<String>> {
    let mut strings = Vec::new();
    while !input.is_empty() {
        let s: LitStr = input.parse()?;
        strings.push(s.value());
        if input.peek(syn::Token![,]) {
            let _comma: syn::Token![,] = input.parse()?;
        }
    }
    Ok(strings)
}

pub(crate) fn entry(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as EntryInput);

    let strings = match &parsed {
        EntryInput::Typed { strings, .. } | EntryInput::Untyped { strings } => strings,
    };
    let string_exprs = strings
        .iter()
        .map(|s| {
            let lit = syn::LitStr::new(s, proc_macro2::Span::call_site());
            quote! { #lit.to_string() }
        })
        .collect::<Vec<_>>();

    let expanded = match parsed {
        EntryInput::Typed { ident, .. } => {
            quote! {
                #ident::new(vec![#(#string_exprs),*])
            }
        }
        EntryInput::Untyped { .. } => {
            quote! {
                vec![#(#string_exprs),*].into()
            }
        }
    };

    expanded.into()
}
