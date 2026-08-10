use proc_macro::TokenStream;
use quote::quote;
use syn::{Ident, TypePath};

use crate::get_global_set;

/// `group_structural!` — like `group!` but also marks the type as supporting
/// structured output via `StructuralData`.
pub(crate) fn group_structural(input: TokenStream) -> TokenStream {
    // Parse the same input as group!
    let input_parsed = syn::parse_macro_input!(input as GroupStructuralInput);

    let is_aliased = matches!(&input_parsed, GroupStructuralInput::Aliased { .. });

    let (type_path, type_name, alias_stmt) = match &input_parsed {
        GroupStructuralInput::Plain(type_path) => {
            let name = type_path
                .path
                .segments
                .last()
                .expect("TypePath must have at least one segment")
                .ident
                .clone();
            (type_path.clone(), name, quote! {})
        }
        GroupStructuralInput::Aliased { alias, type_path } => {
            let alias_stmt = quote! {
                pub(crate) type #alias = #type_path;
            };
            (type_path.clone(), alias.clone(), alias_stmt)
        }
    };

    let type_name_str = type_name.to_string();

    // Register in STRUCTURED_TYPES
    get_global_set(&crate::STRUCTURED_TYPES)
        .lock()
        .unwrap()
        .insert(type_name_str);

    let program_path = crate::default_program_path();

    // Generate unique module name
    let segments: Vec<String> = type_path
        .path
        .segments
        .iter()
        .map(|seg| seg.ident.to_string().to_lowercase())
        .collect();
    let module_name = Ident::new(
        &format!("internal_group_{}", segments.join("_")),
        proc_macro2::Span::call_site(),
    );

    // Generate the appropriate `use` statement for the original type
    let type_use = if type_path.path.segments.len() > 1 {
        quote! { #[allow(unused_imports)] use #type_path; }
    } else {
        let ident = type_path
            .path
            .segments
            .last()
            .expect("TypePath must have at least one segment")
            .ident
            .clone();
        quote! { #[allow(unused_imports)] use super::#ident; }
    };

    let alias_use = if is_aliased {
        quote! { use super::#type_name; }
    } else {
        quote! {}
    };

    let expanded = quote! {
        #alias_stmt
        #[allow(non_camel_case_types)]
        mod #module_name {
            use #program_path as __MinglingProgram;
            #type_use
            #alias_use

            /// SAFETY: This is an internal implementation of the `pack_structural!` macro,
            /// guaranteeing that the enum value registered by the `register_type!` macro
            /// is exactly the same as the actual return value,
            /// which can be confirmed via the `Ident` in the `quote!` block.
            unsafe impl ::mingling::Grouped<__MinglingProgram> for #type_name {
                fn member_id() -> __MinglingProgram {
                    __MinglingProgram::#type_name
                }
            }

            impl ::mingling::__private::StructuralDataSealed<crate::ThisProgram> for #type_name {}
            impl ::mingling::__private::StructuralData<crate::ThisProgram> for #type_name {}

            ::mingling::macros::register_type!(#type_name);
        }
    };

    expanded.into()
}

/// Input for `group_structural!` — same format as `group!`.
enum GroupStructuralInput {
    Plain(TypePath),
    Aliased { alias: Ident, type_path: TypePath },
}

impl syn::parse::Parse for GroupStructuralInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let fork = input.fork();
        let _first: Ident = fork.parse()?;
        if fork.peek(syn::Token![=]) {
            let alias: Ident = input.parse()?;
            let _eq: syn::Token![=] = input.parse()?;
            let type_path: TypePath = input.parse()?;
            Ok(Self::Aliased { alias, type_path })
        } else {
            let type_path: TypePath = input.parse()?;
            Ok(Self::Plain(type_path))
        }
    }
}
