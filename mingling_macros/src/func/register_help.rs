use proc_macro::TokenStream;
use quote::ToTokens;
use syn::TypePath;
use syn::spanned::Spanned;

use crate::get_global_set;

pub(crate) fn register_help(input: TokenStream) -> TokenStream {
    // Parse the input as a comma-separated list of arguments
    let input_parsed = syn::parse_macro_input!(input with syn::punctuated::Punctuated<syn::Expr, syn::Token![,]>::parse_terminated);

    // Check if there are exactly two elements
    if input_parsed.len() != 2 {
        return syn::Error::new(
            input_parsed.span(),
            "Expected exactly two comma-separated arguments: `EntryType, StructName`",
        )
        .to_compile_error()
        .into();
    }

    // Extract the two elements
    let entry_type_expr = &input_parsed[0];
    let struct_name_expr = &input_parsed[1];

    // Convert expressions to TypePath and Ident
    let entry_type = match syn::parse2::<TypePath>(entry_type_expr.to_token_stream()) {
        Ok(ty) => ty,
        Err(e) => return e.to_compile_error().into(),
    };

    let struct_name = match syn::parse2::<syn::Ident>(struct_name_expr.to_token_stream()) {
        Ok(ident) => ident,
        Err(e) => return e.to_compile_error().into(),
    };

    // Register the help request mapping
    let help_entry = build_help_entry(&struct_name, &entry_type);
    let entry_str = help_entry.to_string();

    // Check if entry was already pre-inserted by `#[help]` attribute
    let helps = get_global_set(&crate::HELP_REQUESTS);
    let help_set = helps.lock().unwrap();
    if help_set.contains(&entry_str) {
        // Already registered by `#[help]`, no duplicate check needed
        return quote::quote! {}.into();
    }

    // Check for duplicate variant (different struct, same type)
    let variant_name = entry_type.path.segments.last().unwrap().ident.to_string();
    let dup_check = crate::check_duplicate_variant(
        &help_set,
        &entry_str,
        &variant_name,
        "help",
        entry_type.span(),
    );
    if let Err(err) = dup_check {
        return err.into();
    }

    drop(help_set);
    helps.lock().unwrap().insert(entry_str);

    quote::quote! {}.into()
}

fn build_help_entry(struct_name: &syn::Ident, entry_type: &TypePath) -> proc_macro2::TokenStream {
    let enum_variant = entry_type.path.segments.last().unwrap().ident.clone();
    quote::quote! {
        Self::#enum_variant => {
            // SAFETY: The member_id check ensures that `any` contains a value of type `#entry_type`,
            // so downcasting to `#entry_type` is safe.
            let value = unsafe { any.downcast::<#entry_type>().unwrap_unchecked() };
            <#struct_name as ::mingling::HelpRequest>::render_help(value)
        }
    }
}
