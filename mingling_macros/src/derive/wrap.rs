// Doc Not Optimize
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput, Field, Fields, Type, parse_macro_input};

/// The located inner field of a `Wrap` struct.
struct InnerInfo {
    /// Access the inner value from `self`, e.g. `self.name` or `self.0`.
    self_access: TokenStream2,
    /// Access the inner value mutably from `self`, e.g. `&mut self.name`.
    self_access_mut: TokenStream2,
    /// Access the inner value from a bound `wrapper` variable.
    wrapper_access: TokenStream2,
    /// Expression building `Self` from a bound `inner` variable.
    construct: TokenStream2,
    /// The inner field's type.
    inner_ty: Type,
}

/// Parse the input struct and locate the inner field.
///
/// Rules:
/// - Named struct with a single field → that field is the inner field.
/// - Named struct with multiple fields → exactly one field must be marked `#[wrap]`.
/// - Tuple struct with a single field → that field is the inner field.
fn locate_inner(input: &DeriveInput) -> Result<InnerInfo, TokenStream2> {
    let name = &input.ident;

    match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                let all = &fields.named;
                let as_fields: Vec<&Field> = all
                    .iter()
                    .filter(|f| f.attrs.iter().any(|a| a.path().is_ident("wrap")))
                    .collect();

                if as_fields.len() > 1 {
                    let span = as_fields[1].ident.as_ref().unwrap_or(name);
                    return Err(syn::Error::new_spanned(
                        span,
                        "only one field may be marked `#[wrap]`",
                    )
                    .to_compile_error());
                }

                let (inner_field, other_fields): (&Field, Vec<&Field>) = if as_fields.len() == 1 {
                    let inner = as_fields[0];
                    let inner_ident = inner.ident.as_ref().unwrap();
                    let others = all
                        .iter()
                        .filter(|f| f.ident.as_ref() != Some(inner_ident))
                        .collect();
                    (inner, others)
                } else if all.len() == 1 {
                    (all.first().unwrap(), Vec::new())
                } else {
                    return Err(syn::Error::new_spanned(
                        name,
                        "a struct with multiple fields requires exactly one field marked `#[wrap]`",
                    )
                    .to_compile_error());
                };

                let field_ident = inner_field.ident.clone().unwrap();

                let mut named = vec![quote! { #field_ident: inner }];
                for other in other_fields {
                    let ident = other.ident.as_ref().unwrap();
                    named.push(quote! { #ident: ::core::default::Default::default() });
                }

                Ok(InnerInfo {
                    self_access: quote! { self.#field_ident },
                    self_access_mut: quote! { &mut self.#field_ident },
                    wrapper_access: quote! { wrapper.#field_ident },
                    construct: quote! { Self { #(#named),* } },
                    inner_ty: inner_field.ty.clone(),
                })
            }
            Fields::Unnamed(fields) => {
                if fields.unnamed.len() == 1 {
                    Ok(InnerInfo {
                        self_access: quote! { self.0 },
                        self_access_mut: quote! { &mut self.0 },
                        wrapper_access: quote! { wrapper.0 },
                        construct: quote! { Self(inner) },
                        inner_ty: fields.unnamed.first().unwrap().ty.clone(),
                    })
                } else {
                    Err(syn::Error::new_spanned(
                        name,
                        "tuple structs with multiple fields are not supported by `Wrap`; \
                         use a named struct and mark one field with `#[wrap]`",
                    )
                    .to_compile_error())
                }
            }
            Fields::Unit => Err(syn::Error::new_spanned(
                name,
                "unit structs have no inner type; `Wrap` requires a field",
            )
            .to_compile_error()),
        },
        Data::Enum(_) | Data::Union(_) => Err(syn::Error::new_spanned(
            name,
            "`Wrap` can only be derived on structs",
        )
        .to_compile_error()),
    }
}

pub(crate) fn derive_wrap(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let info = match locate_inner(&input) {
        Ok(info) => info,
        Err(err) => return err.into(),
    };

    let InnerInfo {
        self_access,
        self_access_mut,
        wrapper_access,
        construct,
        inner_ty,
    } = info;

    let expanded = quote! {
        impl #impl_generics ::core::convert::From<#inner_ty> for #name #ty_generics #where_clause {
            fn from(inner: #inner_ty) -> Self {
                #construct
            }
        }

        impl #impl_generics ::core::convert::From<#name #ty_generics> for #inner_ty #where_clause {
            fn from(wrapper: #name #ty_generics) -> #inner_ty {
                #wrapper_access
            }
        }

        impl #impl_generics ::core::ops::Deref for #name #ty_generics #where_clause {
            type Target = #inner_ty;

            fn deref(&self) -> &Self::Target {
                &(#self_access)
            }
        }

        impl #impl_generics ::core::ops::DerefMut for #name #ty_generics #where_clause {
            fn deref_mut(&mut self) -> &mut Self::Target {
                #self_access_mut
            }
        }
    };

    expanded.into()
}
