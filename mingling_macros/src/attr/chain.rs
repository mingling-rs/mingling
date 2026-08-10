#![allow(clippy::too_many_arguments)]

use crate::res_injection::{
    ResourceInjection, extract_args_info, generate_immut_resource_bindings,
    wrap_body_with_mut_resources, wrap_body_with_mut_resources_async,
};
use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::spanned::Spanned;
use syn::{Ident, ItemFn, Pat, ReturnType, Signature, Type, TypePath, parse_macro_input};

/// Checks whether the return type is `()`
fn is_unit_return_type(sig: &Signature) -> bool {
    match &sig.output {
        ReturnType::Type(_, ty) => match &**ty {
            Type::Tuple(tuple) => tuple.elems.is_empty(),
            _ => false,
        },
        ReturnType::Default => true,
    }
}

/// Builds the `proc` function implementation inside the generated `Chain` impl.
///
/// Instead of inlining the user's body, the trait method calls the original
/// function by name, with resources injected from the application context.
#[allow(clippy::needless_pass_by_value)]
fn generate_proc_fn(
    fn_name: &Ident,
    has_resources: bool,
    resources: &[ResourceInjection],
    program_type: &proc_macro2::TokenStream,
    previous_type: &TypePath,
    is_async_fn: bool,
    is_unit_return: bool,
) -> proc_macro2::TokenStream {
    let immut_resource_stmts = generate_immut_resource_bindings(resources.iter(), program_type);
    let mut_resources: Vec<_> = resources.iter().filter(|r| r.is_mut).collect();

    // Use a fixed parameter name `prev` for the trait method, regardless of
    // the user's original parameter name (which may be `_` and cannot be
    // referenced in expression position).
    let fixed_prev: Pat = syn::parse_quote!(prev);

    // Build the call to the original function with resource arguments injected.
    // The variable names come from the resource injection bindings
    // (immut bindings are `let #name = …`, mut closures receive `|#name: &mut T|`),
    // which match the original function's parameter names.
    let resource_args: Vec<_> = resources
        .iter()
        .map(|res| {
            let var_name = &res.var_name;
            quote! { #var_name }
        })
        .collect();

    let fn_call = if has_resources {
        let call = quote! { #fn_name(#fixed_prev, #(#resource_args),*) };
        if is_async_fn {
            quote! { #call.await }
        } else {
            call
        }
    } else {
        let call = quote! { #fn_name(#fixed_prev) };
        if is_async_fn {
            quote! { #call.await }
        } else {
            call
        }
    };

    // Convert the function call to a syn::Stmt so existing wrapping functions can use it.
    // For non-unit returns, wrap in `to_chain()` so mutable-resource closures
    // return `ChainProcess<C>` (as required by `__modify_res_and_return_route`).
    let fn_call_expr: syn::Expr = if is_unit_return {
        syn::parse_quote! { #fn_call }
    } else {
        syn::parse_quote! { ::mingling::Routable::<#program_type>::to_chain(#fn_call) }
    };
    let fn_call_stmt = syn::Stmt::Expr(fn_call_expr, None);

    let wrapped_body = if is_async_fn && !mut_resources.is_empty() {
        wrap_body_with_mut_resources_async(&[fn_call_stmt], &mut_resources, program_type)
    } else {
        wrap_body_with_mut_resources(
            &[fn_call_stmt],
            &mut_resources,
            program_type,
            is_unit_return,
        )
    };

    let proc_body = if is_unit_return {
        let body_with_ending = if has_resources {
            quote! {
                #(#immut_resource_stmts)*
                #wrapped_body;
                <crate::ResultEmpty as ::mingling::Routable::<crate::ThisProgram>>
                ::to_chain(crate::ResultEmpty)
            }
        } else {
            quote! {
                #wrapped_body;
                <crate::ResultEmpty as ::mingling::Routable::<crate::ThisProgram>>
                ::to_chain(crate::ResultEmpty)
            }
        };
        quote! { #body_with_ending }
    } else {
        let body = if has_resources {
            quote! {
                #(#immut_resource_stmts)*
                #wrapped_body
            }
        } else {
            quote! { #wrapped_body }
        };
        quote! {
            let __chain_result = { #body };
            ::mingling::Routable::<#program_type>::to_chain(__chain_result)
        }
    };

    #[cfg(feature = "async")]
    {
        quote! {
            async fn proc(#fixed_prev: #previous_type) -> ::mingling::ChainProcess<#program_type> {
                #proc_body
            }
        }
    }

    #[cfg(not(feature = "async"))]
    {
        quote! {
            fn proc(#fixed_prev: #previous_type) -> ::mingling::ChainProcess<#program_type> {
                #proc_body
            }
        }
    }
}

/// Assembles the final expanded output: hidden struct, `register_chain!` invocation,
/// `Chain` impl with the `proc` method, and the preserved original function.
fn generate_struct_and_impl(
    fn_attrs: &[syn::Attribute],
    vis: &syn::Visibility,
    struct_name: &Ident,
    previous_type: &TypePath,
    previous_type_str: &proc_macro2::TokenStream,
    program_type: &proc_macro2::TokenStream,
    proc_fn: &proc_macro2::TokenStream,
    original_fn: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    quote! {
        #(#fn_attrs)*
        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        #vis struct #struct_name;

        ::mingling::macros::register_chain!(#previous_type_str, #struct_name);

        impl ::mingling::Chain<#program_type> for #struct_name {
            type Previous = #previous_type;

            #proc_fn
        }

        // Keep the original function unchanged
        #original_fn
    }
}

/// Ensures the function is not async when the `async` feature is disabled.
#[cfg(not(feature = "async"))]
fn reject_async(sig: &Signature) -> Result<(), proc_macro2::TokenStream> {
    if sig.asyncness.is_some() {
        return Err(syn::Error::new(
            sig.span(),
            "Chain function cannot be async when async feature is disabled",
        )
        .to_compile_error());
    }
    Ok(())
}

pub(crate) fn chain_attr(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Reject non-empty attribute arguments; #[chain] must be bare
    if !attr.is_empty() {
        return syn::Error::new(
            attr.into_iter().next().unwrap().span().into(),
            "#[chain] does not accept arguments",
        )
        .to_compile_error()
        .into();
    }

    // Parse the function item
    let input_fn = parse_macro_input!(item as ItemFn);

    // Handle async feature gate
    #[cfg(feature = "async")]
    let is_async_fn = input_fn.sig.asyncness.is_some();

    #[cfg(not(feature = "async"))]
    let is_async_fn = false;

    #[cfg(not(feature = "async"))]
    {
        if let Err(err) = reject_async(&input_fn.sig) {
            return err.into();
        }
    }

    // Check if return type is unit
    let is_unit_return = is_unit_return_type(&input_fn.sig);

    // Extract the previous type, parameter name, and resource injection params
    let (_, previous_type, resources) = match extract_args_info(&input_fn.sig) {
        Ok(info) => info,
        Err(e) => return e.to_compile_error().into(),
    };

    // Prepare building blocks
    let mut fn_attrs = input_fn.attrs.clone();
    fn_attrs.retain(|attr| !attr.path().is_ident("chain"));
    let vis = &input_fn.vis;
    let fn_name = &input_fn.sig.ident;
    let has_resources = !resources.is_empty();

    // Generate struct name
    let internal_name = format!(
        "__internal_chain_{}",
        just_fmt::snake_case!(fn_name.to_string())
    );
    let struct_name = Ident::new(&internal_name, fn_name.span());

    // Always use the default crate-defined program path
    let program_type = crate::default_program_path();

    // Generate the `proc` function for the Chain impl
    let proc_fn = generate_proc_fn(
        fn_name,
        has_resources,
        &resources,
        &program_type,
        &previous_type,
        is_async_fn,
        is_unit_return,
    );

    // Preserve the original function untouched
    // Note: do NOT add `#vis` here — `input_fn` (ItemFn) already contains its own visibility.
    let original_fn = quote! {
        #(#fn_attrs)*
        #input_fn
    };

    // Assemble the final output
    let previous_type_str = quote! { #previous_type };
    let expanded = generate_struct_and_impl(
        &fn_attrs,
        vis,
        &struct_name,
        &previous_type,
        &previous_type_str,
        &program_type,
        &proc_fn,
        &original_fn,
    );

    expanded.into()
}

/// Builds a match arm for chain mapping
pub(crate) fn build_chain_arm(
    struct_name: &Ident,
    previous_type: &TypePath,
) -> proc_macro2::TokenStream {
    let enum_variant = &previous_type.path.segments.last().unwrap().ident;
    quote! {
        #struct_name => #enum_variant,
    }
}

/// Builds a match arm for chain existence check
pub(crate) fn build_chain_exist_arm(previous_type: &TypePath) -> proc_macro2::TokenStream {
    let enum_variant = &previous_type.path.segments.last().unwrap().ident;
    quote! {
        Self::#enum_variant => true,
    }
}

pub(crate) fn register_chain(input: TokenStream) -> TokenStream {
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

    // Record the chain mapping: previous_type => struct_name
    let chain_entry = build_chain_arm(&struct_name, &previous_type);

    // Record the chain existence check
    let chain_exist_entry = build_chain_exist_arm(&previous_type);

    let chain_entry_str = chain_entry.to_string();
    let chain_exist_entry_str = chain_exist_entry.to_string();

    // Check for duplicate variant before inserting
    let variant_name = previous_type
        .path
        .segments
        .last()
        .unwrap()
        .ident
        .to_string();
    let value = crate::check_duplicate_variant(
        &crate::get_global_set(&crate::CHAINS).lock().unwrap(),
        &chain_entry_str,
        &variant_name,
        "chain",
        previous_type.span(),
    );
    if let Err(err) = value {
        return err.into();
    }

    crate::get_global_set(&crate::CHAINS)
        .lock()
        .unwrap()
        .insert(chain_entry_str);
    crate::get_global_set(&crate::CHAINS_EXIST)
        .lock()
        .unwrap()
        .insert(chain_exist_entry_str);

    quote! {}.into()
}
