// Doc Not Optimize
use proc_macro::TokenStream;
use quote::ToTokens;
use syn::TypePath;
use syn::spanned::Spanned;

use crate::METADATA;
use crate::get_global_set;

/// Parses and registers a metadata mapping of the form
/// `register_metadata!(EntryGreet, Description)`.
///
/// Stores a match-arm-style string entry `Self::EntryGreet => { ... }` that is
/// later consumed by `program_final_gen!` to generate the `get_metadata`
/// method of `ProgramCollect`.
pub(crate) fn register_metadata_impl(input: TokenStream) -> TokenStream {
    // Parse the input as a comma-separated list of type arguments.
    let input_parsed = syn::parse_macro_input!(
        input with syn::punctuated::Punctuated<syn::Expr, syn::Token![,]>::parse_terminated
    );

    if input_parsed.len() != 2 {
        return syn::Error::new(
            input_parsed.span(),
            "Expected exactly two comma-separated arguments: `EntryVariant, MetadataType`",
        )
        .to_compile_error()
        .into();
    }

    let entry_expr = &input_parsed[0];
    let metadata_expr = &input_parsed[1];

    let entry_type = match syn::parse2::<TypePath>(entry_expr.to_token_stream()) {
        Ok(ty) => ty,
        Err(e) => return e.to_compile_error().into(),
    };
    let metadata_type = match syn::parse2::<TypePath>(metadata_expr.to_token_stream()) {
        Ok(ty) => ty,
        Err(e) => return e.to_compile_error().into(),
    };

    let entry_str = build_metadata_entry(&entry_type, &metadata_type).to_string();

    get_global_set(&METADATA).lock().unwrap().insert(entry_str);

    quote::quote! {}.into()
}

/// Builds the match-arm entry for `get_metadata`, matching on the enum variant
/// and then on the requested `TypeId`.
fn build_metadata_entry(
    entry_type: &TypePath,
    metadata_type: &TypePath,
) -> proc_macro2::TokenStream {
    let enum_variant = entry_type.path.segments.last().unwrap().ident.clone();
    quote::quote! {
        Self::#enum_variant => {
            let __metadata_type_id = ::std::any::TypeId::of::<#metadata_type>();
            match type_id {
                _ if type_id == __metadata_type_id => Some(::std::boxed::Box::new(
                    <#entry_type as ::mingling::Metadata<#metadata_type>>::init_metadata(),
                ) as ::std::boxed::Box<dyn ::std::any::Any>),
                _ => None,
            }
        }
    }
}
