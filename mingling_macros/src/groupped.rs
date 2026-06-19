use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Ident, parse_macro_input};

pub fn derive_groupped(input: TokenStream) -> TokenStream {
    // Parse the input struct/enum
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident;

    let group_ident: proc_macro2::TokenStream = crate::default_program_path();

    let any_output_convert_impls =
        proc_macro2::TokenStream::from(build_any_output_convert_impls(&struct_name, &group_ident));

    // Generate the Groupped trait implementation
    let expanded = quote! {
        ::mingling::macros::register_type!(#struct_name);

        impl ::mingling::Groupped<#group_ident> for #struct_name {
            fn member_id() -> #group_ident {
                #group_ident::#struct_name
            }
        }

        #any_output_convert_impls
    };

    expanded.into()
}

#[cfg(feature = "general_renderer")]
pub fn derive_groupped_serialize(input: TokenStream) -> TokenStream {
    // Parse the input struct/enum
    let input_parsed = parse_macro_input!(input as DeriveInput);
    let struct_name = input_parsed.ident.clone();

    let group_ident: proc_macro2::TokenStream = crate::default_program_path();

    let any_output_convert_impls =
        proc_macro2::TokenStream::from(build_any_output_convert_impls(&struct_name, &group_ident));

    // Generate both Serialize and Groupped implementations
    let expanded = quote! {
        #[derive(serde::Serialize)]
        #input_parsed

        ::mingling::macros::register_type!(#struct_name);

        impl ::mingling::Groupped<#group_ident> for #struct_name {
            fn member_id() -> #group_ident {
                #group_ident::#struct_name
            }
        }

        #any_output_convert_impls
    };

    expanded.into()
}

fn build_any_output_convert_impls(
    struct_name: &Ident,
    group_ident: &proc_macro2::TokenStream,
) -> TokenStream {
    quote! {
        impl ::std::convert::Into<::mingling::AnyOutput<#group_ident>> for #struct_name {
            fn into(self) -> ::mingling::AnyOutput<#group_ident> {
                ::mingling::AnyOutput::new(self)
            }
        }

        impl ::std::convert::Into<::mingling::ChainProcess<#group_ident>> for #struct_name {
            fn into(self) -> ::mingling::ChainProcess<#group_ident> {
                ::mingling::AnyOutput::new(self).route_chain()
            }
        }
    }
    .into()
}
