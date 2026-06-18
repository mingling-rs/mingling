use proc_macro::TokenStream;
use quote::quote;
use syn::{Ident, Token, Type, parse_macro_input};

/// Converts a PascalCase/UpperCamelCase identifier string to snake_case.
///
/// Examples:
/// - `ErrorNotFound` → `"error_not_found"`
/// - `ErrorNotDir` → `"error_not_dir"`
/// - `FileIO` → `"file_io"`
/// - `XMLParser` → `"xml_parser"`
fn to_snake_case(ident: &str) -> String {
    let mut result = String::new();
    let mut prev_is_upper = false;

    for (i, c) in ident.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 && !prev_is_upper {
                result.push('_');
            }
            for lower_c in c.to_lowercase() {
                result.push(lower_c);
            }
            prev_is_upper = true;
        } else {
            result.push(c);
            prev_is_upper = false;
        }
    }

    result
}

enum PackErrInput {
    /// pack_err!(ErrorNotFound)
    Simple { type_name: Ident },
    /// pack_err!(ErrorNotDir = PathBuf)
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
            Ok(PackErrInput::Typed {
                type_name,
                inner_type: Box::new(inner_type),
            })
        } else {
            Ok(PackErrInput::Simple { type_name })
        }
    }
}

#[allow(clippy::too_many_lines)]
pub fn pack_err(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as PackErrInput);

    match parsed {
        PackErrInput::Simple { type_name } => {
            let name_str = type_name.to_string();
            let snake_name = to_snake_case(&name_str);

            #[cfg(not(feature = "general_renderer"))]
            let derive = quote! {
                #[derive(::mingling::Groupped)]
            };

            #[cfg(feature = "general_renderer")]
            let derive = quote! {
                #[derive(::mingling::Groupped, ::serde::Serialize)]
            };

            let expanded = quote! {
                #derive
                pub struct #type_name {
                    /// The snake_case name of this error, automatically set at compile time.
                    name: String,
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
            let snake_name = to_snake_case(&name_str);

            #[cfg(not(feature = "general_renderer"))]
            let derive = quote! {
                #[derive(::mingling::Groupped)]
            };

            #[cfg(feature = "general_renderer")]
            let derive = quote! {
                #[derive(::mingling::Groupped, ::serde::Serialize)]
            };

            let expanded = quote! {
                #derive
                pub struct #type_name {
                    /// The snake_case name of this error, automatically set at compile time.
                    name: String,
                    /// Additional context info for this error.
                    info: #inner_type,
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
