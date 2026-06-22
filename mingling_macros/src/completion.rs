use crate::res_injection::{ResourceInjection, generate_immut_resource_bindings};
use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{FnArg, Ident, ItemFn, Pat, PatType, Type, TypePath, parse_macro_input};

#[cfg(feature = "comp")]
pub fn completion_attr(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the attribute arguments such as HelloEntry or crate::EntryFine from #[completion(crate::EntryFine)]
    use crate::get_global_set;
    let previous_type_path: TypePath = if attr.is_empty() {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            "completion attribute requires a previous type argument, e.g. #[completion(HelloEntry)]",
        )
        .to_compile_error()
        .into();
    } else {
        parse_macro_input!(attr as TypePath)
    };
    let previous_type_ident = &previous_type_path.path.segments.last().unwrap().ident;

    // Parse the function item
    let input_fn = parse_macro_input!(item as ItemFn);

    // Validate the function is not async
    if input_fn.sig.asyncness.is_some() {
        return syn::Error::new(input_fn.sig.span(), "Completion function cannot be async")
            .to_compile_error()
            .into();
    }

    // Get the function signature parts
    let sig = &input_fn.sig;
    let inputs = &sig.inputs;
    let output = &sig.output;

    // Must have at least one parameter ctx
    if inputs.is_empty() {
        return syn::Error::new(
            inputs.span(),
            "Completion function must have at least one parameter: `ctx: &ShellContext`",
        )
        .to_compile_error()
        .into();
    }

    // Extract the first param pattern and type for the ctx parameter
    let first_arg = &inputs[0];
    let (ctx_pat, _ctx_type) = match first_arg {
        FnArg::Typed(PatType { pat, ty, .. }) => {
            let param_pat = (**pat).clone();
            (param_pat, (**ty).clone())
        }
        FnArg::Receiver(_) => {
            return syn::Error::new(
                first_arg.span(),
                "Completion function cannot have self parameter",
            )
            .to_compile_error()
            .into();
        }
    };

    // Extract resources from params 2 through N, skipping ctx
    let resources = match extract_resources_from_args(sig, 1) {
        Ok(r) => r,
        Err(e) => return e.to_compile_error().into(),
    };

    // Get the function body
    let fn_body = &input_fn.block;
    let fn_body_stmts = &fn_body.stmts;

    // Get function attributes excluding the completion attribute
    let mut fn_attrs = input_fn.attrs.clone();
    fn_attrs.retain(|attr| !attr.path().is_ident("completion"));

    // Get function visibility
    let vis = &input_fn.vis;

    // Get function name
    let fn_name = &sig.ident;

    // Generate internal name from function name using snake_case
    let internal_name = format!(
        "__internal_completion_{}",
        just_fmt::snake_case!(fn_name.to_string())
    );
    let struct_name = Ident::new(&internal_name, fn_name.span());

    let program_type = crate::default_program_path();
    let has_resources = !resources.is_empty();
    let mut_resources: Vec<_> = resources.iter().filter(|r| r.is_mut).collect();

    // Generate immutable resource bindings
    let immut_resource_stmts = generate_immut_resource_bindings(resources.iter(), &program_type);

    // Build the comp method body with resource injection
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

    let comp_body = if has_resources {
        quote! {
            #(#immut_resource_stmts)*
            #wrapped_body
        }
    } else {
        quote! { #(#fn_body_stmts)* }
    };

    // Generate the struct and implementation
    // The `comp` trait method only takes `ctx` as the first parameter; resources are injected internally

    let expanded = quote! {
        #(#fn_attrs)*
        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        #vis struct #struct_name;

        impl ::mingling::Completion for #struct_name {
            type Previous = #previous_type_path;

            fn comp(#ctx_pat: &::mingling::ShellContext) #output {
                #comp_body
            }
        }

        // Keep the original function for internal use
        #(#fn_attrs)*
        #vis fn #fn_name(#inputs) #output {
            #fn_body
        }
    };

    let completion_entry = quote! {
        Self::#previous_type_ident => <#struct_name as ::mingling::Completion>::comp(ctx),
    };

    let mut completions = get_global_set(&crate::COMPLETIONS).lock().unwrap();
    let completion_str = completion_entry.to_string();

    // Check for duplicate variant before inserting
    let variant_name = previous_type_ident.to_string();
    if let Err(err) = crate::check_duplicate_variant(
        &completions,
        &completion_str,
        &variant_name,
        "completion",
        previous_type_path.span(),
    ) {
        return err.into();
    }

    completions.insert(completion_str);

    expanded.into()
}

/// Extract resource injection parameters from function arguments (skipping the first N params).
fn extract_resources_from_args(
    sig: &syn::Signature,
    skip: usize,
) -> syn::Result<Vec<ResourceInjection>> {
    let mut resources = Vec::new();
    for arg in sig.inputs.iter().skip(skip) {
        match arg {
            FnArg::Typed(PatType { pat, ty, .. }) => {
                let var_name = match &**pat {
                    Pat::Ident(pat_ident) => pat_ident.ident.clone(),
                    _ => {
                        return Err(syn::Error::new(
                            pat.span(),
                            "Resource injection parameter must be a simple identifier",
                        ));
                    }
                };

                let full_type = *(*ty).clone();

                let (inner_type, is_ref, is_mut) = match &full_type {
                    Type::Reference(ref_type) => match &*ref_type.elem {
                        Type::Path(type_path) => {
                            let is_mut = ref_type.mutability.is_some();
                            (type_path.clone(), true, is_mut)
                        }
                        _ => {
                            return Err(syn::Error::new(
                                ty.span(),
                                "Reference resource type must be a type path",
                            ));
                        }
                    },
                    Type::Path(_) => {
                        return Err(syn::Error::new(
                            ty.span(),
                            "Resource injection parameter must be a reference (`&T` or `&mut T`)",
                        ));
                    }
                    _ => {
                        return Err(syn::Error::new(
                            ty.span(),
                            "Resource injection type must be a type path or reference",
                        ));
                    }
                };

                resources.push(ResourceInjection {
                    var_name,
                    full_type,
                    inner_type,
                    is_ref,
                    is_mut,
                });
            }
            FnArg::Receiver(_) => {
                return Err(syn::Error::new(
                    arg.span(),
                    "Resource injection parameter cannot be self",
                ));
            }
        }
    }
    Ok(resources)
}
