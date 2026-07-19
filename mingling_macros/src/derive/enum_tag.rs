use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Attribute, Data, DeriveInput, Error, Fields, Ident, LitStr, Result, Variant, parse_macro_input,
};

pub(crate) fn derive_enum_tag(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match derive_enum_tag_impl(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// Implementation of the `EnumTag` derive macro
fn derive_enum_tag_impl(input: DeriveInput) -> Result<proc_macro2::TokenStream> {
    let enum_name = &input.ident;
    let generics = &input.generics;

    // Extract enum data
    let data = match input.data {
        Data::Enum(data_enum) => data_enum,
        Data::Struct(_) => {
            return Err(Error::new_spanned(
                enum_name,
                "EnumTag can only be derived for enums, not structs",
            ));
        }
        Data::Union(_) => {
            return Err(Error::new_spanned(
                enum_name,
                "EnumTag can only be derived for enums, not unions",
            ));
        }
    };

    // Process each variant
    let mut variant_info = Vec::new();
    let mut match_arms = Vec::new();
    let mut build_match_arms = Vec::new();

    for variant in data.variants {
        process_variant(
            &variant,
            enum_name,
            &mut variant_info,
            &mut match_arms,
            &mut build_match_arms,
        )?;
    }

    // Generate the implementation
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics ::mingling::EnumTag for #enum_name #ty_generics #where_clause {
            fn enum_info(&self) -> (&'static str, &'static str) {
                match self {
                    #(#match_arms)*
                }
            }

            fn build_enum(name: String) -> Option<Self>
            where
                Self: Sized
            {
                match name.as_str() {
                    #(#build_match_arms)*
                    _ => None,
                }
            }

            fn enums() -> &'static [(&'static str, &'static str)] {
                &[#(#variant_info),*]
            }
        }
    };

    Ok(expanded)
}

/// Process a single enum variant
fn process_variant(
    variant: &Variant,
    enum_name: &Ident,
    variant_info: &mut Vec<proc_macro2::TokenStream>,
    match_arms: &mut Vec<proc_macro2::TokenStream>,
    build_match_arms: &mut Vec<proc_macro2::TokenStream>,
) -> Result<()> {
    let variant_name = variant.ident.clone();

    // Check if variant has fields
    match &variant.fields {
        Fields::Unit => {
            // Good, unit variant
        }
        Fields::Named(_) | Fields::Unnamed(_) => {
            return Err(Error::new_spanned(
                variant,
                format!(
                    "EnumTag cannot be derived for enum variant `{variant_name}` with fields. Only unit variants are supported."
                ),
            ));
        }
    }

    // Extract description from #[enum_desc] attribute
    let description = extract_description(&variant.attrs)?;

    // Extract rename from #[enum_rename] attribute
    let rename = extract_rename(&variant.attrs)?;

    // Generate tokens for this variant
    let variant_name_str = variant_name.to_string();
    let display_name = rename.unwrap_or_else(|| variant_name_str.clone());
    let description_str = description.unwrap_or_default();

    variant_info.push(quote! {
        (#display_name, #description_str)
    });

    match_arms.push(quote! {
        #enum_name::#variant_name => (#display_name, #description_str),
    });

    build_match_arms.push(quote! {
        #display_name => Some(#enum_name::#variant_name),
    });

    Ok(())
}

/// Extract description from #[`enum_desc`] attribute
fn extract_description(attrs: &[Attribute]) -> Result<Option<String>> {
    for attr in attrs {
        if attr.path().is_ident("enum_desc") {
            return match attr.parse_args::<LitStr>() {
                Ok(lit_str) => Ok(Some(lit_str.value())),
                Err(_) => Err(Error::new_spanned(
                    attr,
                    "#[enum_desc] attribute must be in the form `#[enum_desc(\"description\")]`",
                )),
            };
        }
    }

    // If no #[enum_desc] attribute, return None
    Ok(None)
}

/// Extract rename from #[`enum_rename`] attribute
fn extract_rename(attrs: &[Attribute]) -> Result<Option<String>> {
    for attr in attrs {
        if attr.path().is_ident("enum_rename") {
            return match attr.parse_args::<LitStr>() {
                Ok(lit_str) => Ok(Some(lit_str.value())),
                Err(_) => Err(Error::new_spanned(
                    attr,
                    "#[enum_rename] attribute must be in the form `#[enum_rename(\"new_name\")]`",
                )),
            };
        }
    }

    // If no #[enum_rename] attribute, return None
    Ok(None)
}
