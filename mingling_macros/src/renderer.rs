use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::spanned::Spanned;
use syn::{ItemFn, ReturnType, Signature, TypePath, parse_macro_input};

use crate::get_global_set;
use crate::res_injection::{extract_args_info, generate_immut_resource_bindings};

/// Extracts the user's return type, returning `None` for no return type.
fn extract_user_return_type(sig: &Signature) -> Option<proc_macro2::TokenStream> {
    match &sig.output {
        ReturnType::Type(_, ty) => Some(quote! { #ty }),
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

    // Determine the user's return type and whether it needs to be converted to RenderResult
    let user_return_type = extract_user_return_type(&input_fn.sig);

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
    // The user's body now directly creates and returns a RenderResult.
    let render_fn_body = if has_resources {
        quote! {
            #(#immut_resource_stmts)*
            #inner_body_with_resources
        }
    } else {
        quote! { #inner_body_with_resources }
    };

    // The original function preserves the user's exact signature and body.
    // Resource parameters are passed directly by the caller, NOT injected from context.
    let original_inputs = input_fn.sig.inputs.clone();
    let original_return_type = user_return_type.clone().unwrap_or(quote! { () });

    let expanded = quote! {
        #(#fn_attrs)*
        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        #vis struct #struct_name;

        ::mingling::macros::register_renderer!(#previous_type, #struct_name);

        impl ::mingling::Renderer for #struct_name {
            type Previous = #previous_type;

            fn render(#prev_param: Self::Previous) -> ::mingling::RenderResult {
                let __renderer_result = { #render_fn_body };
                ::std::convert::Into::into(__renderer_result)
            }
        }

        // Keep the original function unchanged
        #(#fn_attrs)*
        #vis fn #fn_name(#original_inputs) -> #original_return_type {
            #(#fn_body_stmts)*
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

/// Builds the structural renderer entry
#[cfg(feature = "structural_renderer")]
pub fn build_structural_renderer_entry(previous_type: &TypePath) -> proc_macro2::TokenStream {
    let enum_variant = &previous_type.path.segments.last().unwrap().ident;
    quote! {
        Self::#enum_variant => {
            // SAFETY: Only types that match will enter this branch for forced conversion,
            // and `AnyOutput::new` ensures the type implements serde::Serialize
            let raw = unsafe { any.restore::<#previous_type>().unwrap_unchecked() };
            let mut __renderer_inner_result = ::mingling::RenderResult::default();
            ::mingling::StructuralRenderer::render(&raw, setting, &mut __renderer_inner_result)?;
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
    #[cfg(feature = "structural_renderer")]
    let structural_renderer_entry = build_structural_renderer_entry(&previous_type);

    let renderer_entry_str = renderer_entry.to_string();
    let renderer_exist_entry_str = renderer_exist_entry.to_string();

    #[cfg(feature = "structural_renderer")]
    let structural_renderer_entry_str = structural_renderer_entry.to_string();

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
            &renderer_entry_str,
            &variant_name,
            "renderer",
            previous_type.span(),
        ) {
            return err.into();
        }
    } // renderers lock released here

    let mut renderers = get_global_set(&crate::RENDERERS).lock().unwrap();
    let mut renderer_exist = get_global_set(&crate::RENDERERS_EXIST).lock().unwrap();

    #[cfg(feature = "structural_renderer")]
    let mut structural_renderers = get_global_set(&crate::STRUCTURAL_RENDERERS).lock().unwrap();

    renderers.insert(renderer_entry_str);
    renderer_exist.insert(renderer_exist_entry_str);

    // Only register structural renderer if the type is in STRUCTURED_TYPES
    #[cfg(feature = "structural_renderer")]
    {
        let is_structured = get_global_set(&crate::STRUCTURED_TYPES)
            .lock()
            .unwrap()
            .contains(&variant_name);
        if is_structured {
            structural_renderers.insert(structural_renderer_entry_str);
        }
    }

    quote! {}.into()
}
