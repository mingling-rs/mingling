// Doc Not Optimize
use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{Attribute, ItemFn, ReturnType, TypePath, parse_macro_input};

/// Implements the `#[metadata(EntryVariant)]` attribute macro.
///
/// It takes the enum variant ident to attach metadata to, and rewrites the
/// annotated function into:
/// - an `impl ::mingling::Metadata<ReturnType> for EntryVariant` that calls the
///   original function,
/// - a `::mingling::macros::register_metadata!(EntryVariant, ReturnType)` call,
/// - the preserved original function.
pub(crate) fn metadata_attr(attr: TokenStream, item: TokenStream) -> TokenStream {
    let entry_variant = parse_macro_input!(attr as syn::Ident);

    let input_fn = parse_macro_input!(item as ItemFn);

    // The metadata type is the function's return type.
    let metadata_type = match &input_fn.sig.output {
        ReturnType::Type(_, ty) => match syn::parse2::<TypePath>(quote! { #ty }) {
            Ok(ty) => ty,
            Err(e) => return e.to_compile_error().into(),
        },
        ReturnType::Default => {
            return syn::Error::new(
                input_fn.sig.span(),
                "#[metadata] requires the function to have an explicit return type",
            )
            .to_compile_error()
            .into();
        }
    };

    // Preserve the original return type exactly as written, so the original
    // function signature is reproduced unchanged.
    let original_return_type = match &input_fn.sig.output {
        ReturnType::Type(_, ty) => quote! { #ty },
        ReturnType::Default => quote! { () },
    };

    // Reject async metadata functions: `Metadata::init_metadata` is synchronous.
    if input_fn.sig.asyncness.is_some() {
        return syn::Error::new(input_fn.sig.span(), "Metadata function cannot be async")
            .to_compile_error()
            .into();
    }

    let fn_name = &input_fn.sig.ident;
    let vis = &input_fn.vis;
    let original_inputs = input_fn.sig.inputs.clone();
    let fn_body_stmts = &input_fn.block.stmts;

    // Function attributes, excluding the metadata attribute itself.
    let fn_attrs: Vec<&Attribute> = input_fn
        .attrs
        .iter()
        .filter(|attr| !attr.path().is_ident("metadata"))
        .collect();

    // A metadata provider is a zero-argument function.
    if !original_inputs.is_empty() {
        return syn::Error::new(
            input_fn.sig.span(),
            "#[metadata] function cannot take any parameters",
        )
        .to_compile_error()
        .into();
    }

    let expanded = quote! {
        impl ::mingling::Metadata<#metadata_type> for #entry_variant {
            fn init_metadata() -> #metadata_type {
                #fn_name()
            }
        }

        ::mingling::macros::register_metadata!(#entry_variant, #metadata_type);

        #(#fn_attrs)*
        #vis fn #fn_name(#original_inputs) -> #original_return_type {
            #(#fn_body_stmts)*
        }
    };

    expanded.into()
}
