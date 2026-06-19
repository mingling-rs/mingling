use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, Path, Result as SynResult, Token, TypePath};

/// Input for the `group!` macro
///
/// # Syntax
///
/// ```rust,ignore
/// // Explicit mode: specify both program path and type path
/// group!(crate::ThisProgram, std::io::Error);
///
/// // Implicit mode: only type path, uses default `crate::ThisProgram` as program
/// group!(std::io::Error);
/// group!(ParseIntError);
/// ```
enum GroupInput {
    Explicit {
        program_path: Path,
        type_path: TypePath,
    },
    Implicit {
        type_path: TypePath,
    },
}

impl Parse for GroupInput {
    fn parse(input: ParseStream) -> SynResult<Self> {
        // Parse the first path (could be program path or type path)
        let first_path: Path = input.parse()?;

        // If followed by a comma, it's explicit mode: Path, TypePath
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            let type_path: TypePath = input.parse()?;
            Ok(GroupInput::Explicit {
                program_path: first_path,
                type_path,
            })
        } else {
            // Otherwise it's implicit mode: just a type path
            Ok(GroupInput::Implicit {
                type_path: TypePath {
                    qself: None,
                    path: first_path,
                },
            })
        }
    }
}

/// Convert a type path into a valid module name segment
///
/// e.g. `std::io::Error` -> `internal_group_std_io_error`
fn module_name_from_type(type_path: &TypePath) -> Ident {
    let segments: Vec<String> = type_path
        .path
        .segments
        .iter()
        .map(|seg| seg.ident.to_string().to_lowercase())
        .collect();
    Ident::new(
        &format!("internal_group_{}", segments.join("_")),
        proc_macro2::Span::call_site(),
    )
}

/// Get the last segment name of a type path (the simple type name)
///
/// e.g. `std::io::Error` -> `Error`
fn type_simple_name(type_path: &TypePath) -> Ident {
    type_path
        .path
        .segments
        .last()
        .expect("TypePath must have at least one segment")
        .ident
        .clone()
}

/// Generate the `use` token for the type path inside the generated module.
///
/// - Multi-segment path (e.g. `std::num::ParseIntError`): `use std::num::ParseIntError;`
/// - Single-segment path (e.g. `ParseIntError`): `use super::ParseIntError;`
fn gen_type_use(type_path: &TypePath) -> proc_macro2::TokenStream {
    if type_path.path.segments.len() > 1 {
        // Full path: use it directly
        quote! { use #type_path; }
    } else {
        // Single ident: import from parent scope
        let ident = type_simple_name(type_path);
        quote! { use super::#ident; }
    }
}

pub fn group_macro(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as GroupInput);

    let (program_path, type_path) = match input {
        GroupInput::Explicit {
            program_path,
            type_path,
        } => (quote! { #program_path }, type_path),
        GroupInput::Implicit { type_path } => (crate::default_program_path(), type_path),
    };

    // Use the type's simple name as the enum variant identifier
    let type_name = type_simple_name(&type_path);
    // Create a unique module name from the full type path
    let module_name = module_name_from_type(&type_path);
    // Generate the appropriate `use` statement for the type
    let type_use = gen_type_use(&type_path);

    // Generate the module with the Groupped implementation
    let expanded = quote! {
        #[allow(non_camel_case_types)]
        mod #module_name {
            use #program_path as __MinglingProgram;
            #type_use

            impl ::mingling::Groupped<__MinglingProgram> for #type_name {
                fn member_id() -> __MinglingProgram {
                    __MinglingProgram::#type_name
                }
            }

            ::mingling::macros::register_type!(#type_name);
        }
    };

    expanded.into()
}
