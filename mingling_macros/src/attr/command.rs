use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::parse_macro_input;
use syn::spanned::Spanned;
use syn::token::Comma;
use syn::{FnArg, Ident, ItemFn, LitStr, PatType, Token, Type};

/// Parsed arguments for `#[command(...)]`.
///
/// Supports:
/// - `node = "dot.separated.path"` — explicit command path
/// - `name = CMDName` — explicit CMD struct name
/// - `entry = EntryName` — explicit Entry struct name
/// - bare paths like `routeify`, `::mingling::macros::routeify` — extension attrs for the original fn
struct CommandArgs {
    node: Option<LitStr>,
    name: Option<Ident>,
    entry: Option<Ident>,
    exts: Vec<syn::Path>,
}

impl Parse for CommandArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut node = None;
        let mut name = None;
        let mut entry = None;
        let mut exts = Vec::new();

        while !input.is_empty() {
            if input.peek(Ident) && input.peek2(Token![=]) {
                // key = value pair
                let key: Ident = input.parse()?;
                input.parse::<Token![=]>()?;

                if key == "node" {
                    if node.is_some() {
                        return Err(input.error("duplicate `node` argument"));
                    }
                    node = Some(input.parse()?);
                } else if key == "name" {
                    if name.is_some() {
                        return Err(input.error("duplicate `name` argument"));
                    }
                    name = Some(input.parse()?);
                } else if key == "entry" {
                    if entry.is_some() {
                        return Err(input.error("duplicate `entry` argument"));
                    }
                    entry = Some(input.parse()?);
                } else {
                    return Err(input.error(format!(
                        "unknown key `{}`; expected `node`, `name`, or `entry`",
                        key
                    )));
                }
            } else {
                // Extension path (e.g. `routeify` or `::mingling::macros::routeify`)
                let ext: syn::Path = input.parse()?;
                exts.push(ext);
            }

            // Skip optional trailing comma
            if input.peek(Comma) {
                let _ = input.parse::<Comma>();
            }
        }

        Ok(CommandArgs {
            node,
            name,
            entry,
            exts,
        })
    }
}

/// Returns the default node path as a dot-separated string, derived from the function name.
///
/// Example: `greet_someone` -> `"greet.someone"`, `greet` -> `"greet"`
fn default_node_from_fn(fn_name: &Ident) -> String {
    just_fmt::dot_case!(fn_name.to_string())
}

/// Checks basic function constraints for `#[command]`.
/// Returns `Err(compile_error token stream)` on failure.
fn validate_function(f: &ItemFn) -> Result<(), TokenStream2> {
    if f.sig
        .inputs
        .iter()
        .any(|arg| matches!(arg, FnArg::Receiver(_)))
    {
        return Err(syn::Error::new(
            f.sig.span(),
            "#[command] function cannot have a `self` parameter",
        )
        .to_compile_error());
    }

    Ok(())
}

/// Returns `(wrapper_async_token, await_call)` for the chain wrapper.
///
/// When the original function is async and the `async` feature is enabled,
/// the wrapper needs to be `async` and call `.await` on the original function.
/// Without the feature, async functions are rejected.
fn handle_async(f: &ItemFn) -> Result<(TokenStream2, TokenStream2), TokenStream2> {
    let is_async = f.sig.asyncness.is_some();

    #[cfg(not(feature = "async"))]
    if is_async {
        return Err(syn::Error::new(
            f.sig.span(),
            "#[command] function cannot be async when the `async` feature is disabled",
        )
        .to_compile_error());
    }

    let wrapper_async = is_async.then_some(quote! { async }).unwrap_or_default();
    let await_call = is_async.then_some(quote! { .await }).unwrap_or_default();
    Ok((wrapper_async, await_call))
}

/// All resolved identifiers derived from `#[command]` arguments + function name.
struct ResolvedNames {
    /// `node_str` as a string literal token
    node_lit: LitStr,
    /// Whether the user supplied any explicit override (node/name/entry)
    has_overrides: bool,
    /// CMD struct name (e.g. `CMDGreet`)
    cmd_name: Ident,
    /// Entry struct name (e.g. `EntryGreet`)
    entry_type: Ident,
    /// Chain wrapper function name (e.g. `__command_chain_greet`)
    chain_fn_name: Ident,
}

