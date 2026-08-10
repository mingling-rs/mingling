use just_fmt::snake_case;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Ident, Token, Type, parse_macro_input};

enum PackErrInput {
    /// `pack_err!(ErrorNotFound)`
    Simple { type_name: Ident },
    /// `pack_err!(ErrorNotDir = PathBuf)`
    Typed {
        type_name: Ident,
        inner_type: Box<Type>,
    },
}

impl syn::parse::Parse for PackErrInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let type_name: Ident = input.parse()?;

        if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            let inner_type: Type = input.parse()?;
            Ok(Self::Typed {
                type_name,
                inner_type: Box::new(inner_type),
            })
        } else {
            Ok(Self::Simple { type_name })
        }
    }
}

#[allow(clippy::too_many_lines)]
pub(crate) fn pack_err(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as PackErrInput);

    match parsed {
        PackErrInput::Simple { type_name } => {
            let name_str = type_name.to_string();
            let snake_name = snake_case!(&name_str);

            // Note: No longer derives Serialize under structural_renderer.
            // Use pack_err_structural for structured output support.
            let derive = quote! {
                #[derive(::mingling::Grouped)]
            };

            let expanded = quote! {
                #derive
                pub struct #type_name {
                    /// The snake_case name of this error, automatically set at compile time.
                    pub name: String,
                }

                impl ::std::default::Default for #type_name {
                    fn default() -> Self {
                        Self {
                            name: #snake_name.into(),
                        }
                    }
                }

                ::mingling::macros::register_type!(#type_name);
            };

            expanded.into()
        }
        PackErrInput::Typed {
            type_name,
            inner_type,
        } => {
            let name_str = type_name.to_string();
            let snake_name = snake_case!(&name_str);

            // Note: No longer derives Serialize under structural_renderer.
            // Use pack_err_structural for structured output support.
            let derive = quote! {
                #[derive(::mingling::Grouped)]
            };

            let expanded = quote! {
                #derive
                pub struct #type_name {
                    /// The snake_case name of this error, automatically set at compile time.
                    pub name: String,
                    /// Additional context info for this error.
                    pub info: #inner_type,
                }

                impl #type_name {
                    /// Creates a new error with the given info.
                    /// The `name` field is automatically set to the snake_case of the struct name.
                    pub fn new(info: #inner_type) -> Self {
                        Self {
                            name: #snake_name.into(),
                            info,
                        }
                    }
                }

                ::mingling::macros::register_type!(#type_name);
            };

            expanded.into()
        }
    }
}
