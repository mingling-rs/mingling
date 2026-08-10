use just_fmt::kebab_case;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{LitStr, Result as SynResult};

/// Parses a string literal input for the node macro
struct NodeInput {
    path: LitStr,
}

impl Parse for NodeInput {
    fn parse(input: ParseStream) -> SynResult<Self> {
        Ok(Self {
            path: input.parse()?,
        })
    }
}

pub(crate) fn node(input: TokenStream) -> TokenStream {
    // Parse the input as a string literal
    let input_parsed = syn::parse_macro_input!(input as NodeInput);
    let path_str = input_parsed.path.value();

    // If the input string is empty, return an empty Node
    if path_str.is_empty() {
        return quote! {
            mingling::Node::default()
        }
        .into();
    }

    // Split the path by dots
    let parts: Vec<String> = path_str
        .split('.')
        .map(|s| {
            if s.starts_with('_') {
                s.to_string()
            } else {
                kebab_case!(s)
            }
        })
        .collect();

    // Build the expression starting from Node::default()
    let mut expr: TokenStream2 = quote! {
        mingling::Node::default()
    };

    // Add .join() calls for each part of the path
    for part in parts {
        expr = quote! {
            #expr.join(#part)
        };
    }

    expr.into()
}