/// Resolves `node`, `cmd_name`, `entry_type`, and `chain_fn_name` from
/// the attribute args and the original function name.
fn resolve_names(fn_name: &Ident, args: &CommandArgs) -> ResolvedNames {
    let fn_name_str = fn_name.to_string();

    let node_str = match &args.node {
        Some(lit) => lit.value(),
        None => default_node_from_fn(fn_name),
    };
    let node_lit = syn::LitStr::new(&node_str, fn_name.span());

    let has_overrides = args.node.is_some() || args.name.is_some() || args.entry.is_some();

    let cmd_name = args.name.clone().unwrap_or_else(|| {
        let pascal = just_fmt::pascal_case!(&node_str);
        Ident::new(&format!("CMD{pascal}"), fn_name.span())
    });

    let entry_type = args.entry.clone().unwrap_or_else(|| {
        let pascal = just_fmt::pascal_case!(&node_str);
        Ident::new(&format!("Entry{pascal}"), fn_name.span())
    });

    let chain_fn_name = Ident::new(&format!("__command_chain_{}", fn_name_str), fn_name.span());

    ResolvedNames {
        node_lit,
        has_overrides,
        cmd_name,
        entry_type,
        chain_fn_name,
    }
}

/// Converts extension paths into `#[ext]` attribute token streams.
fn build_ext_attrs(exts: &[syn::Path]) -> Vec<TokenStream2> {
    exts.iter().map(|ext| quote! { #[#ext] }).collect()
}

/// Returns `true` if the function has a first non-reference (owned) parameter that
/// serves as the "args" input.
fn has_args_param(sig: &syn::Signature) -> bool {
    sig.inputs.first().is_some_and(|arg| {
        if let FnArg::Typed(pat_type) = arg {
            !matches!(&*pat_type.ty, Type::Reference(_))
        } else {
            false
        }
    })
}

/// Builds the wrapper function's parameter list.
///
/// - If the function has an "args" param (first non-reference): replaces its type
///   with the entry type (e.g. `Vec<String>` → `EntryGreet`).
/// - If not (no params, or first param is a reference): inserts a new entry param
///   at the front to satisfy `#[chain]`'s requirement for an owned first parameter.
fn build_wrapper_params(
    sig: &syn::Signature,
    entry_type: &Ident,
) -> syn::punctuated::Punctuated<FnArg, syn::token::Comma> {
    if has_args_param(sig) {
        // First param is owned (args) -> replace its type with entry type
        let mut params = sig.inputs.clone();
        if let Some(FnArg::Typed(first)) = params.first_mut() {
            *first.ty = Type::Path(syn::TypePath {
                qself: None,
                path: syn::Path::from(entry_type.clone()),
            });
        }
        params
    } else {
        // No args param -> insert an entry param at the front, keep rest as-is
        let mut params = syn::punctuated::Punctuated::new();
        let entry_param: FnArg = syn::parse_quote! { _args: #entry_type };
        params.push(entry_param);
        for arg in sig.inputs.iter() {
            params.push(arg.clone());
        }
        params
    }
}

/// Builds call arguments for the original function call inside the wrapper.
///
/// - If the function has an "args" param (first non-reference): first arg gets
///   `.into()` (converts `EntryGreet` → `Vec<String>`), rest pass through as-is.
/// - If not (no args): all params are resources — pass none, return empty.
fn build_call_args(sig: &syn::Signature) -> Vec<TokenStream2> {
    let has_args = has_args_param(sig);
    sig.inputs
        .iter()
        .enumerate()
        .map(|(i, arg)| {
            let pat = match arg {
                FnArg::Typed(PatType { pat, .. }) => quote! { #pat },
                FnArg::Receiver(_) => unreachable!(),
            };
            if has_args && i == 0 {
                quote! { #pat.into() }
            } else {
                pat
            }
        })
        .collect()
}

/// Generates the `dispatcher!(...)` call.
///
/// - No overrides → abbreviated form: `dispatcher!("node")`
/// - Any override → explicit form: `dispatcher!("node", CMDName => EntryName)`
fn build_dispatcher_invoke(names: &ResolvedNames) -> TokenStream2 {
    let node_lit = &names.node_lit;
    if names.has_overrides {
        let cmd_name = &names.cmd_name;
        let entry_type = &names.entry_type;
        quote! { ::mingling::macros::dispatcher!(#node_lit, #cmd_name => #entry_type); }
    } else {
        quote! { ::mingling::macros::dispatcher!(#node_lit); }
    }
}

pub(crate) fn command_attr(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    // validation
    if let Err(err) = validate_function(&input_fn) {
        return err.into();
    }

    // parse attribute
    let args: CommandArgs = if attr.is_empty() {
        CommandArgs {
            node: None,
            name: None,
            entry: None,
            exts: Vec::new(),
        }
    } else {
        parse_macro_input!(attr as CommandArgs)
    };

    // async handling
    let (wrapper_async, await_call) = match handle_async(&input_fn) {
        Ok(pair) => pair,
        Err(err) => return err.into(),
    };

    // resolve node / name / entry
    let fn_name = &input_fn.sig.ident;
    let names = resolve_names(fn_name, &args);
    let chain_fn_name = &names.chain_fn_name;

    // build extension attributes (applied to the ORIGINAL function)
    let ext_attrs = build_ext_attrs(&args.exts);

    // build wrapper
    let wrapper_params = build_wrapper_params(&input_fn.sig, &names.entry_type);
    let call_args = build_call_args(&input_fn.sig);

    // build dispatcher invocation
    let dispatcher_invoke = build_dispatcher_invoke(&names);

    // preserve original function
    let mut fn_attrs = input_fn.attrs.clone();
    fn_attrs.retain(|attr| !attr.path().is_ident("command"));

    let vis = &input_fn.vis;
    let asyncness = input_fn.sig.asyncness;
    let generics = &input_fn.sig.generics;
    let orig_params = &input_fn.sig.inputs;
    let orig_return = &input_fn.sig.output;
    let fn_body = &input_fn.block;

    // compute names for the re‑export module and internal structs
    let fn_name_s = fn_name.to_string();
    let mod_name = Ident::new(&format!("__command_{}_module", &fn_name_s), fn_name.span());
    let wrapper_full = format!("__command_chain_{}", &fn_name_s);
    let snaked_wrapper = just_fmt::snake_case!(wrapper_full);
    let chain_internal = Ident::new(
        &format!("__internal_chain_{}", snaked_wrapper),
        fn_name.span(),
    );

    // dispatcher internal static (only exists with dispatch_tree feature)
    #[cfg(feature = "dispatch_tree")]
    let snaked_node = just_fmt::snake_case!(names.node_lit.value());
    #[cfg(feature = "dispatch_tree")]
    let dispatcher_internal = {
        let ident = Ident::new(
            &format!("__internal_dispatcher_{}", snaked_node),
            fn_name.span(),
        );
        quote! { #vis use super::#ident; }
    };
    #[cfg(not(feature = "dispatch_tree"))]
    let dispatcher_internal = quote! {};

    let cmd_name = &names.cmd_name;
    let entry_type = &names.entry_type;

    // assemble output
    let expanded = quote! {
        #dispatcher_invoke

        #[::mingling::macros::chain]
        #vis #wrapper_async fn #chain_fn_name #generics(#wrapper_params) -> crate::Next {
            #fn_name(#(#call_args),*)#await_call.into()
        }

        #(#fn_attrs)*
        #(#ext_attrs)*
        #vis #asyncness fn #fn_name #generics(#orig_params) #orig_return #fn_body

        // hidden module gathering all generated types for pathf / external access
        #[doc(hidden)]
        #vis mod #mod_name {
            #vis use super::#cmd_name;
            #vis use super::#entry_type;
            #vis use super::#chain_internal;
            #dispatcher_internal
        }
    };

    expanded.into()
}
