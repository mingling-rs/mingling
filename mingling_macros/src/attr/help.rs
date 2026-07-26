use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{Ident, ItemFn, Pat, ReturnType, Signature, TypePath, parse_macro_input};

use crate::get_global_set;
use crate::res_injection::{extract_args_info, generate_immut_resource_bindings};

/// Extracts the user's return type, returning `None` for no return type.
fn extract_user_return_type(sig: &Signature) -> Option<proc_macro2::TokenStream> {
    match &sig.output {
        ReturnType::Type(_, ty) => Some(quote! { #ty }),
        ReturnType::Default => None,
    }
}

pub(crate) fn help_attr(item: TokenStream) -> TokenStream {
    // Parse the function item
    let input_fn = parse_macro_input!(item as ItemFn);

    // Validate the function is not async
    if input_fn.sig.asyncness.is_some() {
        return syn::Error::new(input_fn.sig.span(), "Help function cannot be async")
            .to_compile_error()
            .into();
    }

    // Extract the entry type and resource injection params
    let (_, entry_type, resources) = match extract_args_info(&input_fn.sig) {
        Ok(info) => info,
        Err(e) => return e.to_compile_error().into(),
    };

    // Determine the user's return type for preserving the original function
    let user_return_type = extract_user_return_type(&input_fn.sig);

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
    let original_return_type = user_return_type.clone().unwrap_or(quote! { () });

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

    // Build the call to the original function with resource arguments injected
    let resource_args: Vec<_> = resources
        .iter()
        .map(|res| {
            let var_name = &res.var_name;
            quote! { #var_name }
        })
        .collect();

    // Use a fixed parameter name `prev` for the trait method, regardless of
    // the user's original parameter name (which may be `_` and cannot be
    // referenced in expression position).
    let fixed_prev: Pat = syn::parse_quote!(prev);

    let fn_call = if has_resources {
        quote! { #fn_name(#fixed_prev, #(#resource_args),*) }
    } else {
        quote! { #fn_name(#fixed_prev) }
    };

    // Wrap the function call with modify_res for mutable resources
    let inner_call = if mut_resources.is_empty() {
        fn_call
    } else {
        let mut wrapped = fn_call;
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
            #inner_call
        }
    } else {
        quote! { #inner_call }
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

            fn render_help(#fixed_prev: Self::Entry) -> ::mingling::RenderResult {
                let __help_result = { #help_render_body };
                ::std::convert::Into::into(__help_result)
            }
        }

        ::mingling::macros::register_help!(#entry_type, #struct_name);

    // Keep the original function unchanged
        #(#fn_attrs)*
        #vis fn #fn_name(#original_inputs) -> #original_return_type {
            #(#fn_body_stmts)*
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
            <#struct_name as ::mingling::HelpRequest>::render_help(value)
        }
    }
}
