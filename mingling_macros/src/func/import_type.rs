// Doc Not Optimize
use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{Ident, Result as SynResult, TypePath};

/// Input for the `import_type!` macro
///
/// The target type path must be fully qualified.
enum ImportTypeInput {
    /// `import_type!(path::to::Type)`
    Plain(TypePath),

    /// `import_type!(Alias = path::to::Type)`
    Aliased { alias: Ident, type_path: TypePath },
}

impl Parse for ImportTypeInput {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let fork = input.fork();
        let _first: Ident = fork.parse()?;
        if fork.peek(syn::Token![=]) {
            let alias: Ident = input.parse()?;
            let _eq: syn::Token![=] = input.parse()?;
            let type_path: TypePath = input.parse()?;
            Ok(Self::Aliased { alias, type_path })
        } else {
            let type_path: TypePath = input.parse()?;
            Ok(Self::Plain(type_path))
        }
    }
}

/// Convert a fully-qualified type path into a `__mingling_import_*` module name.
///
/// e.g. `std::io::Error` -> `__mingling_import_std_io_error`
fn module_name_from_type(type_path: &TypePath) -> Ident {
    let segments: Vec<String> = type_path
        .path
        .segments
        .iter()
        .map(|seg| seg.ident.to_string().to_lowercase())
        .collect();
    Ident::new(
        &format!("__mingling_import_{}", segments.join("_")),
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

pub(crate) fn import_type_macro(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as ImportTypeInput);

    let is_aliased = matches!(input, ImportTypeInput::Aliased { .. });

    let (type_path, type_name, alias_stmt) = match input {
        ImportTypeInput::Plain(type_path) => {
            let type_name = type_simple_name(&type_path);
            (type_path, type_name, quote! {})
        }
        ImportTypeInput::Aliased { alias, type_path } => {
            let type_name = alias.clone();
            let alias_stmt = quote! {
                pub type #alias = #type_path;
            };
            (type_path, type_name, alias_stmt)
        }
    };

    // `import_type!` requires a fully-qualified path.
    if type_path.path.segments.len() < 2 {
        return syn::Error::new(
            type_path.span(),
            "import_type! requires a fully qualified path, e.g. `import_type!(std::io::Error)`",
        )
        .to_compile_error()
        .into();
    }

    let program_path = crate::default_program_path();

    let module_name = module_name_from_type(&type_path);
    let type_use = if is_aliased {
        quote! { pub use super::#type_name; }
    } else {
        quote! { pub use #type_path; }
    };

    let expanded = quote! {
        #alias_stmt

        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        pub mod #module_name {
            use #program_path as __MinglingProgram;

            #[allow(unused_imports)]
            #type_use

            /// SAFETY: This is an internal implementation of the `import_type!` macro,
            /// guaranteeing that the enum value registered by the `register_type!` macro
            /// is exactly the same as the actual return value,
            /// which can be confirmed via the `Ident` in the `quote!` block.
            unsafe impl ::mingling::Grouped<__MinglingProgram> for #type_name {
                fn member_id() -> __MinglingProgram {
                    __MinglingProgram::#type_name
                }
            }

            ::mingling::macros::register_type!(#type_name);
        }
    };

    expanded.into()
}
