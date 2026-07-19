use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, Result as SynResult, TypePath};

/// Input for the `group!` macro
///
/// # Syntax
///
/// ```rust,ignore
/// /// Only a type path — uses default `crate::ThisProgram` as program
/// group!(std::io::Error);
/// group!(ParseIntError);
///
/// /// With an alias — creates a `pub type Alias = Path;` and uses `Alias` as variant name
/// group!(IoError = std::io::Error);
/// ```
enum GroupInput {
    /// `group!(TypePath)` — variant name is the last path segment
    Plain(TypePath),

    /// `group!(Alias = TypePath)` — variant name is `Alias`, also generates `pub type Alias = TypePath;`
    Aliased { alias: Ident, type_path: TypePath },
}

impl Parse for GroupInput {
    fn parse(input: ParseStream) -> SynResult<Self> {
        // Peek ahead: if the second token is `=`, parse as aliased form
        let fork = input.fork();
        let _first: Ident = fork.parse()?;
        if fork.peek(syn::Token![=]) {
            // Consume the ident and `=` from the real input
            let alias: Ident = input.parse()?;
            let _eq: syn::Token![=] = input.parse()?;
            let type_path: TypePath = input.parse()?;
            Ok(GroupInput::Aliased { alias, type_path })
        } else {
            let type_path: TypePath = input.parse()?;
            Ok(GroupInput::Plain(type_path))
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
        quote! {
            #[allow(unused_imports)]
            use #type_path;
        }
    } else {
        // Single ident: import from parent scope
        let ident = type_simple_name(type_path);
        quote! {
            #[allow(unused_imports)]
            use super::#ident;
        }
    }
}

pub(crate) fn group_macro(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as GroupInput);

    let is_aliased = matches!(input, GroupInput::Aliased { .. });

    let (type_path, type_name, alias_stmt) = match input {
        GroupInput::Plain(type_path) => {
            let type_name = type_simple_name(&type_path);
            (type_path, type_name, quote! {})
        }
        GroupInput::Aliased { alias, type_path } => {
            let type_name = alias.clone();
            let alias_stmt = quote! {
                pub type #alias = #type_path;
            };
            (type_path, type_name, alias_stmt)
        }
    };

    let program_path = crate::default_program_path();

    // Create a unique module name from the type path (use alias name for aliased form)
    let module_name = module_name_from_type(&type_path);
    // Generate the appropriate `use` statement for the type
    let type_use = gen_type_use(&type_path);

    // For aliased form, also import the alias from parent scope
    let alias_use = if is_aliased {
        quote! { use super::#type_name; }
    } else {
        quote! {}
    };

    // Generate the module with the Grouped implementation
    let expanded = quote! {
        #alias_stmt
        #[allow(non_camel_case_types)]
        mod #module_name {
            use #program_path as __MinglingProgram;
            #type_use
            #alias_use

            impl ::mingling::Grouped<__MinglingProgram> for #type_name {
                fn member_id() -> __MinglingProgram {
                    __MinglingProgram::#type_name
                }
            }

            ::mingling::macros::register_type!(#type_name);
        }
    };

    expanded.into()
}
