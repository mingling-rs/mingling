#![allow(clippy::too_many_arguments)]

use crate::res_injection::{
    ResourceInjection, extract_args_info, generate_immut_resource_bindings,
    wrap_body_with_mut_resources,
};
use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::spanned::Spanned;
use syn::{Ident, ItemFn, Pat, ReturnType, Signature, Type, TypePath, parse_macro_input};

/// Validates that the return type of the function is `Next`.
/// Checks whether the return type is `()` (unit).
fn is_unit_return_type(sig: &Signature) -> bool {
    match &sig.output {
        ReturnType::Type(_, ty) => match &**ty {
            Type::Tuple(tuple) => tuple.elems.is_empty(),
            _ => false,
        },
        ReturnType::Default => true,
    }
}

fn validate_return_type(sig: &Signature) -> Result<(), proc_macro2::TokenStream> {
    // If return type is `()`, it's valid (no Next required)
    if is_unit_return_type(sig) {
        return Ok(());
    }

    match &sig.output {
        ReturnType::Type(_, ty) => match &**ty {
            Type::Path(type_path) => {
                let last_segment = type_path.path.segments.last().unwrap();
                if last_segment.ident != "Next" {
                    return Err(syn::Error::new(
                        ty.span(),
                        "Chain function must return `Next` or `()`",
                    )
                    .to_compile_error());
                }
            }
            _ => {
                return Err(syn::Error::new(
                    ty.span(),
                    "Chain function must return `Next` or `()`",
                )
                .to_compile_error());
            }
        },
        ReturnType::Default => {
            return Err(syn::Error::new(
                sig.span(),
                "Chain function must specify a return type (must be `Next` or `()`)",
            )
            .to_compile_error());
        }
    }
    Ok(())
}

