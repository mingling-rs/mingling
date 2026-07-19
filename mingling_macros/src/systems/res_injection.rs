use quote::quote;
use syn::spanned::Spanned;
use syn::{FnArg, Ident, Pat, PatType, Signature, Type, TypePath};

/// Extracted information about a resource injection parameter
pub(crate) struct ResourceInjection {
    pub(crate) var_name: Ident,
    pub(crate) full_type: Type,
    pub(crate) inner_type: TypePath,
    pub(crate) is_ref: bool,
    pub(crate) is_mut: bool,
}

/// Extracts the previous type and parameter name from function arguments,
/// and collects resource injection parameters from the 2nd argument onward.
#[allow(clippy::too_many_lines)]
pub(crate) fn extract_args_info(
    sig: &Signature,
) -> syn::Result<(Pat, TypePath, Vec<ResourceInjection>)> {
    if sig.inputs.is_empty() {
        return Err(syn::Error::new(
            sig.span(),
            "Function must have at least one parameter",
        ));
    }

    // First parameter: required, the previous type (must be owned, not a reference)
    let first_arg = &sig.inputs[0];
    let (prev_param, previous_type) = match first_arg {
        FnArg::Typed(PatType { pat, ty, .. }) => {
            let param_pat = (**pat).clone();
            match &**ty {
                Type::Path(type_path) => (param_pat, type_path.clone()),
                Type::Reference(_) => {
                    return Err(syn::Error::new(
                        ty.span(),
                        "The first parameter (previous type) must be taken by move, \
                         not by reference. \
                         Use `prev: SomeEntry` instead of `prev: &SomeEntry`.",
                    ));
                }
                _ => {
                    return Err(syn::Error::new(
                        ty.span(),
                        "First parameter type must be a type path",
                    ));
                }
            }
        }
        FnArg::Receiver(_) => {
            return Err(syn::Error::new(
                first_arg.span(),
                "Function cannot have self parameter",
            ));
        }
    };

    // 2nd to Nth parameters: optional, for resource injection
    let mut resources = Vec::new();
    for arg in sig.inputs.iter().skip(1) {
        match arg {
            FnArg::Typed(PatType { pat, ty, .. }) => {
                // Extract the variable name – must be a simple identifier
                let var_name = match &**pat {
                    Pat::Ident(pat_ident) => pat_ident.ident.clone(),
                    _ => {
                        return Err(syn::Error::new(
                            pat.span(),
                            "Resource injection parameter must be a simple identifier (e.g., `age: &Age`)",
                        ));
                    }
                };

                let full_type = *(*ty).clone();

                // Try to extract inner type for reference patterns like `&Age` -> `Age`
                // and `&mut Age` -> `Age`
                let (inner_type, is_ref, is_mut) = match &full_type {
                    Type::Reference(ref_type) => match &*ref_type.elem {
                        Type::Path(type_path) => {
                            let is_mut = ref_type.mutability.is_some();
                            (type_path.clone(), true, is_mut)
                        }
                        _ => {
                            return Err(syn::Error::new(
                                ty.span(),
                                "Reference resource type must be a type path (e.g., `age: &Age`)",
                            ));
                        }
                    },
                    Type::Path(_) => {
                        return Err(syn::Error::new(
                            ty.span(),
                            "Resource injection parameter must be a reference (`&T` or `&mut T`), \
                             not an owned value. Use `age: &Age` instead of `age: Age`.",
                        ));
                    }
                    _ => {
                        return Err(syn::Error::new(
                            ty.span(),
                            "Resource injection type must be a type path or reference to one \
                             (e.g., `age: Age` or `age: &Age`)",
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

    Ok((prev_param, previous_type, resources))
}

/// Generates `let` binding statements for immutable resource injection parameters.
///
/// Each immutable reference parameter gets a `_binding` variable that holds the
/// `res_or_default` result, then a shadowing `let` that borrows from it via `.as_ref()`.
pub(crate) fn generate_immut_resource_bindings<'a>(
    resources: impl Iterator<Item = &'a ResourceInjection>,
    program_type: &proc_macro2::TokenStream,
) -> Vec<proc_macro2::TokenStream> {
    resources
        .filter(|r| !r.is_mut)
        .map(|res| {
            let var_binding_name = syn::Ident::new(
                &format!("__{}_binding", &res.var_name.to_string()),
                res.var_name.span(),
            );
            let var_name = &res.var_name;
            let full_type = &res.full_type;
            let inner_type = &res.inner_type;
            if res.is_ref {
                quote! {
                    let #var_binding_name = ::mingling::this::<#program_type>()
                        .res_or_default::<#inner_type>();
                    let #var_name: #full_type = #var_binding_name.as_ref();
                }
            } else {
                quote! {
                    let #var_name: #full_type = ::mingling::this::<#program_type>()
                        .res_or_default::<#full_type>();
                }
            }
        })
        .collect()
}

/// Generates a unique binding name for a mutable resource variable.
fn mut_res_binding_name(var_name: &Ident) -> Ident {
    syn::Ident::new(&format!("__{}_binding", var_name), var_name.span())
}

/// Wraps the function body in mutable resource closures (sync version).
///
/// For unit return types: uses `modify_res` (closure returns `()`).
/// For non-unit return types: uses `__modify_res_and_return_route` (closure returns `ChainProcess<C>`).
///
/// The innermost closure gets the original body, and each mutable parameter wraps
/// outward from last to first.
pub(crate) fn wrap_body_with_mut_resources(
    fn_body_stmts: &[syn::Stmt],
    mut_resources: &[&ResourceInjection],
    program_type: &proc_macro2::TokenStream,
    is_unit_return: bool,
) -> proc_macro2::TokenStream {
    let mut wrapped = quote! {
        #(#fn_body_stmts)*
    };

    for res in mut_resources {
        let var_name = &res.var_name;
        let inner_type = &res.inner_type;
        if is_unit_return {
            wrapped = quote! {
                ::mingling::this::<#program_type>().modify_res(|#var_name: &mut #inner_type| {
                    #wrapped
                })
            };
        } else {
            wrapped = quote! {
                ::mingling::this::<#program_type>().__modify_res_and_return_route(|#var_name: &mut #inner_type| {
                    #wrapped
                })
            };
        }
    }

    wrapped
}

/// Generates code for mutable resource injection in async `#[chain]` functions.
///
/// Instead of wrapping in a closure (which causes lifetime issues with async),
/// this generates a three-part pattern:
///
/// 1. **Extract** each mutable resource from the global store into an owned local.
/// 2. **Body block** — the user's body with `&mut` references scoped to a block.
/// 3. **Store back** each modified resource after the body block completes.
///
/// This avoids holding a `&mut` reference across `.await` points — the borrow
/// ends when the body block closes.
pub(crate) fn wrap_body_with_mut_resources_async(
    fn_body_stmts: &[syn::Stmt],
    mut_resources: &[&ResourceInjection],
    program_type: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    // 1. Generate extract statements and body-block `let` bindings
    let mut extract_stmts = Vec::new();
    let mut body_lets = Vec::new();

    for res in mut_resources {
        let var_name = &res.var_name;
        let inner_type = &res.inner_type;
        let binding_name = mut_res_binding_name(var_name);

        extract_stmts.push(quote! {
            let mut #binding_name = ::mingling::this::<#program_type>()
                .__extract_res_mut::<#inner_type>();
        });

        body_lets.push(quote! {
            let #var_name: &mut #inner_type = &mut #binding_name;
        });
    }

    // 2. Store-back statements (reverse order is fine, resources are independent)
    let mut store_stmts = Vec::new();
    for res in mut_resources {
        let var_name = &res.var_name;
        let inner_type = &res.inner_type;
        let binding_name = mut_res_binding_name(var_name);

        store_stmts.push(quote! {
            ::mingling::this::<#program_type>()
                .__store_res::<#inner_type>(#binding_name);
        });
    }

    quote! {
        #(#extract_stmts)*
        let __chain_result = {
            #(#body_lets)*
            #(#fn_body_stmts)*
        };
        #(#store_stmts)*
        __chain_result
    }
}
