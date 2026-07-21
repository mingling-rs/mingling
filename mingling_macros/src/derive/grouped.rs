use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Ident, parse_macro_input};

pub(crate) fn derive_grouped(input: TokenStream) -> TokenStream {
    // Parse the input struct/enum
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident;

    let group_ident: proc_macro2::TokenStream = crate::default_program_path();

    let any_output_convert_impls =
        proc_macro2::TokenStream::from(build_any_output_convert_impls(&struct_name, &group_ident));

    // Generate the Grouped trait implementation
    let expanded = quote! {
        ::mingling::macros::register_type!(#struct_name);

        /// SAFETY: This is an internal implementation of the `Grouped` derive macro,
        /// guaranteeing that the enum value registered by the `register_type!` macro
        /// is exactly the same as the actual return value,
        /// which can be confirmed via the `Ident` in the `quote!` block.
        unsafe impl ::mingling::Grouped<#group_ident> for #struct_name {
            fn member_id() -> #group_ident {
                #group_ident::#struct_name
            }
        }

        #any_output_convert_impls
    };

    expanded.into()
}

#[cfg(feature = "structural_renderer")]
pub fn derive_grouped_serialize(input: TokenStream) -> TokenStream {
    // Parse the input struct/enum
    let input_parsed = parse_macro_input!(input as DeriveInput);
    let struct_name = input_parsed.ident.clone();

    let group_ident: proc_macro2::TokenStream = crate::default_program_path();

    let any_output_convert_impls =
        proc_macro2::TokenStream::from(build_any_output_convert_impls(&struct_name, &group_ident));

    // Generate both Serialize and Grouped implementations
    let expanded = quote! {
        #[derive(serde::Serialize)]
        #input_parsed

        ::mingling::macros::register_type!(#struct_name);

        /// SAFETY: This is an internal implementation of the `Grouped` derive macro,
        /// guaranteeing that the enum value registered by the `register_type!` macro
        /// is exactly the same as the actual return value,
        /// which can be confirmed via the `Ident` in the `quote!` block.
        unsafe impl ::mingling::Grouped<#group_ident> for #struct_name {
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