/// Builds the `proc` function implementation that serves as the actual chain
/// entry point inside the generated `Chain` impl.
///
/// * Without resources: delegates directly to the original function.
/// * With resources: inlines the body and prepends resource bindings.
#[allow(unused_variables)]
fn generate_proc_fn(
    has_resources: bool,
    resources: &[ResourceInjection],
    program_type: &proc_macro2::TokenStream,
    previous_type: &TypePath,
    prev_param: &Pat,
    fn_name: &Ident,
    fn_body_stmts: &[syn::Stmt],
    is_async_fn: bool,
    is_unit_return: bool,
) -> proc_macro2::TokenStream {
    let immut_resource_stmts = generate_immut_resource_bindings(resources.iter(), program_type);
    let mut_resources: Vec<_> = resources.iter().filter(|r| r.is_mut).collect();

    let body_stmts: &[syn::Stmt] = if is_unit_return && has_resources {
        let mut stmts = fn_body_stmts.to_vec();
        stmts.push(syn::Stmt::Expr(
            syn::parse_quote! {
                <crate::ResultEmpty as ::mingling::Groupped::<crate::ThisProgram>>
                ::to_chain(crate::ResultEmpty::new(()))
            },
            None,
        ));
        // Box::leak to get a &'static [syn::Stmt]
        Box::leak(Box::new(stmts))
    } else {
        fn_body_stmts
    };

    let wrapped_body = wrap_body_with_mut_resources(body_stmts, &mut_resources, program_type);

    // When the function returns `()`, wrap the result with ResultEmpty
    let call_or_wrapped = if is_unit_return {
        if has_resources {
            quote! {
                #(#immut_resource_stmts)*
                #wrapped_body
            }
        } else {
            let call = if is_async_fn {
                quote! { #fn_name(#prev_param).await; }
            } else {
                quote! { #fn_name(#prev_param); }
            };
            quote! {
                #call
                <crate::ResultEmpty as ::mingling::Groupped::<crate::ThisProgram>>
                ::to_chain(crate::ResultEmpty::new(()))
            }
        }
    } else if has_resources {
        quote! {
            #(#immut_resource_stmts)*
            #wrapped_body
        }
    } else {
        let call = if is_async_fn {
            quote! { #fn_name(#prev_param).await.into() }
        } else {
            quote! { #fn_name(#prev_param).into() }
        };
        quote! {
            #call
        }
    };

    #[cfg(feature = "async")]
    {
        quote! {
            async fn proc(#prev_param: #previous_type) -> ::mingling::ChainProcess<#program_type> {
                #call_or_wrapped
            }
        }
    }

    #[cfg(not(feature = "async"))]
    {
        quote! {
            fn proc(#prev_param: #previous_type) -> ::mingling::ChainProcess<#program_type> {
                #call_or_wrapped
            }
        }
    }
}

/// Generates the original function signature (kept for backwards compatibility /
/// internal use), with its return type changed to `impl Into<ChainProcess<..>>`.
#[allow(unused_variables)]
fn generate_original_fn(
    fn_attrs: &[syn::Attribute],
    vis: &syn::Visibility,
    fn_name: &Ident,
    inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
    fn_body: &syn::Block,
    is_async_fn: bool,
    program_type: &proc_macro2::TokenStream,
    is_unit_return: bool,
) -> proc_macro2::TokenStream {
    // Both unit and Next return types need to produce `impl Into<ChainProcess<ProgramType>>`
    let return_type = quote! { impl Into<::mingling::ChainProcess<#program_type>> };

    let body = if is_unit_return {
        quote! {
            {
                #fn_body
                <crate::ResultEmpty as ::mingling::Groupped::<crate::ThisProgram>>::to_chain(crate::ResultEmpty::new(()))
            }
        }
    } else {
        quote! {
            {
                let _: crate::Next;
                let _: Next;
                #fn_body
            }
        }
    };

    #[cfg(feature = "async")]
    {
        let async_kw = if is_async_fn {
            quote! { async }
        } else {
            quote! {}
        };
        quote! {
            #(#fn_attrs)*
            #vis #async_kw fn #fn_name(#inputs) -> #return_type {
                #body
            }
        }
    }

    #[cfg(not(feature = "async"))]
    {
        quote! {
            #(#fn_attrs)*
            #vis fn #fn_name(#inputs) -> #return_type {
                #body
            }
        }
    }
}

/// Assembles the final expanded output: hidden struct, `register_chain!` invocation,
/// `Chain` impl with the `proc` method, and the original function.
fn generate_struct_and_impl(
    fn_attrs: &[syn::Attribute],
    vis: &syn::Visibility,
    struct_name: &Ident,
    previous_type: &TypePath,
    previous_type_str: &proc_macro2::TokenStream,
    program_type: &proc_macro2::TokenStream,
    proc_fn: &proc_macro2::TokenStream,
    origin_proc_fn: &proc_macro2::TokenStream,
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

        // Keep the original function for internal use
        #origin_proc_fn
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

/// Ensures no `&mut` resource injection is used in async functions.
#[cfg(feature = "async")]
fn reject_mut_in_async(resources: &[ResourceInjection]) -> Result<(), proc_macro2::TokenStream> {
    if let Some(mut_res) = resources.iter().find(|r| r.is_mut) {
        return Err(syn::Error::new(
            mut_res.var_name.span(),
            "Cannot use `&mut` resource injection in async chain function.",
        )
        .to_compile_error());
    }
    Ok(())
}

pub fn chain_attr(attr: TokenStream, item: TokenStream) -> TokenStream {
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
    {
        if let Err(err) = reject_async(&input_fn.sig) {
            return err.into();
        }
    }

    // Check if return type is unit
    let is_unit_return = is_unit_return_type(&input_fn.sig);

    // Validate return type
    if let Err(err) = validate_return_type(&input_fn.sig) {
        return err.into();
    }

    // Extract the previous type, parameter name, and resource injection params
    let (prev_param, previous_type, resources) = match extract_args_info(&input_fn.sig) {
        Ok(info) => info,
        Err(e) => return e.to_compile_error().into(),
    };

    // Reject `&mut` in async chains
    #[cfg(feature = "async")]
    if is_async_fn {
        if let Err(err) = reject_mut_in_async(&resources) {
            return err.into();
        }
    }

    // Prepare building blocks
    let sig = &input_fn.sig;
    let inputs = &sig.inputs;
    let fn_body = &input_fn.block;
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

    // Generate the `proc` function
    let proc_fn = generate_proc_fn(
        has_resources,
        &resources,
        &program_type,
        &previous_type,
        &prev_param,
        fn_name,
        &fn_body.stmts,
        #[cfg(feature = "async")]
        is_async_fn,
        #[cfg(not(feature = "async"))]
        false,
        is_unit_return,
    );

    // Generate the original function
    let origin_proc_fn = generate_original_fn(
        &fn_attrs,
        vis,
        fn_name,
        inputs,
        fn_body,
        #[cfg(feature = "async")]
        is_async_fn,
        #[cfg(not(feature = "async"))]
        false,
        &program_type,
        is_unit_return,
    );

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
        &origin_proc_fn,
    );

    expanded.into()
}

/// Builds a match arm for chain mapping
pub fn build_chain_arm(struct_name: &Ident, previous_type: &TypePath) -> proc_macro2::TokenStream {
    let enum_variant = &previous_type.path.segments.last().unwrap().ident;
    quote! {
        #struct_name => #enum_variant,
    }
}

/// Builds a match arm for chain existence check
pub fn build_chain_exist_arm(previous_type: &TypePath) -> proc_macro2::TokenStream {
    let enum_variant = &previous_type.path.segments.last().unwrap().ident;
    quote! {
        Self::#enum_variant => true,
    }
}

pub fn register_chain(input: TokenStream) -> TokenStream {
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

    let mut chains = crate::get_global_set(&crate::CHAINS).lock().unwrap();
    let mut chain_exist = crate::get_global_set(&crate::CHAINS_EXIST).lock().unwrap();

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
    if let Err(err) = crate::check_duplicate_variant(
        &chains,
        &chain_entry_str,
        &variant_name,
        "chain",
        previous_type.span(),
    ) {
        return err.into();
    }

    chains.insert(chain_entry_str);
    chain_exist.insert(chain_exist_entry_str);

    quote! {}.into()
}
