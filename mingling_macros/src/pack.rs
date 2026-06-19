use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Attribute, Ident, Result as SynResult, Token, Type};

enum PackInput {
    Explicit {
        attrs: Vec<Attribute>,
        group_name: syn::Path,
        type_name: Ident,
        inner_type: Type,
    },
    Default {
        attrs: Vec<Attribute>,
        type_name: Ident,
        inner_type: Type,
    },
}

impl Parse for PackInput {
    fn parse(input: ParseStream) -> SynResult<Self> {
        // First, collect any outer attributes (`#[...]` and `///...`) before the main syntax.
        let attrs = input.call(Attribute::parse_outer)?;

        // Now determine the format:
        //   - `Path, TypeName = InnerType`  → Explicit
        //   - `TypeName = InnerType`          → Default

        if (input.peek(Ident) || input.peek(Token![crate]))
            && (input.peek2(Token![,]) || input.peek2(Token![::]))
        {
            // Explicit format: Path, TypeName = InnerType
            let group_name = input.parse::<syn::Path>()?;
            input.parse::<Token![,]>()?;
            let type_name = input.parse()?;
            input.parse::<Token![=]>()?;
            let inner_type = input.parse()?;

            Ok(PackInput::Explicit {
                attrs,
                group_name,
                type_name,
                inner_type,
            })
        } else if input.peek(Ident) && input.peek2(Token![=]) {
            // Default format: TypeName = InnerType
            let type_name = input.parse()?;
            input.parse::<Token![=]>()?;
            let inner_type = input.parse()?;

            Ok(PackInput::Default {
                attrs,
                type_name,
                inner_type,
            })
        } else {
            Err(input.lookahead1().error())
        }
    }
}

#[allow(clippy::too_many_lines)]
pub fn pack(input: TokenStream) -> TokenStream {
    // Parse the input
    let pack_input = syn::parse_macro_input!(input as PackInput);

    let (group_name, type_name, inner_type, attrs, use_default) = match pack_input {
        PackInput::Explicit {
            attrs,
            group_name,
            type_name,
            inner_type,
        } => (quote! { #group_name }, type_name, inner_type, attrs, false),
        PackInput::Default {
            attrs,
            type_name,
            inner_type,
        } => (
            crate::default_program_path(),
            type_name,
            inner_type,
            attrs,
            true,
        ),
    };

    // Generate the struct definition
    #[cfg(not(feature = "general_renderer"))]
    let struct_def = quote! {
        #(#attrs)*
        pub struct #type_name {
            pub(crate) inner: #inner_type,
        }
    };

    #[cfg(feature = "general_renderer")]
    let struct_def = quote! {
        #(#attrs)*
        #[derive(serde::Serialize)]
        pub struct #type_name {
            pub(crate) inner: #inner_type,
        }
    };

    // Generate the new() method
    let new_impl = quote! {
        impl #type_name {
            /// Creates a new instance of the wrapper type
            pub fn new(inner: #inner_type) -> Self {
                Self { inner }
            }
        }
    };

    // Generate From and Into implementations
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

    // Generate AsRef and AsMut implementations
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

    // Generate Deref and DerefMut implementations
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

    // Check if the inner type implements Default by generating conditional code
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

    let any_out_impl = quote! {
        impl Into<mingling::AnyOutput<#group_name>> for #type_name {
            fn into(self) -> mingling::AnyOutput<#group_name> {
                mingling::AnyOutput::new(self)
            }
        }

        impl Into<mingling::ChainProcess<#group_name>> for #type_name {
            fn into(self) -> mingling::ChainProcess<#group_name> {
                mingling::AnyOutput::new(self).route_chain()
            }
        }
    };

    let group_impl = quote! {
        impl ::mingling::Groupped<#group_name> for #type_name {
            fn member_id() -> #group_name {
                #group_name::#type_name
            }
        }
    };

    // Combine all implementations
    let expanded = if use_default {
        // For default case, use ThisProgram
        quote! {
            #struct_def

            #new_impl
            #from_into_impl
            #as_ref_impl
            #deref_impl
            #default_impl
            #register_impl

            impl Into<mingling::AnyOutput<#group_name>> for #type_name {
                fn into(self) -> mingling::AnyOutput<#group_name> {
                    mingling::AnyOutput::new(self)
                }
            }

            impl From<#type_name> for mingling::ChainProcess<#group_name> {
                fn from(value: #type_name) -> Self {
                    mingling::AnyOutput::new(value).route_chain()
                }
            }

            impl ::mingling::Groupped<#group_name> for #type_name {
                fn member_id() -> #group_name {
                    #group_name::#type_name
                }
            }
        }
    } else {
        // For explicit case, use the provided group_name
        quote! {
            #struct_def

            #new_impl
            #from_into_impl
            #as_ref_impl
            #deref_impl
            #default_impl
            #register_impl

            #any_out_impl
            #group_impl
        }
    };

    expanded.into()
}
