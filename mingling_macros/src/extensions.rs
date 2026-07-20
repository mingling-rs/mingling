//! Extension point mechanism for Mingling attribute macros.
//!
//! This module provides a way for attribute macros like `#[chain]`, `#[renderer]`,
//! `#[help]`, and `#[completion]` to accept extension identifiers that are
//! applied as outer attributes before the bare macro.

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, Token};

/// Extension: `#[routeify]` — transforms `expr?` into `route!(expr)`.
#[cfg(feature = "extra_macros")]
pub(crate) mod routeify;

/// Extension: `#[buffer]` — wraps a unit-returning function to return `RenderResult`.
pub(crate) mod buffer;

/// Parsed extensions from an attribute macro like `#[chain(routeify, other_ext)]`.
pub(crate) struct Extensions {
    pub(crate) exts: Vec<Ident>,
}

impl Parse for Extensions {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut exts = Vec::new();
        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            exts.push(ident);
            if input.peek(Token![,]) {
                let _ = input.parse::<Token![,]>();
            }
        }
        Ok(Extensions { exts })
    }
}

/// Parsed extensions for `#[completion(EntryType, routeify, ...)]`.
#[cfg(feature = "comp")]
pub(crate) struct CompletionExt {
    pub(crate) entry_type: proc_macro2::TokenStream,
    pub(crate) exts: Vec<Ident>,
}

#[cfg(feature = "comp")]
impl Parse for CompletionExt {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let entry_type: proc_macro2::TokenStream = input.parse()?;
        let mut exts = Vec::new();
        while !input.is_empty() {
            let _ = input.parse::<Token![,]>();
            if input.is_empty() {
                break;
            }
            let ident: Ident = input.parse()?;
            exts.push(ident);
        }
        Ok(CompletionExt { entry_type, exts })
    }
}

/// Generates a re-dispatch token stream for attribute macros that take **no fixed arguments**
/// (chain, renderer, help).
pub(crate) fn try_redispatch_simple(
    attr: TokenStream,
    item: &TokenStream,
    bare_attr_name: &str,
) -> Option<TokenStream> {
    if attr.is_empty() {
        return None;
    }

    let exts: Extensions = syn::parse(attr).ok()?;
    if exts.exts.is_empty() {
        return None;
    }

    let bare = Ident::new(bare_attr_name, proc_macro2::Span::call_site());
    let exts = &exts.exts;
    let item = proc_macro2::TokenStream::from(item.clone());

    Some(
        quote! {
            #(#[#exts])*
            #[#bare]
            #item
        }
        .into(),
    )
}

/// Generates a re-dispatch token stream for `#[completion(EntryType, ...)]`.
#[cfg(feature = "comp")]
pub(crate) fn try_redispatch_completion(
    attr: TokenStream,
    item: &TokenStream,
) -> Option<TokenStream> {
    if attr.is_empty() {
        return None;
    }

    let parsed: CompletionExt = syn::parse(attr).ok()?;
    if parsed.exts.is_empty() {
        return None;
    }

    let entry_type = &parsed.entry_type;
    let exts = &parsed.exts;
    let item = proc_macro2::TokenStream::from(item.clone());

    Some(
        quote! {
            #(#[#exts])*
            #[::mingling::macros::completion(#entry_type)]
            #item
        }
        .into(),
    )
}
