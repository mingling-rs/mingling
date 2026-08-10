use proc_macro::TokenStream;
use quote::quote;
use syn::Ident;

use crate::get_global_set;

/// `pack_structural!` — like `pack!` but also marks the type as supporting
/// structured output via `StructuralData`.
#[allow(clippy::too_many_lines)]
pub(crate) fn pack_structural(input: TokenStream) -> TokenStream {
    // Parse same input format as `pack!`
    let input_parsed = syn::parse_macro_input!(input as PackStructuralInput);
    let type_name = input_parsed.type_name;
    let inner_type = input_parsed.inner_type;
    let attrs = input_parsed.attrs;
    let program_path = crate::default_program_path();

    // Register in STRUCTURED_TYPES
    let type_name_str = type_name.to_string();
    get_global_set(&crate::STRUCTURED_TYPES)
        .lock()
        .unwrap()
        .insert(type_name_str);

    // Struct definition (with Serialize derive, same as pack! under structural_renderer)
    #[cfg(not(feature = "structural_renderer"))]
    let struct_def = quote! {
        #(#attrs)*
        pub struct #type_name {
            pub inner: #inner_type,
        }
    };

    #[cfg(feature = "structural_renderer")]
    let struct_def = quote! {
        #(#attrs)*
        #[derive(serde::Serialize)]
        pub struct #type_name {
            pub inner: #inner_type,
        }
    };

    // Helper impls (same as pack!)
    let new_impl = quote! {
        impl #type_name {
            pub fn new(inner: #inner_type) -> Self {
                Self { inner }
            }
        }
    };

    let from_into_impl = quote! {
        impl From<#inner_type> for #type_name {
            fn from(inner: #inner_type) -> Self {
                Self::new(inner)
            }
        }
        impl From<#type_name> for #inner_type {
            fn from(wrapper: #type_name) -> #inner_type {
                wrapper.inner
            }
        }
    };

    let as_ref_impl = quote! {
        impl ::std::convert::AsRef<#inner_type> for #type_name {
            fn as_ref(&self) -> &#inner_type {
                &self.inner
            }
        }
        impl ::std::convert::AsMut<#inner_type> for #type_name {
            fn as_mut(&mut self) -> &mut #inner_type {
                &mut self.inner
            }
        }
    };

    let deref_impl = quote! {
        impl ::std::ops::Deref for #type_name {
            type Target = #inner_type;
            fn deref(&self) -> &Self::Target {
                &self.inner
            }
        }
        impl ::std::ops::DerefMut for #type_name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.inner
            }
        }
    };

    let default_impl = quote! {
        impl ::std::default::Default for #type_name
        where
            #inner_type: ::std::default::Default,
        {
            fn default() -> Self {
                Self::new(::std::default::Default::default())
            }
        }
    };

    let register_impl = quote! {
        ::mingling::macros::register_type!(#type_name);
    };

    // StructuralData impl + sealed + registration
    let structural_impl = quote! {
        impl ::mingling::__private::StructuralDataSealed<crate::ThisProgram> for #type_name {}
        impl ::mingling::__private::StructuralData<crate::ThisProgram> for #type_name {}
    };

    let expanded = quote! {
        #struct_def

        #new_impl
        #from_into_impl
        #as_ref_impl
        #deref_impl
        #default_impl
        #register_impl
        #structural_impl

        impl Into<::mingling::AnyOutput<#program_path>> for #type_name {
            fn into(self) -> ::mingling::AnyOutput<#program_path> {
                ::mingling::AnyOutput::new(self)
            }
        }

        impl Into<::mingling::ChainProcess<#program_path>> for #type_name {
            fn into(self) -> ::mingling::ChainProcess<#program_path> {
                ::mingling::AnyOutput::new(self).route_chain()
            }
        }

        /// SAFETY: This is an internal implementation of the `pack_structural!` macro,
        /// guaranteeing that the enum value registered by the `register_type!` macro
        /// is exactly the same as the actual return value,
        /// which can be confirmed via the `Ident` in the `quote!` block.
        unsafe impl ::mingling::Grouped<#program_path> for #type_name {
            fn member_id() -> #program_path {
                #program_path::#type_name
            }
        }
    };

    expanded.into()
}

/// Input for `pack_structural!` — same format as `pack!`.
struct PackStructuralInput {
    attrs: Vec<syn::Attribute>,
    type_name: Ident,
    inner_type: syn::Type,
}

impl syn::parse::Parse for PackStructuralInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;
        let type_name: Ident = input.parse()?;
        input.parse::<syn::Token![=]>()?;
        let inner_type: syn::Type = input.parse()?;
        Ok(Self {
            attrs,
            type_name,
            inner_type,
        })
    }
}
