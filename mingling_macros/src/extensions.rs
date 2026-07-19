//! Extension point mechanism for Mingling attribute macros.
//!
//! This module provides a way for attribute macros like `#[chain]`, `#[renderer]`,
//! `#[help]`, and `#[completion]` to accept extension identifiers that are
//! applied as outer attributes before the bare macro.
//!
//! # How it works
//!
//! When a user writes `#[chain(routeify, other_ext)]`, the extension point mechanism
//! detects `routeify` and `other_ext` as extension identifiers, and generates a
//! re-dispatch token stream equivalent to:
//!
//! ```ignore
//! #[other_ext]
//! #[routeify]
//! #[chain]
//! fn handle(...) { ... }
//! ```
//!
//! The original bare macro (`#[chain]` without extensions) sees no arguments
//! and proceeds as normal — the extensions have already been stripped.

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, Token};

/// Parsed extensions from an attribute macro like `#[chain(routeify, other_ext)]`.
///
/// For macros with no fixed arguments (chain, renderer, help):
/// - `exts` contains all parsed extension identifiers
/// - `fixed` is empty
///
/// For macros with fixed arguments (completion):
/// - Use `parse_completion_attr_extensions` instead.
pub(crate) struct Extensions {
    /// Extension identifiers (e.g., `routeify`, `other_ext`)
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
///
/// The first argument is the entry type path, the rest are extension identifiers.
#[cfg(feature = "comp")]
pub(crate) struct CompletionExt {
    /// The entry type path (first argument, e.g., `HelloEntry` or `crate::Entry`)
    pub(crate) entry_type: proc_macro2::TokenStream,
    /// Extension identifiers (e.g., `routeify`, `other_ext`)
    pub(crate) exts: Vec<Ident>,
}

#[cfg(feature = "comp")]
impl Parse for CompletionExt {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse the first argument as the entry type path
        let entry_type: proc_macro2::TokenStream = input.parse()?;

        // Parse remaining arguments as extensions
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
///
/// If `attr` contains extensions, the output is `#[ext1] #[ext2] ...[#bare_attr] #item`.
/// If `attr` is empty or contains no extensions, returns `None`.
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
            #(#exts)*
            #[#bare]
            #item
        }
        .into(),
    )
}

/// Generates a re-dispatch token stream for `#[completion(EntryType, ...)]`.
///
/// If extensions are found, the output is `#[ext1] #[ext2] #[completion(EntryType)] #item`.
/// If no extensions, returns `None`.
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
            #(#exts)*
            #[::mingling::macros::completion(#entry_type)]
            #item
        }
        .into(),
    )
}
