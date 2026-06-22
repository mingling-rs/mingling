use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::spanned::Spanned;
use syn::{ItemFn, ReturnType, Signature, Type, TypePath, parse_macro_input};

use crate::get_global_set;
use crate::res_injection::{extract_args_info, generate_immut_resource_bindings};

/// Extracts and returns the return type from the function signature (or None for `()` / no return type).
fn extract_return_type(sig: &Signature) -> Option<syn::Type> {
    match &sig.output {
        ReturnType::Type(_, ty) => {
            match &**ty {
                // `()` means no custom return type
                Type::Tuple(tuple) if tuple.elems.is_empty() => None,
                // Any other return type is allowed
                custom_ty => Some((*custom_ty).clone()),
            }
        }
        ReturnType::Default => None,
    }
}

#[allow(clippy::too_many_lines)]
pub fn renderer_attr(attr: TokenStream, item: TokenStream) -> TokenStream {
    // #[renderer] takes no arguments; always use the default program path
    let _ = attr;
    let program_path = crate::default_program_path();
    let program_type = &program_path;

    // Parse the function item
    let input_fn = parse_macro_input!(item as ItemFn);

    // Validate the function is not async
    if input_fn.sig.asyncness.is_some() {
        return syn::Error::new(input_fn.sig.span(), "Renderer function cannot be async")
            .to_compile_error()
            .into();
    }

    // Extract the previous type, parameter name, and resource injection params
    let (prev_param, previous_type, resources) = match extract_args_info(&input_fn.sig) {
        Ok(info) => info,
        Err(e) => return e.to_compile_error().into(),
    };

    // Validate return type – now returns Some(type) if custom type, None if ()
    let return_type = extract_return_type(&input_fn.sig);

    // Get function body statements
    let fn_body_stmts: Vec<syn::Stmt> = input_fn.block.stmts.clone();

    // Get function attributes (excluding the renderer attribute)
    let mut fn_attrs = input_fn.attrs.clone();

    // Remove any #[renderer(...)] attributes to avoid infinite recursion
    fn_attrs.retain(|attr| !attr.path().is_ident("renderer"));

    // Get function visibility
    let vis = &input_fn.vis;

    // Get function name
    let fn_name = &input_fn.sig.ident;

    // Generate struct name from function name using pascal_case
    let internal_name = format!(
        "__internal_renderer_{}",
        just_fmt::snake_case!(fn_name.to_string())
    );
    let struct_name = syn::Ident::new(&internal_name, fn_name.span());

    let has_resources = !resources.is_empty();
    let has_mut_resources = resources.iter().any(|r| r.is_mut);

    // Generate resource bindings for immutable resources
    let immut_resource_stmts = generate_immut_resource_bindings(resources.iter(), program_type);
    let mut_resources: Vec<_> = resources.iter().filter(|r| r.is_mut).collect();

    // Determine public return type and the expression to return dummy_r
    let (public_return_type, result_return) = if let Some(custom_ty) = &return_type {
        // User specified a custom return type (e.g. -> String)
        let ret_ty = quote! { #custom_ty };
        let expr = quote! { dummy_r.into() };
        (ret_ty, expr)
    } else {
        // Return type is () — no custom return type specified
        let ret_ty = quote! { () };
        let expr = quote! {
            if !dummy_r.is_empty() {
                ::std::println!("{}", &*dummy_r);
            }
        };
        (ret_ty, expr)
    };

    let inner_body_with_resources = if has_mut_resources {
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
    } else {
        quote! { #(#fn_body_stmts)* }
    };

    // Build the Renderer::render body with resource injection
    // Renderer::render returns (), output goes through __renderer_inner_result parameter.
    // Resources are injected from the program context here.
    let render_fn_body = if has_resources {
        quote! {
            #(#immut_resource_stmts)*
            #inner_body_with_resources
        }
    } else {
        quote! { #inner_body_with_resources }
    };

    // Build the original function body
    // The original function preserves the user's signature and return type.
    // Resource parameters are passed directly by the caller, NOT injected from context.
    let original_fn_body = {
        quote! {
            let mut dummy_r = ::mingling::RenderResult::default();
            {
                let __renderer_inner_result = &mut dummy_r;
                #(#fn_body_stmts)*
            }
            #result_return
        }
    };

    // Keep the original function signature unchanged (same params as user wrote)
    let original_inputs = input_fn.sig.inputs.clone();

    let expanded = quote! {
        #(#fn_attrs)*
        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        #vis struct #struct_name;

        ::mingling::macros::register_renderer!(#previous_type, #struct_name);

        impl ::mingling::Renderer for #struct_name {
            type Previous = #previous_type;

            fn render(#prev_param: Self::Previous, __renderer_inner_result: &mut ::mingling::RenderResult) {
                #render_fn_body
            }
        }

        // Keep the original function for internal use (without r parameter)
        #(#fn_attrs)*
        #vis fn #fn_name(#original_inputs) -> #public_return_type {
            #original_fn_body
        }
    };

    expanded.into()
}

/// Builds the renderer entry for the global renderers list
pub fn build_renderer_entry(
    struct_name: &syn::Ident,
    previous_type: &TypePath,
) -> proc_macro2::TokenStream {
    let enum_variant = &previous_type.path.segments.last().unwrap().ident;
    quote! {
        #struct_name => #enum_variant,
    }
}

/// Builds the renderer existence check entry
pub fn build_renderer_exist_entry(previous_type: &TypePath) -> proc_macro2::TokenStream {
    let enum_variant = &previous_type.path.segments.last().unwrap().ident;
    quote! {
        Self::#enum_variant => true,
    }
}

/// Builds the general renderer entry
#[cfg(feature = "general_renderer")]
pub fn build_general_renderer_entry(previous_type: &TypePath) -> proc_macro2::TokenStream {
    let enum_variant = &previous_type.path.segments.last().unwrap().ident;
    quote! {
        Self::#enum_variant => {
            // SAFETY: Only types that match will enter this branch for forced conversion,
            // and `AnyOutput::new` ensures the type implements serde::Serialize
            let raw = unsafe { any.restore::<#previous_type>().unwrap_unchecked() };
            let mut __renderer_inner_result = ::mingling::RenderResult::default();
            ::mingling::GeneralRenderer::render(&raw, setting, &mut __renderer_inner_result)?;
            Ok(__renderer_inner_result)
        }
    }
}

pub fn register_renderer(input: TokenStream) -> TokenStream {
    // Parse the input as a comma-separated list of arguments
    let input_parsed = syn::parse_macro_input!(input with syn::punctuated::Punctuated<syn::Expr, syn::Token![,]>::parse_terminated);

    // Check that there are exactly two elements
    if input_parsed.len() != 2 {
        return syn::Error::new(
            input_parsed.span(),
            "Expected exactly two comma-separated arguments: `PreviousType, StructName`",
        )
        .to_compile_error()
        .into();
    }

    // Extract the two elements
    let previous_type_expr = &input_parsed[0];
    let struct_name_expr = &input_parsed[1];

    // Convert expressions to TypePath and Ident
    let previous_type = match syn::parse2::<TypePath>(previous_type_expr.to_token_stream()) {
        Ok(ty) => ty,
        Err(e) => return e.to_compile_error().into(),
    };

    let struct_name = match syn::parse2::<syn::Ident>(struct_name_expr.to_token_stream()) {
        Ok(ident) => ident,
        Err(e) => return e.to_compile_error().into(),
    };

    // Register the renderer in the global list
    let renderer_entry = build_renderer_entry(&struct_name, &previous_type);
    let renderer_exist_entry = build_renderer_exist_entry(&previous_type);
    #[cfg(feature = "general_renderer")]
    let general_renderer_entry = build_general_renderer_entry(&previous_type);

    let renderer_entry_str = renderer_entry.to_string();
    let renderer_exist_entry_str = renderer_exist_entry.to_string();

    #[cfg(feature = "general_renderer")]
    let general_renderer_entry_str = general_renderer_entry.to_string();

    // Check for duplicate variant before acquiring other locks
    let variant_name = previous_type
        .path
        .segments
        .last()
        .unwrap()
        .ident
        .to_string();
    {
        let renderers = get_global_set(&crate::RENDERERS).lock().unwrap();
        if let Err(err) = crate::check_duplicate_variant(
            &renderers,
            &variant_name,
            "renderer",
            previous_type.span(),
        ) {
            return err.into();
        }
    } // renderers lock released here

    let mut renderers = get_global_set(&crate::RENDERERS).lock().unwrap();
    let mut renderer_exist = get_global_set(&crate::RENDERERS_EXIST).lock().unwrap();

    #[cfg(feature = "general_renderer")]
    let mut general_renderers = get_global_set(&crate::GENERAL_RENDERERS).lock().unwrap();

    renderers.insert(renderer_entry_str);
    renderer_exist.insert(renderer_exist_entry_str);

    #[cfg(feature = "general_renderer")]
    general_renderers.insert(general_renderer_entry_str);

    quote! {}.into()
}
