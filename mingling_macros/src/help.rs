use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::spanned::Spanned;
use syn::{Ident, ItemFn, ReturnType, Signature, Type, TypePath, parse_macro_input};

use crate::get_global_set;
use crate::res_injection::{extract_args_info, generate_immut_resource_bindings};

/// Validates the return type is () or empty
fn validate_return_type(sig: &Signature) -> syn::Result<()> {
    match &sig.output {
        ReturnType::Type(_, ty) => match &**ty {
            Type::Tuple(tuple) if tuple.elems.is_empty() => Ok(()),
            _ => Err(syn::Error::new(
                ty.span(),
                "Help function must return () or have no return type",
            )),
        },
        ReturnType::Default => Ok(()),
    }
}

pub fn help_attr(item: TokenStream) -> TokenStream {
    // Parse the function item
    let input_fn = parse_macro_input!(item as ItemFn);

    // Validate the function is not async
    if input_fn.sig.asyncness.is_some() {
        return syn::Error::new(input_fn.sig.span(), "Help function cannot be async")
            .to_compile_error()
            .into();
    }

    // Extract the entry type, parameter name, and resource injection params
    let (prev_param, entry_type, resources) = match extract_args_info(&input_fn.sig) {
        Ok(info) => info,
        Err(e) => return e.to_compile_error().into(),
    };

    // Validate return type
    if let Err(e) = validate_return_type(&input_fn.sig) {
        return e.to_compile_error().into();
    }

    // Get the function body
    let fn_body = &input_fn.block;
    let fn_body_stmts = &fn_body.stmts;

    // Get function attributes excluding the help attribute
    let mut fn_attrs = input_fn.attrs.clone();
    fn_attrs.retain(|attr| !attr.path().is_ident("help"));

    // Get function visibility
    let vis = &input_fn.vis;

    // Get function name
    let fn_name = &input_fn.sig.ident;

    // Get original inputs to keep the original function

    let original_inputs = input_fn.sig.inputs.clone();

    // Generate internal name using snake_case
    let internal_name = format!(
        "__internal_help_{}",
        just_fmt::snake_case!(fn_name.to_string())
    );
    let struct_name = Ident::new(&internal_name, fn_name.span());

    let program_type = crate::default_program_path();
    let has_resources = !resources.is_empty();
    let mut_resources: Vec<_> = resources.iter().filter(|r| r.is_mut).collect();

    // Generate immutable resource bindings
    let immut_resource_stmts = generate_immut_resource_bindings(resources.iter(), &program_type);

    // Build the render_help body with resource injection
    // Use modify_res for mutable resources same pattern as renderer.rs

    let wrapped_body = if mut_resources.is_empty() {
        quote! { #(#fn_body_stmts)* }
    } else {
        let mut wrapped = quote! { #(#fn_body_stmts)* };
        for res in mut_resources.iter().rev() {
            let var_name = &res.var_name;
            let inner_type = &res.inner_type;
            wrapped = quote! {
                ::mingling::this::<#program_type>().modify_res(|#var_name: &mut #inner_type| {
                    #wrapped
                })
            };
        }
        wrapped
    };

    let help_render_body = if has_resources {
        quote! {
            #(#immut_resource_stmts)*
            #wrapped_body
        }
    } else {
        quote! { #(#fn_body_stmts)* }
    };

    // Register the help request mapping
    let help_entry = build_help_entry(&struct_name, &entry_type);
    let entry_str = help_entry.to_string();

    // Check for duplicate variant before inserting
    let variant_name = entry_type.path.segments.last().unwrap().ident.to_string();
    {
        let helps = get_global_set(&crate::HELP_REQUESTS).lock().unwrap();
        if let Err(err) = crate::check_duplicate_variant(
            &helps,
            &entry_str,
            &variant_name,
            "help",
            entry_type.span(),
        ) {
            return err.into();
        }
    }

    get_global_set(&crate::HELP_REQUESTS)
        .lock()
        .unwrap()
        .insert(entry_str);

    // Generate the struct and HelpRequest implementation
    let expanded = quote! {
        #(#fn_attrs)*
        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        #vis struct #struct_name;

        impl ::mingling::HelpRequest for #struct_name {
            type Entry = #entry_type;

            fn render_help(#prev_param: Self::Entry, __renderer_inner_result: &mut ::mingling::RenderResult) {
                #help_render_body
            }
        }

        ::mingling::macros::register_help!(#entry_type, #struct_name);

        // Keep the original function for internal use with original params without __renderer_inner_result

        #(#fn_attrs)*
        #vis fn #fn_name(#original_inputs) {
            let mut dummy_r = ::mingling::RenderResult::default();
            let __renderer_inner_result = &mut dummy_r;
            #fn_body
        }
    };

    expanded.into()
}

/// Builds a help request entry for the global help requests list
fn build_help_entry(struct_name: &Ident, entry_type: &TypePath) -> proc_macro2::TokenStream {
    let enum_variant = &entry_type.path.segments.last().unwrap().ident;
    quote! {
        Self::#enum_variant => {
            // SAFETY: The member_id check ensures that `any` contains a value of type `#entry_type`,
            // so downcasting to `#entry_type` is safe.
            let value = unsafe { any.downcast::<#entry_type>().unwrap_unchecked() };
            <#struct_name as ::mingling::HelpRequest>::render_help(value, __renderer_inner_result);
        }
    }
}

pub fn register_help(input: TokenStream) -> TokenStream {
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
    let mut helps = get_global_set(&crate::HELP_REQUESTS).lock().unwrap();
    if helps.contains(&entry_str) {
        // Already registered by `#[help]`, no duplicate check needed
        return quote! {}.into();
    }

    // Check for duplicate variant (different struct, same type)
    let variant_name = entry_type.path.segments.last().unwrap().ident.to_string();
    if let Err(err) =
        crate::check_duplicate_variant(&helps, &entry_str, &variant_name, "help", entry_type.span())
    {
        return err.into();
    }

    helps.insert(entry_str);

    quote! {}.into()
}
