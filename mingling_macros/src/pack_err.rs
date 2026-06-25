use proc_macro::TokenStream;
use quote::quote;
use syn::{Ident, Token, Type, parse_macro_input};

use crate::get_global_set;

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

            // Note: No longer derives Serialize under general_renderer.
            // Use pack_err_structural for structured output support.
            let derive = quote! {
                #[derive(::mingling::Groupped)]
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
            let snake_name = to_snake_case(&name_str);

            // Note: No longer derives Serialize under general_renderer.
            // Use pack_err_structural for structured output support.
            let derive = quote! {
                #[derive(::mingling::Groupped)]
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

/// `pack_err_structural!` — like `pack_err!` but also marks the type as
/// supporting structured output via `StructuralData`.
///
/// # Syntax
///
/// ```rust,ignore
/// pack_err_structural!(ErrorNotFound);
/// pack_err_structural!(ErrorNotDir = PathBuf);
/// ```
///
/// This is equivalent to:
/// ```rust,ignore
/// pack_err!(ErrorNotFound);
/// impl ::mingling::__private::StructuralDataSealed for ErrorNotFound {}
/// impl ::mingling::__private::StructuralData for ErrorNotFound {}
/// ```
#[cfg(feature = "general_renderer")]
pub fn pack_err_structural(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as PackErrInput);

    let type_name = match &parsed {
        PackErrInput::Simple { type_name } => type_name.clone(),
        PackErrInput::Typed { type_name, .. } => type_name.clone(),
    };

    // Register in STRUCTURED_TYPES
    let type_name_str = type_name.to_string();
    get_global_set(&crate::STRUCTURED_TYPES)
        .lock()
        .unwrap()
        .insert(type_name_str);

    let structural_data = quote! {
        impl ::mingling::__private::StructuralDataSealed for #type_name {}
        impl ::mingling::__private::StructuralData for #type_name {}
    };

    // Generate the struct + impls (same as pack_err! but with Serialize derive + sealed)
    match parsed {
        PackErrInput::Simple { type_name } => {
            let name_str = type_name.to_string();
            let snake_name = to_snake_case(&name_str);

            let expanded = quote! {
                #[derive(::mingling::Groupped, ::serde::Serialize)]
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

                #structural_data
            };

            expanded.into()
        }
        PackErrInput::Typed {
            type_name,
            inner_type,
        } => {
            let name_str = type_name.to_string();
            let snake_name = to_snake_case(&name_str);

            let expanded = quote! {
                #[derive(::mingling::Groupped, ::serde::Serialize)]
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

                #structural_data
            };

            expanded.into()
        }
    }
}
