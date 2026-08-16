// Doc Not Optimize
use crate::res_injection::{ResourceInjection, generate_immut_resource_bindings};
use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{FnArg, Ident, ItemFn, Pat, PatType, Type, TypePath, parse_macro_input};

#[cfg(feature = "comp")]
#[allow(clippy::too_many_lines)]
pub(crate) fn completion_attr(attr: TokenStream, item: TokenStream) -> TokenStream {
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

    let input_fn = parse_macro_input!(item as ItemFn);

    if input_fn.sig.asyncness.is_some() {
        return syn::Error::new(input_fn.sig.span(), "Completion function cannot be async")
            .to_compile_error()
            .into();
    }

    let sig = &input_fn.sig;
    let inputs = &sig.inputs;
    let output = &sig.output;

    // The first parameter (if any) is the completion context. It may be
    // `&ShellContext`, an owned `ShellContext`, or any other type that
    // implements `From<&ShellContext>`. With no parameters, the completion
    // function simply ignores the shell context.
    let ctx_ty: Option<Type> = match inputs.first() {
        None => None,
        Some(FnArg::Typed(PatType { ty, .. })) => Some((**ty).clone()),
        Some(FnArg::Receiver(_)) => {
            return syn::Error::new(
                inputs.span(),
                "Completion function cannot have self parameter",
            )
            .to_compile_error()
            .into();
        }
    };

    // Resource injection starts after the context parameter.
    let resource_skip = usize::from(ctx_ty.is_some());
    let resources = match extract_resources_from_args(sig, resource_skip) {
        Ok(r) => r,
        Err(e) => return e.to_compile_error().into(),
    };
    if ctx_ty.is_none() && !resources.is_empty() {
        return syn::Error::new(
            inputs.span(),
            "A completion function without a context parameter cannot inject resources",
        )
        .to_compile_error()
        .into();
    }

    // Bind the shell context to the declared parameter type (identity `From`
    // covers `&ShellContext` itself).
    let (ctx_bind_stmt, ctx_call_arg) = ctx_ty.as_ref().map_or_else(
        || (quote! { let _ = ctx; }, quote! {}),
        |ty| {
            (
                quote! {
                    let __ctx: #ty =
                        <#ty as ::std::convert::From<&::mingling::ShellContext>>::from(ctx);
                },
                quote! { __ctx },
            )
        },
    );

    let fn_body = &input_fn.block;

    let mut fn_attrs = input_fn.attrs.clone();
    fn_attrs.retain(|attr| !attr.path().is_ident("completion"));

    let vis = &input_fn.vis;
    let fn_name = &sig.ident;

    let internal_name = format!(
        "__internal_completion_{}",
        just_fmt::snake_case!(fn_name.to_string())
    );
    let struct_name = Ident::new(&internal_name, fn_name.span());

    let program_type = crate::default_program_path();
    let has_resources = !resources.is_empty();
    let mut_resources: Vec<_> = resources.iter().filter(|r| r.is_mut).collect();

    let immut_resource_stmts = generate_immut_resource_bindings(resources.iter(), &program_type);

    let resource_args: Vec<_> = resources
        .iter()
        .map(|res| {
            let var_name = &res.var_name;
            quote! { #var_name }
        })
        .collect();

    let fn_call = if has_resources {
        quote! { #fn_name(#ctx_call_arg, #(#resource_args),*) }
    } else if ctx_ty.is_some() {
        quote! { #fn_name(#ctx_call_arg) }
    } else {
        quote! { #fn_name() }
    };

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

    let comp_body = if has_resources {
        quote! {
            #(#immut_resource_stmts)*
            #inner_call
        }
    } else {
        quote! { #inner_call }
    };

    // A `()` return (or no return type) means "no suggestions": map it to an
    // empty `Suggest` instead of requiring `Into<Suggest>`.
    let returns_unit = match &sig.output {
        syn::ReturnType::Default => true,
        syn::ReturnType::Type(_, ty) => {
            matches!(ty.as_ref(), syn::Type::Tuple(t) if t.elems.is_empty())
        }
    };
    let return_stmt = if returns_unit {
        quote! {
            { #comp_body };
            ::mingling::Suggest::new()
        }
    } else {
        quote! {
            let __completion_result = { #comp_body };
            ::std::convert::Into::into(__completion_result)
        }
    };

    let expanded: proc_macro2::TokenStream = quote! {
        #(#fn_attrs)*
        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        #vis struct #struct_name;

        impl ::mingling::Completion for #struct_name {
            type Previous = #previous_type_path;

            fn comp(ctx: &::mingling::ShellContext) -> ::mingling::Suggest {
                #ctx_bind_stmt
                #return_stmt
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

    let completion_str = completion_entry.to_string();
    let variant_name = previous_type_ident.to_string();
    let span = previous_type_path.span();

    let mut completions = get_global_set(&crate::COMPLETIONS).lock().unwrap();
    if let Err(err) = crate::check_duplicate_variant(
        &completions,
        &completion_str,
        &variant_name,
        "completion",
        span,
    ) {
        return err.into();
    }
    completions.insert(completion_str);
    drop(completions);

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
